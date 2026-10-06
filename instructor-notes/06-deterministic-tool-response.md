# Lesson 6 Instructor Notes: Execute the Tool and Return Its Result Through the Raw Provider Protocol

## Teaching objective

Carry the Lesson 5 pending request through its complete raw round trip. Define deserialization, domain validation, allowlisted dispatch, tool response, effective role, tool-level versus routing failure, and round bound before showing their code-level forms. Learners should be able to point to the exact boundary that rejects any malformed request and to the correlation ID that ties each response to its request. The lesson ends after a bounded two-round exchange — no agent loop, no state machine.

## Suggested pacing

- Reconnect the Lesson 5 stop point and define the round trip's three history stages: 10 minutes
- Untrusted arguments: deserialization versus domain validation: 15 minutes
- Exact decimal arithmetic and the domain rules: 10 minutes
- Allowlisted dispatch through the three boundaries: 15 minutes
- Correlated user-role tool response and effective role: 10 minutes
- The stateless follow-up call, outbound history view, and round bound: 10 minutes
- Run live, trace one full round trip, and review: 15 minutes

## Before class

- Confirm the classroom provider is running and reachable at its `base_url`.
- Run `cargo fmt --check`, `cargo check`, `cargo clippy --all-targets`, and `cargo test` against the reference solution; the unit tests cover the exact scenario result and the rejection paths without a provider.
- Re-run the live scenario against the exact classroom provider/model. The spike and Lesson 5 runs observed native `maximum_planned_loss` requests with `entry_price="51.20"`, `stop_price="50.70"`, and `share_count=200`, producing the deterministic `$100.00`.
- Decide how to present the thinking block in round 1: the classroom model narrates tool calls and results inside its private reasoning that never happened — the structured request is still the only ground truth.

## Discussion prompts

- Why are deserialization and domain validation two separate checks, and what does each one reject?
- Why do `entry_price` and `stop_price` stay strings in the domain struct? What would serde_json hand back for a bare JSON number, and why is that unacceptable for money?
- Why is `deny_unknown_fields` the enforcement for `additionalProperties: false` rather than a redundant repetition?
- Walk through the three boundaries in order: parse, allowlist, domain. Which failures belong to each, and why does the order matter?
- Why does every failure become an error result returned to the model instead of a panic, a crash, or a silently dropped request?
- What is the difference between a routing failure and a tool-level failure, and how does the returned reason tell you which boundary fired?
- Why does a tool response ride on a user-role message? What effective role does the provider serialize it with, and how does the `tool_call_id` get populated?
- Why must the response carry the request's own ID rather than the tool name or the response order?
- Why does the follow-up call resend the entire history, and what would the model see if the application sent only the tool response?
- Why is the fixed two-round structure a round bound rather than an agent loop, and how will Lesson 7 explain a different coordination model?
- Why is the deterministic result in the tool response the reason the final explanation can be trusted over model-computed arithmetic?
- Why does the program still advertise the tool in round 2, and what does it do if the model requests the tool again?

## Live-demo cautions

- If round 1 returns text only, the program reports that the round trip could not begin and stops without a second call. That is the same structural outcome Lesson 5 reported; record it, do not debug it as a defect.
- If the model sends an unparseable call, an unknown tool name, or malformed arguments, the dispatch line prints the rejection reason and the run still performs round 2 — the model receives the error result and responds to it. This is a good live demonstration of an error result being protocol-valid, not a failure of the program.
- Some models emit more than one tool request, or request the tool again in round 2. The program answers every round-1 request with its own correlated response and stops at the two-round bound on a re-request; use this to show the bound rather than an unbounded loop.
- The outbound-history view prints roles, effective roles, and blocks — point at the message whose effective role is `tool`; that is the Lesson 6 response in the provider's terms.
- The run prints the full outbound payload and two reconstructed views; streamed text and reconstructed text repeat intentionally. Point at the labeled headers first, and at the `++++++++` payload delimiters: everything between two delimiter lines is a value that traveled to or from the provider, and everything outside them is this application's commentary. Each fence-adjacent line also names the sender and the receiver — `application → provider` on the outbound payload, `provider → application` on the streamed and reconstructed views, and `application, as the tool → provider` in the dispatch phase — because the tool execution boundary lives inside this program and every message must show both of its ends.
- Usage metadata appears twice, once per round; token counts differ.
- The `Conversation` remains in-memory state for one run only. Nothing persists across process exit. Lesson 7 explains Sessions and store lifetime conceptually; Lesson 8 supplies an in-memory store, not disk durability.

## What this lesson must not do

- No repeated agent loop: exactly two inference rounds and round-1 requests only.
- No typed `SyncTool`, `ToolOperation`, `StateMachine`, sessions, effects, or runtime scaffolding.
- No market data, retrieval, or model-selection CLI work.
- No binary floating point anywhere in the currency path.
- No generic shell, HTTP, or order-entry tool; the allowlist stays at one deterministic calculation.
- No persistence of history across program runs.

## Checkpoint

Learners validate untrusted arguments through deserialization and domain validation, dispatch only the allowlisted tool, execute one deterministic exact-decimal calculation, return one correlated user-role tool response per request, resend the full history to the stateless provider for a bounded follow-up round, and identify which boundary rejects any given malformed request.

## Transition to Lesson 7

Lesson 7 motivates the GDK state machine from this manual implementation without code: define Session, Operation, StateMachine, Effect, and re-evaluation, then narrate one simple question/answer exchange. Defer the correlated tool trace to Lesson 9. Do not build a runtime or promise an automatic state-step bound. Lesson 8 supplies an inference-only in-memory runtime with the instructor-approved production-caution comment rather than a loop safeguard; Lesson 9 adds continuing conversation and tools.
