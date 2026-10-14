# Native MLflow evaluation dataset catalog — visible v1

## Purpose and boundary

An **evaluation dataset catalog** stores reproducible case definitions in MLflow so a reviewer can find the intended setup and input before inspecting an execution. A **dataset record** is one case definition, not a tutor answer, a completed execution or a rubric rating. **Seeding** copies the supplied definitions into this catalog; it neither runs the tutor nor evaluates it.

Lesson 14 uses native MLflow evaluation datasets with **manual execution and individual human feedback**. The supplied Rust utility is catalog plumbing, not a runner, judge or learner scaffolding exercise. There is no automatic trace-to-record linkage, native evaluation-run orchestration, trace import into datasets, reusable ten-question UI form or aggregate quality score. The tutor main and reference modules remain unchanged by this catalog addition.

Read [SCENARIOS.md](SCENARIOS.md), [RUBRIC.md](RUBRIC.md) and [COMPARISON.md](COMPARISON.md) alongside the catalog. [VALIDATION.md](VALIDATION.md) alone records actual implementation checks and execution evidence. Commands here are instructions, not successful validation claims.

## What is supplied

The default manifest is [`datasets/visible-v1.json`](datasets/visible-v1.json). The root Cargo manifest registers [`seed_dataset.rs`](seed_dataset.rs) as the `seed_mlflow_dataset` example; this lesson is not a separate Cargo package.

The native dataset is named **`lesson14-visible-v1`**, containing ten records **V01–V10**. V04's missing-evidence and separate synthetic conflict probe are branches within the same record, not an eleventh case. Each record carries:

| Field | Meaning |
| --- | --- |
| `inputs.case_id` | Stable visible case identifier |
| `inputs.setup` | Initial state, consent decisions and execution prerequisites to read before launching |
| `inputs.learner_turns` | Authored learner inputs; inspect branch/conditional instructions rather than blindly pasting every turn |
| `expectations.inspection_targets` | Structural and semantic questions for a human reviewer, not an expected transcript |
| `expectations.calculator` (when relevant) | Exact deterministic calculation anchor, not a model answer or semantic grade |
| `tags.rubric_version` | Rubric version for inspection |
| `expectations.applicable_criteria` | Criteria to consider; applicability is not an automatically completed rating |
| `tags.prerequisites` | Required services/state/fixtures; catalog presence does not satisfy them |

Additional input fields keep conditional behavior explicit: V04 has `inputs.separate_probe` for its blocked conflict branch; V09 includes rehearsal/conditional continuation instructions; V10 has `inputs.execution_status=blocked`. Read the entire input object alongside SCENARIOS, not just `learner_turns`. `tags.review_status=unreviewed` is catalog metadata, not an auto-updated judgment.

A **manifest fingerprint** is the SHA-256 hash of the supplied manifest bytes. The dataset is tagged with this fingerprint and its schema version so a reviewer can identify the exact authored catalog. MLflow's server digest is not this fingerprint and must not substitute for it.

All Q01–Q10 judgments begin **unreviewed** and remain separate. Mark an unavailable prerequisite/branch **blocked**, a deliberately omitted execution **skipped**, and an inapplicable criterion **NA** with a reason; none is a numerical zero or a pass. Catalog expectations are not prerecorded findings. A related deterministic test does not fill a live review cell.

V04's conflict branch and V10's instruction-boundary exposure remain **blocked until a separate compatible synthetic fixture collection and constrained selection are prepared and verified**. This seeder does not prepare embeddings, load fixtures, select a different retrieval collection or change the private 511-passage store. V04's ordinary unsupported-price branch can be executed independently, with its actual search/no-search behavior reported. [HELD-BACK.md](HELD-BACK.md) remains unchanged and **unseeded**; do not use held-back cases to tune the visible set or infer their results from it.

## Seed and verify from the repository root

First follow [MLFLOW-SETUP.md](MLFLOW-SETUP.md) to use the existing pinned local server and obtain an **active numeric experiment ID**. The utility uses only `http://127.0.0.1:5000`, without proxies or redirects. Do not restart existing services or replace their configuration to run these commands.

```bash
# Replace 1 with the actual active experiment ID; do not assume 1 exists.
cargo run --example seed_mlflow_dataset -- 1
# Read-only catalog verification; no create/upsert operations.
cargo run --example seed_mlflow_dataset -- 1 --check
```

Record the actual dataset ID, record IDs, manifest fingerprint and verification result from the utility/server. Open http://127.0.0.1:5000, select the associated experiment, then **Evaluation → Datasets → lesson14-visible-v1**. Inspect the case inputs and expectations; a successful health response alone is not catalog verification. The `--check` command checks catalog identity/content, not tutor execution, trace delivery or rubric completion.

The seeder checks the experiment is active and guards against unintended mutation:

- It refuses ambiguous duplicate dataset names, the same name with a different manifest fingerprint/schema identity, edited existing records or extra records.
- Exact existing records are skipped rather than updated. Nonempty observed outputs or imported trace provenance are refused; keep curated execution examples in a separate dataset. An interrupted seed containing an exact subset can be resumed to add only missing records. Read-only verification requires the complete catalog.
- Do not repair refusal by silently deleting records, rewriting tags or upserting edited expectations. Inspect the discrepancy first and preserve any existing review evidence.

Native immutable dataset versions are unavailable in this OSS workflow. MLflow still permits mutation through other clients/UI; the seeder is a guard, not a server access-control or immutability guarantee. Avoid concurrent catalog edits/seeds and verify again before review. The supplied utility intentionally accepts only the reviewed v1 name/schema and fixed manifest path. An intentional case/schema change requires a **new manifest identity and new dataset name**, for example a separately prepared `lesson14-visible-v2`, preserving v1 and its evidence, plus a separately reviewed seeder/path update to support that new version. Renaming an experiment is not a dataset version.

## Manual workflow: predict → run → inspect

1. **Select and predict.** Open a visible record and read its `inputs.setup`, learner turns, expectations and prerequisites. Fix rubric v1, comparison condition, repetitions, services/profile/corpus and matched initial memory before seeing the response. Report blocked branches explicitly; do not fabricate fixtures or tool calls.
2. **Isolate and launch.** Use a unique `EVALUATION_STORAGE` for each scenario/condition/repetition. It isolates both the learner record and overwritten diagnostics. Do not reuse the real record, let baseline-approved memory leak into the variant, or assume an experiment ID isolates disk state. For example, with the provider/retrieval environment from the setup guide:

   ```bash
   export MLFLOW_EXPERIMENT_ID=1 # actual active experiment ID
   export EVALUATION_CASE=V02
   export EVALUATION_COMPARISON=baseline
   export EVALUATION_STORAGE=catalog-v1-baseline-V02-r1
   export COMPACTION_TRIGGER_BYTES=60000
   cargo run -- custom_aa_llama_qwen3_6-35b.json
   ```

   Check that the name is unused or deliberately prepared before launching. Keep a continuing Session only within that case's multi-turn script. The seeder never drives this process.
3. **Execute actual choices.** Enter the authored learner inputs and follow conditional instructions. Consent and local approvals remain actual decisions. V08's intended approval branch is **not permission to approve whatever the model proposes**: inspect the actual category, genuine quotation and note after a successful run; default remains no, and approve only an appropriate addition. Missing/rejected/inappropriate proposals leave that intended branch unexercised or failed, not silently passed. V07 still rejects its local addition. Catalog text cannot authorize writes.
4. **Inspect evidence.** Locate actual traces and local diagnostics. Preserve ordinary answers, returned passages/correlation, memory outcomes and compaction summaries privately before another launch overwrites diagnostics. Compare claims with current evidence and summaries with genuine learner words. No final answer, no search, missing export and no applied compaction stay visible; do not invent an execution to match an expectation.
5. **Annotate manually.** Use trace Notes for the case/execution overview and each individual named rubric feedback entry for that criterion's evidence/rationale. Copy actual identifiers into the following convention; it is human-maintained text, **not an automated MLflow association**:

   ```text
   dataset_id=<actual>; record_id=<actual>; case=V02;
   manifest_sha256=<actual>; rubric=v1; comparison=baseline;
   reviewer=<name>; evidence=<trace/span/request ID and private diagnostic locator>;
   rationale=<observed claim, evidence and limitation>
   ```

   Apply the rubric's numeric mapping only to actual applicable judgments. Keep `<criterion>_status` String feedback for unreviewed/blocked/skipped/NA as needed, and keep all ten criteria separate in the comparison worksheet. Local unauthenticated reviewer text is attribution, not verified identity. One saved rating is not completion of the record.
6. **Compare honestly.** Repeat with matched initial state and a fresh storage name for the approved 60,000→50,000 trigger change; keep the 84,000 hard allowance unchanged. Preserve every attempt, review actual coverage/unknowns, then decide from evidence rather than catalog membership. Reserve held-back cases for the separate later review.

## Privacy and implementation limits

The manifest contains authored synthetic learner inputs and inspection targets, **not private excerpts, expected tutor answers or imported traces**. Seeding is not permission to distribute the books, real retrieved text, personal memory or diagnostic artifacts. Catalog text is redistributable course material; actual trace/feedback evidence can remain sensitive even on a loopback server. Git ignore is not encryption or complete access control. Keep private evidence out of Git and public screenshots; do not insert credentials, provider configuration or provider reasoning into dataset records or human feedback.

The utility uses the pinned MLflow **v3.17.1** native **`/api/3.0/mlflow` dataset endpoints**, which are publicly undocumented REST surfaces. Its source compatibility was checked against official tagged [`service.proto`](https://github.com/mlflow/mlflow/blob/v3.17.1/mlflow/protos/service.proto), [`handlers.py`](https://github.com/mlflow/mlflow/blob/v3.17.1/mlflow/server/handlers.py) and [`sqlalchemy_store.py`](https://github.com/mlflow/mlflow/blob/v3.17.1/mlflow/store/tracking/sqlalchemy_store.py). The protocol serializes create tags and upsert records as JSON **strings**; the store deduplicates records by inputs and merges matching upserts. That behavior explains why the utility checks all existing content before writing instead of relying on case names to prevent replacement. Do not assume these wire details survive a server upgrade.

No Python package/runner, direct SQL writes, automatic judge, trace importer or evaluation execution engine is added. Review any future API/server migration separately. Success here is a verified catalog plus an honest manually linked execution/review trail, not completed scenario coverage or course release. Use [VALIDATION.md](VALIDATION.md) for what was actually checked.
