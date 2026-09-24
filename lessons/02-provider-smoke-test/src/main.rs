use std::{env, error::Error, fs, path::PathBuf};

use goose_providers::declarative::{from_json, EnvKeyResolver};

const DEFAULT_PROVIDER_CONFIG: &str = "custom_aa_llama_qwen3_6-35b.json";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv().ok();

    let provider_config_path = provider_config_path()?;
    let provider_json = fs::read_to_string(&provider_config_path)?;

    // Constructing the provider confirms that GDK accepts the declarative JSON.
    let provider = from_json(&provider_json, None, EnvKeyResolver {})?;
    let available_models = provider.fetch_supported_models().await?;
    if available_models.is_empty() {
        return Err("Provider is reachable but returned no models".into());
    }

    println!("Connected to provider: {}", provider.get_name());
    println!("Available models:");
    for model_name in available_models {
        println!("- {model_name}");
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
