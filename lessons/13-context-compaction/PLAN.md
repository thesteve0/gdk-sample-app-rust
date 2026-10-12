# Lesson 13 plan — Context compaction for continuing learning

**Planning only — 2026-10-11. No implementation authorized or supplied.** Read [the shared handoff](../final-lessons-handoff.md). Compaction is its own lesson, not part of historical simulation.

## Goal and concepts

A **context window** is the bounded input a model can consider for one inference. **Working context** is the information the application supplies for that inference. **Compaction** replaces selected older working context with a shorter representation so the conversation can continue. A summary is lossy generated content, not an exact archive or proof that its claims are true.

Distinguish authoritative application state, the learner-approved persistent record, and model working context. Changing the last must not silently rewrite the first two. The ordinary long-running learning conversation from Lessons 10–12 is the setting; no simulator, market feed, or new domain subsystem is needed.

## Verified Goose/GDK boundary

[Official Goose smart context management](https://goose-docs.ai/docs/guides/sessions/smart-context-management) documents full Goose application compaction. Do not copy its defaults or environment variables as if they configure this custom Rust app.

Pinned goose-agent 0.1.0-alpha.11 `operation.rs` exposes Operation and conversation replacement/visibility Effects; `machine.rs` provides ordered applicability, effect application and reloading. Our runtime uses its own effect enum and must explicitly support required changes. The crate does not provide the complete Goose compaction assembly or policy. Teach a small application-owned Compaction Operation, with inference still coordinated by the GDK, after verifying exact source again at implementation.

## Required design decisions before coding

1. Establish a verified context capacity or conservative application input budget. Current provider JSON context_limit is null. Select counting/estimation tooling and explain whether it is approximate. Budget includes common instructions, approved record context, tools/schema, retrieved passages, retained history, summary-generation input and output, and answer headroom—not history alone.
2. Select a safe trigger and classroom demonstration threshold. The default should avoid an over-capacity request; an adjustable conservative threshold may make compaction observable without hours of conversation. No numeric value is approved yet. Very large static content needs its own bounds, not repeated compaction.
3. Select the runtime representation: retained recent turns plus a clearly labeled summary, with original evidence kept separately when needed for source checks and diagnostics. Do not assume ReplaceConversation automatically archives anything. A bounded actual-learner quotation ledger may be needed; it must have explicit provenance and lifecycle rather than retaining unbounded hidden state.
4. Define boundaries and atomicity. Compact only complete older exchanges at a safe point, never split an outstanding tool request/response pair or remove the active kickoff/current-turn approval candidates. Build and validate the replacement before applying it. If summary generation fails, retain the original state; do not proceed over budget or silently truncate. Decide an explicit stop/yield/error policy.
5. Define summary format, validation, provenance and resource budgets. A bounded summary call must not gain tools or memory-write authority. Count compaction work and usage; cooperate with cancellation, avoid retry/compaction loops, and explain how current four-request/eight-pass/180-second limits apply. Reaching a bound is not successful compaction.

## Summary policy

Preserve stated goals, actual learner answer evidence with source references, unresolved questions/misconceptions, agreed activities, relevant source references, and qualifications of calculator/tool results. Separate tutor explanations from learner statements and suggestions from accepted activities. Distinguish older evidence from current corrections; no inferred mastery or readiness to trade.

Label summary text as derived, fallible data, not higher-priority instructions. Preserve the common contract and selected teaching guidance separately. Do not promote document or learner instructions through summarization. Exact calculator results and persisted approvals remain authoritative application facts, not values recomputed by the summarizer. Citations need a retained/retrievable mapping; a reference string alone does not preserve its supporting passage.

## Critical Lesson 11 compatibility work

Current proposal validation searches user messages with `.skip(1)` and substring matching. After compaction, a summary quotation is **not** an actual learner reply. Preserve a bounded, application-owned provenance path for eligible actual replies, or restrict proposal eligibility explicitly to retained real replies. Settle this choice with the instructor and document any behavior change; do not weaken exact-source validation.

`messages_since_kickoff`, current-turn inspection, pending requests, proposal correlation, approval discovery, message IDs and metadata must continue working after replacement. Prevent repeated approvals or treating summaries as kickoff messages. Record approval remains local and default-no after a successful run; compaction never writes learner-record.md.

## Classroom and deliverables when authorized

Supply the complete Operation/effect/runtime/concurrency support, editable summary guidance, lesson, instructor notes, and a simple before/after visual with text equivalent. Explain decision points before APIs. No missing Rust scaffolding.

Predict which facts should survive a continuing conversation; run until the conservative demonstration trigger; observe a short learner-facing notice; inspect before/after working context and complete diagnostics privately; ask a follow-up; tweak the threshold or summary guidance and repeat from a fresh selection. Never claim exact transcript fidelity. Do not show developer dumps in normal dialogue or deliberately overflow the provider as an exercise.

## Acceptance and validation

- Compaction applies at a safe boundary, materially reduces measured/estimated working input, retains recent complete exchanges and supports a successful live follow-up.
- Deterministic tests verify trigger/budget behavior, no repeated compaction loop, pairing/IDs/kickoff, cancellation/failure retention, real-quote provenance, and no automatic persistent write or duplicate approval.
- Compare actual learner evidence, source references, qualifications and unresolved questions before/after; report invented, lost or strengthened claims as failures. Generated-summary fidelity requires live inspection, not only schema checks.
- Log compaction reason, counting method, sizes, source coverage, summary, applied replacement, usage and available partial state in plain diagnostics. Preserve complete runtime fields/default metadata; diagnostics are not memory.
- Run standard Rust checks and documented provider/datastore run. Add cases for Lesson 14; do not call a single successful summary reliable long-term memory.

Next: Lesson 14 evaluates retrieval, compaction, teaching behavior and enforced application boundaries together.
