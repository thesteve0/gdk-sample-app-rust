# Lesson 5 Instructor Notes: Define a Deterministic Tool and Inspect Its Raw Request

## Teaching objective

Build on the application-owned workflow established in Lessons 1–4 and make the raw provider tool boundary explicit. Define structured assistant output, tool request, and correlation ID before examining their fields. Learners should understand that an application advertises a deterministic capability, a model may request it, and the application alone authorizes, validates, and executes it. The lesson ends at the pending request—not at a calculation or final response.

## Suggested pacing

- Reconnect the Lesson 1 application/model boundary and define structured output: 10 minutes
- Tool-request boundary and why deterministic arithmetic belongs in application code: 10 minutes
- Scenario card and raw protocol diagram: 10 minutes
- Define one narrowly advertised schema: 15 minutes
- Advertise the tool in a single `stream` call: 10 minutes
- Reconstruct messages with `Conversation::push` merge semantics: 10 minutes
- Inspect every content block and read request ID/name/arguments safely: 15 minutes
- Run live, record the observed block shape, and review: 15 minutes

## Before class

- Confirm the classroom provider is running and reachable at its `base_url`.
- Run `cargo fmt --check`, `cargo check`, `cargo clippy --all-targets`, and `cargo test` against the reference solution.
- Re-run the live scenario against the exact classroom provider/model. The recorded spike observed native `maximum_planned_loss` requests with `entry_price="51.20"`, `stop_price="50.70"`, and `share_count=200`, but this is compatibility evidence rather than a guarantee for every provider/model/run.
- Use the same hypothetical scenario. Lesson 6 will calculate `100.00` before fees, slippage, or a gap through the stop.

## Discussion prompts

- Why is a deterministic Rust calculation preferable to trusting model-generated arithmetic?
- What makes a tool request structured output rather than prose the application must interpret?
- Why does a request ID act as a correlation ID, even before this lesson returns a result?
- What does the model receive when the application advertises a tool, and what does it send back when it wants that capability?
- Why does the advertised schema not validate or authorize received arguments?
- Why are tool names and arguments untrusted model output?
- What information survives in a reconstructed message that would be lost by keeping only `as_concat_text()` output?
- Does the Lesson 5 `Conversation` contain the original user message too? Why or why not?
- Why inspect every content block instead of only the first one?
- Why read the tool call through its `Result` instead of unwrapping it?
- Why reconstruct streamed deltas with `Conversation::push` before inspection?
- Why is a request ID needed even though Lesson 5 does not yet return a result?

## Live-demo cautions

- A model may reply with plain text rather than a structured request. Record that outcome; it is not automatically a code defect.
- The provider JSON does not guarantee native tool calling, and the reference program does not expose a finish reason. Do not promise `tool_calls` as a live-run result.
- `Message::as_concat_text()` prints text blocks only. A provider may supply a `Thinking` or other non-text block; the reconstructed block-inspection phase renders it separately.
- First point out live text printed from stream deltas, then the reconstructed-content view. Their possible text repetition is intentional: it demonstrates two concerns, responsive display and structured protocol inspection.
- The `Conversation` is retained in memory during the program run only. Do not imply that Lesson 5 creates persistent cross-run state.
- Do not extend the exercise into decimal calculation, `CallToolResult`, tool responses, an agent loop, `SyncTool`, `ToolOperation`, sessions, effects, market data, retrieval, or explicit model selection.

## What this lesson must not do

- No `rust_decimal` calculation code.
- No validation or allowlisted dispatch of model arguments.
- No `CallToolResult` or user-role tool response.
- No repeated agent loop.
- No typed `SyncTool` or `ToolOperation`.
- No sessions, effects, runtime scaffolding, or persistent storage.
- No market data, retrieval, or model-selection CLI.

## Checkpoint

Learners define one narrowly advertised deterministic tool schema, advertise it to a single inference call, reconstruct messages with merge semantics, inspect every content block, and safely recognize either a parseable request or an unparseable request. They stop before execution, retaining the complete assistant message only in memory for the next raw-protocol step.

## Transition to Lesson 6

Lesson 6 carries the assistant tool-request message forward, treats the model-provided name and arguments as untrusted input, validates and allowlists the call, parses decimal strings exactly, executes the deterministic calculation, sends a correlated user-role response using the request ID, and requests the final educational explanation with the fees, slippage, and gap-through-stop exclusions.
