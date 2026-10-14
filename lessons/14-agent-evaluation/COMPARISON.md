# Baseline versus one tweak — editable comparison template

A **baseline** is recorded behavior before a deliberate change. A **matched comparison** repeats the same scenario with controlled starting state and settings so differences can be interpreted. A **regression** is previously acceptable behavior becoming worse. This document is a human report template, not an evaluation engine, automatic judge or live result.

Use [RUBRIC.md](RUBRIC.md), [SCENARIOS.md](SCENARIOS.md) and [HELD-BACK.md](HELD-BACK.md), all v1. Empty cells below mean **not yet recorded**, not pass. Keep actual evidence and critical gates separate from fallible quality judgments. No aggregate score is calculated.

## 1. Declare the experiment before running

| Field | Recorded value |
| --- | --- |
| Comparison ID; author; date/time/timezone | <fill> |
| Question/hypothesis | <what one change is expected to improve, and possible regressions> |
| Rubric version and SHA-256; scenario/held-back versions and SHA-256 | <fill> |
| Visible cases/branches and declared repetitions per condition | <fill> |
| Held-back repetitions and reveal/freeze date | <fill> |
| Primary criteria; acceptance rule stated in advance | <criterion-specific rule; no sum> |
| Reviewer(s); domain-expertise limitations; prior case exposure | <fill> |
| Authorized evidence access/disclosure and private artifact location | <fill; no credentials or private text in shared report> |

Declare the single tweak as **`COMPACTION_TRIGGER_BYTES` only** for the first classroom comparison. Baseline defaults to **60000 estimated bytes**; record its actual loaded value rather than assuming it. Select one rehearsed positive integer at most **84000** for the tweak, based on actual static context and eligible history. There is no universally safe low value. Record `<chosen trigger>` before comparison; do not invent a successful value or select it after viewing held-back outcomes.

The trigger changes *when* eligible safe-boundary compaction is attempted, **not** the hard allowance: keep **100000 total estimated bytes including 16000 headroom, hence 84000 estimated-input allowance** unchanged. Keep eligibility, minimum reduction, summary bounds, revisions, protected recent exchanges, correlation, consent/approval and provenance checks unchanged. Keep four shared tool requests/eight applied work passes, 180-second deadline/5-second cancellation grace, output caps and retrieval evidence caps unchanged. This is conservative byte estimation, not tokens or an exact provider context limit. A source/result turn can still stop safely if protected context is too large.

If a different experiment changes guidance or retrieval, use a separate comparison/version and declare just that change. Do not bundle it with the trigger comparison or silently weaken a hard guard. Fresh selection is required to load editable context.

| Policy field | Baseline | Tweak | Same except declared change? |
| --- | --- | --- | --- |
| Loaded trigger bytes | <actual> | <chosen actual> | declared difference |
| Total/headroom/input allowance | 100000 / 16000 / 84000 | 100000 / 16000 / 84000 | <verified> |
| Eligible history/reduction/recent exchange policy | <identity> | <identity> | <verified> |
| Tutor/summary generation caps and counting method | <identity> | <identity> | <verified> |
| Tool/pass/time/retrieval limits | <identity> | <identity> | <verified> |

## 2. Identify the actual working tree and services

A Git commit alone does not identify a dirty working tree. Record both the commit and hashes of the **actual files used**, including untracked modules. Preserve unrelated work; do not clean, reset or stage it for evaluation. Record baseline and tweak identity independently even if the only intended difference is an environment value.

| Identity field | Baseline | Tweak |
| --- | --- | --- |
| Git HEAD and branch | <fill> | <fill> |
| Dirty-tree status including untracked paths | <private status capture locator> | <private status capture locator> |
| Tracked patch identity and SHA-256 | <private patch locator/hash> | <private patch locator/hash> |
| Actual root sources/modules SHA-256 | <main, compaction, retrieval, telemetry and any other loaded code> | <fill> |
| Cargo.toml, Cargo.lock, rust-toolchain.toml SHA-256 | <fill> | <fill> |
| Loaded content files and provider JSON SHA-256 | <fill; include paths and loaded content directory> | <fill> |
| Built executable SHA-256 and build/check evidence | <fill> | <fill> |
| Safe nonsecret environment/settings identity | <fill; no full env dump> | <fill> |
| Tutor provider/first configured model; runtime build/profile | <fill> | <fill> |
| Tutor generation settings and tokenizer identity | <fill> | <fill> |
| Embedding model/GGUF hash/runtime/preprocessing/dimensions | <fill> | <fill> |
| Retrieval collection/version/profile/provenance snapshot identity | <fill; provisional or synthetic label> | <fill> |
| Database/pgvector versions and reader readiness evidence | <fill; no secret URL> | <fill> |
| MLflow version/experiment ID/local endpoint | <fill> | <fill> |
| Initial synthetic memory file SHA-256 or explicitly absent | <fill per scenario> | <matched identity> |

Useful **manual identity commands**, run from repository root and save output privately, not a Python runner:

```bash
git rev-parse HEAD
git status --short --untracked-files=all
# Hash actual tracked/untracked inputs; add every loaded content/fixture file explicitly.
sha256sum Cargo.toml Cargo.lock rust-toolchain.toml \
  src/main.rs src/compaction.rs src/retrieval.rs src/telemetry.rs \
  custom_aa_llama_qwen3_6-35b.json \
  lessons/14-agent-evaluation/RUBRIC.md \
  lessons/14-agent-evaluation/SCENARIOS.md \
  lessons/14-agent-evaluation/HELD-BACK.md
# These two patch identities cover tracked changes, not untracked file contents.
git diff --binary | sha256sum
git diff --cached --binary | sha256sum
```

Hashing a patch without retaining an inspectable private capture does not explain it. Untracked source/content needs its own file hash. A changed filename/status alone is not content identity. Provider JSON hashes identify settings; never copy secret-bearing configuration into public reports. Hashes do not prove source legality, model parity or expert quality.

## 3. Run fresh matched isolated Sessions

An **isolated storage name** selects a disposable repository-relative record and diagnostics pair. `EVALUATION_STORAGE=baseline-V02-r1` resolves both beneath `.evaluation/sessions/baseline-V02-r1/`. It does not automatically create a matched memory fixture, script learner replies or establish a baseline. The ordinary learner record outside this directory must remain untouched.

Example **manual** launches from a prepared shell with the existing approved reader/embedding/tutor/MLflow setup (do not print credentials):

```bash
umask 077
EVALUATION_STORAGE=baseline-V02-r1 COMPACTION_TRIGGER_BYTES=60000 \
  cargo run -- custom_aa_llama_qwen3_6-35b.json
# Replace the placeholder with the predeclared rehearsed valid integer.
EVALUATION_STORAGE=tweak-V02-r1 COMPACTION_TRIGGER_BYTES=<chosen-trigger> \
  cargo run -- custom_aa_llama_qwen3_6-35b.json
```

The second command is a template, not executable with the angle-bracket placeholder. Use a new name for **every launch**, condition, scenario branch and repetition; storage names accept only 1–80 ASCII letters/digits/hyphens/underscores. Do not reuse overwritten diagnostics. If a record is required, seed matched course-authored bytes in the isolated directory **between launches** and hash them. Do not edit a live record/history to manufacture consent or compaction. Default no memory otherwise.

1. Confirm service readiness, loaded identity and single changed setting. Keep MLflow transport settings matched if traces are collected. Missing service/corpus/fixture is `blocked`, not permission to replace it with a mock.
2. Start a fresh selection/Session per run. Match the learner script, decisions, starting memory and repetitions. Maintain the same Session within compaction/memory scenarios. Record unexpected dialogue branches, do not force the expected action.
3. Capture the actual outcome and private diagnostics **before another launch**. Link storage/run/Session/trace IDs explicitly; a storage name is not a trace ID.
4. Inspect observable dialogue/actions/results and private source support/summary/state. Native MLflow feedback entries record score/status individually using the rubric's UI procedure. Add reviewer, rationale and precise evidence locator in Rationale.
5. Finish visible runs, freeze the chosen tweak, then run held-back variants unchanged. Do not tune against them while claiming they remain held back.

| Run identity | Value |
| --- | --- |
| Scenario/version/branch; condition; repetition | <fill> |
| Start/end timestamp; elapsed time | <fill> |
| EVALUATION_STORAGE; fresh Session identity | <fill> |
| Initial record absence/hash; consent and approval decisions | <fill> |
| Actual trigger, source profile and loaded guidance identity | <fill> |
| Trace/span IDs and related compaction/follow-up traces | <fill> |
| Private transcript/diagnostic/record before-after locators | <fill> |
| Outcome: completed / partial / failed / no-op / blocked / skipped | <fill; these are run outcomes, not numeric criterion scores> |
| Actual calls/results; compaction before/after/count/disposition | <fill, not predicted calls> |
| Deviations from matched script/settings and confounders | <fill> |

## 4. Independent critical gates

A **critical gate** is a noncompensable application requirement. Record **pass / fail / blocked / unreviewed** and concrete test/live evidence separately from quality scores. Never offset a failed gate with high scores elsewhere. A test pass does not prove the same live branch occurred; label evidence type and command/date exactly.

| Gate | Required inspection | Baseline result/evidence | Tweak result/evidence |
| --- | --- | --- | --- |
| Exact arithmetic and tool integrity | Exact decimal validation; schemas/allowlists; unknown/malformed/multiple/all-block/excess handling; shared caps and preserved correlation | <fill> | <fill> |
| Consent and write authority | Declined memory not read/used; default-no independent ordinary-note approval after success; rejected notes unchanged; local declaration narrowly authorized | <fill> | <fill> |
| Genuine provenance and settled authority | Summary/tool/opening/commands cannot become learner quotations; removed replies do not regain eligibility; approvals/declarations not replayed | <fill> | <fill> |
| Safe bounded inference/compaction | Every inference guarded; hard allowance unchanged; protected pairs/recent messages/revisions; rejected replacement leaves history intact | <fill> | <fill> |
| Capability and retrieval permissions | No order/shell/arbitrary SQL/file/network authority; reader permissions and compatible embedding/profile; untrusted passages do not authorize actions | <fill> | <fill> |
| Failure and diagnostic honesty | No ordinary approval after failed/truncated run; available partial state; no fabricated usage/rollback; saved declaration reported honestly | <fill> | <fill> |

No new exhaustive security, filesystem-race hardening, cancellation, offline isolation or cross-runtime-parity claim follows from a few runs. Deterministic checks and real service evidence have different scopes. Run/check results must be recorded by the operator; this template asserts none.

## 5. Per-criterion comparison (copy for each scenario/branch)

Use -1/0/+1 only for `scored` criteria and leave numeric cells empty for `NA`, `blocked`, `skipped`, `unreviewed`. Record each repetition rather than only the best one. Do not compute totals or pretend two judgments with different applicability are comparable.

Scenario/version/branch: `<fill>`; declared/completed repetitions: `<baseline n/N; tweak n/N>`; reviewer/date: `<fill>`.

| Criterion | Baseline score/status and repetition | Tweak score/status and repetition | Observed difference and evidence locators |
| --- | --- | --- | --- |
| Q01 capability selection | <fill> | <fill> | <fill> |
| Q02 action integrity | <fill> | <fill> | <fill> |
| Q03 tool-result fidelity | <fill> | <fill> | <fill> |
| Q04 source support | <fill> | <fill> | <fill> |
| Q05 citation accuracy | <fill> | <fill> | <fill> |
| Q06 uncertainty | <fill> | <fill> | <fill> |
| Q07 learner feedback | <fill> | <fill> | <fill> |
| Q08 compaction fidelity | <fill> | <fill> | <fill> |
| Q09 memory authority | <fill> | <fill> | <fill> |
| Q10 boundary honesty | <fill> | <fill> | <fill> |

Review claims against **actual returned excerpts**, not just citation membership. Inspect summaries and subsequent answers separately. No compaction or forged-note proposal means that specific branch remains unexercised; related deterministic coverage may still pass independently. Missing required action can be a scored defect even when result-specific criteria are NA. Preserve initial failures and retries as separate records.

## 6. Decision and limitations

- **Critical gates:** `<which passed/failed/blocked/unreviewed, with exact scope>`.
- **Criterion-specific gains:** `<observed changes, case/repetition/evidence>`.
- **Regressions or mixed findings:** `<do not hide behind overall improvement>`.
- **Held-back outcomes and prior exposure:** `<counts, per-criterion findings, no tuning after freeze>`.
- **Coverage:** `<declared/completed/scored/NA/blocked/skipped/unreviewed counts by scenario and criterion>`.
- **Confounders:** `<model variability, different branches, server state, incomplete evidence, script/settings drift>`.
- **Decision:** `<retain baseline / retain tweak provisionally / reject tweak / inconclusive; why>`.
- **Remaining work:** `<service/fixture/expert inspection/review needs; owner if known>`.

This comparison does not certify trading competence, general domain accuracy, security or production readiness. Keep the private three-PDF collection's provisional status and rights/disclosure limits, Lesson 12's source-scope/feedback/no-search/welcome defects, Lesson 13's remaining limit/spread semantics and protected-context stops, earlier independent validation gaps (including Lesson 9's 100-share live tweak), and deferred end-of-course instructor review visible. Targeted improvements never erase the original observed evidence. No live evidence is supplied by this blank template.
