use std::{
    borrow::Cow,
    env, fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use anyhow::{anyhow, bail, Context, Result};
use async_trait::async_trait;
use futures::StreamExt;
use goose_agent::{
    inference::{InferenceEffect, InferenceRunner},
    machine::{EffectHandler, MachineSession, SessionLoader, StateMachine, Step},
    operation::{
        not_applicable, ConversationEffect, Emitter, MachineEffect, Operation, OperationResult,
    },
    tool::ToolOperation,
};
use goose_providers::{
    base::Provider,
    conversation::{
        message::{Message, MessageContent},
        token_usage::ProviderUsage,
        Conversation,
    },
    declarative::{from_json, EnvKeyResolver},
    model::ModelConfig,
};
use rmcp::{
    handler::server::router::tool::{SyncTool, ToolBase},
    model::{CallToolRequestParams, CallToolResult, ErrorData, Tool},
};
use rust_decimal::Decimal;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

const DEFAULT_PROVIDER_CONFIG: &str = "custom_aa_llama_qwen3_6-35b.json";
const TOOL_NAME: &str = "maximum_planned_loss";
const MAX_RAW_ROUNDS: usize = 3;
const MAX_TOOL_REQUESTS: usize = 4;
const MAX_MACHINE_STEPS: usize = 6;
const SYSTEM_INSTRUCTION: &str = "You are a day-trading teaching assistant. For the supplied hypothetical trade, use the maximum_planned_loss tool. Explain the deterministic result, explicitly note that it excludes fees, slippage, and a gap through the stop, and do not claim to place a trade.";
const USER_PROMPT: &str = "A hypothetical long trade enters at $51.20, uses a stop at $50.70, and has 200 shares. What is the maximum planned loss?";

#[derive(Debug, Default, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct RiskInput {
    /// Decimal dollar price, for example "51.20".
    entry_price: String,
    /// Decimal dollar stop price, for example "50.70".
    stop_price: String,
    share_count: u64,
}

#[derive(Debug, Serialize, JsonSchema)]
struct RiskOutput {
    entry_price: String,
    stop_price: String,
    share_count: u64,
    risk_per_share: String,
    maximum_planned_loss: String,
    exclusions: Vec<&'static str>,
}

struct MaximumPlannedLoss;

impl ToolBase for MaximumPlannedLoss {
    type Parameter = RiskInput;
    type Output = RiskOutput;
    type Error = ErrorData;

    fn name() -> Cow<'static, str> {
        TOOL_NAME.into()
    }

    fn description() -> Option<Cow<'static, str>> {
        Some("Calculate planned loss for a hypothetical long stock position from decimal-dollar entry and stop prices and a whole-number share count. This excludes fees, slippage, and gaps through the stop.".into())
    }
}

impl SyncTool<SpikeSession> for MaximumPlannedLoss {
    fn invoke(_session: &SpikeSession, input: RiskInput) -> Result<RiskOutput, ErrorData> {
        calculate_risk(input).map_err(|error| ErrorData::invalid_params(error.to_string(), None))
    }
}

fn calculate_risk(input: RiskInput) -> Result<RiskOutput> {
    if input.share_count == 0 {
        bail!("share_count must be greater than zero");
    }
    let entry = parse_price("entry_price", &input.entry_price)?;
    let stop = parse_price("stop_price", &input.stop_price)?;
    if stop >= entry {
        bail!("stop_price must be below entry_price for this long-position tool");
    }

    let risk_per_share = entry - stop;
    let maximum_loss = risk_per_share
        .checked_mul(Decimal::from(input.share_count))
        .ok_or_else(|| anyhow!("maximum planned loss overflowed"))?;

    Ok(RiskOutput {
        entry_price: money(entry),
        stop_price: money(stop),
        share_count: input.share_count,
        risk_per_share: money(risk_per_share),
        maximum_planned_loss: money(maximum_loss),
        exclusions: vec!["fees", "slippage", "gap through the stop"],
    })
}

fn parse_price(field: &str, value: &str) -> Result<Decimal> {
    let decimal: Decimal = value
        .parse()
        .with_context(|| format!("{field} must be a decimal string"))?;
    if decimal <= Decimal::ZERO {
        bail!("{field} must be greater than zero");
    }
    if decimal.scale() > 4 {
        bail!("{field} supports at most four decimal places");
    }
    Ok(decimal)
}

fn money(value: Decimal) -> String {
    format!("{value:.2}")
}

fn tool_definition() -> Tool {
    let schema = serde_json::from_value(json!({
        "type": "object",
        "properties": {
            "entry_price": {"type": "string", "pattern": "^[0-9]+(\\.[0-9]{1,4})?$"},
            "stop_price": {"type": "string", "pattern": "^[0-9]+(\\.[0-9]{1,4})?$"},
            "share_count": {"type": "integer", "minimum": 1}
        },
        "required": ["entry_price", "stop_price", "share_count"],
        "additionalProperties": false
    }))
    .expect("static input schema must be an object");
    Tool::new(
        TOOL_NAME,
        "Calculate maximum planned loss for a hypothetical long stock position. Prices are decimal-dollar strings.",
        Arc::new(schema),
    )
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    let (mode, config_path) = arguments()?;
    let (provider, model) = load_provider(&config_path)?;

    match mode.as_str() {
        "raw" => run_raw(provider.as_ref(), &model).await?,
        "state" => run_state_machine(provider, model).await?,
        "all" => {
            run_raw(provider.as_ref(), &model).await?;
            run_state_machine(provider, model).await?;
        }
        _ => unreachable!(),
    }
    Ok(())
}

fn arguments() -> Result<(String, PathBuf)> {
    let mut mode = "all".to_string();
    let mut path = None;
    for argument in env::args().skip(1) {
        match argument.as_str() {
            "--raw" => mode = "raw".to_string(),
            "--state" => mode = "state".to_string(),
            "--all" => mode = "all".to_string(),
            value if path.is_none() => path = Some(PathBuf::from(value)),
            _ => bail!("Usage: cargo run --example post-lesson4-spike -- [--raw|--state|--all] [provider.json]"),
        }
    }
    let path = path.unwrap_or_else(|| PathBuf::from(DEFAULT_PROVIDER_CONFIG));
    if !path.exists() {
        bail!("Provider configuration not found: {}", path.display());
    }
    Ok((mode, path))
}

fn load_provider(path: &Path) -> Result<(Arc<dyn Provider>, ModelConfig)> {
    let provider_json = fs::read_to_string(path)?;
    let config: Value = serde_json::from_str(&provider_json)?;
    let model_name = config
        .get("models")
        .and_then(Value::as_array)
        .and_then(|models| models.first())
        .and_then(|model| model.get("name"))
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("Provider JSON has no configured model"))?;
    let provider: Arc<dyn Provider> = from_json(&provider_json, None, EnvKeyResolver {})?.into();
    Ok((provider, ModelConfig::new(model_name)))
}

async fn run_raw(provider: &dyn Provider, model: &ModelConfig) -> Result<()> {
    println!("\n=== raw provider protocol ===");
    let tools = [tool_definition()];
    let mut messages = vec![Message::user().with_text(USER_PROMPT)];
    let mut total_requests = 0;

    for round in 1..=MAX_RAW_ROUNDS {
        let response = collect_stream(provider, model, &messages, &tools).await?;
        let requests = response
            .iter()
            .flat_map(|message| message.content.iter())
            .filter_map(MessageContent::as_tool_request)
            .cloned()
            .collect::<Vec<_>>();
        messages.extend(response);

        if requests.is_empty() {
            let final_text = messages
                .iter()
                .rev()
                .find(|message| message.role == rmcp::model::Role::Assistant)
                .map(Message::as_concat_text)
                .unwrap_or_default();
            if final_text.trim().is_empty() {
                bail!("raw round {round} ended without a tool request or final text");
            }
            validate_final_answer(&final_text)?;
            println!("final answer: {final_text}");
            return Ok(());
        }

        total_requests += requests.len();
        if total_requests > MAX_TOOL_REQUESTS {
            bail!("raw tool-request limit exceeded ({MAX_TOOL_REQUESTS})");
        }

        let mut response_message = Message::user();
        for request in requests {
            println!("request id={} call={:?}", request.id, request.tool_call);
            let result = dispatch_raw(request.tool_call.as_ref());
            response_message.add_tool_response_with_metadata(
                request.id,
                result,
                request.metadata.as_ref(),
            );
        }
        messages.push(response_message);
    }

    bail!("raw round limit exceeded ({MAX_RAW_ROUNDS})")
}

fn dispatch_raw(
    call: Result<&CallToolRequestParams, &ErrorData>,
) -> Result<CallToolResult, ErrorData> {
    let call = call.map_err(Clone::clone)?;
    if call.name.as_ref() != TOOL_NAME {
        return Err(ErrorData::invalid_params(
            format!("unknown or disallowed tool: {}", call.name),
            None,
        ));
    }
    let input: RiskInput = serde_json::from_value(Value::Object(
        call.arguments.clone().unwrap_or_default(),
    ))
    .map_err(|error| ErrorData::invalid_params(format!("invalid arguments: {error}"), None))?;
    let output = calculate_risk(input)
        .map_err(|error| ErrorData::invalid_params(error.to_string(), None))?;
    let structured = serde_json::to_value(output)
        .map_err(|error| ErrorData::internal_error(error.to_string(), None))?;
    Ok(CallToolResult::structured(structured))
}

async fn collect_stream(
    provider: &dyn Provider,
    model: &ModelConfig,
    messages: &[Message],
    tools: &[Tool],
) -> Result<Vec<Message>> {
    let mut stream = provider
        .stream(model, SYSTEM_INSTRUCTION, messages, tools)
        .await?;
    // Stream chunks may share a message ID. Conversation::push applies the GDK's
    // merge semantics so history contains complete messages rather than deltas.
    let mut response = Conversation::empty();
    while let Some((message, usage)) = stream.next().await.transpose()? {
        if let Some(message) = message {
            response.push(message);
        }
        if let Some(usage) = usage {
            eprintln!("raw usage: {usage:#?}");
        }
    }
    if response.is_empty() {
        bail!("provider stream returned no messages");
    }
    Ok(response.messages().clone())
}

#[derive(Clone)]
struct SpikeSession {
    id: String,
    conversation: Conversation,
}

impl MachineSession for SpikeSession {
    fn id(&self) -> &str {
        &self.id
    }

    fn conversation(&self) -> Option<&Conversation> {
        Some(&self.conversation)
    }
}

enum SpikeEffect {
    Conversation(ConversationEffect),
    Usage(ProviderUsage),
}

impl From<Message> for SpikeEffect {
    fn from(message: Message) -> Self {
        Self::Conversation(ConversationEffect::AppendMessage(message))
    }
}

impl InferenceEffect for SpikeEffect {
    fn record_usage(usage: ProviderUsage) -> Self {
        Self::Usage(usage)
    }
}

impl MachineEffect for SpikeEffect {
    fn ensure_message_ids(&mut self) {
        if let Self::Conversation(effect) = self {
            effect.ensure_message_ids();
        }
    }
}

struct InMemoryRuntime {
    session: Mutex<SpikeSession>,
}

#[async_trait]
impl SessionLoader<SpikeSession> for InMemoryRuntime {
    async fn load(&self, session_id: &str) -> Result<SpikeSession> {
        let session = self.session.lock().expect("runtime mutex poisoned");
        if session.id != session_id {
            bail!("unknown session: {session_id}");
        }
        Ok(session.clone())
    }
}

#[async_trait]
impl EffectHandler<SpikeSession, SpikeEffect> for InMemoryRuntime {
    async fn apply_effects(
        &self,
        _session: &SpikeSession,
        effects: &mut [SpikeEffect],
        _emit: &Emitter,
    ) -> Result<()> {
        let mut stored = self.session.lock().expect("runtime mutex poisoned");
        for effect in effects {
            match effect {
                SpikeEffect::Conversation(ConversationEffect::AppendMessage(message)) => {
                    stored.conversation.push(message.clone());
                }
                SpikeEffect::Conversation(ConversationEffect::ReplaceConversation(
                    conversation,
                )) => {
                    stored.conversation = conversation.clone();
                }
                SpikeEffect::Conversation(ConversationEffect::PatchToolRequestMeta { .. })
                | SpikeEffect::Conversation(ConversationEffect::SetMessageVisibility { .. }) => {
                    bail!("spike runtime does not implement metadata patch effects")
                }
                SpikeEffect::Usage(usage) => eprintln!("state-machine usage: {usage:#?}"),
            }
        }
        Ok(())
    }
}

struct DomainPrompt;

#[async_trait]
impl Operation<SpikeSession, SpikeEffect> for DomainPrompt {
    fn name(&self) -> &'static str {
        "domain_prompt"
    }

    async fn prompt_parts(
        &self,
        _session: &SpikeSession,
        _conversation: &Conversation,
    ) -> Result<Vec<(String, String)>> {
        Ok(vec![("domain".to_string(), SYSTEM_INSTRUCTION.to_string())])
    }

    async fn run(
        &self,
        _session: &SpikeSession,
        _conversation: &Conversation,
        _emit: &Emitter,
    ) -> Result<OperationResult<SpikeEffect>> {
        not_applicable()
    }
}

async fn run_state_machine(provider: Arc<dyn Provider>, model: ModelConfig) -> Result<()> {
    println!("\n=== goose-agent state machine ===");
    let runtime = InMemoryRuntime {
        session: Mutex::new(SpikeSession {
            id: "spike".to_string(),
            conversation: Conversation::new([Message::user().with_text(USER_PROMPT)])?,
        }),
    };
    let cancel = CancellationToken::new();
    let tools = ToolOperation::<SpikeSession>::new().with_sync_tool::<MaximumPlannedLoss>();
    let inference = InferenceRunner::<SpikeSession, SpikeEffect>::new(provider, model);
    let machine = StateMachine::new(
        vec![
            Step::Operation(Arc::new(tools)),
            Step::Operation(Arc::new(DomainPrompt)),
            Step::Inference(Arc::new(inference)),
        ],
        cancel.clone(),
    );
    let (event_tx, mut event_rx) = mpsc::channel(32);
    let emitter = Emitter::new(event_tx, cancel);
    let event_drain = tokio::spawn(async move { while event_rx.recv().await.is_some() {} });

    let session = run_machine_bounded(&machine, &runtime, "spike", &emitter).await?;
    drop(emitter);
    event_drain.await?;

    let final_text = session
        .conversation
        .messages()
        .iter()
        .rev()
        .find(|message| message.role == rmcp::model::Role::Assistant)
        .map(Message::as_concat_text)
        .unwrap_or_default();
    if final_text.trim().is_empty() {
        bail!("state machine stopped without a final assistant answer");
    }
    validate_final_answer(&final_text)?;
    println!("final answer: {final_text}");
    Ok(())
}

fn validate_final_answer(text: &str) -> Result<()> {
    let lowercase = text.to_lowercase();
    for required in ["100.00", "fees", "slippage", "gap"] {
        if !lowercase.contains(required) {
            bail!("final answer omitted required evidence/caveat: {required}");
        }
    }
    Ok(())
}

async fn run_machine_bounded<'a>(
    machine: &StateMachine<'a, SpikeSession, SpikeEffect>,
    runtime: &InMemoryRuntime,
    session_id: &str,
    emitter: &Emitter,
) -> Result<SpikeSession> {
    for step_number in 1..=MAX_MACHINE_STEPS {
        let session = runtime.load(session_id).await?;
        let Some(mut result) = machine.step(&session, emitter).await? else {
            println!(
                "state machine stopped after {} applied steps",
                step_number - 1
            );
            return runtime.load(session_id).await;
        };
        println!(
            "step {step_number}: {}",
            result.applied_step.unwrap_or("unknown")
        );
        machine
            .apply(runtime, &session, &mut result, emitter)
            .await?;
        if result.yield_to_client {
            return runtime.load(session_id).await;
        }
    }
    bail!("state-machine step limit exceeded ({MAX_MACHINE_STEPS})")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decimal_strings_produce_exact_expected_loss() {
        let output = calculate_risk(RiskInput {
            entry_price: "51.20".to_string(),
            stop_price: "50.70".to_string(),
            share_count: 200,
        })
        .unwrap();
        assert_eq!(output.risk_per_share, "0.50");
        assert_eq!(output.maximum_planned_loss, "100.00");
    }

    #[test]
    fn rejects_unknown_fields_and_invalid_domain_values() {
        let malformed = serde_json::from_value::<RiskInput>(json!({
            "entry_price": "51.20",
            "stop_price": "50.70",
            "share_count": 200,
            "execute_trade": true
        }));
        assert!(malformed.is_err());

        let invalid = calculate_risk(RiskInput {
            entry_price: "50.00".to_string(),
            stop_price: "51.00".to_string(),
            share_count: 200,
        });
        assert!(invalid.is_err());
    }

    #[test]
    fn integer_cents_are_exact_but_less_natural_at_the_boundary() {
        let entry_cents = 5_120_i64;
        let stop_cents = 5_070_i64;
        let shares = 200_i64;
        assert_eq!((entry_cents - stop_cents) * shares, 10_000);
    }

    #[test]
    fn unknown_tool_is_rejected_by_raw_allowlist() {
        let call = CallToolRequestParams::new("place_order");
        assert!(dispatch_raw(Ok(&call)).is_err());
    }

    #[test]
    fn final_answer_requires_result_and_caveats() {
        assert!(
            validate_final_answer("The result is 100.00 before fees, slippage, or a gap.").is_ok()
        );
        assert!(validate_final_answer("The result is 100.00.").is_err());
    }
}
