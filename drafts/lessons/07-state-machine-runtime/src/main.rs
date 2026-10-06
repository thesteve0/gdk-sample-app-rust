//! Lesson 7: state-machine concepts over a minimal in-memory runtime.
//!
//! The Lesson 6 work was coordinated by a hand-written loop: exactly two
//! inference rounds, then stop. This lesson hands that coordination to the GDK
//! state machine: an ordered list of steps, each re-deriving its decision from
//! the persisted conversation on every pass. There is no provider call here;
//! the session is seeded with the same conversation Lesson 6 reconstructed,
//! so the machine trace is deterministic and reproducible in class.

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use goose_agent::{
    events::AgentEvent,
    machine::{EffectHandler, MachineSession, SessionLoader, StateMachine},
    operation::{
        applied, last_effective_role, not_applicable, yielded, ConversationEffect, Emitter,
        Operation, OperationResult,
    },
};
use goose_providers::conversation::{
    effective_role,
    message::{Message, ToolRequest, ToolResult},
    Conversation, EffectiveRole,
};
use rmcp::model::{CallToolRequestParams, CallToolResult, ContentBlock, ErrorData, JsonObject};
use rust_decimal::Decimal;
use serde::Deserialize;
use serde_json::{json, Value};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use goose_agent::machine::Step;

const SESSION_ID: &str = "lesson-07";
const TOOL_NAME: &str = "maximum_planned_loss";
/// The state-step bound: the counterpart of Lesson 6's round bound. The
/// machine stops after this many applied steps even if a step would keep
/// applying. The deterministic scenario needs two passes.
const MAX_STATE_STEPS: u32 = 3;

/// A delimiter printed before and after every block of protocol payload. This
/// lesson receives no provider traffic, but the seeded assistant tool request
/// is protocol-shaped (it is exactly what the provider sent in Lesson 6), so
/// it is fenced and labeled the same way.
const PAYLOAD_DELIMITER: &str = "++++++++";

const SYSTEM_INSTRUCTION: &str = "You are a day-trading teaching assistant. For the supplied hypothetical trade, use the maximum_planned_loss tool. Never claim to place, modify, or cancel a trade.";
const USER_PROMPT: &str = "A hypothetical long trade enters at $51.20, uses a stop at $50.70, and has 200 shares. What is the maximum planned loss?";
fn main() {
    // A single-threaded runtime is enough: this lesson makes no provider calls.
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("tokio runtime")
        .block_on(run());
}

async fn run() {
    println!("── Lesson 7: the GDK state machine over the Lesson 6 conversation ──");

    // The store is the persistence layer: an in-memory map from session id to
    // conversation. Lesson 8 keeps this shape; a later lesson could replace it
    // with files or a database without touching the machine.
    let store = Arc::new(InMemoryStore::seeded());

    let seeded = store.conversations.lock().unwrap().get(SESSION_ID).cloned();
    let seeded_conversation = match seeded {
        Some(conversation) => conversation,
        None => {
            println!("the seed conversation was not found; nothing to do");
            return;
        }
    };
    print_seeded_conversation(&seeded_conversation);

    // The machine owns the ordered steps and the cancellation token. Step
    // order matters: tool requests must be answered before the yield step is
    // even consulted.
    let steps: Vec<Step<'_, TradingSession>> = vec![
        Step::Operation(Arc::new(AnswerPendingTools)),
        Step::Operation(Arc::new(YieldToClient)),
    ];
    let cancel = CancellationToken::new();
    let machine = StateMachine::new(steps, cancel.clone());

    // The emitter streams events to the client while a step runs. This
    // application is the client, so it drains the channel between passes.
    let (event_tx, mut event_rx) = mpsc::channel::<AgentEvent>(64);
    let emit = Emitter::new(event_tx, cancel);

    println!("\n── Running the machine (bounded loop over step + apply) ──");
    let outcome = run_state_machine(&machine, &store, &emit, &mut event_rx, MAX_STATE_STEPS).await;

    // The final persisted state: what a reload would see after the run.
    println!("\n── Final persisted conversation (what the next reload sees) ──");
    let final_messages = store
        .conversations
        .lock()
        .unwrap()
        .get(SESSION_ID)
        .map(|conversation| conversation.messages().len());
    match final_messages {
        Some(count) => println!("  session '{}' holds {} message(s)", SESSION_ID, count),
        None => println!("  session '{}' is no longer present", SESSION_ID),
    }
    let _ = outcome;
}

/// The hand-written loop that mirrors `StateMachine::run`, minus the fact
/// that `run` has no application-specific step bound. Every pass:
/// reload the session, run one step, persist the effects, repeat. The machine
/// never caches the session; the loader returns a fresh view every pass.
async fn run_state_machine(
    machine: &StateMachine<'_, TradingSession>,
    store: &InMemoryStore,
    emit: &Emitter,
    event_rx: &mut mpsc::Receiver<AgentEvent>,
    max_steps: u32,
) -> RunOutcome {
    let mut applied_steps = 0;
    loop {
        if applied_steps >= max_steps {
            println!(
                "\n  state-step bound reached after {} applied step(s); the machine stops",
                applied_steps
            );
            return RunOutcome::StepBoundReached;
        }

        let session = store.load(SESSION_ID).await.expect("session load");
        println!(
            "\n  ── Pass {}: reload session '{}' ──",
            applied_steps + 1,
            session.id()
        );

        let step_result = machine.step(&session, emit).await.expect("machine step");
        let mut result = match step_result {
            Some(result) => result,
            None => {
                println!("  no operation applies: the machine stops");
                return RunOutcome::StoppedNoOperation;
            }
        };
        match result.applied_step {
            Some(name) => println!("  step '{}' applied", name),
            None => println!("  a step applied"),
        }

        machine
            .apply(store, &session, &mut result, emit)
            .await
            .expect("apply effects");

        applied_steps += 1;

        // Events the step emitted while it ran: this application is the client.
        while let Ok(event) = event_rx.try_recv() {
            if let AgentEvent::Message(message) = event {
                println!("  event: message appended (id {:?})", message.id);
            }
        }

        if result.yield_to_client {
            println!("  a step yielded control to the client: the machine stops");
            return RunOutcome::YieldedToClient;
        }
    }
}

enum RunOutcome {
    StoppedNoOperation,
    YieldedToClient,
    StepBoundReached,
}

/// The in-memory session: an id plus its conversation. `MachineSession` is the
/// trait the machine reads; `conversation()` returns the state a step reasons
/// over, and `id()` is the key the loader reloads by.
struct TradingSession {
    id: String,
    conversation: Conversation,
}

impl MachineSession for TradingSession {
    fn id(&self) -> &str {
        &self.id
    }

    fn conversation(&self) -> Option<&Conversation> {
        Some(&self.conversation)
    }
}

/// The persistence layer. `SessionLoader` produces a fresh session view for
/// every pass, and `EffectHandler` writes effects back — the only way a step
/// changes the world.
struct InMemoryStore {
    conversations: Mutex<HashMap<String, Conversation>>,
}

impl InMemoryStore {
    /// Seed the store with the conversation Lesson 6 reconstructed: the user
    /// message and the assistant's structured tool request. Built in code so
    /// the machine trace is deterministic; Lesson 8 reattaches the provider.
    fn seeded() -> Self {
        let mut conversations = HashMap::new();
        conversations.insert(SESSION_ID.to_string(), seed_conversation());
        InMemoryStore {
            conversations: Mutex::new(conversations),
        }
    }
}

#[async_trait]
impl SessionLoader<TradingSession> for InMemoryStore {
    async fn load(&self, session_id: &str) -> anyhow::Result<TradingSession> {
        let loaded = self.conversations.lock().unwrap().get(session_id).cloned();
        let conversation = match loaded {
            Some(conversation) => conversation,
            None => {
                let failure = format!("session '{}' is not in the store", session_id);
                return Err(anyhow::anyhow!(failure));
            }
        };
        Ok(TradingSession {
            id: session_id.to_string(),
            conversation,
        })
    }
}

#[async_trait]
impl EffectHandler<TradingSession, ConversationEffect> for InMemoryStore {
    async fn apply_effects(
        &self,
        session: &TradingSession,
        effects: &mut [ConversationEffect],
        _emit: &Emitter,
    ) -> anyhow::Result<()> {
        let mut conversation = session.conversation().cloned().ok_or_else(|| {
            anyhow::anyhow!("session '{}' loaded without conversation", session.id())
        })?;
        for effect in effects {
            match effect {
                ConversationEffect::AppendMessage(message) => {
                    let role = effective_role(message);
                    println!(
                        "  persisting effect: append message (effective role {:?})",
                        role
                    );
                    conversation.push(message.clone());
                }
                ConversationEffect::ReplaceConversation(_) => {
                    return Err(anyhow::anyhow!(
                        "this runtime only applies AppendMessage effects"
                    ));
                }
                ConversationEffect::PatchToolRequestMeta { .. } => {
                    return Err(anyhow::anyhow!(
                        "this runtime only applies AppendMessage effects"
                    ));
                }
                ConversationEffect::SetMessageVisibility { .. } => {
                    return Err(anyhow::anyhow!(
                        "this runtime only applies AppendMessage effects"
                    ));
                }
            }
        }
        self.conversations
            .lock()
            .unwrap()
            .insert(session.id().to_string(), conversation);
        Ok(())
    }
}

/// Step 1: answer exactly the tool requests that have no matching response in
/// the persisted conversation. This re-derivation is the heart of the state
/// machine: on pass 1 the seeded request is unanswered, so the step applies;
/// on pass 2 the response is in history, so the step declines.
struct AnswerPendingTools;

#[async_trait]
impl Operation<TradingSession> for AnswerPendingTools {
    fn name(&self) -> &'static str {
        "answer_pending_tools"
    }

    async fn run(
        &self,
        _session: &TradingSession,
        conversation: &Conversation,
        _emit: &Emitter,
    ) -> anyhow::Result<OperationResult<ConversationEffect>> {
        let unanswered = unanswered_tool_requests(conversation);
        if unanswered.is_empty() {
            return not_applicable();
        }
        let mut effects = Vec::new();
        for request in unanswered {
            let result = execute_allowed_tool(&request.tool_call);
            let id = request.id.clone();
            print_tool_result(&id, &result);
            effects.push(ConversationEffect::AppendMessage(
                Message::user().with_tool_response(id, result),
            ));
        }
        applied(effects)
    }
}

/// Step 2: a placeholder marking the gap Lesson 8 fills. After a tool result
/// reaches the conversation, a real agent would call inference again; this
/// lesson has no inference step, so the machine yields control to the client
/// instead. The step only fires when the conversation ends in a tool response.
struct YieldToClient;

#[async_trait]
impl Operation<TradingSession> for YieldToClient {
    fn name(&self) -> &'static str {
        "yield_to_client"
    }

    async fn run(
        &self,
        _session: &TradingSession,
        conversation: &Conversation,
        _emit: &Emitter,
    ) -> anyhow::Result<OperationResult<ConversationEffect>> {
        let last_role = match last_effective_role(conversation.messages()) {
            Ok(role) => role,
            Err(_) => return not_applicable(),
        };
        if matches!(last_role, EffectiveRole::Tool) {
            return yielded();
        }
        not_applicable()
    }
}

/// Re-derive "unanswered" from the persisted conversation: a request counts
/// only when no response in history carries its id. Lesson 6 collected every
/// request because round 1 was the only round; the machine must re-check
/// on every pass.
fn unanswered_tool_requests(conversation: &Conversation) -> Vec<&ToolRequest> {
    let mut answered = Vec::new();
    for message in conversation.messages() {
        for block in &message.content {
            if let Some(response) = block.as_tool_response() {
                answered.push(response.id.clone());
            }
        }
    }
    let mut requests = Vec::new();
    for message in conversation.messages() {
        for block in &message.content {
            if let Some(request) = block.as_tool_request() {
                if !answered.contains(&request.id) {
                    requests.push(request);
                }
            }
        }
    }
    requests
}

fn seed_conversation() -> Conversation {
    let mut conversation = Conversation::empty();
    conversation.push(Message::user().with_text(USER_PROMPT));

    // The request the provider actually sent in Lesson 6, replayed in code.
    // The id is fixed so the trace is reproducible across runs.
    let raw_arguments = json!({
        "entry_price": "51.20",
        "stop_price": "50.70",
        "share_count": 200
    });
    let arguments = match raw_arguments {
        Value::Object(object) => object,
        _ => JsonObject::new(),
    };
    let params = CallToolRequestParams::new(TOOL_NAME).with_arguments(arguments);
    conversation.push(Message::assistant().with_tool_request("call_seed_001", Ok(params)));
    conversation
}

fn print_seeded_conversation(conversation: &Conversation) {
    println!(
        "\n── Seeded session '{}' (replayed Lesson 6 conversation) ──",
        SESSION_ID
    );
    println!("  system instruction (application → provider, replayed):");
    print_delimiter();
    print_indented(SYSTEM_INSTRUCTION, "    > ");
    print_delimiter();
    println!("  assistant tool request (provider → application, replayed from Lesson 6):");
    for message in conversation.messages() {
        let role = effective_role(message);
        for block in &message.content {
            if let Some(request) = block.as_tool_request() {
                let name = match &request.tool_call {
                    Ok(params) => params.name.to_string(),
                    Err(_) => "(unparseable call)".to_string(),
                };
                println!(
                    "    role={:?} request id={} tool={}",
                    role, request.id, name
                );
            }
        }
    }
}

fn print_tool_result(id: &str, result: &ToolResult<CallToolResult>) {
    match result {
        Ok(response) => {
            let mut text = String::new();
            for block in &response.content {
                if let Some(content) = block.as_text() {
                    text.push_str(&content.text);
                }
            }
            println!(
                "  ✓ request {} → deterministic result (application, as the tool → provider)",
                id
            );
            print_delimiter();
            print_indented(&text, "    ");
            print_delimiter();
        }
        Err(error) => {
            println!(
                "  ✗ request {} → error result (application, as the tool → provider)",
                id
            );
            print_delimiter();
            print_indented(&error.message, "    ");
            print_delimiter();
        }
    }
}

fn print_delimiter() {
    println!("{}", PAYLOAD_DELIMITER);
}

fn print_indented(text: &str, prefix: &str) {
    for line in text.lines() {
        println!("{}{}", prefix, line);
    }
}

// ---------------------------------------------------------------------------
// Tool definition, validation, and execution: unchanged from Lesson 6. The
// machine coordinates the conversation; the boundaries still guard execution.
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LossArguments {
    entry_price: String,
    stop_price: String,
    share_count: u32,
}

fn parse_price(field: &str, raw: &str) -> Result<Decimal, String> {
    let parse_result = Decimal::from_str_exact(raw);
    let value = match parse_result {
        Ok(parsed_price) => parsed_price,
        Err(_parse_error) => {
            let failure = format!("{} '{}' is not a plain decimal number", field, raw);
            return Err(failure);
        }
    };
    if value.scale() > 4 {
        return Err(format!(
            "{} '{}' has more than four decimal places",
            field, raw
        ));
    }
    if value <= Decimal::ZERO {
        return Err(format!("{} must be a positive dollar amount", field));
    }
    Ok(value)
}

fn execute_maximum_planned_loss(arguments: &JsonObject) -> Result<String, String> {
    let deserialized = serde_json::from_value(Value::Object(arguments.clone()));
    let args: LossArguments = match deserialized {
        Ok(parsed) => parsed,
        Err(error) => {
            let failure = format!("arguments do not match the advertised schema: {}", error);
            return Err(failure);
        }
    };

    let entry = parse_price("entry_price", &args.entry_price)?;
    let stop = parse_price("stop_price", &args.stop_price)?;
    if args.share_count == 0 {
        return Err("share_count must be a positive whole number".to_string());
    }
    if entry <= stop {
        return Err(format!(
            "entry_price {} must be above stop_price {} for a long position",
            entry, stop
        ));
    }

    let shares = Decimal::from(args.share_count);
    let loss = (entry - stop) * shares;
    Ok(format!("${:.2}", loss))
}

fn execute_allowed_tool(
    tool_call: &ToolResult<CallToolRequestParams>,
) -> ToolResult<CallToolResult> {
    let parsed_call = match tool_call {
        Ok(call) => call,
        Err(error) => {
            let failure = format!("the tool call could not be parsed: {}", error.message);
            return Err(ErrorData::invalid_params(failure, None));
        }
    };

    let requested_name = parsed_call.name.as_ref();
    if requested_name != TOOL_NAME {
        let failure = format!(
            "tool '{}' is not on this application's allowlist; '{}' is the only executable tool",
            requested_name, TOOL_NAME
        );
        return Err(ErrorData::invalid_params(failure, None));
    }

    let arguments = match &parsed_call.arguments {
        Some(arguments) => arguments,
        None => {
            return Err(ErrorData::invalid_params(
                "the tool call carried no arguments".to_string(),
                None,
            ));
        }
    };

    match execute_maximum_planned_loss(arguments) {
        Ok(loss) => Ok(CallToolResult::success(vec![ContentBlock::text(loss)])),
        Err(failure) => Err(ErrorData::invalid_params(failure, None)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn call_with(arguments: Value) -> ToolResult<CallToolRequestParams> {
        let object = match arguments {
            Value::Object(object) => object,
            _ => JsonObject::new(),
        };
        Ok(CallToolRequestParams::new(TOOL_NAME).with_arguments(object))
    }

    #[test]
    fn executes_the_exact_scenario() {
        let result = execute_allowed_tool(&call_with(json!({
            "entry_price": "51.20", "stop_price": "50.70", "share_count": 200
        })));
        let response = match result {
            Ok(response) => response,
            Err(error) => panic!("expected success, got {}", error.message),
        };
        let text = match &response.content[0].as_text() {
            Some(content) => content.text.clone(),
            None => panic!("expected a text block"),
        };
        assert_eq!(text, "$100.00");
    }

    #[test]
    fn rejects_a_non_allowlisted_name() {
        let mut params = call_with(json!({
            "entry_price": "51.20", "stop_price": "50.70", "share_count": 200
        }))
        .unwrap();
        params.name = "other_tool".into();
        let failure = match execute_allowed_tool(&Ok(params)) {
            Err(error) => error.message,
            Ok(_) => panic!("expected an allowlist rejection"),
        };
        assert!(failure.contains("allowlist"), "unexpected: {}", failure);
    }

    #[test]
    fn rejects_unknown_fields_before_any_calculation() {
        let failure = match execute_allowed_tool(&call_with(json!({
            "entry_price": "51.20", "stop_price": "50.70", "share_count": 200,
            "extra": true
        }))) {
            Err(error) => error.message,
            Ok(_) => panic!("expected a schema rejection"),
        };
        assert!(failure.contains("schema"), "unexpected: {}", failure);
    }

    #[test]
    fn rejects_an_inverted_trade() {
        let failure = match execute_allowed_tool(&call_with(json!({
            "entry_price": "50.70", "stop_price": "51.20", "share_count": 200
        }))) {
            Err(error) => error.message,
            Ok(_) => panic!("expected a domain rejection"),
        };
        assert!(
            failure.contains("above stop_price"),
            "unexpected: {}",
            failure
        );
    }

    #[tokio::test]
    async fn machine_answers_the_seeded_request_and_yields() {
        let store = Arc::new(InMemoryStore::seeded());
        let steps: Vec<Step<'_, TradingSession>> = vec![
            Step::Operation(Arc::new(AnswerPendingTools)),
            Step::Operation(Arc::new(YieldToClient)),
        ];
        let cancel = CancellationToken::new();
        let machine = StateMachine::new(steps, cancel.clone());
        let (event_tx, mut event_rx) = mpsc::channel::<AgentEvent>(64);
        let emit = Emitter::new(event_tx, cancel);

        let outcome =
            run_state_machine(&machine, &store, &emit, &mut event_rx, MAX_STATE_STEPS).await;

        match outcome {
            RunOutcome::YieldedToClient => {}
            other => panic!("expected a yield, got {:?}", discriminant_of(&other)),
        }
        let conversation = store
            .conversations
            .lock()
            .unwrap()
            .get(SESSION_ID)
            .cloned()
            .unwrap();
        let unanswered = unanswered_tool_requests(&conversation);
        assert!(unanswered.is_empty(), "the request must be answered");

        // The tool result payload lives inside a ToolResponse content block,
        // not a Text block, so as_concat_text() cannot see it. Extract the
        // result text by hand and assert the deterministic dollar figure.
        let mut result_text = String::new();
        for message in conversation.messages() {
            for block in &message.content {
                let maybe_response = block.as_tool_response();
                if let Some(response) = maybe_response {
                    let parsed_result = match &response.tool_result {
                        Ok(payload) => payload,
                        Err(_) => panic!("the seeded request must produce an Ok result"),
                    };
                    for result_block in &parsed_result.content {
                        if let Some(text_block) = result_block.as_text() {
                            result_text.push_str(&text_block.text);
                        }
                    }
                }
            }
        }
        assert!(
            result_text.contains("$100.00"),
            "unexpected tool result text: {}",
            result_text
        );
    }

    #[tokio::test]
    async fn machine_stops_when_no_operation_applies() {
        let mut conversations = HashMap::new();
        let mut conversation = seed_conversation();
        let arguments = match json!({"entry_price": "51.20", "stop_price": "50.70", "share_count": 200})
        {
            Value::Object(object) => object,
            _ => JsonObject::new(),
        };
        let params = CallToolRequestParams::new(TOOL_NAME).with_arguments(arguments);
        let result = execute_allowed_tool(&Ok(params));
        conversation.push(Message::user().with_tool_response("call_seed_001", result));
        conversations.insert(SESSION_ID.to_string(), conversation);
        let store = Arc::new(InMemoryStore {
            conversations: Mutex::new(conversations),
        });

        let steps: Vec<Step<'_, TradingSession>> =
            vec![Step::Operation(Arc::new(AnswerPendingTools))];
        let cancel = CancellationToken::new();
        let machine = StateMachine::new(steps, cancel.clone());
        let (event_tx, mut event_rx) = mpsc::channel::<AgentEvent>(64);
        let emit = Emitter::new(event_tx, cancel);

        let outcome =
            run_state_machine(&machine, &store, &emit, &mut event_rx, MAX_STATE_STEPS).await;
        assert!(matches!(outcome, RunOutcome::StoppedNoOperation));
    }

    /// A step that always applies would loop forever without the bound. The
    /// hand-written loop must stop at the limit — the Lesson 6 round bound's
    /// state-machine counterpart.
    #[tokio::test]
    async fn state_step_bound_holds() {
        struct AlwaysApplies;
        #[async_trait]
        impl Operation<TradingSession> for AlwaysApplies {
            fn name(&self) -> &'static str {
                "always_applies"
            }
            async fn run(
                &self,
                _session: &TradingSession,
                _conversation: &Conversation,
                _emit: &Emitter,
            ) -> anyhow::Result<OperationResult<ConversationEffect>> {
                applied(Vec::new())
            }
        }

        let store = Arc::new(InMemoryStore::seeded());
        let steps: Vec<Step<'_, TradingSession>> = vec![Step::Operation(Arc::new(AlwaysApplies))];
        let cancel = CancellationToken::new();
        let machine = StateMachine::new(steps, cancel.clone());
        let (event_tx, mut event_rx) = mpsc::channel::<AgentEvent>(64);
        let emit = Emitter::new(event_tx, cancel);

        let outcome = run_state_machine(&machine, &store, &emit, &mut event_rx, 2).await;
        assert!(matches!(outcome, RunOutcome::StepBoundReached));
    }

    fn discriminant_of(outcome: &RunOutcome) -> &'static str {
        match outcome {
            RunOutcome::StoppedNoOperation => "stopped_no_operation",
            RunOutcome::YieldedToClient => "yielded",
            RunOutcome::StepBoundReached => "bound",
        }
    }
}
