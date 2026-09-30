use std::{env, error::Error, fs, path::PathBuf, sync::Arc};

use futures::StreamExt;
use goose_providers::{
    base::Provider,
    conversation::{message::Message, Conversation},
    declarative::{from_json, EnvKeyResolver},
    model::ModelConfig,
};
use rmcp::model::{JsonObject, Tool};
use serde_json::{json, Value};

const DEFAULT_PROVIDER_CONFIG: &str = "custom_aa_llama_qwen3_6-35b.json";
const TOOL_NAME: &str = "maximum_planned_loss";

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

    // A single deterministic tool. Advertising it to inference is this lesson's
    // whole point: it lets the model *request* a capability the application
    // still authorizes and executes in Lesson 6.
    let tools = [tool_definition()];
    let messages = vec![Message::user().with_text(USER_PROMPT)];

    println!("Model response (assistant role, tool advertised):");
    let conversation = stream_and_collect(provider.as_ref(), &model, &messages, &tools).await?;

    // Inspect every content block across every reconstructed message. A single
    // streamed response can carry several blocks, and one response can carry
    // more than one tool request, so we look at all of them.
    println!("\nReconstructed content blocks:");
    let mut saw_tool_request = false;
    for message in conversation.messages() {
        for block in &message.content {
            if let Some(request) = block.as_tool_request() {
                // `tool_call` is a Result: tool-call parsing can fail inside a
                // request, so never blindly unwrap it.
                match &request.tool_call {
                    Ok(call) => {
                        saw_tool_request = true;
                        println!("  tool request id={}", request.id);
                        println!("  tool name={}", call.name);
                        println!("  tool arguments={:?}", call.arguments);
                    }
                    Err(error) => {
                        saw_tool_request = true;
                        println!(
                            "  tool request id={} has an unparseable call: {}",
                            request.id, error
                        );
                    }
                }
            } else if let Some(text) = block.as_text() {
                if !text.trim().is_empty() {
                    println!("  text: {text}");
                }
            } else {
                println!("  {block}");
            }
        }
    }

    println!();
    if !saw_tool_request {
        println!("No structured maximum_planned_loss request was received this run.");
        println!(
            "Advertising or receiving a request does not execute the tool. Lesson 6 \
             validates the untrusted arguments and dispatches the calculation."
        );
    }

    // The application owns this history; the provider does not preserve it.
    // It remains in memory for this run only; persistence arrives in a later lesson.
    println!(
        "\nRetained {} reconstructed assistant-role message(s) in memory for the next raw-protocol step.",
        conversation.messages().len()
    );

    Ok(())
}

/// Stream one inference call, printing text as it arrives and reconstructing the
/// complete messages with GDK merge semantics so the request boundary stays
/// visible instead of collapsing to a single partial fragment.
async fn stream_and_collect(
    provider: &dyn Provider,
    model: &ModelConfig,
    messages: &[Message],
    tools: &[Tool],
) -> Result<Conversation, Box<dyn Error>> {
    let mut stream = provider
        .stream(model, SYSTEM_INSTRUCTION, messages, tools)
        .await?;

    // A streamed response is not one stream item and one response. Conversation::push
    // coalesces the partial deltas that share a message back into complete messages.
    let mut conversation = Conversation::empty();
    while let Some((message, usage)) = stream.next().await.transpose()? {
        if let Some(message) = message {
            let text = message.as_concat_text();
            if !text.is_empty() {
                print!("{text}");
            }
            // Text alone is enough for Lessons 3 and 4, but a tool-capable model
            // response can also contain structured requests, arguments, and correlation
            // IDs. Preserve the complete assistant-role message for the next agentic step.
            conversation.push(message);
        }
        if let Some(usage) = usage {
            eprintln!("\nusage: {usage:#?}");
        }
    }

    Ok(conversation)
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
    provider_config
        .get("models")
        .and_then(Value::as_array)
        .and_then(|models| models.first())
        .and_then(|model| model.get("name"))
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| "Provider JSON has no configured model".into())
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

        let required: Vec<&str> = schema
            .get("required")
            .and_then(Value::as_array)
            .map(|array| array.iter().filter_map(Value::as_str).collect::<Vec<_>>())
            .unwrap_or_default();
        assert!(required.contains(&"entry_price"));
        assert!(required.contains(&"stop_price"));
        assert!(required.contains(&"share_count"));
    }
}
