# Lesson 4: System Instructions and Conversation History

## Goal

Extend Lesson 3's one-message request into a two-turn conversation. Keep the system instruction separate from conversation messages, reconstruct the first streamed assistant reply, and send the complete ordered history with a follow-up user message.

## Concepts introduced

- The separate `system` argument versus `Message` history.
- `Message::user()` and `Message::assistant()`.
- Capturing streamed text while printing it.
- Re-sending explicit history on every provider call.

## Prerequisites

- Lessons 1–3 are complete.
- The selected provider streams text and accepts ordinary user/assistant history.

## Step 4.1: Name the system instruction

```rust
const SYSTEM_INSTRUCTION: &str =
    "You are a concise programming instructor. Answer in no more than three sentences.";
```

The system instruction controls the request but is not a conversation turn. Pass it as the separate `system` argument to both calls rather than inserting it into `messages`.

## Step 4.2: Make a stream helper return text

A conversation needs the first assistant response as a future history entry. The helper continues printing partial messages but stores their text:

```rust
let mut text_parts = Vec::new();

while let Some((message, usage)) = stream.next().await.transpose()? {
    if let Some(message) = message {
        let text = message.as_concat_text();
        if !text.is_empty() {
            print!("{text}");
            text_parts.push(text);
        }
    }
    if let Some(usage) = usage {
        eprintln!("\nusage: {usage:#?}");
    }
}

Ok(text_parts.concat())
```

The complete helper fails if no text arrives or if the stream ends without completion usage metadata.

## Step 4.3: Start with one user message

```rust
let mut messages = vec![Message::user()
    .with_text("Should I build a Goose SDK application with Rust or Python?")];

let first_response = stream_response(provider.as_ref(), &model, &messages).await?;
```

## Step 4.4: Append assistant and follow-up turns

```rust
messages.extend([
    Message::assistant().with_text(first_response),
    Message::user().with_text("Summarize your recommendation in five words or fewer."),
]);
```

The second request receives all three entries, in order: original user question, assistant response, follow-up user question. The application owns this history; the provider object does not preserve it for later calls.

## Step 4.5: Run it

The full reference solution is [`src/main.rs`](src/main.rs). Temporarily make it the root source as directed by the instructor, then run:

```bash
cargo run
```

Use another provider configuration with:

```bash
cargo run -- path/to/provider.json
```

## Expected structural behavior

- The first response streams to stdout and is reconstructed in memory.
- The first response becomes an assistant message.
- The second request receives all three history messages in order.
- Both calls use the same separate system instruction.
- Usage diagnostics are printed only when completion usage is supplied.

## Success criteria

- Both calls receive text and completion usage metadata.
- The first response is reconstructed from its text fragments.
- The history is user, assistant, user in that order.
- System instructions remain separate from history.
- No exact model wording is required.

## Next

Lesson 5 will add explicit model selection and validation instead of always using the first configured model.
