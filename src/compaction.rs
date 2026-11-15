// Lesson 14
//! Application policy, not the full Goose application's automatic compaction assembly.
use super::*;
use goose_agent::inference::{InferenceRequestPreparer, PreparedInferenceRequest};
use goose_agent::operation::InferenceInput;

// Previously taught policy and pure mechanisms are supplied by the library.
use gdk_hello::learning::compaction::{
    self as mechanics, candidate, material_reduction, payload_bytes, MAX_COMPLETIONS,
};
pub use gdk_hello::learning::compaction::{
    Outcome, Policy, Report, MAX_REPLY_IDS, MAX_SETTLED_APPROVALS,
};
pub use gdk_hello::learning::record::{append_completion, completion_key, parse_learned};
#[cfg(test)]
use mechanics::{
    valid_summary_output, SUMMARY_BYTES, SUMMARY_INPUT_BYTES, SUMMARY_SOURCE_BOUNDARY,
};

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
    mechanics::check_estimate(measured, &session.policy)
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
        telemetry::inference_input(conversation.messages(), &prompt, &input.tools);
        tracing::info!(
            "course.estimated_bytes" = bytes,
            "course.allowance_bytes" = session.policy.total_bytes - session.policy.headroom_bytes
        );
        if bytes
            > session
                .policy
                .total_bytes
                .saturating_sub(session.policy.headroom_bytes)
        {
            return Err(anyhow::anyhow!("Inference refused: conservative input estimate is {} bytes, exceeding the {}-byte allowance after reserved output/protocol headroom. No repeated compaction or over-budget provider request was attempted.", bytes, session.policy.total_bytes - session.policy.headroom_bytes));
        }
        // Capture only accepted prepared input; preparation is not proof of delivery.
        self.captures.capture(&prompt).await;
        Ok(PreparedInferenceRequest {
            system_prompt: prompt,
            tools: input.tools,
            additional_messages: Vec::new(),
        })
    }
}

#[tracing::instrument(name = "learner_declaration", skip_all, fields(course.session_id = id, mlflow.spanType = "CHAIN", mlflow.spanInputs = concept), err)]
pub async fn declare(
    runtime: &ChatRuntime,
    id: &str,
    concept: &str,
    diagnostics: &mut DiagnosticLog,
    path: &PathBuf,
) -> anyhow::Result<()> {
    telemetry::identity_context(id);
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
    // Explicit /learned authorizes its append, independently of consent to load notes.
    let updated = append_completion(path, concept)?;
    tracing::info!(
        "course.declaration_saved" = true,
        "course.memory_consent" = session.lesson.persistence_enabled
    );
    let disposition = if session.lesson.persistence_enabled {
        "saved"
    } else {
        "saved; memory consent declined (context not reloaded)"
    };
    {
        let mut sessions = runtime.sessions.lock().await;
        let stored = match sessions.get_mut(id) {
            Some(stored) => stored,
            None => return Err(anyhow::anyhow!("missing Session")),
        };
        if session.lesson.persistence_enabled {
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
            "Completion declaration saved to learner-record.md (not assessed mastery). This conversation did not load the record, so its saved notes are not in tutor context.",
        );
    }
    Ok(())
}

fn partition(session: &ChatSession) -> anyhow::Result<Option<mechanics::Partition>> {
    mechanics::partition(
        &session.conversation,
        &session.genuine_reply_ids,
        &session.settled_approvals,
    )
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
    summary: gdk_hello::learning::summary::SummaryRunner,
    policy: Policy,
    pending_reply: Option<PendingReply>,
}

impl Compaction {
    pub fn new(
        provider: Arc<dyn Provider>,
        model: ModelConfig,
        policy: Policy,
    ) -> anyhow::Result<Self> {
        let path = PathBuf::from(REPOSITORY_ROOT)
            .join(CONTENT_DIRECTORY)
            .join("compaction-guidance.md");
        let guidance = fs::read_to_string(path)?;
        let summary = gdk_hello::learning::summary::SummaryRunner::new(provider, model, guidance)?;
        Ok(Self {
            summary,
            policy,
            pending_reply: None,
        })
    }

    #[tracing::instrument(name = "summarize_context", skip_all, fields(mlflow.spanType = "CHAIN"), err)]
    async fn summarize(
        &self,
        source: &[Message],
        genuine_reply_ids: &[String],
        parent_cancel: &CancellationToken,
    ) -> anyhow::Result<gdk_hello::learning::summary::SummaryResult> {
        self.summary
            .summarize(
                source,
                genuine_reply_ids,
                parent_cancel,
                gdk_hello::learning::summary::Observer {
                    input: telemetry::inference_input,
                    saved: telemetry::saved_message,
                    inference: telemetry::inference_message,
                },
            )
            .await
    }
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
            Ok(None) => report.disposition = "no-op: insufficient eligible older history (retain latest two genuine exchanges; at least 4000 eligible serialized bytes needed)".to_string(),
            Err(error) => report.disposition = format!("unsafe boundary: {}; history retained", error),
        }
        yielded_with([ChatEffect::Maintenance(Outcome {
            revision: session.revision,
            replacement,
            report,
        })])
    }
}

pub async fn maintain(
    operation: &Compaction,
    runtime: &ChatRuntime,
    id: &str,
    reason: &str,
    diagnostics: &mut DiagnosticLog,
) -> anyhow::Result<()> {
    maintain_with_pending(operation, runtime, id, reason, diagnostics, None).await
}

#[tracing::instrument(name = "context_maintenance", skip_all, fields(course.session_id = id, mlflow.spanType = "CHAIN", mlflow.spanInputs = reason), err)]
pub async fn maintain_with_pending(
    operation: &Compaction,
    runtime: &ChatRuntime,
    id: &str,
    reason: &str,
    diagnostics: &mut DiagnosticLog,
    pending: Option<&PendingReply>,
) -> anyhow::Result<()> {
    telemetry::identity_context(id);
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
        operation.summary.guidance(),
    )?;
    print_heading("Maintaining conversation context…");
    // A maintenance machine has one Operation and yields once; never enters tutor inference.
    let machine = StateMachine::new(
        vec![Step::Operation(Arc::new(Compaction {
            summary: operation.summary.clone(),
            policy: operation.policy.clone(),
            pending_reply: pending.cloned(),
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
        telemetry::maintenance_output(report);
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
