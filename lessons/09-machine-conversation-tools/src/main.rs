// Lesson 9
// local modules hold previously taught calculator code and supplied support.
mod calculator;
mod presentation;
mod provider_setup;
#[cfg(test)]
mod tests;

use calculator::{execute_allowed_tool, parse_price, tool_definition, LossArguments, TOOL_NAME};
use presentation::{
    display_session, print_delimiter, print_error, print_heading, print_status, print_warning,
    stdout_color_enabled, styled, TextStyle,
};
use provider_setup::{first_configured_model, provider_config_path};
use std::{
    collections::{HashMap, HashSet},
    fs,
    io::{self, Write},
    sync::Arc,
    time::Duration,
};

use async_trait::async_trait;
use goose_agent::{
    events::AgentEvent,
    inference::{InferenceEffect, InferenceRunner},
    machine::{EffectHandler, MachineSession, SessionLoader, StateMachine, Step},
    operation::{
        applied, messages_since_kickoff, not_applicable, yielded_with, Emitter, MachineEffect,
        Operation, OperationResult,
    },
    tool::{ToolOperation, ToolProvider},
};
use goose_providers::{
    base::Provider,
    conversation::{
        message::{Message, ToolRequest},
        token_usage::ProviderUsage,
        Conversation,
    },
    declarative::{from_json, EnvKeyResolver},
    model::ModelConfig,
};
use rmcp::model::{CallToolRequestParams, CallToolResult, ContentBlock, ErrorData, Tool};
use rust_decimal::Decimal;
use serde_json::{json, Value};
use tokio::sync::{mpsc, Mutex};
use tokio_util::sync::CancellationToken;

const SESSION_ID: &str = "lesson-09";
const USER_PROMPT: &str = "What is stop-loss referring to in day trading?";
const SHARE_COUNT: u32 = 200;
const SYSTEM_INSTRUCTION: &str = "You are a day-trading teaching assistant. Explain concepts directly. Use maximum_planned_loss when a hypothetical long position requires that calculation and the needed inputs are supplied. Do not invent missing inputs. Never claim to place, modify, or cancel a trade. Maximum planned loss excludes fees, slippage, and a gap through the stop; it is not expected statistical loss or a guaranteed ceiling on realized loss.";
const MAX_APPLIED_PASSES: usize = 8;
const MAX_TOOL_REQUESTS: usize = 4;
const RUN_TIMEOUT_SECONDS: u64 = 180;

#[tokio::main]
async fn main() {
    if let Err(error) = run_application().await {
        print_error(&format!("Error: {:#}", error));
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

    // Keep the same store, Session ID, machine, and capability across both turns.
    let runtime = ChatRuntime::seeded();
    let tools = SafeTools {
        inner: ToolOperation::new().with_provider(Arc::new(LossToolProvider)),
    };
    let runner = InferenceRunner::new(provider, model);
    let steps: Vec<Step<'_, ChatSession, ChatEffect>> = vec![
        Step::Operation(Arc::new(RunLimit)),
        Step::Operation(Arc::new(tools)),
        Step::Inference(Arc::new(runner)),
    ];
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
    let first_ok = run_turn(&machine, &runtime, &cancel, 1).await?;
    let first_finished = runtime.load(SESSION_ID).await?;
    // A new user turn is appended by the application, not by the run loop.
    let follow_up = format!(
        "A hypothetical long trade enters at $51.20, uses a stop at $50.70, and has {} shares. What is the maximum planned loss?",
        SHARE_COUNT,
    );
    runtime.append_user(&follow_up).await?;
    let second_started = runtime.load(SESSION_ID).await?;
    if second_started.conversation.messages()[..first_finished.conversation.len()]
        != *first_finished.conversation.messages()
    {
        return Err(anyhow::anyhow!("the first exchange was not retained"));
    }
    let second_ok = run_turn(&machine, &runtime, &cancel, 2).await?;
    if !first_ok || !second_ok {
        return Err(anyhow::anyhow!(
            "the observed tool-selection/result pattern did not validate Lesson 9; inspect both turns"
        ));
    }
    print_status("Both turns retained in one Session; structural scenario checks passed.");
    println!("{}", styled("Review the final explanation for the exclusions: fees, slippage, and gaps through the stop.", TextStyle::Yellow, stdout_color_enabled()));
    Ok(())
}

async fn run_turn(
    machine: &StateMachine<'_, ChatSession, ChatEffect>,
    runtime: &ChatRuntime,
    cancel: &CancellationToken,
    turn_number: usize,
) -> anyhow::Result<bool> {
    let before = runtime.load(SESSION_ID).await?;
    print_heading(&format!("\nTurn {}", turn_number));
    display_session("after appending input, before machine run", &before);
    print_heading("application → provider (retained conversation-history preview; not the final serialized request):");
    print_delimiter();
    println!("{}", presentation::conversation_history_preview(&before));
    print_delimiter();
    print_heading(
        "application → provider (new human input only; also included in the history preview):",
    );
    print_delimiter();
    let turn = messages_since_kickoff(&before.conversation)?;
    if let Some(input) = turn.first() {
        println!(
            "{}",
            styled(
                &input.as_concat_text(),
                TextStyle::Blue,
                stdout_color_enabled()
            )
        );
    }
    print_delimiter();
    print_heading("application → provider (instruction contribution):");
    print_delimiter();
    println!("{}", SYSTEM_INSTRUCTION);
    print_delimiter();
    print_heading("application → provider (advertised tool definition; not an execution request):");
    print_delimiter();
    println!("{:#?}", tool_definition());
    print_delimiter();
    let (event_tx, event_rx) = mpsc::channel::<AgentEvent>(32);
    let emit = Emitter::new(event_tx, cancel.clone());
    // The engine still owns the loop; the timeout bounds a stalled provider too.
    let run = async {
        let timed_result = tokio::time::timeout(
            Duration::from_secs(RUN_TIMEOUT_SECONDS),
            machine.run(runtime, SESSION_ID, &emit),
        )
        .await;
        let run_result = match timed_result {
            Ok(result) => result,
            Err(_elapsed) => {
                cancel.cancel();
                Err(anyhow::anyhow!(
                    "turn exceeded the {}-second execution bound",
                    RUN_TIMEOUT_SECONDS
                ))
            }
        };
        // Close the only sender on success or error; drain before viewing state.
        drop(emit);
        run_result
    };
    let (run_result, display_result) = tokio::join!(run, display_events(event_rx));
    let summary = match display_result {
        Ok(summary) => summary,
        Err(error) => return Err(error.into()),
    };
    // Even a failed run may have saved partial state: show it honestly.
    let saved = runtime.load(SESSION_ID).await?;
    display_session("AFTER", &saved);
    let final_session = match run_result {
        Ok(session) => session,
        Err(error) => return Err(error),
    };
    inspect_turn(&final_session, &summary, turn_number)
}

// A loaded Session is a snapshot, not a list of completed Operations.
#[derive(Clone)]
struct ChatSession {
    id: String,
    conversation: Conversation,
    usage: Vec<ProviderUsage>,
    applied_passes: usize,
    halt_reason: Option<String>,
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
            applied_passes: 0,
            halt_reason: None,
        };
        let mut sessions = HashMap::new();
        sessions.insert(SESSION_ID.to_string(), session);
        Self {
            sessions: Mutex::new(sessions),
        }
    }

    async fn append_user(&self, text: &str) -> anyhow::Result<()> {
        let mut sessions = self.sessions.lock().await;
        let stored = match sessions.get_mut(SESSION_ID) {
            Some(stored) => stored,
            None => return Err(anyhow::anyhow!("missing Session")),
        };
        stored.conversation.push(Message::user().with_text(text));
        // These bounds belong to a run/user turn, not the whole conversation.
        stored.applied_passes = 0;
        stored.halt_reason = None;
        Ok(())
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
        emit: &Emitter,
    ) -> anyhow::Result<()> {
        let mut sessions = self.sessions.lock().await;
        let stored = match sessions.get_mut(session.id()) {
            Some(stored) => stored,
            None => {
                let failure = format!("no Session '{}' exists in the store", session.id());
                return Err(anyhow::anyhow!(failure));
            }
        };
        let mut request_displays = Vec::new();
        for effect in effects {
            match effect {
                ChatEffect::AppendMessage(message) => {
                    stored.conversation.push(message.clone());
                    if !message.get_tool_request_ids().is_empty() {
                        // Display a complete saved request, not incremental argument fragments.
                        let mut display = message.clone();
                        display.metadata.set_operation_note(
                            "lesson-display",
                            "reconstructed_request",
                            json!(true),
                        );
                        request_displays.push(display);
                    }
                }
                ChatEffect::RecordUsage(usage) => stored.usage.push(usage.clone()),
                ChatEffect::Halt(reason) => stored.halt_reason = Some(reason.clone()),
            }
        }
        stored.applied_passes += 1;
        // Release the store before waiting on presentation backpressure.
        drop(sessions);
        for display in request_displays {
            // Same FIFO as tool results and the next inference's text Events.
            emit.emit(AgentEvent::Message(display)).await;
        }
        Ok(())
    }
}

// Inference returns complete-message and usage Effects, separate from display Events.
enum ChatEffect {
    AppendMessage(Message),
    RecordUsage(ProviderUsage),
    Halt(String),
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
    displayed_text: bool,
}

// This marker exists only on the presentation clone, never in saved protocol data.
fn is_reconstructed_request_display(message: &Message) -> bool {
    message
        .metadata
        .operation_note("lesson-display", "reconstructed_request")
        == Some(&json!(true))
}

async fn display_events(mut receiver: mpsc::Receiver<AgentEvent>) -> io::Result<EventSummary> {
    let mut summary = EventSummary::default();
    while let Some(event) = receiver.recv().await {
        match event {
            AgentEvent::Message(message) => {
                summary.message_events += 1;
                if is_reconstructed_request_display(&message) {
                    if summary.saw_text {
                        println!();
                        print_delimiter();
                        summary.saw_text = false;
                    }
                    for block in &message.content {
                        if let Some(request) = block.as_tool_request() {
                            println!(
                                "{}",
                                styled(
                                    "provider → application (reconstructed request, saved):",
                                    TextStyle::Magenta,
                                    stdout_color_enabled()
                                )
                            );
                            print_delimiter();
                            println!(
                                "{}",
                                styled(
                                    &format!("{:#?}", request),
                                    TextStyle::Magenta,
                                    stdout_color_enabled()
                                )
                            );
                            print_delimiter();
                        }
                    }
                    // Its text was already streamed; do not replay it.
                    continue;
                }
                if message.is_tool_response() {
                    if summary.saw_text {
                        println!();
                        print_delimiter();
                        summary.saw_text = false;
                    }
                    println!("{}", styled(
                        "application, as the tool → provider (response Event; saving is separate):",
                        TextStyle::Green, stdout_color_enabled()));
                    print_delimiter();
                    println!(
                        "{}",
                        styled(
                            &format!("{:#?}", message),
                            TextStyle::Green,
                            stdout_color_enabled()
                        )
                    );
                    print_delimiter();
                }
                let text = message.as_concat_text();
                for content in &message.content {
                    if content.as_text().is_none() {
                        summary.non_text_blocks += 1;
                    }
                }
                if !text.is_empty() {
                    if !summary.saw_text {
                        print_heading("provider → application (stream):");
                        print_delimiter();
                        summary.saw_text = true;
                    }
                    summary.displayed_text = true;
                    print!("{}", text);
                    io::stdout().flush()?;
                }
            }
            // Usage is saved through Effects; these other Events add no text here.
            AgentEvent::Usage(_)
            | AgentEvent::MessageUsage { .. }
            | AgentEvent::McpNotification(_)
            | AgentEvent::HistoryReplaced(_) => {}
        }
    }
    if summary.saw_text {
        println!();
        print_delimiter();
    }
    Ok(summary)
}

fn inspect_turn(
    session: &ChatSession,
    summary: &EventSummary,
    turn_number: usize,
) -> anyhow::Result<bool> {
    if let Some(reason) = &session.halt_reason {
        return Err(anyhow::anyhow!("run stopped at a safeguard: {}", reason));
    }
    let turn = messages_since_kickoff(&session.conversation)?;
    let mut requests = Vec::new();
    let mut responses = Vec::new();
    let mut saw_answer = false;
    for message in turn {
        if message.error_kind().is_some()
            || message.as_concat_text()
                == "The model returned an empty response. Please resend your message to continue."
        {
            return Err(anyhow::anyhow!(
                "saved inference diagnostic; inspect Session after"
            ));
        }
        for block in &message.content {
            if let Some(request) = block.as_tool_request() {
                requests.push(request);
            }
            if let Some(response) = block.as_tool_response() {
                responses.push(response);
            }
        }
    }
    if let Some(last) = turn.last() {
        saw_answer = last.role == rmcp::model::Role::Assistant
            && !last.as_concat_text().trim().is_empty()
            && last.get_tool_request_ids().is_empty();
    }
    print_heading(&format!(
        "Observed turn {}: {} tool request(s), {} tool response(s).",
        turn_number,
        requests.len(),
        responses.len()
    ));
    if !summary.displayed_text || !saw_answer {
        return Err(anyhow::anyhow!(
            "no ordinary final answer was both displayed and saved"
        ));
    }
    if turn_number == 1 {
        if !requests.is_empty() {
            print_warning("Unexpected model selection: the conceptual first turn requested a tool. Not a successful Lesson 9 scenario.");
            return Ok(false);
        }
        return Ok(true);
    }
    if requests.is_empty() {
        print_warning(
            "Unexpected model selection: the numeric second turn did not request the calculator.",
        );
        return Ok(false);
    }
    let mut valid = requests.len() == responses.len();
    let mut ids = HashSet::new();
    for request in &requests {
        if request.id.is_empty() || !ids.insert(request.id.as_str()) {
            print_error("Ambiguous request correlation ID");
            valid = false;
        }
    }
    for request in requests {
        let call = match &request.tool_call {
            Ok(call) => call,
            Err(error) => {
                print_error(&format!(
                    "Request {} was unparseable: {}",
                    request.id, error.message
                ));
                valid = false;
                continue;
            }
        };
        let expected_result = execute_allowed_tool(&Ok(call.clone()));
        let result = match expected_result {
            Ok(result) => result,
            Err(error) => {
                print_error(&format!(
                    "Request {} failed validation: {}",
                    request.id, error.message
                ));
                valid = false;
                continue;
            }
        };
        // Check the supplied scenario, not just a coincidentally identical answer.
        let args = match &call.arguments {
            Some(args) => args,
            None => {
                valid = false;
                continue;
            }
        };
        let parsed_args = serde_json::from_value::<LossArguments>(Value::Object(args.clone()));
        let args = match parsed_args {
            Ok(args) => args,
            Err(_error) => {
                valid = false;
                continue;
            }
        };
        let entry = match parse_price("entry_price", &args.entry_price) {
            Ok(entry) => entry,
            Err(failure) => return Err(anyhow::anyhow!(failure)),
        };
        let stop = match parse_price("stop_price", &args.stop_price) {
            Ok(stop) => stop,
            Err(failure) => return Err(anyhow::anyhow!(failure)),
        };
        if entry != Decimal::new(5120, 2)
            || stop != Decimal::new(5070, 2)
            || args.share_count != SHARE_COUNT
        {
            print_error("Unexpected calculator inputs; compare the saved request to the user's numeric question.");
            valid = false;
        }
        let expected_loss = format!("${:.2}", Decimal::new(50, 2) * Decimal::from(SHARE_COUNT));
        let expected_payload = CallToolResult::success(vec![ContentBlock::text(expected_loss)]);
        if result != expected_payload {
            print_error("Unexpected deterministic amount");
            valid = false;
        }
        let mut matching = Vec::new();
        for response in &responses {
            if response.id == request.id {
                matching.push(*response);
            }
        }
        if matching.len() != 1 {
            print_error(&format!(
                "Request {} does not have exactly one correlated response",
                request.id
            ));
            valid = false;
            continue;
        }
        if matching[0].tool_result != Ok(result) {
            print_error(&format!(
                "Request {} has an unexpected saved result",
                request.id
            ));
            valid = false;
        }
    }
    Ok(valid)
}

// The typed domain shape remains LossArguments; this provider reuses Lesson 6 validation.
struct LossToolProvider;

#[async_trait]
impl ToolProvider<ChatSession> for LossToolProvider {
    async fn tools(&self, _session: &ChatSession) -> anyhow::Result<Vec<Tool>> {
        Ok(vec![tool_definition()])
    }

    async fn call(
        &self,
        _session: &ChatSession,
        _request_id: &str,
        call: CallToolRequestParams,
        _emit: &Emitter,
    ) -> Result<CallToolResult, ErrorData> {
        execute_allowed_tool(&Ok(call))
    }
}

// The shipped operation skips unknown names and has no request bound.
// Answer rejected calls first; on re-evaluation the shipped operation dispatches the rest.
struct SafeTools {
    inner: ToolOperation<ChatSession>,
}

#[async_trait]
impl Operation<ChatSession, ChatEffect> for SafeTools {
    fn name(&self) -> &'static str {
        "tools"
    }

    async fn inference_tools(&self, session: &ChatSession) -> anyhow::Result<Vec<Tool>> {
        <ToolOperation<ChatSession> as Operation<ChatSession, ChatEffect>>::inference_tools(
            &self.inner,
            session,
        )
        .await
    }

    async fn prompt_parts(
        &self,
        _session: &ChatSession,
        _conversation: &Conversation,
    ) -> anyhow::Result<Vec<(String, String)>> {
        Ok(vec![(
            "education".to_string(),
            SYSTEM_INSTRUCTION.to_string(),
        )])
    }

    async fn run(
        &self,
        session: &ChatSession,
        conversation: &Conversation,
        emit: &Emitter,
    ) -> anyhow::Result<OperationResult<ChatEffect>> {
        let turn = messages_since_kickoff(conversation)?;
        let requests = turn_requests(turn);
        let mut seen = HashSet::new();
        for request in &requests {
            if request.id.is_empty() || !seen.insert(request.id.as_str()) {
                let failure = "empty or duplicate correlation ID; cannot safely correlate results"
                    .to_string();
                return yielded_with([ChatEffect::Halt(failure)]);
            }
        }
        let mut answered = HashSet::new();
        for message in turn {
            for id in message.get_tool_response_ids() {
                answered.insert(id);
            }
        }
        let mut rejected = Message::user();
        for (index, request) in requests.iter().enumerate() {
            if answered.contains(request.id.as_str()) || request.was_executed_externally() {
                continue;
            }
            let failure = if index >= MAX_TOOL_REQUESTS {
                Some(format!(
                    "request limit is {} per user turn; this request was not executed",
                    MAX_TOOL_REQUESTS
                ))
            } else {
                match &request.tool_call {
                    Ok(call) => {
                        if call.name.as_ref() != TOOL_NAME {
                            Some(format!(
                                "unknown tool '{}'; only '{}' is allowed",
                                call.name, TOOL_NAME
                            ))
                        } else {
                            None
                        }
                    }
                    // The SDK returns correlated parse errors itself.
                    Err(_error) => None,
                }
            };
            if let Some(failure) = failure {
                rejected.add_tool_response_with_metadata(
                    request.id.clone(),
                    Err(ErrorData::invalid_params(failure, None)),
                    request.metadata.as_ref(),
                );
            }
        }
        if !rejected.content.is_empty() {
            let response = emit.message(rejected).await;
            return applied([ChatEffect::from(response)]);
        }
        <ToolOperation<ChatSession> as Operation<ChatSession, ChatEffect>>::run(
            &self.inner,
            session,
            conversation,
            emit,
        )
        .await
    }
}

fn turn_requests(messages: &[Message]) -> Vec<&ToolRequest> {
    let mut requests = Vec::new();
    for message in messages {
        for block in &message.content {
            if let Some(request) = block.as_tool_request() {
                requests.push(request);
            }
        }
    }
    requests
}

// Count applied batches, not Events or messages. Normal completion wins over the cap.
struct RunLimit;

#[async_trait]
impl Operation<ChatSession, ChatEffect> for RunLimit {
    fn name(&self) -> &'static str {
        "run-limit"
    }

    async fn run(
        &self,
        session: &ChatSession,
        conversation: &Conversation,
        _emit: &Emitter,
    ) -> anyhow::Result<OperationResult<ChatEffect>> {
        let turn = messages_since_kickoff(conversation)?;
        let requests = turn_requests(turn);
        let mut answered = HashSet::new();
        for message in turn {
            for id in message.get_tool_response_ids() {
                answered.insert(id);
            }
        }
        let all_answered = requests
            .iter()
            .all(|request| answered.contains(request.id.as_str()));
        if requests.len() > MAX_TOOL_REQUESTS && all_answered {
            return yielded_with([ChatEffect::Halt(format!(
                "model exceeded {} requests; excess requests received errors",
                MAX_TOOL_REQUESTS,
            ))]);
        }
        let finished = goose_agent::operation::ends_turn(turn) && all_answered;
        if session.applied_passes >= MAX_APPLIED_PASSES && !finished {
            return yielded_with([ChatEffect::Halt(format!(
                "reached {} applied passes",
                MAX_APPLIED_PASSES,
            ))]);
        }
        not_applicable()
    }
}
