# Local MLflow infrastructure — verified 2026-10-13

A **tracking server** stores observable traces and human feedback. It does not decide whether the tutor taught well. An **OTLP exporter** sends OpenTelemetry spans from Rust to that server. This supplied infrastructure is not a Rust scaffolding assignment.

## Pinned official container

The official [registry](https://github.com/mlflow/mlflow/pkgs/container/mlflow) supplied both `v3.17.1` and `v3.17.1-full`. The smaller `v3.17.1` works for SQLite, artifact storage, OTLP/HTTP, trace inspection and human feedback; the full image was not needed or downloaded. Selected multi-platform manifest digest:

```text
ghcr.io/mlflow/mlflow:v3.17.1@sha256:65aa7a75fe37554bd7dd9e539b58e1ec130dac99f27fef9756df316a272dfee2
```

Test host amd64 image manifest: `sha256:e1badda9c35ea87e325357b4e4d8e3ceef571e6bac92dc24b885ef908dba519a`; local unpacked image size about 951 MB. Do not substitute `latest`. Other architectures are pinned by the index but were not executed here.

From repository root:

```bash
bash lessons/14-agent-evaluation/mlflow.sh start
# Docker alternative (same configuration; tested with Podman, not separately Docker):
CONTAINER_ENGINE=docker bash lessons/14-agent-evaluation/mlflow.sh start
curl --fail http://127.0.0.1:5000/health
```

Wait for `OK`; first boot creates schema. Open http://127.0.0.1:5000 . Port publication is loopback-only; the container listens on its internal interfaces. The script never touches existing tutor, embedding or retrieval services. SQLite lives in `.evaluation/mlflow/mlflow.db`, artifacts in `.evaluation/mlflow/artifacts`; neither uses the retrieval database. `.evaluation/` is ignored, not encrypted. Conversations and private excerpts can be present.

Linux/SELinux: `:Z` gives the dedicated bind mount a private container label. Do not mount or relabel your entire home/repository. On non-SELinux Docker installations it is normally accepted; if unsupported remove only the `:Z` suffix. Rootless Podman successfully wrote this directory. For permission errors inspect container UID and mount ownership with your engine; do not solve them with world-writable permissions. This image/setup is classroom local infrastructure, not authenticated production hosting.

## Experiment and Rust launch

Create an experiment in the UI, or use this official server API:

```bash
curl --fail -H 'Content-Type: application/json' \
  -d '{"name":"lesson14-baseline"}' \
  http://127.0.0.1:5000/api/2.0/mlflow/experiments/create
```

Use the returned numeric ID (do not assume `1`). Set the existing private reader connection as in [Lesson 12 setup](../12-specialized-retrieval/DATABASE-SETUP.md):

```bash
source day_trading/embeddings/postgres-local/connection.env
export RETRIEVAL_DATABASE_URL="$READER_DATABASE_URL"
export MLFLOW_EXPERIMENT_ID=1 # replace with your returned ID
export EVALUATION_STORAGE=baseline-V02
export EVALUATION_CASE=V02
export EVALUATION_COMPARISON=baseline
export COMPACTION_TRIGGER_BYTES=60000
cargo run --
```

`EVALUATION_STORAGE` is a simple 1–80 character ASCII name (letters/digits/hyphen/underscore), not a path. Both record and overwritten diagnostics resolve under `.evaluation/sessions/<name>/`. Use a fresh name or deliberately prepare matched starting memory before each launch. Working-directory changes alone do not isolate storage. All consented record reads, approved appends and `/learned` writes use this path. Do not run simultaneous processes with one storage name.

No telemetry machinery appears in Tutor/Learner dialogue. With the ID absent, export is disabled; storage isolation is independent. This custom app selects a fixed local endpoint `http://127.0.0.1:5000/v1/traces`, numeric `x-mlflow-experiment-id` header, no proxy/redirect client and traces only. It does not implement the full Goose CLI's environment-variable exporter. Do not assume arbitrary `OTEL_*` settings work here.

## Native evaluation dataset catalog (separate from execution)

A **dataset catalog** stores authored evaluation case definitions; it does not run or rate the tutor. After creating an active numeric experiment, follow [DATASETS.md](DATASETS.md) for the supplied native Rust seeder and manual review workflow. From the root:

```bash
# Replace 1 with your actual active numeric experiment ID.
cargo run --example seed_mlflow_dataset -- 1
cargo run --example seed_mlflow_dataset -- 1 --check
```

The default manifest `lessons/14-agent-evaluation/datasets/visible-v1.json` seeds `lesson14-visible-v1`, ten records V01–V10. V04's conflict branch remains inside V04; V04 conflict/V10 prerequisites stay blocked until separately prepared. Held-back cases are not seeded. `--check` is read-only catalog verification, not scenario completion. The seeder validates active experiment/content identity, skips exact existing records and resumes an exact partial seed, but refuses duplicate names, changed manifests and edited/extra records. Record actual dataset/record IDs and manifest SHA-256 for manual Notes/feedback rationale. Intentional version changes need a new dataset name/manifest; the OSS server has no native immutable version guarantee.

This utility fixes `http://127.0.0.1:5000`, no redirects/proxy, and uses v3.17.1 publicly undocumented native `/api/3.0/mlflow` dataset endpoints checked against tagged server source. It does not use a Python package or direct SQL writes. Do not substitute another server version or restart services as an implicit setup step. No automatic trace-to-record association, trace import, native evaluation-run orchestration or judge is promised. All criteria start unreviewed; consult [VALIDATION.md](VALIDATION.md) for actual checks.

## Native feedback UI (actually tested)

Choose experiment → Traces → a trace → root span → **Assess** → **Add feedback** → **Human feedback**. Enter the rubric criterion name, select **Data Type → Number**, enter `-1`, `0` or `1`, write rationale, then **Create**. Include `dataset_id=<actual>; record_id=<actual>; case=V02; manifest_sha256=<actual>; rubric=v1; comparison=baseline; reviewer=<name>; evidence=<trace/span/request ID and private diagnostic locator>; rationale=<observation>` in the rationale for catalog-backed cases. Copy actual identifiers after catalog verification; these are manual annotations, not an automatic dataset association. Local unauthenticated UI attribution is not a verified reviewer identity; record it explicitly. Repeat for applicable criteria; the name field supports reuse, but there is no supplied reusable ten-question form. Use [RUBRIC.md](RUBRIC.md) alongside the UI.

For not applicable/blocked/skipped/unreviewed, use separate **String** feedback named `<criterion>_status` and put status/evidence/reviewer in rationale. Do not enter numerical zero for these. Use trace Notes for case/comparison overview if helpful; MLflow owns feedback storage. Do not click **Detect Issues**, **Analyze trace** or **LLM scorer feedback** as part of this lesson: no automatic or hosted judge is enabled by our code.

UI onboarding tips may block clicks: dismiss **Got it** first. Root inputs/outputs are ordinary-message projections, not exact provider wire payloads. Expand tool spans to match `course.request_id`, arguments and result. SDK chat attributes supply model/usage, and preparation adds observable input. Effect conversion inside the inference span records each ordinary assembled output as an Event and its last Message as the output field; parent saved-message Events and turn output independently record saved state. Missing usage is unavailable, not zero. MLflow inferred dollar costs were observed despite a local model: these are not measured local expenditure and must not be presented as billing evidence.

## Shutdown, persistence and cleanup

```bash
bash lessons/14-agent-evaluation/mlflow.sh stop
bash lessons/14-agent-evaluation/mlflow.sh start
bash lessons/14-agent-evaluation/mlflow.sh restart
bash lessons/14-agent-evaluation/mlflow.sh logs
```

After restart reopen the same trace and feedback. Experiments, real traces and a numeric human feedback entry survived restart in validation. `remove` removes only the container; next start reuses the directory. To erase tracking data, stop/remove first, verify your backup/export needs, then manually delete **only** `.evaluation/mlflow/`. To erase one disposable conversation delete **only** `.evaluation/sessions/<name>/` between launches. Neither action touches `learner-record.md` or the retrieval store. Never commit database, books, excerpts, screenshots or transcripts.

## Troubleshooting and limitations

- Port 5000 busy: stop and ask before changing an existing service; this supplied Rust endpoint is fixed.
- No traces: verify health, experiment ID and orderly application exit. The batch exporter runs in a background thread; Rust explicitly flushes/shuts down on normal exits, including returned application errors. Export initialization/flush failures return errors; mid-run export failures may be SDK diagnostics, not a durable local retry queue.
- Ctrl-C, SIGKILL, crash or power loss can lose unfinished/queued spans. No interrupted-run delivery guarantee or signal-handler addition is claimed. Existing complete Markdown diagnostics remain separate.
- A span completed without exception is not automatically an acceptable answer. Human rubric findings and deterministic gates stay separate; application inspection errors are marked errors in final instrumentation.
- No source-support expert review, converted-runtime parity, final corpus freeze or production readiness is established by traces.

## Verified sources

Read 2026-10-13: official [Goose integration](https://mlflow.org/docs/latest/genai/tracing/integrations/listing/goose/), [OpenTelemetry](https://mlflow.org/docs/latest/genai/tracing/opentelemetry/), [ingestion](https://mlflow.org/docs/latest/genai/tracing/opentelemetry/ingest/), [attribute mapping](https://mlflow.org/docs/latest/genai/tracing/opentelemetry/attribute-mapping/), [human feedback](https://mlflow.org/docs/latest/genai/assessments/feedback/), and [Goose observability variables](https://goose-docs.ai/docs/guides/environment-variables). Goose's integration describes its CLI, not automatic native GDK assembly. Cached published alpha.11 `goose-agent/src/inference.rs` establishes existing chat spans/usage; `machine.rs` and `tool.rs` do not supply equivalent application spans. Telemetry APIs were checked against locked crate source (`opentelemetry-otlp` 0.32.0, SDK 0.32.1, tracing-opentelemetry 0.33.0). No SDK/provider/toolchain pins changed.

Dataset wire compatibility also uses the official tagged v3.17.1 [service protocol](https://github.com/mlflow/mlflow/blob/v3.17.1/mlflow/protos/service.proto), [handlers](https://github.com/mlflow/mlflow/blob/v3.17.1/mlflow/server/handlers.py) and [SQLAlchemy store](https://github.com/mlflow/mlflow/blob/v3.17.1/mlflow/store/tracking/sqlalchemy_store.py): tags/records are JSON-string wire fields and record upserts deduplicate by inputs. See DATASETS for identity/mutation guards; this is not a public stable REST contract or a validation result.
