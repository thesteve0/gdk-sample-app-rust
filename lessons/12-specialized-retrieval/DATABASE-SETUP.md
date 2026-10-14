# Lesson 12 — Start and populate the test database

## Scope and prerequisites

This is **instructor-side preparation** for Lesson 12, not the learner application's retrieval integration. Root now runs the separately authorized Lesson 12 integration; see [LESSON.md](LESSON.md) for learner-runtime setup. A **collection** is a named set of passages embedded with one compatible model/profile. The supplied test collection is `harrier-llamacpp-test-511-v1`: **511 draft passages** from three PDFs, embedded with **Microsoft Harrier OSS v1 0.6B**. Draft review flags and source provenance are retained; loading does not certify source accuracy or freeze the final corpus.

Run the following Bash commands **from the repository root**. Required:

- The repository's pinned Rust toolchain and Cargo.
- Docker or Podman, `openssl`, and the verified `llama-server` build (plus `curl` for the health check). Python/uv are not needed for this workflow.
- The prepared Harrier F16 GGUF under `day_trading/model-preparation/`, with SHA-256 `02b8bc790139170d3710b411777342945c0b36d35fdaf0e24684c2d3e135a23b`. See [model/runtime details](../../embedding-code/RETRIEVAL.md#model-and-serving-contract).
- The saved private corpus directory `day_trading/embeddings/pdf-draft-1791825122-21376647/`, including its manifest and all source/chunk artifacts. These ignored private files are **not included in a Git clone**. Obtain authorized local copies from the instructor; do not substitute another run under the same collection name.

The active runtime is **llama.cpp on port 9999** using the converted F16 GGUF. The old Transformers collection `harrier-test-511-v1` remains intact, but is not compatible with the new client profile. Load the separate collection below; do not search old document vectors with the new query runtime. No private text is sent to an external inference service. Cross-runtime parity is not claimed.

## 1. Start PostgreSQL and create the schema

Choose one container engine and use it consistently for every database command:

```sh
# Use docker on a Docker installation:
export CONTAINER_ENGINE=docker

# On the instructor machine, use Podman instead (the image/data were tested there):
# export CONTAINER_ENGINE=podman

embedding-code/postgres.sh up
```

The script starts or reuses container `gdk-harrier-postgres`, waits for readiness, and applies [schema.sql](../../embedding-code/sql/schema.sql). It creates the `harrier` database, pgvector extension, `retrieval` schema, collection/document/passage tables, cosine HNSW index, and a read-only `retrieval_reader` role. Startup also revokes PUBLIC temporary-table privileges on this dedicated database so the reader cannot create temporary tables. No separate manual DDL step is necessary.

PostgreSQL is bound to **`127.0.0.1:55432`**, with **1,024-dimensional `halfvec`** embeddings. The digest-pinned image supplies PostgreSQL 18 and pgvector 0.8.7; the validated server reported PostgreSQL 18.6. Data persists in named volume `gdk-harrier-pg18`. Docker and Podman have separate volumes/image stores: switching engines does not access the same loaded database.

First startup generates local random passwords in ignored owner-only files under `day_trading/embeddings/postgres-local/`. Do not print, commit, or share these files, and do not enable shell tracing around credential commands. Keep the saved credentials with the matching database volume. The script is for local preparation, not hardened public deployment.

## 2. Start the matching Harrier embedding service

A **query embedding** is the vector representation of a search question. Both loading passages and embedding questions require the matching Harrier service, so keep it running while using the CLI.

If the matching llama.cpp service already runs on port 9999, keep using it. Otherwise, in terminal A from the repository root:

```sh
llama-server \
  -m "$PWD/day_trading/model-preparation/models/harrier-0.6b/harrier-0.6b-F16.gguf" \
  --alias TheSteve0/harrier-oss-v1-0.6b-GGUF \
  --embeddings --pooling last --embd-normalize 2 \
  -ngl 99 -c 8192 -np 8 -fa on -b 2048 -ub 2048 \
  --host 127.0.0.1 --port 9999
```

Leave the foreground service running. This example uses the verified installed llama.cpp **0.6.0 / build `b11429-d81235049`** and local GGUF, with no serving-time download. GPU/backend availability may differ on another machine. This example configuration has **1,024 tokens per slot** (the existing service inspected during integration reports 2,048), sufficient for this test set, not 8,192 tokens per request. Larger future inputs must fail rather than silently truncate.

Check an existing service instead of launching a second instance:

```sh
curl --noproxy '*' --fail --silent --show-error http://127.0.0.1:9999/health
curl --noproxy '*' --fail --silent --show-error http://127.0.0.1:9999/v1/models
```

Expect health `status: ok`, model alias `TheSteve0/harrier-oss-v1-0.6b-GGUF`, and `meta.n_embd: 1024`. Rust checks `/props`, model/build identity and the local GGUF hash as well; health alone is insufficient. The client and server must run on the same machine because the client verifies the file named by `/props`.

**Embedding path:** Rust adds Harrier's required instruction to the question, counts tokens through `POST /tokenize`, and sends the prepared question to **`POST /v1/embeddings`**. Passage input receives no query prefix. The returned vector is passed to PostgreSQL for similarity search. PostgreSQL does not embed question text. The service must be configured for **last-token pooling and L2 normalization**; never use a chat-completion endpoint or chat template here.

The instructor's inspected process uses `--host 0.0.0.0`. Connecting from localhost does not restrict its network exposure. This guide uses `--host 127.0.0.1`; restart the existing process yourself when appropriate. The client change does not restart it.

## 3. Embed and load the 511 entries

In terminal B, from the repository root:

```sh
umask 077
source day_trading/embeddings/postgres-local/connection.env
export HARRIER_URL=http://127.0.0.1:9999
RUN="$PWD/day_trading/embeddings/pdf-draft-1791825122-21376647"

cargo run --locked --manifest-path embedding-code/Cargo.toml --bin harrier -- \
  load --run "$RUN" --collection harrier-llamacpp-test-511-v1
```

The sourced `DATABASE_URL` is the **admin connection**, needed for loading. The loader verifies artifact hashes and passage identities, embeds all passages, then inserts the collection in one transaction. Each entry stores passage text, embedding, tokenizer count, source checksums, physical-page locations, and complete original chunk metadata, including raw-fragment provenance and draft flags.

Expected first-load completion:

```text
Loaded 3 documents and 511 draft passages transactionally.
```

An identical repeat load is safe and does not duplicate or re-embed entries:

```text
Collection already loaded: 511 passages.
```

Changing the corpus or embedding profile under the same collection name is rejected. Use a new collection name for a deliberately different test set. A failed embedding pass publishes no partial collection, but may need recomputation when retried. Do not run the PDF extraction command to recreate this exact collection: it creates a new run rather than restoring the saved one.

## 4. Verify the stored entries

In terminal B, retain the engine selection from step 1 (export it again if this is a new shell):

```sh
embedding-code/postgres.sh status
embedding-code/postgres.sh sql < embedding-code/sql/inspect.sql
```

Expected stored counts:

| Source | Entries |
| --- | ---: |
| `DayTrading.com-PDF-Day-Trading-For-Beginners.pdf` | 56 |
| `The-Complete-Guide-To-Day-Trading.pdf` | 273 |
| `The-Complete-Guide-to-Trading.pdf` | 182 |
| **Total** | **511** |

The declared/stored collection counts should both be 511, dimensions 1024, and corpus status `draft_pending_layout_and_tokenizer_review`. The llama.cpp live load measured **3–522 tokens per passage**, below both the example 1,024-token slot limit and the inspected running service’s 2,048-token slot capacity. The original Transformers collection remains separate; `inspect.sql` shows both. Reader privilege checks should show SELECT allowed and INSERT/UPDATE/DELETE denied.

## 5. Run a read-only similarity query

Use the reader connection only for this command, leaving the shell's admin connection unchanged:

```sh
DATABASE_URL="$READER_DATABASE_URL" \
cargo run --locked --manifest-path embedding-code/Cargo.toml --bin harrier -- \
  query --collection harrier-llamacpp-test-511-v1 \
  --text 'How does liquidity affect the bid-ask spread?' --top-k 5
```

The CLI embeds the question with the matching query profile and executes [query.sql](../../embedding-code/sql/query.sql) with bound parameters. Expect a JSON object containing collection/model/profile identity and a `results` array with similarity, passage text, source filename/checksum, physical pages, and chunk metadata.

**Query output contains private book excerpts.** If saving it, use an ignored protected location, for example:

```sh
DATABASE_URL="$READER_DATABASE_URL" \
cargo run --locked --manifest-path embedding-code/Cargo.toml --bin harrier -- \
  query --collection harrier-llamacpp-test-511-v1 \
  --text 'Why should I keep a trading journal?' --top-k 5 \
  > day_trading/embeddings/postgres-local/journal-query.json
```

Similarity is not a probability of relevance or proof of correctness. HNSW is approximate; PostgreSQL may prefer a sequential scan for this small dataset. These query checks do not replace retrieval/evidence-quality evaluation.

## 6. Stop and restart without losing entries

Stop the foreground Harrier server with **Ctrl-C** in terminal A. Stop PostgreSQL with:

```sh
embedding-code/postgres.sh stop
```

Stopping preserves the database volume. On a later launch, repeat PostgreSQL `up`, start or verify the Harrier service, and source `connection.env` in the query shell. **No reload is needed for an existing populated volume.** If you repeat the load, the loader reports already loaded. Do not remove the volume or regenerate credentials as a routine restart step.

The old Python reference server is not used by this path. Its historical PID/log details are retained in [RETRIEVAL.md](../../embedding-code/RETRIEVAL.md#historical-transformers-validation--2026-10-12); do not stop unrelated processes as part of restarting llama.cpp.

## Troubleshooting and validation boundary

- **Missing run/model files:** private artifacts are not tracked. Ask the instructor for the authorized saved artifacts rather than silently using another corpus or model.
- **Embedding service unavailable/port occupied:** check `/health`; start only one matching service on port 9999.
- **Database connection failure:** use the same container engine and saved credential files as initialization, confirm `status`, and source `connection.env` in the current shell.
- **Permission denied while loading:** loading requires admin `DATABASE_URL`; `READER_DATABASE_URL` is intentionally read-only.
- **Collection conflict:** do not overwrite it. Confirm the selected run/model/profile or choose a new collection name.

The pipeline was validated with real local embeddings and database loading, repeat-load avoidance, read-only queries, permissions, and source hashes. Container startup was tested with **Podman**, not separately with Docker. Full evidence and open corpus/model/runtime gates are in [the preparation report](../../embedding-code/RETRIEVAL.md). This guide documents the existing preparation workflow; learner integration was separately authorized and is documented in [Lesson 12](LESSON.md).
