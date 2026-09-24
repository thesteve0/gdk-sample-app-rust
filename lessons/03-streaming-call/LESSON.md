# Lesson 3: First Streaming Call

## Goal

Build on Lesson 2 by making the first inference request. Introduce the minimum native Rust message, model-selection, and asynchronous stream handling needed to receive a response.

## Concepts introduced

- `ModelConfig` selects a model for one request.
- `Message` represents user and assistant turns in native Rust.
- `Provider::stream` returns an asynchronous message stream.
- `futures::StreamExt` supplies `.next()` for consuming that stream.

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

## Step 3.2: Select the first configured model

The provider JSON configures the endpoint and declares models. `ModelConfig` selects one model for an individual request. This lesson intentionally uses the first configured model; model selection becomes explicit later.

```rust
let provider = from_json(&provider_json, None, EnvKeyResolver {})?;
let model = ModelConfig::new(first_configured_model(&provider_config)?);
```

## Step 3.3: Build a user message

```rust
let messages = [Message::user().with_text(
    "Should I build a Goose SDK application with Rust or with Python? Answer in no more than three sentences.",
)];
```

This first request has one user text message. Assistant history is introduced in Lesson 4.

## Step 3.4: Start and consume the stream

Keep system instructions, messages, and tools separate:

```rust
let mut stream = provider
    .stream(
        &model,
        "you are an expert on the goose SDK",
        &messages,
        &[],
    )
    .await?;

while let Some((message, usage)) = stream.next().await.transpose()? {
    if let Some(message) = message {
        print!("{}", message.as_concat_text());
    }
    if let Some(usage) = usage {
        eprintln!("\nusage: {usage:#?}");
    }
}
```

Each item is a `Result` holding an optional partial `Message` and optional completion usage. `transpose()?` propagates a provider error. A partial message may contain a fragment of assistant text; a non-`None` usage value marks normal completion. The reference implementation requires both text and that completion metadata before reporting success.

## Step 3.5: Run the reference

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
- No foreign-language binding, manually assembled HTTP request, authentication header, or endpoint path appears in application code.
- No exact generated text is required.

## Next

Lesson 4 keeps a system instruction separate, reconstructs the streamed assistant response, and sends it back as ordered conversation history.
