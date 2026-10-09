# Lesson 3 Instructor Notes: Streaming Call

## Teaching objective

Introduce the first inference request while connecting it to Lesson 1's application-owned workflow. Define inference, response stream, partial update, message, and turn before learners see the corresponding code; then keep Rust and GDK concepts to the minimum required for message construction, per-request model selection, async streaming, and structural stream handling.

## Suggested pacing

- Trace the application → provider/model → streamed-response flow: 5 minutes
- Define inference, message, turn, stream, and partial update: 5 minutes
- Review provider setup and first-model selection: 5 minutes
- Construct a native `Message`: 15 minutes
- Explain `ModelConfig` and `provider.stream`: 10 minutes
- Walk through Tokio, `StreamExt`, and the message stream: 15 minutes
- Run, inspect usage, and review: 10 minutes

## Before class

- Complete the Lesson 2 connectivity check against the demonstration provider.
- Run the reference solution with the current exact provider-library release.
- Confirm the provider returns streamed text promptly.
- Review the prose and source together before teaching.

## Discussion prompts

- Which responsibilities remain with the application during an inference request?
- Why is this a single inference exchange rather than an agentic loop?
- What is inference, and which part is the application versus the model?
- What is the difference between a message and a turn?
- What does streaming change about when the application can display model output?
- Why does a request select a model when the JSON already declares models?
- What does `Message::user().with_text(...)` express about a turn?
- Why is each stream item allowed to contain a message, usage, or both?
- Why use `StreamExt::next` and `transpose` in this loop?
- Why keep text on stdout and diagnostics on stderr?
- What does completion usage prove that a text fragment does not?

## Live-demo cautions

- Generated text is nondeterministic; validate structure, not wording.
- Do not imply that stream exhaustion alone is proof of normal completion; the reference requires completion usage metadata.
- Provider errors are returned through the stream item's `Result` and propagated by `transpose()?`.
- Define inference before using it as shorthand for `provider.stream`. Avoid saying that the application “thinks” or “answers”: the application assembles and handles the request, while the selected model generates the assistant-role response.
- A partial update is not a complete retained conversation message; this distinction becomes important in Lessons 4 and 5.
- Do not introduce tool requests in detail yet.

## Terminal presentation guidance

- Point out bold cyan application headings and the green connection check; the actual streamed answer and metadata values stay normal. Usage is not dimmed.
- Headings keep their existing destinations (including the streaming heading on stderr). The supplied `main` wrapper prints fatal errors once in bold red on stderr and exits 1.
- stdout and stderr are gated independently by `IsTerminal`; redirected output, a present `NO_COLOR` (including empty), or `TERM=dumb` disables styling. Plain output has the same strings and layout.
- Treat the local presentation helpers and error wrapper as supplied support, not a Rust fill-in exercise. There are no new printed user-input sites or payload fences; color is not protocol data or a validation criterion.

## Checkpoint

Learners receive streamed text, observe completion metadata, and identify the provider, selected model, system instruction, user message, stream, and partial message values.


## Terminal styling validation — 2026-10-06

- Using this lesson's complete source with the shared pinned manifest in temporary staging, `cargo fmt --check`, `cargo check`, `cargo clippy --all-targets`, and `cargo test` passed (2 tests). Root source and dependencies were not replaced.
- Successful terminal-colored, redirected/plain, and empty-`NO_COLOR` runs passed. `TERM=dumb` and independent stdout/stderr gating were also verified with error-output probes; errors retained exit status 1 and terminal stderr was bold red.
- The live exchange returned streamed text and complete undimmed usage metadata identifying the answering model.
