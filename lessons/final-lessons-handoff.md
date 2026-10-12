# Final three lessons — implementation handoff

## Decision and authorization — 2026-10-11

The course ends with **Lesson 12: specialized retrieval, Lesson 13: context compaction, and Lesson 14: evaluation**. The instructor approved this curriculum and requested planning documentation only. **Do not implement these lessons yet.** Each needs a separate implementation request; planning is not authorization to change root source, dependencies, configuration, or infrastructure.

Day trading is the teaching vehicle, not the selection criterion for agent architecture. Each lesson must explain a transferable application-building concept. Historical simulation is a separate documentation-only possibility at [potential-lesson-15/](../potential-lesson-15/README.md), not a fourth required lesson and not combined with compaction. The former Lessons 12–18 roadmap is superseded; a full CLI, explicit model-selection lesson, ingestion pipeline, live market data, and fine-tuning are not promised course outcomes.

## Start here in a fresh implementation chat

1. Read root README and AGENTS, inspect Git status and diffs, and preserve staged/unstaged work. At this handoff Lesson 10's `TEMP-issues.txt` has pre-existing staged and unstaged changes; do not resolve it incidentally.
2. Read this handoff and the active lesson's PLAN, then root `src/main.rs`, Lesson 11's complete reference, lesson, instructor notes, and editable teaching content. Root remains the Lesson 11 learning partner. Read the preceding final lesson's reference once it exists.
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

These folders contain **plans, not implemented lessons**. They deliberately have no source, runnable commands, invented output, or new manifest. The exact lesson runtime/evaluation command will be written and verified during implementation.

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
