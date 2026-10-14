# Harrier test embeddings in local PostgreSQL

Separately authorized instructor preparation on 2026-10-12. Root and learner Lesson 11 are unchanged; this does **not** implement Lesson 12. The selected model is the already downloaded **Microsoft Harrier OSS v1 0.6B**, not the 270M or 27B variant. This pipeline loads the latest saved **511 draft chunks**, not either proposed page-overlap/section-aware policy. Remaining layout/section/Heitkoetter-margin review is not silently accepted.

## Files

- `postgres.sh`: Docker-compatible start/stop/status/SQL; use `CONTAINER_ENGINE=podman` for the locally cached image.
- `sql/schema.sql`: collection/document/passage DDL, `halfvec(1024)`, cosine HNSW index, reader role.
- `sql/query.sql`: parameterized nearest-neighbor query used by Rust.
- `sql/inspect.sql`: counts, dimensions/norms, source review status and reader privilege checks.
- `src/bin/harrier.rs`: Rust `init`, atomic `load`, and `query` commands.
- `harrier_server.py`, `requirements-harrier.txt`, `harrier-service.sh`: retained historical Transformers reference runtime; **not used by the active Rust load/query path**.

## Model and serving contract

Model: `microsoft/harrier-oss-v1-0.6b`, revision `f9b9dc8d367d443f2479d27aa5d8d2850c0774ee`. The publisher's local model card/config specifies last non-padding token pooling, L2 normalization, 1,024 dimensions and 32,768 tokens. Retrieval queries use exactly:

```text
Instruct: Given a web search query, retrieve relevant passages that answer the query
Query: <question>
```

Passages have **no instruction prefix** and no chat template. Rust prepends the query instruction itself, checks the llama.cpp tokenizer count with `POST /tokenize`, then sends each prepared string to `POST /v1/embeddings`. The returned `usage.prompt_tokens` must equal that preflight count. The active slot context limit is read from `/props` and oversize inputs fail before inference. Rust checks the service model alias, dimensions, F16 weights, pinned build and local GGUF SHA-256, then validates finite unit-normalized vectors. Stored counts are tokenizer counts, not word estimates.

**Active endpoint: `HARRIER_URL=http://127.0.0.1:9999` (the default).** This is now the llama.cpp service, not the Python `/embed` API. Model alias is `TheSteve0/harrier-oss-v1-0.6b-GGUF`, GGUF SHA-256 `02b8bc790139170d3710b411777342945c0b36d35fdaf0e24684c2d3e135a23b`, and verified build `b11429-d81235049` (installed version 0.6.0). The GGUF is byte-identical to the prepared official-checkpoint F16 conversion. The client deliberately pins runtime identity; a new model/build requires an explicit profile decision, not silent reuse of vectors.

The original Transformers collection **`harrier-test-511-v1` is retained**, but the active client refuses its old embedding profile. All passages are re-embedded into **`harrier-llamacpp-test-511-v1`** so query and document vectors use the same runtime. This does not claim equivalence with the older bfloat16 Transformers embeddings. PostgreSQL still stores `halfvec(1024)`. Similarity is not a probability of relevance, truth or mastery.

## Start locally

Prerequisites: pinned Rust toolchain, Docker or Podman, `openssl`, the verified installed `llama-server` build and the prepared Harrier F16 GGUF. **Python/uv are not required for the active load/query workflow.** The Rust adapter checks the local file named by `/props`, so these instructions assume the client and model server run on the same machine. No external inference fallback exists; proxies/redirects and non-loopback HTTP/database hosts are rejected.

From the repository root:

```sh
CONTAINER_ENGINE=podman embedding-code/postgres.sh up

# If port 9999 already serves the matching model, use it rather than launching twice.
# Otherwise, leave this process running in terminal A:
llama-server \
  -m "$PWD/day_trading/model-preparation/models/harrier-0.6b/harrier-0.6b-F16.gguf" \
  --alias TheSteve0/harrier-oss-v1-0.6b-GGUF \
  --embeddings --pooling last --embd-normalize 2 \
  -ngl 99 -c 8192 -np 8 -fa on -b 2048 -ub 2048 \
  --host 127.0.0.1 --port 9999
```

The flags reproduce the inspected server's model/pooling/context/offload configuration, but restrict the bind address to loopback. The model is loaded from a verified local file, not downloaded during serving. GPU/backend availability can differ on another machine. With `-c 8192 -np 8`, `/props` reports **1,024 tokens per slot**, not 8,192 per request or the model's 32,768 training context. Test passages fit this bound; larger future inputs require a deliberately changed context configuration and validation.

Check the existing service:

```sh
curl --noproxy '*' --fail --silent --show-error http://127.0.0.1:9999/health
curl --noproxy '*' --fail --silent --show-error http://127.0.0.1:9999/v1/models
```

Health reports `status: ok`; `/v1/models` must identify the alias above and `meta.n_embd: 1024`. Health alone does not verify pooling/model correctness; the Rust adapter performs additional checks. **The instructor's existing process was started with `--host 0.0.0.0`**: the client connects over loopback, but that does not make the server private. Restart it yourself with the loopback command above when appropriate; this change does not automatically restart your process.

Database image is digest-pinned `docker.io/pgvector/pgvector@sha256:60cddfb66e0c6c077f33a81c974ad12f688f2d26e055d38cd696004af7911cd8` (PostgreSQL 18 / pgvector 0.8.7). PostgreSQL 18 uses the named volume `gdk-harrier-pg18` mounted at `/var/lib/postgresql`. Stops preserve data. `postgres.sh up` reuses its labeled container and reapplies idempotent DDL; it is not a schema migration engine. `PGVECTOR_PORT` may be changed only before first initialization (saved connection settings persist). Docker and Podman have separate image/volume stores.

## Load the saved test corpus

In another terminal:

```sh
umask 077
source day_trading/embeddings/postgres-local/connection.env
export HARRIER_URL=http://127.0.0.1:9999
RUN="$PWD/day_trading/embeddings/pdf-draft-1791825122-21376647"
cargo run --locked --manifest-path embedding-code/Cargo.toml --bin harrier -- \
  load --run "$RUN" --collection harrier-llamacpp-test-511-v1
```

Loading verifies saved artifact hashes, passage hashes/identities and counts before embedding. Original complete document/chunk metadata, source checksums, physical pages, raw-fragment provenance and draft review flags remain in JSONB. Collection publication is one PostgreSQL transaction: no partially loaded collection is searchable after failure. Repeating an identical load returns already loaded rather than creating duplicates; changing the manifest or model/profile under the same collection name fails. Choose a new collection name for a different corpus/policy. Failed pre-transaction embedding work may need to be recomputed; this is not an exactly-once/resumable ingestion guarantee. Concurrent collection publication is locked.

`init` optionally applies the same schema through the Rust PostgreSQL connection; the startup script already does so. Only the private admin connection should load/init. `query` works with the read-only connection:

```sh
DATABASE_URL="$READER_DATABASE_URL" \
cargo run --locked --manifest-path embedding-code/Cargo.toml --bin harrier -- \
  query --collection harrier-llamacpp-test-511-v1 \
  --text 'How can I limit the loss on a day trade with a stop order?' --top-k 5

CONTAINER_ENGINE=podman embedding-code/postgres.sh sql < embedding-code/sql/inspect.sql
CONTAINER_ENGINE=podman embedding-code/postgres.sh status
```

Queries return JSON with similarity, full passage text and source provenance. **Results contain private book text**: redirect only to a protected ignored path, not a public log or tracked fixture. Model/collection mismatches fail instead of silently searching incompatible vectors. HNSW is approximate; use exact-search comparison during later retrieval evaluation. For small collections PostgreSQL may legitimately choose a sequential scan. Collection filters and approximate scans need recall evaluation as more collections are added.

## Privacy and lifecycle

Generated random admin/reader passwords are in ignored, mode-0600 `day_trading/embeddings/postgres-local/` files. They are not printed; do not commit/share them. Named volume data, embeddings and excerpts remain private derived material. Ignore rules are not encryption or source redistribution rights. Avoid shell tracing (`set -x`) around credential commands. Local machine/container-admin access can inspect environment/data; this is a local development deployment, not hardened multi-tenant infrastructure. The reader role receives SELECT but no INSERT/UPDATE/DELETE privileges. Admin is for preparation only.

```sh
CONTAINER_ENGINE=podman embedding-code/postgres.sh stop
```

No automatic volume-deletion command is supplied. Retain or back up data according to your source rights and retention policy. The Rust loader sends individual bounded requests to the llama.cpp service. Stop a foreground llama.cpp process with Ctrl-C; no process is automatically killed by this workflow.

## Checks and evidence

```sh
cargo fmt --manifest-path embedding-code/Cargo.toml --check
cargo check --locked --manifest-path embedding-code/Cargo.toml
cargo clippy --locked --manifest-path embedding-code/Cargo.toml --all-targets
cargo test --locked --manifest-path embedding-code/Cargo.toml
```

Current llama.cpp live outcomes are recorded below after validation; the older Transformers evidence is retained as history. A functioning similarity query is not accepted retrieval quality or final corpus/model benchmarking.



### llama.cpp switch validation — 2026-10-12

- The existing user's llama.cpp service on **port 9999** was used without restart. Model alias, F16 GGUF checksum and build `b11429-d81235049` were checked. The client accepts either the HF cached snapshot path or a copied local GGUF with the same verified bytes.
- All **511 passages re-embedded and loaded** into **`harrier-llamacpp-test-511-v1`**; **56/273/182** per book. Actual llama.cpp token counts **3–522**, below the current **1,024-token slot limit**. `/tokenize` preflight and `/v1/embeddings` usage counts agreed for every input; no truncation was accepted. Stored halfvec norms **0.999964–1.000039**.
- Original **`harrier-test-511-v1` remains at 511 entries**. Searching it with the active client fails on incompatible profile. Repeating the new collection's load returns already loaded without duplicates/re-embedding. All 511 draft flags survived and PostgreSQL reported **zero passage-content hash mismatches**.
- Three live read-only five-result queries passed with matching source hashes/metadata: stop-loss risk ranked beginners page 8 first (approximately **0.6760**), journaling/emotions ranked beginners page 12 (**0.6754**), liquidity ranked CFI page 41 (**0.5381**). These are smoke checks, not a parity benchmark or expert relevance assessment.
- **32 automatic preparation Rust tests passed**, plus explicitly enabled private-corpus integrity and live synthetic llama.cpp query/passage/oversize-rejection tests (**34 total**). Formatting, Cargo check, all-targets Clippy and IDE build passed. The live test verifies oversize rejection before embedding and the matching normalized response contract. Root remains unchanged; its previously recorded checks remain historical, not a fresh tutor run.
- A first attempt to run both opt-in tests omitted `HARRIER_TEST_RUN`, so the real-corpus test correctly failed. Both passed when rerun with the explicit run path. No pipeline failure or partial collection resulted.

No automatic server stop/restart, remote inference, corpus freeze or Lesson 12 learner integration occurred. The server still binds **0.0.0.0** as started by the user; localhost client access does not remove network exposure. Use the documented loopback bind when restarting. Cross-runtime parity, final retrieval quality and full-context inference remain unclaimed.

### Historical Transformers validation — 2026-10-12

- Local Podman startup and repeat startup passed; server reports **PostgreSQL 18.6 / pgvector 0.8.7**. Docker-compatible script syntax is supplied; a fresh Docker-engine deployment was not separately run.
- At that validation, the reference Harrier service was running on **127.0.0.1:8091**; PostgreSQL is running on **127.0.0.1:55432**. Background server PID/log are in ignored `day_trading/model-preparation/logs/harrier-server.pid` and `harrier-server.log`. To stop this background instance: `kill "$(cat day_trading/model-preparation/logs/harrier-server.pid)"`. Normal documented serving is foreground. Do not launch a second instance on the same port.
- **33 input artifacts / 3 documents / 511 chunks** verified and embedded without truncation; atomic collection **`harrier-test-511-v1`** published. Stored document counts: **56 beginners, 273 Heitkoetter, 182 CFI**. Actual passage token counts **3–522**. Float16 stored vector norms **0.999931–1.000083**. All 511 draft flags retained and PostgreSQL SHA-256 checks found **zero passage hash mismatches**.
- Repeating the same load returned already loaded with 511 passages, without another embedding pass. Rust `init` and idempotent schema initialization passed.
- Live reader queries for stop-loss risk, journaling/emotions, and bid-ask/liquidity returned five ordered results with full matching content hashes/metadata and physical-page provenance. The stop-loss query ranked beginners page 8 first (cosine similarity approximately **0.6760**); journaling/emotions ranked beginners page 12 (**0.6751**); liquidity ranked CFI page 41 (**0.5406**). A final three-result journal query also passed. These are **smoke checks**, not relevance labels, expert validation, or equal-budget retrieval evaluation. The stop-loss source oversimplifies guaranteed execution/loss limits; successful retrieval does not certify that claim.
- Reader SELECT passed; attempted schema initialization and INSERT were rejected. INSERT rollback left zero forbidden rows. HNSW exists and a forced-index EXPLAIN uses it; the normal small-corpus planner may prefer exact sequential search. No production recall/latency benchmark is claimed.
- **26 automatic Rust preparation tests passed**, plus the explicitly enabled private-corpus integrity test (27 total). **22 embedding-server tests** and **3 chunk-comparison tests** passed. Formatting, Cargo check, all-targets Clippy and IDE build passed. The unchanged root separately passed formatting/check/Clippy and its **26 tests**; no fresh tutor live run was performed.
- Runtime: Python **3.11.15**, torch **2.11.0+cpu**, Transformers **4.57.6**, tokenizers **0.22.2**. Query/passage vectors matched independent checkpoint inference exactly on synthetic samples (maximum absolute difference **0.0**); four synthetic tokenizer samples matched upstream token IDs, with the unrelated Mistral regex rewrite explicitly disabled. Live oversize/batch rejection passed. Full 32,768-token inference was not performed; saved passages require at most 522 tokens.

Remaining gates: final layout/source/section acceptance, outstanding Heitkoetter margin cleanup, alternative-policy comparison, full PDF/EPUB corpus, expert evidence-use evaluation, GGUF/llama.cpp compatibility, rights for learner distribution, and separately authorized Lesson 12 integration. No corpus freeze, external private-text upload, root-source change or later lesson implementation occurred.
