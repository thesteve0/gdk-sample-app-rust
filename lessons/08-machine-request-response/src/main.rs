use std::{
    collections::HashMap,
    env, fs,
    io::{self, IsTerminal, Write},
    path::PathBuf,
    sync::Arc,
};

use async_trait::async_trait;
use goose_agent::{
    events::AgentEvent,
    inference::{InferenceEffect, InferenceRunner},
    machine::{EffectHandler, MachineSession, SessionLoader, StateMachine, Step},
    operation::{Emitter, MachineEffect},
};
use goose_providers::{
    base::Provider,
    conversation::{message::Message, token_usage::ProviderUsage, Conversation},
    declarative::{from_json, EnvKeyResolver},
    model::ModelConfig,
};
use serde_json::Value;
use tokio::sync::{mpsc, Mutex};
use tokio_util::sync::CancellationToken;

const DEFAULT_PROVIDER_CONFIG: &str = "custom_aa_llama_qwen3_6-35b.json";
const SESSION_ID: &str = "lesson-08";
const USER_PROMPT: &str = "What is the capital of France?";

#[tokio::main]
async fn main() {
    if let Err(error) = run_application().await {
        print_error(&format!("Error: {:?}", error));
        std::process::exit(1);
    }
}

async fn run_application() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let config_path = provider_config_path()?;
    let provider_json = fs::read_to_string(config_path)?;
    let provider_config: Value = serde_json::from_str(&provider_json)?;
    let boxed_provider = from_json(&provider_json, None, EnvKeyResolver {})?;
    let provider: Arc<dyn Provider> = Arc::from(boxed_provider);
    let model_name = first_configured_model(&provider_config)?;
    let model = ModelConfig::new(model_name);

    // The application owns recorded state; this store lasts only in this process.
    let runtime = ChatRuntime::seeded();
    // Assemble one inference step; the engine owns checking and re-evaluation.
    let runner = InferenceRunner::new(provider, model);
    let steps: Vec<Step<'_, ChatSession, ChatEffect>> = vec![Step::Inference(Arc::new(runner))];
    // The GDK-provided InferenceRunner hard-codes its Operation name as "llm".
    // Show the configured order before moving the steps into the machine.
    print_heading("State-machine operations (in order):");
    for (index, step) in steps.iter().enumerate() {
        let name = match step {
            Step::Operation(operation) => operation.name(),
            Step::Inference(inference) => inference.name(),
        };
        println!("  {}. {}", index + 1, name);
    }
    let cancel = CancellationToken::new();
    let machine = StateMachine::new(steps, cancel.clone());
    let before = runtime.load(SESSION_ID).await?;
    display_session("BEFORE", &before);
    print_heading("application → provider:");
    let (event_tx, event_rx) = mpsc::channel::<AgentEvent>(32);
    let emit = Emitter::new(event_tx, cancel);

    // Poll both paths together: display Events while the machine is running.
    let run = async {
        // In production, consider adding a safeguard to prevent a runaway loop.
        let run_result = machine.run(&runtime, SESSION_ID, &emit).await;
        // Close the only sender even on error, so the consumer drains and finishes.
        drop(emit);
        run_result
    };
    let display = display_events(event_rx);
    let (run_result, display_result) = tokio::join!(run, display);

    // Both paths have finished before the final recorded-state view is printed.
    let final_session = match run_result {
        Ok(session) => session,
        Err(error) => return Err(error),
    };
    let event_summary = match display_result {
        Ok(summary) => summary,
        Err(error) => return Err(error.into()),
    };
    display_session("AFTER", &final_session);
    validate_exchange(&final_session, &event_summary)?;
    Ok(())
}

// A loaded Session is a snapshot, not a list of completed Operations.
#[derive(Clone)]
struct ChatSession {
    id: String,
    conversation: Conversation,
    usage: Vec<ProviderUsage>,
}

impl MachineSession for ChatSession {
    fn id(&self) -> &str {
        &self.id
    }

    fn conversation(&self) -> Option<&Conversation> {
        Some(&self.conversation)
    }
}

// The runtime supplies the engine's loading and saving boundaries.
struct ChatRuntime {
    sessions: Mutex<HashMap<String, ChatSession>>,
}

impl ChatRuntime {
    fn seeded() -> Self {
        let mut conversation = Conversation::empty();
        conversation.push(Message::user().with_text(USER_PROMPT));
        let session = ChatSession {
            id: SESSION_ID.to_string(),
            conversation,
            usage: Vec::new(),
        };
        let mut sessions = HashMap::new();
        sessions.insert(SESSION_ID.to_string(), session);
        Self {
            sessions: Mutex::new(sessions),
        }
    }
}

#[async_trait]
impl SessionLoader<ChatSession> for ChatRuntime {
    async fn load(&self, session_id: &str) -> anyhow::Result<ChatSession> {
        let sessions = self.sessions.lock().await;
        let session = match sessions.get(session_id) {
            Some(session) => session,
            None => {
                let failure = format!("no Session '{}' exists in the store", session_id);
                return Err(anyhow::anyhow!(failure));
            }
        };
        Ok(session.clone())
    }
}

#[async_trait]
impl EffectHandler<ChatSession, ChatEffect> for ChatRuntime {
    async fn apply_effects(
        &self,
        session: &ChatSession,
        effects: &mut [ChatEffect],
        _emit: &Emitter,
    ) -> anyhow::Result<()> {
        let mut sessions = self.sessions.lock().await;
        let stored = match sessions.get_mut(session.id()) {
            Some(stored) => stored,
            None => {
                let failure = format!("no Session '{}' exists in the store", session.id());
                return Err(anyhow::anyhow!(failure));
            }
        };
        for effect in effects {
            match effect {
                ChatEffect::AppendMessage(message) => stored.conversation.push(message.clone()),
                ChatEffect::RecordUsage(usage) => stored.usage.push(usage.clone()),
            }
        }
        Ok(())
    }
}

// Inference returns complete-message and usage Effects, separate from display Events.
enum ChatEffect {
    AppendMessage(Message),
    RecordUsage(ProviderUsage),
}

impl From<Message> for ChatEffect {
    fn from(message: Message) -> Self {
        ChatEffect::AppendMessage(message)
    }
}

impl InferenceEffect for ChatEffect {
    fn record_usage(usage: ProviderUsage) -> Self {
        ChatEffect::RecordUsage(usage)
    }
}

impl MachineEffect for ChatEffect {
    fn ensure_message_ids(&mut self) {
        if let ChatEffect::AppendMessage(message) = self {
            let with_id = message.clone().with_generated_id_if_missing();
            *message = with_id;
        }
    }
}

#[derive(Default)]
struct EventSummary {
    message_events: usize,
    non_text_blocks: usize,
    saw_text: bool,
}

async fn display_events(mut receiver: mpsc::Receiver<AgentEvent>) -> io::Result<EventSummary> {
    let mut summary = EventSummary::default();
    while let Some(event) = receiver.recv().await {
        match event {
            AgentEvent::Message(message) => {
                summary.message_events += 1;
                let text = message.as_concat_text();
                for content in &message.content {
                    if content.as_text().is_none() {
                        summary.non_text_blocks += 1;
                    }
                }
                if !text.is_empty() {
                    if !summary.saw_text {
                        print_heading("provider → application (stream):");
                        summary.saw_text = true;
                    }
                    print!("{}", text);
                    io::stdout().flush()?;
                }
            }
            // No tools/history replacement in this assembly; these carry no display text here.
            AgentEvent::Usage(_)
            | AgentEvent::MessageUsage { .. }
            | AgentEvent::McpNotification(_)
            | AgentEvent::HistoryReplaced(_) => {}
        }
    }
    if summary.saw_text {
        println!();
    }
    Ok(summary)
}

fn display_session(label: &str, session: &ChatSession) {
    print_heading(&format!("\nSession {}", label.to_lowercase()));
    print!(
        "{}",
        session_fields_with_color(session, stdout_color_enabled())
    );
}

#[cfg(test)]
fn session_fields(session: &ChatSession) -> String {
    session_fields_with_color(session, false)
}

// Color field labels gold, not Debug type names or string values containing colons.
fn gold_session_attributes(fields: &str, enabled: bool) -> String {
    if !enabled {
        return fields.to_string();
    }
    let mut output = String::new();
    for line in fields.split_inclusive('\n') {
        let trimmed = line.trim_start();
        let indentation = line.len() - trimmed.len();
        let mut label_end = None;
        if trimmed.starts_with('"') {
            let mut escaped = false;
            for (index, character) in trimmed.char_indices().skip(1) {
                if escaped {
                    escaped = false;
                } else if character == '\\' {
                    escaped = true;
                } else if character == '"' {
                    if trimmed[index + 1..].starts_with(':') {
                        label_end = Some(index + 1);
                    }
                    break;
                }
            }
        } else if let Some(colon) = trimmed.find(':') {
            let name = &trimmed[..colon];
            if !name.is_empty()
                && name
                    .chars()
                    .all(|character| character.is_ascii_alphanumeric() || character == '_')
                && !name.as_bytes()[0].is_ascii_digit()
                && !trimmed[colon + 1..].starts_with(':')
            {
                label_end = Some(colon);
            }
        }
        match label_end {
            Some(end) => {
                output.push_str(&line[..indentation]);
                output.push_str("\x1b[38;2;255;215;0m");
                output.push_str(&trimmed[..end]);
                // Restore the default foreground without resetting usage dimming.
                output.push_str("\x1b[39m");
                output.push_str(&trimmed[end..]);
            }
            None => output.push_str(line),
        }
    }
    output
}

fn session_fields_with_color(session: &ChatSession, color_enabled: bool) -> String {
    // Debug formatting includes absent/default fields that JSON serialization can omit.
    let conversation = format!(
        "id: {:?}\nconversation: {:#?}\n",
        session.id,
        session.conversation.messages()
    );
    let usage = format!("usage: {:#?}", session.usage);
    let displayed_conversation = gold_session_attributes(&conversation, color_enabled);
    let gold_usage = gold_session_attributes(&usage, color_enabled);
    let displayed_usage = styled(&gold_usage, TextStyle::Dim, color_enabled);
    format!("{}{}\n\n\n", displayed_conversation, displayed_usage)
}

fn validate_exchange(session: &ChatSession, summary: &EventSummary) -> anyhow::Result<()> {
    if !summary.saw_text {
        return Err(anyhow::anyhow!("no response text was displayed"));
    }
    let mut saw_assistant_text = false;
    for message in session.conversation.messages() {
        // The pinned runner saves a diagnostic and explicitly yields on an empty response.
        if message.as_concat_text()
            == "The model returned an empty response. Please resend your message to continue."
        {
            return Err(anyhow::anyhow!(
                "the runner reported an empty provider response"
            ));
        }
        if message.error_kind().is_some() {
            return Err(anyhow::anyhow!(
                "the saved response contains an inference error; inspect the AFTER view"
            ));
        }
        for content in &message.content {
            if content.as_tool_request().is_some() {
                return Err(anyhow::anyhow!(
                    "unexpected tool request in this inference-only exchange"
                ));
            }
        }
        if message.role == rmcp::model::Role::Assistant
            && !message.as_concat_text().trim().is_empty()
        {
            saw_assistant_text = true;
        }
    }
    if !saw_assistant_text {
        return Err(anyhow::anyhow!("no assistant text was saved"));
    }
    Ok(())
}

// Terminal styling is presentation only; redirected transcripts contain no ANSI codes.
#[derive(Clone, Copy)]
enum TextStyle {
    Heading,
    Error,
    Dim,
}

fn color_enabled(is_terminal: bool, no_color: bool, dumb_terminal: bool) -> bool {
    is_terminal && !no_color && !dumb_terminal
}

fn terminal_color_enabled(is_terminal: bool) -> bool {
    let no_color = env::var_os("NO_COLOR").is_some();
    let dumb_terminal = env::var_os("TERM") == Some("dumb".into());
    color_enabled(is_terminal, no_color, dumb_terminal)
}

fn stdout_color_enabled() -> bool {
    terminal_color_enabled(io::stdout().is_terminal())
}

fn styled(text: &str, style: TextStyle, enabled: bool) -> String {
    if !enabled {
        return text.to_string();
    }
    let code = match style {
        TextStyle::Heading => "1;36",
        TextStyle::Error => "1;31",
        TextStyle::Dim => "2",
    };
    format!("\x1b[{}m{}\x1b[0m", code, text)
}

fn print_heading(text: &str) {
    println!(
        "{}",
        styled(text, TextStyle::Heading, stdout_color_enabled())
    );
}

fn print_error(text: &str) {
    let enabled = terminal_color_enabled(io::stderr().is_terminal());
    eprintln!("{}", styled(text, TextStyle::Error, enabled));
}

fn provider_config_path() -> anyhow::Result<PathBuf> {
    let mut arguments = env::args_os();
    let _program = arguments.next();
    let path = match (arguments.next(), arguments.next()) {
        (Some(path), None) => PathBuf::from(path),
        (None, None) => PathBuf::from(DEFAULT_PROVIDER_CONFIG),
        _ => return Err(anyhow::anyhow!("Usage: cargo run -- [provider.json]")),
    };
    Ok(path)
}

fn first_configured_model(config: &Value) -> anyhow::Result<String> {
    let models = match config.get("models").and_then(Value::as_array) {
        Some(models) => models,
        None => {
            return Err(anyhow::anyhow!(
                "provider JSON declares no configured models"
            ))
        }
    };
    let first = match models.first() {
        Some(first) => first,
        None => {
            return Err(anyhow::anyhow!(
                "provider JSON declares no configured models"
            ))
        }
    };
    let name = match first.get("name").and_then(Value::as_str) {
        Some(name) => name,
        None => return Err(anyhow::anyhow!("first configured model has no name")),
    };
    Ok(name.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use goose_agent::operation::Inference;
    use goose_providers::conversation::token_usage::Usage;

    #[test]
    fn terminal_colors_respect_redirection_no_color_and_dumb_terminals() {
        assert!(color_enabled(true, false, false));
        assert!(!color_enabled(false, false, false));
        assert!(!color_enabled(true, true, false));
        assert!(!color_enabled(true, false, true));
        assert_eq!(styled("payload", TextStyle::Heading, false), "payload");
        assert_eq!(
            styled("payload", TextStyle::Heading, true),
            "\x1b[1;36mpayload\x1b[0m"
        );
        assert_eq!(
            styled("failure", TextStyle::Error, true),
            "\x1b[1;31mfailure\x1b[0m"
        );
        assert_eq!(styled("usage", TextStyle::Dim, true), "\x1b[2musage\x1b[0m");
    }

    #[test]
    fn session_attribute_gold_leaves_values_and_type_names_unchanged() {
        let fields = "id: \"value: unchanged\"\n    text: \"role: User\",\n    \"entry_price\": String(\"51.20\"),\n    TextContent {\n        \"not a key: just text\",\n    }\n";
        let gold = gold_session_attributes(fields, true);
        assert!(gold.contains("\x1b[38;2;255;215;0mid\x1b[39m: \"value: unchanged\""));
        assert!(gold.contains("\x1b[38;2;255;215;0mtext\x1b[39m: \"role: User\""));
        assert!(gold.contains("\x1b[38;2;255;215;0m\"entry_price\"\x1b[39m: String(\"51.20\")"));
        assert_eq!(gold.matches("\x1b[38;2;255;215;0m").count(), 3);
        assert_eq!(
            gold.replace("\x1b[38;2;255;215;0m", "")
                .replace("\x1b[39m", ""),
            fields
        );
        assert_eq!(gold_session_attributes(fields, false), fields);
    }

    #[tokio::test]
    async fn session_styling_dims_only_usage_and_preserves_complete_plain_text(
    ) -> anyhow::Result<()> {
        let runtime = ChatRuntime::seeded();
        let mut session = runtime.load(SESSION_ID).await?;
        session.usage.push(ProviderUsage::new(
            "test-model".to_string(),
            Usage::default(),
        ));
        let mut message = Message::assistant().with_text("Paris.");
        message.metadata.usage = Some(Box::new(
            goose_providers::conversation::message::MessageUsage::default(),
        ));
        session.conversation.push(message);
        let colored = session_fields_with_color(&session, true);
        let usage = format!("usage: {:#?}", session.usage);
        let gold_usage = gold_session_attributes(&usage, true);
        let expected_dimmed = format!("\x1b[2m{}\x1b[0m", gold_usage);
        assert!(colored.contains(&expected_dimmed));
        assert_eq!(colored.matches("\x1b[2m").count(), 1);
        let conversation_end = colored.find("\x1b[2m").expect("top-level usage style");
        let displayed_conversation = &colored[..conversation_end];
        assert!(displayed_conversation.contains("\x1b[38;2;255;215;0musage\x1b[39m: Some("));
        assert!(
            displayed_conversation.contains("\x1b[38;2;255;215;0mcache_read_tokens\x1b[39m: None")
        );
        assert_eq!(colored.matches("\x1b[0m").count(), 1);
        let plain = colored
            .replace("\x1b[2m", "")
            .replace("\x1b[0m", "")
            .replace("\x1b[38;2;255;215;0m", "")
            .replace("\x1b[39m", "");
        assert_eq!(plain, session_fields(&session));
        let original_fields = format!(
            "id: {:?}\nconversation: {:#?}\nusage: {:#?}\n\n\n",
            session.id,
            session.conversation.messages(),
            session.usage
        );
        assert_eq!(plain, original_fields);
        assert!(!session_fields_with_color(&session, false).contains('\x1b'));
        assert!(colored.ends_with("\x1b[0m\n\n\n"));
        assert!(colored.contains("\x1b[38;2;255;215;0mrole\x1b[39m: User"));
        assert!(colored.contains("\x1b[38;2;255;215;0minput_tokens\x1b[39m:"));
        Ok(())
    }

    #[tokio::test]
    async fn effects_are_saved_and_loading_returns_an_independent_snapshot() -> anyhow::Result<()> {
        let runtime = ChatRuntime::seeded();
        let before = runtime.load(SESSION_ID).await?;
        let (sender, _receiver) = mpsc::channel(2);
        let emit = Emitter::new(sender, CancellationToken::new());
        let usage = ProviderUsage::new("test-model".to_string(), Usage::default());
        let mut effects = vec![
            ChatEffect::from(Message::assistant().with_text("Paris.")),
            ChatEffect::record_usage(usage),
        ];
        for effect in &mut effects {
            effect.ensure_message_ids();
        }
        runtime.apply_effects(&before, &mut effects, &emit).await?;
        let after = runtime.load(SESSION_ID).await?;
        assert_eq!(before.conversation.len(), 1);
        assert_eq!(after.conversation.len(), 2);
        assert_eq!(after.usage.len(), 1);
        let fields = session_fields(&after);
        for name in [
            "model:",
            "input_tokens:",
            "output_tokens:",
            "total_tokens:",
            "cache_read_input_tokens:",
            "cache_write_input_tokens:",
            "stats:",
            "cost:",
            "cost_source:",
            "finish_reasons:",
            "response_id:",
            "additional_data:",
        ] {
            assert!(fields.contains(name), "missing usage field {}", name);
        }
        assert!(fields.contains("stats: None"));
        let saved = match after.conversation.last() {
            Some(saved) => saved,
            None => return Err(anyhow::anyhow!("missing saved reply")),
        };
        assert_eq!(saved.as_concat_text(), "Paris.");
        assert!(saved.id.is_some());
        assert!(runtime.load("missing").await.is_err());
        Ok(())
    }

    #[tokio::test]
    async fn session_view_includes_all_message_and_metadata_fields() -> anyhow::Result<()> {
        let runtime = ChatRuntime::seeded();
        let session = runtime.load(SESSION_ID).await?;
        let fields = session_fields(&session);
        for name in [
            "id:",
            "conversation:",
            "usage:",
            "role:",
            "created:",
            "content:",
            "metadata:",
            "user_visible:",
            "agent_visible:",
            "inference:",
            "output_token_limit_reached:",
            "steer:",
            "turn_context:",
            "operations:",
        ] {
            assert!(fields.contains(name), "missing field {}", name);
        }
        assert!(fields.contains("inference: None"));
        assert!(fields.contains("output_token_limit_reached: false"));
        assert!(
            fields.ends_with("\n\n\n"),
            "two blank lines after raw fields"
        );
        Ok(())
    }

    #[tokio::test]
    async fn empty_response_diagnostic_is_not_an_ordinary_answer() -> anyhow::Result<()> {
        let runtime = ChatRuntime::seeded();
        let mut session = runtime.load(SESSION_ID).await?;
        session.conversation.push(Message::assistant().with_text(
            "The model returned an empty response. Please resend your message to continue.",
        ));
        let summary = EventSummary {
            saw_text: true,
            ..EventSummary::default()
        };
        assert!(validate_exchange(&session, &summary).is_err());
        Ok(())
    }

    #[tokio::test]
    async fn event_consumption_does_not_save_messages_and_drains_after_close() -> anyhow::Result<()>
    {
        let runtime = ChatRuntime::seeded();
        let (sender, receiver) = mpsc::channel(1);
        let producer = async {
            for _index in 0..3 {
                sender
                    .send(AgentEvent::Message(
                        Message::assistant().with_text("Paris."),
                    ))
                    .await?;
            }
            drop(sender);
            Ok::<(), anyhow::Error>(())
        };
        let (produced, displayed) = tokio::join!(producer, display_events(receiver));
        produced?;
        let summary = displayed?;
        assert_eq!(summary.message_events, 3);
        assert!(summary.saw_text);
        let session = runtime.load(SESSION_ID).await?;
        assert_eq!(session.conversation.len(), 1);
        Ok(())
    }

    #[tokio::test]
    async fn ordinary_saved_answer_makes_inference_decline_without_a_provider_call(
    ) -> anyhow::Result<()> {
        let json = fs::read_to_string(DEFAULT_PROVIDER_CONFIG)?;
        let provider = from_json(&json, None, EnvKeyResolver {})?;
        let config: Value = serde_json::from_str(&json)?;
        let model = ModelConfig::new(first_configured_model(&config)?);
        let runner: InferenceRunner<'_, ChatSession, ChatEffect> =
            InferenceRunner::new(Arc::from(provider), model);
        let runtime = ChatRuntime::seeded();
        let mut session = runtime.load(SESSION_ID).await?;
        assert!(runner.applies(&session.conversation));
        session
            .conversation
            .push(Message::assistant().with_text("Paris."));
        assert!(!runner.applies(&session.conversation));
        let steps = vec![Step::Inference(Arc::new(runner))];
        let machine = StateMachine::new(steps, CancellationToken::new());
        runtime
            .sessions
            .lock()
            .await
            .insert(SESSION_ID.to_string(), session);
        let (sender, mut receiver) = mpsc::channel(1);
        let emit = Emitter::new(sender, CancellationToken::new());
        let final_session = machine.run(&runtime, SESSION_ID, &emit).await?;
        drop(emit);
        assert_eq!(final_session.conversation.len(), 2);
        assert!(receiver.recv().await.is_none());
        Ok(())
    }
}
