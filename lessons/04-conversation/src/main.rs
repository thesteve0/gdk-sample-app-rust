use std::{
    env,
    error::Error,
    fs,
    io::{self, IsTerminal},
    path::PathBuf,
};

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
    "You are a history and geography expert. Answer in no more than three sentences.";

#[tokio::main]
async fn main() {
    // Own error presentation rather than Rust's unstyled runtime diagnostic.
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

    // Application-owned conversation history begins with one user turn. It is
    // explicitly sent with each inference request.
    let mut messages = vec![Message::user().with_text("What is the capital of France?")];

    print_heading("Model response (assistant role, turn 1):");
    // The model generates the first assistant-role turn from the system instruction
    // and the current application-owned history.
    let first_response = stream_response(provider.as_ref(), &model, &messages).await?;

    // Preserve the generated assistant-role turn, then append the follow-up user turn.
    // The application, not the provider, maintains this ordered history.
    messages.extend([
        Message::assistant().with_text(first_response),
        Message::user().with_text(
            "Tell me the historical origin of this city. Write no more than 2 sentences",
        ),
    ]);

    print_heading("\nModel response (assistant role, turn 2):");
    stream_response(provider.as_ref(), &model, &messages).await?;

    Ok(())
}

async fn stream_response(
    provider: &dyn Provider,
    model: &ModelConfig,
    messages: &[Message],
) -> Result<String, Box<dyn Error>> {
    // Each inference call receives the same system instruction plus the complete
    // history the application wants the model to use.
    let mut stream = provider
        .stream(model, SYSTEM_INSTRUCTION, messages, &[])
        .await?;

    let mut text_parts = Vec::new();
    let mut saw_completion = false;
    while let Some((message, usage)) = stream.next().await.transpose()? {
        if let Some(message) = message {
            let text = message.as_concat_text();
            if !text.is_empty() {
                print!("{}", text);
                text_parts.push(text);
            }
        }
        if let Some(usage) = usage {
            saw_completion = true;
            eprintln!("\nusage: {:#?}", usage);
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

// Presentation support only; redirected transcripts remain plain text.
#[derive(Clone, Copy)]
enum TextStyle {
    Heading,
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

fn print_error(text: &str) {
    let enabled = terminal_color_enabled(io::stderr().is_terminal());
    eprintln!("{}", styled(text, TextStyle::Error, enabled));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colors_respect_terminal_no_color_and_dumb_gates() {
        for is_terminal in [false, true] {
            for no_color in [false, true] {
                for dumb_terminal in [false, true] {
                    let expected = is_terminal && !no_color && !dumb_terminal;
                    assert_eq!(
                        color_enabled(is_terminal, no_color, dumb_terminal),
                        expected
                    );
                }
            }
        }
    }

    #[test]
    fn used_styles_reset_and_preserve_plain_layout() {
        let text = "\nHeading: value\n";
        let styles = [(TextStyle::Heading, "1;36"), (TextStyle::Error, "1;31")];
        for (style, code) in styles {
            assert_eq!(
                styled(text, style, true),
                format!("\x1b[{}m{}\x1b[0m", code, text)
            );
            assert_eq!(styled(text, style, false), text);
        }
    }
}
