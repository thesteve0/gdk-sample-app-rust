# Lesson 14 — Lesson 13 RAG library integration plan

**Date:** 2026-11-13  
**Status:** Implemented RAG-only checkpoint (2026-11-13); root activated to the complete Lesson 14 tree. Instructor review pending. See [actual validation](VALIDATION.md#rag-only-library-integration--2026-11-13); editable-prompt reconciliation and §6 remain deferred.

## Goal and scope

Reuse Lesson 13's RAG library and its necessary application wiring in Lesson 14 so retrieval is inherited functionality, not a new implementation taught in the evaluation lesson.

This phase covers RAG only. Moving Lesson 13's compaction and memory features into a library, integrating those features into Lesson 14, and moving the editable teaching prompts are later work. Do not copy Lesson 13's entire application over Lesson 14.

## 1. Preserve the baseline

- Inspect the working tree before implementation and preserve unrelated changes.
- Use the complete `lessons/13-context-compaction/src/lib.rs` and `src/rag/` subtree as the canonical RAG implementation, including child tests.
- Preserve Lesson 12 and Lesson 13 references, Lesson 14's evaluation-storage isolation, telemetry lifecycle, existing evaluation assets, and historical validation records.
- Keep the existing package, toolchain and dependency pins. No workspace, separate retrieval package, shared course framework, service restart, corpus mutation, or catalog write.
- Do not activate another root lesson during this planning phase. Decide activation at the implementation checkpoint; never overwrite real learner records or diagnostics for validation.

## 2. Supply the same library in Lesson 14

Add the same-package library target through this source layout:

```text
src/lib.rs
src/rag/
├── mod.rs
├── retrieval.rs
├── retrieval/tests.rs
├── grounding.rs
├── citations.rs
├── transport.rs
└── tests.rs
```

- Reuse the Lesson 13 implementation rather than independently rewriting its algorithms.
- Follow the Lesson 14 source-header convention; otherwise keep library behavior and tests aligned with Lesson 13.
- Pass service defaults, retrieval limits, grounding policy and the RAG-specific grounding contract explicitly from the application, as Lesson 13 does.
- Retain local-only transport checks, read-only access, provenance validation, tokenizer accounting, search serialization, cancellation and database-driver ownership.
- Remove Lesson 14's superseded local retrieval module after all consumers use `gdk_hello::rag`.

## 3. Integrate the inherited RAG workflow

Adapt Lesson 14's application at the existing context, runtime and presentation boundaries:

1. Initialize retrieval through the library with explicit defaults and limits.
2. Prepare grounding once per genuine learner question, using Lesson 13's selection policy and budget. Do not retrieve again for the application-authored correction attempt.
3. Include the pending question and replacement grounding attachment in input-budget checks before accepting them into the Session.
4. Store accepted question/evidence state together and contribute the RAG-specific grounding prompt part in Lesson 13's order. Evidence remains separate from conversation history.
5. Remove the model-selected search provider, advertised search definition, search allowlist branch and obsolete shared search-tool evidence ledger. Preserve calculator and note-proposal tools and their limits.
6. Carry forward the citation registry, citation warnings/reference footer, buffered answer presentation and single bounded acknowledgment correction.
7. Continue consuming Events concurrently. Preserve timeout, cancellation, partial diagnostics, sender shutdown and Drop ordering.
8. Keep note approval independent: settle first-attempt proposals before appending a correction reminder, exclude the reminder from genuine learner provenance, and retain attempt-scoped settlement where needed for this workflow.

Only make the compaction/memory compatibility edits required to keep this RAG workflow safe. Do not extract those subsystems, replace their broader policies, or introduce their later features in this phase. Keep application authority and approval decisions visible in main.

**Prompt boundary:** the RAG-specific contract and correction text are part of the inherited RAG wiring. Do not migrate or rewrite `content/tutor-contract.md`, `getting-started-guidance.md`, `getting-started-material.md`, or `compaction-guidance.md` in this phase. Their reconciliation is explicitly deferred below; passing structural checks must not be described as completed teaching/prompt integration.

## 4. Preserve evaluation instrumentation

- Keep Lesson 14's telemetry initialization, shutdown/flush, storage isolation, inference observations, approval observations and summary instrumentation.
- Trace automatic grounding as application preparation/retrieval, not as a model-requested tool execution.
- Make selected evidence, attempt boundaries, withheld replies, the one correction and the learner-visible answer distinguishable in traces.
- Compose existing inference instrumentation with RAG prompt diagnostics without exporting provider reasoning, credentials or configuration.
- Update `course.content_sha256` to cover `src/lib.rs` and every production RAG module, removing the obsolete local retrieval path. Preserve the metadata schema and purpose.
- Preserve collection/profile identity through a nonsecret interface. If a shared identity accessor is needed, make the smallest identical addition in both library copies rather than duplicating profile literals in Lesson 14.
- Leave the dataset seeder, immutable catalog definitions and catalog contents untouched.

## 5. Migrate and validate coverage

- Preserve the library's tests and compare the two RAG implementations, allowing only documented source-header or shared identity-interface changes.
- Replace obsolete search-tool orchestration tests with automatic-grounding tests; retain equivalent coverage of budgets, failures, privacy and source provenance.
- Cover replacement evidence, no-evidence/unavailable outcomes, citation-registry membership, buffered presentation, the one-retry limit, correction-message provenance, approval ordering and pending-input budget checks.
- Retain Lesson 14's evaluation isolation and telemetry tests; add checks for the new source inventory and attempt/visible-output distinctions.
- Run formatting, check, all-target Clippy, tests and build, including the existing calibration consumer where applicable. Do not invoke the dataset seeder against the catalog.
- Use isolated disposable storage for real-service and CLI checks. Temporary packages must have independent `CARGO_TARGET_DIR` values and verified compile-time repository paths.
- Exercise actual retrieval, a grounded answer, reference resolution, calculator continuity and independent note decisions. Rehearse correction branches where possible; label deterministic-only coverage honestly when a live model does not produce the required branch.
- Record unavailable prerequisites as blocked. Never substitute mocks for claimed live validation, restart services, or overwrite real records/diagnostics.
- Report structural outcomes separately from teaching quality. Deferred prompt conflicts remain a limitation of this intermediate phase.

## 6. Deferred work for the compaction/memory library integration

Track these items for the later phase; do not resolve them incidentally during RAG reuse:

- Move Lesson 13's compaction and memory features behind the separately agreed library boundaries, then integrate their application-facing APIs into Lesson 14.
- Move and reconcile Lesson 13's editable teaching prompts, including automatic retrieval versus optional search, citation scope, continuity, activity/source-review instructions and memory guidance. Do not advertise commands or guarantees absent from the integrated source.
- Reconcile compaction triggers, eligible-history minimums, `/learned` persistence/read-consent behavior and informational context displays explicitly rather than inheriting them accidentally.
- Review quotation eligibility, approval-ledger behavior and continuity across compaction beyond the minimal compatibility required by this RAG phase.
- Review citation edge cases: registry registration before accepted delivery; fabricated references on no-evidence turns; lexical versus strict marker syntax; citation-plus-decline handling; and presentation after an unsuccessful correction. Keep exact Lesson 13 parity in this phase rather than silently strengthening or changing those policies.
- Reconcile prompt promises with actual enforcement. Citation identity and acknowledgment are not semantic-support checks; explicit declines are not independently justified by the application.
- Revisit evaluation scenario instructions, rubric applicability and comparison baselines for the fully integrated application. Preserve existing catalog versions; any changed case definitions require a separately versioned, authorized update.
- Re-run full compaction, memory, prompt and end-to-end evaluation checks after that integration. This RAG phase does not validate those later changes or eliminate historical semantic limitations.

## Completion checkpoint

Deliver the inherited RAG library, necessary Lesson 14 wiring, updated source-map/instrumentation documentation and an exact validation record. Clearly identify the editable prompts and broader compaction/memory integration as still deferred. Stop for instructor review; no commit, release approval or later integration is implied.
