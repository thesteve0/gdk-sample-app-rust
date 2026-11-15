// Lesson 14
//! Isolated, tool-free summary inference. The caller owns state replacement and diagnostics.
use super::compaction::{
    summary_request, valid_summary_output, COMPACTION_GUIDANCE_BYTES, SUMMARY_BYTES,
    SUMMARY_MAX_TOKENS, SUMMARY_PARTIAL_MESSAGES,
};
use async_trait::async_trait;
use goose_agent::{
    events::AgentEvent,
    inference::{
        InferenceEffect, InferenceRequestPreparer, InferenceRunner, PreparedInferenceRequest,
    },
    machine::{EffectHandler, MachineSession, SessionLoader, StateMachine, Step},
    operation::{
        not_applicable, yielded_with, Emitter, InferenceInput, MachineEffect, Operation,
        OperationResult,
    },
};
use goose_providers::{
    base::Provider,
    conversation::{message::Message, token_usage::ProviderUsage, Conversation},
    model::ModelConfig,
};
use rmcp::model::Tool;
use serde_json::json;
use std::{sync::Arc, time::Duration};
use tokio::sync::{mpsc, Mutex};
use tokio_util::sync::CancellationToken;

const RUN_TIMEOUT_SECONDS: u64 = 180;
const CANCELLATION_GRACE_SECONDS: u64 = 5;
const STREAM_PROTOCOL_BYTES: usize = 2_000_000;

/// Optional observations only; callbacks must not block or confer write authority.
/// `inference` runs under the chat span captured at complete-message conversion,
/// immediately before `saved`, after the SDK has returned its inference effects.
#[derive(Clone)]
pub struct Observer {
    pub input: fn(&[Message], &str, &[Tool]),
    pub saved: fn(&Message),
    pub inference: fn(&Message),
}

fn ignore_input(_messages: &[Message], _prompt: &str, _tools: &[Tool]) {}
fn ignore_message(_message: &Message) {}

impl Default for Observer {
    fn default() -> Self {
        Self {
            input: ignore_input,
            saved: ignore_message,
            inference: ignore_message,
        }
    }
}

#[derive(Clone)]
pub struct SummaryRunner {
    provider: Arc<dyn Provider>,
    model: ModelConfig,
    guidance: String,
}

fn validate_guidance(guidance: &str) -> anyhow::Result<()> {
    if guidance.trim().is_empty() || guidance.len() > COMPACTION_GUIDANCE_BYTES {
        return Err(anyhow::anyhow!(
            "compaction guidance must be nonempty and at most 12000 bytes"
        ));
    }
    Ok(())
}

impl SummaryRunner {
    pub fn new(
        provider: Arc<dyn Provider>,
        mut model: ModelConfig,
        guidance: String,
    ) -> anyhow::Result<Self> {
        validate_guidance(&guidance)?;
        model.max_tokens = Some(SUMMARY_MAX_TOKENS);
        Ok(Self {
            provider,
            model,
            guidance,
        })
    }

    pub fn guidance(&self) -> &str {
        &self.guidance
    }

    // No learner files, paths or model-requested capabilities enter this runtime.
    pub async fn summarize(
        &self,
        source: &[Message],
        genuine_reply_ids: &[String],
        parent_cancel: &CancellationToken,
        observer: Observer,
    ) -> anyhow::Result<SummaryResult> {
        let (guidance, conversation) = summary_request(source, genuine_reply_ids, &self.guidance)?;
        (observer.input)(conversation.messages(), &guidance, &[]);
        let runtime = SummaryRuntime {
            state: Mutex::new(SummarySession {
                conversation,
                usage: Vec::new(),
                batches: 0,
            }),
            observer: observer.clone(),
        };
        let cancel = parent_cancel.child_token();
        let runner = InferenceRunner::new(self.provider.clone(), self.model.clone())
            .with_request_preparer(Arc::new(SummaryPreparer { observer }));
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
        let drain = drain_events(&mut receiver, &cancel, &finished);
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

/// Available summary data even when cancellation, limits or validation prevent use.
#[derive(Debug)]
pub struct SummaryResult {
    pub text: String,
    pub usage: Vec<ProviderUsage>,
    pub partial: Vec<Message>,
    pub failure: Option<String>,
}

// Retain the originating chat span with the output, without globals or task locals.
// The only effects this isolated runtime can apply are messages and usage.
enum SummaryEffect {
    AppendMessage(Message, tracing::Span),
    RecordUsage(ProviderUsage),
}

impl From<Message> for SummaryEffect {
    fn from(message: Message) -> Self {
        Self::AppendMessage(message, tracing::Span::current())
    }
}

impl InferenceEffect for SummaryEffect {
    fn record_usage(usage: ProviderUsage) -> Self {
        Self::RecordUsage(usage)
    }
}

impl MachineEffect for SummaryEffect {
    fn ensure_message_ids(&mut self) {
        if let Self::AppendMessage(message, _span) = self {
            let with_id = message.clone().with_generated_id_if_missing();
            *message = with_id;
        }
    }
}

struct SummaryPreparer {
    observer: Observer,
}

#[async_trait]
impl InferenceRequestPreparer<SummarySession> for SummaryPreparer {
    async fn prepare(
        &self,
        _session: &SummarySession,
        conversation: &Conversation,
        input: InferenceInput,
    ) -> anyhow::Result<PreparedInferenceRequest> {
        let prompt = input
            .prompt_parts
            .into_iter()
            .map(|(_name, text)| text)
            .collect::<Vec<_>>()
            .join("\n\n");
        (self.observer.input)(conversation.messages(), &prompt, &input.tools);
        Ok(PreparedInferenceRequest {
            system_prompt: prompt,
            tools: input.tools,
            additional_messages: Vec::new(),
        })
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
    observer: Observer,
}
#[async_trait]
impl SessionLoader<SummarySession> for SummaryRuntime {
    async fn load(&self, _id: &str) -> anyhow::Result<SummarySession> {
        Ok(self.state.lock().await.clone())
    }
}
#[async_trait]
impl EffectHandler<SummarySession, SummaryEffect> for SummaryRuntime {
    async fn apply_effects(
        &self,
        _session: &SummarySession,
        effects: &mut [SummaryEffect],
        _emit: &Emitter,
    ) -> anyhow::Result<()> {
        let mut state = self.state.lock().await;
        for effect in effects {
            match effect {
                SummaryEffect::AppendMessage(message, inference_span) => {
                    // Re-enter the span captured at From<Message>, not the caller's span.
                    // No await occurs while the span guard is entered.
                    inference_span.in_scope(|| (self.observer.inference)(message));
                    (self.observer.saved)(message);
                    state.conversation.push(message.clone());
                }
                SummaryEffect::RecordUsage(usage) => state.usage.push(usage.clone()),
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
impl Operation<SummarySession, SummaryEffect> for SummaryStop {
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
    ) -> anyhow::Result<OperationResult<SummaryEffect>> {
        if session.batches > 0 {
            return yielded_with([]);
        }
        not_applicable()
    }
}

// Completion closes the bounded queue even if the SDK retains an Emitter clone.
async fn drain_events(
    receiver: &mut mpsc::Receiver<AgentEvent>,
    cancel: &CancellationToken,
    finished: &CancellationToken,
) -> anyhow::Result<(bool, bool, Vec<Message>)> {
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
}

struct CooperativeRun<T> {
    result: Option<anyhow::Result<T>>,
    deadline_reached: bool,
    hard_abort: bool,
}

// Cancel first, then keep polling so the SDK can apply accumulated messages/usage.
async fn cooperative_run<T>(
    future: impl std::future::Future<Output = anyhow::Result<T>>,
    cancel: &CancellationToken,
    deadline: Duration,
    grace: Duration,
) -> CooperativeRun<T> {
    tokio::pin!(future);
    let deadline_reached = tokio::select! {
        result = &mut future => return CooperativeRun { result: Some(result), deadline_reached: false, hard_abort: false },
        _ = tokio::time::sleep(deadline) => true,
        _ = cancel.cancelled() => false,
    };
    cancel.cancel();
    match tokio::time::timeout(grace, &mut future).await {
        Ok(result) => CooperativeRun {
            result: Some(result),
            deadline_reached,
            hard_abort: false,
        },
        Err(_elapsed) => CooperativeRun {
            result: None,
            deadline_reached,
            hard_abort: true,
        },
    }
}

// Count received deltas, not the reconstructed answer; metadata/arguments count too.
#[derive(Default)]
struct StreamBudget {
    text_bytes: usize,
    protocol_bytes: usize,
}
impl StreamBudget {
    fn observe(&mut self, event: &AgentEvent, text_limit: usize) -> anyhow::Result<bool> {
        let protocol = match event {
            AgentEvent::Message(message) => {
                self.text_bytes = self
                    .text_bytes
                    .saturating_add(message.as_concat_text().len());
                serde_json::to_vec(message)?.len()
            }
            AgentEvent::Usage(usage) => serde_json::to_vec(usage)?.len(),
            AgentEvent::MessageUsage { message_id, usage } => {
                serde_json::to_vec(&json!({"id": message_id, "usage": usage}))?.len()
            }
            AgentEvent::McpNotification(notification) => serde_json::to_vec(notification)?.len(),
            AgentEvent::HistoryReplaced(conversation) => {
                serde_json::to_vec(conversation.messages())?.len()
            }
        };
        self.protocol_bytes = self.protocol_bytes.saturating_add(protocol);
        Ok(self.text_bytes > text_limit || self.protocol_bytes > STREAM_PROTOCOL_BYTES)
    }
}

#[cfg(test)]
mod tests;
