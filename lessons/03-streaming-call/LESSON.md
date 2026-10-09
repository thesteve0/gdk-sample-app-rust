# Lesson 3: First Streaming Call

## Goal

Build on Lesson 2 by making the first inference request. Introduce the minimum native Rust message, model-selection, and asynchronous stream handling needed to receive a response.

## Concepts introduced

- An **inference request** is one application request to a selected model; an **inference response** is the model's answer.
- A **stream** delivers one response as a sequence of partial updates instead of waiting for the entire answer.
- A **message** is one typed contribution to the request or response. This lesson sends one user message and receives partial assistant-role message updates.
- `ModelConfig` selects a model for one request.
- `Provider::fetch_supported_models` discovers which models a provider advertises before a request.
- `Message` represents the `user` and `assistant` message roles in native Rust.
- `Provider::stream` returns an asynchronous message stream.
- `futures::StreamExt` supplies `.next()` for consuming that stream.
- The streamed usage (`ProviderUsage`) reports the model the server actually used to answer, which can differ from the one we requested.

## Where this fits in the application

The application now makes its first inference request. It chooses a configured model, sends the provider a system instruction and one user message, then renders text updates as the provider streams them back:

```text
Rust application
    |
    | inference request: selected model + system instruction + user message
    v
Provider / model
    |
    | streamed partial assistant-role message updates
    v
Rust application prints text and checks completion usage
```

This is **not yet an agentic loop**. The application makes one inference request, receives one response stream, and has no tools to advertise or execute. The application still owns the request, error handling, and output; the model only generates the response.

## Prerequisites

- Lessons 1–2 are complete.
- The configured provider supports streaming and is reachable.
- The selected provider JSON contains at least one model.

## Terminal presentation (supplied support)

Supplied **terminal presentation helpers** color application headings bold cyan and the successful connection check green. Errors are bold red on stderr. Streamed model text and metadata values remain normal; usage is not dimmed. Each output stream independently uses plain text when redirected, when `NO_COLOR` is present (even empty), or when `TERM=dumb`. Existing text, spacing, and stdout/stderr destinations are unchanged. These helpers are provided in the complete standalone reference; learners do not need to implement them.

The supplied `main` wrapper displays a returned error once and exits with status 1; the lesson's fallible work remains in `run_application`. This keeps errors under application presentation control rather than Rust's default runtime formatting.

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
print_status(&format!("Connected to provider: {}", provider.get_name()));
print_heading("Available models:");
for model_name in &available_models {
    println!("- {}", model_name);
}
```

`fetch_supported_models()` reports what the server says it serves. It is a weak signal for which model will actually answer: many OpenAI-compatible endpoints ignore the requested model. That gap is closed below, from the streamed usage.

## Step 3.3: Select the first configured model

The provider JSON configures the endpoint and declares models. `ModelConfig` selects one model for an individual request. This lesson intentionally uses the first configured model; model selection becomes explicit later.

```rust
let model = ModelConfig::new(first_configured_model(&provider_config)?);
```

## Step 3.4: Build a user message

A **user message** is the application's typed representation of what the user asks. A **turn** is one participant's contribution: this first request has one user turn and no assistant-role history turn yet. Lesson 4 stores an assistant-role turn and sends ordered history on a later inference request.

```rust
let messages = [Message::user().with_text(
    "What is the capital of France?",
)];
```

This first request has one user text message. Assistant-role history is introduced in Lesson 4.

## Step 3.5: Start and consume the stream

The application requests inference through the provider. Its response arrives as a stream of **partial updates** (also called deltas): each update can contain only part of the model's eventual assistant-role answer. Printing a text fragment immediately makes the CLI responsive, but this lesson does not need to retain a complete response after it finishes.

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
print_stderr_heading("\n-------------------------\nStreaming response...");

while let Some((message, usage)) = stream.next().await.transpose()? {
    if let Some(message) = message {
        print!("{}", message.as_concat_text());
    }
    if let Some(usage) = usage {
        // The streamed usage reports the model the server actually used to
        // answer, which can differ from the one we requested.
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
```

Each item is a `Result` holding an optional partial `Message` and optional completion usage. `transpose()?` propagates a provider error. A partial message may contain a fragment of assistant-role text; a non-`None` usage value marks normal completion. The reference implementation requires both text and that completion metadata before reporting success.

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

## Expected structural behavior

The program loads `.env`, reads one provider JSON path (the bundled default or a positional override), constructs a provider through `from_json`, awaits `fetch_supported_models`, rejects an empty model list, and selects the first configured model. It sends exactly one inference request — one user message plus a system instruction, no history and no tools — streams text fragments to stdout, and fails unless the stream returns both text and completion usage metadata. It prints, from the streamed usage, the model that actually answered.

## Success criteria

- The provider is constructed directly from the selected provider JSON.
- The request selects its first configured model.
- At least one text fragment reaches stdout.
- The stream supplies completion usage metadata.
- The program reports, from the streamed usage, the model the server actually used to answer.
- No foreign-language binding, manually assembled HTTP request, authentication header, or endpoint path appears in application code.
- No exact generated text is required.

## Next

Lesson 4 keeps a system instruction separate, reconstructs the streamed assistant-role response, and sends it back as ordered conversation history.