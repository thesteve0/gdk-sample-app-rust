# Lesson 8: One Streamed Exchange Through the State Machine

## Goal

Run Lesson 7's question/answer exchange. Explain the two paths back to our application: **Events for display** and **Effects for recorded state**.

The [complete supplied implementation](src/main.rs) is ready from the outset. We will predict, run, explain, change the question, and run again—not fill in Rust scaffolding.

## Prerequisites and run setup

- Complete Lesson 7 and have the working provider setup from Lessons 2–6.
- Use the repository's pinned Rust toolchain and shared root manifest. `goose-agent` and `goose-providers` are **0.1.0-alpha.11**; no dependency change is needed.
- Have a live streaming provider. The bundled JSON is a replaceable default, not a required service.

The root working copy now contains Lesson 9. To run this earlier reference, use the shared root workflow (save any root edits first):

```bash
cp lessons/08-machine-request-response/src/main.rs src/main.rs
cargo run -- custom_aa_llama_qwen3_6-35b.json
```

You may supply your own provider JSON instead, or omit the argument to use the bundled default. As before, the application loads `.env` before provider construction and selects the first configured model. Lesson directories are not independent Cargo packages.

## 8.1 — Predict the recorded state

A **Session snapshot** is a loaded view of recorded data at a particular point. It is not the exact serialized request sent to a provider.

Our Session has an ID, a conversation, and any recorded usage metadata. The **in-memory store** retains those values across the machine's passes, but not across process exit. The **runtime** supplies the application boundaries the engine needs: load a Session and apply its Effects.

```rust
struct ChatSession {
    id: String,
    conversation: Conversation,
    usage: Vec<ProviderUsage>,
}
```

The supplied `MachineSession` implementation exposes the ID and conversation. `SessionLoader` returns a cloned snapshot from the store. `EffectHandler` changes the stored Session, not the earlier snapshot.

**Predict:** BEFORE contains one user question. What must AFTER contain for re-evaluation to stop? Run the program now and compare the two views.

Before the first Session view, the application prints the configured operations in evaluation order. Names come from the actual steps supplied to the machine; this lesson has one inference step named `llm`. **That name is hard-wired by the GDK-provided `InferenceRunner`; we do not assign it.** It identifies the Operation, not the configured provider or model. The `Session before` and `Session after` headings identify recorded-state views. Each field name is followed by its value: `id`, `conversation`, and `usage`, including every nested message field and its metadata. Absent values (`None`), false flags, and empty collections remain visible. These are application-owned snapshots, not exact prepared provider requests.

**Terminal styling** is presentation-only color that helps distinguish application labels from recorded values. As in Lesson 9, headings and actor labels are bold cyan; every Session attribute name is gold (`#FFD700`), including nested metadata and quoted object keys. Values remain in the default foreground, unbolded. Only the top-level Session `usage` section is dimmed; conversation metadata, including any nested usage, is not dimmed. Live model text stays in the default foreground. Errors are bold red on stderr. These supplied helpers do not change Events, Effects, saved data, or provider input and are not a formatter-writing exercise. Output is plain when its stream is redirected, `NO_COLOR` is set (even empty), or `TERM=dumb`. The excerpt below shows that plain output.

The display uses Rust's expanded field formatting so serialization does not silently omit absent/default metadata fields. The supplied helper prints the Session fields explicitly and expands their values; learners do not need to implement a formatter. Two blank lines after each raw Session dump separate its Rust field values from the next narrative label. There are no payload fences or repeated commentary lines.

## 8.2 — Assemble one inference step

**Inference** asks the provider/model for a response. The GDK supplies an **InferenceRunner** that checks whether a reply is needed, consumes the provider stream, and reconstructs response messages. Our application supplies its Session, runtime, and Effect vocabulary—the assembly choices introduced in Lesson 7.

```rust
let runner = InferenceRunner::new(provider, model);
let steps: Vec<Step<'_, ChatSession, ChatEffect>> =
    vec![Step::Inference(Arc::new(runner))];
let cancel = CancellationToken::new();
let machine = StateMachine::new(steps, cancel.clone());
```

`Arc` provides the shared ownership these interfaces require. The cancellation token is required wiring; this lesson does not trigger it or assign a cancellation exercise.

There is only one configured step. The engine owns the pass loop:

```rust
// In production, consider adding a safeguard to prevent a runaway loop.
let run_result = machine.run(&runtime, SESSION_ID, &emit).await;
```

This lesson deliberately has no loop safeguard. Normal completion follows from the saved ordinary assistant answer, not a pass counter. The successful run returns the final reloaded Session.

## 8.3 — Two paths: display and save

An **Event** is a notification sent to the application during work. The **Emitter** sends Events; it does not save conversation history. A message Event may contain just a fragment, not a complete answer.

An **Effect** proposes a recorded-data change. Our effect handler saves complete reconstructed messages and usage metadata returned by inference.

| Path | Purpose | Application responsibility |
|---|---|---|
| Inference → Emitter → Events | Present output while inference runs | Display text fragments |
| Inference → Effects → effect handler | Retain recorded state | Append complete messages and record usage |

Our two Effect variants are:

```rust
enum ChatEffect {
    AppendMessage(Message),
    RecordUsage(ProviderUsage),
}
```

The supplied `From<Message>` converts a reconstructed message into an append Effect. `InferenceEffect` provides usage Effects. `MachineEffect` ensures message IDs exist before saving. These are interface requirements, not extra decision steps; learners need not reproduce their Rust implementations.

**Predict:** if text appears on the terminal, has it necessarily been saved? No. Only the effect handler changes the store. The code keeps that handler silent during streaming so explanatory lines do not interrupt the answer.

## 8.4 — Display while the run executes

A **channel** is the application-owned queue between the Emitter and our display consumer. Its bounded capacity allows the producer to wait when the consumer falls behind rather than retaining a whole response indefinitely.

**Concurrent consumption** means progressing the machine and the receiver together. Here, `tokio::join!` polls both asynchronous paths in the same task; no separate spawned task is required.

```rust
let (event_tx, event_rx) = mpsc::channel::<AgentEvent>(32);
let emit = Emitter::new(event_tx, cancel);
let run = async {
    // In production, consider adding a safeguard to prevent a runaway loop.
    let run_result = machine.run(&runtime, SESSION_ID, &emit).await;
    // Close the only sender even on error, so the consumer drains and finishes.
    drop(emit);
    run_result
};
let display = display_events(event_rx);
let (run_result, display_result) = tokio::join!(run, display);
```

Before starting the run, the application prints `application → provider:` to mark the outbound exchange—not an exact request dump. The receiver prints one short `provider → application (stream):` label, then each text fragment, flushing stdout as it arrives. Live display is text-only; the Session view later expands all saved content blocks, including any non-text content, along with message and usage metadata. A reasoning-capable model may therefore make the saved-state view longer than the streamed answer.

**Shutdown:** when the run returns—even with an error—dropping the only sender closes the channel. The receiver consumes any queued Events before ending. The application waits for both paths before printing AFTER. If display I/O fails, it reports that error instead; output-failure recovery is outside this lesson. It does not wait until the run ends to begin receiving.

## 8.5 — Explain the result, then tweak

Illustrative excerpt, not an exact transcript. The actual Session views expand **all fields**, with no omissions; this excerpt shortens the conversation and usage values only to focus the walkthrough.

```text
State-machine operations (in order):
  1. llm

Session before
id: "lesson-08"
conversation: [ ... user Message with content and metadata ... ]
usage: []


application → provider:
provider → application (stream):
The capital of France is Paris.

Session after
id: "lesson-08"
conversation: [ ... user Message, saved assistant Message ... ]
usage: [ ... ProviderUsage fields ... ]


```

Inside each message, inspect `id`, `role`, `created`, `content`, and `metadata`. Metadata includes `user_visible`, `agent_visible`, `inference`, `output_token_limit_reached`, `steer`, `turn_context`, `usage`, and `operations`. Expand the inference and usage values when present; their field names and values are printed too. No extra metadata is manufactured just for display.

The answer appears twice intentionally: once through live Events, then inside the saved content. In the ordinary case there is one provider exchange: the first pass acts; the next pass loads the saved answer, inference declines, and the run returns control. Events are not passes or saved messages. The stream label describes the ordinary provider exchange; SDK-generated error/empty-response diagnostics are not model answers. The application inspects the final Session and reports failures rather than declaring success merely because the run returned.

**Tweak:** change only `USER_PROMPT` in root `src/main.rs`, for example to “What is the capital of Japan?” Predict the answer and run again. The store is freshly seeded on each process start; this is not a follow-up turn.

## Validation and success criteria

From the repository root:

```bash
cargo fmt --check
cargo check
cargo clippy --all-targets
cargo test
cargo run -- custom_aa_llama_qwen3_6-35b.json
```

Success means:

- BEFORE contains the question; AFTER retains it and an ordinary assistant reply.
- Response text is displayed through Events during the run, then found in recorded state.
- The consumer drains and finishes before AFTER; the ordinary run terminates without a counter.
- You can identify the Emitter/display path and the separate Effect/saving path.

Text, saved content/metadata, message counts, and usage availability depend on the provider. Do not require exact wording. If the provider is unavailable, ask the instructor to start or supply one; tests do not replace live validation.

## Next and references

Lesson 9 will retain one Session across successive user turns and add the planned-loss tool. This lesson adds neither tools nor follow-up input.

Scope follows the [README](../../README.md#replacement-state-machine-lesson-plan) and [Working Mental Model](../../reference-material/Goose%20GDK%20State%20Machine%20%E2%80%94%20Working%20Mental%20Model.md). API behavior was verified against `goose-agent` **0.1.0-alpha.11**, especially `machine.rs`, `inference.rs`, `operation.rs`, and `events.rs`; provider/message types were checked in the matching pinned source. See the [instructor notes](../../instructor-notes/08-machine-request-response.md) for validation and teaching guidance.
