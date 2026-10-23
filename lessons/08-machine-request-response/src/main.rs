// Lesson 8: keep the exchange, Events, and Effects visible in this file.
mod presentation;
mod provider_setup;

#[cfg(test)]
mod tests;

use presentation::{display_session, print_error, print_heading};
use provider_setup::{first_configured_model, provider_config_path};

use std::{
    collections::HashMap,
    fs,
    io::{self, Write},
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
