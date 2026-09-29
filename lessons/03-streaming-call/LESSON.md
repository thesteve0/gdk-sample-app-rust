# Lesson 3: First Streaming Call

## Goal

Build on Lesson 2 by making the first inference request. Introduce the minimum native Rust message, model-selection, and asynchronous stream handling needed to receive a response.

## Concepts introduced

- `ModelConfig` selects a model for one request.
- `Provider::fetch_supported_models` discovers which models a provider advertises before a request.
- `Message` represents user and assistant turns in native Rust.
- `Provider::stream` returns an asynchronous message stream.
- `futures::StreamExt` supplies `.next()` for consuming that stream.
- The streamed usage (`ProviderUsage`) reports the model the server actually used to answer, which can differ from the one we requested.

## Prerequisites

- Lessons 1–2 are complete.
- The configured provider supports streaming and is reachable.
- The selected provider JSON contains at least one model.

## Step 3.1: Import the native provider types

```rust
use futures::StreamExt;
use goose_providers::{
    base::Provider,
    conversation::message::Message,
    declarative::{from_json, EnvKeyResolver},
    model::ModelConfig,
};
```

These are the native Rust types for this standalone CLI. The program constructs the provider directly from its JSON configuration; it does not use the SDK's UniFFI bindings, which exist for foreign-language callers. `#[tokio::main]` makes the binary's `main` function asynchronous.

## Step 3.2: Reach out to the provider before sending a message

Before making an inference request, confirm the provider is reachable and learn which models it advertises. `fetch_supported_models()` may perform a network request, so it is awaited. Fail closed if the provider answers with no models; a broken endpoint should surface here rather than during the stream.

```rust
let provider = from_json(&provider_json, None, EnvKeyResolver {})?;
let available_models = provider.fetch_supported_models().await?;
if available_models.is_empty() {
    return Err("Provider is reachable but returned no models".into());
}
println!("Connected to provider: {}", provider.get_name());
println!("Available models:");
for model_name in &available_models {
    println!("- {model_name}");
}
```

`fetch_supported_models()` reports what the server says it serves. It is a weak signal for which model will actually answer: many OpenAI-compatible endpoints ignore the requested model. That gap is closed below, from the streamed usage.

## Step 3.3: Select the first configured model

The provider JSON configures the endpoint and declares models. `ModelConfig` selects one model for an individual request. This lesson intentionally uses the first configured model; model selection becomes explicit later.

```rust
let model = ModelConfig::new(first_configured_model(&provider_config)?);
```

## Step 3.4: Build a user message

```rust
let messages = [Message::user().with_text(
    "What is the capital of France?",
)];
```

This first request has one user text message. Assistant history is introduced in Lesson 4.

## Step 3.5: Start and consume the stream

Keep system instructions, messages, and tools separate:

```rust
let mut stream = provider
    .stream(
        &model,
        "you are an expert on geography",
        &messages,
        &[],
    )
    .await?;

while let Some((message, usage)) = stream.next().await.transpose()? {
    if let Some(message) = message {
        print!("{}", message.as_concat_text());
    }
    if let Some(usage) = usage {
        // The streamed usage reports the model the server actually used to
        // answer, which can differ from the one we requested.
        println!("\nThe model that answered: {}", usage.model);
        eprintln!("usage: {usage:#?}");
    }
}
```

Each item is a `Result` holding an optional partial `Message` and optional completion usage. `transpose()?` propagates a provider error. A partial message may contain a fragment of assistant text; a non-`None` usage value marks normal completion. The reference implementation requires both text and that completion metadata before reporting success.

The completion usage is more than token counts: its `model` field names the model the server actually used to answer. Prefer this ground truth over the model you requested — a provider can ignore the requested model and serve another. Reporting `usage.model` therefore tells you which model produced the response, which is the answer to "did it really use the model I asked for?".

## Step 3.6: Run the reference

The full implementation is [`src/main.rs`](src/main.rs). Temporarily make it the root source as directed by the instructor, then run:

```bash
cargo run
```

Pass a different provider JSON with:

```bash
cargo run -- path/to/provider.json
```

## Success criteria

- The provider is constructed directly from the selected provider JSON.
- The request selects its first configured model.
- At least one text fragment reaches stdout.
- The stream supplies completion usage metadata.
- The program reports, from the streamed usage, the model the server actually used to answer.
- No foreign-language binding, manually assembled HTTP request, authentication header, or endpoint path appears in application code.
- No exact generated text is required.

## Next

Lesson 4 keeps a system instruction separate, reconstructs the streamed assistant response, and sends it back as ordered conversation history.