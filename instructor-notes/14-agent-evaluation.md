# Instructor notes — Lesson 14: Evaluate the complete application

## Aim and handoff

Use [Lesson 14](../lessons/14-agent-evaluation/LESSON.md) to teach a defensible evaluation decision: observable structure plus human meaning judgments under controlled conditions. This evaluates the **tutor**, not the learner's mastery or readiness to trade. The instructor has source books but no expert-authored trading assessment rubric; acknowledge that limit aloud.

Complete main/retrieval/compaction/telemetry modules and Lesson 13-derived teaching content are supplied and synchronized with root. Do not assign exporter, concurrency, runtime traits, storage routing or async wiring as missing Rust scaffolding. Predict → run → explain → tweak → run again. Published GDK alpha.11 and provider/toolchain pins stay fixed; migration is deferred until the course is finished and verified.

[VALIDATION.md](../lessons/14-agent-evaluation/VALIDATION.md) is the authoritative record of actual implementation checks. Do not infer final scenario coverage or invent a test count from these teaching notes. The narrow existing proof is official MLflow **v3.17.1** receiving native Rust OTLP, observable inference/usage and parent/tool relationships, and persistence of named numeric feedback through the native UI. This is transport/UI evidence, not semantic acceptance or release approval.

## Before class

1. Read README, Lesson 13's complete lesson/notes and its targeted correction record, then this lesson's [setup guide](../lessons/14-agent-evaluation/MLFLOW-SETUP.md), [rubric](../lessons/14-agent-evaluation/RUBRIC.md), [scenarios](../lessons/14-agent-evaluation/SCENARIOS.md), [dataset catalog guide](../lessons/14-agent-evaluation/DATASETS.md), [comparison worksheet](../lessons/14-agent-evaluation/COMPARISON.md) and validation record. Historical planning status does not replace the current implementation handoff.
2. Preserve unrelated working-tree changes. Confirm root and all four reference modules plus teaching content agree. Use the root manifest, not a nonexistent lesson manifest. Review the supplied exporter and actual storage behavior before explaining them.
3. Verify existing local tutor/tokenizer, Harrier runtime/profile, `harrier-llamacpp-test-511-v1` collection and reader role using [DATABASE-SETUP.md](../lessons/12-specialized-retrieval/DATABASE-SETUP.md). No database reload, embedding conversion, source ingestion or model change belongs to this lesson. Services absent? Ask for them; do not relabel mock evidence as live validation.
4. Use the official pinned MLflow setup and record the actual numeric experiment ID. Verify local native UI receipt and parentage, not just server health. No Python judge/evaluation runner. Server and dataset seeder infrastructure are supplied; learners need not become MLflow administrators. Verify the active experiment and native `lesson14-visible-v1` catalog separately from trace receipt. Catalog membership is not execution or review evidence.
5. Establish a disposable baseline name such as `baseline-V02`; confirm it is unused or deliberately prepared. Both record and diagnostic files must resolve under `.evaluation/sessions/baseline-V02/`. Ordinary root files must remain untouched by the rehearsal. Diagnostic evidence is overwritten on launch; archive it privately before restarting.
6. Fix case version, initial memory/consent, wording, guidance, models/profile/corpus, repetitions and evaluator criteria before observing responses. Simplest demonstration: memory declined in both variants. For a memory case, use byte-identical seed records prepared between launches; do not pass approved additions from baseline into variant. Reserve a held-back case.
7. Rehearse enough ordinary complete exchanges to make automatic compaction eligible under 60,000 and 50,000 estimated bytes. Keep the 84,000 estimated-input allowance unchanged. Neither two replies nor a lower trigger guarantees replacement. Do not intentionally overflow the provider or break services as learner exercises.
8. Prepare one permissioned sanitized satisfactory observation and one imperfect observation. Label historical versus current evidence and date each. Do not invent a successful live transcript if the rehearsal was blocked. Do not project raw private books, credentials, environment connection strings or personal conversation.

## Commands to use from the root

Use the setup guide's actual numeric experiment ID in place of the illustrative `1`:

```bash
umask 077
source day_trading/embeddings/postgres-local/connection.env
export RETRIEVAL_DATABASE_URL="$READER_DATABASE_URL"
export HARRIER_URL=http://127.0.0.1:9999
export TUTOR_URL=http://127.0.0.1:8080
export MLFLOW_EXPERIMENT_ID=1
export EVALUATION_STORAGE=baseline-V02
export COMPACTION_TRIGGER_BYTES=60000
cargo fmt --check
cargo check
cargo clippy --all-targets
cargo test
cargo run -- custom_aa_llama_qwen3_6-35b.json
```

Run real-service checks separately with services/environment configured:

```bash
cargo test live_local_smoke -- --ignored --nocapture
cargo test live_same_snapshot_sdk_search_batch_obeys_one_evidence_budget -- --ignored --nocapture
```

After ending the baseline process and preserving its evidence, change **only** the comparison trigger and disposable storage variant:

```bash
export EVALUATION_STORAGE=trigger50000-V02
export COMPACTION_TRIGGER_BYTES=50000
cargo run -- custom_aa_llama_qwen3_6-35b.json
```

Keep the same numeric experiment, all content/settings and matched initial memory. Use new run names for repetitions; do not accidentally reuse a prior record or overwrite the only diagnostic copy. The paths are repository-relative, not subprocess-working-directory-relative. Restore the 60,000 default afterward; end evaluation before deliberately returning to ordinary storage.

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

Find the learner-turn parent, inference observations and applicable tool children in MLflow. Compare correlation and ordinary outputs with local diagnostic state. Say: “An absent span is not yet proof that nothing happened.” Check export receipt and local evidence. Report usage when present and unknown when absent; cancellation/partial state do not justify zero cost. Summary generation contributes inference cost.

The learner terminal retains clean Tutor/Learner dialogue, brief results and presentation-only colors. Do not turn it into a raw trace or Session viewer. Exported ordinary dialogue/tool evidence can still contain private material; obtain consent and sanitize views.

### 3. Explain and rate without pretending automation (10 minutes)

Inspect actual returned passages and claims, not just IDs. Inspect learner statements alongside a summary/answer, not just its length. Use the rubric and comparison worksheet to distinguish deterministic enforcement, observable completion and semantic support.

Enter selected **individual named numeric feedback entries** in the native MLflow UI with the rubric's mapping and a short evidence rationale including actual dataset/record ID, case, manifest fingerprint, rubric, comparison, reviewer and private evidence locator. Refresh/reopen to verify persistence on the intended trace. Say explicitly: “This UI is not a reusable ten-question form. The supplied worksheet is our repeatable checklist; no Python judge fills it for us.” Ten criteria do not become completed because one feedback entry saved. Uncertain/blocked status needs explicit worksheet context, not an invented pass or aggregate shortcut.

Ask two attendees to inspect the same claim. If they disagree, retain both rationales and adjudicate against the actual text or mark uncertain. Do not assert expertise you do not have. A model-generated reference answer is not independent truth.

Only now open supplied code: `telemetry.rs` optional initialization and fixed local transport; `main.rs` learner-turn/tool spans; native SDK inference observations; storage helper selecting both files. These collect evidence; they do not determine educational correctness. Complete local diagnostics remain distinct from the exported observation view. Do not teach full Goose assembly as required infrastructure.

### 4. Predict and tweak one variable (5 minutes)

Define trigger versus hard allowance. Change `COMPACTION_TRIGGER_BYTES` **60000 → 50000**, not the **84000** estimated-input allowance. Total 100,000 less 16,000 headroom, native serialization/argument/thinking/protocol margins: estimates, not exact tokens or captured wire size. All other guards remain unchanged.

Prediction: compaction may become eligible earlier, but eligible older history, more than two genuine complete exchanges, reduction/size/revision checks and a safe boundary are still required. More frequent lossy summaries might worsen fidelity or cost. Do not present “smaller context” as automatic improvement.

Start a fresh Session with matched starting memory and decisions. Changing the experiment does not isolate disk state; changing storage does not automatically make a repeated directory clean. `/learned` has existing command maintenance semantics; do not use its manual maintenance as proof that automatic threshold behavior was exercised.

### 5. Run again and compare (12 minutes)

Replay the scenario and repetitions. When actual replacement occurs, ask the identical follow-up about unresolved uncertainty and agreed activity. Compare the summary **and** answer with actual learner words. Check no strengthened accepted scope, no tutor explanation treated as confirmation, no execution guarantee, exact calculator qualifications and fresh source evidence for new-turn citations.

If neither configuration compacts, record **unexercised compaction**, not a tie proving fidelity. If protected input stops safely, record safety-boundary success separately from task noncompletion. If trace export fails, retain local evidence and mark observability blocked; do not fabricate parentage or feedback.

Complete the worksheet: invariant gates, observable actions, semantic pass/fail/uncertain, time/usage/summary count, unknowns and held-back cases. Report observed numerator/denominator and configured repetitions, not universal reliability. Discuss whether any apparent improvement could be ordinary model variation or different initial memory.

Decision choices: adopt with limited evidence and documented remaining gates, reject a regressing variant, or keep baseline because evidence is inconclusive. Classroom comparison is not release authority.

### 6. Explain the handoff (5 minutes)

Ask attendees to name an invariant test, a semantic inspection and one remaining uncertain claim. Ask what further evidence would change their decision. Restore defaults and preserve only permissioned private artifacts. End with review obligations, not a graduation claim that the agent is safe to trade with.

## Gates to keep distinct

| Gate | Required inspection | Not sufficient |
| --- | --- | --- |
| Calculator/capability boundary | Exact decimals, allowlists/schema/all-block limits, actual correlation | Fluent explanation or one green UI rating |
| Memory authority | Per-selection consent, independent local approval, no post-failure writes, declaration semantics | Proposal acknowledgement or summary quotation |
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
