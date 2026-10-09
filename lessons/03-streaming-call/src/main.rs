use std::{
    env,
    error::Error,
    fs,
    io::{self, IsTerminal},
    path::PathBuf,
};

use futures::StreamExt;
use goose_providers::{
    conversation::message::Message,
    declarative::{from_json, EnvKeyResolver},
    model::ModelConfig,
};
use serde_json::Value;

const DEFAULT_PROVIDER_CONFIG: &str = "custom_aa_llama_qwen3_6-35b.json";

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

    // Before sending a message, query the provider to confirm it is reachable
    // and learn which models it advertises. Fail closed if it answers with none,
    // so a broken endpoint surfaces before we attempt to stream.
    let available_models = provider.fetch_supported_models().await?;
    if available_models.is_empty() {
        return Err("Provider is reachable but returned no models".into());
    }
    print_status(&format!("Connected to provider: {}", provider.get_name()));
    print_heading("Available models:");
    for model_name in &available_models {
        println!("- {}", model_name);
    }

    // Select the first configured model for this request. Whether the server
    // actually serves it is confirmed below, from the streamed usage.
    let configured_model = first_configured_model(&provider_config)?;
    let model = ModelConfig::new(configured_model);

    // The application supplies one user turn as input to this inference request.
    // The model's assistant-role message will arrive incrementally through the stream.
    let messages = [Message::user().with_text("What is the capital of France?")];

    // Start inference. Streaming returns model-generated assistant-role message fragments as they
    // are produced, so the application can print each one without waiting for
    // the complete response.
    let mut stream = provider
        .stream(&model, "you are an expert on geography", &messages, &[])
        .await?;
    print_stderr_heading("\n-------------------------\nStreaming response...");

    let mut saw_text = false;
    let mut saw_completion = false;
    while let Some((message, usage)) = stream.next().await.transpose()? {
        if let Some(message) = message {
            let text = message.as_concat_text();
            if !text.is_empty() {
                saw_text = true;
                print!("{}", text);
            }
        }
        if let Some(usage) = usage {
            saw_completion = true;
            // The streamed usage reports the model the server actually used to
            // answer — the ground truth. A provider can ignore the model you
            // requested, so prefer this over the requested model.
            println!(
                "{} {}",
                styled(
                    "\n-------------------------\nThe model that answered:",
                    TextStyle::Heading,
                    stdout_color_enabled()
                ),
                usage.model
            );
            eprintln!(
                "{} {:#?}",
                styled(
                    "-------------------------\nFull usage metadata response:",
                    TextStyle::Heading,
                    terminal_color_enabled(io::stderr().is_terminal())
                ),
                usage
            );
        }
    }

    if !saw_text {
        return Err("The stream completed without returning text".into());
    }
    if !saw_completion {
        return Err("The stream completed without usage metadata".into());
    }
    println!();

    Ok(())
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
    Green,
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
        TextStyle::Green => "32",
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

fn print_stderr_heading(text: &str) {
    let enabled = terminal_color_enabled(io::stderr().is_terminal());
    eprintln!("{}", styled(text, TextStyle::Heading, enabled));
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
        let styles = [
            (TextStyle::Heading, "1;36"),
            (TextStyle::Green, "32"),
            (TextStyle::Error, "1;31"),
        ];
        for (style, code) in styles {
            assert_eq!(
                styled(text, style, true),
                format!("\x1b[{}m{}\x1b[0m", code, text)
            );
            assert_eq!(styled(text, style, false), text);
        }
    }
}
