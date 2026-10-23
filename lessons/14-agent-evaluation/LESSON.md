# Lesson 14 — Evaluate the complete agentic application

## Goal and supplied implementation

Judge the learning partner's behavior using observable evidence, not fluent prose alone. Establish a baseline, make one controlled compaction-policy change, and decide whether any improvement is worth its regressions or unresolved risks.

The complete reference source tree and [editable teaching content](content/) are supplied from the outset; use the linked source map below. Root and reference sources are synchronized; the teaching content continues Lesson 13. These sources share the root manifest; this directory is **not** a separate Cargo package. Tracing transport, concurrency, tool adapters, storage isolation and runtime safeguards are supplied infrastructure, not missing scaffolding. The classroom rhythm remains **predict → run → explain → tweak → run again**.

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

## Supplied modules and reading path

A **module** is a named part of the same Rust application: it groups related code so we can read the current lesson's decisions without repeatedly reading already-taught mechanics. A `mod` declaration loads that local code; `use` brings selected names into scope. These are local source files, not separate applications, dependencies or a shared course framework. They are supplied, not fill-in exercises. In `main.rs`, for example:

```rust
mod calculator;
mod chooser;
mod diagnostics;
mod learning_record;
mod presentation;
mod provider_setup;
#[cfg(test)]
mod tests;
```

The same file also declares the cohesive `compaction`, `retrieval` and `telemetry` modules. Every Rust source starts with `// Lesson 14`. Read this complete source map in order, returning to main at each boundary:

| Reading order / source | What to inspect |
| --- | --- |
| 1. [main.rs](src/main.rs) | Context loading/prompt assembly, selection/reply lifecycle, consent, authoritative Session state, genuine-learner provenance and `/learned`/maintenance calls. Then `run_with_telemetry`, instrumentation flow, runtime/Effects, concurrent Event consumption, dispatch/allowlists/limits, machine assembly/run/cancellation/shutdown, current-turn evidence/citation inspection and successful-run gate. Finish with proposal validation and the approval → decision log → append call → Session refresh sequence. |
| 2. [calculator.rs](src/calculator.rs) | Previously taught exact-decimal calculator schema, argument validation and execution; its local tests stay beside those mechanics. Main still decides dispatch. |
| 3. [learning_record.rs](src/learning_record.rs) | Record types/schema, bounded file access, confirmation-input mechanics and append implementation, with local tests. Main retains consent, actual-learner-source validation and write authority. |
| 4. [retrieval.rs](src/retrieval.rs) | Cohesive bounded read-only retrieval, runtime/profile/provenance checks and resource guards. Return to main for registration, shared evidence budget and current-turn citation inspection. |
| 5. [compaction.rs](src/compaction.rs) | Cohesive authoritative declaration/summary/maintenance algorithms, protected history/provenance, revision and budget checks. Return to main for command routing, preflight, genuine reply append, successful-run/approval and maintenance boundaries. |
| 6. [telemetry.rs](src/telemetry.rs) | Optional native transport, storage isolation helpers, ordinary observable projections and identity metadata. Return to main for initialization/application/shutdown ordering, `run_with_telemetry`, span instrumentation and authority decisions. |
| 7. [provider_setup.rs](src/provider_setup.rs) | Default/optional provider path and first configured model; local tutor restrictions remain visible in main. |
| 8. [chooser.rs](src/chooser.rs) | Supplied arrow/numbered selection, line input and raw-terminal cleanup; main owns the selection lifecycle. |
| 9. [presentation.rs](src/presentation.rs) | Terminal colors/gating and `TerminalProse`, with local tests. This is rendering support, not Event consumption or saved/provider-text rewriting. |
| 10. [diagnostics.rs](src/diagnostics.rs) | Complete plain Markdown diagnostics, isolated-path routing and local tests; diagnostics are never learner memory. |
| 11. [tests.rs](src/tests.rs) | Application orchestration/authority tests. |
| 12. [retrieval/tests.rs](src/retrieval/tests.rs), [compaction/tests.rs](src/compaction/tests.rs), [telemetry/tests.rs](src/telemetry/tests.rs) | Module-local child tests retain access to their parent's implementation without splitting cohesive production algorithms. |

The six inherited support modules are calculator, chooser, diagnostics, learning_record, presentation and provider_setup. Moving mechanics does not move authority: storage isolation helpers route both files, but main still shows the consent/approval/lifecycle decisions and their instrumentation. Resource cleanup and cancellation/Drop ordering are unchanged.

Telemetry's explicit production-source inventory now also hashes `src/calculator.rs`, `src/chooser.rs`, `src/diagnostics.rs`, `src/learning_record.rs`, `src/presentation.rs` and `src/provider_setup.rs`. The `course.content_sha256` metadata schema and identification purpose are unchanged; source locations and resulting hashes change. No test file is in that inventory. The standalone [dataset seeder](seed_dataset.rs) and [dataset assets](datasets/) are unchanged and remain separate from tutor execution.

### Activate the complete working copy

Root `src/` is an ephemeral exercise copy. From the repository root, replace it with **all** reference sources, including child test directories:

```bash
rm -rf src
mkdir src
cp -R lessons/14-agent-evaluation/src/. src/
```

Do not copy only main or only top-level `.rs` files. No backup/restore is required for this exercise copy; preserve authoritative lesson references and unrelated work elsewhere. Finish with root matching Lesson 14. Do not reset it to hello world before the combined review.

## Prerequisites, services and exact commands

Complete Lesson 13's distinction between authoritative state, learner-authorized memory and lossy working context. Keep its local tutor, compatible Harrier/llama.cpp embedding runtime and read-only PostgreSQL setup. Use [Lesson 12 DATABASE-SETUP.md](../12-specialized-retrieval/DATABASE-SETUP.md); do not reload the provisional 511-passage collection, change models or prepare a corpus for this exercise. Rights to private text and disclosure remain operator responsibilities.

Follow [MLFLOW-SETUP.md](MLFLOW-SETUP.md) for the pinned official **MLflow v3.17.1** server, local experiment creation and numeric experiment ID. No Python evaluation runner or model judge is introduced. Server setup is supplied infrastructure, not a new learner programming assignment. Do not install an unofficial exporter or migrate the exact published alpha.11 GDK during class.

The **evaluation storage name** selects a disposable local directory so rehearsals do not reuse the everyday learner record or overwrite everyday diagnostics. Define it before launching. From the repository root, with the supplied Lesson 14 working copy and the existing private connection file:

```bash
umask 077
source day_trading/embeddings/postgres-local/connection.env
export RETRIEVAL_DATABASE_URL="$READER_DATABASE_URL"
export HARRIER_URL=http://127.0.0.1:9999
export TUTOR_URL=http://127.0.0.1:8080
# Replace 1 with the numeric experiment ID from MLFLOW-SETUP.md.
export MLFLOW_EXPERIMENT_ID=1
export EVALUATION_STORAGE=baseline-V02
export COMPACTION_TRIGGER_BYTES=60000
cargo run -- custom_aa_llama_qwen3_6-35b.json
```

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
cargo check
cargo clippy --all-targets
cargo test --all-targets
cargo build
```

The explicitly opt-in real-service tests are separate from ordinary tests. With the configured real services above:

```bash
cargo test live_local_smoke -- --ignored --nocapture
cargo test live_same_snapshot_sdk_search_batch_obeys_one_evidence_budget -- --ignored --nocapture
```

These integration checks are not a replacement for the conversational scenarios. Record actual commands/results, skipped cases and unavailable services in the validation/comparison evidence; do not infer live behavior from the deterministic suite.

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

In MLflow inspect a learner-turn parent and its available inference/tool children. Compare span inputs/outputs with the local correlated requests/results and final ordinary answer. Record observed token usage and timing when present; missing or partial usage is **unknown**, not zero. Summary work is extra inference, not a free reduction in cost. The trace is a debugging/evidence view, not a learner transcript or private chain-of-thought review exercise.

The established minimal infrastructure proof is narrow: official v3.17.1 accepted native Rust OTLP traces, inference/usage and parent/tool relationships were observed, and named numeric feedback entered in the native UI persisted. This does **not** establish that every application path exports completely or that every evaluation case passed. Consult [VALIDATION.md](VALIDATION.md) for the actual final checks and gaps.

## 14.3 — Explain: attach judgments to evidence

Read the actual learner words, returned passages, tools, summary and final answer before rating. Separate an infrastructure block from an application failure and an uncertain semantic judgment. An eloquent answer with unsupported relationship wording is not a semantic pass. A controlled budget stop can satisfy the safety boundary while failing to complete the learning task; record both.

Apply the rubric's stable criterion names and numeric mapping when entering feedback. Add a short rationale identifying the claim and its supporting or contradicting evidence. Manually include actual dataset ID, record ID, case ID, manifest SHA-256, rubric version, comparison, reviewer and trace/span/private evidence locator in trace Notes and feedback rationale using [DATASETS.md](DATASETS.md). This text does not create automated record linkage or a native evaluation run. Preserve uncertain/not-applicable/blocked status in the worksheet; do not translate unavailable evidence into success or silently treat it as zero. Avoid an aggregate score that hides a consent or provenance violation.

The native MLflow UI supports **individual named feedback entries**. It does not provide this course's reusable ten-question form. The supplied rubric/worksheet supplies the repeatable checklist; feedback entries attach selected judgments to traces. Persisting one entry does not mean ten criteria were filled in, reviewed or automatically applied to later traces. Refresh/reopen to verify persistence and associate entries with the right case/variant. No Python judge/runner fills the rubric for you.

Open the supplied code only after these concepts are clear. The optional initialization in `telemetry.rs` connects Rust tracing to OTLP. The learner-turn span in `main.rs` supplies application parentage; tool spans expose bounded correlated work. SDK inference spans contribute inference observations. The storage helper selects the isolated files. None of these functions decides whether the prose was educationally correct. Full Session diagnostics remain local; ordinary observable dialogue/tool evidence can still contain personal data or private excerpts when exported.

## 14.4 — Tweak: change only the compaction trigger

The **trigger** is when the application considers automatic compaction, not a model context-window measurement or the hard input allowance. The controlled classroom change is **60,000 → 50,000 estimated bytes**. Keep the **84,000 estimated-input allowance** unchanged (100,000 total minus 16,000 reserved headroom), as well as all eligibility, provenance, approval, output and timeout safeguards. Word/token counts are not these byte estimates.

After saving baseline evidence and ending its process, launch a fresh variant:

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

Restore `COMPACTION_TRIGGER_BYTES=60000` for the default afterward. End the evaluation process before removing the evaluation variables or deliberately returning to ordinary storage. Preserve private artifacts only with appropriate permission; the setup guide supplies service shutdown steps. Do not delete someone else's records to clean a rehearsal.

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

## Modularization checkpoint — 2026-10-22

This is a behavior-preserving instructional reorganization, not a semantic fix or release approval. All 68 deterministic application tests and seven unchanged seeder tests passed with `cargo test --all-targets`; the two opt-in live tests remain ignored normally. Format/check/Clippy/build passed. Both live tests were explicitly attempted but failed because `RETRIEVAL_DATABASE_URL` was missing; the documented tutor launch stopped before the chooser for the same reason. The private connection file was absent, Harrier on 9999 and MLflow on 5000 were unavailable, while tutor health on 8080 returned 200. That health response is not end-to-end validation. See the [dated validation entry](VALIDATION.md#modularization-checkpoint--2026-10-22) for preservation audits, terminal error checks and exactly what remains blocked. No fresh conversational, trace-receipt or teaching success is claimed; historical findings remain intact.
