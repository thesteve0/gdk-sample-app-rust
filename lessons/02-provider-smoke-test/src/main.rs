use std::{
    env,
    error::Error,
    fs,
    io::{self, IsTerminal},
    path::PathBuf,
};

use goose_providers::declarative::{from_json, EnvKeyResolver};
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
    // Parse the JSON so we can compare what it *asks* for against what the
    // server actually advertises.
    let provider_config: Value = serde_json::from_str(&provider_json)?;

    // The provider is the application's configured interface to the model
    // service. Constructing it does not yet send a prompt to a model.
    let provider = from_json(&provider_json, None, EnvKeyResolver {})?;
    // Model discovery asks which model IDs this provider makes available. It is
    // not an inference request and produces no model-generated response.
    let available_models = provider.fetch_supported_models().await?;
    if available_models.is_empty() {
        return Err("Provider is reachable but returned no models".into());
    }

    print_status(&format!("Connected to provider: {}", provider.get_name()));
    print_heading("Available models:");
    for model_name in &available_models {
        println!("- {}", model_name);
    }

    // The server told us what it serves. Check whether the model the JSON
    // configured is among them; if not, surface the mismatch.
    if let Some(configured_model) = first_configured_model(&provider_config)? {
        if available_models
            .iter()
            .any(|model| model == &configured_model)
        {
            print_status(&format!(
                "The configured model '{}' is available.",
                configured_model
            ));
        } else {
            print_warning(&format!(
                "Your model is not available: the JSON asks for '{}', \
                 but the provider advertised {:#?}.\n\
                 We will address this in the next exercise",
                configured_model, available_models
            ));
        }
    }

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

fn first_configured_model(provider_config: &Value) -> Result<Option<String>, Box<dyn Error>> {
    let configured = provider_config
        .get("models")
        .and_then(Value::as_array)
        .and_then(|models| models.first())
        .and_then(|model| model.get("name"))
        .and_then(Value::as_str);

    match configured {
        Some(name) => Ok(Some(name.to_owned())),
        None => Ok(None),
    }
}

// Presentation support only; redirected transcripts remain plain text.
#[derive(Clone, Copy)]
enum TextStyle {
    Heading,
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
    println!("{}", styled(text, TextStyle::Green, stdout_color_enabled()));
}

fn print_warning(text: &str) {
    // This existing model-mismatch diagnostic stays on stdout.
    println!(
        "{}",
        styled(text, TextStyle::Yellow, stdout_color_enabled())
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
        let styles = [
            (TextStyle::Heading, "1;36"),
            (TextStyle::Green, "32"),
            (TextStyle::Yellow, "33"),
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
