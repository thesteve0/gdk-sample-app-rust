# Instructor notes — Lesson 14: Evaluate the complete application

## Aim and handoff

Use [Lesson 14](../lessons/14-agent-evaluation/LESSON.md) to teach a defensible evaluation decision: observable structure plus human meaning judgments under controlled conditions. This evaluates the **tutor**, not the learner's mastery or readiness to trade. The instructor has source books but no expert-authored trading assessment rubric; acknowledge that limit aloud.

The complete integrated source tree is supplied and recursively matches the activated root for instructor review; [November 14 validation](../lessons/14-agent-evaluation/VALIDATION.md#learning-library-integration--2026-11-14) records actual checks and live limits separately. The November 14 continuation adds same-package learning support using actual Lesson 13 mechanics/newer policies, leaving all Lesson 13 code and the RAG API/call structure unchanged; retrieval inherits Lesson 13's same-package RAG library; the four editable Lesson 14 prompts remain unchanged and their reconciliation is deferred. Use the lesson’s [linked source map and reading path](../lessons/14-agent-evaluation/LESSON.md#supplied-modules-and-reading-path), including module-local child tests. Do not assign exporter, concurrency, runtime traits, storage routing or async wiring as missing Rust scaffolding. Predict → run → explain → tweak → run again. Published GDK alpha.11 and provider/toolchain pins stay fixed; migration is deferred until the course is finished and verified.

[VALIDATION.md](../lessons/14-agent-evaluation/VALIDATION.md) is the authoritative record of actual implementation checks. Do not infer final scenario coverage or invent a test count from these teaching notes. The narrow existing proof is official MLflow **v3.17.1** receiving native Rust OTLP, observable inference/usage and parent/tool relationships, and persistence of named numeric feedback through the native UI. This is transport/UI evidence, not semantic acceptance or release approval.

## Before class

1. Read README, Lesson 13's complete lesson/notes and its targeted correction record, then this lesson's [setup guide](../lessons/14-agent-evaluation/MLFLOW-SETUP.md), [rubric](../lessons/14-agent-evaluation/RUBRIC.md), [scenarios](../lessons/14-agent-evaluation/SCENARIOS.md), [dataset catalog guide](../lessons/14-agent-evaluation/DATASETS.md), [comparison worksheet](../lessons/14-agent-evaluation/COMPARISON.md) and validation record. Historical planning status does not replace the current implementation handoff.
2. Preserve unrelated working-tree changes. Confirm the final root activation matches the entire Lesson 14 reference source tree, including `lib.rs`, `rag/`, `learning/` and all child test directories. Do not overwrite the four editable prompts with Lesson 13 versions. Use the root manifest, not a nonexistent lesson manifest. Review the supplied exporter and actual storage behavior before explaining them.
3. Verify existing local tutor/tokenizer, Harrier runtime/profile, `harrier-llamacpp-test-511-v1` collection and reader role using [DATABASE-SETUP.md](../lessons/12-specialized-retrieval/DATABASE-SETUP.md). No database reload, embedding conversion, source ingestion or model change belongs to this lesson. Services absent? Ask for them; do not relabel mock evidence as live validation.
4. Use the official pinned MLflow setup and record the actual numeric experiment ID. Verify local native UI receipt and parentage, not just server health. No Python judge/evaluation runner. Server and dataset seeder infrastructure are supplied; learners need not become MLflow administrators. Verify the active experiment and native `lesson14-visible-v1` catalog separately from trace receipt. Catalog membership is not execution or review evidence.
5. Establish a disposable baseline name such as `baseline-V02`; confirm it is unused or deliberately prepared. Both record and diagnostic files must resolve under `.evaluation/sessions/baseline-V02/`. Ordinary root files must remain untouched by the rehearsal. Diagnostic evidence is overwritten on launch; archive it privately before restarting.
6. Fix case version, initial memory/consent, wording, guidance, models/profile/corpus, repetitions and evaluator criteria before observing responses. Identify old search-tool assumptions in the unchanged scenarios/catalog; revising them for the fully integrated application is deferred, not silently authorized here. Simplest demonstration: memory declined in both variants. For a memory case, use byte-identical seed records prepared between launches; do not pass approved additions from baseline into variant. Reserve a held-back case.
7. The actual new default trigger is 84,000 bytes and eligible-history minimum 4,000. If using the authored legacy 60,000/50,000 comparison, explicitly configure both conditions, label them legacy, and rehearse enough ordinary complete exchanges to exercise eligibility. Scenarios/catalog applicability remains unreviewed, not silently revalidated. Keep the 84,000 estimated-input allowance unchanged. Neither two replies nor a lower trigger guarantees replacement. Do not intentionally overflow the provider or break services as learner exercises.
8. Prepare one permissioned sanitized satisfactory observation and one imperfect observation. Label historical versus current evidence and date each. Do not invent a successful live transcript if the rehearsal was blocked. Do not project raw private books, credentials, environment connection strings or personal conversation.

## Modular source reveal and activation

Define a **module** as a named part of this application; `mod` loads local code and `use` brings names into scope. Define a **library target** before showing `gdk_hello::rag`: `src/lib.rs` exposes reusable RAG and learning code within the **same package**, not a workspace, separate retrieval package, new dependency or shared course framework. The binary imports the library rather than declaring the old local `retrieval` module. Use the lesson's complete linked source map, not a module-construction exercise. Every supplied Rust file starts `// Lesson 14`.

The six support modules remain binary-local: calculator, chooser, diagnostics, learning_record, presentation and provider_setup. Binary-local compaction now retains application orchestration/budget/prospective/revision/declaration/Operation decisions while calling library mechanics; telemetry remains binary-local, with child tests. The record file is a thin isolated-path/confirmation/traced-append adapter, not the storage implementation. The inherited `rag/` subtree contains `mod.rs`, `retrieval.rs`, `grounding.rs`, `citations.rs`, `transport.rs`, `tests.rs` and `retrieval/tests.rs`. Main supplies service defaults, retrieval limits, grounding policy and the RAG-specific contract; main still owns context assembly, consent/authority, Session/runtime/Effects, concurrent Events, dispatch/limits, lifecycle/cancellation and successful-run inspection. Keep approval → decision log → append → Session refresh visible. `TerminalProse` is rendering support, not Event consumption or write authority.

Reveal main's `run_grounded_reply` → `prepare_turn_grounding` → library selection, then return for pending-input checks and `append_grounded_user`. Define **grounding** as supplying excerpts for attribution, **replacement evidence** as replacing the separate attachment rather than appending excerpts to history, and a **pending reply** as a not-yet-stored question plus its replacement evidence. A **citation registry** remembers prepared source IDs/reference metadata; membership does not prove support or delivery. A **citation gate** checks membership before showing buffered text, not truth. Only then show `TurnGrounding`, `compaction::PendingReply` and `CitationRegistry`.

- Retrieval prepares **once per genuine learner reply**, not kickoff, commands or correction. The default threshold is **0.55** (`GROUNDING_MIN_SIMILARITY` may override validly), at most **3** ranked excerpts are selected and the search response has a **2,500 tutor-token budget**. The prompt's three-statement cap is not a semantic check. Similarity does not certify relevance or truth. `no_evidence` and `unavailable` must remain honest outcomes.
- Evidence stays outside `Conversation`, with the `source-grounding` contribution first in each inference's system-prompt parts. New-turn evidence replaces old evidence, including an empty replacement. No model-selected search provider/definition/allowlist/dispatch or shared search-tool evidence ledger remains. The library preserves local transport restrictions, read-only profile/provenance checks, serialized search, tutor-tokenizer accounting and owned database-driver cancellation/Drop guards. Calculator and consent-enabled proposal tools retain their limits.
- Prospective preflight and candidate before/after maintenance measurements count the same pending question and replacement evidence without storing either first. Summary input remains eligible stored history, not pending text or the separate excerpts. Accept question/evidence/genuine message ID together only after candidate-budget checks; inference retains its own hard guard. This prospective counting is retained by the learning-library continuation; it does not imply migration of features merely claimed in historical docs.
- Provider Events are consumed concurrently into a bounded **buffer**, not displayed live to the learner. The successful attempt's final answer is checked against current IDs and `CitationRegistry` before releasing that buffer. Never-supplied IDs withhold the first reply; a silent-ignore reply without invalid IDs can be displayed before acknowledgment correction. Stream completion or saved history is not presentation evidence.
- Grounded silent-ignore/invalid-ID outcomes share **one correction**, reusing evidence and fresh attempt bounds. Settle first-attempt proposals with independent default-no approval **before** appending the application reminder. Only genuine learner IDs can justify quotes; the reminder cannot. Settlement uses kickoff ID plus request ID, so a new attempt can reuse a request ID without silently inheriting the old decision. Classify/report the correction without another retry. A still-invalid second answer is shown unverified with warnings; a valid explicit decline is reported, not semantically justified by the application.

Be explicit about parity limitations: registration precedes question/evidence acceptance, so membership is not proof of accepted delivery. Earlier-turn registered IDs resolve and are labeled as such, not certified as current support. The lexical extractor is not a full strict-syntax validator; cite-plus-decline is warned rather than automatically rejected. The no-evidence classifier returns `NotSupplied`, so withheld fabricated citations on that path do not necessarily trigger correction. A legitimate-looking ID or `[source:none]` cannot establish relevance, adjacent-claim support, truth or justified refusal. Do not claim a semantic gate, guarantee that every source-derived sentence is cited or suggest a database lookup validates support. These edge cases remain deferred, per [RAG-INTEGRATION-PLAN.md](../lessons/14-agent-evaluation/RAG-INTEGRATION-PLAN.md).

Trace the instrumented boundaries, without moving authority into telemetry:

| Boundary | What to reveal |
| --- | --- |
| `run_with_telemetry` | Storage isolation, optional initialization off the async worker and flush/shutdown on ordinary exit remain. |
| `learner_exchange` → `automatic_grounding` | Application-owned `RETRIEVER` preparation records query, threshold, status and selected evidence, never a model search-tool call. Pending maintenance and the subsequent turn belong to that genuine reply's exchange. Kickoff does not retrieve. |
| `learner_turn` → attempt projections | `run_attempt` records attempt number, lexical acknowledgment, ordinary answer, invalid markers and `withheld`/`not_yet_presented`. These are attributes/events, not separate attempt spans. Saved inference messages can include withheld answers. |
| Appended correction → learner-visible output | `course.rag.correction` follows accepted reminder append/first-attempt settlement. `course.rag.learner_visible_output` follows presentation, including a warned still-invalid final correction. Do not infer visibility from the span's last output or saved history; inspect phase events and diagnostics. |
| Inference/tools/approvals/summary | Preserve correlated calculator/proposal work, independent local approval observations and `context_maintenance`/`summarize_context` evidence and cost. |

`compaction::BudgetPreparer` composes conservative bounds, ordinary inference telemetry and exact prepared-system-prompt capture. Captures drain by attempt/request even on failed runs; accepted preparation is not proof of network delivery. Observational projections exclude provider reasoning/message metadata and do not deliberately export credentials/configuration. Ordinary instructions, excerpts and approved notes can still be private. `not_semantically_verified` is an intentional limitation, not a grading result.

Explain source identity: `course.content_sha256` still maps paths to hashes, not source text. It covers main/lib, six support modules, the five production RAG files, compaction/telemetry, manifest/lock/toolchain, rubric/scenarios and four editable prompts. Library paths replace obsolete `src/retrieval.rs`; the five production learning files (`mod.rs`, `record.rs`, `context_size.rs`, `compaction.rs`, `summary.rs`) are also hashed; test paths are not inventoried. Collection/profile identity comes from the library's nonsecret interface. Do not attach prior source hashes or trace success to these new bytes. Seeder/catalog definitions and contents remain untouched.

### Learning continuation and remaining integration limits

Define **learning mechanics** as already-taught record storage, history projection/partition/candidate rules, isolated summary inference and informational sizing. A **binary adapter** attaches application paths/IO/telemetry without taking over consent or approval. Define a **summary observer** as an observational callback, never authority to append or replace. Only then open `gdk_hello::learning` and the linked `learning/{mod,record,compaction,summary,context_size}.rs` source map. There is no separate package, binary-telemetry dependency or framework exercise.

Main retains memory/provenance/approval/runtime/RAG decisions with optional runtime context sizing and explicit `Policy::from_env(Policy::default())`. Binary compaction owns orchestration, prospective input/budgets, revision acceptance, declaration and maintenance Operation; the library supplies mechanics and `SummaryRunner`. The summary Effect captures its originating chat span at complete-message conversion; on application after SDK ID assignment, the inference-output callback re-enters that same span, then the saved hook runs. Explain why surrounding maintenance parentage alone would not establish same-span output. Preserve bounded cancellation/drains/Drop order and partial failure/usage evidence.

Current policies adopt actual Lesson 13: **84,000-byte trigger**, **4,000-byte eligible minimum**, latest two genuine exchanges and unchanged **84,000-byte hard allowance**. `/learned` ALWAYS appends its explicit local declaration independently of consent to read notes, subject to duplicate/capacity/storage checks. With consent declined, do not reload the record into context. Command maintenance remains unconditional and can no-op; a failed summary does not undo an already-saved declaration or grant retry-write authority. Model proposals still need independent default-no approval after successful inspection; neither a declaration nor a summary establishes mastery.

The four editable prompts remain unchanged/unreconciled. Their optional-search and consent-dependent memory instructions may conflict with automatic grounding and always-append; **application authority wins**. No activity/continuity/source-review/teaching modules exist in actual Lesson 13 to migrate, despite historical prose; do not demonstrate or claim them here. Prompt/scenario/rubric/catalog/comparison applicability remains unreviewed. Do not silently mutate versioned data, relabel old evidence as current, or claim semantic fixes.

Root and the complete reference are synchronized. Activate recursively from root, including `learning/` and all child tests, not by copying main or a top-level glob:

```bash
rm -rf src
mkdir src
cp -R lessons/14-agent-evaluation/src/. src/
```

Root is ephemeral: no backup/restore or final hello-world reset now. Preserve authoritative references and unrelated work, and leave root matching Lesson 14 for combined review.

## Commands to use from the root

Use the setup guide's actual numeric experiment ID in place of the illustrative `1`:

```bash
umask 077
export MLFLOW_EXPERIMENT_ID=1
export EVALUATION_STORAGE=baseline-V02
unset COMPACTION_TRIGGER_BYTES  # actual current default: 84000
cargo fmt --check
cargo check
cargo clippy --all-targets
cargo test --all-targets
cargo build
cargo run -- custom_aa_llama_qwen3_6-35b.json
```

Run real-service checks separately with services/environment configured. The automatic-grounding test uses a test selection policy, not semantic acceptance of the default threshold:

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


For the authored **legacy** 60,000/50,000 comparison, first explicitly set `COMPACTION_TRIGGER_BYTES=60000` with fresh `EVALUATION_STORAGE=legacy60000-V02` and capture current evidence. An unset 84,000 run is not that baseline. After ending the explicitly configured 60,000 process and preserving evidence, change **only** its comparison trigger and disposable storage variant:

```bash
export EVALUATION_STORAGE=trigger50000-V02
export COMPACTION_TRIGGER_BYTES=50000
cargo run -- custom_aa_llama_qwen3_6-35b.json
```

Keep the same numeric experiment, all content/settings and matched initial memory. Use new run names for repetitions; do not accidentally reuse a prior record or overwrite the only diagnostic copy. The paths are repository-relative, not subprocess-working-directory-relative. Use `unset COMPACTION_TRIGGER_BYTES` to restore the actual 84,000 default afterward; end evaluation before deliberately returning to ordinary storage.

## Dataset-catalog teaching script (before the execution reveal)

Define an **evaluation dataset catalog** as stored authored case definitions, a **record** as one setup/input/inspection target, and **seeding** as registering definitions, not executing or grading. Say: “Ten records tell us what we intended to inspect. They do not tell us what the tutor did.” The supplied Rust example is not a runner/judge or another incomplete Rust exercise.

1. **Predict:** open V02 in [`datasets/visible-v1.json`](../lessons/14-agent-evaluation/datasets/visible-v1.json). Ask “Which field contains an actual tutor answer?” None. Point out `inputs.case_id/setup/learner_turns`, `expectations.inspection_targets` and the optional arithmetic anchor, then `tags.rubric_version/prerequisites` and `expectations.applicable_criteria`. Arithmetic 100.00 is a deterministic inspection anchor, not a filled semantic rating.
2. **Seed/verify:** using the existing active numeric experiment ID, from root:

   ```bash
   cargo run --example seed_mlflow_dataset -- 1
   cargo run --example seed_mlflow_dataset -- 1 --check
   ```

   Replace illustrative `1`; do not restart services. Record the actual dataset/record IDs and SHA-256. Read-only verification checks catalog identity/content, not the tutor. Exact records are skipped; an exact interrupted subset can resume. Same-name/different-manifest, duplicate names, edited/extra records are refused, not silently repaired. The manifest/schema tags are guards, not immutable native versions: intentional updates need a new dataset name/manifest and preservation of old evidence.
3. **Inspect catalog:** find ten visible records V01–V10. V04 conflict stays in V04, as a separate fresh execution branch. Keep V04 conflict and V10 blocked pending separately prepared compatible fixtures/constrained selection. The seeder installs neither fixtures nor retrieval data. Held-back cases remain unchanged/unseeded. Every Q01–Q10 judgment starts unreviewed, with separate blocked/skipped/NA statuses, never a catalog-derived pass or summed score.
4. **Run manually:** read setup plus learner turns, then use a unique `EVALUATION_STORAGE` for each condition/case/repetition. Preserve synthetic matched starting memory and consent decisions. Do not treat V08 as permission to approve any note: inspect actual quotation/category/note after a successful run, default no, and approve only an appropriate addition. A missing or inappropriate proposal is a coverage finding, not a reason to force model behavior or click yes.
5. **Inspect execution:** preserve actual trace/local evidence and enter separate criterion feedback. In Notes and each rationale include actual dataset ID, record ID, case, manifest SHA-256, rubric, comparison, reviewer and trace/span/private evidence locator. Say: “We linked these by human-readable identifiers. This is not automatic trace-to-record linkage or a native evaluation run.” No trace import, expected private excerpt answers or hosted judge is supplied.

The publicly undocumented native `/api/3.0/mlflow` dataset wire surface is pinned to v3.17.1 source; JSON-string tags/records and input-deduplicating upserts explain the defensive checks. Do not ask learners to author raw endpoint requests, install a Python package or modify SQLite directly. Source compatibility is not live validation: use [VALIDATION.md](../lessons/14-agent-evaluation/VALIDATION.md) for the actual checks. Public catalog text is authored synthetic course content; actual answers, memory and returned book excerpts can still be private. Never project credentials/private source dumps while demonstrating linkage.

## Reveal script (approximately 50–60 minutes, plus catalog demonstration)

### 1. Define, motivate and predict (8 minutes)

Say: “Tracing tells us what happened. Evaluation asks whether it was acceptable. A beautiful answer can still use the wrong source or invent your agreement.” Define diagnostics, trace, span, parentage, telemetry/OTLP, evaluation case/set/catalog/record, rubric, baseline, regression/gate and semantic judgment **before** showing code or UI terms.

Prediction prompts:

- “Would a valid citation ID prove the sentence is supported?” No: membership versus meaning.
- “Would an exact $100 calculation prove the tutor avoided an execution guarantee?” No: arithmetic versus qualifications.
- “Would a persisted numeric rating prove an expert reviewed it?” No: storage versus judgment.
- “Can improved fluency offset an unauthorized note append?” No: a blocking gate, not an average.

Locate stable criteria in the supplied rubric, identify the verified catalog record and fixed cases/repetitions, and announce a held-back case. Use pass/fail/uncertain with evidence; blocked/not-applicable remain visible. Do not tune the rubric after reading the result.

### 2. Run and identify actual evidence (10 minutes)

Launch the isolated baseline. Let the learner make the specified consent and approval choices. Run the rehearsed conceptual, planned-loss or source case without forcing tool selection. Record actual no-call/call/result/failure; no selectively discarded attempts.

Find `learner_exchange`, its automatic-grounding preparation and applicable maintenance, then the `learner_turn` attempts, inference observations and applicable calculator/proposal children in MLflow. Distinguish withheld output, appended correction and learner-visible output; absence of a search-tool span is expected, not retrieval failure. Compare correlation and ordinary outputs with local diagnostic state. Say: “An absent span is not yet proof that nothing happened.” Check export receipt and local evidence. Report usage when present and unknown when absent; cancellation/partial state do not justify zero cost. Summary generation contributes inference cost.

The learner terminal retains clean Tutor/Learner dialogue, brief results and presentation-only colors. Do not turn it into a raw trace or Session viewer. Exported ordinary dialogue/tool evidence can still contain private material; obtain consent and sanitize views.

### 3. Explain and rate without pretending automation (10 minutes)

Inspect actual returned passages and claims, not just IDs. Inspect learner statements alongside a summary/answer, not just its length. Use the rubric and comparison worksheet to distinguish deterministic enforcement, observable completion and semantic support.

Enter selected **individual named numeric feedback entries** in the native MLflow UI with the rubric's mapping and a short evidence rationale including actual dataset/record ID, case, manifest fingerprint, rubric, comparison, reviewer and private evidence locator. Refresh/reopen to verify persistence on the intended trace. Say explicitly: “This UI is not a reusable ten-question form. The supplied worksheet is our repeatable checklist; no Python judge fills it for us.” Ten criteria do not become completed because one feedback entry saved. Uncertain/blocked status needs explicit worksheet context, not an invented pass or aggregate shortcut.

Ask two attendees to inspect the same claim. If they disagree, retain both rationales and adjudicate against the actual text or mark uncertain. Do not assert expertise you do not have. A model-generated reference answer is not independent truth.

Only now open supplied code: `telemetry.rs` optional initialization/fixed local transport; main's exchange/preparation/attempt/presentation and tool/approval boundaries; native SDK inference observations; storage helper selecting both files. These collect evidence; they do not determine educational correctness. Complete local diagnostics remain distinct from the exported observation view. Do not teach full Goose assembly as required infrastructure.

### 4. Predict and tweak one variable (5 minutes)

Define trigger versus hard allowance. The current default is **84000**, not 60000. In the explicitly authored legacy comparison only, change `COMPACTION_TRIGGER_BYTES` **60000 → 50000**, not the **84000** estimated-input allowance. Keep scenario/catalog assumptions labeled unreviewed for this continuation; no fresh applicability judgment is implied. Total 100,000 less 16,000 headroom, native serialization/argument/thinking/protocol margins: estimates, not exact tokens or captured wire size. All other guards remain unchanged.

Prediction: compaction may become eligible earlier, but eligible older history, more than two genuine complete exchanges, reduction/size/revision checks and a safe boundary are still required. More frequent lossy summaries might worsen fidelity or cost. Do not present “smaller context” as automatic improvement.

Start a fresh Session with matched starting memory and decisions. Changing the experiment does not isolate disk state; changing storage does not automatically make a repeated directory clean. `/learned` has existing command maintenance semantics; do not use its manual maintenance as proof that automatic threshold behavior was exercised.

### 5. Run again and compare (12 minutes)

Replay the scenario and repetitions. When actual replacement occurs, ask the identical follow-up about unresolved uncertainty and agreed activity. Compare the summary **and** answer with actual learner words. Check no strengthened accepted scope, no tutor explanation treated as confirmation, no execution guarantee, exact calculator qualifications and the actual new-turn evidence. Earlier registry IDs can pass membership; compare their relevance to the current claim rather than treating a resolved footer as fresh support.

If neither configuration compacts, record **unexercised compaction**, not a tie proving fidelity. If protected input stops safely, record safety-boundary success separately from task noncompletion. If trace export fails, retain local evidence and mark observability blocked; do not fabricate parentage or feedback.

Complete the worksheet: invariant gates, observable actions, semantic pass/fail/uncertain, time/usage/summary count, unknowns and held-back cases. Report observed numerator/denominator and configured repetitions, not universal reliability. Discuss whether any apparent improvement could be ordinary model variation or different initial memory.

Decision choices: adopt with limited evidence and documented remaining gates, reject a regressing variant, or keep baseline because evidence is inconclusive. Classroom comparison is not release authority.

### 6. Explain the handoff (5 minutes)

Ask attendees to name an invariant test, a semantic inspection and one remaining uncertain claim. Ask what further evidence would change their decision. Restore defaults and preserve only permissioned private artifacts. End with review obligations, not a graduation claim that the agent is safe to trade with.

## Current sizing pattern — completed exchanges only

The learning library now supplies `ContextSize`, following actual Lesson 13 code rather than the more ambitious historical prose: `Context: ~N tokens (estimate)` appears only at the end of a completed exchange (including welcome), after approval handling. The binary keeps an optional runtime estimator and assembles actual stored prompt parts/tools/history/evidence for it. Do not promise a status at every blank/command/local prompt or token before/after-compaction pairs. Applied-summary notices and conservative byte diagnostics remain distinct.

Native-format content UTF-8 JSON bytes divided by four, rounded up, includes the completed answer and no unknown future learner text; it excludes cumulative usage/application-only metadata/safety margins. Selected-model/thinking formatting is respected. No tokenizer/service call or state mutation occurs; unavailable sizing is nonfatal. This is not measured tokens, wire size, capacity, a guard or summary-fidelity evidence. The dated discrepancy below remains an honest description of the November 13 tree, now superseded by this explicit addition.

## Current display discrepancy — 2026-11-13

> Historical baseline observation, superseded by the November 14 learning-library addition above. The dated text is retained, not a claim about the new current implementation.

The tracked integrated Lesson 14 tree has **no `context_size.rs`, no sizing child tests and no prompt-boundary sizing calls**. Do not rehearse a context-token display from this source. Maintenance currently reports conservative **byte** measurements in diagnostics/telemetry; the terminal's applied-summary notice is distinct from no-op/failure/rejected replacement. Neither smaller bytes nor a shorter summary proves fidelity or measured token savings.

Preserve the [November 12 historical display evidence](#cli-context-display-evidence--2026-11-12) and its [validation entry](../lessons/14-agent-evaluation/VALIDATION.md#cli-context-estimates--2026-11-12) unchanged as dated reports, not current implementation claims. This pass corrects documentation; it does **not** claim to remove an implemented display. Reconciliation/implementation remains deferred with the broader compaction/memory phase.

## Gates to keep distinct

| Gate | Required inspection | Not sufficient |
| --- | --- | --- |
| Calculator/capability boundary | Exact decimals, allowlists/schema/all-block limits, actual correlation | Fluent explanation or one green UI rating |
| Memory authority | Consent to read/use saved notes, independent proposal approval, no post-failure proposal writes; explicit `/learned` write authority independent of read consent | Proposal acknowledgement or summary quotation |
| Retrieval support | Current returned evidence, source scope and adjacent-claim meaning | Citation membership, high similarity or old summary IDs |
| Compaction authority/fidelity | Protected exchanges/pairs, revision/provenance/settled approvals; uncertainty/corrections/accepted scope | Smaller byte estimate or sensible follow-up alone |
| Observability | Local evidence plus actual trace/feedback receipt, partial/unknown usage disclosed | Export enabled, health response or persisted score |
| Teaching quality | Cautious source-supported feedback, no inferred mastery, appropriate activity agreement | Tutor confidence or source agreement alone |

Maintenance/adversarial cases stay in supplied coverage. No classroom failure injection, private instruction attack, provider overflow or guard removal is implicitly authorized. A human rating cannot override a deterministic authority violation.

## Preserve inherited defects and limits

Carry these forward in comparison notes and final review, even if a selected case improves:

- Lesson 12 source-scope/feedback relationship errors despite valid IDs, unsupported-price refusal without search, a loaded activity not offered in welcome, and a source run without a final ordinary answer.
- Lesson 13 execution guarantees, tutor explanations closing unconfirmed questions and invented accepted-activity details. The 2026-10-13 targeted corrections have limited positive replays, not global correctness. Remaining buy/sell limit, price-or-better and spread wording problems must remain visible.
- Rejected/truncated summaries, protected-input safe stops and missing mid-turn rescue. Cancellation is not proof the remote service stopped; a previously saved declaration is not automatically rolled back or repeated.
- Provisional three-PDF/511-passage coverage, unfinished corpus acceptance/freeze/full-source conversion, no runtime parity or expert retrieval/teaching benchmark. Source agreement is not certification of book correctness.
- Earlier review/UI/teaching and live-validation gaps, including Lesson 9's 100-share tweak. End-of-course content/teaching/UI/release review is deferred, not satisfied by this lesson.

No complete network-isolation/adversarial reliability, malicious pathname-race hardening, transactional/exactly-once persistence or production deployment claim. Git ignore is not security; local storage/server access must be assessed independently. Do not upload private source text to hosted judges or publish private transcripts as public fixtures.

## Validation and end-of-course review

Update judgments from actual [VALIDATION.md](../lessons/14-agent-evaluation/VALIDATION.md) evidence, never from commands listed as instructions. Keep deterministic checks, real-service tests, live scenarios, native UI proof and semantic judgments separate. Record skipped/blocked cases and observations that did not exercise compaction. The narrow official v3.17.1 Rust OTLP/feedback proof is not a final application acceptance result.

Final review should consider complete materials, observed teaching/UI behavior, baseline/variant evidence, consent/capability gates, known defects and independent prior gaps. A local numeric feedback value is one evaluator's annotation, not a reusable assessment instrument or mastery score. No Python judge/runner, automatic trade readiness assessment, simulator, general CLI, fine-tuning or SDK migration is added as an implicit final requirement. The course ends with an honest method for deciding what to improve next, subject to instructor authorization and review.

## RAG documentation handoff — 2026-11-13

This update documents inspected source and deferred boundaries only. Full checks, synchronization/root activation, real-service tests, CLI conversations, correction branches and trace receipt still await the coordinating agent's evidence. Consult the **forthcoming dated RAG-integration record in [VALIDATION.md](../lessons/14-agent-evaluation/VALIDATION.md)** before making any validation claim. Do not invent counts, reuse old passes as new integration evidence or promise semantic improvement. Finish with the complete Lesson 14 source at root for instructor review; no release approval or later integration is implied. Historical records below retain their original dates and limitations.

## Modularization validation checkpoint — 2026-10-22

The [dated validation entry](../lessons/14-agent-evaluation/VALIDATION.md#modularization-checkpoint--2026-10-22) records 68 deterministic application plus seven unchanged seeder tests, formatting/check/Clippy/build, preserved bodies/constants/test names and the corrected missing test import. Both opt-in live checks were attempted and failed for missing `RETRIEVAL_DATABASE_URL`; the documented launch failed before selection too. The private connection file was absent; Harrier 9999 and MLflow 5000 were unavailable, tutor 8080 health returned 200. Do not call that a live teaching or export pass. Parent validation checked plain/empty-NO_COLOR/dumb errors and independent stderr gating only, not a fresh learner interaction. No service restart, catalog write, semantic fix, dependency change or commit belongs to this checkpoint. Preserve all historical failures, incomplete V04/V05/V10 and held-back cases, and pending review; ask the instructor to provide prerequisites before fresh end-to-end validation.


## Local service defaults — 2026-10-28

The three local classroom defaults are `const` values at the top of `src/main.rs`: `RETRIEVAL_DATABASE_URL` uses the instructor-supplied `retrieval_reader` connection to `127.0.0.1:55432/harrier`, `HARRIER_URL` is `http://127.0.0.1:9999`, and `TUTOR_URL` is `http://127.0.0.1:8080`. No exports or private connection file are required for these defaults. Existing environment variables (including values loaded by `dotenvy` from `.env`) override them. For another prepared database, set `RETRIEVAL_DATABASE_URL` to its reader connection, never the loader/admin connection. The supplied password is **plaintext in source and compiled binaries**; this is an explicit local classroom convenience, not secret storage or a production credential pattern. Rotate/replace it for any other deployment and do not project the credential-bearing source. Defaults do not start services or bypass runtime/profile/reader validation. `TUTOR_URL` selects the tokenizer origin and must match the tutor provider JSON; it does not override generation configuration.


### Service-default validation — 2026-10-28

Added the instructor-supplied reader connection and Harrier/tutor origins as constants at the top of main. Retrieval uses them only when the corresponding environment variable is absent; the main tokenizer-origin comparison uses the same tutor default. Invalid/non-Unicode retrieval environment values still fail instead of silently selecting a default. No process environment is rewritten, and no credential is deliberately added to diagnostics or telemetry. Provider JSON, compatibility checks, reader checks, dependencies and services are unchanged. The plaintext default is a classroom-only exception, not secure credential management.

Locked format/check/all-target Clippy/all-target tests/build passed in temporary staging for each complete reference: Lesson 12 **41**, Lesson 13 **67**, Lesson 14 **69** deterministic application tests, plus seven unchanged seeder tests each. One new test per lesson validates the reader host/role/database/port, local service origins and absent-variable fallback. The active Lesson 12 root independently passed the same checks and byte-matches its reference. Existing staged changes were preserved; no lesson activation replaced the user's root.

With all three override variables unset, both opt-in actual-service tests and the documented launch were attempted for each reference. All stopped with **`embedding runtime identity mismatch`** before the chooser: the existing Harrier service does not satisfy the pinned runtime identity check. This is blocked live validation, not a passed database connection, retrieval, tutor exchange, approval, compaction or telemetry run. No service restart, compatibility relaxation, corpus/catalog write or mock substitution occurred. No fresh IDE/projector/UI or teaching-quality validation is claimed; historical findings and review gates remain.


## Embedding server build flexibility — 2026-10-28

At the instructor's request, the learner application no longer requires an exact llama.cpp build string. A different or absent `/props` `build_info` is accepted. Model alias, F16 format, GGUF checksum, dimensions, context capacity, pooling, normalized vectors, query formatting, stored collection/profile identity, provenance and reader restrictions remain checked. The stored profile's build suffix records the original embedding preparation; it is not a requirement that the current query server report that build. Accepting another build does not establish numerical equivalence or retrieval-quality parity. The standalone preparation loader/client retains its separate pinned-build policy. No service restart, re-embedding or collection mutation is needed solely to pass the removed build check.

Validation for this change: formatting, locked check, all-target Clippy, all-target tests and build passed (69 deterministic application tests plus seven unchanged seeder tests). Both opt-in actual-service retrieval tests passed against the currently running services, including server build `b11669-620d2225c`; the documented launch reached the chooser and exited on EOF. Lesson 12 was checked in the active root; Lessons 13–14 were checked in temporary staging without replacing root. Regression coverage accepts changed/absent build metadata and still rejects incorrect model alias/format, sleeping/no-slot runtime and mismatched context. No full tutor conversation, approval/compaction/telemetry workflow, numerical cross-build parity benchmark, fresh UI/IDE/projector or teaching-quality review was performed. No services restarted, corpus/catalog modified, preparation client changed or dependencies updated. Existing historical defects/review gates remain.

## Learner-led scope follow-up — 2026-10-28

The content now explicitly accepts learner-chosen day-trading questions beyond the four supplied examples. Offer the menu only when requested or when the learner wants help choosing. Preserve lesson-specific capability, memory, citation and continuity instructions rather than copying a common calculator-only contract across lessons. For a fresh-selection rehearsal, ask “What should I consider when comparing brokers?” and then choose another topic outside the examples. Inspect whether the tutor answers educationally without redirecting to the preset units, inventing current facts or recommending a personalized financial decision. In retrieval lessons, separately ask what the supplied books support and inspect actual returned evidence/citations.

Validation of this edit: static content checks across Lessons 10–14 and the active Lesson 12 context-loading test passed. No new live tutor, memory approval, retrieval, compaction or telemetry behavior is established by these checks; fresh teaching validation remains pending.


## CLI context display evidence — 2026-11-12

The preserved Lesson14 layout passed warning-free staging checks with **85 deterministic application tests**, seven unchanged seeder tests and both actual-service tests. Its live rehearsal displayed 13 status/prompt pairs and applied compactions **~12617→11153**, **~12096→10662**, separately **70329→59790**, **69501→57826 bytes**. TERM=dumb calculator$100, blank/commands/reselection and colored-arrow five-pair/independent-stderr checks passed. Use the corrected binary-target maintenance commands after root activation: calibration still needs Lesson13's library. Health/version readiness did not supply the missing MLflow experiment1 (404), so no fresh trace export or inventory-receipt claim. See [VALIDATION](../lessons/14-agent-evaluation/VALIDATION.md#cli-context-estimates--2026-11-12) for exact evidence and teaching defects, including misattributed unresolved uncertainty.

Estimates are not measured tokens, safety limits, summary-fidelity proof or cumulative inference cost. Live consented-note saves, formatter failure, rejected replacement and colored actual compaction were not freshly checked; deterministic coverage is separate. No services restarted, corpus/catalog mutated, real records used, pins/dependencies/prompts changed, new quality/rubric/IDE/projector review or release approval. Historical gaps and pending instructor review remain.

## RAG-only checkpoint evidence — 2026-11-13

Root/reference recursively match the complete lib/bin/RAG tree. Final fmt/check/all-targetClippy/tests/build passed(121 deterministic app+7 unchanged seeder); both actual-service tests passed. Isolated live continuing grounding/reference/$100 calculator, one acknowledgment correction, separate approved/rejected notes and native trace readback were observed. See [VALIDATION](../lessons/14-agent-evaluation/VALIDATION.md#rag-only-library-integration--2026-11-13) for exact hashes/trace relationships, initial failed commands, semantic findings and unexercised branches. Four editable prompts remain unchanged/unreconciled. Do not imply a teaching fix, full compaction/memory/display migration, held-back evaluation or release approval. Stop for instructor review; no catalog or service changes authorized by the evidence.


## Learning-library authorization/current architecture — 2026-11-14

Approved same-package learning support follows actual Lesson 13 policies/patterns, not absent activity/continuity/source-review/teaching implementations. ALL Lesson 13 code and RAG API/call structure remain untouched. Five learning production paths join the same-schema source inventory. The binary retains local authority and telemetry adapters, including originating-chat-span summary output callbacks after ID assignment; record storage and isolated tool-free summary mechanics live in the library. Current default is 84,000/4,000 bytes; `/learned` always appends without reloading declined memory and requests maintenance unconditionally. Sizing is completed-exchange-only, not every-local-prompt or token before/after compaction.

**Root/reference recursively match; final warning-free locked checks passed: 186 deterministic application tests (91 library + 95 binary), seven unchanged seeder tests; three opt-in real-service tests separately passed.** Isolated inspected approval/new0600 and rejected note branches, declined-consent declaration/no reload, completed-exchange estimates and plain/empty-NO_COLOR/dumb/colored-chooser startup are observed. Authored-history real-provider summary applied64,947→48,843 conservative bytes with protected state retained; native receipt matched29 hashes and summary chat output callback placement. Earlier timeout/hard-abort and externally stopped incomplete CLI branches remain evidence. See [VALIDATION](../lessons/14-agent-evaluation/VALIDATION.md#learning-library-integration--2026-11-14): no completed continuing-chat compaction/follow-up, new matched comparison or semantic improvement is claimed. Preserve all dated evidence/limitations. Four editable prompts remain unreconciled, with old memory instructions potentially conflicting with always-append (application authority wins); legacy comparison/versioned catalog assumptions remain unreviewed. No prompt/corpus/pin/service/catalog/seeder change, commit or release approval is included. Stop for instructor review after coordinating validation.
