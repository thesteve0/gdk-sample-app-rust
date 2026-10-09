use std::{
    collections::{HashMap, HashSet},
    env, fs,
    io::{self, IsTerminal, Write},
    path::PathBuf,
    sync::Arc,
    time::Duration,
};

use async_trait::async_trait;
use crossterm::{
    cursor::MoveUp,
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
};
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
#[cfg(test)]
const SESSION_ID: &str = "test-session";
const TOOL_NAME: &str = "maximum_planned_loss";
const LESSON_TITLE: &str = "Getting started in day trading";
const OPENING_INPUT: &str =
    "Begin the selected learning conversation by asking about my experience and learning goals.";
// Resolve course files against the shared manifest, not the process working directory.
const REPOSITORY_ROOT: &str = env!("CARGO_MANIFEST_DIR");
const CONTENT_DIRECTORY: &str = "lessons/10-day-trading-learning-partner/content";
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

// Editable application-owned context is fixed for the lifetime of a selected lesson.
#[derive(Clone, Debug)]
struct LessonContext {
    title: String,
    contract: String,
    material: String,
    guidance: String,
}

impl LessonContext {
    fn load() -> anyhow::Result<Self> {
        let directory = PathBuf::from(REPOSITORY_ROOT).join(CONTENT_DIRECTORY);
        let context = Self {
            title: LESSON_TITLE.to_string(),
            contract: fs::read_to_string(directory.join("tutor-contract.md"))?,
            material: fs::read_to_string(directory.join("getting-started-material.md"))?,
            guidance: fs::read_to_string(directory.join("getting-started-guidance.md"))?,
        };
        if context.contract.trim().is_empty()
            || context.material.trim().is_empty()
            || context.guidance.trim().is_empty()
        {
            return Err(anyhow::anyhow!("selected lesson content must not be empty"));
        }
        Ok(context)
    }

    fn prompt_parts(&self) -> Vec<(String, String)> {
        vec![
            ("tutor-contract".to_string(), self.contract.clone()),
            ("educational-material".to_string(), self.material.clone()),
            ("teaching-guidance".to_string(), self.guidance.clone()),
        ]
    }
}

async fn run_application() -> anyhow::Result<()> {
    // Truncate once per launch, then append every lesson and turn to this file.
    let mut diagnostics = DiagnosticLog::create()?;
    let result = run_learning_partner(&mut diagnostics).await;
    if let Err(error) = &result {
        diagnostics.block("Launch failure", "text", &format!("{:#}", error))?;
    }
    result
}

async fn run_learning_partner(diagnostics: &mut DiagnosticLog) -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let config_path = provider_config_path()?;
    let provider_json = fs::read_to_string(config_path)?;
    let provider_config: Value = serde_json::from_str(&provider_json)?;
    let boxed_provider = from_json(&provider_json, None, EnvKeyResolver {})?;
    let provider: Arc<dyn Provider> = Arc::from(boxed_provider);
    let model_name = first_configured_model(&provider_config)?;
    let model = ModelConfig::new(model_name);
    print_heading("Day-trading learning partner");
    println!("Education only; no live market data or trade execution.");
    println!("Diagnostics: session-output.md (overwritten each launch; may contain personal conversation).");
    let mut session_number = 0;
    while choose_lesson()? {
        session_number += 1;
        let session_id = format!("lesson-10-{}", session_number);
        let context = LessonContext::load()?;
        diagnostics.lesson(&session_id, &context)?;
        let runtime = ChatRuntime::new(&session_id, context);
        let tools = SafeTools {
            inner: ToolOperation::new().with_provider(Arc::new(LossToolProvider)),
        };
        let runner = InferenceRunner::new(provider.clone(), model.clone());
        let steps: Vec<Step<'_, ChatSession, ChatEffect>> = vec![
            Step::Operation(Arc::new(RunLimit)),
            Step::Operation(Arc::new(tools)),
            Step::Inference(Arc::new(runner)),
        ];
        let cancel = CancellationToken::new();
        let machine = StateMachine::new(steps, cancel.clone());
        print_heading(&format!("\n{}", LESSON_TITLE));
        println!(
            "Ask questions and explore at your pace. /finish returns to lessons; /quit exits."
        );
        let mut turn_number = 1;
        // Selection initiates the welcome; this is application input, not a learner reply.
        run_turn(
            &machine,
            &runtime,
            &cancel,
            &session_id,
            turn_number,
            diagnostics,
        )
        .await?;
        loop {
            let input = match read_line("Learner > ")? {
                Some(input) => input,
                None => {
                    diagnostics.block(
                        "Session ended",
                        "text",
                        "End of input; application exited.",
                    )?;
                    return Ok(());
                }
            };
            match reply_action(&input) {
                ReplyAction::Finish => {
                    diagnostics.block(
                        "Session ended",
                        "text",
                        "/finish: returned to the chooser.",
                    )?;
                    print_status("Conversation finished. Starting again creates a fresh Session.");
                    break;
                }
                ReplyAction::Quit => {
                    diagnostics.block("Session ended", "text", "/quit: application exited.")?;
                    return Ok(());
                }
                ReplyAction::Empty => continue,
                ReplyAction::Message => {}
            }
            runtime.append_user(&session_id, &input).await?;
            turn_number += 1;
            run_turn(
                &machine,
                &runtime,
                &cancel,
                &session_id,
                turn_number,
                diagnostics,
            )
            .await?;
        }
    }
    Ok(())
}

async fn run_turn(
    machine: &StateMachine<'_, ChatSession, ChatEffect>,
    runtime: &ChatRuntime,
    cancel: &CancellationToken,
    session_id: &str,
    turn_number: usize,
    diagnostics: &mut DiagnosticLog,
) -> anyhow::Result<()> {
    let before = runtime.load(session_id).await?;
    diagnostics.turn(turn_number, &before)?;
    let (event_tx, event_rx) = mpsc::channel::<AgentEvent>(32);
    let emit = Emitter::new(event_tx, cancel.clone());
    // The GDK owns all passes within a turn; the application waits for input afterward.
    let run = async {
        let timed_result = tokio::time::timeout(
            Duration::from_secs(RUN_TIMEOUT_SECONDS),
            machine.run(runtime, session_id, &emit),
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
        // Close the sender even on failure and drain concurrently before inspecting state.
        drop(emit);
        run_result
    };
    let (run_result, display_result) = tokio::join!(run, display_events(event_rx));
    let saved = runtime.load(session_id).await?;
    diagnostics.after(&saved)?;
    let summary = match display_result {
        Ok(summary) => summary,
        Err(error) => {
            diagnostics.block(
                "Display failure (partial Session above)",
                "text",
                &error.to_string(),
            )?;
            return Err(error.into());
        }
    };
    for message in &summary.tool_events {
        diagnostics.block(
            "Tool presentation Event (saving is separate)",
            "rust",
            &format!("{:#?}", message),
        )?;
    }
    let final_session = match run_result {
        Ok(session) => session,
        Err(error) => {
            diagnostics.block(
                "Run failure (partial Session above)",
                "text",
                &format!("{:#}", error),
            )?;
            return Err(error);
        }
    };
    let validation = inspect_turn(&final_session, &summary);
    let warnings = match validation {
        Ok(warnings) => warnings,
        Err(error) => {
            diagnostics.block(
                "Turn failure (partial Session above)",
                "text",
                &format!("{:#}", error),
            )?;
            return Err(error);
        }
    };
    for warning in warnings {
        print_warning(&warning);
        diagnostics.block("Warning", "text", &warning)?;
    }
    Ok(())
}

// Markdown diagnostics are presentation, never an input to the tutor or learner memory.
struct DiagnosticLog {
    file: fs::File,
}

impl DiagnosticLog {
    fn create() -> anyhow::Result<Self> {
        let path = PathBuf::from(REPOSITORY_ROOT).join("session-output.md");
        Self::create_at(path)
    }

    fn create_at(path: PathBuf) -> anyhow::Result<Self> {
        let mut file = fs::File::create(path)?;
        writeln!(file, "# Learning partner — launch diagnostics\n\nOverwritten at every launch. May contain personal conversation; ignored by Git, not access-controlled. Not learner memory. No provider configuration or credentials are deliberately recorded.\n")?;
        file.flush()?;
        Ok(Self { file })
    }

    fn block(&mut self, heading: &str, language: &str, text: &str) -> anyhow::Result<()> {
        let block = markdown_block(heading, language, text);
        self.file.write_all(block.as_bytes())?;
        // Surface a logging failure now rather than hiding it until process exit.
        self.file.flush()?;
        Ok(())
    }

    fn lesson(&mut self, session_id: &str, context: &LessonContext) -> anyhow::Result<()> {
        writeln!(
            self.file,
            "\n## Lesson: {} — Session {}\n",
            context.title, session_id
        )?;
        self.block(
            "Application → provider: selected context (prompt parts, not an exact request dump)",
            "rust",
            &format!("{:#?}", context.prompt_parts()),
        )?;
        self.block(
            "Available calculator definition",
            "json",
            &serde_json::to_string_pretty(&tool_definition())?,
        )
    }

    fn turn(&mut self, turn_number: usize, session: &ChatSession) -> anyhow::Result<()> {
        writeln!(self.file, "\n### Turn {}\n", turn_number)?;
        let turn = messages_since_kickoff(&session.conversation)?;
        let input = match turn.first() {
            Some(input) => input,
            None => return Err(anyhow::anyhow!("missing turn input")),
        };
        let heading = if turn_number == 1 {
            "Application → provider: opening input (not a learner reply)"
        } else {
            "Learner input → application → provider"
        };
        self.block(heading, "text", &input.as_concat_text())?;
        self.block(
            "Complete Session before run",
            "rust",
            &session_fields(session),
        )
    }

    fn after(&mut self, session: &ChatSession) -> anyhow::Result<()> {
        self.block(
            "Complete Session after run (possibly partial on failure)",
            "rust",
            &session_fields(session),
        )?;
        let turn = messages_since_kickoff(&session.conversation)?;
        for message in turn {
            for block in &message.content {
                if let Some(request) = block.as_tool_request() {
                    self.block(
                        &format!("Provider → application: saved request {}", request.id),
                        "rust",
                        &format!("{:#?}", request),
                    )?;
                    if let Ok(call) = &request.tool_call {
                        self.block(
                            "Tool name and arguments",
                            "json",
                            &serde_json::to_string_pretty(call)?,
                        )?;
                    }
                }
                if let Some(response) = block.as_tool_response() {
                    self.block(
                        &format!(
                            "Application, as the tool → provider: saved result {}",
                            response.id
                        ),
                        "rust",
                        &format!("{:#?}", response),
                    )?;
                }
            }
            if message.role == rmcp::model::Role::Assistant {
                let text = message.as_concat_text();
                if !text.is_empty() {
                    self.block(
                        "Saved assembled tutor response (not streaming deltas)",
                        "text",
                        &text,
                    )?;
                }
            }
        }
        Ok(())
    }
}

// Remove terminal control sequences from file presentation without changing recorded state.
fn plain_output(text: &str) -> String {
    let mut output = String::new();
    let mut characters = text.chars();
    while let Some(character) = characters.next() {
        if character == '\x1b' {
            match characters.next() {
                Some('[') => {
                    for next in characters.by_ref() {
                        if ('@'..='~').contains(&next) {
                            break;
                        }
                    }
                }
                Some(']') => {
                    let mut escaped = false;
                    for next in characters.by_ref() {
                        if next == '\x07' || (escaped && next == '\\') {
                            break;
                        }
                        escaped = next == '\x1b';
                    }
                }
                Some(_) | None => {}
            }
        } else if !character.is_control() || character == '\n' || character == '\t' {
            output.push(character);
        }
    }
    output
}

fn markdown_block(heading: &str, language: &str, text: &str) -> String {
    let plain = plain_output(text);
    // A learner/model can include Markdown fences; choose a longer enclosing fence.
    let mut longest_run = 0;
    for run in plain.split(|character| character != '`') {
        longest_run = std::cmp::max(longest_run, run.len());
    }
    let fence = "`".repeat(std::cmp::max(3, longest_run + 1));
    let safe_heading = plain_output(heading).replace('\n', " ");
    format!(
        "\n#### {}\n\n{}{}\n{}\n{}\n",
        safe_heading, fence, language, plain, fence
    )
}

#[derive(Debug, PartialEq)]
enum ReplyAction {
    Finish,
    Quit,
    Empty,
    Message,
}

fn reply_action(input: &str) -> ReplyAction {
    match input.trim() {
        "/finish" => ReplyAction::Finish,
        "/quit" => ReplyAction::Quit,
        "" => ReplyAction::Empty,
        _ => ReplyAction::Message,
    }
}

fn read_line(prompt: &str) -> io::Result<Option<String>> {
    print!(
        "{}",
        styled(prompt, TextStyle::Blue, stdout_color_enabled())
    );
    io::stdout().flush()?;
    let mut input = String::new();
    let bytes = io::stdin().read_line(&mut input)?;
    if bytes == 0 {
        return Ok(None);
    }
    while input.ends_with('\n') || input.ends_with('\r') {
        input.pop();
    }
    Ok(Some(input))
}

// Raw mode is confined to the chooser and restored before any provider call/input line.
struct RawMode;

impl RawMode {
    fn enter() -> io::Result<Self> {
        enable_raw_mode()?;
        Ok(Self)
    }
}

impl Drop for RawMode {
    fn drop(&mut self) {
        // Best-effort restoration on an earlier error; success is checked explicitly below.
        let _restore = disable_raw_mode();
    }
}

fn choose_lesson() -> anyhow::Result<bool> {
    if io::stdin().is_terminal()
        && io::stdout().is_terminal()
        && env::var_os("TERM") != Some("dumb".into())
        && env::var_os("NO_COLOR").is_none()
    {
        return arrow_chooser();
    }
    print_heading("\nChoose a lesson:");
    println!("1. {}\nq. Quit", LESSON_TITLE);
    loop {
        let input = match read_line("Choice > ")? {
            Some(input) => input,
            None => return Ok(false),
        };
        match input.trim() {
            "1" => return Ok(true),
            "q" | "/quit" => return Ok(false),
            _ => print_warning("Choose 1 or q."),
        }
    }
}

fn arrow_chooser() -> anyhow::Result<bool> {
    print_heading("\nChoose a lesson (↑/↓, Enter; Esc or q exits):");
    let raw_mode = RawMode::enter()?;
    let mut selected = 0;
    let mut stdout = io::stdout();
    loop {
        let lesson_line = if selected == 0 { "> " } else { "  " };
        let quit_line = if selected == 1 { "> " } else { "  " };
        write!(
            stdout,
            "{}{}\r\n{}Quit\r\n",
            lesson_line,
            styled(LESSON_TITLE, TextStyle::Heading, stdout_color_enabled()),
            quit_line
        )?;
        stdout.flush()?;
        loop {
            let event = event::read()?;
            if let Event::Key(key) = event {
                if key.kind == KeyEventKind::Release {
                    continue;
                }
                let mut decision = None;
                match key.code {
                    KeyCode::Up | KeyCode::Down => {
                        selected = 1 - selected;
                    }
                    KeyCode::Enter => decision = Some(selected == 0),
                    KeyCode::Esc | KeyCode::Char('q') => decision = Some(false),
                    KeyCode::Char('c') => {
                        if key.modifiers.contains(KeyModifiers::CONTROL) {
                            decision = Some(false);
                        } else {
                            continue;
                        }
                    }
                    _ => continue,
                }
                if let Some(start_lesson) = decision {
                    disable_raw_mode()?;
                    drop(raw_mode);
                    return Ok(start_lesson);
                }
                execute!(stdout, MoveUp(2), Clear(ClearType::FromCursorDown))?;
                break;
            }
        }
    }
}

// A loaded Session is a snapshot, not a list of completed Operations.
#[derive(Clone)]
struct ChatSession {
    id: String,
    conversation: Conversation,
    usage: Vec<ProviderUsage>,
    applied_passes: usize,
    halt_reason: Option<String>,
    lesson: LessonContext,
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
    fn new(session_id: &str, lesson: LessonContext) -> Self {
        let mut conversation = Conversation::empty();
        conversation.push(Message::user().with_text(OPENING_INPUT));
        let session = ChatSession {
            id: session_id.to_string(),
            conversation,
            usage: Vec::new(),
            applied_passes: 0,
            halt_reason: None,
            lesson,
        };
        let mut sessions = HashMap::new();
        sessions.insert(session_id.to_string(), session);
        Self {
            sessions: Mutex::new(sessions),
        }
    }

    #[cfg(test)]
    fn seeded() -> Self {
        let lesson = LessonContext::load().expect("supplied lesson content");
        Self::new(SESSION_ID, lesson)
    }

    async fn append_user(&self, session_id: &str, text: &str) -> anyhow::Result<()> {
        let mut sessions = self.sessions.lock().await;
        let stored = match sessions.get_mut(session_id) {
            Some(stored) => stored,
            None => return Err(anyhow::anyhow!("missing Session")),
        };
        stored.conversation.push(Message::user().with_text(text));
        // Bounds reset for a user turn; history and selected context remain unchanged.
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
    tool_events: Vec<Message>,
}

// This marker exists only on the presentation clone, never in saved protocol data.
fn is_reconstructed_request_display(message: &Message) -> bool {
    message
        .metadata
        .operation_note("lesson-display", "reconstructed_request")
        == Some(&json!(true))
}

async fn display_events(receiver: mpsc::Receiver<AgentEvent>) -> io::Result<EventSummary> {
    let mut stdout = io::stdout();
    display_events_to(receiver, &mut stdout, stdout_color_enabled()).await
}

async fn display_events_to<W: Write>(
    mut receiver: mpsc::Receiver<AgentEvent>,
    output: &mut W,
    color: bool,
) -> io::Result<EventSummary> {
    let mut summary = EventSummary::default();
    while let Some(event) = receiver.recv().await {
        match event {
            AgentEvent::Message(message) => {
                summary.message_events += 1;
                if is_reconstructed_request_display(&message) {
                    close_stream(&mut summary, output)?;
                    for block in &message.content {
                        if let Some(request) = block.as_tool_request() {
                            let name = match &request.tool_call {
                                Ok(call) => call.name.to_string(),
                                Err(_error) => "unparseable request".to_string(),
                            };
                            writeln!(
                                output,
                                "{}",
                                styled(
                                    &format!(
                                        "provider → application: calculator activity {} ({})",
                                        request.id, name
                                    ),
                                    TextStyle::Magenta,
                                    color
                                )
                            )?;
                        }
                    }
                    summary.tool_events.push(message);
                    continue;
                }
                if message.is_tool_response() {
                    close_stream(&mut summary, output)?;
                    for block in &message.content {
                        if let Some(response) = block.as_tool_response() {
                            let text = match &response.tool_result {
                                Ok(result) => {
                                    let mut parts = Vec::new();
                                    for content in &result.content {
                                        if let Some(text) = content.as_text() {
                                            parts.push(text.text.clone());
                                        }
                                    }
                                    parts.join(" ")
                                }
                                Err(error) => format!("rejected: {}", error.message),
                            };
                            writeln!(
                                output,
                                "{}",
                                styled(
                                    &format!(
                                        "application, as the tool → provider: calculator result {}",
                                        response.id
                                    ),
                                    TextStyle::Green,
                                    color
                                )
                            )?;
                            print_delimiter(output)?;
                            writeln!(output, "{}", styled(&text, TextStyle::Green, color))?;
                            print_delimiter(output)?;
                        }
                    }
                    summary.tool_events.push(message);
                    continue;
                }
                let text = message.as_concat_text();
                for content in &message.content {
                    if content.as_text().is_none() {
                        summary.non_text_blocks += 1;
                    }
                }
                if !text.is_empty() {
                    if !summary.saw_text {
                        writeln!(
                            output,
                            "{}",
                            styled("Tutor (provider → application):", TextStyle::Heading, color)
                        )?;
                        print_delimiter(output)?;
                        summary.saw_text = true;
                    }
                    summary.displayed_text = true;
                    write!(output, "{}", text)?;
                    output.flush()?;
                }
            }
            AgentEvent::Usage(_)
            | AgentEvent::MessageUsage { .. }
            | AgentEvent::McpNotification(_)
            | AgentEvent::HistoryReplaced(_) => {}
        }
    }
    close_stream(&mut summary, output)?;
    Ok(summary)
}

fn close_stream<W: Write>(summary: &mut EventSummary, output: &mut W) -> io::Result<()> {
    if summary.saw_text {
        writeln!(output)?;
        print_delimiter(output)?;
        summary.saw_text = false;
    }
    Ok(())
}

// Debug formatting includes fields that JSON serialization can omit.
fn session_fields(session: &ChatSession) -> String {
    format!(
        "id: {:?}\nconversation: {:#?}\nusage: {:#?}\napplied_passes: {:?}\nhalt_reason: {:?}\nlesson: {:#?}\n",
        session.id, session.conversation.messages(), session.usage,
        session.applied_passes, session.halt_reason, session.lesson,
    )
}

fn inspect_turn(session: &ChatSession, summary: &EventSummary) -> anyhow::Result<Vec<String>> {
    if let Some(reason) = &session.halt_reason {
        return Err(anyhow::anyhow!("run stopped at a safeguard: {}", reason));
    }
    let turn = messages_since_kickoff(&session.conversation)?;
    let requests = turn_requests(turn);
    let mut responses = Vec::new();
    let mut warnings = Vec::new();
    for message in turn {
        if message.error_kind().is_some()
            || message.as_concat_text()
                == "The model returned an empty response. Please resend your message to continue."
        {
            return Err(anyhow::anyhow!(
                "saved inference diagnostic; inspect session-output.md"
            ));
        }
        for block in &message.content {
            if let Some(response) = block.as_tool_response() {
                responses.push(response);
                if response.tool_result.is_err() {
                    warnings.push(format!(
                        "Calculator request {} was rejected; see session-output.md.",
                        response.id
                    ));
                }
            }
        }
    }
    let last = match turn.last() {
        Some(last) => last,
        None => return Err(anyhow::anyhow!("missing turn")),
    };
    if !summary.displayed_text
        || last.role != rmcp::model::Role::Assistant
        || last.as_concat_text().trim().is_empty()
        || !last.get_tool_request_ids().is_empty()
    {
        return Err(anyhow::anyhow!(
            "no ordinary final tutor answer was both displayed and saved"
        ));
    }
    let mut ids = HashSet::new();
    for request in &requests {
        if request.id.is_empty() || !ids.insert(request.id.as_str()) {
            return Err(anyhow::anyhow!("ambiguous request correlation ID"));
        }
        let matches = responses
            .iter()
            .filter(|response| response.id == request.id)
            .count();
        if matches != 1 {
            return Err(anyhow::anyhow!(
                "request {} lacks exactly one saved correlated response",
                request.id
            ));
        }
    }
    if responses.len() != requests.len() {
        return Err(anyhow::anyhow!("unmatched saved tool response"));
    }
    // Tool selection and teaching quality require human review, not a forced script.
    Ok(warnings)
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
        session: &ChatSession,
        _conversation: &Conversation,
    ) -> anyhow::Result<Vec<(String, String)>> {
        Ok(session.lesson.prompt_parts())
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

fn print_delimiter<W: Write>(output: &mut W) -> io::Result<()> {
    writeln!(output, "{}", PAYLOAD_DELIMITER)
}

fn provider_config_path() -> anyhow::Result<PathBuf> {
    let mut arguments = env::args_os();
    let _program = arguments.next();
    let path = match (arguments.next(), arguments.next()) {
        (Some(path), None) => PathBuf::from(path),
        (None, None) => PathBuf::from(REPOSITORY_ROOT).join(DEFAULT_PROVIDER_CONFIG),
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
    #[tokio::test]
    async fn presentation_output_is_framed_ordered_and_fallible() -> anyhow::Result<()> {
        let (sender, receiver) = mpsc::channel(8);
        let mut reconstructed = request("shown-call", TOOL_NAME, 200);
        reconstructed.metadata.set_operation_note(
            "lesson-display",
            "reconstructed_request",
            json!(true),
        );
        sender.send(AgentEvent::Message(reconstructed)).await?;
        sender
            .send(AgentEvent::Message(Message::user().with_tool_response(
                "shown-call",
                Ok(CallToolResult::success(vec![ContentBlock::text("$100.00")])),
            )))
            .await?;
        sender
            .send(AgentEvent::Message(
                Message::assistant().with_text("Final answer"),
            ))
            .await?;
        drop(sender);
        let mut bytes = Vec::new();
        let summary = display_events_to(receiver, &mut bytes, true).await?;
        assert!(summary.displayed_text);
        assert_eq!(summary.tool_events.len(), 2);
        let output = String::from_utf8(bytes)?;
        let plain = plain_output(&output);
        assert!(plain.contains("calculator result shown-call\n++++++++\n$100.00\n++++++++"));
        assert!(plain.contains("Tutor (provider → application):\n++++++++\nFinal answer\n++++++++"));
        assert!(plain.find("calculator activity") < plain.find("calculator result"));
        assert!(plain.find("calculator result") < plain.find("Final answer"));
        assert!(output.contains("\x1b[35m"));
        assert!(output.contains("\x1b[32m"));
        assert!(output.contains("\x1b[1;36m"));
        assert!(!output.contains("\x1b[2m"));
        assert!(output.contains("\x1b[0m\n++++++++\nFinal answer"));
        // A broken stream returns an error rather than panicking past partial-state logging.
        struct FailedOutput;
        impl Write for FailedOutput {
            fn write(&mut self, _bytes: &[u8]) -> io::Result<usize> {
                Err(io::Error::new(
                    io::ErrorKind::BrokenPipe,
                    "test broken output",
                ))
            }
            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }
        let (sender, receiver) = mpsc::channel(1);
        sender
            .send(AgentEvent::Message(
                Message::assistant().with_text("partial"),
            ))
            .await?;
        drop(sender);
        let mut failed = FailedOutput;
        assert!(display_events_to(receiver, &mut failed, false)
            .await
            .is_err());
        Ok(())
    }

    #[test]
    fn reply_commands_are_application_controls_not_tutor_messages() {
        assert_eq!(reply_action(" /finish "), ReplyAction::Finish);
        assert_eq!(reply_action("/quit"), ReplyAction::Quit);
        assert_eq!(reply_action("  "), ReplyAction::Empty);
        assert_eq!(reply_action("Explain /finish to me"), ReplyAction::Message);
    }

    #[test]
    fn markdown_fences_and_control_sequences_are_presentation_only() {
        let payload = "text\n```rust\ncode\n```\n\x1b[35mrequest\x1b[0m\x1b]0;title\x07";
        let block = markdown_block("Saved response", "text", payload);
        assert!(block.contains("````text\n"));
        assert!(block.contains("```rust\ncode\n```\nrequest"));
        assert!(!block.contains('\x1b'));
        assert!(payload.contains('\x1b')); // No mutation of protocol data.
        for style in [
            TextStyle::Heading,
            TextStyle::Blue,
            TextStyle::Magenta,
            TextStyle::Green,
            TextStyle::Yellow,
            TextStyle::Error,
        ] {
            let colored = styled("value", style, true);
            assert!(colored.ends_with("\x1b[0m"));
            assert_eq!(plain_output(&colored), "value");
            assert_eq!(styled("value", style, false), "value");
        }
    }

    #[tokio::test]
    async fn selected_context_is_contributed_before_inference_and_stays_fixed() -> anyhow::Result<()>
    {
        let runtime = ChatRuntime::seeded();
        let before = runtime.load(SESSION_ID).await?;
        let parts = safe_tools()
            .prompt_parts(&before, &before.conversation)
            .await?;
        assert_eq!(parts.len(), 3);
        assert_eq!(parts[0].1, before.lesson.contract);
        assert_eq!(parts[1].1, before.lesson.material);
        assert_eq!(parts[2].1, before.lesson.guidance);
        assert!(parts[1].1.contains("## Source attribution"));
        runtime
            .append_user(SESSION_ID, "I want to understand orders")
            .await?;
        let after = runtime.load(SESSION_ID).await?;
        assert_eq!(
            safe_tools()
                .prompt_parts(&after, &after.conversation)
                .await?,
            parts
        );
        // The selected context is separate from conversation turns.
        assert_eq!(after.conversation.len(), 2);
        assert_eq!(
            after.conversation.messages()[0].as_concat_text(),
            OPENING_INPUT
        );
        Ok(())
    }

    #[tokio::test]
    async fn reselecting_starts_fresh_history_and_bounds() -> anyhow::Result<()> {
        let first = ChatRuntime::new("first", LessonContext::load()?);
        first
            .append_user("first", "Learner private history")
            .await?;
        let second = ChatRuntime::new("second", LessonContext::load()?);
        let session = second.load("second").await?;
        assert_eq!(session.id, "second");
        assert_eq!(session.conversation.len(), 1);
        assert_eq!(session.applied_passes, 0);
        assert_eq!(session.halt_reason, None);
        assert!(!session_fields(&session).contains("Learner private history"));
        assert!(second.load("first").await.is_err());
        Ok(())
    }

    #[tokio::test]
    async fn diagnostic_log_truncates_launch_and_retains_complete_turns_and_partial_state(
    ) -> anyhow::Result<()> {
        let path = env::temp_dir().join(format!("gdk-lesson10-test-{}.md", std::process::id()));
        fs::write(&path, "obsolete launch")?;
        let mut log = DiagnosticLog::create_at(path.clone())?;
        let runtime = ChatRuntime::seeded();
        let mut session = runtime.load(SESSION_ID).await?;
        log.lesson(SESSION_ID, &session.lesson)?;
        log.turn(1, &session)?;
        session
            .conversation
            .push(request("diagnostic-call", TOOL_NAME, 200));
        session
            .conversation
            .push(Message::user().with_tool_response(
                "diagnostic-call",
                Ok(CallToolResult::success(vec![ContentBlock::text("$100.00")])),
            ));
        session
            .conversation
            .push(Message::assistant().with_text("One assembled answer, not fragments."));
        session.halt_reason = Some("partial failure".to_string());
        log.after(&session)?;
        log.block("Failure", "text", "partial failure")?;
        log.lesson("another-session", &session.lesson)?;
        let text = fs::read_to_string(&path)?;
        assert!(!text.contains("obsolete launch"));
        assert!(!text.contains('\x1b'));
        assert!(text.contains(&session_fields(&session)));
        assert!(text.contains("user_visible: true"));
        assert!(text.contains("inference: None"));
        assert!(text.contains("halt_reason: Some("));
        assert!(text.contains("saved request diagnostic-call"));
        assert!(text.contains("saved result diagnostic-call"));
        assert!(text.contains("One assembled answer, not fragments."));
        assert!(text.contains("Session another-session"));
        let mut next_launch = DiagnosticLog::create_at(path.clone())?;
        next_launch.block("New launch", "text", "fresh")?;
        let fresh = fs::read_to_string(&path)?;
        assert!(!fresh.contains("diagnostic-call"));
        assert!(fresh.contains("New launch"));
        drop(log);
        drop(next_launch);
        fs::remove_file(path)?;
        // A bad destination is not silently ignored.
        assert!(DiagnosticLog::create_at(env::temp_dir()).is_err());
        // Linux's /dev/full accepts opening but fails every write, covering append failures.
        let file = fs::OpenOptions::new().write(true).open("/dev/full")?;
        let mut failed_log = DiagnosticLog { file };
        assert!(failed_log.block("Failure", "text", "cannot save").is_err());
        Ok(())
    }

    #[tokio::test]
    async fn optional_tool_use_does_not_require_a_calculation_for_every_turn() -> anyhow::Result<()>
    {
        let runtime = ChatRuntime::seeded();
        let mut session = runtime.load(SESSION_ID).await?;
        session
            .conversation
            .push(Message::assistant().with_text("What would you like to understand?"));
        let summary = EventSummary {
            displayed_text: true,
            ..EventSummary::default()
        };
        assert!(inspect_turn(&session, &summary)?.is_empty());
        // Orphan responses cannot masquerade as a valid completed round trip.
        session.conversation.push(
            Message::user().with_tool_response("orphan", Ok(CallToolResult::success(vec![]))),
        );
        session
            .conversation
            .push(Message::assistant().with_text("Done"));
        assert!(inspect_turn(&session, &summary).is_err());
        Ok(())
    }
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
        assert!(fields.contains("lesson:"));
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
        assert!(inspect_turn(&session, &summary).is_err());
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
        let mut output = Vec::new();
        let (produced, displayed) =
            tokio::join!(producer, display_events_to(receiver, &mut output, false));
        produced?;
        let summary = displayed?;
        assert_eq!(summary.message_events, 3);
        assert!(!summary.saw_text);
        assert!(summary.displayed_text);
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
            .append_user(SESSION_ID, "Calculate for my hypothetical trade")
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