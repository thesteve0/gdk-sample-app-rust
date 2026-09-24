# Lesson 2: Provider Smoke Test

## Goal

Verify that a configured provider can be constructed and use the public Goose Rust provider API to discover the models it makes available. This is deliberately a smoke test: it does not create messages, make an inference request, or process a stream.

## Concepts introduced

- A declarative provider JSON is the source of truth for provider setup.
- The public `goose_providers` Rust crate constructs a provider from that JSON.
- `fetch_supported_models()` asks that provider for its supported models.
- Tokio runs the asynchronous model-discovery call.

## What this test checks

1. The provider JSON can be parsed and used to construct a Goose provider.
2. The provider's configured authentication, headers, base URL, and model-discovery behavior are used together.
3. At least one model is available.

For the configured OpenAI-compatible provider, `fetch_supported_models()` queries its models endpoint. Its exact behavior comes from the provider configuration: `dynamic_models: false` uses the configured static list; `true` or `null` attempts discovery and can fall back to configured models if the endpoint is unavailable with a 404. A successful discovery call proves neither inference nor streaming; Lesson 3 covers those separately.

## Prerequisites

- Lesson 1 is complete.
- The exact `goose-providers` dependency from the root manifest is available.
- A compatible provider is running at the selected JSON configuration's `base_url` when its configuration uses dynamic model discovery.

## Step 2.1: Load environment and configuration

Start with the imports and default path:

```rust
use goose_providers::declarative::{from_json, EnvKeyResolver};

const DEFAULT_PROVIDER_CONFIG: &str = "custom_aa_llama_qwen3_6-35b.json";
```

Load `.env` before constructing the provider, then read the selected JSON file:

```rust
dotenvy::dotenv().ok();
let provider_json = fs::read_to_string(provider_config_path)?;
let provider = from_json(&provider_json, None, EnvKeyResolver {})?;
```

`from_json` is the public constructor in the Goose Rust provider library. It resolves the declarative configuration—including its environment-variable settings—before returning a provider object. Keep the provider JSON as the authority; do not reconstruct its URL, authentication, headers, or endpoint paths in application code.

## Step 2.2: Discover models through the provider

Model discovery is asynchronous because it may make a network request. Make the entry point asynchronous with Tokio, then call the provider method:

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // ... load environment and construct `provider`
    let available_models = provider.fetch_supported_models().await?;

    if available_models.is_empty() {
        return Err("Provider returned no models".into());
    }
    // ... print the names
    Ok(())
}
```

This keeps provider-specific details inside the Goose library. In particular, the configured provider's normal authentication and headers are applied if model discovery makes an HTTP request. The complete reference source is [`src/main.rs`](src/main.rs).

## Step 2.3: Run it

From the repository root, temporarily copy this lesson's reference source into `src/main.rs` as directed by the instructor, then run:

```bash
cargo run
```

To select another provider JSON:

```bash
cargo run -- path/to/provider.json
```

Example output:

```text
Connected to provider: custom_aa_llama_qwen3_6-35b
Available models:
- qwen3.6-35b-a3b
```

Provider names and model IDs depend on the selected configuration and endpoint.

## Expected structural behavior

The program loads `.env`, reads one provider JSON path, constructs a provider through `from_json`, awaits `fetch_supported_models`, rejects an empty result, and prints the provider and each returned model ID. It must not make an inference request.

## Success criteria

- The selected provider configuration constructs successfully.
- Model discovery completes, either through the configured endpoint or the configuration's documented static fallback behavior.
- At least one model is returned and printed.
- No manually assembled HTTP request, authentication header, or endpoint path appears in application code.

## Next

Lesson 3 introduces typed messages, request-specific model configuration, async streaming, and chunk handling.
