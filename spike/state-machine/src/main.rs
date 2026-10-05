//! Experiment 1: one question and one answer, using the GDK state machine.
//!
//! START HERE — compare this with Lesson 3:
//!
//! Lesson 3: main -> provider.stream -> main reads fragments -> main prints.
//! This file: main -> machine.run -> inference runner -> provider.stream.
//! The runner reads the fragments; our application still prints the answer.
//!
//! WHAT IS RUNNING?
//! This is ONE Rust application. The state machine is an object inside it,
//! not another server, process, or independently running background agent.
//! Constructing the objects below only wires them together. Work starts when
//! main calls and awaits machine.run(). Tokio drives the asynchronous work.
//! We do not spawn a separate task for the machine or the display in this file.
//!
//! THE OBJECTS, IN PLAIN WORDS:
//! - Provider: the adapter that talks to the model service, as in Lesson 3.
//! - InferenceRunner: the GDK component that asks the model and reads its stream.
//!   "Inference" here just means asking the model to generate a response.
//! - StateMachine: coordinates loading the question, invoking the runner when
//!   appropriate, saving its result, and checking whether more work is needed.
//! - InMemoryStore: keeps conversations in a map inside this process.
//! - ChatSession: a loaded snapshot of one conversation from that map.
//! - Emitter: sends response updates to an in-process queue for our display code.
//!
//! TWO OUTPUT PATHS — do not confuse them:
//!
//!                     provider response fragments
//!                                 |
//!                       InferenceRunner reads them
//!                          /                  \
//!                         v                    v
//!                Emitter sends a copy    runner accumulates them
//!                to the event queue      into response messages
//!                         |                    |
//!                         v                    v
//!                main reads and prints   runner returns effects
//!                the queued fragments    (instructions to save)
//!                                              |
//!                                              v
//!                                        machine calls our
//!                                        store's effect handler
//!                                              |
//!                                              v
//!                                        saved conversation
//!
//! An EVENT is an update for presentation. An EFFECT is an instruction to
//! change application state. Printing an event does not save the answer;
//! applying the append-message effect does. Both paths carry message data,
//! but they serve different purposes.
//!
//! IMPORTANT DIFFERENCE FROM LESSON 3:
//! The provider streams its answer, but this sample waits for machine.run()
//! to finish BEFORE printing the queued fragments. It is buffered playback,
//! not live display. Live display would require reading events while the
//! machine runs; that is intentionally not implemented here.
//!
//! READING ORDER:
//! 1. Follow the numbered sections of main for the overall request/response.
//! 2. Read drain_and_print_events for the final handoff to the terminal.
//! 3. Read ChatSession and InMemoryStore for where the question/answer live.
//! 4. Read ChatEffect and its implementations for the saving instructions.
//!
//! Verified against goose-agent and goose-provider-types 0.1.0-alpha.11:
//! - goose-agent/src/machine.rs: run, step, and apply.
//! - goose-agent/src/inference.rs: provider.stream and response accumulation.
//! - goose-agent/src/operation.rs: Emitter and the effect contracts.
//! - goose-provider-types/src/conversation.rs: push merges same-ID fragments.
//!
//! These are dependency files, not code duplicated in this application.
//!
//! Run from the repository root so the provider JSON and optional .env resolve:
//!   cargo run --manifest-path spike/state-machine/Cargo.toml -- custom_aa_llama_qwen3_6-35b.json

use std::{
    collections::HashMap,
    env,
    error::Error,
    fs,
    path::PathBuf,
    sync::{Arc, Mutex},
};

// A trait is a contract: it describes methods a type must supply. Below,
// `impl SomeTrait for OurType` is how we provide those methods to the GDK.
// async_trait lets our implementations match the GDK's async trait methods.
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

// mpsc is a queue with sender and receiver handles. Here its messages travel
// within this application, not over the network to the model service.
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

const DEFAULT_PROVIDER_CONFIG: &str = "custom_aa_llama_qwen3_6-35b.json";

// A session ID is the key used to find a particular conversation in our store.
// This experiment has only one conversation, so its key is fixed.
const SESSION_ID: &str = "spike-experiment-1";

// We hard-code the question, as in Lesson 3. No terminal input is read here.
const USER_PROMPT: &str = "What is the capital of France?";

// A payload delimiter visually separates provider-related message data from
// our application's explanatory output. Printing a delimiter or a message
// does NOT send a provider request; these helpers only write to the terminal.
const PAYLOAD_DELIMITER: &str = "++++++++";

// Tokio supplies the async execution runtime. This is different from the
// storage "runtime" expected by machine.run(), which is our InMemoryStore.
// `.await` drives an async call to completion while allowing other scheduled
// work to progress when that call is waiting. It does not itself spawn a task.
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // 1. CREATE THE PROVIDER — familiar setup from the raw API lessons.
    // Load optional environment variables before resolving the provider JSON.
    // A missing .env is allowed; credentials can also already be in the environment.
    dotenvy::dotenv().ok();

    let provider_config_path = provider_config_path()?;
    let provider_json = fs::read_to_string(&provider_config_path)?;
    let provider_config: Value = serde_json::from_str(&provider_json)?;
    let boxed_provider: Box<dyn Provider> = from_json(&provider_json, None, EnvKeyResolver {})?;

    // The runner expects a shared provider handle. Arc::from changes how we
    // hold the existing adapter, not which endpoint or model service it uses.
    // `dyn Provider` lets different provider implementations share one interface.
    let provider: Arc<dyn Provider> = Arc::from(boxed_provider);
    let model = ModelConfig::new(first_configured_model(&provider_config)?);
    println!(
        "Provider '{}' with model '{}'",
        provider.get_name(),
        model.model_name
    );

    // 2. PUT THE QUESTION WHERE THE MACHINE CAN LOAD IT.
    // A conversation is an ordered collection of messages. Its initial state
    // is one user message; later it will also hold the assistant's answer.
    // "Seed" means supply the starting data before running the machine.
    let seed = seed_conversation();

    // This is a preview of the question we are about to store, not the network
    // send itself. The actual provider.stream call lives inside InferenceRunner.
    println!(
        "Seeded session '{}' with 1 user message: {}",
        SESSION_ID, USER_PROMPT
    );
    println!("\nOutbound payload (application → provider):");
    print_delimiter();
    print_messages(seed.messages());
    print_delimiter();

    // Move the conversation into the store. From now on the store is the
    // source of truth. machine.run() borrows this store; it does not own it.
    let store = InMemoryStore::seeded(seed);

    // 3. GIVE THE PROVIDER TO THE COMPONENT THAT WILL MAKE THE REQUEST.
    // In Lesson 3 main called provider.stream and read stream.next itself.
    // Here the GDK's InferenceRunner contains that call and the reading loop.
    // Creating the runner does not start inference; we are still wiring objects.
    let runner = InferenceRunner::new(provider, model);

    // A step is a registered candidate for work, not a predetermined "step 1"
    // in a script. The machine can consider this SAME candidate on each pass.
    // We register exactly one candidate: asking the model for a response.
    //
    // Read this type as: "a list of steps using ChatSession for loaded state
    // and ChatEffect for instructions to change that state". The lifetime '_
    // lets Rust infer how long the registered step objects can be referenced.
    // Arc wraps the runner in the shared handle the Step API expects.
    let steps: Vec<Step<'_, ChatSession, ChatEffect>> = vec![Step::Inference(Arc::new(runner))];

    // A cancellation token is a shared stop signal. clone() gives the machine
    // another handle to the SAME signal, not a separate cancellation state.
    // This sample never calls cancel(); the handles are required by the APIs.
    let cancel = CancellationToken::new();

    // The machine holds the step list and cancellation handle, NOT the store,
    // NOT the conversation, and NOT a directly accessible provider stream.
    let machine = StateMachine::new(steps, cancel.clone());

    // 4. SET UP A PATH FROM THE RUNNER TO OUR DISPLAY CODE.
    // event_tx is the sender; event_rx is the receiver. An AgentEvent is an
    // update for a client (our terminal), not an HTTP response object.
    // The emitter takes the sender; main keeps the receiver.
    //
    // This queue has space for 4096 EVENTS, not 4096 tokens or characters.
    // Because main only reads it AFTER run returns, it must fit every event
    // from this short response. That capacity is a demo assumption, not a
    // general guarantee: if it fills, Emitter's send waits for a reader, while
    // our reader is waiting for run to finish. The run would stall.
    // A live-streaming version would consume events while run is in progress.
    let (event_tx, mut event_rx) = mpsc::channel::<AgentEvent>(4096);
    let emit = Emitter::new(event_tx, cancel);

    // 5. START THE WORK — this is the call that drives the workflow.
    // Everything above constructed objects; this awaited call invokes them.
    //
    // Here is the normal successful call chain INSIDE the pinned GDK:
    //
    // main: machine.run(&store, SESSION_ID, &emit)
    //   -> store.load(SESSION_ID)                  [our SessionLoader below]
    //   -> machine.step(&session, &emit)
    //      -> runner.applies(conversation)        [is a model reply needed?]
    //      -> runner.infer(...)
    //         -> provider.stream(...)            [actual inference request]
    //         -> repeatedly read stream.next()   [actual streamed response]
    //            -> emit.message(chunk)          [queue a display copy]
    //            -> accumulator.push(chunk)      [assemble response messages]
    //         -> return effects: usage, then accumulated response messages
    //   -> machine.apply(...)
    //      -> store.apply_effects(...)           [our EffectHandler below]
    //   -> start another pass by loading again
    //
    // The accumulator belongs to the RUNNER, not to main or to the machine.
    // Its Conversation::push merges consecutive same-ID message fragments,
    // appending their text. This depends on fragments sharing a message ID;
    // fragments with different IDs remain separate messages. Emitter fills
    // a missing ID without replacing an existing one.
    // Printing fragments is a separate path and is not this accumulation.
    //
    // No other registered step supplies instructions or tools here, so the
    // request has an empty system prompt and no tools. Unlike Lesson 3, we
    // do not supply "you are an expert on geography" or do a model-list preflight.
    // This recreates its basic question/answer, not every Lesson 3 detail.
    //
    // PASS 1: the stored conversation ends with our user question. The runner
    // applies, calls the provider, accumulates the reply, and returns effects.
    // Our handler saves the reply before the next pass can inspect it.
    // PASS 2: the newly loaded conversation ends with the assistant's reply.
    // The runner declines. No other step exists, so run stops without making
    // a second inference request. run then loads the final session to return.
    //
    // run() can also stop after a step requests a handoff to the client
    // (yield_to_client); cancellation is handled cooperatively. In alpha.11,
    // run() has NO built-in maximum-pass bound. Do not assume an operation
    // that applies forever will be stopped automatically by this method.
    //
    // `&store` and `&emit` borrow existing objects. `SESSION_ID` identifies
    // the conversation to load. The returned value is a ChatSession, NOT the
    // stream and NOT just an answer String. `?` propagates a returned error.
    // However, the runner can represent a provider failure as a structured
    // error Message and return it as an effect rather than an Err. Our minimal
    // renderer only extracts text, so it may not show that error content.
    // A successful process exit alone therefore does not prove that the model
    // answered. Live validation must confirm actual answer text and usage.
    let final_session = machine.run(&store, SESSION_ID, &emit).await?;

    // 6. GIVE THE QUEUED ANSWER FRAGMENTS TO THE USER.
    // main resumes here only after the whole machine run finishes. The runner
    // has already read the provider stream and the store has saved the answer.
    // This label refers to where the data came from; printing is delayed.
    println!("\nStreamed response (provider → application):");
    drain_and_print_events(&mut event_rx);

    // 7. SHOW THE SAVED COPY AS A SEPARATE DIAGNOSTIC.
    // The returned session holds a snapshot reloaded from the store. Its
    // conversation contains the question and assembled response messages.
    // This does NOT drain the queue or send another request. We deliberately
    // print the saved answer again to distinguish storage from presentation.
    let final_conversation = match final_session.conversation() {
        Some(conversation) => conversation,
        None => return Err("the returned session has no conversation".into()),
    };
    println!(
        "\nFinal persisted conversation ({} message(s), handed back by run()):",
        final_conversation.len()
    );
    print_messages(final_conversation.messages());

    // "Persisted" here means saved in our map for the life of this process,
    // not written to disk. Once main exits, this experiment's store disappears.
    Ok(())
}

// DISPLAY PATH: turn queued message events into terminal text.
// This function never reads the provider stream; the runner already did that.
// It never updates the store either. Its only output is what the user sees.
fn drain_and_print_events(event_rx: &mut mpsc::Receiver<AgentEvent>) {
    // Track whether any text was printed so we can surround the entire answer
    // with one pair of delimiters, rather than fencing every little fragment.
    let mut streamed_text = false;

    // try_recv reads an available event without waiting. It returns an error
    // when the queue is empty (or disconnected), which ends this loop.
    // Here no more events are expected because machine.run already returned.
    // This would NOT be the correct wait-for-next-event loop for live display.
    while let Ok(event) = event_rx.try_recv() {
        // AgentEvent has several variants. Our normal runner response arrives
        // as Message events; this renderer intentionally ignores other variants.
        let AgentEvent::Message(message) = event else {
            continue;
        };

        // A message can carry more than plain text. Only its text content is
        // extracted for this minimal terminal display. Empty text is skipped.
        let text = message.as_concat_text();
        if text.is_empty() {
            continue;
        }
        if !streamed_text {
            print_delimiter();
        }

        // No newline per fragment: printing "Paris", " is the", " capital..."
        // consecutively displays one answer. We do not build an answer String
        // here; the runner separately assembled the saved message(s).
        print!("{}", text);
        streamed_text = true;
    }
    if streamed_text {
        println!();
        print_delimiter();
    }
}

// LOADED STATE: one snapshot of the conversation used for a machine pass.
// This is our application type, not a built-in GDK session implementation.
// It contains data, not a running task, an API connection, or a display queue.
struct ChatSession {
    conversation: Conversation,
}

// MachineSession is the contract that lets the machine read our session.
// A more elaborate application could put other per-session data in this type.
impl MachineSession for ChatSession {
    fn id(&self) -> &str {
        // All ChatSession values in this experiment refer to its one fixed ID.
        SESSION_ID
    }

    fn conversation(&self) -> Option<&Conversation> {
        // The interface permits a missing conversation, but our sessions
        // always have one. Some returns a borrowed view, not another clone.
        Some(&self.conversation)
    }
}

// STORED STATE: a map from session ID to conversation. It lives outside the
// machine so the machine can reload the latest state after applying effects.
// This store also supplies the machine's read/write interface itself; there
// is no separate ChatRuntime object or database service in this experiment.
struct InMemoryStore {
    // Mutex protects access to the map. We use short synchronous lock scopes
    // and do not hold this lock across an await or during a provider request.
    conversations: Mutex<HashMap<String, Conversation>>,
}

impl InMemoryStore {
    fn seeded(seed: Conversation) -> Self {
        // This initial write is done by main BEFORE the machine runs. The
        // later answer write is requested by a runner effect during the run.
        let mut conversations = HashMap::new();
        conversations.insert(SESSION_ID.to_string(), seed);
        InMemoryStore {
            conversations: Mutex::new(conversations),
        }
    }
}

// READ CONTRACT: machine.run calls this at the start of every pass and once
// more when returning the final session. The machine knows the contract,
// not our HashMap implementation. An async signature also accommodates real
// storage implementations that would need to wait for file or database I/O.
#[async_trait]
impl SessionLoader<ChatSession> for InMemoryStore {
    async fn load(&self, session_id: &str) -> anyhow::Result<ChatSession> {
        let conversations = self.conversations.lock().expect("store lock");
        let conversation = match conversations.get(session_id) {
            // clone makes a snapshot for the pass. Changing this snapshot
            // would not itself update the conversation stored in the map.
            Some(conversation) => conversation.clone(),
            None => {
                let failure = format!("no session '{}' exists in the store", session_id);
                return Err(anyhow::anyhow!(failure));
            }
        };
        // The map lock is released when this method returns; the provider
        // call happens later, with this owned conversation snapshot.
        Ok(ChatSession { conversation })
    }
}

// WRITE CONTRACT: after a step returns effects, machine.apply calls this.
// The runner specifies WHAT should change; this handler implements HOW our
// application records it. The generic machine need not understand ChatEffect.
#[async_trait]
impl EffectHandler<ChatSession, ChatEffect> for InMemoryStore {
    async fn apply_effects(
        &self,
        session: &ChatSession,
        effects: &mut [ChatEffect],
        // The contract also permits emitting display updates during saving.
        // Our handler does not do that; the underscore marks an unused argument.
        _emit: &Emitter,
    ) -> anyhow::Result<()> {
        // Handle effects in their returned order. For this normal response the
        // runner supplies usage (if reported), then accumulated response messages.
        for effect in effects {
            match effect {
                ChatEffect::AppendMessage(message) => {
                    // This is explanatory output, not printing the answer text.
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

                    // THIS LINE saves the answer in the application's state.
                    // It is not saved by printing, by emitting an event, or by
                    // changing the loaded session snapshot. Pass 2 reloads
                    // this updated stored conversation and sees the answer.
                    stored.push(message.clone());
                }
                ChatEffect::RecordUsage(usage) => {
                    // Usage is metadata such as the answering model and token
                    // counts. In this spike we only PRINT it; despite the output
                    // label saying "persisting", no usage record is stored.
                    // Optional counts are rendered as "-" when unavailable.
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

// SAVING INSTRUCTIONS: this is an application-defined enum. Each variant
// describes one kind of requested change and carries the data for that change.
// Creating a ChatEffect does not save anything; the handler above must apply it.
//
// Why supply our own enum? The default ConversationEffect in alpha.11 does not
// implement InferenceEffect (the runner's usage-recording contract). Our small
// vocabulary supports both the machine's ID contract and the runner's needs.
enum ChatEffect {
    // The runner converts its assembled response message(s) to this variant.
    AppendMessage(Message),
    // The runner converts provider usage metadata to this variant.
    RecordUsage(ProviderUsage),
}

// MACHINE CONTRACT: the machine asks effects to guarantee message IDs before
// handing them to the effect handler. IDs identify messages and also enable
// merging same-message streaming fragments; they are not the session's map key.
impl MachineEffect for ChatEffect {
    fn ensure_message_ids(&mut self) {
        // A usage effect has no Message, so it needs no message ID.
        // Messages emitted by the runner already have an ID, but this contract
        // also covers messages from other producers. Existing IDs are preserved.
        if let ChatEffect::AppendMessage(message) = self {
            // Rust detail: the builder method below consumes a Message and
            // returns a Message. `message` here is only a mutable reference.
            // Temporarily replace its value so we can take ownership, run the
            // builder, and put the result back. The temporary assistant message
            // is NOT emitted or saved; it is replaced on the next two lines.
            let taken = std::mem::replace(message, Message::assistant());
            let with_id = taken.with_generated_id_if_missing();
            *message = with_id;
        }
    }
}

// MESSAGE CONVERSION CONTRACT: the generic runner does not know our enum's
// variant names. It calls E::from(message); this implementation tells it how
// to wrap a Message as our application's append-message saving instruction.
impl From<Message> for ChatEffect {
    fn from(message: Message) -> Self {
        ChatEffect::AppendMessage(message)
    }
}

// RUNNER CONTRACT: similarly, the runner calls E::record_usage(usage) when
// provider usage is available. This wraps the metadata; it does not print or
// save it yet. Our effect handler decides what to do with it.
impl InferenceEffect for ChatEffect {
    fn record_usage(usage: ProviderUsage) -> Self {
        ChatEffect::RecordUsage(usage)
    }
}

// SETUP HELPERS: familiar message and configuration work, not machine steps.
fn seed_conversation() -> Conversation {
    let mut conversation = Conversation::empty();
    // Constructing a user Message just creates data; it sends no request.
    conversation.push(Message::user().with_text(USER_PROMPT));
    conversation
}

fn provider_config_path() -> Result<PathBuf, Box<dyn Error>> {
    // Use the first positional argument if provided, otherwise our default.
    // Relative paths resolve from the working directory, not this source folder.
    match env::args().nth(1) {
        Some(path) => Ok(PathBuf::from(path)),
        None => Ok(PathBuf::from(DEFAULT_PROVIDER_CONFIG)),
    }
}

fn first_configured_model(provider_config: &Value) -> Result<String, Box<dyn Error>> {
    // Select from our JSON configuration, not from a live model-list request.
    // Each match extracts one value or returns a clear setup failure.
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
    // Diagnostic display of stored/setup data; no network call or state change.
    for message in messages {
        println!("  [{:?}] {}", message.role, message.as_concat_text());
    }
}

fn print_delimiter() {
    println!("{}", PAYLOAD_DELIMITER);
}

// These tests check our local starting state and loading contract. They make
// no provider calls and do not validate inference, accumulation, or the full
// machine workflow. The documented cargo run is needed to observe that live.
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
