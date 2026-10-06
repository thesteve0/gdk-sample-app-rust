use std::{collections::HashMap, env, error::Error, fs, path::PathBuf, sync::Arc};

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
use tokio::sync::{mpsc, Mutex};
use tokio_util::sync::CancellationToken;

const DEFAULT_PROVIDER_CONFIG: &str = "custom_aa_llama_qwen3_6-35b.json";
const SESSION_ID: &str = "lesson-08";
const USER_PROMPT: &str = "What is the capital of France?";
const MAX_STATE_STEPS: u32 = 4;

/// A delimiter printed before and after every block of protocol payload: the
/// conversation sent to the provider and the response streamed back. Everything
/// between two delimiter lines is a value that traveled to or from the provider;
/// everything else on the terminal is this application's explanatory text.
const PAYLOAD_DELIMITER: &str = "++++++++";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv().ok();

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
    // so this conversation is the whole context of the request; the machine
    // will reload it from this store on every pass.
    let store = Arc::new(InMemoryStore {
        conversations: Mutex::new(HashMap::new()),
    });
    store
        .conversations
        .lock()
        .await
        .insert(SESSION_ID.to_string(), seed_conversation());
    println!(
        "Seeded session '{}' with 1 user message: {}",
        SESSION_ID, USER_PROMPT
    );

    // The single registered step is the GDK-shipped inference runner. The
    // machine asks it whether the current conversation requires a model
    // response (re-derivation from persisted state), and only then fires it.
    let runner = InferenceRunner::new(provider, model);
    let steps: Vec<Step<'_, ChatSession, ChatEffect>> = vec![Step::Inference(Arc::new(runner))];
    let cancel = CancellationToken::new();
    let machine = StateMachine::new(steps, cancel.clone());

    // A large buffer: the inference runner emits one event per streamed chunk
    // while the machine's step is still running, and this loop drains events
    // only after the step returns. The buffer must hold a whole response so
    // the emitter never blocks mid-stream.
    let (event_tx, mut event_rx) = mpsc::channel::<AgentEvent>(4096);
    let emit = Emitter::new(event_tx, cancel);

    let runtime = ChatRuntime { store };
    let outcome =
        run_state_machine(&machine, &runtime, &emit, &mut event_rx, MAX_STATE_STEPS).await?;

    match outcome {
        RunOutcome::StoppedNoOperation => {
            println!("\nNo step applies to the final conversation — the machine stops.");
        }
        RunOutcome::YieldedToClient => {
            println!("\nA step yielded control to the client — the machine stops.");
        }
        RunOutcome::StepBoundReached => {
            println!(
                "\nThe state-step bound ({}) was reached — the machine stops.",
                MAX_STATE_STEPS
            );
        }
    }

    // The final conversation lives in the store, not in loop memory. Reload it
    // one last time to prove that the reply was persisted as an effect.
    let final_session = runtime.load(SESSION_ID).await?;
    let final_conversation = match final_session.conversation() {
        Some(conversation) => conversation,
        None => return Err("the loaded session has no conversation".into()),
    };
    println!(
        "Final persisted conversation: {} message(s)",
        final_conversation.len()
    );
    for message in final_conversation.messages() {
        println!("  [{:?}] {}", message.role, message.as_concat_text());
    }

    Ok(())
}

/// How this run of the machine ended. The machine has three stop conditions:
/// no step applies, a step yields control to the client, or (in this
/// hand-written loop) an explicit state-step bound is reached.
enum RunOutcome {
    StoppedNoOperation,
    YieldedToClient,
    StepBoundReached,
}

/// The pass loop, written by hand so the state-step bound stays explicit.
/// Every pass is the same shape: reload, ask the steps, print the round trip,
/// persist the effects. Nothing about the decision is remembered between
/// passes — the next pass re-derives it from the reloaded conversation.
async fn run_state_machine(
    machine: &StateMachine<'_, ChatSession, ChatEffect>,
    runtime: &ChatRuntime,
    emit: &Emitter,
    event_rx: &mut mpsc::Receiver<AgentEvent>,
    max_state_steps: u32,
) -> Result<RunOutcome, Box<dyn Error>> {
    for state_step in 1..=max_state_steps {
        // 1. Reload. The machine never caches the session; the store is the
        // source of truth at the start of every pass.
        let session = runtime.load(SESSION_ID).await?;
        let conversation = match session.conversation() {
            Some(conversation) => conversation,
            None => return Err("the loaded session has no conversation".into()),
        };
        println!(
            "\n── Pass {}: reloaded the persisted conversation ({} message(s)) ──",
            state_step,
            conversation.len()
        );

        // 2. Ask the registered steps, in order. The inference step is asked
        // whether it applies; if it declines, the machine reports that no
        // step applies and this loop stops.
        let outcome = machine.step(&session, emit).await?;
        let mut result = match outcome {
            Some(result) => result,
            None => return Ok(RunOutcome::StoppedNoOperation),
        };

        // 3. A step applied and already called the provider. The provider is
        // stateless, so the outbound payload is the whole conversation; print
        // it, then the streamed response, so the round trip reads in order.
        let applied_step = result.applied_step.unwrap_or("unknown");
        println!(
            "\nThe '{}' step applied — it asked the provider.",
            applied_step
        );
        println!("\nOutbound payload sent with this call (application → provider):");
        print_delimiter();
        for message in conversation.messages() {
            println!("  [{:?}] {}", message.role, message.as_concat_text());
        }
        print_delimiter();

        println!("\nStreamed response (provider → application):");
        drain_and_print_events(event_rx);

        // 4. Persist the effects. This is the only way the world changes:
        // the reply enters the conversation because the effect handler
        // writes it into the store.
        machine.apply(runtime, &session, &mut result, emit).await?;

        // 5. Drain anything the effect application emitted.
        drain_and_print_events(event_rx);

        // 6. The machine also stops when a step yields control to the client.
        // This lesson's single inference step yields only for an empty
        // response, but the check belongs to the loop's contract.
        if result.yield_to_client {
            return Ok(RunOutcome::YieldedToClient);
        }
    }

    Ok(RunOutcome::StepBoundReached)
}

/// Drain every buffered event and print the streamed text between payload
/// delimiters. The inference runner emits one event per streamed chunk while
/// it runs; drained in order, the chunk texts reconstruct the response. The
/// delimiter opens at the first text and closes after the stream ends —
/// individual deltas cannot each carry a fence of their own.
fn drain_and_print_events(event_rx: &mut mpsc::Receiver<AgentEvent>) {
    let mut streamed_text = false;
    while let Ok(event) = event_rx.try_recv() {
        let AgentEvent::Message(message) = event else {
            // The GDK defines other event kinds; none are produced by this lesson.
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

/// The machine's whole persistence interface: it loads sessions and applies
/// effects. The machine knows nothing about the store behind this type.
struct ChatRuntime {
    store: Arc<InMemoryStore>,
}

#[async_trait]
impl SessionLoader<ChatSession> for ChatRuntime {
    async fn load(&self, session_id: &str) -> anyhow::Result<ChatSession> {
        let conversations = self.store.conversations.lock().await;
        let conversation = match conversations.get(session_id) {
            Some(conversation) => conversation,
            None => {
                let failure = format!("no session '{}' exists in the store", session_id);
                return Err(anyhow::anyhow!(failure));
            }
        };
        Ok(ChatSession {
            conversation: conversation.clone(),
        })
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
        for effect in effects {
            match effect {
                ChatEffect::AppendMessage(message) => {
                    let role = effective_role(message);
                    println!(
                        "persisting effect: append message (effective role {})",
                        role
                    );
                    let mut conversations = self.store.conversations.lock().await;
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
/// lesson defines its own: the machine only requires an effect type it can
/// give message ids to, and the shipped inference runner requires one that
/// can also record usage.
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

fn print_delimiter() {
    println!("{}", PAYLOAD_DELIMITER);
}

#[cfg(test)]
mod tests {
    use super::*;
    use goose_agent::operation::{applied, not_applicable, Operation, OperationResult};

    #[test]
    fn seeded_conversation_holds_one_user_question() {
        let conversation = seed_conversation();
        assert_eq!(conversation.len(), 1);
        let mut text = String::new();
        for message in conversation.messages() {
            text.push_str(&message.as_concat_text());
        }
        assert!(
            text.contains("capital of France"),
            "unexpected seed text: {}",
            text
        );
    }

    #[tokio::test]
    async fn loader_round_trips_the_seeded_conversation() {
        let store = Arc::new(InMemoryStore {
            conversations: Mutex::new(HashMap::new()),
        });
        store
            .conversations
            .lock()
            .await
            .insert(SESSION_ID.to_string(), seed_conversation());
        let runtime = ChatRuntime { store };
        let session = runtime.load(SESSION_ID).await.unwrap();
        assert_eq!(session.conversation().map(|c| c.len()), Some(1));
        assert!(runtime.load("missing").await.is_err());
    }

    #[tokio::test]
    async fn effect_handler_persists_appended_messages() {
        let store = Arc::new(InMemoryStore {
            conversations: Mutex::new(HashMap::new()),
        });
        store
            .conversations
            .lock()
            .await
            .insert(SESSION_ID.to_string(), seed_conversation());
        let runtime = ChatRuntime { store };
        let session = runtime.load(SESSION_ID).await.unwrap();
        let reply = Message::assistant().with_text("Paris is the capital of France.");
        let mut effects = vec![ChatEffect::AppendMessage(reply)];
        let (event_tx, _event_rx) = mpsc::channel::<AgentEvent>(16);
        let emit = Emitter::new(event_tx, CancellationToken::new());
        runtime
            .apply_effects(&session, &mut effects, &emit)
            .await
            .unwrap();
        let reloaded = runtime.load(SESSION_ID).await.unwrap();
        let conversation = reloaded.conversation().unwrap();
        assert_eq!(conversation.len(), 2);
        let last = &conversation.messages()[1];
        assert!(last.as_concat_text().contains("Paris"));
    }

    #[tokio::test]
    async fn a_declining_step_stops_the_machine() {
        let steps: Vec<Step<'_, ChatSession, ChatEffect>> =
            vec![Step::Operation(Arc::new(AlwaysDeclines))];
        let machine = StateMachine::new(steps, CancellationToken::new());
        let (event_tx, mut event_rx) = mpsc::channel::<AgentEvent>(16);
        let emit = Emitter::new(event_tx, CancellationToken::new());
        let store = Arc::new(InMemoryStore {
            conversations: Mutex::new(HashMap::new()),
        });
        store
            .conversations
            .lock()
            .await
            .insert(SESSION_ID.to_string(), seed_conversation());
        let runtime = ChatRuntime { store };
        let outcome = run_state_machine(&machine, &runtime, &emit, &mut event_rx, MAX_STATE_STEPS)
            .await
            .unwrap();
        match outcome {
            RunOutcome::StoppedNoOperation => {}
            _ => panic!("a step that never applies must stop the machine"),
        }
    }

    #[tokio::test]
    async fn the_state_step_bound_stops_a_run_that_never_stalls() {
        let steps: Vec<Step<'_, ChatSession, ChatEffect>> =
            vec![Step::Operation(Arc::new(AlwaysApplies))];
        let machine = StateMachine::new(steps, CancellationToken::new());
        let (event_tx, mut event_rx) = mpsc::channel::<AgentEvent>(16);
        let emit = Emitter::new(event_tx, CancellationToken::new());
        let store = Arc::new(InMemoryStore {
            conversations: Mutex::new(HashMap::new()),
        });
        store
            .conversations
            .lock()
            .await
            .insert(SESSION_ID.to_string(), seed_conversation());
        let runtime = ChatRuntime { store };
        let outcome = run_state_machine(&machine, &runtime, &emit, &mut event_rx, MAX_STATE_STEPS)
            .await
            .unwrap();
        match outcome {
            RunOutcome::StepBoundReached => {}
            _ => panic!("a run whose step always applies must reach the bound"),
        }
    }

    struct AlwaysDeclines;

    #[async_trait]
    impl Operation<ChatSession, ChatEffect> for AlwaysDeclines {
        fn name(&self) -> &'static str {
            "always_declines"
        }

        async fn run(
            &self,
            _session: &ChatSession,
            _conversation: &Conversation,
            _emit: &Emitter,
        ) -> anyhow::Result<OperationResult<ChatEffect>> {
            not_applicable()
        }
    }

    struct AlwaysApplies;

    #[async_trait]
    impl Operation<ChatSession, ChatEffect> for AlwaysApplies {
        fn name(&self) -> &'static str {
            "always_applies"
        }

        async fn run(
            &self,
            _session: &ChatSession,
            _conversation: &Conversation,
            _emit: &Emitter,
        ) -> anyhow::Result<OperationResult<ChatEffect>> {
            applied(Vec::<ChatEffect>::new())
        }
    }
}
