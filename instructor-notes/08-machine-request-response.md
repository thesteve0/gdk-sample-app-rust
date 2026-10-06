# Instructor Notes: Lesson 8 — One Streamed Exchange

## Purpose and pacing

Turn Lesson 7's ordinary question/answer trace into a working application. Keep the center of attention on **Events display; Effects save**. Supply the entire reference implementation; do not ask attendees to write trait implementations or concurrency machinery.

Suggested 30–35 minutes:

1. Recall the Session and predict BEFORE/AFTER: 5 minutes.
2. Run once and locate the two copies of the answer: 5 minutes.
3. Explain the one-step assembly and saving boundary: 8 minutes.
4. Follow the Event path and its shutdown: 7 minutes.
5. Change the question, predict, run again, and checkpoint: 5–10 minutes.

## Run preparation

The instructor authorized replacement Lesson 8 and updating root `src/main.rs` on 2026-10-05. Root and the complete lesson reference use the shared manifest; dependencies and provider configuration are unchanged. Withdrawn drafts remain untouched.

From the repository root, save any learner edits before restoring:

```bash
cp lessons/08-machine-request-response/src/main.rs src/main.rs
cargo run -- custom_aa_llama_qwen3_6-35b.json
```

Substitute a valid provider JSON when necessary. Check endpoint availability before class. If unavailable, ask for a live provider; do not present unit-test output as model streaming. Keep `.env` and credentials private.

## Prediction prompts and answers

- **What should be in BEFORE?** One recorded user question; no answer yet.
- **Does seeing text prove it was saved?** No. Events support presentation; the handler applies Effects to the store.
- **Why is the answer printed twice?** First the text fragments from message Events, later inside the complete saved Session field view.
- **Who owns the loop?** `StateMachine::run`. There is no application-written pass loop.
- **Why does inference stop?** In the ordinary case, reloaded history ends with an assistant answer, so the inference predicate declines. This is not a remembered completion flag or necessarily explicit yield.
- **Are 100 message Events 100 saved messages or passes?** No. Stream fragments may merge into a complete message; non-text blocks and provider metadata affect counts.
- **Does a new terminal run continue the conversation?** No. It creates a new in-memory store. Change `USER_PROMPT` to a question about Japan; do not add another user turn in this lesson.

## Source walkthrough

Use small regions of the supplied file, not a full-file lecture:

1. `ChatSession` and `ChatRuntime::seeded`: the application assembly owns conversation and usage records. Loader clones snapshots; saving changes the stored Session, not the BEFORE value.
2. `InferenceRunner`, one `Step::Inference`, and `StateMachine::run`: the engine coordinates the work.
3. `ChatEffect` and the handler: message Effects append through `Conversation::push`; usage Effects are actually retained, not merely logged.
4. `display_events` and `tokio::join!`: the receiver progresses during execution and flushes text as it arrives.

Explain the supporting Rust only as needed: `Arc` is shared ownership required by the interfaces, the mutex protects store access, and the async trait implementations are supplied loading/saving contracts. `From<Message>`, `InferenceEffect`, and `MachineEffect` adapt the application's Effects to the runner. Message-ID plumbing is provided, not a learner exercise.

### Streaming lifecycle

The bounded channel holds 32 Events; it need not hold the entire response because the receiver runs concurrently. `tokio::join!` polls the two futures in one task; there is no spawned task to abort or join. An await on a full channel gives the receiver an opportunity to progress.

The run future drops the sole Emitter after either success or an error. This closes the channel; with healthy display I/O, the receiver drains remaining Events and finishes. Both paths finish before AFTER. If display I/O fails, the receiver can terminate early and the application reports an error; no output-recovery exercise is included.

The instructor revised Lesson 8 output on 2026-10-05: no `++++++++` fences, no repeated actor labels inside Session views, and no Event-count/completion commentary. Print the configured operations in order before the first Session view (`1. llm` here), using the actual steps' names. Explain that `llm` is hard-wired by the GDK-provided `InferenceRunner`, not chosen by our application and not a provider/model identifier. Use short Session before/after headings, two blank lines after each raw Rust Session dump, an `application → provider:` label before the run, and one live-stream label. The outbound label marks the exchange beginning, not an exact request dump. The handler remains silent while streaming.

`session_fields` prints all three Session fields and expands the stored values with Rust Debug formatting. This includes every message field, absent/default metadata, non-text content blocks, and complete usage records; JSON serialization would omit some absent/default fields. No exact outbound request is printed: the runner can project/filter/merge history before provider serialization. The complete field view may be long for a reasoning-capable model. Do not silently hide or truncate content or metadata to shorten it.

The live consumer intentionally presents text only; the saved Session view shows all retained content and metadata, including non-text blocks. The pinned inference runner emits message Events but records final provider usage as an Effect; do not promise a separate usage Event or fixed usage count.

### Scope and safeguards

**Instructor decision (2026-10-05): no loop safeguard in Lesson 8.** The source comment beside `StateMachine::run` says production users may want one to prevent a runaway loop. Do not add a load budget, timeout, or manual bounded loop. The cancellation token is required wiring, not a triggered classroom behavior. Lesson 9 execution/request bounds remain a separate implementation decision.

The final validator checks that text was displayed and saved, rejects recorded inference errors and unexpected tool requests, and recognizes the pinned runner's empty-response diagnostic. That diagnostic uses explicit yield, so it must not be called ordinary no-work completion. The stream actor label describes the ordinary provider exchange; failure diagnostics can be generated by the runner, not the model. Do not assign deliberate failure-path exercises.

## Checkpoint and next step

Ask the learner to point at one display path and one save path, then narrate the ordinary load → infer → save → reload → decline trace. Changing only the question should leave that structure intact.

Lesson 9 is not implemented here. It will retain the same Session for successive turns and introduce the deferred planned-loss trace, tool selection, correlation, validation, and explicit execution/request bounds.

## Pinned-source verification

Reviewed `goose-agent` **0.1.0-alpha.11**:

- `README.md`, `machine.rs`: ordered first-applicable scan, effect application, reload between passes, final reload; `run` has no application-specific step-limit argument.
- `operation.rs`: `Emitter` awaits channel sends; message emission generates IDs; Events and Effects have separate interfaces.
- `inference.rs`: applicability from recorded conversation; stream Events while reconstructing through `Conversation::push`; complete message/usage Effects afterward; ordinary answer declines; empty response synthesizes a diagnostic and explicitly yields.
- `events.rs`: the Event variants consumed by this text-only presentation.

Reviewed matching `goose-provider-types` conversation/message/usage source and `goose-providers` declarative construction/model source. The [official provider documentation](https://goose-docs.ai/docs/getting-started/providers) provides supplemental provider-level orientation; pinned source is authoritative for this engine implementation, rather than Goose's full assembly.

## Validation record — 2026-10-05

- `cargo fmt --check`, `cargo check`, `cargo clippy --all-targets`, `cargo test`, the IDE build, local Markdown-link checks, and `git diff --check` passed.
- Tests cover stored message/usage Effects and snapshot independence; missing Session lookup; concurrent consumption with a capacity-one queue, channel closure/drain, and no saving by display; full Session fields including absent/default metadata; ordinary-answer applicability and a no-work `StateMachine::run` without calling the provider.
- Live default-provider run returned a France answer through 127 message Events (119 non-text blocks), then showed two recorded messages and one usage record. These are observed sample counts, not assertions for other models.
- The Event consumer is polled concurrently by construction; the capacity-one test exceeds its queue capacity, so buffering an entire run before consumption would not work. That deterministic test supports lifecycle behavior but does not substitute for the live run.

The approved Japan-question tweak also completed live: 335 message Events (292 non-text blocks), two recorded messages, and one usage record. The France default was restored and rerun successfully. Dependencies, provider configuration, and archived drafts were unchanged. On 2026-10-06 the instructor confirmed Lesson 8 complete and ready to teach; this approval is separate from the successful build/provider runs.

Output revision validation: all five tests and the live France exchange passed with no payload fences, only the stream label and Session headings, and complete saved message/usage fields. Metadata such as `inference`, visibility flags, optional usage, and operation notes remained visible.
