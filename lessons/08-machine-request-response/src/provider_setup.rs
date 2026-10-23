// Familiar configuration helpers from earlier lessons; selection behavior is unchanged.
use std::{env, path::PathBuf};

use serde_json::Value;

pub(super) const DEFAULT_PROVIDER_CONFIG: &str = "custom_aa_llama_qwen3_6-35b.json";

pub(super) fn provider_config_path() -> anyhow::Result<PathBuf> {
    let mut arguments = env::args_os();
    let _program = arguments.next();
    let path = match (arguments.next(), arguments.next()) {
        (Some(path), None) => PathBuf::from(path),
        (None, None) => PathBuf::from(DEFAULT_PROVIDER_CONFIG),
        _ => return Err(anyhow::anyhow!("Usage: cargo run -- [provider.json]")),
    };
    Ok(path)
}

pub(super) fn first_configured_model(config: &Value) -> anyhow::Result<String> {
    let models = match config.get("models").and_then(Value::as_array) {
        Some(models) => models,
        None => {
            return Err(anyhow::anyhow!(
                "provider JSON declares no configured models"
            ))
        }
    };
    let first = match models.first() {
        Some(first) => first,
        None => {
            return Err(anyhow::anyhow!(
                "provider JSON declares no configured models"
            ))
        }
    };
    let name = match first.get("name").and_then(Value::as_str) {
        Some(name) => name,
        None => return Err(anyhow::anyhow!("first configured model has no name")),
    };
    Ok(name.to_owned())
}
