# Lesson 4: System Instructions and Conversation History

## Goal

Extend Lesson 3's one-message request into a two-turn conversation. Keep the system instruction separate from conversation messages, reconstruct the first streamed assistant-role reply, and send the complete ordered history with a follow-up user message.

## Concepts introduced

- A **turn** is one `user`- or `assistant`-role contribution; **conversation history** is their ordered sequence.
- The application, not the provider object, owns and re-sends that history for each later inference request.
- The separate `system` argument versus `Message` history.
- `Message::user()` and `Message::assistant()`.
- Capturing streamed text while printing it.
- Re-sending explicit history on every provider call.

## How a one-request exchange becomes a discussion

Lesson 3 made one inference request: one user message went in and streamed model-generated assistant-role text came back. A **conversation** is a discussion across multiple turns. The application creates it by retaining prior messages and supplying them in order on each later request; the provider object does not remember earlier calls for this program.

This lesson performs two requests:

```text
Application-owned history             Inference request
-------------------------             -----------------
user: question                     ->  first request
assistant role: first response     ->  retained after streaming
user: follow-up                    ->  second request receives all three turns
```

For now, the program reconstructs only the first response's text because that is all it needs to create `Message::assistant().with_text(...)`. Lesson 5 introduces structured model output, where preserving plain text is no longer enough.

## Prerequisites

- Lessons 1–3 are complete.
- The selected provider streams text and accepts ordinary user/assistant-role history.

## Step 4.1: Name the system instruction

```rust
const SYSTEM_INSTRUCTION: &str =
    "You are a history and geography expert. Answer in no more than three sentences.";
```

A **system instruction** is application-supplied guidance for the model's behavior. It controls each request but is not a conversation turn: a user does not say it and an assistant-role message does not answer it. Pass it as the separate `system` argument to both calls rather than inserting it into `messages`.

### The system instruction is one of four things a call receives

The helper sends everything the model needs in a single `stream` call, in this order:

```rust
provider.stream(model, SYSTEM_INSTRUCTION, messages, &[])
//               │          │             │          └── tools (none this lesson)
//               │          │             └── the ordered conversation history
//               │          └── the system instruction (role: system)
//               └── which model to use
```

- `model` selects which model to run.
- `SYSTEM_INSTRUCTION` is passed as its **own dedicated `system` argument** — not as an entry in `messages`. The SDK places it as a distinct `role: "system"` message in the request, ahead of the conversation history.
- `messages` carries the actual user↔assistant-role turns.

Why a separate argument instead of an entry in `messages`? A system message is not a dialogue turn: the model never replies to it, and it does not count as a user or assistant-role exchange. The SDK surfaces it as its own parameter so that distinction stays explicit in code.

> **Note on cost and context.** The system instruction's tokens do count toward the size of every request — the model must read them to follow the instruction — so it does use context-window budget for that request. But it is *treated differently* from the history: it is a fixed, constant overhead (the same instruction on every call), whereas the `messages` list grows by two entries each turn. It sits outside the turn history and shapes every turn rather than being one turn itself.

> Some servers reduce the marginal cost of that fixed instruction by caching an identical system prefix and skipping re-prefilling on each new turn. That is a backend-specific optimization layered on top of a correct, simple contract: the application cannot rely on any backend's caching behavior, so it always re-sends the full instruction with every call.

## Step 4.2: Make a stream helper return text

The application needs the first assistant-role response as a future history entry. The helper continues printing partial messages but stores their text. The model does not add this history itself; after the stream ends, the application creates the next assistant-role turn from the collected text:

```rust
let mut text_parts = Vec::new();

while let Some((message, usage)) = stream.next().await.transpose()? {
    if let Some(message) = message {
        let text = message.as_concat_text();
        if !text.is_empty() {
            print!("{}", text);
            text_parts.push(text);
        }
    }
    if let Some(usage) = usage {
        eprintln!("\nusage: {:#?}", usage);
    }
}

Ok(text_parts.concat())
```

The complete helper fails if no text arrives or if the stream ends without completion usage metadata.

### Understanding `Some` and `None`

The loop above is your first close look at a very common Rust pattern. The word `Some` appears three times, and it is worth slowing down because it expresses how Rust handles "maybe a value, maybe not."

Rust has no `null`. If a value might be absent, its type says so explicitly as an `Option<T>`. `Some` means "a value is present here"; `None` means "there is no value." This is exactly Java's `java.util.Optional<T>`, where `Some(x)` ≈ `Optional.of(x)` and `None` ≈ `Optional.empty()`. The compiler forces you to deal with the empty case, so you can't accidentally dereference a missing value the way you can hit a `NullPointerException`.

Walking the loop:

```rust
while let Some((message, usage)) = stream.next().await.transpose()? {
    if let Some(message) = message {
        ...
    }
    if let Some(usage) = usage {
        ...
    }
}
```

- **`while let Some((message, usage)) = ...`** — A stream yields a sequence of items. `next()` returns `Some(item)` while data remains and `None` once the stream is exhausted, which is what ends the loop. `Some((message, usage))` pulls the inner tuple apart and binds `message` and `usage` for the body.
- **`.await.transpose()?`** — `next()` here returns a double-nested container: *maybe there is an item, and if so, the work might have failed.* `transpose()` reshapes that so errors are handled first, and `?` bails out of the whole function if the stream reports an error. After `?`, the only thing left to match is whether an item exists at all.
- **`if let Some(message) = message`** — Inside the tuple, each field is itself `Some` or `None`. The `message` field holds streamed text on a text-delta event but is absent on a completion event (which instead carries `usage`). So the inner check means "only touch the message when one is actually present."

Every `Some` is the compiler saying "a value is here, go ahead and use it"; every matching `None` is the case the loop treats as "nothing to do here." Learning to read `Some`/`None` is the key to reading idiomatic Rust.

## Step 4.3: Start with one user message

The `messages` vector is the application's in-memory conversation history. It starts with the user's first turn:

```rust
let mut messages = vec![Message::user()
    .with_text("What is the capital of France?")];

let first_response = stream_response(provider.as_ref(), &model, &messages).await?;
```

## Step 4.4: Append assistant-role and follow-up turns

After the first stream completes, the application extends the history with two more turns: the assistant-role text it retained and the user's follow-up. The assistant-role message must come before the follow-up, because this preserves what each participant said and when.

```rust
messages.extend([
    Message::assistant().with_text(first_response),
    Message::user().with_text("Tell me the historical origin of this city. Write no more than 2 sentences"),
]);
```

The second request receives all three entries, in order: original user question, assistant-role response, follow-up user question. The application owns this history; the provider object does not preserve it for later calls.

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
- The first response becomes an assistant-role message.
- The second request receives all three history messages in order.
- Both calls use the same separate system instruction.
- Usage diagnostics are printed only when completion usage is supplied.

## Success criteria

- Both calls receive text and completion usage metadata.
- The first response is reconstructed from its text fragments.
- The history is user, assistant-role, user in that order.
- System instructions remain separate from history.
- No exact model wording is required.

## Next

Lesson 5 begins the day-trading teaching-assistant domain with one deterministic maximum-planned-loss tool. Learners will define its raw tool schema, advertise it to provider inference, inspect every returned content block, and preserve the ID of a structured request. They will stop at that authorization boundary: advertising or receiving a request does not execute the tool. Lesson 6 will validate the untrusted arguments, dispatch the allowlisted Rust calculation, return a correlated user-role tool response, and request the final educational explanation.

The first tool uses decimal-string prices parsed into exact decimal arithmetic rather than binary floating point. The initial scenario—entry `51.20`, stop `50.70`, and 200 shares—has a deterministic maximum planned loss of `100.00` before fees, slippage, or a gap through the stop. Explicit model selection is intentionally deferred to the later CLI lesson, when models can be compared on structured tool reliability and other application requirements.