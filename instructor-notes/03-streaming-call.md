# Lesson 3 Instructor Notes: Streaming Call

## Teaching objective

Introduce the first inference request while keeping Rust and GDK concepts to the minimum required for message construction, per-request model selection, async streaming, and structural stream handling.

## Suggested pacing

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
- Do not introduce tool requests in detail yet.

## Checkpoint

Learners receive streamed text, observe completion metadata, and identify the provider, selected model, system instruction, user message, stream, and partial message values.
