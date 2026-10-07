use std::{
    collections::{HashMap, HashSet},
    env, fs,
    io::{self, IsTerminal, Write},
    path::PathBuf,
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
        message::{Message, ToolRequest, ToolResult},
        token_usage::ProviderUsage,
        Conversation,
    },
    declarative::{from_json, EnvKeyResolver},
    model::ModelConfig,
};
use rmcp::model::{
    CallToolRequestParams, CallToolResult, ContentBlock, ErrorData, JsonObject, Tool,
};
use rust_decimal::Decimal;
use serde::Deserialize;
use serde_json::{json, Value};
use tokio::sync::{mpsc, Mutex};
use tokio_util::sync::CancellationToken;

const DEFAULT_PROVIDER_CONFIG: &str = "custom_aa_llama_qwen3_6-35b.json";
const SESSION_ID: &str = "lesson-09";
const USER_PROMPT: &str = "What is stop-loss referring to in day trading?";
const SHARE_COUNT: u32 = 200;
const TOOL_NAME: &str = "maximum_planned_loss";
const SYSTEM_INSTRUCTION: &str = "You are a day-trading teaching assistant. Explain concepts directly. Use maximum_planned_loss when a hypothetical long position requires that calculation and the needed inputs are supplied. Do not invent missing inputs. Never claim to place, modify, or cancel a trade. Maximum planned loss excludes fees, slippage, and a gap through the stop; it is not expected statistical loss or a guaranteed ceiling on realized loss.";
const MAX_APPLIED_PASSES: usize = 8;
const MAX_TOOL_REQUESTS: usize = 4;
const RUN_TIMEOUT_SECONDS: u64 = 180;
const PAYLOAD_DELIMITER: &str = "++++++++";

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
    display_session("BEFORE", &before);
    print_heading("application → provider (new human input):");
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
    print_heading("application → provider (instruction and available capability):");
    print_delimiter();
    println!("{}", SYSTEM_INSTRUCTION);
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
    let bounds = format!(
        "\napplied_passes: {:?}\nhalt_reason: {:?}\n\n\n",
        session.applied_passes, session.halt_reason
    );
    let displayed_conversation = gold_session_attributes(&conversation, color_enabled);
    let gold_usage = gold_session_attributes(&usage, color_enabled);
    let displayed_usage = styled(&gold_usage, TextStyle::Dim, color_enabled);
    let displayed_bounds = gold_session_attributes(&bounds, color_enabled);
    format!(
        "{}{}{}",
        displayed_conversation, displayed_usage, displayed_bounds
    )
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

// Terminal styling is presentation only; redirected transcripts contain no ANSI codes.
#[derive(Clone, Copy)]
enum TextStyle {
    Heading,
    Blue,
    Magenta,
    Green,
    Yellow,
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
        TextStyle::Blue => "34",
        TextStyle::Magenta => "35",
        TextStyle::Green => "32",
        TextStyle::Yellow => "33",
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

fn print_status(text: &str) {
    println!("{}", styled(text, TextStyle::Green, stdout_color_enabled()));
}

fn print_warning(text: &str) {
    let enabled = terminal_color_enabled(io::stderr().is_terminal());
    eprintln!("{}", styled(text, TextStyle::Yellow, enabled));
}

fn print_error(text: &str) {
    let enabled = terminal_color_enabled(io::stderr().is_terminal());
    eprintln!("{}", styled(text, TextStyle::Error, enabled));
}

fn print_delimiter() {
    println!("{}", PAYLOAD_DELIMITER);
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

fn execute_allowed_tool(
    tool_call: &ToolResult<CallToolRequestParams>,
) -> ToolResult<CallToolResult> {
    // Parse boundary: model output can already carry a protocol error.
    let parsed_call = match tool_call {
        Ok(call) => call,
        Err(error) => {
            let failure = format!("the tool call could not be parsed: {}", error.message);
            return Err(ErrorData::invalid_params(failure, None));
        }
    };

    // Capability boundary: only the advertised calculator may execute.
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

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LossArguments {
    entry_price: String,
    stop_price: String,
    share_count: u32,
}

fn execute_maximum_planned_loss(arguments: &JsonObject) -> Result<String, String> {
    // Shape boundary: reject missing, unknown, and wrongly typed fields.
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
    let per_share = match entry.checked_sub(stop) {
        Some(value) => value,
        None => return Err("price difference exceeds exact decimal range".to_string()),
    };
    let loss = match per_share.checked_mul(shares) {
        Some(value) => value,
        None => return Err("planned loss exceeds exact decimal range".to_string()),
    };
    Ok(format!("${:.2}", loss))
}

fn parse_price(field: &str, raw: &str) -> Result<Decimal, String> {
    // Domain boundary: plain positive decimal strings, never binary floating point.
    let mut parts = raw.split('.');
    let whole = match parts.next() {
        Some(whole) => whole,
        None => return Err(format!("{} is not a plain decimal number", field)),
    };
    if whole.is_empty() || !whole.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(format!("{} '{}' is not a plain decimal number", field, raw));
    }
    if let Some(fraction) = parts.next() {
        if fraction.is_empty()
            || fraction.len() > 4
            || !fraction.bytes().all(|byte| byte.is_ascii_digit())
        {
            return Err(format!("{} requires one to four decimal places", field));
        }
    }
    if parts.next().is_some() {
        return Err(format!("{} '{}' is not a plain decimal number", field, raw));
    }
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

fn tool_definition() -> Tool {
    let schema: JsonObject = serde_json::from_value(json!({
        "type": "object",
        "properties": {
            "entry_price": {"type": "string", "pattern": "^[0-9]+(\\.[0-9]{1,4})?$"},
            "stop_price": {"type": "string", "pattern": "^[0-9]+(\\.[0-9]{1,4})?$"},
            "share_count": {"type": "integer", "minimum": 1}
        },
        "required": ["entry_price", "stop_price", "share_count"],
        "additionalProperties": false
    }))
    .expect("static input schema must be a JSON object");

    Tool::new(
        TOOL_NAME,
        "Calculate the maximum planned loss for a hypothetical long stock position from \
         decimal-dollar entry and stop prices and a whole-number share count. This excludes \
         fees, slippage, and a gap through the stop.",
        Arc::new(schema),
    )
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
        assert_eq!(styled("payload", TextStyle::Blue, false), "payload");
        assert_eq!(
            styled("payload", TextStyle::Blue, true),
            "\x1b[34mpayload\x1b[0m"
        );
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
        let colored = session_fields_with_color(&session, true);
        let usage = format!("usage: {:#?}", session.usage);
        let gold_usage = gold_session_attributes(&usage, true);
        let expected_dimmed = format!("\x1b[2m{}\x1b[0m", gold_usage);
        assert!(colored.contains(&expected_dimmed));
        assert_eq!(colored.matches("\x1b[2m").count(), 1);
        assert_eq!(colored.matches("\x1b[0m").count(), 1);
        let plain = colored
            .replace("\x1b[2m", "")
            .replace("\x1b[0m", "")
            .replace("\x1b[38;2;255;215;0m", "")
            .replace("\x1b[39m", "")
            .replace("\x1b[39m", "");
        assert_eq!(plain, session_fields(&session));
        assert!(!session_fields_with_color(&session, false).contains('\x1b'));
        assert!(colored.contains("\x1b[0m\n\x1b[38;2;255;215;0mapplied_passes\x1b[39m:"));
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
            displayed_text: true,
            ..EventSummary::default()
        };
        assert!(inspect_turn(&session, &summary, 1).is_err());
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
    fn arguments(shares: u32) -> JsonObject {
        let value = json!({"entry_price": "51.20", "stop_price": "50.70", "share_count": shares});
        match value {
            Value::Object(arguments) => arguments,
            _ => panic!("test fixture is a JSON object"),
        }
    }

    fn request(id: &str, name: &str, shares: u32) -> Message {
        let call = CallToolRequestParams::new(name.to_string()).with_arguments(arguments(shares));
        Message::assistant().with_tool_request(id, Ok(call))
    }

    fn safe_tools() -> SafeTools {
        SafeTools {
            inner: ToolOperation::new().with_provider(Arc::new(LossToolProvider)),
        }
    }

    fn applied_effects(result: OperationResult<ChatEffect>) -> anyhow::Result<Vec<ChatEffect>> {
        match result {
            OperationResult::Applied(result) => Ok(result.effects),
            OperationResult::NotApplicable => Err(anyhow::anyhow!("expected applied operation")),
        }
    }

    #[test]
    fn exact_loss_and_tweak_are_validated() -> Result<(), String> {
        let default_loss = execute_maximum_planned_loss(&arguments(200))?;
        let tweaked_loss = execute_maximum_planned_loss(&arguments(100))?;
        assert_eq!(default_loss, "$100.00");
        assert_eq!(tweaked_loss, "$50.00");
        Ok(())
    }

    #[test]
    fn untrusted_shape_and_domain_values_fail_without_panicking() {
        for value in [
            json!({"entry_price":51.20,"stop_price":"50.70","share_count":200}),
            json!({"entry_price":"51.20","stop_price":"50.70","share_count":200,"ticker":"X"}),
            json!({"entry_price":"51.20","stop_price":"50.70"}),
            json!({"entry_price":"51.20","stop_price":"50.70","share_count":-1}),
            json!({"entry_price":"51.20","stop_price":"50.70","share_count":0}),
            json!({"entry_price":"51.20","stop_price":"50.70","share_count":1.5}),
            json!({"entry_price":"50.70","stop_price":"50.70","share_count":200}),
            json!({"entry_price":"49.00","stop_price":"50.70","share_count":200}),
            json!({"entry_price":"51.20000","stop_price":"50.70","share_count":200}),
            json!({"entry_price":"0","stop_price":"50.70","share_count":200}),
            json!({"entry_price":"51.20","stop_price":"-1","share_count":200}),
            json!({"entry_price":"79228162514264337593543950335","stop_price":"1","share_count":200}),
        ] {
            let args = match value {
                Value::Object(args) => args,
                _ => panic!("test fixture is a JSON object"),
            };
            assert!(execute_maximum_planned_loss(&args).is_err());
        }
        for raw in ["1e2", "+1", " 1", "1_0", ".5", "5.", "1.2.3", "not-money"] {
            assert!(parse_price("price", raw).is_err());
        }
        assert!(execute_allowed_tool(&Ok(CallToolRequestParams::new(TOOL_NAME))).is_err());
        let unknown = CallToolRequestParams::new("place_order").with_arguments(arguments(200));
        assert!(execute_allowed_tool(&Ok(unknown)).is_err());
        assert!(execute_allowed_tool(&Err(ErrorData::invalid_params("broken", None))).is_err());
    }

    #[tokio::test]
    async fn same_capability_and_history_survive_the_new_user_turn() -> anyhow::Result<()> {
        let runtime = ChatRuntime::seeded();
        let first = runtime.load(SESSION_ID).await?;
        let tools = safe_tools();
        let definitions_first = tools.inference_tools(&first).await?;
        let (sender, _receiver) = mpsc::channel(8);
        let emit = Emitter::new(sender, CancellationToken::new());
        let mut effects = vec![ChatEffect::from(
            Message::assistant().with_text("A stop-loss is a planned exit."),
        )];
        runtime.apply_effects(&first, &mut effects, &emit).await?;
        let before = runtime.load(SESSION_ID).await?;
        runtime
            .append_user("Calculate for my hypothetical trade")
            .await?;
        let second = runtime.load(SESSION_ID).await?;
        assert_eq!(
            &second.conversation.messages()[..before.conversation.len()],
            before.conversation.messages()
        );
        assert_eq!(second.id, first.id);
        assert_eq!(second.applied_passes, 0);
        assert_eq!(tools.inference_tools(&second).await?, definitions_first);
        assert_eq!(definitions_first.len(), 1);
        assert_eq!(definitions_first[0].name.as_ref(), TOOL_NAME);
        Ok(())
    }

    #[tokio::test]
    async fn every_block_gets_a_correlated_response_including_unknown_and_malformed_calls(
    ) -> anyhow::Result<()> {
        let runtime = ChatRuntime::seeded();
        let mut session = runtime.load(SESSION_ID).await?;
        let mixed = request("valid-1", TOOL_NAME, 200)
            .with_text("interleaved text")
            .with_tool_request("unknown", Ok(CallToolRequestParams::new("place_order")))
            .with_tool_request("broken", Err(ErrorData::invalid_params("bad syntax", None)))
            .with_tool_request(
                "valid-2",
                Ok(CallToolRequestParams::new(TOOL_NAME).with_arguments(arguments(100))),
            );
        session.conversation.push(mixed);
        let (sender, mut receiver) = mpsc::channel(8);
        let emit = Emitter::new(sender, CancellationToken::new());
        let tools = safe_tools();
        // Unknown requests are answered first, without hiding the known ones.
        let first_result = tools.run(&session, &session.conversation, &emit).await?;
        let first_effects = applied_effects(first_result)?;
        for effect in first_effects {
            if let ChatEffect::AppendMessage(message) = effect {
                session.conversation.push(message);
            }
        }
        let second_result = tools.run(&session, &session.conversation, &emit).await?;
        let second_effects = applied_effects(second_result)?;
        for effect in second_effects {
            if let ChatEffect::AppendMessage(message) = effect {
                session.conversation.push(message);
            }
        }
        let mut responses = HashMap::new();
        for message in session.conversation.messages() {
            for block in &message.content {
                if let Some(response) = block.as_tool_response() {
                    assert_eq!(message.role, rmcp::model::Role::User);
                    assert!(responses
                        .insert(response.id.clone(), response.tool_result.clone())
                        .is_none());
                }
            }
        }
        assert_eq!(responses.len(), 4);
        assert!(matches!(responses.get("unknown"), Some(Err(_))));
        assert!(matches!(responses.get("broken"), Some(Err(_))));
        assert_eq!(
            responses.get("valid-1"),
            Some(&Ok(CallToolResult::success(vec![ContentBlock::text(
                "$100.00"
            )])))
        );
        assert_eq!(
            responses.get("valid-2"),
            Some(&Ok(CallToolResult::success(vec![ContentBlock::text(
                "$50.00"
            )])))
        );
        let declined = tools.run(&session, &session.conversation, &emit).await?;
        assert!(matches!(declined, OperationResult::NotApplicable));
        drop(emit);
        let mut events = 0;
        while receiver.recv().await.is_some() {
            events += 1;
        }
        assert_eq!(events, 2);
        Ok(())
    }

    #[tokio::test]
    async fn request_bound_answers_excess_without_executing_and_then_yields() -> anyhow::Result<()>
    {
        let runtime = ChatRuntime::seeded();
        let mut session = runtime.load(SESSION_ID).await?;
        let mut message = Message::assistant();
        for index in 0..MAX_TOOL_REQUESTS + 2 {
            let id = format!("call-{}", index);
            message = message.with_tool_request(
                id,
                Ok(CallToolRequestParams::new(TOOL_NAME).with_arguments(arguments(200))),
            );
        }
        session.conversation.push(message);
        let (sender, _receiver) = mpsc::channel(8);
        let emit = Emitter::new(sender, CancellationToken::new());
        let tools = safe_tools();
        for _pass in 0..2 {
            let result = tools.run(&session, &session.conversation, &emit).await?;
            let effects = applied_effects(result)?;
            for effect in effects {
                if let ChatEffect::AppendMessage(message) = effect {
                    session.conversation.push(message);
                }
            }
        }
        let mut count = 0;
        let mut errors = 0;
        for message in session.conversation.messages() {
            for block in &message.content {
                if let Some(response) = block.as_tool_response() {
                    count += 1;
                    if response.tool_result.is_err() {
                        errors += 1;
                    }
                }
            }
        }
        assert_eq!(count, MAX_TOOL_REQUESTS + 2);
        assert_eq!(errors, 2);
        let stopped = RunLimit.run(&session, &session.conversation, &emit).await?;
        match stopped {
            OperationResult::Applied(result) => {
                assert!(result.yield_to_client);
                assert!(matches!(result.effects.first(), Some(ChatEffect::Halt(_))));
            }
            OperationResult::NotApplicable => {
                return Err(anyhow::anyhow!("request limit did not halt"))
            }
        }
        Ok(())
    }

    #[tokio::test]
    async fn duplicate_ids_halt_and_pass_bound_keeps_the_gdk_run_loop() -> anyhow::Result<()> {
        let runtime = ChatRuntime::seeded();
        let mut session = runtime.load(SESSION_ID).await?;
        session.conversation.push(
            request("same", TOOL_NAME, 200)
                .with_tool_request("same", Ok(CallToolRequestParams::new(TOOL_NAME))),
        );
        let (sender, _receiver) = mpsc::channel(8);
        let emit = Emitter::new(sender, CancellationToken::new());
        let result = safe_tools()
            .run(&session, &session.conversation, &emit)
            .await?;
        match result {
            OperationResult::Applied(result) => assert!(result.yield_to_client),
            OperationResult::NotApplicable => {
                return Err(anyhow::anyhow!("duplicate IDs did not halt"))
            }
        }
        // No provider needed: the first operation halts before inference could act.
        let mut stored = runtime.sessions.lock().await;
        let session = match stored.get_mut(SESSION_ID) {
            Some(session) => session,
            None => return Err(anyhow::anyhow!("missing Session")),
        };
        session.applied_passes = MAX_APPLIED_PASSES;
        drop(stored);
        let steps = vec![Step::Operation(Arc::new(RunLimit))];
        let machine = StateMachine::new(steps, CancellationToken::new());
        let final_session = machine.run(&runtime, SESSION_ID, &emit).await?;
        assert!(final_session.halt_reason.is_some());
        assert_eq!(final_session.applied_passes, MAX_APPLIED_PASSES + 1);
        // Normal completion at the bound is not misreported as a runaway loop.
        let mut finished = final_session.clone();
        finished
            .conversation
            .push(Message::assistant().with_text("Done."));
        let result = RunLimit
            .run(&finished, &finished.conversation, &emit)
            .await?;
        assert!(matches!(result, OperationResult::NotApplicable));
        Ok(())
    }
    #[tokio::test]
    async fn complete_request_display_precedes_tool_result_and_final_answer() -> anyhow::Result<()>
    {
        let runtime = ChatRuntime::seeded();
        let before = runtime.load(SESSION_ID).await?;
        let (sender, mut receiver) = mpsc::channel(8);
        let emit = Emitter::new(sender, CancellationToken::new());
        let request_message = request("order-call", TOOL_NAME, 200).with_text("Calculating.");
        // Provider deltas may contain request fragments; they are not the saved display.
        emit.emit(AgentEvent::Message(request_message.clone()))
            .await;
        let mut effects = vec![ChatEffect::from(request_message)];
        runtime.apply_effects(&before, &mut effects, &emit).await?;
        let saved = runtime.load(SESSION_ID).await?;
        let tools_result = safe_tools().run(&saved, &saved.conversation, &emit).await?;
        let mut effects = applied_effects(tools_result)?;
        runtime.apply_effects(&saved, &mut effects, &emit).await?;
        emit.emit(AgentEvent::Message(
            Message::assistant().with_text("The planned loss is $100.00."),
        ))
        .await;
        drop(emit);
        let mut messages = Vec::new();
        while let Some(event) = receiver.recv().await {
            if let AgentEvent::Message(message) = event {
                messages.push(message);
            }
        }
        assert_eq!(messages.len(), 4);
        assert!(!is_reconstructed_request_display(&messages[0]));
        assert!(is_reconstructed_request_display(&messages[1]));
        assert_eq!(
            messages[1].get_tool_request_ids(),
            HashSet::from(["order-call"])
        );
        assert!(messages[2].is_tool_response());
        assert_eq!(
            messages[2].get_tool_response_ids(),
            HashSet::from(["order-call"])
        );
        assert_eq!(messages[3].as_concat_text(), "The planned loss is $100.00.");
        for message in saved.conversation.messages() {
            assert!(!is_reconstructed_request_display(message));
        }
        Ok(())
    }
}
