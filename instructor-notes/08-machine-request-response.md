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

Replacement Lesson 8 was authorized on 2026-10-05. The instructor authorized this behavior-preserving modularization on 2026-10-22. The current root is Lesson 14; **this refactor leaves that implementation unchanged**. Lesson 8 still uses the shared root manifest, lockfile, toolchain, and provider JSON; withdrawn drafts remain untouched.

Follow the [lesson's backup/copy/restore workflow](../lessons/08-machine-request-response/LESSON.md#prerequisites-and-run-setup) from the repository root. Back up the entire root `src/`, copy all four Lesson 8 `.rs` files, then run. A single-file copy is no longer sufficient. Keep the backup path and preserve Lesson 8 learner edits when restoring. Do not copy root back into the lesson reference.

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

Use small regions of supplied `main.rs`, not a full-file lecture:

1. `ChatSession` and `ChatRuntime::seeded`: the application assembly owns conversation and usage records. Loader clones snapshots; saving changes the stored Session, not the BEFORE value.
2. `InferenceRunner`, one `Step::Inference`, and `StateMachine::run`: the engine coordinates the work.
3. `ChatEffect` and the handler: message Effects append through `Conversation::push`; usage Effects are actually retained, not merely logged.
4. `display_events` and `tokio::join!`: the receiver progresses during execution and flushes text as it arrives.

### Module navigation, not a new Rust exercise

A module groups code under a name; here sibling files separate supplied helpers from the exchange under study. The `mod` declarations include them and `use` brings selected helpers into scope. Explain this briefly, then return to the four walkthrough regions above; no learner-authored scaffolding is required.

- `main.rs` (289 lines) retains the Session/runtime/Effects, provider construction, machine/channel assembly, concurrent Event consumer, and final validator. The approved `USER_PROMPT` tweak stays here.
- `presentation.rs` owns the color rules and complete Session display, with four local tests. Its renderer reads the parent Session; no public fields/getters or generic formatting framework were added.
- `provider_setup.rs` owns the default path and two existing helpers. Provider construction and `.env` order remain visible in main.
- `tests.rs` contains the four cross-cutting application tests. All eight original tests remain; their locations change, not their assertions.

Only the helpers needed by main (and a test-only plain Session formatter) are `pub(super)`. There is no library crate or shared cross-lesson module. Do not move `display_events` into presentation: following its Event path alongside the saving boundary is the teaching point. Other lessons may be modularized after separate review; none change in this pass.

Explain the supporting Rust only as needed: `Arc` is shared ownership required by the interfaces, the mutex protects store access, and the async trait implementations are supplied loading/saving contracts. `From<Message>`, `InferenceEffect`, and `MachineEffect` adapt the application's Effects to the runner. Message-ID plumbing is provided, not a learner exercise.

### Streaming lifecycle

The bounded channel holds 32 Events; it need not hold the entire response because the receiver runs concurrently. `tokio::join!` polls the two futures in one task; there is no spawned task to abort or join. An await on a full channel gives the receiver an opportunity to progress.

The run future drops the sole Emitter after either success or an error. This closes the channel; with healthy display I/O, the receiver drains remaining Events and finishes. Both paths finish before AFTER. If display I/O fails, the receiver can terminate early and the application reports an error; no output-recovery exercise is included.

Terminal colors now match Lesson 9: bold cyan headings/actor labels, gold (`#FFD700`) Session attribute names at every nesting level, default unbolded values and live model text, bold red stderr errors. Only the top-level Session `usage` section is dimmed, not message metadata or any other content. Color adds no provider input or saved state. Keep the supporting formatter supplied, not an exercise. Redirected streams, `NO_COLOR` (including an empty value), and `TERM=dumb` disable ANSI styling independently for stdout and stderr. Text labels and spacing remain sufficient for plain transcripts.

The instructor revised Lesson 8 output on 2026-10-05: no `++++++++` fences, no repeated actor labels inside Session views, and no Event-count/completion commentary. Print the configured operations in order before the first Session view (`1. llm` here), using the actual steps' names. Explain that `llm` is hard-wired by the GDK-provided `InferenceRunner`, not chosen by our application and not a provider/model identifier. Use short Session before/after headings, two blank lines after each raw Rust Session dump, an `application → provider:` label before the run, and one live-stream label. The outbound label marks the exchange beginning, not an exact request dump. The handler remains silent while streaming.

`session_fields_with_color` prints all three Session fields and expands the stored values with Rust Debug formatting. This includes every message field, absent/default metadata, non-text content blocks, and complete usage records; JSON serialization would omit some absent/default fields. No exact outbound request is printed: the runner can project/filter/merge history before provider serialization. The complete field view may be long for a reasoning-capable model. Do not silently hide or truncate content or metadata to shorten it.

The live consumer intentionally presents text only; the saved Session view shows all retained content and metadata, including non-text blocks. The pinned inference runner emits message Events but records final provider usage as an Effect; do not promise a separate usage Event or fixed usage count.

### Scope and safeguards

**Instructor decision (2026-10-05): no loop safeguard in Lesson 8.** The source comment beside `StateMachine::run` says production users may want one to prevent a runaway loop. Do not add a load budget, timeout, or manual bounded loop. The cancellation token is required wiring, not a triggered classroom behavior. Lesson 9 execution/request bounds remain a separate implementation decision.

The final validator checks that text was displayed and saved, rejects recorded inference errors and unexpected tool requests, and recognizes the pinned runner's empty-response diagnostic. That diagnostic uses explicit yield, so it must not be called ordinary no-work completion. The stream actor label describes the ordinary provider exchange; failure diagnostics can be generated by the runner, not the model. Do not assign deliberate failure-path exercises.

## Checkpoint and next step

Ask the learner to point at one display path and one save path, then narrate the ordinary load → infer → save → reload → decline trace. Changing only the question should leave that structure intact.

The separate Lesson 9 reference retains the same Session for successive turns and introduces the deferred planned-loss trace, tool selection, correlation, validation, and explicit execution/request bounds.

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

The approved Japan-question tweak also completed live: 335 message Events (292 non-text blocks), two recorded messages, and one usage record. The France default was restored and rerun successfully. Dependencies, provider configuration, and archived drafts were unchanged. A 2026-10-06 completion record was superseded by the instructor's 2026-10-07 clarification: Lesson 8 still needs instructor review. Historical successful build/provider runs do not constitute current release approval.

Output revision validation: all five tests and the live France exchange passed with no payload fences, only the stream label and Session headings, and complete saved message/usage fields. Metadata such as `inference`, visibility flags, optional usage, and operation notes remained visible.


## Terminal styling validation — 2026-10-06

- Using this lesson's complete source with the shared pinned manifest in temporary staging, `cargo fmt --check`, `cargo check`, `cargo clippy --all-targets`, and `cargo test` passed (8 tests). Root source and dependencies were not replaced.
- Successful terminal-colored, redirected/plain, and empty-`NO_COLOR` runs passed. `TERM=dumb` and independent stdout/stderr gating were also verified with error-output probes; errors retained exit status 1 and terminal stderr was bold red.
- The live France exchange displayed default-color model text and complete before/after Session views. Gold attributes, default unbolded values, two blank lines, and the no-fence layout were retained. Only the top-level Session `usage` section was dimmed; tests also cover non-dimmed nested message usage and complete original Debug text.


## Modularization validation — 2026-10-22

- Scope: Lesson 8 only; supporting presentation/provider helpers and tests relocated into local modules. Session/runtime/Effects, live Event handling, concurrent execution/shutdown, and final validation stay in main. No functionality, prompt, output convention, dependencies, or provider schema changed.
- With the unchanged shared manifest/lockfile/toolchain/provider JSON in temporary staging, `cargo fmt --check`, `cargo check --locked`, `cargo clippy --locked --all-targets`, `cargo test --locked` (8 tests), and `cargo build --locked` passed. Staging also copied the existing root-registered dataset example/manifest solely so Cargo's all-target checks could resolve it; no catalog operation was run.
- Root Lesson 14 source, earlier/later references, preparation code, and pins remain unchanged. No IDE build or fresh root tutor execution is claimed.
- The endpoint initially refused connections, then became available without an agent-initiated restart. A fresh documented `cargo run --locked -- custom_aa_llama_qwen3_6-35b.json` in staging passed with a France answer, retained user/assistant messages, and recorded usage. No mock provider was used.
- Further real-service runs passed under a PTY with colors, empty `NO_COLOR`, and `TERM=dumb`, and with redirected/plain stdout. Checks confirmed one before/after view and stream label, their order, full retained fields, no payload fences, and two blank lines after the raw dump. Colored snapshots retained cyan headings/gold fields and exactly two dim blocks (top-level usage in BEFORE and AFTER); suppressed-color transcripts contained no application ANSI. The existing tests independently verify complete Debug rendering, unchanged values, and non-dimmed nested usage.
- In the colored run, the stream label arrived about 8.44 seconds after launch and AFTER at 8.90 seconds; the consumer code and retained capacity-one test verify concurrent draining rather than post-run buffering. Timing is observational, not a performance guarantee.
- Separate PTY/pipe combinations verified stdout/stderr gating independently: argument-count errors exited 1 and stderr was red only when terminal-enabled, regardless of stdout. No learner-facing failure exercise was added.
- The approved Japan tweak was applied only in staging, returned Tokyo, and preserved the before/after structure. France was restored and successfully rerun. All 32 original function/test bodies match the pre-refactor source after ignoring formatting whitespace; only module/import placement and required visibility changed.
- Instructor content/teaching/release review remains pending. The instructor wants the modular pattern extended to other lessons eventually, not implicitly applied in this change.
