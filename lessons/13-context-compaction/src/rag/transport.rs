// Lesson 13
//! Local tutor endpoint, declared-output-envelope and redirect restrictions.
use serde_json::{json, Value};
use std::env;

pub fn validate_local_tutor(config: &Value, max_model_name_bytes: usize) -> anyhow::Result<()> {
    // alpha.11 applies declared request_params after ModelConfig, including max_tokens.
    // Keep this lesson's explicit output cap and envelope allowance authoritative.
    if let Some(models) = config.get("models").and_then(Value::as_array) {
        for model in models {
            if model
                .get("name")
                .and_then(Value::as_str)
                .is_some_and(|name| name.len() > max_model_name_bytes)
            {
                return Err(anyhow::anyhow!(
                    "tutor model name exceeds the bounded envelope allowance"
                ));
            }
            if model
                .get("request_params")
                .is_some_and(|params| !params.is_null() && params != &json!({}))
            {
                return Err(anyhow::anyhow!("Lesson 13 requires absent/empty model request_params: declared overrides can defeat output limits and byte allowances"));
            }
        }
    }
    if config.get("engine").and_then(Value::as_str) != Some("openai") {
        return Err(anyhow::anyhow!(
            "Lesson 12 requires a local OpenAI-compatible tutor"
        ));
    }
    let base = match config.get("base_url").and_then(Value::as_str) {
        Some(base) => base,
        None => return Err(anyhow::anyhow!("local tutor base_url is required")),
    };
    let parsed = match reqwest::Url::parse(base) {
        Ok(parsed) => parsed,
        Err(_error) => return Err(anyhow::anyhow!("invalid local tutor endpoint")),
    };
    let authority = match base.split_once("://") {
        Some((_scheme, tail)) => tail.split('/').next().unwrap_or_default(),
        None => return Err(anyhow::anyhow!("invalid local tutor endpoint")),
    };
    let literal_loopback = authority == "127.0.0.1"
        || authority.starts_with("127.0.0.1:")
        || authority == "[::1]"
        || authority.starts_with("[::1]:");
    if !literal_loopback
        || !matches!(parsed.scheme(), "http" | "https")
        || !parsed.username().is_empty()
        || parsed.password().is_some()
        || parsed.query().is_some()
        || parsed.fragment().is_some()
    {
        return Err(anyhow::anyhow!(
            "Lesson 12 tutor must use a literal loopback HTTP(S) endpoint without URL credentials"
        ));
    }
    // Same-origin redirects are supplied by the pinned provider; reject ambient proxies too.
    for name in [
        "HTTP_PROXY",
        "HTTPS_PROXY",
        "ALL_PROXY",
        "http_proxy",
        "https_proxy",
        "all_proxy",
    ] {
        if let Some(value) = env::var_os(name) {
            if !value.is_empty() {
                return Err(anyhow::anyhow!(
                    "disable proxy environment variables before using private Lesson 12 evidence"
                ));
            }
        }
    }
    Ok(())
}

pub fn local_tutor_transport(
    client: goose_providers::api_client::ApiClient,
) -> anyhow::Result<goose_providers::api_client::ApiClient> {
    client.with_same_origin_redirects()
}
