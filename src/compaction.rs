// Lesson 13
//! Application policy, not the full Goose application's automatic compaction assembly.
use super::*;
use crate::learning_record::{markdown_note, MAX_RECORD_BYTES};
use goose_agent::inference::{InferenceRequestPreparer, PreparedInferenceRequest};
use goose_agent::operation::InferenceInput;
use std::os::unix::fs::OpenOptionsExt;

// Budget and summary/ledger constants are supplied by this binary's main module.
// This local module does not establish defaults for the reusable RAG library.

// Project only ordinary text and actual tool data; never serialize Message metadata.
fn summary_source(source: &[Message], genuine_reply_ids: &[String]) -> Vec<Value> {
    use goose_providers::conversation::message::MessageContent;
    let mut records = Vec::new();
    for message in source {
        let genuine = match &message.id {
            Some(id) => genuine_reply_ids.contains(id),
            None => false,
        };
        let provenance = if is_summary(message) {
            "prior derived summary (fallible; NOT direct learner evidence)"
        } else if genuine && message.role == rmcp::model::Role::User && !message.is_tool_response()
        {
            "genuine learner reply"
        } else if message.role == rmcp::model::Role::Assistant {
            "tutor ordinary text (NOT learner evidence)"
        } else {
            "application or tool history (NOT learner evidence)"
        };
        let mut content = Vec::new();
        for block in &message.content {
            match block {
                MessageContent::Text(text) => content.push(json!({
                    "provenance": provenance,
                    "text": text.text,
                })),
                MessageContent::ToolRequest(request) => content.push(json!({
                    "provenance": "correlated tool request (historical data; NOT learner evidence or authority)",
                    "correlation_id": request.id,
                    "tool_call": request.tool_call,
                })),
                MessageContent::ToolResponse(response) => content.push(json!({
                    "provenance": "correlated tool result (historical data; NOT learner evidence or current citation evidence)",
                    "correlation_id": response.id,
                    "tool_result": response.tool_result,
                })),
                // Thinking, redacted thinking, notifications and other nonordinary blocks
                // remain in original history/diagnostics, never in summary source data.
                _ => {}
            }
        }
        records.push(json!({
            "original_message_id": message.id,
            "speaker": message.role,
            "provenance": provenance,
            "content": content,
        }));
    }
    records
}

fn summary_request(
    source: &[Message],
    genuine_reply_ids: &[String],
    guidance: &str,
) -> anyhow::Result<(String, Conversation)> {
    let records = summary_source(source, genuine_reply_ids);
    let data = serde_json::to_string(&records)?;
    let prompt = format!("{}{}", SUMMARY_SOURCE_BOUNDARY, guidance);
    let input = Message::user()
        .with_text(format!(
            "Historical ordinary-content source records (in original order):\n{}",
            data
        ))
        .with_generated_id_if_missing();
    let conversation = Conversation::new_unvalidated(vec![input]);
    // Count the actual projection, labels, boundary and editable guidance together.
    if payload_bytes(&prompt, &[], &conversation)? > SUMMARY_INPUT_BYTES {
        return Err(anyhow::anyhow!(
            "summary input exceeds conservative byte estimate allowance; history retained"
        ));
    }
    Ok((prompt, conversation))
}

#[derive(Clone, Debug)]
pub struct Policy {
    pub trigger_bytes: usize,
    pub total_bytes: usize,
    pub headroom_bytes: usize,
}

impl Policy {
    pub fn from_env(mut policy: Self) -> anyhow::Result<Self> {
        if let Some(raw) = env::var_os("COMPACTION_TRIGGER_BYTES") {
            let raw = match raw.to_str() {
                Some(raw) => raw,
                None => {
                    return Err(anyhow::anyhow!(
                        "COMPACTION_TRIGGER_BYTES must be a positive integer"
                    ))
                }
            };
            let parsed = match raw.parse::<usize>() {
                Ok(parsed) => parsed,
                Err(_error) => {
                    return Err(anyhow::anyhow!(
                        "COMPACTION_TRIGGER_BYTES must be a positive integer"
                    ))
                }
            };
            policy.trigger_bytes = parsed;
        }
        let allowance = policy.total_bytes.saturating_sub(policy.headroom_bytes);
        if policy.trigger_bytes == 0 || policy.trigger_bytes > allowance {
            return Err(anyhow::anyhow!(
                "COMPACTION_TRIGGER_BYTES must be 1..={} (byte budget minus reserved headroom)",
                allowance
            ));
        }
        Ok(policy)
    }
}

// Conservative application estimate, NOT the actual provider request or a tokenizer.
// alpha.11 OpenAI serializes call arguments a second time and can replay thinking
// across split calls. Charge only these extra copies, not all already-escaped text.
fn payload_bytes(
    prompt: &str,
    tools: &[Tool],
    conversation: &Conversation,
) -> anyhow::Result<usize> {
    let serialized = serde_json::to_vec(
        &json!({"system_prompt": prompt, "tools": tools, "messages": conversation.messages()}),
    )?;
    let mut bytes = serialized.len().saturating_add(PROTOCOL_ENVELOPE_BYTES); // model/options/stream envelope allowance
    let mut thinking_bytes = 0usize;
    let mut requests = 0usize;
    for message in conversation.messages() {
        bytes = bytes.saturating_add(PROTOCOL_ITEM_BYTES); // projected message envelope
        for block in &message.content {
            bytes = bytes.saturating_add(PROTOCOL_ITEM_BYTES); // split/merged protocol envelope
            if let Some(request) = block.as_tool_request() {
                requests += 1;
                if let Ok(call) = &request.tool_call {
                    let arguments = serde_json::to_string(&call.arguments)?;
                    bytes = bytes.saturating_add(serde_json::to_vec(&arguments)?.len());
                }
            }
            if let goose_providers::conversation::message::MessageContent::Thinking(thinking) =
                block
            {
                thinking_bytes =
                    thinking_bytes.saturating_add(serde_json::to_vec(&thinking.thinking)?.len());
            }
        }
    }
    bytes = bytes.saturating_add(thinking_bytes.saturating_mul(requests));
    bytes = bytes.saturating_add(tools.len().saturating_mul(PROTOCOL_ITEM_BYTES));
    Ok(bytes)
}

fn tutor_payload(session: &ChatSession) -> anyhow::Result<usize> {
    let prompt = session
        .prompt_parts()
        .into_iter()
        .map(|(_name, part)| part)
        .collect::<Vec<_>>()
        .join("\n\n");
    let mut tools = vec![tool_definition()];
    if session.lesson.persistence_enabled {
        tools.push(record_tool_definition());
    }
    payload_bytes(&prompt, &tools, &session.conversation)
}

pub fn check_budget(session: &ChatSession) -> anyhow::Result<()> {
    let measured = tutor_payload(session)?;
    if measured
        > session
            .policy
            .total_bytes
            .saturating_sub(session.policy.headroom_bytes)
    {
        return Err(anyhow::anyhow!("Working input exceeds the conservative byte estimate allowance ({} bytes plus {} reserved headroom; total {}). Static content or the active exchange may be too large; history was not silently truncated. Start a fresh selection or reduce editable content/input.", measured, session.policy.headroom_bytes, session.policy.total_bytes));
    }
    Ok(())
}

pub fn prospective_bytes(
    session: &ChatSession,
    input: &str,
    grounding: &TurnGrounding,
) -> anyhow::Result<usize> {
    let mut candidate = session.clone();
    // New-turn evidence replaces the previous turn's attachment; it never enters history.
    candidate.turn_grounding = grounding.clone();
    candidate.conversation.push(
        Message::user()
            .with_text(input)
            .with_generated_id_if_missing(),
    );
    tutor_payload(&candidate)
}

// Count a not-yet-appended learner reply without changing stored history or evidence.
#[derive(Clone)]
pub struct PendingReply {
    pub input: String,
    pub grounding: TurnGrounding,
}

fn maintenance_payload(
    session: &ChatSession,
    pending: Option<&PendingReply>,
) -> anyhow::Result<usize> {
    match pending {
        Some(pending) => prospective_bytes(session, &pending.input, &pending.grounding),
        None => tutor_payload(session),
    }
}

pub struct BudgetPreparer {
    pub captures: PromptCaptures,
}
#[async_trait]
impl InferenceRequestPreparer<ChatSession> for BudgetPreparer {
    async fn prepare(
        &self,
        session: &ChatSession,
        conversation: &Conversation,
        input: InferenceInput,
    ) -> anyhow::Result<PreparedInferenceRequest> {
        let prompt = input
            .prompt_parts
            .into_iter()
            .map(|(_name, part)| part)
            .collect::<Vec<_>>()
            .join("\n\n");
        let bytes = payload_bytes(&prompt, &input.tools, conversation)?;
        if bytes
            > session
                .policy
                .total_bytes
                .saturating_sub(session.policy.headroom_bytes)
        {
            return Err(anyhow::anyhow!("Inference refused: conservative input estimate is {} bytes, exceeding the {}-byte allowance after reserved output/protocol headroom. No repeated compaction or over-budget provider request was attempted.", bytes, session.policy.total_bytes - session.policy.headroom_bytes));
        }
        // Diagnostics capture the exact prepared prompt only after budget acceptance.
        // Preparation is not proof that a request reached the network/provider.
        self.captures.capture(&prompt).await;
        Ok(PreparedInferenceRequest {
            system_prompt: prompt,
            tools: input.tools,
            additional_messages: Vec::new(),
        })
    }
}

pub fn parse_learned(input: &str) -> Result<String, String> {
    if input.chars().any(char::is_control) {
        return Err("The command must be a single line without control characters.".to_string());
    }
    let command = input.trim();
    let tail = match command.strip_prefix("/learned ") {
        Some(tail) => tail.trim(),
        None => {
            return Err(
                "Use /learned <concept> to declare your own completion, not assessed mastery."
                    .to_string(),
            )
        }
    };
    if tail.is_empty() || tail.len() > 200 || tail.chars().any(char::is_control) {
        return Err("The concept must be one nonempty line of at most 200 UTF-8 bytes, without control characters.".to_string());
    }
    // No markup, terminal controls, path or hidden multiline command interpretation.
    if tail
        .chars()
        .any(|character| !character.is_alphanumeric() && !" -'.,():/".contains(character))
    {
        return Err(
            "Use plain words for the concept (letters, numbers, spaces, and simple punctuation)."
                .to_string(),
        );
    }
    Ok(tail.to_string())
}

pub fn completion_key(concept: &str) -> String {
    concept
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

pub fn append_completion(path: &PathBuf, concept: &str) -> anyhow::Result<String> {
    let existing = read_record(path)?;
    let header = if existing.is_empty() {
        "# Learner learning record\n\nLearner-approved notes and explicit declarations, not a mastery certificate.\n"
    } else {
        ""
    };
    let addition = format!("{}\n## Learner-declared completed concepts\n\n- {} (learner declaration; not assessed mastery)\n", header, markdown_note(concept));
    if existing.len() + addition.len() > MAX_RECORD_BYTES {
        return Err(anyhow::anyhow!(
            "learning record is full; edit it between launches"
        ));
    }
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .mode(0o600)
        .open(path)?;
    file.write_all(addition.as_bytes())?;
    file.sync_all()?;
    read_record(path)
}

pub async fn declare(
    runtime: &ChatRuntime,
    id: &str,
    concept: &str,
    diagnostics: &mut DiagnosticLog,
    path: &PathBuf,
) -> anyhow::Result<()> {
    let session = runtime.load(id).await?;
    let key = completion_key(concept);
    if session.completed_concepts.contains_key(&key) {
        diagnostics.block(
            "Learner completion disposition",
            "text",
            "Duplicate within this Session: no append and no tutor inference.",
        )?;
        print_status("Already declared in this conversation; not saved again.");
        return Ok(());
    }
    if session.completed_concepts.len() >= MAX_COMPLETIONS {
        print_warning(
            "Completion declaration ledger is full; start a fresh selection. Nothing saved.",
        );
        return Ok(());
    }
    diagnostics.block("Learner completion intent BEFORE local append", "text", &format!("Concept: {}\nMemory consent: {}\nExplicit local learner declaration, not mastery or model authority.", concept, session.lesson.persistence_enabled))?;
    let updated = if session.lesson.persistence_enabled {
        Some(append_completion(path, concept)?)
    } else {
        None
    };
    let disposition = if updated.is_some() {
        "saved"
    } else {
        "ephemeral (memory consent disabled)"
    };
    {
        let mut sessions = runtime.sessions.lock().await;
        let stored = match sessions.get_mut(id) {
            Some(stored) => stored,
            None => return Err(anyhow::anyhow!("missing Session")),
        };
        if let Some(updated) = updated {
            stored.lesson.learner_record = updated;
        }
        stored
            .completed_concepts
            .insert(key, disposition.to_string());
        stored.revision += 1;
    }
    diagnostics.block(
        "Learner completion disposition AFTER local append",
        "text",
        disposition,
    )?;
    diagnostics.block(
        "Complete Session after completion declaration",
        "rust",
        &session_fields(&runtime.load(id).await?),
    )?;
    if session.lesson.persistence_enabled {
        print_status("Completion declaration saved (not assessed mastery).");
    } else {
        print_status(
            "Completion declared for this conversation only (not saved; not assessed mastery).",
        );
    }
    Ok(())
}

#[derive(Clone, Debug)]
pub struct Report {
    pub reason: String,
    pub counting_method: String,
    pub before_bytes: usize,
    pub after_bytes: usize,
    pub covered_message_ids: Vec<Option<String>>,
    pub summary: Option<String>,
    pub summary_usage: Vec<ProviderUsage>,
    pub summary_partial_messages: Vec<Message>,
    pub disposition: String,
}

pub struct Outcome {
    pub revision: u64,
    pub replacement: Option<Conversation>,
    pub report: Report,
}

fn is_summary(message: &Message) -> bool {
    message
        .metadata
        .operation_note("compaction", "derived_summary")
        == Some(&json!(true))
}

struct Partition {
    opening: Vec<Message>,
    eligible: Vec<Message>,
    recent: Vec<Message>,
}

// Validate each whole exchange, not just the total counts: never split a tool pair.
fn complete_exchange(messages: &[Message]) -> bool {
    if !goose_agent::operation::ends_turn(messages) {
        return false;
    }
    let mut requests = HashSet::new();
    let mut responses = HashSet::new();
    let mut attempt_start = 0;
    for (index, message) in messages.iter().enumerate() {
        // Match the pinned SDK kickoff window, including an application escalation.
        let kickoff = message.role == rmcp::model::Role::User
            && message.is_user_visible()
            && !message.is_tool_response();
        if kickoff && index > attempt_start {
            if !goose_agent::operation::ends_turn(&messages[attempt_start..index])
                || requests != responses
            {
                return false;
            }
            requests.clear();
            responses.clear();
            attempt_start = index;
        }
        for block in &message.content {
            if let Some(request) = block.as_tool_request() {
                if request.id.is_empty() || !requests.insert(request.id.clone()) {
                    return false;
                }
            }
            if let Some(response) = block.as_tool_response() {
                if !requests.contains(&response.id) || !responses.insert(response.id.clone()) {
                    return false;
                }
            }
        }
    }
    requests == responses
}

fn partition(session: &ChatSession) -> anyhow::Result<Option<Partition>> {
    let mut opening = Vec::new();
    let mut summary = Vec::new();
    let mut exchanges: Vec<Vec<Message>> = Vec::new();
    for message in session.conversation.messages() {
        if is_summary(message) {
            if message.role != rmcp::model::Role::Assistant {
                return Err(anyhow::anyhow!("invalid summary role"));
            }
            summary.push(message.clone());
            continue;
        }
        let genuine = match &message.id {
            Some(id) => session.genuine_reply_ids.contains(id),
            None => false,
        };
        if genuine {
            if message.role != rmcp::model::Role::User || message.is_tool_response() {
                return Err(anyhow::anyhow!("invalid reply provenance"));
            }
            exchanges.push(Vec::new());
        }
        match exchanges.last_mut() {
            Some(exchange) => exchange.push(message.clone()),
            None => opening.push(message.clone()),
        }
    }
    if !complete_exchange(&opening)
        || exchanges
            .iter()
            .any(|exchange| !complete_exchange(exchange))
    {
        return Err(anyhow::anyhow!("maintenance requires complete opening and learner exchanges with whole correlated tool pairs"));
    }
    // Match requests, responses and local decisions inside the same SDK kickoff window.
    let messages = session.conversation.messages();
    let is_kickoff = |message: &Message| {
        message.role == rmcp::model::Role::User
            && message.is_user_visible()
            && !message.is_tool_response()
    };
    let mut attempt_start = match messages.iter().position(is_kickoff) {
        Some(index) => index,
        None => messages.len(),
    };
    while attempt_start < messages.len() {
        let next_kickoff = messages[attempt_start + 1..].iter().position(is_kickoff);
        let attempt_end = match next_kickoff {
            Some(offset) => attempt_start + 1 + offset,
            None => messages.len(),
        };
        let attempt = &messages[attempt_start..attempt_end];
        let kickoff = &messages[attempt_start];
        for request in turn_requests(attempt) {
            if let Ok(call) = &request.tool_call {
                if call.name.as_ref() == RECORD_TOOL {
                    let key = approval_key(kickoff, &request.id)?;
                    if session.settled_approvals.contains_key(&key) {
                        continue;
                    }
                    let success = attempt
                        .iter()
                        .flat_map(|message| &message.content)
                        .filter_map(|block| block.as_tool_response())
                        .any(|response| {
                            response.id == request.id
                                && tool_response_succeeded(&response.tool_result)
                        });
                    if success {
                        return Err(anyhow::anyhow!(
                            "local learning note approval is still pending"
                        ));
                    }
                }
            }
        }
        attempt_start = attempt_end;
    }
    if exchanges.len() <= COMPACTION_RECENT_EXCHANGES {
        return Ok(None);
    }
    let recent = exchanges
        .split_off(exchanges.len() - COMPACTION_RECENT_EXCHANGES)
        .into_iter()
        .flatten()
        .collect();
    let mut eligible = summary;
    eligible.extend(exchanges.into_iter().flatten());
    if serde_json::to_vec(&eligible)?.len() < COMPACTION_MIN_HISTORY_BYTES {
        return Ok(None);
    }
    Ok(Some(Partition {
        opening,
        eligible,
        recent,
    }))
}

fn material_reduction(before: usize, after: usize, policy: &Policy) -> bool {
    after.saturating_add(COMPACTION_MIN_REDUCTION_BYTES) <= before
        && after
            <= before.saturating_mul(COMPACTION_REDUCTION_NUMERATOR)
                / COMPACTION_REDUCTION_DENOMINATOR
        && after <= policy.total_bytes.saturating_sub(policy.headroom_bytes)
}

fn candidate(partition: Partition, text: &str) -> Conversation {
    let mut summary = Message::assistant()
        .with_text(format!("{}{}", SUMMARY_MARKER, text))
        .with_generated_id_if_missing();
    summary
        .metadata
        .set_operation_note("compaction", "derived_summary", json!(true));
    let mut messages = partition.opening;
    messages.push(summary);
    messages.extend(partition.recent);
    Conversation::new_unvalidated(messages)
}

pub fn apply_outcome(stored: &mut ChatSession, outcome: &mut Outcome) -> anyhow::Result<()> {
    // Candidate was built off-lock; reject stale state rather than overwriting newer work.
    if stored.revision != outcome.revision {
        outcome.report.disposition =
            "revision changed: replacement rejected; history retained".to_string();
        outcome.report.after_bytes = outcome.report.before_bytes;
    } else if let Some(replacement) = outcome.replacement.take() {
        stored.conversation = replacement;
        stored.genuine_reply_ids.retain(|id| {
            stored
                .conversation
                .messages()
                .iter()
                .any(|message| message.id.as_ref() == Some(id))
        });
    }
    stored.usage.extend(outcome.report.summary_usage.clone());
    stored.maintenance = None;
    stored.maintenance_report = Some(outcome.report.clone());
    Ok(())
}

pub struct Compaction {
    provider: Arc<dyn Provider>,
    model: ModelConfig,
    guidance: String,
    policy: Policy,
    pending_reply: Option<PendingReply>,
}

impl Compaction {
    pub fn new(
        provider: Arc<dyn Provider>,
        mut model: ModelConfig,
        policy: Policy,
    ) -> anyhow::Result<Self> {
        let path = PathBuf::from(REPOSITORY_ROOT)
            .join(CONTENT_DIRECTORY)
            .join("compaction-guidance.md");
        let guidance = fs::read_to_string(path)?;
        if guidance.trim().is_empty() || guidance.len() > COMPACTION_GUIDANCE_BYTES {
            return Err(anyhow::anyhow!(
                "compaction guidance must be nonempty and at most 12000 bytes"
            ));
        }
        model.max_tokens = Some(SUMMARY_MAX_TOKENS);
        Ok(Self {
            provider,
            model,
            guidance,
            policy,
            pending_reply: None,
        })
    }

    async fn summarize(
        &self,
        source: &[Message],
        genuine_reply_ids: &[String],
        parent_cancel: &CancellationToken,
    ) -> anyhow::Result<SummaryResult> {
        let (guidance, conversation) = summary_request(source, genuine_reply_ids, &self.guidance)?;
        let runtime = SummaryRuntime {
            state: Mutex::new(SummarySession {
                conversation,
                usage: Vec::new(),
                batches: 0,
            }),
        };
        let cancel = parent_cancel.child_token();
        let runner = InferenceRunner::new(self.provider.clone(), self.model.clone());
        let machine = StateMachine::new(
            vec![
                Step::Operation(Arc::new(SummaryStop { guidance })),
                Step::Inference(Arc::new(runner)),
            ],
            cancel.clone(),
        );
        let (sender, mut receiver) = mpsc::channel(32);
        let emit = Emitter::new(sender, cancel.clone());
        let finished = CancellationToken::new();
        let run = async {
            let result = cooperative_run(
                machine.run(&runtime, "compaction-summary", &emit),
                &cancel,
                Duration::from_secs(RUN_TIMEOUT_SECONDS),
                Duration::from_secs(CANCELLATION_GRACE_SECONDS),
            )
            .await;
            drop(emit);
            finished.cancel();
            result
        };
        let drain = async {
            let mut budget = StreamBudget::default();
            let mut exceeded = false;
            let mut token_limit = false;
            let mut partial_messages = Vec::new();
            let mut closing = false;
            loop {
                let event = tokio::select! {
                    _ = finished.cancelled(), if !closing => {
                        // No new events after the run ends; drain only the bounded queue.
                        receiver.close();
                        closing = true;
                        continue;
                    }
                    event = receiver.recv() => match event { Some(event) => event, None => break },
                };
                let observed = match budget.observe(&event, SUMMARY_BYTES) {
                    Ok(observed) => observed,
                    Err(error) => {
                        cancel.cancel();
                        return Err(error);
                    }
                };
                exceeded |= observed;
                if let AgentEvent::Message(message) = event {
                    token_limit |= message.metadata.output_token_limit_reached;
                    if !exceeded && partial_messages.len() < SUMMARY_PARTIAL_MESSAGES {
                        partial_messages.push(message);
                    }
                }
                if exceeded || token_limit {
                    cancel.cancel();
                }
            }
            Ok::<_, anyhow::Error>((exceeded, token_limit, partial_messages))
        };
        let (run_result, drain_result) = tokio::join!(run, drain);
        let saved = runtime.load("compaction-summary").await?;
        let (exceeded, token_limit, stream_partial) = drain_result?;
        let failure = if run_result.hard_abort {
            Some("summary hard-aborted after 5-second cancellation grace; received usage may be unavailable; history retained".to_string())
        } else if run_result.deadline_reached {
            Some("summary timed out after 180 seconds; cooperative cancellation preserved available partial state/usage; history retained".to_string())
        } else if exceeded || token_limit {
            Some("summary output byte or output-token limit reached; history retained".to_string())
        } else if cancel.is_cancelled() {
            Some("summary cancelled; history retained".to_string())
        } else {
            match run_result.result {
                Some(Err(error)) => Some(format!("summary inference failed: {}", error)),
                Some(Ok(_state)) => None,
                None => Some("summary result unavailable; history retained".to_string()),
            }
        };
        let outputs = &saved.conversation.messages()[1..];
        let text = outputs
            .iter()
            .map(Message::as_concat_text)
            .collect::<String>();
        let invalid = !valid_summary_output(outputs, &text);
        let failure = match failure {
            Some(failure) => Some(failure),
            None => {
                if invalid {
                    Some(
                        "summary was empty, oversized, diagnostic, or non-text; history retained"
                            .to_string(),
                    )
                } else {
                    None
                }
            }
        };
        Ok(SummaryResult {
            text,
            usage: saved.usage,
            partial: if outputs.is_empty() {
                stream_partial
            } else {
                outputs.to_vec()
            },
            failure,
        })
    }
}

fn valid_summary_output(outputs: &[Message], text: &str) -> bool {
    !text.trim().is_empty()
        && text.len() <= SUMMARY_BYTES
        && text != "The model returned an empty response. Please resend your message to continue."
        && outputs.iter().all(|message| {
            message.role == rmcp::model::Role::Assistant
                && message.error_kind().is_none()
                && !message.metadata.output_token_limit_reached
                && message
                    .content
                    .iter()
                    .all(|block| matches!(block,
                        goose_providers::conversation::message::MessageContent::Text(_)
                        | goose_providers::conversation::message::MessageContent::Thinking(_)
                        | goose_providers::conversation::message::MessageContent::RedactedThinking(_)))
        })
}

struct SummaryResult {
    text: String,
    usage: Vec<ProviderUsage>,
    partial: Vec<Message>,
    failure: Option<String>,
}

#[async_trait]
impl Operation<ChatSession, ChatEffect> for Compaction {
    fn name(&self) -> &'static str {
        "compaction"
    }
    async fn run(
        &self,
        session: &ChatSession,
        _conversation: &Conversation,
        emit: &Emitter,
    ) -> anyhow::Result<OperationResult<ChatEffect>> {
        let reason = match &session.maintenance {
            Some(reason) => reason.clone(),
            None => return not_applicable(),
        };
        let before_bytes = maintenance_payload(session, self.pending_reply.as_ref())?;
        let pending_counting = match &self.pending_reply {
            Some(_pending) => "; pending learner question and replacement new-turn grounding attachment counted; previous stored attachment excluded",
            None => "; no pending learner question or replacement attachment; stored grounding attachment counted",
        };
        let mut report = Report { reason, counting_method: "conservative native JSON UTF-8 bytes + argument JSON-string escaping + thinking replay + 512 bytes per message/block/tool + 4096 model/stream envelope; NOT actual provider request, tokens or verified context capacity; 16000 reserved output/protocol headroom".to_string() + pending_counting, before_bytes, after_bytes: before_bytes, covered_message_ids: Vec::new(), summary: None, summary_usage: Vec::new(), summary_partial_messages: Vec::new(), disposition: String::new() };
        let partition_result = partition(session);
        let mut replacement = None;
        match partition_result {
            Ok(Some(parts)) => {
                report.covered_message_ids = parts.eligible.iter().map(|message| message.id.clone()).collect();
                if emit.cancel_token().is_cancelled() {
                    report.disposition = "cancelled before summary attempt; history retained".to_string();
                    return yielded_with([ChatEffect::Maintenance(Outcome { revision: session.revision, replacement: None, report })]);
                }
                match self.summarize(&parts.eligible, &session.genuine_reply_ids, emit.cancel_token()).await {
                    Ok(summary) => {
                        report.summary_usage = summary.usage;
                        report.summary_partial_messages = summary.partial;
                        report.summary = Some(summary.text.clone());
                        match summary.failure {
                            Some(failure) => report.disposition = failure,
                            None => {
                                let proposed = candidate(parts, &summary.text);
                                let mut proposed_session = session.clone();
                                proposed_session.conversation = proposed.clone();
                                let after_bytes = maintenance_payload(&proposed_session, self.pending_reply.as_ref())?;
                                if material_reduction(before_bytes, after_bytes, &self.policy) {
                                    report.after_bytes = after_bytes;
                                    report.disposition = "applied: material reduction; opening and latest two genuine complete exchanges retained".to_string();
                                    replacement = Some(proposed);
                                } else { report.disposition = "candidate did not materially reduce input or fit allowance; history retained".to_string(); }
                            }
                        }
                    }
                    Err(error) => report.disposition = format!("summary not applied: {}", error),
                }
            }
            Ok(None) => report.disposition = "no-op: insufficient eligible older history (retain latest two genuine exchanges; at least 8000 eligible serialized bytes needed)".to_string(),
            Err(error) => report.disposition = format!("unsafe boundary: {}; history retained", error),
        }
        yielded_with([ChatEffect::Maintenance(Outcome {
            revision: session.revision,
            replacement,
            report,
        })])
    }
}

// Isolated state: no learner record, proposal providers, calculator, or retrieval capability.
#[derive(Clone)]
struct SummarySession {
    conversation: Conversation,
    usage: Vec<ProviderUsage>,
    batches: usize,
}
impl MachineSession for SummarySession {
    fn id(&self) -> &str {
        "compaction-summary"
    }
    fn conversation(&self) -> Option<&Conversation> {
        Some(&self.conversation)
    }
}
struct SummaryRuntime {
    state: Mutex<SummarySession>,
}
#[async_trait]
impl SessionLoader<SummarySession> for SummaryRuntime {
    async fn load(&self, _id: &str) -> anyhow::Result<SummarySession> {
        Ok(self.state.lock().await.clone())
    }
}
#[async_trait]
impl EffectHandler<SummarySession, ChatEffect> for SummaryRuntime {
    async fn apply_effects(
        &self,
        _session: &SummarySession,
        effects: &mut [ChatEffect],
        _emit: &Emitter,
    ) -> anyhow::Result<()> {
        let mut state = self.state.lock().await;
        for effect in effects {
            match effect {
                ChatEffect::AppendMessage(message) => state.conversation.push(message.clone()),
                ChatEffect::RecordUsage(usage) => state.usage.push(usage.clone()),
                _ => return Err(anyhow::anyhow!("unexpected summary effect")),
            }
        }
        state.batches += 1;
        Ok(())
    }
}
struct SummaryStop {
    guidance: String,
}
#[async_trait]
impl Operation<SummarySession, ChatEffect> for SummaryStop {
    fn name(&self) -> &'static str {
        "summary-once"
    }
    async fn prompt_parts(
        &self,
        _session: &SummarySession,
        _conversation: &Conversation,
    ) -> anyhow::Result<Vec<(String, String)>> {
        Ok(vec![(
            "summary-guidance".to_string(),
            self.guidance.clone(),
        )])
    }
    async fn run(
        &self,
        session: &SummarySession,
        _conversation: &Conversation,
        _emit: &Emitter,
    ) -> anyhow::Result<OperationResult<ChatEffect>> {
        if session.batches > 0 {
            return yielded_with([]);
        }
        not_applicable()
    }
}

pub async fn maintain(
    operation: &Compaction,
    runtime: &ChatRuntime,
    id: &str,
    reason: &str,
    diagnostics: &mut DiagnosticLog,
    pending_reply: Option<PendingReply>,
) -> anyhow::Result<()> {
    {
        let mut sessions = runtime.sessions.lock().await;
        let stored = match sessions.get_mut(id) {
            Some(stored) => stored,
            None => return Err(anyhow::anyhow!("missing Session")),
        };
        stored.maintenance = Some(reason.to_string());
        stored.revision += 1;
    }
    diagnostics.block(
        "Complete Session BEFORE context maintenance",
        "rust",
        &session_fields(&runtime.load(id).await?),
    )?;
    diagnostics.block(
        "Application-owned summary guidance (isolated tool-free inference)",
        "text",
        &operation.guidance,
    )?;
    print_heading("Maintaining conversation context…");
    // A maintenance machine has one Operation and yields once; never enters tutor inference.
    let machine = StateMachine::new(
        vec![Step::Operation(Arc::new(Compaction {
            provider: operation.provider.clone(),
            model: operation.model.clone(),
            guidance: operation.guidance.clone(),
            policy: operation.policy.clone(),
            pending_reply,
        }))],
        CancellationToken::new(),
    );
    let (sender, mut receiver) = mpsc::channel(4);
    let emit = Emitter::new(sender, CancellationToken::new());
    let run = async {
        let result = machine.run(runtime, id, &emit).await;
        drop(emit);
        result
    };
    let drain = async { while receiver.recv().await.is_some() {} };
    let (result, ()) = tokio::join!(run, drain);
    let saved = runtime.load(id).await?;
    diagnostics.block(
        "Complete Session AFTER context maintenance (includes summary/usage/disposition)",
        "rust",
        &session_fields(&saved),
    )?;
    if let Some(report) = &saved.maintenance_report {
        diagnostics.block(
            "Maintenance measurements and reason",
            "text",
            &format!(
                "Reason: {}\nCounting: {}\nBefore: {} bytes\nAfter: {} bytes",
                report.reason, report.counting_method, report.before_bytes, report.after_bytes
            ),
        )?;
    }
    match result {
        Ok(_state) => {
            if let Some(report) = saved.maintenance_report {
                if report.disposition.starts_with("applied:") {
                    print_status("Older context summarized; your latest exchanges remain. Summary is fallible, not mastery.");
                } else {
                    print_warning(&format!("Context maintenance: {}. Continuing at the same learner prompt; input allowance still applies.", report.disposition));
                }
            }
        }
        Err(error) => print_warning(&format!(
            "Context maintenance failed; history retained: {}",
            error
        )),
    }
    Ok(())
}

#[cfg(test)]
mod tests;
