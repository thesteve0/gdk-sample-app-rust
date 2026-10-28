# Lesson 14 implementation evidence — 2026-10-13

Implementation authorization is for Lesson14 only. This record distinguishes executed checks, observable successes, imperfect outcomes and unexercised branches. It is not a tutor-quality certificate or instructor release review. The course's earlier records remain intact.

## Implemented scope

- Root and complete standalone Lesson14 main/retrieval/compaction/telemetry modules and editable content; earlier references/preparation unchanged. GDK/provider/provider-types stay published exactly alpha.11, RMCP3.4.1, Rust1.94.1 and provider JSON unchanged. Lesson13 content copied without semantic edits.
- Optional native Rust tracing/OTLP HTTP into local MLflow, reusing SDK chat spans/usage. Application spans: learner turn, correlated tool execution, local approvals/storage, consent, declaration, summary/maintenance. Ordinary dialogue/tool data exported; no reasoning metadata projected. No custom Python package/runner, judge, score engine, database or dashboard.
- Isolated record **and** diagnostics via `EVALUATION_STORAGE`, conservative named-directory validation and symlink/nonregular preflight; unchanged consent/approval semantics. This is single-process storage, not atomic protection against concurrent path swaps.
- Official minimal v3.17.1 image/index digest, loopback-only Podman startup with SQLite/artifacts, restart/stop/remove instructions. Docker-compatible configuration, **not independently Docker-tested**.
- Full lesson/notes, editable rubric v1 (10 questions), ten visible cases, three held-back variants, fixed authored compaction rehearsal and comparison worksheet. V10 adversarial live exposure and synthetic-conflict branch are explicitly blocked, not silently mocked.

## Automated/build checks

Final `cargo fmt --check`, `cargo check`, `cargo clippy --all-targets` passed without warnings. `cargo test`: **68 deterministic passed, two live tests ignored by default**. Separately enabled `live_local_smoke` and `live_same_snapshot_sdk_search_batch_obeys_one_evidence_budget` each passed against actual configured reader/Harrier/tutor services. An earlier combined `--include-ignored` invocation passed all70. No `--all-features`, SDK upgrade, root reset or unrelated work cleanup.

**IDE build attempted but blocked:** this repository is not an open JetBrains project; the tool reported only an unrelated `openshift-setup` project. No build of that unrelated project was requested. Cargo build passed. To finish that requested check, open this repository in the IDE and run its build tool; an IDE success is not claimed here.

Root/reference source bytes were compared after synchronization. Prior Lesson13 modules remain available unchanged. Private artifacts are in ignored `.evaluation/`, not committed books/transcripts/databases. Temporary shell/Python/JavaScript probes used during validation were not supplied as an evaluation runner/package.

## Compatibility proof and persistence

Official registry listed v3.17.1 and v3.17.1-full; pulled/tested smaller minimal image. Multiarch index `65aa7a75...a272dfee2`; executed amd64 manifest `e1badda9...dba519a`. Health/UI, SQLite schema and actual Rust OTLP worked. An experiment and real tutor traces survived restart. Native UI was exercised using **Assess → Add feedback → Human feedback → Number**, name `Q03_tool_result_fidelity`, value1 and rationale including reviewer/evidence. Feedback survived restart and was returned by the server. This tests one real named numeric entry, **not an automatic reusable ten-question form**. All ten editable questions use this individual-entry workflow; String statuses remain separate.

Initial calculator proof: exact correlated `$100.00` returned, **no ordinary final answer**. Partial diagnostics survived and application stopped. Trace `tr-8ea19c3290342f0bf7f63c08bcdcdd97` showed inference→tool→inference,15.871s,15122 available tokens. The UI feedback's +1 applies only to the displayed exact result, not a successful final tutor answer; under final rubric, absent answer fidelity is unscored/partial, and completion failure remains separately recorded. Early instrumentation left the trace state OK despite inspection failure; final `err` instrumentation was added. Do not retrofit early evidence as final error-status validation.

Final calculator run: trace `tr-b364488672afb5ddbab2bd05736c3a28`,15.033s,15126 tokens; exact decimal inputs/result, request `ciPWFuCKHdoanrSEf2bd33Ujl6LjGePr`, ordinary final explanation with fees/slippage/gap exclusions and no guaranteed realized ceiling. UI inspection of each SDK chat verified inputs and ordinary outputs; last chat7772 tokens. No duplicate usage-bearing application span. Input is the application native ordinary projection, **not exact formatter wire payload**. Each assembled inference Message is available as an Event; output field shows the last Message if several exist. Summary-specific preparer and effect conversion supply equivalent observation; a final real compaction replay completed and summary trace checks are recorded in the follow-up below.

MLflow inferred dollar costs despite local inference; they are **not measured local expenditure**. Missing usage stays unavailable. Normal returned exits flush/shutdown; kill/crash delivery and durable offline retries are not guaranteed. Header/resource environment overrides are rejected so ambient OTEL headers cannot replace the experiment or deliberately add credentials. No comprehensive privacy-filtering workstream was added.

## Actual classroom before/after findings

One controlled setting: trigger60000 baseline versus50000 tweak; total100000/headroom16000/input84000 and all other guards unchanged. Fresh Sessions, initially absent memory, same authored inputs per matched comparison. These are **small rehearsals, not statistical evidence**. Dynamic model outputs vary and therefore downstream histories differ. Relevant source/content/manifest/lock/executable hashes and dirty-tree snapshot are saved privately under `.evaluation/evidence/`; trace attributes carry file hashes, case, collection and loaded trigger. Captures identify the actual code at each run; final instrumentation refinements occurred afterward and are not retroactively attributed to older traces.

| Case/criterion | Baseline observation | Tweak observation | Finding |
| --- | --- | --- | --- |
| V01 conceptual / Q01,Q02,Q06 | No tool; relevant explanation, gap described too narrowly as between sessions | No tool; relevant but stronger automatic-sale framing and only price uncertainty | Q02+1 both; Q06mixed0, not universal safe wording |
| V02 calculator / Q01,Q02,Q03,Q06 | One correlated `$100.00`, exclusions, complete explanation | Same exact outcome/exclusions | +1 inspected arithmetic/result/qualification; no regression in these runs |
| V03 sources / Q04,Q05 | Three returned excerpts; claimed single-source scope despite a Heitkoetter result present | Three excerpts, Heitkoetter/CFI distinguished; attributed broad volume/liquidity relationship beyond specific returned forex scope | Current IDs valid but semantic source scope remains mixed; Q04/Q05 not blanket+1 |
| V09 compaction / Q08 | Actual replacements66256→56045 and72371→57415 bytes; continued same Session; follow-up retained unconfirmed gap but strengthened activity into detailed analysis | Actual replacements75153→59942 and76329→60248; same Session; uncertainty retained but accepted limit-versus-stop detail omitted in follow-up | Baseline Q08−1 for strengthened scope; tweak Q08 0 for partial scope. Do not claim statistically proven improvement |
| V06/V07/V08 / Q09 | Declined seeded record unused; ephemeral declaration; rejected proposal unchanged | Separate consented approved addition preserved seed and narrow `/learned` saved | Observed authorization+1; not a model/mastery assessment |
| Approved tutor UI / Q10 | Rejected run described proposal simply | Approved run echoed internal proposal instruction text in ordinary tutor answer | Presentation defect remains; deterministic authorization still held |

Compaction r2 earlier baseline also applied68133→54809 and70687→55967; its follow-up incorrectly said no accepted activity. A first long historical replay made an unjustified calculator call with fabricated100-share inputs, returned$50, then correctly refused95579>84000 estimated input. A first50000 tweak run reached the output/token cap before compaction/follow-up; no approval/write. **These failed attempts remain evidence**, not discarded successes. A combined shell timeout during the later long comparison returned before the second child process exited; that process was observed finishing naturally with complete diagnostics/traces. No shared tutor/embedding/retrieval restart was used to make it finish.

Visible rehearsal text contains repeated course-authored hypothetical context; it is not natural learner language or invented understanding. r3 wording declared an actual agreed comparison rather than asserting the tutor had offered it; both matched conditions received identical bytes. Held-back variants were not run or used to tune this change. No held-back success claim or full10-case quality benchmark.

## Memory, terminal, trace coverage and limits

- Consented/rejected live proposal: actual tool, successful ordinary answer, local `n`; seeded record SHA95b46ce6...f2a8f unchanged. Consented/approved separate run: local `y`, append retained seed and synced; `/learned order types` saved its separate declaration. Actual learner quote verified. No instructor root record created/moved/overwritten by evaluation. Ordinary root record was absent before/after.
- Declined sentinel memory: no goal recalled, no sentinel in loaded context; `/learned liquidity` ephemeral; file unchanged. Normal no-export run worked with isolated storage.
- Plain redirect, empty-NO_COLOR and TERM=dumb actual transcripts and complete Markdown diagnostics were ANSI-free. PTY arrow/Enter selection, cyan headings, blue prompts, default live text, green ephemeral declaration and yellow no-op warning passed. First PTY probe waited for the wrong chooser text and was terminated before selection; retry passed. Colored actual compaction/retrieval not freshly rerun; deterministic presentation tests passed.
- Tool errors marked by instrumented `err`; safely rejected unknown/excess requests remain parent warning Events plus correlated saved responses, not fictitious executed tool spans. Actual forced live malformed requests/outages were **not** added as classroom exercises. Tests enforce schemas/correlation/all blocks and limits separately.
- Consent is a short separate root trace with Session identity; local approvals are child work within the turn, approved append nested under its decision. Declaration/maintenance are distinct roots associated by storage/Session/case. Session IDs repeat across launches; use storage name and trace IDs, not Session ID alone.
- SDK usage spans reused; application saved outputs omit thinking/redacted-thinking/metadata. Full private diagnostics retain original metadata. Exact all-path delivery, signal recovery, complete crash durability, private chain-of-thought judging or credential scanning is not claimed.

## Explicitly blocked/unreviewed and inherited gaps

V04 live missing/conflicting evidence, V05 misconception feedback and V10 retrieved instruction exposure were not newly completed. Synthetic adversarial/conflict prose is redistributable but **not installed** in a separate compatible collection; root still fixes the provisional real collection. Implementing a fixture collection-selector/ingestion seam needs a separately settled scope rather than changing the book collection silently. Three held-back cases unreviewed; no deliberate service-outage rehearsal. No expert trading correctness claim, full corpus freeze, GGUF parity, offline network-isolation verification or new model/server deployment.

Earlier source-scope/relationship issues, execution wording defects, accepted-activity strengthening, incomplete memory-aware welcome and protected-long-input stops remain. Two current memory welcomes used the seed activity, but this does not erase earlier failures or prove all welcomes reliable. Prior Lesson9 live100-share tweak and earlier independent validation gaps remain; a separate old replay's fabricated100-share call is not that authorized Lesson9 exercise. Instructor content/teaching/UI review and course release remain pending. Potential Lesson15 and SDK migration are untouched.


### Final feedback follow-up

Native UI also saved final complete-calculator Q03=+1 on `tr-b364488672afb5ddbab2bd05736c3a28` and actual imperfect-compaction Q08=−1 on `tr-86fc99bf599cd7de2e55864f3cc67b3d`, each with reviewer, case/rubric and rationale/evidence. Initial browser automation retries timed out at UI controls; the final native interaction succeeded and server retrieval verified entries. No custom feedback form or programmatic grading service was supplied. Rubric applicability/questions were corrected to the instructor’s exact relevance/capability distinction before final handoff; prior proof feedback is explicitly limited above.

### Final instrumented V09 replay

After final inference-output instrumentation, actual `cargo run --` with the supplied rehearsal, fresh isolated `final-compaction`, trigger50000 and absent memory completed **exit0 in211 seconds**. Automatic compactions **69501→55046** and **69436→55018 estimated bytes** applied. `/learned` was ephemeral and its maintenance honestly no-op. Same-Session follow-up retained unconfirmed stop-market gap uncertainty and accepted hypothetical limit-versus-stop comparison, without added deadline/prices/format. Earlier tutor wording nevertheless falsely said the learner was comfortable with gap/liquidity fills; Q07 concern remains, not a blanket semantic pass.

Summary traces `tr-15f6b6420f40b875715fb8fbb3d4cc6e` and `tr-002e8f722b96dadd5ec0f998c48c3189` had ordinary inputs/outputs, empty capabilities and assembled-output Events. API confirmed metadata; **persisted span details were checked read-only in MLflow SQLite, not via summary UI**. Tutor chat UI was separately checked as above. All12 usage-bearing spans were SDK chats; aggregate86000 input+8746 output=94746 tokens included both summaries, with no duplicate application usage. No tools in this replay, no record before/after, plain stdout/stderr/diagnostics ANSI-free. Final evidence lives in ignored `.evaluation/evidence/final-compaction.*` and persisted MLflow. No tutor/embedding/retrieval restart or upgrade.

## Scope A — native evaluation dataset catalog, 2026-10-13

Instructor authorized catalog seeding plus manual execution/review instructions only. Added authored `datasets/visible-v1.json`, supplied `seed_dataset.rs`, root Cargo example registration, and `DATASETS.md` with aligned lesson/setup/scenario/instructor/README guidance. No tutor, reference module, GDK/provider API, lockfile, service configuration, private retrieval collection or held-back-case changes. SHA-256 comparison confirmed root and Lesson14 reference Rust modules and Cargo.lock unchanged during this addition.

### Actual native server checks

Existing pinned MLflow **3.17.1**, loopback port5000, active experiment **1 (`lesson14-compatibility`)** accepted native API creation and ten records. Dataset **`lesson14-visible-v1`**, ID **`d-c2a3478604b54bb795b2b342a6840f5a`**, manifest SHA-256 **`7af1eeaaa6542c9d789dc33f32535f4c5dfddbb89309a8cb924b57a319e2acdb`**. Native API metadata reported `num_records=10`; dedicated association API returned experiment1. Full readback matched every authored input, expectation and tag for V01–V10. V04's conflict probe is within V04; V10 and the conflict branch explicitly retain blocked prerequisites. No H-case records or private excerpts/observed answers were seeded. Server adds an empty `outputs={}` storage field; that is not a captured tutor response.

`cargo run --example seed_mlflow_dataset -- 1` initially added10; subsequent invocation and `1 --check` added0. Native metadata, record IDs, timestamps and content were identical before/after repeat/check, so this is a real no-update idempotency check, not merely deduplication with fresh timestamps. Wrong association (experiment0), nonexistent experiment999999 and invalid ID were rejected without changing the catalog. `--check` never creates/upserts. Plain redirected, empty-NO_COLOR and TERM=dumb output had no ANSI. Actual PTY check showed cyan heading/green success; separately captured redirected stderr stayed plain. Private check capture: `.evaluation/evidence/scope-a-dataset-checks.json`.

Pinned official `service.proto`, `handlers.py`, `sqlalchemy_store.py` and dataset wrapper source were inspected: publicly undocumented native `/api/3.0/mlflow` dataset routes, JSON-string create tags/upsert records, input-based dedup/merge, separate experiment associations, and no OSS immutable-version facility. Earlier exploratory `/api/2.0/mlflow/datasets/search` returned404; correct3.0 native endpoint worked. Initial Rust build used unavailable reqwest `.query` with current selected features; corrected to URL encoding/validated numeric interpolation without dependency changes, and all final checks passed.

### Automatic checks and limits

Final `cargo fmt --check`, `cargo check --all-targets`, `cargo clippy --all-targets`, `cargo build`, example build and `cargo test --all-targets` passed: **7 new catalog tests plus68 unchanged application tests;2 real-service application tests remained ignored for this subset**. Catalog tests cover authored cases/exact anchor/blocked fixtures, invalid/held-back manifest rejection, arguments, exact-subset resume, idempotency, edited/extra/duplicate record refusal, imported observed-output/provenance refusal, JSON wire fields, identity mismatch, palette and gating. Interrupted-subset recovery and content/mismatch refusal are deterministic tests, **not deliberately induced mutations/failures against the retained native catalog**. IDE build was attempted but blocked because only an unrelated project was open; no IDE success claimed. Scoped whitespace checks passed; full dirty-tree check still reports a pre-existing trailing space in Lesson12 PLAN, preserved as unrelated work.

No fresh tutor scenario execution, trace-to-record link creation, human rubric completion, native evaluation-run orchestration, UI dataset browser automation, container/service restart or new restart-persistence test was performed. API readback establishes native stored catalog contents and experiment association; the browser click path is supplied for instructor inspection, not newly validated UI evidence. SQLite/MLflow restart persistence was previously validated for traces/feedback, not specifically this new dataset. No retrieval fixture preparation, held-back evaluation, expert quality judgment, Docker-specific validation, SDK migration, Lesson15 or release approval. Catalog membership and seven passing seeder tests must not erase earlier semantic findings or blocked/incomplete cases.

## Modularization checkpoint — 2026-10-22

### Scope and preservation audit

Completed behavior-preserving instructional modularization. The complete [source map/reading path](LESSON.md#supplied-modules-and-reading-path) covers main, the six inherited support modules, cohesive retrieval/compaction/telemetry, application tests and module-local child tests. Main retains context assembly, Session/runtime/Effects, concurrent Event consumption, dispatch/allowlists/budgets, machine/cancellation/shutdown lifecycle, genuine-learner-source validation and approval authority. Lesson 14 telemetry lifecycle/`run_with_telemetry`, instrumentation flow and storage-isolation integration remain visible there, alongside inherited evidence/provenance/command/preflight/revision/maintenance boundaries. Terminal/privacy semantics, local approvals, cancellation and Drop ordering were preserved.

The approved `telemetry::identify_current_span` source inventory explicitly adds six production paths: `src/calculator.rs`, `src/chooser.rs`, `src/diagnostics.rs`, `src/learning_record.rs`, `src/presentation.rs`, `src/provider_setup.rs`. No test files are in the hash inventory. The `course.content_sha256` schema and evidence-identification purpose are unchanged; source locations and resulting hashes change. Older stored evidence is not relabeled as current source evidence. Standalone `seed_dataset.rs` and dataset assets are untouched.

The preservation audit found **242 function/test bodies preserved**, apart from the approved `identify_current_span` inventory edit and necessary module paths/visibility. Constants and test names were preserved. An independent audit compared strings, types and attributes, allowing only imports/visibility/`cfg` boundaries and the six approved source-path strings. The missing test-only `DEFAULT_PROVIDER_CONFIG` import in `compaction/tests.rs` was corrected; this was extraction wiring, not a behavior fix. No prompts/content, semantics, dependencies, SDK/toolchain/provider pins or budgets changed.

Root is the ephemeral complete Lesson 14 working copy, including child test directories. Recursive activation is `rm -rf src; mkdir src; cp -R lessons/14-agent-evaluation/src/. src/` from root. No backup/restore or hello-world reset; root finishes matching Lesson 14. Earlier references and unrelated work, including existing Lesson 11 changes, remain preserved.

### Actual checks and blocked live validation

Validation recorded `cargo fmt --check`, `cargo check`, `cargo clippy --all-targets`, `cargo build` and `cargo test --all-targets` passing: **68 deterministic application tests plus seven unchanged seeder tests**. The two real-service tests are normally ignored. Both were explicitly attempted:

```bash
cargo test live_local_smoke -- --ignored --nocapture
cargo test live_same_snapshot_sdk_search_batch_obeys_one_evidence_budget -- --ignored --nocapture
cargo run -- custom_aa_llama_qwen3_6-35b.json
```

**Both live tests failed because `RETRIEVAL_DATABASE_URL` was missing.** The documented launch failed before the chooser for the same missing prerequisite. `day_trading/embeddings/postgres-local/connection.env` was absent; Harrier on port 9999 and MLflow on port 5000 were unavailable. Tutor port 8080 health returned HTTP 200, which does not establish a complete application exchange. No mocks replaced these blocked checks and no service was restarted.

Validation verified plain redirected, empty-`NO_COLOR` and `TERM=dumb` launch-error output and independent stderr terminal gating. This is error-path presentation evidence, not fresh chooser/chat coverage. Scoped documentation links and whitespace/diff checks were also checked for this update.

**Not freshly validated:** tutor chat, export receipt, span parentage, summary generation/continuation, consent/approval/storage receipt, MLflow restart persistence, evaluation feedback or teaching semantics. No fresh IDE/projector review or release approval is claimed. No evaluation-catalog writes, corpus mutation, semantic fix, dependency change or commit occurred in this checkpoint.

All historical observations above remain evidence at their original dates, including absent final answers, fabricated calculator inputs/budget stops, source-scope errors, strengthened/omitted activities and false learner comfort. V04/V05/V10, fixture-dependent branches, held-back cases, expert quality and deferred instructor/content/teaching/UI/release review remain incomplete or blocked as recorded; passing deterministic tests does not resolve them. Request the missing prerequisites before repeating actual-service and conversational/telemetry checks.


Validation-tooling note: archive-preserving temporary source copies initially let Cargo reuse a newer build from another activated lesson. Final validation touched every activated Rust file, then reran the complete checks, both explicit live-test attempts and presentation probes sequentially for Lessons 12, 13 and 14. The reported 40/66/68 application test counts and seven seeder tests per all-target run are from those final rebuilt trees, not cached binaries. Activation instructions now use ordinary recursive `cp -R` rather than preserving old file timestamps. A final isolated Lesson 14 launch produced the expected missing-reader diagnostic under `.evaluation/sessions/refactor14-20261022/`; no learning record was created.


### Service-default validation — 2026-10-28

Added the instructor-supplied reader connection and Harrier/tutor origins as constants at the top of main. Retrieval uses them only when the corresponding environment variable is absent; the main tokenizer-origin comparison uses the same tutor default. Invalid/non-Unicode retrieval environment values still fail instead of silently selecting a default. No process environment is rewritten, and no credential is deliberately added to diagnostics or telemetry. Provider JSON, compatibility checks, reader checks, dependencies and services are unchanged. The plaintext default is a classroom-only exception, not secure credential management.

Locked format/check/all-target Clippy/all-target tests/build passed in temporary staging for each complete reference: Lesson 12 **41**, Lesson 13 **67**, Lesson 14 **69** deterministic application tests, plus seven unchanged seeder tests each. One new test per lesson validates the reader host/role/database/port, local service origins and absent-variable fallback. The active Lesson 12 root independently passed the same checks and byte-matches its reference. Existing staged changes were preserved; no lesson activation replaced the user's root.

With all three override variables unset, both opt-in actual-service tests and the documented launch were attempted for each reference. All stopped with **`embedding runtime identity mismatch`** before the chooser: the existing Harrier service does not satisfy the pinned runtime identity check. This is blocked live validation, not a passed database connection, retrieval, tutor exchange, approval, compaction or telemetry run. No service restart, compatibility relaxation, corpus/catalog write or mock substitution occurred. No fresh IDE/projector/UI or teaching-quality validation is claimed; historical findings and review gates remain.
