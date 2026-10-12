# Lesson 12 plan — Local specialized educational retrieval

**Detailed planning only — 2026-10-11.** The instructor authorized this document, not implementation, model conversion, ingestion, infrastructure launch, dependency changes, or root replacement. Root remains Lesson 11. Read [README](../../README.md) and [the shared handoff](../final-lessons-handoff.md) first.

## 1. Goal and lesson boundary

Connect the existing learning partner to prepared book passages in local PostgreSQL using **pgvector**, an extension that stores numeric vectors and supports similarity search. An **embedding** is a model-produced numeric representation of a passage or question. **Retrieval** selects source passages to accompany a tutor request; it does not retrain the tutor. **Provenance** records where each passage came from so its support can be inspected.

Teach bounded access to external evidence, source-grounded answers, absence of evidence, and the distinction between application instructions and untrusted retrieved text. Day trading is the teaching vehicle, not permission to build a trading platform.

Two workstreams are deliberately separate:

1. **Instructor-side preparation:** build the PDF/EPUB pipeline, compare embedding models, prepare private artifacts and the database.
2. **Learner Lesson 12:** integrate the supplied read-only datastore and matching query-embedding service into the learning partner.

Learners do not install PostgreSQL, convert models, ingest books, design chunking, or tune indexes in class. These are prerequisites supplied before the workshop. No Lesson 13 compaction, simulation, general CLI, market-data ingestion, brokerage, or fine-tuning implementation is included.

## 2. Instructor decisions now fixed

| Area | Decision |
| --- | --- |
| Execution | Entire workshop uses local services and local models; no external inference or fallback |
| Database | PostgreSQL 18 with pgvector 0.8.7; project image `pgvector/pgvector:0.8.7-pg18-trixie`, exact digest frozen at deployment |
| Index/metric | HNSW approximate nearest-neighbor index with cosine distance |
| Database vectors | Native float16 `halfvec`; no int8/binary-quantization detour |
| Embedding serving | llama.cpp HTTP embedding service; Rust sends documents and queries |
| Candidates | EmbeddingGemma 2, Microsoft Harrier OSS v1 0.6B, NVIDIA Nemotron 3 Embed 1B |
| Excluded candidates | Qwen embedding models excluded; Nomic was discussed but not selected for the three-model comparison |
| Weight precision | F16 or BF16, chosen by correctness and measured AMD behavior; no initial Q4/Q8 comparison |
| Model files | Unsloth GGUF where available; otherwise reproducible official-checkpoint conversion through llama.cpp |
| Dimensions | Up to 1,024; Harrier 1,024, Nemotron reduced to 1,024, EmbeddingGemma native 768 without padding |
| Chunking | Paragraph/section-first, structural overlap, token counts only as safeguards |
| Full corpus | All PDFs and EPUBs in `day_trading/books/`; exclude AZW3 |
| Preparation code | Separate Rust application in `embedding-code/`, not a premature root workspace |
| Private outputs | Text, chunks, vectors, manifests, exports, content-bearing reports in `day_trading/embeddings/` |
| Public materials | Code, DDL, nonsecret examples, preparation documentation, synthetic tests |
| Approval | Local extraction is permitted; ownership is not redistribution permission |

HNSW is the index algorithm, not a similarity metric. HNSW will use cosine distance; the normal application path will not be exhaustive nearest-neighbor search. Cosine distance is not a probability of relevance or truth.

Weights and output-vector precision are separate choices. An F16/BF16 GGUF produces embeddings that are validated and then stored as float16 in PostgreSQL. pgvector `halfvec` is float16, not BF16. A small float32 disk reference is permitted for reproducibility and measuring rounding effects; it is not an alternative production database representation.

## 3. Local-only privacy and deployment

Book passages, learner text, and embeddings never go to an external inference service. There is no remote fallback on failure. Model and container downloads are setup activities; pre-download them so workshop operation does not require internet access.

Bind services to loopback by default. A local-network embedding/tutor host is allowed for the instructor's machines; explicitly configure and verify its identity. Do not claim arbitrary URL syntax proves a service is local: public DNS, redirects, and proxies can defeat simplistic checks. During implementation, document the actual topology, use approved endpoints, disable unintended proxy/redirect behavior, and verify offline operation.

Credentials belong in private environment/configuration, never source, logs, commands committed to the repo, or URLs printed in diagnostics. Separate ingestion/admin privileges from the Lesson 12 read-only role. Do not expose PostgreSQL or inference servers publicly. Decide TLS/authentication for any LAN connection when establishing that topology.

`day_trading/` is already ignored. Ignore rules are not encryption or access controls. Protect artifacts and database volumes with appropriate local permissions and retain them only as needed. Embeddings are also private derived artifacts, not presumed safe to redistribute. Private evaluation questions, annotations, or reports containing source text stay ignored.

The workshop must not redistribute private books, excerpts, embeddings, or database exports without separate rights. Public automated tests use clearly course-authored/synthetic material, distinct from real private-book validation. Confirm authorized access/distribution arrangements before supplying private data on learner machines.

## 4. Initial PDF MVP corpus and inspection evidence

Use these exact three files for the first comparison:

| Source file | Identity | Pages | Approximate extracted words |
| --- | --- | ---: | ---: |
| `DayTrading.com-PDF-Day-Trading-For-Beginners.pdf` | DayTrading.com beginners guide | 18 | 4,455 |
| `The-Complete-Guide-To-Day-Trading.pdf` | Markus Heitkoetter, The Complete Guide to Day Trading, 2008 | 273 | 52,688 |
| `The-Complete-Guide-to-Trading.pdf` | Corporate Finance Institute, The Complete Guide to Trading | 116 | 33,965 |
| **Total** | | **407** | **91,108** |

These are approximate local `pdftotext -layout` inspection counts, including front matter, headings and footers, not accepted passage counts. No model embeddings or retrieval tests have been produced. Sampled Heitkoetter pages were single-column; the whole document is not yet layout-certified. Extraction reported font warnings but yielded readable text. CFI contains charts and other layout features requiring inspection.

Warrior Trading was removed from the **initial test**, not from the eventual all-PDF/EPUB corpus. Its two-column layout requires a later extraction gate before inclusion. The similarly named Heitkoetter and CFI files need distinct document IDs; do not identify them by a lowercased basename alone.

The corpus has overlapping topics (risk, order behavior, indicators and psychology) and distinct examples, making it useful for difficult retrieval rather than trivial keyword matching. It does not establish software/code retrieval quality. Add representative passages from the financial-software EPUBs before the final full-corpus model freeze, or explicitly record that quality as unvalidated.

Historical publication dates matter. Test whether a passage supports a question, not whether a 2008/2018 book proves current rules or trading correctness.

## 5. Model preparation and swappable embedding contract

### Candidates

| Candidate | Source | Required behavior | Output policy |
| --- | --- | --- | --- |
| EmbeddingGemma 2 | `google/embeddinggemma-2`; Unsloth GGUF available | Verify exact text-only GGUF prompts, pooling and tokenizer against selected source revision | Native 768; normalize as prescribed |
| Harrier 0.6B | `microsoft/harrier-oss-v1-0.6b` | Last non-padding token pooling; retrieval-instructed queries; no query instruction on passages; L2 normalization | 1,024 |
| Nemotron 1B | `nvidia/Nemotron-3-Embed-1B-BF16` | Bidirectional attention; mean pooling; `query: ` and `passage: ` prefixes; L2 normalization | Native 2,048; take leading 1,024 and normalize again |

Newest does not mean best on this corpus. Publisher benchmark claims are dated evidence, not independently verified current global rankings. Sentence Transformers compatibility does not automatically mean llama.cpp support. Harrier's underlying Qwen architecture is not the excluded Qwen embedding model.

### Conversion procedure when authorized

1. Download an exact official checkpoint revision, retaining Safetensors, tokenizer/special-token files, architecture config, Sentence Transformers module/pooling/prompt files, licenses and notices.
2. Pin a llama.cpp commit and isolated conversion environment. Use its official `convert_hf_to_gguf.py`; do not write a Rust converter.
3. Check supported architecture and tensor mappings before conversion. Do not rename architecture fields to bypass unsupported behavior.
4. Produce F16 first; produce BF16 where the selected runtime supports it correctly. No training or low-bit quantization is part of this step.
5. Inspect GGUF architecture, attention, pooling, tokenizer, dimensions and tensor metadata. A context field is not proof of a validated usable embedding length.
6. Compare against the official Transformers/Sentence Transformers checkpoint using fixed synthetic inputs: short/long texts, correct prefixes, individual/batched requests, relevant/unrelated pairs, finite values, norms, vector similarity and retrieval rankings.
7. Freeze accepted GGUF checksum, source revision, converter/runtime revision, backend and precision. Retain numerical deviations and acceptance rationale; do not demand bitwise identity or accept a load-only smoke test as fidelity validation.

Conversion can run on CPU. Reference inference is preparation tooling and may use Python; ingestion remains Rust. No requirement to implement GPU inference in Rust. Community Nemotron conversion evidence is a starting point, not proof of parity; explicitly verify noncausal attention and mean pooling.

### Configuration boundary

The Rust program uses a small explicit model configuration, not an elaborate plugin framework. Record model ID, checkpoint/GGUF identity, endpoint, tokenizer identity, query/document formatting, native/selected dimensions, normalization/truncation rules, context/batch limits, runtime and precision. Use one formatting path for preparation and the matching Lesson 12 query path.

Equal dimensions do not imply compatible embedding spaces. Never query one model's vectors with another model's query embeddings. Unknown identities, wrong dimensions, malformed responses and incompatible preprocessing fail explicitly.

### Hardware validation

The repository host reports Ryzen AI 9 HX PRO 370/Radeon 890M (Strix Point), Fedora 44. Intended additional host is Ryzen AI Max+ 395/Radeon 8060S (Strix Halo); the instructor also has an RTX 4080 laptop with 12 GB VRAM. The Rust HTTP interface is unchanged across hosts. Vulkan/HIP/CUDA builds, BF16 support and numerical behavior still need verification; speed is not assumed to be the only difference.

Establish correctness locally first. Measure warm and cold timing, fixed passage lengths, batch sizes, GPU offload and competing tutor-model load. Compare the NVIDIA host only if useful; no chat-generation benchmark substitutes for embedding throughput.

## 6. Extraction and chunking

A **text block** is a paragraph, heading, list, table or code block recovered from the source. A **chunk** is a bounded coherent passage assembled from those blocks. It must retain enough context to retrieve and cite it.

1. Inventory allowed files and hash source bytes. Record title/author/edition/publication metadata only when verified; unknown values stay unknown. Separate publication date from preparation date.
2. For PDFs, recover page-aware text and reading order. Remove verified repeated headers/footers, repair line-wrap hyphenation conservatively, preserve paragraphs/lists and report warnings. Avoid damaging real hyphens, numbers or examples.
3. Inspect representative pages and anomalies before accepting extraction. Low text can mean a chart, front matter or failed extraction—not automatically a blank page. Record visual dependence rather than pretending omitted charts were understood.
4. Keep short coherent sections together. Split long sections into paragraph groups. Split an oversized paragraph at sentence boundaries only when needed. Preserve examples, procedures, lists, tables and code where practical.
5. Use complete sentences/paragraphs for overlap; avoid crossing document/section boundaries without a reason. Record overlap and parent-section relationships so duplicated context is visible.
6. Token limits are safety guards, not fixed-size chunk targets. Use each candidate's tokenizer to verify the shared corpus fits all candidates, including prefixes. Freeze one common chunk set without silent per-model truncation; if a shared chunk is too long, revise it for all candidates or report exclusion.
7. PDF chunks may span adjacent pages, with exact physical page spans. Printed labels are separate and only stored when verified. EPUBs preserve spine order, chapter/resource paths and useful anchors, never invented PDF pages.
8. Preserve exact extracted passage separately from any title/heading context prepended for embedding. Version normalization and embedding-input construction.

Use deterministic passage IDs derived from source/preparation identity and location/content; record content hashes, chunk order and extraction/chunking version. Detect duplicate files and alternate editions without silently discarding provenance.

Start text-only. OCR and chart interpretation are not implicitly authorized additions to the MVP. If required later, propose local tooling and document its quality/rights implications. Unresolved layout or visual-dependent passages are explicitly excluded/quarantined with counts, not silently lost.

## 7. Rust preparation flow and deliverables

Future application: `embedding-code/`, with its own Cargo manifest/lockfile and documentation, using the repository's pinned Rust toolchain. Keep GDK, provider, RMCP and root dependencies unchanged during preparation. Select narrow dependencies and verify current licenses/APIs during implementation: page-aware PDF extraction, EPUB ZIP/XML/XHTML reading, tokenizer-aware splitting, HTTP client, hashing/serialization and a PostgreSQL client with pgvector support. EPUB crate licensing and external PDF library distribution deserve explicit review.

Stages:

**Inventory → extract → inspect → normalize → chunk → freeze corpus → embed → validate → save → load PostgreSQL → build HNSW → evaluate**

The approved MVP implements PDFs first. Add EPUB support after the end-to-end PDF flow works; the eventual scope includes every allowed book, including financial software/AI books. Label collections for filtering without excluding software topics.

Private disk outputs precede database writes. Save accepted batches atomically/checkpoint progress, validate checksums and resume without re-embedding completed work. Retry bounded transient failures, but not identity/dimension errors. Use stable IDs and transactional, parameterized loading so retries do not duplicate rows. State durability limits honestly; do not promise exactly-once cross-service behavior.

Suggested public deliverables (not files supplied yet):

- `embedding-code/README.md`, manifest/lockfile and complete source
- `embedding-code/schema/` DDL, roles and HNSW definitions
- Nonsecret model/service/database configuration examples and conversion manifest templates
- Synthetic extraction, vector-response and storage fixtures/tests
- Setup, conversion, ingestion, evaluation and restore instructions

Suggested private artifacts in `day_trading/embeddings/`:

- Run manifest and source inventory
- Extracted page/block records and extraction warnings
- Frozen passage JSONL and locations
- Per-model float-vector shards keyed by passage ID and manifest/checkpoint files
- Private relevance annotations and evaluation reports
- PostgreSQL-loadable metadata/vector export
- Copy of public DDL for restoration

Binary float32 vector shards plus JSONL metadata are the proposed disk interchange; PostgreSQL stores native float16. Validate float16 rounding/overflow and measure ranking effects. Artifact formats remain implementation choices, documented before first full run. Do not generate a giant SQL file of interpolated passage literals; database loading is parameterized or uses a verified bulk-copy interface.

## 8. Proposed database contract

Define a **preparation run** as the versioned extraction/chunking/embedding process. Define an **embedding space** as the exact model/preprocessing/dimension contract; different spaces cannot be mixed.

Proposed entities, to become DDL during implementation:

- Documents: stable ID, verified bibliographic metadata, source hash, collection, publication information.
- Passages: ID, document ID, order, exact excerpt, section/location/page spans, content hash and preparation identity.
- Preparation runs / embedding spaces: revisions, settings, dimensions, normalization, runtime/precision and corpus manifest identity.
- Embeddings: passage ID, embedding-space identity, vector and completion status as needed.

Use model-specific dimension-constrained embedding tables/indexes for the comparison (768 versus 1,024), or an equally explicit reviewed design; do not use a mixed unconstrained vector index. Expose only the selected complete preparation through a read-only retrieval view/contract. Enforce foreign keys, uniqueness and dimension/space consistency.

Build HNSW with `halfvec_cosine_ops` after bulk loading accepted rows, then update planner statistics. Tune construction and query search settings against recall, build time and memory. Verify the intended filtered query actually uses the index; filtering can reduce approximate-search recall. Collection restrictions belong to the application and use parameters.

The approved image is pinned by digest at deployment. Verify actual server and extension versions rather than assuming a tag inherits every latest PostgreSQL patch. PostgreSQL 18's image volume is `/var/lib/postgresql`; account for its changed layout. Size shared memory for index-building settings and persist the database in a protected local volume.

No credentials or invented live connection addresses are part of this plan. Connection topology, role setup, finite pool/concurrency, authentication and timeout configuration are supplied and verified during implementation.

## 9. Small real-book comparison

Use the entire accepted three-PDF chunk collection as distractors, not only the answer-bearing passages. Freeze approximately 40 manually checked questions before examining model rankings:

| Type | Initial count |
| --- | ---: |
| Direct passage questions | 12 |
| Paraphrased/conceptual questions | 12 |
| Source-specific details/examples | 8 |
| Comparisons requiring multiple passages | 4 |
| Unsupported-by-this-corpus questions | 4 |

Distribute cases across books/topics and short/long passages. Label acceptable passage sets and alternate support, not just a single exact chunk when several overlap. Human-check any generated questions. These are source-support judgments, not expert assessments of trading validity. Exclude front-matter keyword leakage and inspect ambiguous or visual-dependent cases.

Measure Recall@5/10, MRR@10, and multi-passage coverage; nDCG is optional only with graded relevance labels. Unsupported cases are evaluated separately, not assigned a misleading answerable-query recall. Inspect raw distance distributions before choosing model-specific thresholds: nearest-neighbor search can return irrelevant neighbors, and no universal threshold establishes absence of evidence.

Record extraction/chunk counts, failures/exclusions, passage/token length distribution, precision, dimensions, model/runtime identities, hardware, warm/cold query latency (including p50/p95), embedding passages/tokens per second, index size/build time and HNSW settings. Repeat timing with comparable batches and tutor-service load. Distinguish embedding quality, float16 rounding and HNSW search effects in failure analysis; if exact comparison is needed as a private validation oracle, it is not the deployed application search path.

Choose the model based on relevant-passage retrieval, stable runtime fidelity and practical local latency, not release date or total multilingual scores. If results are close, prefer the simpler validated deployment. Report uncertainty from a small test and add an EPUB/software sample before claiming coverage of the full corpus. Freeze the winning embedding contract and re-embed new/final corpus consistently.

## 10. Lesson 12 runtime integration

A **retrieval tool** is a narrow read-only capability that returns bounded source evidence. It is not SQL execution or file/network access.

Proposed flow:

**Validate search request → embed query locally → parameterized read-only HNSW query → bound excerpts and provenance → correlated tool response → local tutor answer with readable citations**

Proposed tool name is `search_educational_passages`, an application choice to verify during implementation, not a preexisting GDK API. Accept a bounded query; reject unknown fields. Keep top-k and collection scope application-owned initially. The model cannot choose database identifiers, SQL, file paths, credentials or endpoint URLs.

Initial tuning proposals, not claims of tested defaults:

- Query: nonblank, at most 2,000 UTF-8 bytes.
- Results: at most five; top-k not supplied by the model.
- Query embedding wait: at most 15 seconds.
- Database query: at most five seconds with a server-side statement timeout.
- Returned excerpts: approximately 1,000 selected-model tokens each, with a total per-turn retrieval contribution of at most 4,000 tokens, counting provenance/serialization as well as excerpts.
- Preserve the existing shared four-request, eight-applied-pass and 180-second per-turn limits. Retrieval requests consume that same budget; no extra unbounded inference path.

Adjust these only with measured evidence and documented reasoning. Avoid cutting evidence mid-sentence or presenting clipped text as a complete passage. Include truncation status and retain full private source mapping. Multiple retrieval calls share an aggregate contribution budget; returned evidence and identities remain traceable.

Return distinguishable statuses: successful matches, insufficient relevant evidence, unavailable embedding/database service, incompatible preparation, and malformed data. A service outage is not evidence that the books contain no answer. Refuse incomplete/incompatible collections rather than silently querying a partial stale index.

Results include stable citation IDs, document title/author when known, edition/publication details where known, section/location, exact returned excerpt, preparation identity and labeled cosine distance. Citation IDs map to actually returned passages; missing membership is detectable but semantic support still requires inspection.

Tutor guidance: use passages as evidence, explain book-specific qualifications, distinguish source claims from added background, identify disagreements/outdated claims, and ask a narrower question when support is weak. No mastery/readiness claim, instructor-authored rubric claim, guaranteed correctness or promise that RAG prevents hallucinations. Retrieved instructions are untrusted data, not application policy.

## 11. Preserve Lesson 11 behavior

Read the complete root/reference, lesson, instructor notes and content before coding. Consult official GDK documentation and exact pinned GDK/RMCP source before API changes. No SDK upgrade for convenience.

Preserve:

- GDK-owned within-run loop, distinct Operations/Effects/Events and streamed event consumption.
- Exact-decimal calculator validation, tool allowlists, all-block handling, correlation and unknown/excess-request rejection.
- Fresh Session on selection, existing interactive lesson workflow, commands and EOF behavior.
- Default-no memory consent and independent default-no approval for every addition after a successful run.
- Actual learner quote provenance; retrieved text must never become an actual learner quotation or authorize a record write.
- Plain Tutor/Learner interface, course colors and terminal gating; readable citations without protocol dumps or retrieved-passage walls.
- Complete plain `session-output.md` diagnostics, partial failure state and post-approval state, never used as learner memory.

Diagnostics record retrieval query/results, provenance, selected contract, warnings and available partial state, without credentials, secret-bearing database URLs or vector dumps. They may contain private excerpts and learner text. No general shell, arbitrary SQL, file-access, network or trading capability.

Do not implement compaction now. Keep the known `.skip(1)` learner-quotation trap documented for Lesson 13 rather than changing conversation history opportunistically.

## 12. Classroom teaching materials

When implementation is separately authorized, supply complete `LESSON.md`, self-contained `src/main.rs` reference, nonsecret examples and `instructor-notes/12-specialized-retrieval.md`. Preparation tooling is separate, and the instructor confirms root replacement explicitly.

Define terms before code: embedding → embedding space → retrieval → provenance → cosine distance/HNSW → read-only retrieval boundary → source-grounded answer. Clearly distinguish current functionality from Lesson 13's future context management.

Classroom rhythm:

1. Predict whether a source-specific question requires retrieval.
2. Run the supplied application against prepared local services.
3. Observe readable citations and inspect exact evidence in private instructor diagnostics.
4. Explain query validation, local embedding, bounded database response and tutor inference.
5. Tweak only the question to a different supported detail; predict and run again.
6. Ask a corpus-unsupported question and inspect honest uncertainty, without deliberately breaking infrastructure.

Use private corpus questions only where learner access is authorized. Supply a public synthetic equivalent for reproducible setup/tests. Add exit questions distinguishing model knowledge from retrieved evidence, index from distance metric, correlation from support, and retrieval from learner mastery. Show source conflict where actual evidence permits it; never invent a disagreement to satisfy a test.

## 13. Implementation order and gates

Each phase requires a separate implementation request; this plan does not launch work.

1. **Local service/model preparation:** freeze image/runtime identities; verify endpoints, privileges, precision, conversions and reference parity.
2. **PDF MVP:** complete extraction → frozen chunks → embedding → private disk → PostgreSQL → HNSW → retrieval; no root replacement.
3. **Three-model comparison:** private relevance set, measured results, report and instructor model-selection checkpoint.
4. **Full corpus preparation:** EPUB support, software sample, multi-column/visual extraction gates, final consistent embedding and restore verification.
5. **Lesson integration:** complete retrieval reference, runtime content, prose/tests/notes; separately confirm root update.
6. **Live validation:** actual datastore, actual embeddings and local tutor; record results and remaining limitations.

Technical gates still open, not unresolved instructor permission questions: selected llama.cpp commit, conversion fidelity, F16/BF16/backend performance, extraction libraries/licensing, final schema/artifact format, network topology, HNSW tuning, thresholds/bounds and winning model. If a candidate cannot be served faithfully or source data cannot be supplied legally, report and ask rather than silently substitute.

## 14. Acceptance and validation

Preparation tests: source IDs/checksums, page/anchor mapping, normalization, boundaries/overlap, shared-chunk token limits, malformed/mismatched embedding responses, finite values/norms/dimensions, float16 conversion, resume/idempotent loads, model-space isolation, artifact integrity and restore.

Database/integration tests: real pgvector round trip, HNSW index/query behavior, permissions, collection filters, bounds/timeouts, missing/stale data, parameterized access and credential redaction. Synthetic fixtures are public; real-book metrics stay private.

Runtime tests: strict arguments, unknown/excess names/requests, all blocks and IDs, partial failures, no-evidence/service-error distinction, citation membership, bounded prompt contribution, local-only configuration, plain UI and unchanged memory/calculator behavior. Model-generated semantic support is evaluated separately; deterministic membership tests do not prove truth.

Live outcomes required:

- Matching query/corpus embeddings and real prepared PostgreSQL exercised.
- Source-specific details visibly supported by returned passages; generic plausible answers alone are insufficient.
- Actual retrieval tool-selection pattern reported, not forced or hidden.
- Unsupported and source-conflict cases handled honestly where available.
- No private data leaves configured local services; no remote fallback.
- Fresh selection and approved-memory behavior remain intact.

During implementation run the narrow relevant formatting, check, Clippy and test commands for both Rust applications, plus their documented local live commands. Do not use `--all-features` by default. Document exact commands only after they exist and are verified. If services are unavailable, ask the instructor; mocks do not replace real integration evidence.

Preserve previous validation gaps: Lessons 8–10 review, Lesson 9's 100-share live tweak, Lesson 10's inconclusive guidance experiment, and Lesson 11's expanded teaching evaluation/colored UI gaps remain independent. Instructor review is deferred to end-of-course, not completed by this plan.

## 15. Research references and next lesson

Supplemental references, not substitutes for supplied workshop instructions:

- [pgvector image and installation](https://github.com/pgvector/pgvector#docker), [HNSW](https://github.com/pgvector/pgvector#hnsw), [half-precision vectors](https://github.com/pgvector/pgvector#half-precision-vectors)
- [PostgreSQL version policy](https://www.postgresql.org/support/versioning/)
- [llama.cpp model conversion](https://github.com/ggml-org/llama.cpp/blob/master/docs/models.md), [converter](https://github.com/ggml-org/llama.cpp/blob/master/convert_hf_to_gguf.py), [server](https://github.com/ggml-org/llama.cpp/blob/master/tools/server/README.md)
- [Nemotron official checkpoint](https://huggingface.co/nvidia/Nemotron-3-Embed-1B-BF16), [community conversion evidence](https://huggingface.co/zenmagnets/Nemotron-3-Embed-1B-Q4_K_M-GGUF/blob/main/QUANTIZATION.md)
- [Harrier official checkpoint](https://huggingface.co/microsoft/harrier-oss-v1-0.6b)
- [EmbeddingGemma 2 guide](https://developers.googleblog.com/embeddinggemma-2-the-developer-guide/), [Unsloth GGUF files](https://huggingface.co/unsloth/embeddinggemma-2-GGUF/tree/main)
- [Sentence Transformers backends](https://www.sbert.net/docs/sentence_transformer/usage/efficiency.html), [pgvector Rust integration](https://github.com/pgvector/pgvector-rust)

Next: Lesson 13 manages working context as conversation and retrieved evidence accumulate. It must preserve source/learner provenance, tool pairing and local approval state. This plan does not authorize Lesson 13 or historical simulation.
