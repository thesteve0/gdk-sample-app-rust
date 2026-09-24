use std::{env, error::Error, fs, path::PathBuf};

use futures::StreamExt;
use goose_providers::{
    base::Provider,
    conversation::message::Message,
    declarative::{from_json, EnvKeyResolver},
    model::ModelConfig,
};
use serde_json::Value;

const DEFAULT_PROVIDER_CONFIG: &str = "custom_aa_llama_qwen3_6-35b.json";
const SYSTEM_INSTRUCTION: &str =
    "You are a concise programming instructor. Answer in no more than three sentences.";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv().ok();

    let provider_config_path = provider_config_path()?;
    let provider_json = fs::read_to_string(&provider_config_path)?;
    let provider_config: Value = serde_json::from_str(&provider_json)?;
    let provider = from_json(&provider_json, None, EnvKeyResolver {})?;
    let model = ModelConfig::new(first_configured_model(&provider_config)?);

    let mut messages = vec![Message::user()
        .with_text("Should I build a Goose SDK application with Rust or Python?")];

    println!("Assistant (turn 1):");
    let first_response = stream_response(provider.as_ref(), &model, &messages).await?;

    messages.extend([
        Message::assistant().with_text(first_response),
        Message::user().with_text("Summarize your recommendation in five words or fewer."),
    ]);

    println!("\nAssistant (turn 2):");
    stream_response(provider.as_ref(), &model, &messages).await?;

    Ok(())
}

async fn stream_response(
    provider: &dyn Provider,
    model: &ModelConfig,
    messages: &[Message],
) -> Result<String, Box<dyn Error>> {
    let mut stream = provider.stream(model, SYSTEM_INSTRUCTION, messages, &[]).await?;

    let mut text_parts = Vec::new();
    let mut saw_completion = false;
    while let Some((message, usage)) = stream.next().await.transpose()? {
        if let Some(message) = message {
            let text = message.as_concat_text();
            if !text.is_empty() {
                print!("{text}");
                text_parts.push(text);
            }
        }
        if let Some(usage) = usage {
            saw_completion = true;
            eprintln!("\nusage: {usage:#?}");
        }
    }

    if text_parts.is_empty() {
        return Err("The stream completed without returning text".into());
    }
    if !saw_completion {
        return Err("The stream completed without usage metadata".into());
    }

    println!();
    Ok(text_parts.concat())
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
