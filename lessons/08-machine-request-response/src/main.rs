use std::{
    collections::HashMap,
    env, fs,
    io::{self, Write},
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
async fn main() -> anyhow::Result<()> {
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
    println!("State-machine operations (in order):");
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
    println!("application → provider:");
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
                        println!("provider → application (stream):");
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
    println!("\nSession {}", label.to_lowercase());
    print!("{}", session_fields(session));
}

fn session_fields(session: &ChatSession) -> String {
    // Debug formatting includes absent/default fields that JSON serialization can omit.
    format!(
        "id: {:?}\nconversation: {:#?}\nusage: {:#?}\n\n\n",
        session.id,
        session.conversation.messages(),
        session.usage
    )
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
