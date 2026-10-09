use std::{
    env,
    error::Error,
    fs,
    io::{self, IsTerminal},
    path::PathBuf,
    sync::Arc,
};

use futures::StreamExt;
use goose_providers::{
    base::Provider,
    conversation::{message::Message, token_usage::ProviderUsage, Conversation},
    declarative::{from_json, EnvKeyResolver},
    model::ModelConfig,
};
use rmcp::model::{JsonObject, Tool};
use serde_json::{json, Value};

const DEFAULT_PROVIDER_CONFIG: &str = "custom_aa_llama_qwen3_6-35b.json";
const TOOL_NAME: &str = "maximum_planned_loss";

/// A delimiter printed before and after every block of protocol payload: the
/// outbound prompts and tool definition, and the model's output. Everything
/// between two delimiter lines is a value that traveled to or from the
/// provider; everything else on the terminal is this application's
/// explanatory text.
const PAYLOAD_DELIMITER: &str = "++++++++";

const SYSTEM_INSTRUCTION: &str = "You are a day-trading teaching assistant. For the supplied hypothetical trade, use the maximum_planned_loss tool. Never claim to place, modify, or cancel a trade.";
const USER_PROMPT: &str = "A hypothetical long trade enters at $51.20, uses a stop at $50.70, and has 200 shares. What is the maximum planned loss?";

#[tokio::main]
async fn main() {
    // Style fatal errors on stderr and retain the failing exit status.
    if let Err(error) = run_application().await {
        print_error(&format!("Error: {:?}", error));
        std::process::exit(1);
    }
}

async fn run_application() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv().ok();

    let provider_config_path = provider_config_path()?;
    let provider_json = fs::read_to_string(&provider_config_path)?;
    let provider_config: Value = serde_json::from_str(&provider_json)?;
    let provider = from_json(&provider_json, None, EnvKeyResolver {})?;
    let model = ModelConfig::new(first_configured_model(&provider_config)?);

    // A single deterministic tool. Advertising it to inference is this lesson's
    // whole point: it lets the model *request* a capability the application
    // still authorizes and executes in Lesson 6.
    let tools = [tool_definition()];
    let messages = vec![Message::user().with_text(USER_PROMPT)];

    // Phase 1: what the application sends. The tool is advertised, never run.
    print_heading("── Sending ─────────────────────────────────────────────");
    println!("1 user message; advertising 1 tool: {}", TOOL_NAME);
    println!("  (the model may now request it)");

    // The full outbound payload is these prompts plus the advertised tool
    // definition: the entire context the model sees in the first call.
    // Printing all of it makes it visible — the model answers only what this
    // call contains, and nothing here executes anything.
    print_heading(
        "\nOutbound payload sent with this call (application → provider), quoted in full:",
    );
    print_delimiter();
    println!("  system instruction:");
    print_indented(SYSTEM_INSTRUCTION, "    > ");
    println!("  user message:");
    print_indented_styled(USER_PROMPT, "    > ", TextStyle::Blue);
    print_tool_definition(&tools[0])?;
    print_delimiter();

    // Phase 2: stream the response, printing text blocks as they arrive.
    print_heading("\n── Streaming response deltas (provider → application) ──");
    let (conversation, usage) =
        stream_and_collect(provider.as_ref(), &model, &messages, &tools).await?;

    // Phase 3: compact telemetry. A finish reason of `tool_calls` is evidence
    // of the request boundary, but the program only displays it, never acts on it.
    print_usage_summary(&usage);

    // Phase 4: inspect every block of every reconstructed message. Structured
    // requests, thinking, and text are labeled; the request ID is the
    // correlation ID Lesson 6 must reuse.
    print_heading(
        "\n── All the response messages (provider → application, after Conversation::push) ──",
    );
    let saw_tool_request = inspect_reconstructed(&conversation)?;

    // Phase 5: show exactly where this run stops inside the normal
    // request/response cycle. The stop is artificial, for teaching: the
    // protocol itself would continue at the step named below, and Lesson 6
    // implements that step.
    print_heading("\n── We stopped here ────────────────────────────────────");
    println!("The full request/response cycle for a tool-using turn:");
    println!("  1. send system instruction + user message (+ the advertised tool)");
    println!("  2. model responds with thinking + a tool request");
    println!("  3. application validates the request and executes the tool");
    println!("  4. send the full history + tool response; model writes the final answer");
    println!("  5. application returns the final answer to the user");

    if saw_tool_request {
        print_status("\nThis run stopped between steps 2 and 3 — an artificial stop for teaching.");
        println!("The tool request above was received, but nothing executed: the application");
        println!("alone authorizes, validates, and runs the calculation. Lesson 6 resumes");
        println!("the cycle at step 3.");
    } else {
        print_warning("\nThis run stopped after step 2: no tool request arrived, so there was");
        println!("nothing to execute. Advertising a tool permits a request; it does not");
        println!("cause one. Lesson 6 validates and dispatches when one does arrive.");
    }

    Ok(())
}

/// Stream one inference call, printing text as it arrives and reconstructing the
/// complete messages with GDK merge semantics so the request boundary stays
/// visible instead of collapsing to a single partial fragment. Returns the
/// reconstructed Conversation and the call's usage telemetry.
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
            // Text alone is enough for Lessons 3 and 4, but a tool-capable model
            // response can also contain structured requests, arguments, and correlation
            // IDs. Preserve the complete assistant-role message for the next agentic step.
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
        print_warning(
            "(no text blocks streamed; the response arrived as non-text blocks — see the reconstructed view below)"
        );
    }

    Ok((conversation, usage))
}

/// Print one compact usage line. A finish reason of `tool_calls` means the
/// model ended its turn expecting a tool result: display it, never act on it.
fn print_usage_summary(usage: &Option<ProviderUsage>) {
    print_heading(
        "\n── Relevant response metadata (Usage block of the response)──────────────────",
    );
    let Some(usage) = usage else {
        print_warning("(no usage telemetry was provided by the provider)");
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

/// Inspect every content block across every reconstructed message. A single
/// streamed response can carry several blocks, and one response can carry
/// more than one tool request, so label and inspect all of them.
fn inspect_reconstructed(conversation: &Conversation) -> Result<bool, Box<dyn Error>> {
    let mut saw_tool_request = false;
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
                saw_tool_request = true;
                print_styled(
                    &format!(
                        "  {} ToolRequest — the model asks the application to run a tool",
                        label
                    ),
                    TextStyle::Magenta,
                );
                print_delimiter();
                // The request ID is the correlation ID: Lesson 6 must attach it to
                // the tool response so the provider can match result to request.
                print_styled(
                    &format!("    request id (correlation ID): {}", request.id),
                    TextStyle::Magenta,
                );
                // `tool_call` is a Result: tool-call parsing can fail inside a
                // request, so never blindly unwrap it.
                match &request.tool_call {
                    Ok(call) => {
                        print_styled(&format!("    tool name: {}", call.name), TextStyle::Magenta);
                        print_styled(
                            "    arguments (untrusted model output; JSON object, order not guaranteed):",
                            TextStyle::Magenta,
                        );
                        match &call.arguments {
                            Some(arguments) => print_indented_styled(
                                &serde_json::to_string_pretty(arguments)?,
                                "      ",
                                TextStyle::Magenta,
                            ),
                            None => print_styled("      (none)", TextStyle::Magenta),
                        }
                    }
                    Err(error) => {
                        print_styled(
                            &format!(
                                "    unparseable call (reported without panicking): {}",
                                error
                            ),
                            TextStyle::Magenta,
                        );
                    }
                }
                print_delimiter();
            } else if let Some(thinking) = block.as_thinking() {
                print_warning(&format!("  {} Thinking block — Not ground truth", label));
                print_delimiter();
                print_indented(&thinking.thinking, "             ");
                print_delimiter();
            } else if let Some(text) = block.as_text() {
                if !text.trim().is_empty() {
                    print_heading(&format!("  {} Text", label));
                    print_delimiter();
                    print_indented(text, "    ");
                    print_delimiter();
                }
            } else {
                println!("  {} other non-text block: {}", label, block);
            }
        }
    }

    Ok(saw_tool_request)
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
    print_styled(text, TextStyle::Green);
}

fn print_styled(text: &str, style: TextStyle) {
    println!("{}", styled(text, style, stdout_color_enabled()));
}

fn print_indented_styled(text: &str, indent: &str, style: TextStyle) {
    for line in text.lines() {
        print_styled(&format!("{}{}", indent, line), style);
    }
}

fn print_warning(text: &str) {
    print_styled(text, TextStyle::Yellow);
}

fn print_error(text: &str) {
    let enabled = terminal_color_enabled(io::stderr().is_terminal());
    eprintln!("{}", styled(text, TextStyle::Error, enabled));
}

/// Print one payload delimiter line. Every payload block opens and closes
/// with it, so a reader can see exactly where commentary ends and provider
/// payload begins. The line printed immediately before each block names the
/// sender and the receiver of that message: the round trip has multiple
/// actors (this application and the provider), so every message must show
/// both of its ends.
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
    fn terminal_colors_respect_redirection_no_color_and_dumb_terminals() {
        for terminal in [false, true] {
            for no_color in [false, true] {
                for dumb in [false, true] {
                    assert_eq!(
                        color_enabled(terminal, no_color, dumb),
                        terminal && !no_color && !dumb
                    );
                }
            }
        }
    }

    #[test]
    fn styles_preserve_plain_text_and_reset_without_dimming() {
        let text = "label → payload\n$100.00";
        for (style, code) in [
            (TextStyle::Heading, "1;36"),
            (TextStyle::Blue, "34"),
            (TextStyle::Magenta, "35"),
            (TextStyle::Green, "32"),
            (TextStyle::Yellow, "33"),
            (TextStyle::Error, "1;31"),
        ] {
            assert_eq!(styled(text, style, false), text);
            let colored = styled(text, style, true);
            assert_eq!(colored, format!("\x1b[{}m{}\x1b[0m", code, text));
            assert!(!colored.contains("\x1b[2m"));
        }
    }

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
}
