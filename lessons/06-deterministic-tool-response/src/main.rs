use std::{env, error::Error, fs, path::PathBuf, sync::Arc};

use futures::StreamExt;
use goose_providers::{
    base::Provider,
    conversation::{
        effective_role,
        message::{Message, MessageContentBlock, ToolRequest, ToolResult},
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

const DEFAULT_PROVIDER_CONFIG: &str = "custom_aa_llama_qwen3_6-35b.json";
const TOOL_NAME: &str = "maximum_planned_loss";

/// A delimiter printed before and after every block of protocol payload: the
/// outbound prompts and tool definition, the model's output, and the tool
/// responses this application sends back. Everything between two delimiter
/// lines is a value that traveled to or from the provider; everything else on
/// the terminal is this application's explanatory text.
const PAYLOAD_DELIMITER: &str = "++++++++";

const SYSTEM_INSTRUCTION: &str = "You are a day-trading teaching assistant. For the supplied hypothetical trade, use the maximum_planned_loss tool. Never claim to place, modify, or cancel a trade.";
const USER_PROMPT: &str = "A hypothetical long trade enters at $51.20, uses a stop at $50.70, and has 200 shares. What is the maximum planned loss?";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv().ok();

    let provider_config_path = provider_config_path()?;
    let provider_json = fs::read_to_string(&provider_config_path)?;
    let provider_config: Value = serde_json::from_str(&provider_json)?;
    let provider = from_json(&provider_json, None, EnvKeyResolver {})?;
    let model = ModelConfig::new(first_configured_model(&provider_config)?);

    // The advertised tool is unchanged from Lesson 5. This lesson completes the
    // round trip: the application alone validates, allowlists, and executes it.
    let tools = [tool_definition()];
    let first_messages = vec![Message::user().with_text(USER_PROMPT)];

    // Phase 1: what the application sends in round 1. The tool is advertised,
    // never run. The full outbound payload is these prompts plus the advertised
    // tool definition: the entire context the model sees in the first call.
    println!("── Sending ─────────────────────────────────────────────");
    println!("1 user message; advertising 1 tool: {}", TOOL_NAME);
    println!("\nOutbound payload sent with this call (application → provider), quoted in full:");
    print_delimiter();
    println!("  system instruction:");
    print_indented(SYSTEM_INSTRUCTION, "    > ");
    println!("  user message:");
    print_indented(USER_PROMPT, "    > ");
    print_tool_definition(&tools[0])?;
    print_delimiter();

    // Phase 2: round 1 — the model may request the tool.
    println!("\n── Round 1: streaming response deltas (provider → application) ──");
    let (conversation, round1_usage) =
        stream_and_collect(provider.as_ref(), &model, &first_messages, &tools).await?;
    print_usage_summary(&round1_usage);

    // Phase 3: inspect every reconstructed block, exactly as in Lesson 5.
    println!(
        "\n── Round 1 reconstructed messages (provider → application, after Conversation::push) ──"
    );
    inspect_reconstructed(&conversation)?;

    // Phase 4: validate and dispatch the requests this run received. This is a
    // bounded two-round exchange, not an open agent loop.
    let pending = pending_tool_requests(&conversation);
    if pending.is_empty() {
        print_stopped_without_request();
        return Ok(());
    }

    // The provider is stateless: the follow-up call must resend the entire
    // history. It starts from the original user message, then gains the
    // assistant tool-request message(s) and one correlated response per request.
    let mut history = vec![Message::user().with_text(USER_PROMPT)];
    history.extend(conversation.messages().iter().cloned());

    println!("\n── Validate, dispatch, and respond ─────────────────────");
    for request in pending {
        let result = execute_allowed_tool(&request.tool_call);
        match &result {
            Ok(response) => {
                let mut text = String::new();
                for block in &response.content {
                    if let Some(content) = block.as_text() {
                        text.push_str(&content.text);
                    }
                }
                println!(
                    "  ✓ request {} → deterministic result (application, as the tool → provider)",
                    request.id
                );
                print_delimiter();
                print_indented(&text, "    ");
                print_delimiter();
            }
            Err(error) => {
                println!(
                    "  ✗ request {} → error result (application, as the tool → provider)",
                    request.id
                );
                print_delimiter();
                print_indented(&error.message, "    ");
                print_delimiter();
            }
        }
        // One correlated user-role response per request. The request ID is the
        // correlation key: the provider matches each result to its request
        // through it, never by order or name.
        history.push(Message::user().with_tool_response(request.id.clone(), result));
    }

    // Phase 5: round 2 — the stateless follow-up call.
    println!("\n── Round 2: the stateless follow-up call ───────────────");
    print_outbound_history(&history, tools.len());
    println!("\nFinal educational answer (provider → application, streaming):");
    let (final_conversation, round2_usage) =
        stream_and_collect(provider.as_ref(), &model, &history, &tools).await?;
    print_usage_summary(&round2_usage);

    // Phase 6: name the step this run reached inside the same cycle.
    print_where_the_run_ends(pending_tool_requests(&final_conversation).is_empty());

    Ok(())
}

/// Stream one inference call, printing text as it arrives and reconstructing the
/// complete messages with GDK merge semantics. Returns the reconstructed
/// Conversation and the call's usage telemetry. Unchanged from Lesson 5: both
/// rounds of this lesson use the same streaming helper.
async fn stream_and_collect(
    provider: &dyn Provider,
    model: &ModelConfig,
    messages: &[Message],
    tools: &[Tool],
) -> Result<(Conversation, Option<ProviderUsage>), Box<dyn Error>> {
    let mut stream = provider
        .stream(model, SYSTEM_INSTRUCTION, messages, tools)
        .await?;

    // A streamed response is not one stream item and one response. Conversation::push
    // coalesces the partial deltas that share a message back into complete messages.
    let mut conversation = Conversation::empty();
    let mut streamed_text = false;
    let mut usage = None;
    while let Some((message, item_usage)) = stream.next().await.transpose()? {
        if let Some(message) = message {
            let text = message.as_concat_text();
            if !text.is_empty() {
                // Streamed text is the model's output, not commentary. Open the
                // delimiter at the first delta and close it after the stream ends;
                // individual deltas cannot each carry a fence of their own.
                if !streamed_text {
                    print_delimiter();
                }
                print!("{}", text);
                streamed_text = true;
            }
            // Preserve the complete assistant-role message, structured blocks included.
            conversation.push(message);
        }
        if let Some(item_usage) = item_usage {
            usage = Some(item_usage);
        }
    }

    if streamed_text {
        // Streamed deltas do not guarantee a trailing newline, so start the
        // closing delimiter on a fresh line.
        println!();
        print_delimiter();
    } else {
        println!(
            "(no text blocks streamed; the response arrived as non-text blocks — see the reconstructed view below)"
        );
    }

    Ok((conversation, usage))
}

/// Print one compact usage line. A finish reason of `tool_calls` in round 1
/// means the model ended its turn expecting a tool result: display it, never
/// act on it. This lesson acts on the request itself, not the finish reason.
fn print_usage_summary(usage: &Option<ProviderUsage>) {
    println!("\n── Relevant response metadata (Usage block of the response) ──");
    let Some(usage) = usage else {
        println!("(no usage telemetry was provided by the provider)");
        return;
    };
    let input_tokens = match usage.usage.input_tokens {
        Some(count) => count.to_string(),
        None => "-".to_string(),
    };
    let output_tokens = match usage.usage.output_tokens {
        Some(count) => count.to_string(),
        None => "-".to_string(),
    };
    println!(
        "model={}  tokens in={} out={}",
        usage.model, input_tokens, output_tokens
    );
    if let Some(reasons) = &usage.finish_reasons {
        println!("finish_reasons={:?}", reasons);
        if reasons.iter().any(|reason| reason == "tool_calls") {
            println!("  ← the model ended its turn expecting a tool result.");
            println!("    The application, not the finish reason, decides what happens next.");
        }
    }
}

/// Inspect every content block across every reconstructed message, as in
/// Lesson 5. A single streamed response can carry several blocks, and one
/// response can carry more than one tool request.
fn inspect_reconstructed(conversation: &Conversation) -> Result<(), Box<dyn Error>> {
    for (message_index, message) in conversation.messages().iter().enumerate() {
        println!(
            "message {}: role={:?}, {} content block(s)",
            message_index + 1,
            message.role,
            message.content.len()
        );

        for (block_index, block) in message.content.iter().enumerate() {
            let label = format!("block {}/{}:", block_index + 1, message.content.len());

            if let Some(request) = block.as_tool_request() {
                println!(
                    "  {} ToolRequest — the model asks the application to run a tool",
                    label
                );
                print_delimiter();
                // The request ID is the correlation ID: the response below must
                // attach it so the provider can match result to request.
                println!("    request id (correlation ID): {}", request.id);
                // `tool_call` is a Result: tool-call parsing can fail inside a
                // request, so never blindly unwrap it.
                match &request.tool_call {
                    Ok(call) => {
                        println!("    tool name: {}", call.name);
                        println!(
                            "    arguments (untrusted model output; JSON object, order not guaranteed):"
                        );
                        match &call.arguments {
                            Some(arguments) => {
                                print_indented(&serde_json::to_string_pretty(arguments)?, "      ")
                            }
                            None => println!("      (none)"),
                        }
                    }
                    Err(error) => {
                        println!(
                            "    unparseable call (reported without panicking): {}",
                            error
                        );
                    }
                }
                print_delimiter();
            } else if let Some(thinking) = block.as_thinking() {
                println!("  {} Thinking block — Not ground truth", label);
                print_delimiter();
                print_indented(&thinking.thinking, "             ");
                print_delimiter();
            } else if let Some(text) = block.as_text() {
                if !text.trim().is_empty() {
                    println!("  {} Text", label);
                    print_delimiter();
                    print_indented(text, "    ");
                    print_delimiter();
                }
            } else {
                println!("  {} other non-text block: {}", label, block);
            }
        }
    }

    Ok(())
}

/// Every ToolRequest block across the reconstructed messages, in order. A
/// single response can carry more than one request; each one receives exactly
/// one correlated response below.
fn pending_tool_requests(conversation: &Conversation) -> Vec<&ToolRequest> {
    let mut requests = Vec::new();
    for message in conversation.messages() {
        for block in &message.content {
            if let Some(request) = block.as_tool_request() {
                requests.push(request);
            }
        }
    }
    requests
}

/// Validate and execute one pending request through three boundaries, in order:
///
/// 1. parse: `tool_call` is a `Result`; an unparseable call is never unwrapped;
/// 2. allowlist: only a known tool name dispatches — the application, not the
///    model, decides what is executable;
/// 3. domain validation and deterministic execution.
///
/// Every failure becomes a protocol-valid error result the model can read. The
/// program never panics on model output and never drops a request.
fn execute_allowed_tool(
    tool_call: &ToolResult<CallToolRequestParams>,
) -> ToolResult<CallToolResult> {
    // Boundary 1: tool_call is a Result because parsing can fail inside a
    // request. Extract the parsed call, or turn the parse error into a
    // protocol-valid error response.
    let parsed_call = match tool_call {
        Ok(call) => call,
        Err(error) => {
            let failure = format!("the tool call could not be parsed: {}", error.message);
            return Err(ErrorData::invalid_params(failure, None));
        }
    };

    // Boundary 2: the allowlist. The model names the tool it wants; only the
    // name this application advertises may execute.
    let requested_name = parsed_call.name.as_ref();
    if requested_name != TOOL_NAME {
        let failure = format!(
            "tool '{}' is not on this application's allowlist; '{}' is the only executable tool",
            requested_name, TOOL_NAME
        );
        return Err(ErrorData::invalid_params(failure, None));
    }

    // Boundary 3: the arguments. A well-formed call may still carry none.
    let arguments = match &parsed_call.arguments {
        Some(arguments) => arguments,
        None => {
            return Err(ErrorData::invalid_params(
                "the tool call carried no arguments".to_string(),
                None,
            ));
        }
    };

    // All three boundaries passed. A domain Err(String) becomes the protocol
    // error the model will read.
    match execute_maximum_planned_loss(arguments) {
        Ok(loss) => Ok(CallToolResult::success(vec![ContentBlock::text(loss)])),
        Err(failure) => Err(ErrorData::invalid_params(failure, None)),
    }
}

/// The untrusted argument shape. Deserialization is the shape boundary: wrong
/// types, missing required fields, and unknown fields fail here, before any
/// domain code runs. Prices stay strings so parsing stays an explicit domain
/// decision — serde_json would otherwise hand back binary floating point.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LossArguments {
    entry_price: String,
    stop_price: String,
    share_count: u32,
}

/// Validate and execute the one allowlisted tool. Deserialization checked the
/// argument shape; domain validation checks the values the advertised schema
/// cannot enforce. The result is exact decimal arithmetic, formatted to two
/// places, and excludes fees, slippage, and a gap through the stop.
fn execute_maximum_planned_loss(arguments: &JsonObject) -> Result<String, String> {
    // Shape boundary: deserialize the untrusted arguments into the domain
    // struct. Wrong types, missing fields, and unknown fields fail here.
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

/// Parse one untrusted decimal-string price with exact decimal arithmetic.
/// Never binary floating point: money arithmetic must be exact. At most four
/// decimal places and a positive value are domain rules, not schema rules.
fn parse_price(field: &str, raw: &str) -> Result<Decimal, String> {
    // Try to read the string as an exact decimal. The leading underscore in
    // _parse_error marks the parse library's own error as deliberately
    // unused: the String reason below is what the model will read.
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

/// Print the full history the stateless follow-up call resends: the original
/// user message, the assistant tool-request message(s), and one user-role
/// tool-response message per request. A user-role message carrying a tool
/// response has effective role `tool` — the role the provider serializes it as.
fn print_outbound_history(history: &[Message], advertised_tool_count: usize) {
    println!("The provider is stateless: the follow-up call resends the entire history.");
    println!(
        "outbound (application → provider): {} advertised tool(s) and {} message(s):",
        advertised_tool_count,
        history.len()
    );
    for (index, message) in history.iter().enumerate() {
        let mut descriptions: Vec<String> = Vec::new();
        for block in &message.content {
            descriptions.push(describe_block(block));
        }
        let blocks = if descriptions.is_empty() {
            "(none)".to_string()
        } else {
            descriptions.join(", ")
        };
        println!(
            "  {}. role={:?}, effective role={}, block(s): {}",
            index + 1,
            message.role,
            effective_role(message),
            blocks
        );
    }
}

/// Label one content block for the outbound-history view.
fn describe_block(block: &MessageContentBlock) -> String {
    if let Some(request) = block.as_tool_request() {
        format!("tool request (id {})", request.id)
    } else if let Some(response) = block.as_tool_response() {
        format!("tool response to id {}", response.id)
    } else if block.as_thinking().is_some() {
        "thinking".to_string()
    } else if block.as_text().is_some() {
        "text".to_string()
    } else {
        "other".to_string()
    }
}

/// The no-request path: advertising a tool permits a request but does not
/// cause one. Without a request there is nothing to execute and no round trip.
fn print_stopped_without_request() {
    println!("\n── We stopped here ─────────────────────────────────────");
    println!("No tool request arrived, so the Lesson 6 round trip could not begin:");
    println!("nothing executed, no tool response was created, and no follow-up call");
    println!("was made. The complete assistant-role message is retained in memory only");
    println!("for this run; the program performs no second inference round.");
}

/// Print the same request/response cycle Lesson 5 listed, now naming the step
/// this run reached. With a streamed final answer and no re-request, the cycle
/// is complete; with a re-request, the run stops at its explicit two-round bound.
fn print_where_the_run_ends(final_round_had_no_request: bool) {
    println!("\n── Where this run ends ─────────────────────────────────");
    println!("The full request/response cycle for a tool-using turn:");
    println!("  1. send system instruction + user message (+ the advertised tool)");
    println!("  2. model responds with thinking + a tool request");
    println!("  3. application validates the request and executes the tool");
    println!("  4. send the full history + tool response; model writes the final answer");
    println!("  5. application returns the final answer to the user");
    println!();

    if final_round_had_no_request {
        println!("This run completed steps 3–5. The tool response carried only the");
        println!("deterministic number; the streamed answer above is the model's");
        println!("follow-up explanation built on that result. The tool description told");
        println!("the model the calculation excludes fees, slippage, and a gap through");
        println!("the stop — whether this run's answer repeats those exclusions is model");
        println!("behavior to observe, not a structural guarantee. This program still");
        println!("performs exactly two inference rounds by construction, not an open");
        println!("agent loop; Lesson 7 replaces this fixed structure with a state machine.");
    } else {
        println!("The model requested a tool again in round 2. This lesson performs exactly");
        println!("two inference rounds and dispatches only the requests received in round 1,");
        println!("so the run stops at that bound instead of looping. Lesson 7 replaces this");
        println!("fixed two-round structure with a state machine.");
    }
}

/// Print one payload delimiter line. Every payload block opens and closes
/// with it, so a reader can see exactly where commentary ends and provider
/// payload begins. The line printed immediately before each block names the
/// sender and the receiver of that message: the round trip has multiple
/// actors (this application, the provider, and this application acting as
/// the tool), so every message must show both of its ends.
fn print_delimiter() {
    println!("{}", PAYLOAD_DELIMITER);
}

/// Print multi-line text with a fixed indent so outbound prompts, streamed
/// prose, private reasoning, advertised schemas, and structured arguments
/// stay visually inside their labeled block.
fn print_indented(text: &str, indent: &str) {
    for line in text.lines() {
        println!("{}{}", indent, line);
    }
}

/// Print the advertised tool definition the model receives with the prompts:
/// name, description, and the JSON input schema. All three travel in the same
/// inference call, so they belong in the outbound payload view.
fn print_tool_definition(tool: &Tool) -> Result<(), Box<dyn Error>> {
    println!("  advertised tool definition:");
    println!("    name: {}", tool.name);
    if let Some(description) = &tool.description {
        println!("    description:");
        print_indented(description, "      > ");
    }
    println!("    input schema (advertised shape, not runtime validation):");
    print_indented(
        &serde_json::to_string_pretty(tool.input_schema.as_ref())?,
        "      ",
    );
    Ok(())
}

/// The one narrowly described advertised schema for this lesson. It tells the
/// model the expected shape but does not validate requests or authorize execution.
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

fn provider_config_path() -> Result<PathBuf, Box<dyn Error>> {
    let mut arguments = env::args_os();
    let _program = arguments.next();
    let path = match (arguments.next(), arguments.next()) {
        (Some(path), None) => PathBuf::from(path),
        (None, None) => PathBuf::from(DEFAULT_PROVIDER_CONFIG),
        _ => return Err("Usage: cargo run -- [provider.json]".into()),
    };

    if !path.exists() {
        return Err(format!(
            "Provider configuration not found: {}. Run from the repository root or provide a valid JSON path.",
            path.display()
        )
            .into());
    }

    Ok(path)
}

fn first_configured_model(provider_config: &Value) -> Result<String, Box<dyn Error>> {
    let missing = "Provider JSON has no configured model";

    let models = match provider_config.get("models").and_then(Value::as_array) {
        Some(models) => models,
        None => return Err(missing.into()),
    };
    let first_model = match models.first() {
        Some(first_model) => first_model,
        None => return Err(missing.into()),
    };
    let name = match first_model.get("name").and_then(Value::as_str) {
        Some(name) => name,
        None => return Err(missing.into()),
    };

    Ok(name.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_definition_advertises_one_narrow_allowedlist_tool() {
        let tool = tool_definition();
        assert_eq!(tool.name.as_ref(), TOOL_NAME);

        let schema = tool.input_schema.as_ref();
        assert_eq!(
            schema.get("additionalProperties").and_then(Value::as_bool),
            Some(false),
            "unknown fields must be rejected"
        );

        let mut required: Vec<&str> = Vec::new();
        if let Some(array) = schema.get("required").and_then(Value::as_array) {
            for entry in array {
                if let Some(name) = entry.as_str() {
                    required.push(name);
                }
            }
        }
        assert!(required.contains(&"entry_price"));
        assert!(required.contains(&"stop_price"));
        assert!(required.contains(&"share_count"));
    }

    #[test]
    fn scenario_arguments_execute_to_the_exact_dollar_result() {
        let loss = execute_maximum_planned_loss(scenario_arguments().as_object().unwrap()).unwrap();
        assert_eq!(loss, "$100.00");
    }

    #[test]
    fn a_parseable_allowed_request_returns_the_result_text() {
        let call = CallToolRequestParams::new(TOOL_NAME)
            .with_arguments(scenario_arguments().as_object().unwrap().clone());
        let result = execute_allowed_tool(&Ok(call)).unwrap();
        let mut text = String::new();
        for block in &result.content {
            if let Some(content) = block.as_text() {
                text.push_str(&content.text);
            }
        }
        assert_eq!(text, "$100.00");
    }

    #[test]
    fn single_decimal_place_prices_are_accepted() {
        let arguments = json!({"entry_price": "51.2", "stop_price": "50.7", "share_count": 10});
        let loss = execute_maximum_planned_loss(arguments.as_object().unwrap()).unwrap();
        assert_eq!(loss, "$5.00");
    }

    #[test]
    fn unknown_fields_are_rejected_before_any_calculation() {
        let mut arguments = scenario_arguments();
        arguments
            .as_object_mut()
            .unwrap()
            .insert("ticker".to_string(), json!("FAKE"));
        assert!(execute_maximum_planned_loss(arguments.as_object().unwrap()).is_err());
    }

    #[test]
    fn non_string_prices_are_rejected_rather_than_float_parsed() {
        let arguments = json!({"entry_price": 51.20, "stop_price": "50.70", "share_count": 200});
        assert!(execute_maximum_planned_loss(arguments.as_object().unwrap()).is_err());
    }

    #[test]
    fn more_than_four_decimal_places_are_rejected() {
        let arguments =
            json!({"entry_price": "51.20000", "stop_price": "50.70", "share_count": 200});
        assert!(execute_maximum_planned_loss(arguments.as_object().unwrap()).is_err());
    }

    #[test]
    fn entry_at_or_below_stop_is_rejected() {
        let at_stop = json!({"entry_price": "50.70", "stop_price": "50.70", "share_count": 200});
        assert!(execute_maximum_planned_loss(at_stop.as_object().unwrap()).is_err());
        let below_stop = json!({"entry_price": "49.00", "stop_price": "50.70", "share_count": 200});
        assert!(execute_maximum_planned_loss(below_stop.as_object().unwrap()).is_err());
    }

    #[test]
    fn zero_and_negative_shares_are_rejected() {
        let zero = json!({"entry_price": "51.20", "stop_price": "50.70", "share_count": 0});
        assert!(execute_maximum_planned_loss(zero.as_object().unwrap()).is_err());
        let negative = json!({"entry_price": "51.20", "stop_price": "50.70", "share_count": -1});
        assert!(execute_maximum_planned_loss(negative.as_object().unwrap()).is_err());
    }

    #[test]
    fn non_positive_prices_are_rejected() {
        let zero_entry = json!({"entry_price": "0.00", "stop_price": "50.70", "share_count": 200});
        assert!(execute_maximum_planned_loss(zero_entry.as_object().unwrap()).is_err());
        let negative_stop = json!({"entry_price": "51.20", "stop_price": "-1", "share_count": 200});
        assert!(execute_maximum_planned_loss(negative_stop.as_object().unwrap()).is_err());
    }

    #[test]
    fn non_allowlisted_tool_names_never_dispatch() {
        let call = CallToolRequestParams::new("place_order")
            .with_arguments(scenario_arguments().as_object().unwrap().clone());
        assert!(
            execute_allowed_tool(&Ok(call)).is_err(),
            "only the allowlisted tool may execute"
        );
    }

    #[test]
    fn a_call_with_no_arguments_is_rejected() {
        assert!(execute_allowed_tool(&Ok(CallToolRequestParams::new(TOOL_NAME))).is_err());
    }

    fn scenario_arguments() -> Value {
        json!({"entry_price": "51.20", "stop_price": "50.70", "share_count": 200})
    }
}
