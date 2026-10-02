//! Spike experiment 1: the machine runs one request and one response, by itself.
//!
//! Lessons 3-8 all hand-wrote the pass loop: load the state, ask a step,
//! persist the effects, repeat. This experiment hands that loop to the GDK.
//! `StateMachine::run` is the GDK's own driver — it loads the session, asks
//! the registered steps in order, applies the winning step's effects, and
//! repeats until one of its own stop conditions fires. This file contains no
//! `loop`, no pass counter, and no hand-written bound: one seeded question
//! goes in, one reply comes back, and the machine stops on its own.
//!
//! Why it stops: after the reply is persisted, the reloaded conversation ends
//! with an assistant message, so the inference step declines and no registered
//! step applies. That "no step applies" stop belongs to the machine; nothing
//! in this file decides when the run is over.
//!
//! Run from the repository root (so .env and the provider JSON resolve):
//!   cargo run --manifest-path spike/state-machine/Cargo.toml -- custom_aa_llama_qwen3_6-35b.json

use std::{
    collections::HashMap,
    env,
    error::Error,
    fs,
    path::PathBuf,
    sync::{Arc, Mutex},
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
    conversation::{effective_role, message::Message, token_usage::ProviderUsage, Conversation},
    declarative::{from_json, EnvKeyResolver},
    model::ModelConfig,
};
use serde_json::Value;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

const DEFAULT_PROVIDER_CONFIG: &str = "custom_aa_llama_qwen3_6-35b.json";
const SESSION_ID: &str = "spike-experiment-1";
const USER_PROMPT: &str = "What is the capital of France?";

/// A delimiter printed before and after every block of protocol payload: the
/// conversation sent to the provider and the response streamed back. Everything
/// between two delimiter lines is a value that traveled to or from the provider;
/// everything else on the terminal is this application's explanatory text.
const PAYLOAD_DELIMITER: &str = "++++++++";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv().ok();

    // creating the provider
    let provider_config_path = provider_config_path()?;
    let provider_json = fs::read_to_string(&provider_config_path)?;
    let provider_config: Value = serde_json::from_str(&provider_json)?;
    let boxed_provider: Box<dyn Provider> = from_json(&provider_json, None, EnvKeyResolver {})?;
    let provider: Arc<dyn Provider> = Arc::from(boxed_provider);
    let model = ModelConfig::new(first_configured_model(&provider_config)?);
    println!(
        "Provider '{}' with model '{}'",
        provider.get_name(),
        model.model_name
    );

    // Seed the store with exactly one user message. The provider is stateless,
    // so this conversation is the whole context of the request. run() reloads
    // it from this store at the start of every pass.
    let seed = seed_conversation();

    // The outbound payload of the only request this run should make: the
    // seeded conversation, exactly as it sits in the store.
    println!(
        "Seeded session '{}' with 1 user message: {}",
        SESSION_ID, USER_PROMPT
    );
    println!("\nOutbound payload (application → provider):");
    print_delimiter();
    print_messages(seed.messages());
    print_delimiter();

    let store = InMemoryStore::seeded(seed);

    // The single registered step is the GDK-shipped inference runner. Nobody
    // in this file decides when the model is called; the machine asks the
    // step whether the persisted conversation needs a model response.
    let runner = InferenceRunner::new(provider, model);
    let steps: Vec<Step<'_, ChatSession, ChatEffect>> = vec![Step::Inference(Arc::new(runner))];
    let cancel = CancellationToken::new();
    let machine = StateMachine::new(steps, cancel.clone());

    // A large buffer: the machine owns the loop now, so this application
    // cannot drain events between passes. The buffer must hold a whole
    // streamed response so the emitter never blocks mid-stream; the drain
    // happens once run() has returned.
    let (event_tx, mut event_rx) = mpsc::channel::<AgentEvent>(4096);
    let emit = Emitter::new(event_tx, cancel);

    // The whole experiment is this one call. Inside run(), the GDK repeats
    // the same shape Lessons 3-8 wrote by hand:
    //   1. load the session from the store (a fresh view on every pass),
    //   2. ask the registered steps, in order, for the first that applies,
    //   3. apply that step's effects back through the effect handler,
    //   4. stop when no step applies or a step yields to the client.
    // This file supplies the store, the step, and the emitter — not the loop.
    //
    // Expected trace: pass 1 — the inference step applies (the conversation
    // ends with a user question), the provider streams a reply, the effect
    // handler appends it. Pass 2 — the reloaded conversation ends with an
    // assistant message, so the inference step declines; no step applies and
    // run() stops. One request, one reply, no loop in this file.
    let final_session = machine.run(&store, SESSION_ID, &emit).await?;

    println!("\nStreamed response (provider → application):");
    drain_and_print_events(&mut event_rx);

    // run() hands back the final session, reloaded from the store. The reply
    // reached the conversation only because the effect handler wrote it there.
    let final_conversation = match final_session.conversation() {
        Some(conversation) => conversation,
        None => return Err("the returned session has no conversation".into()),
    };
    println!(
        "\nFinal persisted conversation ({} message(s), handed back by run()):",
        final_conversation.len()
    );
    print_messages(final_conversation.messages());

    Ok(())
}

/// Drain every buffered event and print the streamed text between payload
/// delimiters. The inference runner emits one event per streamed chunk while
/// the machine's step is still running; drained in order after run() returns,
/// the chunk texts reconstruct the response. The delimiter opens at the first
/// text and closes after the stream ends — individual deltas cannot each
/// carry a fence of their own.
fn drain_and_print_events(event_rx: &mut mpsc::Receiver<AgentEvent>) {
    let mut streamed_text = false;
    while let Ok(event) = event_rx.try_recv() {
        let AgentEvent::Message(message) = event else {
            // The GDK defines other event kinds; none are produced here.
            continue;
        };
        let text = message.as_concat_text();
        if text.is_empty() {
            continue;
        }
        if !streamed_text {
            print_delimiter();
        }
        print!("{}", text);
        streamed_text = true;
    }
    if streamed_text {
        println!();
        print_delimiter();
    }
}

/// The state the machine reloads on every pass. It pairs a session id with
/// the conversation that session owns.
struct ChatSession {
    conversation: Conversation,
}

impl MachineSession for ChatSession {
    fn id(&self) -> &str {
        SESSION_ID
    }

    fn conversation(&self) -> Option<&Conversation> {
        Some(&self.conversation)
    }
}

struct InMemoryStore {
    conversations: Mutex<HashMap<String, Conversation>>,
}

impl InMemoryStore {
    fn seeded(seed: Conversation) -> Self {
        let mut conversations = HashMap::new();
        conversations.insert(SESSION_ID.to_string(), seed);
        InMemoryStore {
            conversations: Mutex::new(conversations),
        }
    }
}

/// `run()` requires one runtime object that both loads sessions and applies
/// effects. The machine knows nothing about the store behind these impls.
#[async_trait]
impl SessionLoader<ChatSession> for InMemoryStore {
    async fn load(&self, session_id: &str) -> anyhow::Result<ChatSession> {
        let conversations = self.conversations.lock().expect("store lock");
        let conversation = match conversations.get(session_id) {
            Some(conversation) => conversation.clone(),
            None => {
                let failure = format!("no session '{}' exists in the store", session_id);
                return Err(anyhow::anyhow!(failure));
            }
        };
        Ok(ChatSession { conversation })
    }
}

#[async_trait]
impl EffectHandler<ChatSession, ChatEffect> for InMemoryStore {
    async fn apply_effects(
        &self,
        session: &ChatSession,
        effects: &mut [ChatEffect],
        _emit: &Emitter,
    ) -> anyhow::Result<()> {
        for effect in effects {
            match effect {
                ChatEffect::AppendMessage(message) => {
                    let role = effective_role(message);
                    println!(
                        "persisting effect: append message (effective role {})",
                        role
                    );
                    let mut conversations = self.conversations.lock().expect("store lock");
                    let stored = match conversations.get_mut(session.id()) {
                        Some(stored) => stored,
                        None => {
                            let failure =
                                format!("no session '{}' exists in the store", session.id());
                            return Err(anyhow::anyhow!(failure));
                        }
                    };
                    stored.push(message.clone());
                }
                ChatEffect::RecordUsage(usage) => {
                    let input_tokens = match usage.usage.input_tokens {
                        Some(count) => count.to_string(),
                        None => "-".to_string(),
                    };
                    let output_tokens = match usage.usage.output_tokens {
                        Some(count) => count.to_string(),
                        None => "-".to_string(),
                    };
                    println!(
                        "persisting effect: record usage (model={}, tokens in={} out={})",
                        usage.model, input_tokens, output_tokens
                    );
                }
            }
        }
        Ok(())
    }
}

/// The application's effect vocabulary. The GDK's default `ConversationEffect`
/// has no `InferenceEffect` implementation in this pinned release, so this
/// experiment defines its own effect type: the machine only requires an
/// effect type it can hand message ids to, and the shipped inference runner
/// requires one that can also record usage.
enum ChatEffect {
    AppendMessage(Message),
    RecordUsage(ProviderUsage),
}

impl MachineEffect for ChatEffect {
    fn ensure_message_ids(&mut self) {
        // The emitter already generated an id for every streamed message, but
        // the machine asks each effect to guarantee one before it persists.
        if let ChatEffect::AppendMessage(message) = self {
            let taken = std::mem::replace(message, Message::assistant());
            let with_id = taken.with_generated_id_if_missing();
            *message = with_id;
        }
    }
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

/// The seed: exactly one user message. Nothing else has happened yet.
fn seed_conversation() -> Conversation {
    let mut conversation = Conversation::empty();
    conversation.push(Message::user().with_text(USER_PROMPT));
    conversation
}

fn provider_config_path() -> Result<PathBuf, Box<dyn Error>> {
    match env::args().nth(1) {
        Some(path) => Ok(PathBuf::from(path)),
        None => Ok(PathBuf::from(DEFAULT_PROVIDER_CONFIG)),
    }
}

fn first_configured_model(provider_config: &Value) -> Result<String, Box<dyn Error>> {
    let failure = "provider JSON declares no configured models";
    let models = match provider_config.get("models").and_then(Value::as_array) {
        Some(models) => models,
        None => return Err(failure.into()),
    };
    let first_model = match models.first() {
        Some(first_model) => first_model,
        None => return Err(failure.into()),
    };
    let name = match first_model.get("name").and_then(Value::as_str) {
        Some(name) => name,
        None => return Err(failure.into()),
    };
    Ok(name.to_owned())
}

fn print_messages(messages: &[Message]) {
    for message in messages {
        println!("  [{:?}] {}", message.role, message.as_concat_text());
    }
}

fn print_delimiter() {
    println!("{}", PAYLOAD_DELIMITER);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seeded_store_holds_the_user_question() {
        let store = InMemoryStore::seeded(seed_conversation());
        let conversations = store.conversations.lock().expect("store lock");
        let conversation = match conversations.get(SESSION_ID) {
            Some(conversation) => conversation,
            None => panic!("the seeded session must be present"),
        };
        assert_eq!(conversation.len(), 1);
        let text = conversation.messages()[0].as_concat_text();
        assert!(
            text.contains("capital of France"),
            "unexpected seed text: {}",
            text
        );
    }

    #[tokio::test]
    async fn loader_reloads_the_seeded_session() {
        let store = InMemoryStore::seeded(seed_conversation());
        let session = store.load(SESSION_ID).await.expect("session load");
        assert_eq!(session.conversation().map(|c| c.len()), Some(1));
        assert!(store.load("missing").await.is_err());
    }
}