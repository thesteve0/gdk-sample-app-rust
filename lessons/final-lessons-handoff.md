# Final three lessons — implementation handoff

## Decision and authorization — 2026-10-11

The course ends with **Lesson 12: specialized retrieval, Lesson 13: context compaction, and Lesson 14: evaluation**. The instructor approved this curriculum and requested planning documentation only. **Historical planning-only decision:** each needed a separate implementation request; Lesson 12 was subsequently authorized and implemented on 2026-10-12. Lesson 13 was subsequently authorized on 2026-10-12; Lesson 14 was subsequently authorized and implemented on 2026-10-13; planning is not authorization to change root source, dependencies, configuration, or infrastructure.

Day trading is the teaching vehicle, not the selection criterion for agent architecture. Each lesson must explain a transferable application-building concept. Historical simulation is a separate documentation-only possibility at [potential-lesson-15/](../potential-lesson-15/README.md), not a fourth required lesson and not combined with compaction. The former Lessons 12–18 roadmap is superseded; a full CLI, explicit model-selection lesson, ingestion pipeline, live market data, and fine-tuning are not promised course outcomes.

## Subsequent preparation authorization — 2026-10-11

The instructor separately authorized downloads/conversions/container pull, followed by **PDF ingestion and draft chunking only**. Public Rust preparation code now lives in [embedding-code/](../embedding-code/README.md), with its own manifest/lockfile; private artifacts remain in ignored `day_trading/embeddings/`. Root and its dependencies remain Lesson 11. Real extraction of the three initial PDFs produced 407 pages and 2,405 draft chunks; layout acceptance, tokenizer checks and corpus freezing remain open. No embedding calls, PostgreSQL work, services or learner Lesson 12 integration are authorized in this phase. Model preparation still has the reported unsupported Nemotron GGUF conversion blocker; download/conversion does not establish runtime fidelity. Historical planning-only statements below describe the original handoff, not a prohibition on this newly authorized preparation subset.

## Start here in a fresh implementation chat

1. Read root README and AGENTS, inspect Git status and diffs, and preserve staged/unstaged work. At this handoff Lesson 10's `TEMP-issues.txt` has pre-existing staged and unstaged changes; do not resolve it incidentally.
2. Read this handoff and the active lesson's PLAN, then root `src/main.rs`, Lesson 11's complete reference, lesson, instructor notes, and editable teaching content. Root now runs Lesson 14; Lesson 11 remains the preserved predecessor. Read all active source modules and Lesson 13 materials, plus the preserved Lesson 12 predecessor. Read the preceding final lesson's reference once it exists.
3. Read Cargo.toml, Cargo.lock, rust-toolchain.toml, and the selected provider JSON. Current pins: Rust 1.94.1; goose-agent/goose-providers 0.1.0-alpha.11; RMCP 3.4.1; crossterm 0.29.0. This planning work changes none of them. New database/embedding dependencies require documented choices and Cargo-managed lockfile updates during authorized implementation, not a GDK upgrade.
4. Read relevant official Goose docs and exact pinned GDK/RMCP sources before making API claims. Verify external PostgreSQL/pgvector and embedding interfaces against their selected versions' official documentation. Do not invent provider embedding support, database schema, credentials, dimensions, or context capacity.
5. Settle the active PLAN's prerequisite decisions with the instructor. If a required service or corpus is unavailable, ask for it. Deterministic tests are useful but do not substitute for actual datastore/provider integration.
6. Implement only the authorized lesson, including complete reference code from the outset, prose, instructor notes, supplied support machinery, tests, and a documented shared-root workflow. Confirm root replacement as part of that authorization. Do not copy root into a lesson as an end-of-class solution.
7. Run the repository's relevant formatting/build/lint/test checks and documented live run; record exact results and remaining gaps. Review is deferred until the course is fully implemented, not a blocker for separately authorized work and not release approval.

## Plans and intended paths

| Unit | Planning document | Later implementation deliverables |
| --- | --- | --- |
| 12 | [12-specialized-retrieval/PLAN.md](12-specialized-retrieval/PLAN.md) | LESSON.md, complete src/main.rs, supplied context/config examples without secrets, instructor-notes/12-specialized-retrieval.md |
| 13 | [13-context-compaction/PLAN.md](13-context-compaction/PLAN.md) | LESSON.md, complete src/main.rs, authored compaction policy/prompt, explanatory visual, instructor-notes/13-context-compaction.md |
| 14 | [14-agent-evaluation/PLAN.md](14-agent-evaluation/PLAN.md) | LESSON.md, complete src/main.rs, redistributable evaluation cases/report workflow, instructor-notes/14-agent-evaluation.md |

Lesson 14 now supplies the complete native Rust application/reference, local MLflow container setup, editable ten-question rubric, ten visible scenarios, three held-back variants, comparison template and instructor materials. Read its [actual validation record](14-agent-evaluation/VALIDATION.md); this is implementation, not release approval. Synthetic retrieval/adversarial branch readiness and earlier quality defects remain explicit gaps. Earlier planning entries below remain historical.

## Corpus reality: books, not expert rubrics

The instructor owns day-trading EPUBs and PDFs but does not claim domain expertise and has no instructor-authored assessment rubrics. Do not assume expert review, certification, or a prepared rubric collection. Books may offer specialized explanations, worked examples, disagreements, and exercises that the model can use as evidence. They are not automatically authoritative, current, or correct.

The datastore and extracted/embedded data are intended to be supplied before learners attend; preparation remains an instructor-side prerequisite, not a learner ingestion exercise. The instructor subsequently approved the [detailed Lesson 12 plan](12-specialized-retrieval/PLAN.md): local-only tutor and llama.cpp embedding services, PostgreSQL 18/pgvector 0.8.7, native float16 vectors with HNSW/cosine distance, paragraph/section-first chunks, and a three-model real-PDF comparison (EmbeddingGemma 2, Harrier 0.6B, Nemotron 1B). Rust preparation will live in `embedding-code/`; private text/embeddings/exports in ignored `day_trading/embeddings/`, with public code/DDL/synthetic tests. All PDFs/EPUBs are included eventually, AZW3 excluded. Local extraction is permitted; no external inference or remote fallback. Exact schema, runtime/conversion fidelity, winning model, legal learner access arrangements and actual service/data readiness remain implementation gates. This is documentation authorization only; root remains Lesson 11 and no infrastructure or source is supplied yet.

Use book-grounded feedback to compare a specific learner answer with cited passages. Say what evidence is present or missing; ask a follow-up when support is insufficient. Do not claim retrieval changes model weights, guarantees novel knowledge, creates a validated assessment, or proves readiness to trade. Any model-derived criteria must be labeled tentative and source-grounded, not instructor-authored rubrics. Lesson 14 evaluates visible support and application behavior, not specialist correctness the instructor cannot certify.

Private source books, extracts, embeddings, database exports, and personal conversations must not be committed. Book ownership is not blanket permission to redistribute extracts or send them to an external service. Decide permitted processing and provider disclosure before use. Supply a small redistributable/course-authored integration test corpus with honest provenance; do not silently replace unavailable book data and call the real integration validated.

## Preserve the application foundation

- Keep GDK-owned within-run iteration; do not restore a manual provider/tool loop. Operations, Effects, and streamed Events retain distinct roles.
- Preserve exact-decimal calculator validation, strict schemas, tool allowlists, all-content-block handling, correlation, unknown/excess errors, and independent learner approval for record writes.
- Current per-turn limits are four aggregate tool requests, eight applied work passes plus possible halt batch, and 180 seconds. Additional retrieval/compaction work must fit explicit budgets or receive a justified, documented adjustment. No silently unbounded second inference path.
- Keep memory consent per selection, default-no per-addition approval, hand-editable fixed-path record, and no mastery inference. Compaction is not persistent memory approval.
- Preserve simple Tutor/Learner UI, course palette/terminal gating, and presentation-only styling. No protocol fences, arrows, IDs, raw arguments, retrieved-passage dumps, or Session dumps in ordinary learner dialogue. Citations must still be readable; account for TerminalProse's limitations.
- Keep complete plain overwritten `session-output.md` diagnostics, loaded context, tools/results, decisions, failures, and available partial state; never read diagnostics as memory. Include new retrieval/compaction data without credentials, database URLs containing passwords, or embeddings dumps. Diagnostics may contain private book excerpts and learner text.
- No brokerage, real order execution, shell, arbitrary SQL, arbitrary file access, or generic network tools. Retrieval is a narrow read-only application capability.
- Supply concurrency, adapters, storage, and presentation helpers; teach predict → run → explain → tweak → run again, not missing Rust scaffolding.

## Source verification and important integration traps

Official [Goose smart context management](https://goose-docs.ai/docs/guides/sessions/smart-context-management) describes full-application compaction, not automatic behavior of this pinned SDK. Verified locally in goose-agent 0.1.0-alpha.11: `Operation` can contribute prompt parts and tools; `StateMachine` gathers them before inference, applies the first applicable step, and reloads state; `ConversationEffect` includes ReplaceConversation and visibility changes. The custom runtime must implement equivalent effects if it uses its own effect enum. There is no supplied application-specific compaction policy in this crate.

**Lesson 11 compatibility trap:** proposal validation currently searches conversation user messages with `.skip(1)` to exclude the application opening. A generated summary must never become an apparent actual learner quotation. Compaction can also remove the kickoff used by `messages_since_kickoff`, damage request/response pairing, or repeat prior approval handling. Lesson 13 must redesign source provenance as needed and test these boundaries explicitly; do not preserve the old logic blindly.

The default provider JSON currently has no declared context_limit. Do not infer an exact token capacity from the model name or run until the provider rejects input. A verified capacity or explicit conservative application budget, with an honest counting/estimation method, is a Lesson 13 prerequisite.

## Existing validation and review status

Lesson 11 root/reference had 26 tests plus formatting/check/Clippy and recorded live memory/calculator successes. An initial proposal follow-up timed out; partial diagnostics survived and no approval/write occurred. Its expanded teaching guidance has not received a fresh live teaching evaluation, and colored arrow UI was not revalidated. Lessons 8–10 review, Lesson 9's live 100-share tweak, and Lesson 10's inconclusive guidance experiment remain independently pending. End-of-course review is deferred, not completed. Do not overwrite historical evidence with new planning claims.


### Reviewed PDF margin cleanup - 2026-10-12

The instructor rejected beginners-guide cover/branding/margin chunks. `embedding-code/src/cleanup.rs` now uses a reviewed source-checksum-bound rule, not generic repetition removal: cover page 1, exact first-line brand headers and matching last-line footers on pages 2-18. Raw pages remain intact; cleaned pages preserve line numbers and excluded-lines JSONL records every removal. Beginners chunks decreased 92 to 56 (38 lines excluded); other books are unchanged, total 2,369 draft chunks over 407 pages. New run and preview instructions are in `embedding-code/README.md`. Fourteen preparation tests plus formatting/check/Clippy and real artifact/provenance checks passed. Full layout acceptance, other-book cleanup, tokenizer/freeze gates and embedding calls remain deferred; no root or dependency changes.


## Chunk-policy comparison decision — 2026-10-12

Instructor agreed to compare **one nonempty physical page plus 10% preceding-page word-tail overlap** against **section-aware paragraph groups targeting roughly 300–500 words**. Whole major sections/subsections are too variable for direct chunking; they remain diagnostic reports only. Review inferred headings and remaining Heitkoetter margins first. Page overlap uses ceil(10% of previous physical page base words), no recursive overlap, with empty pages resetting carry-over; structural overlap for section-aware groups must be explicitly settled/versioned within section boundaries. Word targets are not model-token limits. Use the same reviewed text/exclusions and source-span relevance questions; compare coverage/boundaries, size/overlap distributions, retrieval/evidence completeness and cost under fixed model settings and equal returned-token budgets. Freeze policy-specific collections and a common collection per policy across candidate models, then choose policy/model from evidence. See Section 6 and Section 9 of `lessons/12-specialized-retrieval/PLAN.md`. This request updates documentation only: current 511 draft chunks, Rust/Python code, root application and dependencies remain unchanged; no embedding calls, corpus freeze or database work authorized by this decision.


## Harrier selection and test datastore authorization — 2026-10-12

The instructor selected **Microsoft Harrier OSS v1 0.6B** (the downloaded variant) and explicitly authorized embedding the saved test set, local PostgreSQL loading, Docker-compatible startup/DDL/SQL and working query code. This supersedes the earlier preparation-only prohibition on embeddings/services/database work, but **does not authorize learner Lesson 12 integration or root replacement**. See [the preparation workflow](../embedding-code/RETRIEVAL.md). The test collection uses the current 511 draft chunks and preserves review flags/provenance; this is not a freeze or implementation of either alternative chunk policy. Final layout/section review, full-corpus expansion, legal distribution and retrieval-quality evaluation remain open.

For the initial test load (superseded by the llama.cpp switch below), the supplied local reference runtime used the pinned official Harrier checkpoint with publisher-specified last-token pooling, L2 normalization and web-search query instructions. It does not claim converted-GGUF fidelity. The earlier llama.cpp deployment choice remains a learner-integration compatibility gate, not a reason to silently mix embedding profiles. PostgreSQL stores `halfvec(1024)` with cosine HNSW, collection/model/profile identity and a separate read-only role. Other-model comparison is no longer a prerequisite to this selected-Harrier test; historical comparison plans above are preserved as evidence.


### Harrier llama.cpp runtime switch — 2026-10-12

The instructor explicitly requested the existing **llama.cpp service on port 9999** instead of the Python reference runtime. The preparation Rust client now targets its OpenAI `/v1/embeddings` API, applies the query instruction in Rust and preflights tokenizer/context counts. The active collection is **`harrier-llamacpp-test-511-v1`**, re-embedded with the pinned F16 GGUF/runtime profile; the original `harrier-test-511-v1` Transformers collection is preserved and must not be mixed. Follow `embedding-code/RETRIEVAL.md` and Lesson 12 `DATABASE-SETUP.md` for current startup/load/query instructions. This changes only preparation, not root Lesson 11 or learner Lesson 12 integration. The existing user's server is bound to 0.0.0.0; instructions use 127.0.0.1, but no automatic restart occurs. Cross-runtime parity and final retrieval quality are not claimed.


## Superseding Lesson 12 implementation authorization — 2026-10-12

The instructor explicitly authorized implementing Lesson 12, its complete lesson/instructor materials, and root replacement after confirming the tutor service was running. Root now advances from the preserved Lesson 11 reference to Lesson 12. Use the existing provisional three-PDF `harrier-llamacpp-test-511-v1` collection; no final corpus freeze, full-source conversion, runtime parity claim, or Lesson 13–14 implementation is implied. The complete Lesson 12 and its validation record supersede earlier learner-integration restrictions only for this scope. Keep all earlier references and unrelated preparation work intact. Private-data access/distribution rights remain the operator's responsibility; do not distribute the books or excerpts with the public course.


### Lesson 12 integrated validation — 2026-10-12

Root/reference are supplied and synchronized. Format/check/Clippy/IDE build,40 deterministic tests and2 explicitly enabled real-service tests passed. Live source search/citations and correlated$100 calculator, memory approved/rejected additions, plain/NO_COLOR/dumb and colored arrow UI retry passed. Keep teaching defects visible: paraphrased/feedback answers confused source scope or relationship wording despite valid IDs; unsupported-price refusal made no search; consented hand-edited memory loaded but welcome did not offer its activity. Initial colored search had no final ordinary answer; partial diagnostics survived and no approval occurred. Reader PUBLIC TEMP was revoked and startup now preserves that restriction. No corpus reload/freeze, server restart, parity/expert benchmark or offline network-isolation test. See Lesson12's detailed validation record; instructor review remains pending.


## Superseding Lesson 13 authorization — 2026-10-12

The instructor authorized Lesson 13 application implementation, lesson and instructor documentation, continuing the same root learning partner. Preserve Lesson 12 references and all earlier validation gaps. Root now uses Lesson 13 content and the published exact alpha.11 registry dependencies again; the proposed local-checkout SDK migration is explicitly deferred until the class is finished and verified. No toolchain/provider/preparation upgrade or Lesson 14 implementation is implied. Read Lesson 13's LESSON.md and validation record before continuation. `/learned` is local learner-declared completion, never assessed mastery; consent controls persistence and compaction summaries never authorize writes or new citations.


## Lesson 14 current handoff — 2026-10-13

Root is the separately authorized Lesson14 continuation. Read its full LESSON and VALIDATION before changes. Full Goose CLI observability integration does not initialize this native GDK application. Supplied telemetry reuses pinned SDK inference spans and adds application spans, optional local export, shutdown flush and disposable memory/diagnostics selection. Official MLflow minimal v3.17.1 digest tested; numeric human feedback UI and persistent SQLite/artifacts validated. Rubric is editable prose beside individual native feedback, not custom annotation/scoring infrastructure. No custom Python runner or automatic/hosted judge. Keep previous evidence, incomplete comparisons, semantic defects, blocked fixtures and instructor review pending. Potential Lesson15 remains documentation-only, no SDK upgrade or release implied.
