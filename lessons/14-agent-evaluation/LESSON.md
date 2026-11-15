# Lesson 14 — Evaluate the complete agentic application

## Goal and supplied implementation

Judge the learning partner's behavior using observable evidence, not fluent prose alone. Establish a baseline, make one controlled compaction-policy change, and decide whether any improvement is worth its regressions or unresolved risks.

The complete reference source tree and [editable teaching content](content/) are supplied from the outset; use the linked source map below. The complete Lesson 14 tree is activated at root and recursively matches this reference; [November 14 validation](VALIDATION.md#learning-library-integration--2026-11-14) records checks, actual live outcomes and limitations separately from instructor review. Retrieval inherits Lesson 13's same-package RAG library; the November 14 continuation adds same-package learning support using actual Lesson 13 mechanics and newer policies, without editing Lesson 13 or changing the RAG API/call structure. The four editable teaching prompts remain this lesson's existing versions, with reconciliation deferred. These sources share the root manifest; this directory is **not** a separate Cargo package. Tracing transport, concurrency, tool adapters, storage isolation and runtime safeguards are supplied infrastructure, not missing scaffolding. The classroom rhythm remains **predict → run → explain → tweak → run again**.

This is evaluation of the **tutor application**, not assessment of a learner's trading competence. The instructor has books, not an expert-authored trading assessment rubric. Do not certify mastery, readiness to trade, production readiness or release approval. [VALIDATION.md](VALIDATION.md) records implementation evidence and gaps; commands below are instructions, not claims that those commands or all scenarios have passed.

## Concepts introduced — before opening code

**Diagnostics** record application state and activity so we can inspect what happened. The complete private Markdown diagnostic file remains useful even when trace export is disabled.

**Tracing** records related operations over time. A **trace** groups one observable execution; a **span** records one operation within it. A **parent/child relationship** connects work to the operation that caused it: for example, an inference or tool execution within a learner turn. It helps explain sequence, latency and capability use. It does not prove correctness. A missing tool span might mean no call, a refused call or incomplete export; inspect local evidence before deciding.

**Telemetry** is exported observational data. **OpenTelemetry** supplies a standard representation; **OTLP** is its transport protocol. Here native Rust exports traces to a local MLflow server, a tool for storing and inspecting traces and feedback. Exporting data is not evaluating it and does not turn the application into the full Goose CLI.

An **evaluation case** specifies inputs, initial state and criteria for acceptable behavior. An **evaluation set** groups cases covering different application risks. A **rubric** makes the inspection criteria explicit, including what counts as sufficient evidence and when judgment remains uncertain.

A **baseline** is recorded behavior under a named, fixed configuration before a change. A **comparison** evaluates another configuration under matched conditions. A **regression** is previously acceptable behavior becoming worse. A **regression gate** is a criterion that must remain acceptable before we adopt a change; it is not an average score that permits unauthorized writes to be offset by better prose.

A **deterministic invariant** is a boundary enforced and testable without relying on model wording: exact arithmetic, quotation provenance, correlation or consent, for example. A **semantic judgment** asks whether the words preserve meaning, support a claim or teach appropriately. It needs human inspection and may remain uncertain. Model agreement with a source is not proof that the source is correct.

**An evaluation dataset catalog** stores authored case definitions in MLflow. A **dataset record** holds the setup, learner inputs and inspection targets for one case, not a completed execution or model answer. **Seeding** registers these definitions so a human can find and manually link the intended case to later evidence. A manifest SHA-256 fingerprint identifies the exact catalog bytes; it does not grade a response or make the server immutable.

**Human feedback** records an evaluator's judgment on a trace, with a named criterion, value and explanation. Persisting that feedback proves storage worked, not that the judgment is expert or reproducible. An **evaluation run** is our recorded execution of a case; it is not automatically an MLflow training run or a whole conversational Session.

A **held-back case** is a case not used to choose the tweak. Inspecting it afterward helps reveal narrow tuning to the rehearsed examples. Model behavior varies; repeated live observations support limited claims, not universal guarantees.

**Grounding** supplies retrieved excerpts alongside a question so source-derived claims can be attributed. **Replacement evidence** is a new question's separate attachment replacing the previous one, not excerpts appended to history. A **pending reply** is the not-yet-accepted question plus that replacement evidence, counted prospectively without pretending it is stored history. A **citation registry** remembers source IDs and reference metadata prepared for the conversation; membership is structural, not proof of support or delivery. A **citation gate** checks that membership before presenting a buffered reply, not whether the answer is true.

## Supplied modules and reading path

A **module** is a named part of the same Rust application: it groups related code so we can read the current lesson's decisions without repeatedly reading already-taught mechanics. A `mod` declaration loads local code; `use` brings selected names into scope. A **library target** exposes reusable code from the same Cargo package. Here `src/lib.rs` exposes `rag` and `learning`, imported as `gdk_hello::rag` and `gdk_hello::learning`. This is not a workspace, separate retrieval package, new dependency or shared course framework. A **learning module** groups previously taught record storage, compaction mechanics, isolated summary inference and informational context sizing. A **binary adapter** connects reusable mechanics to this application's paths, IO and telemetry without transferring consent or approval authority. A **summary observer** is a supplied callback that records inference input/output; it neither approves a note nor commits a replacement. These boundaries keep the current lesson's application decisions visible. They are supplied, not fill-in exercises. In `main.rs`, for example:

```rust
mod calculator;
mod chooser;
mod diagnostics;
mod learning_record;
mod presentation;
mod provider_setup;
use gdk_hello::learning::context_size::ContextSize;
#[cfg(test)]
mod tests;
```

The same file declares binary-local `compaction` and `telemetry`; it imports retrieval/grounding/citation/transport support from `gdk_hello::rag` rather than declaring the old local `retrieval` module. Every Rust source starts with `// Lesson 14`. Read this complete source map in order, returning to main at each boundary:

| Reading order / source | What to inspect |
| --- | --- |
| 1. [main.rs](src/main.rs) | Context loading/prompt assembly, selection/reply lifecycle, consent, authoritative Session state, genuine-learner provenance and `/learned`/maintenance calls. Then `run_with_telemetry`, instrumentation flow, runtime/Effects, concurrent Event consumption, dispatch/allowlists/limits, machine assembly/run/cancellation/shutdown, automatic grounding, pending-input preflight, buffered citation gate/correction and successful-run inspection. Finish with proposal validation and the approval → decision log → append call → Session refresh sequence. |
| 2. [calculator.rs](src/calculator.rs) | Previously taught exact-decimal calculator schema, argument validation and execution; its local tests stay beside those mechanics. Main still decides dispatch. |
| 3. [learning_record.rs](src/learning_record.rs) | Thin binary adapter: isolated record path, confirmation IO and traced approved append. Library `learning/record.rs` supplies types/schema, bounded storage and declaration parsing; main retains consent, actual-learner-source validation and write authority. |
| 4. [lib.rs](src/lib.rs), [rag/mod.rs](src/rag/mod.rs), [rag/retrieval.rs](src/rag/retrieval.rs), [rag/grounding.rs](src/rag/grounding.rs), [rag/citations.rs](src/rag/citations.rs), [rag/transport.rs](src/rag/transport.rs) | Inherited same-package library: bounded read-only retrieval, profile/provenance/resource checks, per-turn selection, citation registry/lexical checks and local transport. Main supplies defaults, limits, grounding policy/contract and all acceptance, presentation and approval decisions. |
| 5. [compaction.rs](src/compaction.rs) | Application orchestration, budget/prospective pending reply/replacement evidence, revision checks, declaration authority and maintenance Operation. It calls library partition/candidate mechanics and `SummaryRunner` with telemetry observer callbacks; return to main for command routing, atomic genuine reply append and successful-run/approval boundaries. |
| 5a. [learning/mod.rs](src/learning/mod.rs), [learning/record.rs](src/learning/record.rs), [learning/compaction.rs](src/learning/compaction.rs), [learning/summary.rs](src/learning/summary.rs), [learning/context_size.rs](src/learning/context_size.rs) | Same-package learning mechanics: record schema/storage; pure history/provenance projection/partition/candidate/policy; isolated tool-free summary runtime and observers; informational context sizing. Callers keep paths, consent, approvals and replacement commits. |
| 6. [telemetry.rs](src/telemetry.rs) | Optional native transport, storage isolation helpers, ordinary observable projections and identity metadata. Return to main for initialization/application/shutdown ordering, `run_with_telemetry`, span instrumentation and authority decisions. |
| 7. [provider_setup.rs](src/provider_setup.rs) | Default/optional provider path and first configured model; local tutor restrictions remain visible in main. |
| 8. [chooser.rs](src/chooser.rs) | Supplied arrow/numbered selection, line input and raw-terminal cleanup; main owns the selection lifecycle. |
| 9. [presentation.rs](src/presentation.rs) | Terminal colors/gating and `TerminalProse`, with local tests. This is rendering support, not Event consumption or saved/provider-text rewriting. |
| 10. [diagnostics.rs](src/diagnostics.rs) | Complete plain Markdown diagnostics, isolated-path routing and local tests; diagnostics are never learner memory. |
| 11. [tests.rs](src/tests.rs) | Application orchestration/authority tests. |
| 12. [rag/tests.rs](src/rag/tests.rs), [rag/retrieval/tests.rs](src/rag/retrieval/tests.rs), [compaction/tests.rs](src/compaction/tests.rs), [telemetry/tests.rs](src/telemetry/tests.rs) | Library and module-local child tests, also `learning/{record,compaction,summary,context_size}/tests.rs`; keep all when activating the complete tree. |

The six inherited support modules are calculator, chooser, diagnostics, learning_record, presentation and provider_setup. Moving mechanics does not move authority: storage isolation helpers route both files, but main still shows the consent/approval/lifecycle decisions and their instrumentation. Resource cleanup and cancellation/Drop ordering are unchanged.

Telemetry's explicit source inventory covers `src/main.rs`, `src/lib.rs`, the six binary-local support files, all five production RAG files (`mod.rs`, `retrieval.rs`, `grounding.rs`, `citations.rs`, `transport.rs`), `src/compaction.rs` and `src/telemetry.rs`. It also hashes the manifest, lockfile, toolchain, rubric, scenarios and four editable prompts. The obsolete `src/retrieval.rs` path is replaced by library paths; the five production learning files (`mod.rs`, `record.rs`, `context_size.rs`, `compaction.rs`, `summary.rs`) are also inventoried. Test files are not inventoried. `course.content_sha256` remains a path-to-hash identity map, not exported source bytes or a correctness certificate. Collection/profile identity comes from the library's nonsecret interface, not duplicated profile literals. The standalone [dataset seeder](seed_dataset.rs) and [dataset assets](datasets/) are unchanged and separate from tutor execution.

### Inherited automatic grounding: application policy, not tool choice

Read `run_grounded_reply` → `prepare_turn_grounding` → `rag::prepare_grounding`, then return to main:

1. Prepare retrieval once per genuine non-command learner reply, not for the application kickoff or correction. The default similarity threshold is **0.55** (a valid `GROUNDING_MIN_SIMILARITY` override is allowed), selection takes at most **3** ranked excerpts, and the retrieval response uses a **2,500 tutor-token budget**. The authored contract asks for at most three grounded statements; a statement cap is not semantic enforcement. Similarity is provisional ranking, not a relevance or truth verdict. Empty selection is `no_evidence`; retrieval failure is `unavailable`, not invented evidence.
2. `TurnGrounding` stays outside `Conversation`. Its `source-grounding` prompt contribution is inserted first for each inference in the attempt. A new genuine reply replaces it, including clearing old evidence on no-evidence/unavailable outcomes. The library retains local transport restrictions, read-only profile/provenance checks, serialized search, tutor-tokenizer accounting and owned database-driver cancellation/Drop guards. The calculator and consent-enabled proposal tool remain available; no search tool is advertised, allowlisted or dispatched, and no old shared search-tool evidence ledger remains.
3. Preflight and any `maintain_with_pending` comparison include the **same pending question and replacement evidence** before/after candidate compaction. Summary input uses eligible stored history only, not the pending question or the separate excerpts. `append_grounded_user` checks the candidate before committing question, genuine message ID and evidence together; failure must not partially append them. Inference still has its own hard-budget preparer.
4. Continue consuming bounded Events concurrently with the GDK run, but render into a buffer. The provider streams to the application, **not live to the learner**. After a successful attempt, compare the final answer's extracted citation markers against current evidence and the conversation's `CitationRegistry` before releasing the buffer. A never-supplied ID withholds the first reply; silent-ignore text without invalid IDs can already have been displayed before the acknowledgment correction. Saved inference output is not proof that a learner saw it.
5. On a grounded turn, silent acknowledgment failure or an invalid ID can cause **one correction total**, reusing the same evidence with fresh per-attempt bounds. Before appending the application-authored reminder, settle first-attempt proposals through independent default-no approval. The reminder has no genuine learner ID and cannot supply quotation provenance; settlement keys combine kickoff ID and request ID so reused request IDs in a new attempt do not suppress decisions. The second outcome is reported, never retried indefinitely. A still-invalid corrected reply is shown **unverified with warnings**, not represented as a passed gate. Valid explicit `[source:none]` declines are reported without semantic adjudication or another attempt.

The registry resolves current IDs and earlier-turn IDs, with earlier references labeled in the footer; it is not a database lookup or proof that a passage supports a current claim. Registration occurs **before** pending-input acceptance, so registry membership does not prove accepted delivery. The inherited lexical extractor is not a complete strict-syntax validator; cite-plus-decline is warned about, not automatically rejected. On no-evidence turns the classifier returns `NotSupplied`, so a withheld fabricated-ID answer does not necessarily receive correction. These edge cases and unsuccessful-correction presentation are preserved for later review, not silently strengthened here. Excerpts can be irrelevant or inaccurate, legitimate-looking IDs can accompany unsupported prose, and an explicit decline's justification is not independently checked. See [RAG-INTEGRATION-PLAN.md](RAG-INTEGRATION-PLAN.md) for the deliberate parity/deferred-work boundary.

### Learning continuation: mechanics in the library, authority in the application

The November 14 continuation adopts the **actual** Lesson 13 newer policies: **84,000-byte default trigger**, **4,000-byte minimum eligible older history**, latest-two-exchange protection and the unchanged **84,000-byte hard input allowance**. Main constructs an explicit default policy and then applies the environment override:

```rust
let policy = compaction::Policy::from_env(compaction::Policy::default())?;
```

The binary keeps orchestration/budget/prospective/revision/declaration/Operation decisions. The library supplies pure projection/partition/candidate mechanics and an isolated tool-free summary runner. Pending question/replacement evidence is counted consistently before/after a candidate without storing it first; summary input is stored eligible history only. Main's memory/provenance/approvals/runtime and existing RAG call structure remain unchanged apart from optional runtime estimator wiring and explicit default-policy configuration.

`/learned` is explicit local authority to append its declaration **even when read consent was declined**, subject to existing duplicate/capacity/storage checks. It does not reload saved notes into a consent-declined Session. Command handling invokes maintenance unconditionally, although insufficient eligible history can yield a no-op. A failed summary does not undo an already-saved declaration or authorize an automatic second append. This is not mastery assessment. Model proposals still require successful-run inspection and independent default-no approval; no summary observer or library helper creates that authority.

Actual inherited proposal validation requires a bounded single-line note and an exact substring quotation from **any retained genuine learner reply**; it does not require `note == learner_quote` or latest-exchange-only evidence. Quotation occurrence is not semantic correctness or approval. Historical claims of stricter policies do not describe the actual Lesson 13 code reused here.

The thin binary record adapter retains isolated-path selection, confirmation IO and traced append. `SummaryRunner` accepts callbacks into binary telemetry without depending on that binary module. A complete-output Effect captures its originating chat span; on application after SDK message-ID assignment it re-enters that span for the inference-output hook, then invokes the saved-message hook. This preserves same-span output observation rather than incorrectly attributing it to the surrounding maintenance span. Existing cancellation/bounded-drain/Drop/partial-usage/failure boundaries remain supplied.

**Deferred and conflicting content:** `tutor-contract.md`, `getting-started-guidance.md`, `getting-started-material.md` and `compaction-guidance.md` remain unchanged/unreconciled. Their optional-search and consent-dependent memory wording may conflict with automatic grounding and always-append declarations. **Application authority wins**; these are known teaching limitations, not completed prompt reconciliation. Historical docs mention activities, continuity, source review and teaching modules absent from actual Lesson 13; none is implemented or claimed here. Scenarios, comparison worksheet and versioned catalog assumptions remain unreviewed for this architecture/policy. Do not silently rewrite/revalidate them or mutate the catalog. No prompt/corpus/pin/service/catalog/seeder change is part of this continuation.

### Activate the complete working copy

Root `src/` is an ephemeral exercise copy, currently synchronized with this complete reference. From the repository root replace it with **all** reference sources, including `lib.rs`, `rag/`, `learning/` and every child test directory:

```bash
rm -rf src
mkdir src
cp -R lessons/14-agent-evaluation/src/. src/
```

Do not copy only main or only top-level `.rs` files. No backup/restore is required for this exercise copy; preserve authoritative lesson references and unrelated work elsewhere. Finish with root matching Lesson 14. Do not reset it to hello world before the combined review.

## Prerequisites, services and exact commands

Complete Lesson 13's distinction between authoritative state, learner-authorized memory and lossy working context. Keep its local tutor, compatible Harrier/llama.cpp embedding runtime and read-only PostgreSQL setup. Use [Lesson 12 DATABASE-SETUP.md](../12-specialized-retrieval/DATABASE-SETUP.md); do not reload the provisional 511-passage collection, change models or prepare a corpus for this exercise. Rights to private text and disclosure remain operator responsibilities.

Follow [MLFLOW-SETUP.md](MLFLOW-SETUP.md) for the pinned official **MLflow v3.17.1** server, local experiment creation and numeric experiment ID. No Python evaluation runner or model judge is introduced. Server setup is supplied infrastructure, not a new learner programming assignment. Do not install an unofficial exporter or migrate the exact published alpha.11 GDK during class.

The **evaluation storage name** selects a disposable local directory so rehearsals do not reuse the everyday learner record or overwrite everyday diagnostics. Define it before launching. From the repository root, with the supplied Lesson 14 working copy:

```bash
umask 077
# Replace 1 with the numeric experiment ID from MLFLOW-SETUP.md.
export MLFLOW_EXPERIMENT_ID=1
export EVALUATION_STORAGE=baseline-V02
unset COMPACTION_TRIGGER_BYTES  # actual current default: 84000
cargo run -- custom_aa_llama_qwen3_6-35b.json
```

The three local classroom defaults are `const` values at the top of `src/main.rs`: `RETRIEVAL_DATABASE_URL` uses the instructor-supplied `retrieval_reader` connection to `127.0.0.1:55432/harrier`, `HARRIER_URL` is `http://127.0.0.1:9999`, and `TUTOR_URL` is `http://127.0.0.1:8080`. No exports or private connection file are required for these defaults. Existing environment variables (including values loaded by `dotenvy` from `.env`) override them. For another prepared database, set `RETRIEVAL_DATABASE_URL` to its reader connection, never the loader/admin connection. The supplied password is **plaintext in source and compiled binaries**; this is an explicit local classroom convenience, not secret storage or a production credential pattern. Rotate/replace it for any other deployment and do not project the credential-bearing source. Defaults do not start services or bypass runtime/profile/reader validation. `TUTOR_URL` selects the tokenizer origin and must match the tutor provider JSON; it does not override generation configuration.


Never project the private connection file, print environment credentials or enable shell tracing. The tutor origin must match the provider JSON and tokenizer service. Existing loopback/proxy/redirect/profile restrictions remain; client URLs do not repair a server listening on all interfaces. Ask the instructor to provide/start unavailable services. Do not silently substitute mocks for live evidence.

`MLFLOW_EXPERIMENT_ID` enables optional export and must be numeric. The supplied transport fixes the destination to `http://127.0.0.1:5000/v1/traces`; it is not a configurable hosted endpoint. Without the variable, the application still has local diagnostics but no enabled MLflow export. Initialization and flush/shutdown are supplied. Verify receipt in MLflow: enabling export alone is not proof of delivery or durable storage.

`EVALUATION_STORAGE=baseline-V02` resolves **both** files under the repository:

```text
.evaluation/sessions/baseline-V02/learner-record.md
.evaluation/sessions/baseline-V02/session-output.md
```

The record persists if locally authorized; diagnostics are overwritten at each launch. Changing only the experiment ID does **not** isolate files. Reusing a storage name does **not** create a clean baseline. Names accept 1–80 ASCII letters, digits, hyphens or underscores, not paths. Without the variable, normal root record/diagnostic paths apply. Isolated does not mean encrypted, transactional or protected against every filesystem race.

Run the standard checks from the root:

```bash
cargo fmt --check
cargo check --all-targets
cargo clippy --all-targets
cargo test --all-targets
cargo build
```

The inherited `gdk_hello::rag` library is now present for the calibration consumer as well as the tutor. The November 14 validation record reports the actual all-target results for this integration; commands alone are not evidence of a completed live scenario. Do not execute calibration or the catalog seeder merely to establish build compatibility.

The explicitly opt-in real-service tests are separate from ordinary tests. With the configured real services above:

```bash
cargo test live_local_smoke -- --ignored --nocapture
cargo test live_automatic_grounding_once_obeys_budget_provenance_and_no_search_advertisement -- --ignored --nocapture
```

A third opt-in test uses authored history and the **actual tutor**, not a mocked provider, to exercise summary application and protected-state assertions. It uses temporary diagnostics, never the everyday record. Set a fresh evaluation name; export is optional and, if enabled, must target an already prepared active experiment:

```bash
EVALUATION_STORAGE=learning14-summary-review-unique cargo test --bin gdk-hello \
  compaction::tests::live_learning_summary_preserves_protected_state_and_observation_boundaries \
  --locked -- --exact --ignored --nocapture
```

The library smoke test requires explicit reader/Harrier/tutor environment settings; obtain the existing classroom values locally without printing credentials. The summary test is real inference and can take minutes. A passed authored-history summary test is not a completed learner conversation, semantic-fidelity certificate or human evaluation. Normal all-target tests ignore all three live tests.


These integration checks are not a replacement for the conversational scenarios. In particular, the automatic-grounding service test uses a test selection policy; it is not a live semantic acceptance test of the default 0.55 threshold. Record actual commands/results, skipped cases and unavailable services in the validation/comparison evidence; do not infer live behavior from the deterministic suite.

## 14.0 — Catalog: register cases, not results

Follow [DATASETS.md](DATASETS.md) after local experiment creation. The supplied `seed_mlflow_dataset` Rust example registers the default [`datasets/visible-v1.json`](datasets/visible-v1.json) as native MLflow dataset **`lesson14-visible-v1`**: ten visible records V01–V10, with V04's separate conflict probe within the same record. From the root, replacing `1` with the actual active numeric experiment ID:

```bash
cargo run --example seed_mlflow_dataset -- 1
cargo run --example seed_mlflow_dataset -- 1 --check
```

The second command is read-only catalog verification, not execution/review. Inspect case setup/learner turns, inspection targets, optional arithmetic anchors and rubric/applicability/prerequisite tags. Start all Q judgments as **unreviewed**. V04 conflict and V10 remain **blocked** pending separate compatible fixtures and constrained selection; seeding does not install them. Held-back cases remain unchanged and unseeded. No private excerpts, expected tutor transcripts or trace imports belong in this manifest.

The seeder validates the active experiment and refuses duplicate names, a same-name/different manifest, edited or extra existing records. It skips exact records and can resume an exact interrupted subset. The dataset is tagged with manifest SHA-256 and schema version, but OSS supplies no native immutable versions: intentional changes need a new dataset name/manifest, and later external edits still require detection. See the dataset guide for the pinned, publicly undocumented native REST compatibility boundary.

Predict → seed/verify → inspect the **catalog**, then predict → manually run → inspect the **application**. These are separate outcomes. The utility supplies no runner, judge, automatic trace-to-record linkage or native evaluation-run orchestration; the tutor main/reference behavior is unchanged. Record the actual dataset/record IDs and fingerprint for later manual Notes/feedback linkage.

## 14.1 — Predict: decide what evidence would be enough

Read the supplied [rubric](RUBRIC.md), [scenarios](SCENARIOS.md) and [comparison worksheet](COMPARISON.md). Before interacting with the tutor, choose cases and repetition counts, write expected structure, and reserve a held-back case. Do not revise acceptance criteria afterward merely to make a pleasing answer pass.

Use three distinct layers:

| Layer | Question | Evidence and limit |
| --- | --- | --- |
| Deterministic boundary | Did the enforced invariant hold? | Tests plus actual state/tool/record evidence; cannot establish generated truth |
| Live observable behavior | What capabilities, evidence and replacements actually occurred? | Trace, local Sessions, returned passages, approvals and failures; missing export is not absence of work |
| Human semantic inspection | Was the answer supported, cautious and faithful to the learner? | Actual statements/passages beside claims, with pass/fail/uncertain and rationale; not expert mastery certification |

Predict a few concrete cases:

- A conceptual question without prices should not need the calculator. Inspect actual selection rather than forcing it.
- Entry `51.20`, stop `50.70`, `200` shares should yield **$100.00 maximum planned loss** through a correlated exact-decimal result. The tutor must retain fees/slippage/gap exclusions and not promise a realized-loss ceiling or statistical expected loss.
- A book-specific question should use relevant current evidence. Resolving a citation ID establishes membership, not support for its adjacent claim or correct book scope.
- After compaction, uncertainty remains unresolved unless the learner confirmed otherwise. A tutor explanation is not confirmation, and a suggested activity is not an agreement.
- Declining memory must prevent its use; rejecting a proposal must prevent its append. `/learned` is a local declaration, not mastery and not permission for other notes.

Maintenance cases for malformed/unknown/excess calls, provenance, failures and instruction-like source text belong in the supplied evaluation coverage. They are not invitations to inject faults into a classroom service or bypass safeguards. Follow the scenario instructions; do not invent unapproved failure-path exercises.

## 14.2 — Run: establish a baseline on fresh state

Use a fresh Session and matched memory conditions. For the simplest comparison, decline memory in both variants and keep disposable directories free of old records. If a case needs consented memory, prepare identical permissioned seed bytes **between launches**, record their identity privately, and make the same consent/approval choices. Do not allow one variant's approved additions to become the other's starting context. A new selection is a fresh Session, but the same on-disk record can still influence it.

Read the selected dataset record's setup and inputs alongside its scenario before launching; use a unique storage name for every condition/scenario/repetition. Execute the chosen scenario wording and local decisions. A catalog expectation never authorizes a memory write: inspect the actual proposed category, quotation and note after a successful run, retain default-no approval, and approve only an appropriate addition. If the intended branch does not occur, report it as unexercised rather than forcing it. Generated tutor responses will vary; do not replace an inconvenient response or count only the best attempt. Record the case/version, variant, repetition, storage name, Session/trace identities, UTC date, provider/model, corpus/profile, loaded guidance, trigger/counting policy, seed memory and decisions without credentials. Archive permissioned diagnostic evidence before another launch overwrites it. Follow the comparison worksheet for private evidence locations rather than checking in excerpts.

In MLflow inspect the `learner_exchange` parent, its automatic-grounding preparation and maintenance when applicable, then the `learner_turn` attempt records and available inference/tool children. Kickoff has no genuine-reply retrieval parent. Compare span inputs/outputs with the local correlated requests/results and final ordinary answer. Record observed token usage and timing when present; missing or partial usage is **unknown**, not zero. Summary work is extra inference, not a free reduction in cost. The trace is a debugging/evidence view, not a learner transcript or private chain-of-thought review exercise.

The established minimal infrastructure proof is narrow: official v3.17.1 accepted native Rust OTLP traces, inference/usage and parent/tool relationships were observed, and named numeric feedback entered in the native UI persisted. This does **not** establish that every application path exports completely or that every evaluation case passed. Consult [VALIDATION.md](VALIDATION.md) for the actual final checks and gaps.

## Current context estimate: completed exchanges only

The learning library now supplies `ContextSize`, following the actual Lesson 13 call pattern: display `Context: ~N tokens (estimate)` **at the end of each completed exchange, including welcome**, after its approval handling. Main loads current stored parts, active grounding, advertised tools and conversation; an optional runtime estimator produces the status. There is **no display at every local prompt**, blank/invalid command or `/learned` boundary, and **no token-estimate before/after-compaction pair**. Applied maintenance still has its distinct summary notice and conservative byte report.

The estimator uses pinned alpha.11 formatting with selected-model/thinking settings and retains the completed trailing assistant answer without counting a hypothetical next question. It counts serialized content UTF-8 bytes divided by four, rounded up; static instructions, loaded consented notes, local declarations, tools, history/latest answer and active evidence contribute. It excludes application-only metadata, cumulative usage and safety margins. It makes no tokenizer/service call, mutates no conversation and is not measured tokens, actual wire size, a guaranteed bound or a capacity percentage. Failure reports an unavailable estimate nonfatally. Smaller estimates do not prove summary fidelity or lower cumulative inference cost.

The dated discrepancy below describes the November 13 baseline, superseded by this explicit November 14 addition, not retroactively validated by historical display evidence.

## Current context-display discrepancy — 2026-11-13

> Historical baseline observation, superseded by the November 14 learning-library addition above. The dated text is retained, not a claim about the new current implementation.

The tracked Lesson 14 tree inspected for this integration contains **no `context_size.rs`, no context-size child tests and no prompt-boundary context-estimate calls**. Do not expect `Context: ~N tokens` or token-estimate before/after lines from this source. Current maintenance reports conservative **bytes** in diagnostics/telemetry and prints an applied-summary notice only after an applied replacement; no-op/failure/revision rejection must not claim replacement.

The [November 12 display record below](#cli-context-estimates--2026-11-12) and its [VALIDATION evidence](VALIDATION.md#cli-context-estimates--2026-11-12) are preserved as historical reports. They do not establish a display in the currently tracked tree. This documentation correction does **not** claim to remove a display; implementing/reconciling it is deferred. Neither conservative bytes nor historical token estimates prove summary fidelity or measured provider usage.

## 14.3 — Explain: attach judgments to evidence

Read the actual learner words, returned passages, tools, summary and final answer before rating. Separate an infrastructure block from an application failure and an uncertain semantic judgment. An eloquent answer with unsupported relationship wording is not a semantic pass. A controlled budget stop can satisfy the safety boundary while failing to complete the learning task; record both.

Apply the rubric's stable criterion names and numeric mapping when entering feedback. Add a short rationale identifying the claim and its supporting or contradicting evidence. Manually include actual dataset ID, record ID, case ID, manifest SHA-256, rubric version, comparison, reviewer and trace/span/private evidence locator in trace Notes and feedback rationale using [DATASETS.md](DATASETS.md). This text does not create automated record linkage or a native evaluation run. Preserve uncertain/not-applicable/blocked status in the worksheet; do not translate unavailable evidence into success or silently treat it as zero. Avoid an aggregate score that hides a consent or provenance violation.

The native MLflow UI supports **individual named feedback entries**. It does not provide this course's reusable ten-question form. The supplied rubric/worksheet supplies the repeatable checklist; feedback entries attach selected judgments to traces. Persisting one entry does not mean ten criteria were filled in, reviewed or automatically applied to later traces. Refresh/reopen to verify persistence and associate entries with the right case/variant. No Python judge/runner fills the rubric for you.

### Trace the exchange, not just a saved answer

Open the supplied code only after these concepts are clear. `run_with_telemetry` prepares isolated storage, initializes optional native export off the async worker and flushes/shuts it down on ordinary exit. Main retains these lifecycle decisions. Trace the boundaries in this order:

| Boundary | Observable evidence and limit |
| --- | --- |
| `learner_exchange` → `automatic_grounding` | A genuine reply's application-owned `RETRIEVER` preparation records query, threshold, status and selected excerpts. It is not a model-requested tool execution. Pending maintenance remains within the exchange. |
| `learner_turn` → attempts | `run_attempt` records attempt number, acknowledgment classification, invalid markers and ordinary answer. Attempt projections are attributes/events within the turn, not separately instrumented attempt spans. `withheld` and `not_yet_presented` explicitly differ from visibility. |
| Correction appended | `course.rag.correction` is emitted only after the application reminder was accepted, after first-attempt proposal settlement. No second retrieval occurs. |
| Learner-visible output | `course.rag.learner_visible_output` follows presentation and carries attempt/answer/warnings, including a warned final correction that still fails membership. A saved or inferred answer is not automatically learner-visible. Span output can be updated by later phases; retain the phase events and local diagnostics rather than reading only its last value. |
| Inference, tools, approval and summary | Native SDK inference observations, correlated calculator/proposal work, local approvals, `context_maintenance` and `summarize_context` remain distinct. Summary inference has its own cost and fallibility. |

`compaction::BudgetPreparer` composes conservative input checks, ordinary inference telemetry and `PromptCaptures`. Diagnostics drain each accepted prepared system prompt by attempt/request even on failure; preparation capture is not proof of provider delivery. The observational projection excludes provider reasoning/message metadata and does not deliberately export credentials or provider configuration. Ordinary dialogue, instructions, approved notes and selected excerpts can still be private. Structural output projections say `not_semantically_verified`; none grades educational correctness. Local full Session diagnostics and exported observations serve different purposes.

## 14.4 — Tweak: change only the compaction trigger

The **trigger** is when the application considers automatic compaction, not a model context-window measurement or the hard input allowance. The authored legacy controlled comparison is explicitly configured **60,000 → 50,000 estimated bytes**, not the current **84,000-byte default baseline**. The unchanged worksheet/scenarios/catalog have not been revalidated for this continuation. Record that applicability limit rather than calling old evidence a fresh default-baseline result. Keep the **84,000 estimated-input allowance** unchanged (100,000 total minus 16,000 reserved headroom), as well as all eligibility, provenance, approval, output and timeout safeguards. Word/token counts are not these byte estimates.

For that legacy comparison only, first set `COMPACTION_TRIGGER_BYTES=60000` explicitly with a fresh unused storage name (for example `legacy60000-V02`) and capture new evidence under the current source. Do not compare an unset 84,000 run with 50,000 and label it 60,000/50,000. After saving that evidence and ending its process, launch a fresh variant:

```bash
export EVALUATION_STORAGE=trigger50000-V02
export COMPACTION_TRIGGER_BYTES=50000
cargo run -- custom_aa_llama_qwen3_6-35b.json
```

Keep the same numeric experiment ID, service/model/profile, content, case wording, decisions and matched starting memory. The environment changes the supplied policy; no Rust surgery, hard-budget increase or source-guidance rewrite is part of this comparison. Do not change retrieval settings simultaneously.

Predict earlier eligibility, not guaranteed compaction on the next reply. More than two complete genuine learner exchanges and enough eligible older history must exist; a valid replacement must satisfy savings, reduction, revision and input checks. If neither run actually compacts, report that the comparison did not exercise compaction. Use the rehearsed scenario to obtain eligible history, not artificial overflow or silent removal of protected context. `/learned` can request maintenance under its existing semantics; do not confuse a command-triggered replacement with evidence about the automatic threshold.

After **actual** replacement, ask the same follow-up about the learner's unresolved question and agreed activity. Inspect both the generated summary and the answer. Check corrections, execution uncertainty, exact accepted scope, calculator qualifications, historical versus current citations and genuine quotation provenance. A good follow-up does not retroactively make a bad summary faithful.

## 14.5 — Run again: compare, gate and decide

Complete the comparison worksheet with observed counts, not universal percentages. Distinguish:

1. invariant violations, which block adoption;
2. live capability/evidence/continuation failures;
3. uncertain or unacceptable meaning/teaching judgments;
4. latency, usage and number of compaction attempts, including unknown values;
5. skipped, blocked or unexercised cases and held-back results.

An earlier trigger might prevent growth sooner but add inference cost or expose more turns to lossy summaries. A smaller input estimate alone is not success. Compare uncertainty and activity fidelity beside the original learner words; compare citations beside the actual returned passages. Repetition and controlled starting conditions reduce confounding, not nondeterminism. If the evidence is inconclusive, keep the baseline and state what further observation is needed.

Run `unset COMPACTION_TRIGGER_BYTES` to restore the current **84,000-byte default** afterward; 60,000 is only an explicit legacy-comparison condition. End the evaluation process before removing the evaluation variables or deliberately returning to ordinary storage. Preserve private artifacts only with appropriate permission; the setup guide supplies service shutdown steps. Do not delete someone else's records to clean a rehearsal.

## Privacy, inherited boundaries and known defects

Local MLflow storage, diagnostic archives and learner records may contain personal conversation and private book excerpts. Git ignore and loopback are not encryption or complete access control. Obtain consent, inspect/sanitize before projection, avoid credential/configuration/secret-URL/vector dumps, and do not upload private text to hosted judging services. Traces expose ordinary evidence; private reasoning is not an assessment target. The native trace exporter is not a guarantee that all third-party telemetry fields are harmless.

Retain GDK-owned iteration, strict schemas/allowlists/all-block handling/correlation, read-only retrieval, current-evidence citation checks, default-no consent and independent per-proposal approvals. Memory proposals acknowledge without writing; only appropriate local authority can append. Failed/truncated runs cannot authorize proposals or replacement. Compaction changes working context, not authoritative facts or persistent-note authority. Records are single-process appends with no transactional/crash/exactly-once or malicious concurrent-path-swap guarantee. Failure/cancellation preserves available partial evidence, not proof the server stopped. Terminal colors/Markdown suppression remain presentation-only; no new raw developer payloads enter learner dialogue.

Keep the inherited findings visible:

- Lesson 12 had source-scope/feedback relationship defects despite valid IDs, refusal without search, a loaded activity not offered in welcome, and a source run without a final ordinary answer.
- Lesson 13 recorded unsafe execution guarantees, tutor explanations falsely resolving uncertainty, and invented agreed-activity details. Targeted correction replays improved these cases, not all semantics. Later buy/sell limit, price-or-better and spread wording defects remain.
- Protected-input budget stops and rejected/truncated summary attempts remain evidence, not erased failures. No automatic mid-turn rescue is promised.
- The three-PDF 511-passage collection remains provisional; full layout/tokenizer/corpus freeze, full-source conversion, expert retrieval acceptance and cross-runtime parity are not certified.
- Earlier UI/teaching/content review and validation gaps remain independent, including Lesson 9's live 100-share tweak. Instructor review is deferred to end of course, not completed by trace receipt or feedback persistence.

These are regression scenarios and review obligations, not claims that Lesson 14 has fixed them. No network-isolation, comprehensive adversarial reliability or expert-quality benchmark is implied by a local smoke check.

## Success criteria and final course handoff

You can distinguish a seeded case catalog from execution and review; verify its identity, manually locate/link a record to actual trace evidence, and keep fixture-dependent branches blocked. You can distinguish tracing from evaluation; explain why structural tests and citation membership cannot certify meaning; find an actual inference/tool boundary; apply a rubric judgment with evidence; and keep uncertain/blocked cases visible. You have a named baseline, matched fresh-state variant, one unchanged hard-budget gate, actual compaction evidence when exercised, and an honest adoption or no-adoption decision.

Use [VALIDATION.md](VALIDATION.md) for the implementation's recorded checks, not an invented final test count here. The minimal trace/feedback proof is separate from full scenario coverage, semantic reliability, domain expertise and release review.

Exit questions:

1. Which defect could pass all arithmetic/schema tests and still harm learning?
2. What would distinguish no tool use from missing trace export?
3. Why does a rejected memory addition remain a blocking regression even if prose improves?
4. What starting-state differences would invalidate the 60,000/50,000 comparison?
5. What did your held-back case reveal, and what remains uncertain?

This completes the required course sequence with an **evaluation method**, not a trading platform or completion certificate. The next step is instructor end-of-course review and explicitly authorized follow-up on measured gaps. No simulator, general CLI, fine-tuning, deployment or SDK migration is an implicit requirement.

## RAG integration documentation handoff — 2026-11-13

This source-map/instrumentation update describes the inspected integrated source, not completed checks. Root activation is to finish on the full Lesson 14 tree for instructor review. Synchronization, full formatting/build/lint/test checks, real-service/CLI behavior and trace receipt still await the coordinating agent's evidence; use the **forthcoming dated RAG-integration entry in [VALIDATION.md](VALIDATION.md)** for actual results, blocked branches and semantic limitations. No counts, live correction success, completed prompt reconciliation or release approval are inferred here. Earlier dated records below remain evidence for their own source/configuration, not validation of this continuation.

## Modularization checkpoint — 2026-10-22

This is a behavior-preserving instructional reorganization, not a semantic fix or release approval. All 68 deterministic application tests and seven unchanged seeder tests passed with `cargo test --all-targets`; the two opt-in live tests remain ignored normally. Format/check/Clippy/build passed. Both live tests were explicitly attempted but failed because `RETRIEVAL_DATABASE_URL` was missing; the documented tutor launch stopped before the chooser for the same reason. The private connection file was absent, Harrier on 9999 and MLflow on 5000 were unavailable, while tutor health on 8080 returned 200. That health response is not end-to-end validation. See the [dated validation entry](VALIDATION.md#modularization-checkpoint--2026-10-22) for preservation audits, terminal error checks and exactly what remains blocked. No fresh conversational, trace-receipt or teaching success is claimed; historical findings remain intact.


### Service-default validation — 2026-10-28

Added the instructor-supplied reader connection and Harrier/tutor origins as constants at the top of main. Retrieval uses them only when the corresponding environment variable is absent; the main tokenizer-origin comparison uses the same tutor default. Invalid/non-Unicode retrieval environment values still fail instead of silently selecting a default. No process environment is rewritten, and no credential is deliberately added to diagnostics or telemetry. Provider JSON, compatibility checks, reader checks, dependencies and services are unchanged. The plaintext default is a classroom-only exception, not secure credential management.

Locked format/check/all-target Clippy/all-target tests/build passed in temporary staging for each complete reference: Lesson 12 **41**, Lesson 13 **67**, Lesson 14 **69** deterministic application tests, plus seven unchanged seeder tests each. One new test per lesson validates the reader host/role/database/port, local service origins and absent-variable fallback. The active Lesson 12 root independently passed the same checks and byte-matches its reference. Existing staged changes were preserved; no lesson activation replaced the user's root.

With all three override variables unset, both opt-in actual-service tests and the documented launch were attempted for each reference. All stopped with **`embedding runtime identity mismatch`** before the chooser: the existing Harrier service does not satisfy the pinned runtime identity check. This is blocked live validation, not a passed database connection, retrieval, tutor exchange, approval, compaction or telemetry run. No service restart, compatibility relaxation, corpus/catalog write or mock substitution occurred. No fresh IDE/projector/UI or teaching-quality validation is claimed; historical findings and review gates remain.


## Embedding server build flexibility — 2026-10-28

At the instructor's request, the learner application no longer requires an exact llama.cpp build string. A different or absent `/props` `build_info` is accepted. Model alias, F16 format, GGUF checksum, dimensions, context capacity, pooling, normalized vectors, query formatting, stored collection/profile identity, provenance and reader restrictions remain checked. The stored profile's build suffix records the original embedding preparation; it is not a requirement that the current query server report that build. Accepting another build does not establish numerical equivalence or retrieval-quality parity. The standalone preparation loader/client retains its separate pinned-build policy. No service restart, re-embedding or collection mutation is needed solely to pass the removed build check.

Validation for this change: formatting, locked check, all-target Clippy, all-target tests and build passed (69 deterministic application tests plus seven unchanged seeder tests). Both opt-in actual-service retrieval tests passed against the currently running services, including server build `b11669-620d2225c`; the documented launch reached the chooser and exited on EOF. Lesson 12 was checked in the active root; Lessons 13–14 were checked in temporary staging without replacing root. Regression coverage accepts changed/absent build metadata and still rejects incorrect model alias/format, sleeping/no-slot runtime and mismatched context. No full tutor conversation, approval/compaction/telemetry workflow, numerical cross-build parity benchmark, fresh UI/IDE/projector or teaching-quality review was performed. No services restarted, corpus/catalog modified, preparation client changed or dependencies updated. Existing historical defects/review gates remain.
## Learner-led topic scope — 2026-10-28

The four supplied teaching units are optional background examples, not a topic whitelist or required progression. Learners may ask open-ended day-trading questions outside those examples and choose the depth of discussion. Offer topic suggestions only when asked or when the learner wants help choosing a direction. General educational discussion of broker-comparison criteria is allowed; personalized trading recommendations and unsupported claims about current fees, regulations or broker terms are not. Limited source coverage is an evidence limitation, not a ban on the topic.

The three editable content files retain this lesson's own capability and authority instructions; do not replace later-lesson contracts with Lesson 10's calculator-only contract. Content changes take effect on a fresh selection, not in an already-running selected conversation. Historical teaching findings remain evidence about earlier content, not proof that this revision fixes model behavior.

Validation for this content-only revision: static scope/capability/original-section checks passed across Lessons 10–14. The active Lesson 12 context-loading test and all-target tests passed (41 application tests and seven unchanged dataset-seeder tests; two opt-in real-service tests were not run). No new live teaching, approval, retrieval, compaction or telemetry validation is claimed.


## CLI context estimates — 2026-11-12

The approved display is supplied in this reference's existing binary-local layout, without migrating it to Lesson 13's library or changing prompts, byte guards, approval/provenance or tool behavior. Warning-free format/check/all-target Clippy/tests/build passed in isolated staging: **85 deterministic application tests** (16 new), two actual-service tests normally ignored, plus seven unchanged seeder tests with no catalog calls. Both actual-service tests passed explicitly. Staging omits the Lesson 13-only calibration example; use the binary-target maintenance commands above after activating this reference.

An isolated live rehearsal completed exit 0 in **211.2 seconds**, showing **13 correctly ordered estimate/prompt pairs** and applied stored-context reductions **~12,617 → ~11,153** and **~12,096 → ~10,662 tokens**. Their separate conservative byte measurements were **70,329 → 59,790** and **69,501 → 57,826**. Blank input, invalid `/learned`, ephemeral declaration, no-op maintenance and same-Session continuation were checked. A separate `TERM=dumb` run returned correlated **$100.00** with exclusions, and a colored-arrow PTY run showed five bold-cyan status/prompt pairs with independently plain redirected stderr. Plain/diagnostic output had no application-added ANSI. Memory was declined; no actual learner record was accessed.

The old MLflow experiment 1 returned 404 despite health/version readiness, so tracing was left disabled: no fresh export/identity-receipt, catalog mutation, experiment creation or review evidence is claimed. See [the detailed validation entry](VALIDATION.md#cli-context-estimates--2026-11-12) for initial test-fixture corrections and remaining semantic findings. No fresh live approved/rejected-note save, formatter failure, rejected replacement, colored actual compaction, held-back/rubric/teaching/IDE/projector review, service restart, corpus change, pin/dependency change or release approval. Historic findings remain; estimates are not measured tokens or summary-fidelity proof.

## RAG-only integration checkpoint — 2026-11-13

The complete reference is now activated at root and recursively synchronized. Final checks passed121 deterministic application tests plus7 unchanged seeder tests; both explicitly enabled actual-service tests passed. Isolated live grounding/reference/calculator and independent note decisions completed; one acknowledgment correction and actual MLflow trace readback were observed. These are structural observations, not semantic or prompt-integration approval. The four editable prompts and broader compaction/memory/display work remain deferred; source-scope/execution-wording findings and deterministic-only branches remain visible in [the exact validation record](VALIDATION.md#rag-only-library-integration--2026-11-13). Stop for instructor review; no release approval.


## Learning-library authorization/current architecture — 2026-11-14

This entry supersedes the November 13 RAG-only deferral for the authorized learning continuation only. Same-package record/compaction/summary/context-size mechanics now follow actual Lesson 13 patterns and newer policies; ALL Lesson 13 code and existing RAG API/call structure remain untouched. The five learning production files join the explicit telemetry hash inventory without changing its schema. Binary orchestration, local authority and same-span summary observers are described above; no activity/continuity/source-review/teaching modules or prompt reconciliation is claimed.

**Root/reference recursively match.** Final warning-free locked format/check/all-target Clippy/tests/build passed: **186 deterministic application tests (91 library + 95 binary), seven unchanged seeder tests**, with **three opt-in real-service tests separately passed**. Isolated live notes exercised inspected approval/new0600, rejection preserving bytes, and declaration/no-op maintenance. Declined consent separately preserved an unread fixture while saving a declaration. Completed-exchange sizing and startup output modes were checked. A real-provider authored-history summary applied **64,947→48,843 conservative bytes**, preserving protected state; native readback verified all29 hashes and output on the summary chat span. CLI timeouts/incomplete branches and summary attribution findings remain explicit in [VALIDATION.md](VALIDATION.md#learning-library-integration--2026-11-14); this is not a continuing-conversation compaction/follow-up, semantic-quality or matched-comparison pass. Do not reuse older results as validation of these new bytes. Four editable prompts and versioned case/comparison assumptions remain unreconciled; keep old limitations and all dated history. No services restarted, corpus/catalog/seeder/pins/prompts changed or release approval granted by this documentation handoff.
