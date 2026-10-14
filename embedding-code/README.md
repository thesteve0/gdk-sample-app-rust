# Local PDF ingestion and draft chunking

Instructor preparation tooling, separately authorized on 2026-10-11. This is **not the Lesson 12 learner application**. Root remains Lesson 11. The original PDF ingestion phase stopped before embedding and PostgreSQL; the instructor separately authorized **Harrier test embeddings, database loading and querying on 2026-10-12**. See [RETRIEVAL.md](RETRIEVAL.md) for Docker/Podman startup, DDL, SQL, Rust loading/querying and the local model runtime. The historical PDF evidence below remains unchanged. Corpus layout acceptance/freezing, alternative chunk-policy implementation, OCR and EPUB processing remain open.

## Run

Linux, repository Rust 1.94.1 toolchain, Poppler `pdftotext` and `pdfinfo`, and coreutils `timeout` are required. On Fedora, install missing tools with `sudo dnf install poppler-utils coreutils`. Rust invokes installed executables directly with argument arrays, not a shell. Each extraction/info invocation has a 120-second timeout.

From the repository root:

```sh
cargo run --locked --manifest-path embedding-code/Cargo.toml
```

Defaults select the three exact PDF filenames in the [preparation plan](../lessons/12-specialized-retrieval/PLAN.md), from `day_trading/books/`, and create a new private run directory in `day_trading/embeddings/`. Paths are resolved against this package's manifest, not the caller's current directory.

To explicitly select another PDF or adjust the draft character bound:

```sh
cargo run --locked --manifest-path embedding-code/Cargo.toml -- \
  --pdf DayTrading.com-PDF-Day-Trading-For-Beginners.pdf --max-chars 3000
```

`--pdf BASENAME` can be repeated; source identity retains case and full filename. Optional `--books DIR` and `--output DIR` override directories. Output overrides may place private data outside Git's ignore rules: use a protected, ignored location. No `--all` shortcut silently includes multi-column books. Other PDFs require separate layout review; EPUB and AZW3 are not processed.

## What happens

**Inventory** records the exact source filename, SHA-256 and byte size. A **document ID** hashes case-sensitive filename and source bytes' hash; the similarly named Heitkoetter and CFI books cannot collide through lowercasing. Byte-identical files are flagged without discarding their separate provenance. Title, author and publication metadata remain null rather than inferred from filenames or unverified PDF metadata. Input files must be regular, nonsymlink PDFs, at most 256 MiB.

**Extraction** runs `pdfinfo` and `pdftotext -layout -enc UTF-8 -eol unix` against a private hashed source snapshot. Form-feed page boundaries must match `pdfinfo`'s physical page count, including empty pages; mismatches fail rather than invent locations. Stderr and tool versions are retained. Low-text pages and replacement characters raise review warnings; low text does not imply a blank page. Printed page labels are not guessed.

A **block** is a page-local run of nonblank extracted lines. Blocks record exact raw text and inclusive 1-based line ranges. Plain paragraph wraps are joined with spaces, but ambiguous hyphenated wraps remain visible. Indentation, list markers or apparent columns conservatively mark a layout block; its text stays unchanged. These are heuristics, not certified PDF structure. Only reviewed, checksum-bound cleanup rules remove cover/margin lines before block construction; no generic repetition heuristic is applied. No dehyphenation is performed, and charts/images are not interpreted.

A **draft chunk** groups whole paragraph blocks within one physical page, with one whole preceding block of overlap when it fits alongside new content. Layout blocks remain isolated. Page and layout boundaries stop overlap; unknown section relationships remain null. The 3,000 Unicode-character default is a provisional size guard, **not a tokenizer count or validated model limit**. Oversized ordinary paragraphs may split at conservative sentence/source-line boundaries; unsafe oversized blocks are quarantined with warnings and retained in block records, never truncated. Source-line fallback can break a sentence and therefore requires review. No cross-page paragraph repair or inferred heading hierarchy is claimed.

Chunk IDs bind chunk version, source document, size configuration, raw location/content and normalized content. Chunks record ordered block IDs, overlap block IDs, normalized text and its SHA-256, and exact raw fragments with UTF-8 byte offsets into each block. `raw_excerpt` concatenates those fragments; intervening blank separators between distinct blocks are not included. Full exact page text remains in `pages.jsonl`/`extracted.txt`. No title/heading or model-specific prefix is prepended yet.

All chunks and the manifest explicitly carry `draft_pending_layout_and_tokenizer_review`. Neither successful extraction nor absence of a warning makes a passage accepted or embedding-ready. Review every document's reading order, visual-dependent passages, repeated headers/footers, front matter, and section boundaries before acceptance. Verified cleanup rules, section relationships, and candidate-tokenizer limits (including prefixes) are tomorrow's prerequisites before freezing one shared corpus. Keep overlapping passages visible to later evaluation.

## Private artifact contract

Each timestamped `pdf-draft-*` run contains:

- `manifest.json`: completed **draft** preparation, tool versions, settings, source inventory, warning summaries, and sorted per-artifact SHA-256/byte counts.
- `<document-id>/source.pdf`: exact private snapshot used by extraction.
- `pdfinfo.txt`, `pdfinfo-stderr.txt`, `pdftotext-stderr.txt`: extraction evidence, not verified bibliographic claims.
- `extracted.txt`, `pages.jsonl`: unmodified page-aware extraction.
- `cleaned-pages.jsonl`: extraction with reviewed exclusions blanked, preserving original line numbering.
- `excluded-lines.jsonl`: every removed line, physical page/line, exact text, reason and cleanup version; empty for unreviewed documents.
- `blocks.jsonl`: raw and normalized text, location and block classification, including quarantined blocks, built from cleaned pages.
- `chunks.jsonl`: deterministic draft passage IDs, raw/normalized content and provenance.
- `document.json`: source identity, counts, duplicate-content reference and warnings.

New directories request mode 0700; files request 0600. Existing parent permissions are not changed. Artifact directory symlinks are rejected. Ignore rules are not encryption; books/extracts and snapshots remain private and must not be redistributed without rights. Terminal output is limited to filenames/counts/status, never excerpts. Cyan headings, green status, yellow pending-review warnings and red stderr errors are presentation-only; redirected streams, `NO_COLOR` (including empty) and `TERM=dumb` disable ANSI independently for stdout/stderr.

Each run creates new files rather than overwriting previous work. File contents are synced; a manifest is written only after every requested PDF finishes. A failed run retains available partial evidence and `FAILED.txt`, without a completion manifest. This is not transactional/crash-safe publication, resumable ingestion or exactly-once storage. Treat any absent/invalid manifest as incomplete; verify artifact hashes before later consumption. No completed-draft manifest authorizes embedding automatically.

## Dependencies and licenses

The small independent Cargo manifest/lockfile includes `anyhow`, `serde`, `serde_json` and `sha2` (MIT OR Apache-2.0, checked in selected crate manifests). There is no GDK dependency, root workspace or root dependency change. Poppler is an external executable, not linked or vendored into Rust; installed Fedora Poppler 26.01.0 includes GPL and other licensed components (`/usr/share/licenses/poppler/COPYING`, package metadata). Any binary redistribution needs its own license review. Its installed man pages/help and [official pdftotext manual source](https://gitlab.freedesktop.org/poppler/poppler/-/blob/master/utils/pdftotext.1) document layout/form-feed/UTF-8 behavior. Layout mode preserves physical spacing as best it can, not a guarantee of semantic reading order.

## Validation

```sh
cargo fmt --manifest-path embedding-code/Cargo.toml --check
cargo check --locked --manifest-path embedding-code/Cargo.toml
cargo clippy --locked --manifest-path embedding-code/Cargo.toml --all-targets
cargo test --locked --manifest-path embedding-code/Cargo.toml
cargo run --locked --manifest-path embedding-code/Cargo.toml
```

The synthetic tests include an authored minimal PDF passed through the actual installed Poppler, physical empty-page preservation, failure markers, private file permissions, no-ANSI redirected output, case-sensitive IDs, Unicode limits, structural overlap/progress, quarantine, and raw-fragment reconstruction. Tests require Poppler/coreutils and do not silently skip extraction.

Real-book validation on 2026-10-11: 18 + 273 + 116 = **407 physical pages**, **92 + 1,318 + 995 = 2,405 draft chunks**, no oversized blocks quarantined at the default bound. Heitkoetter emitted 114 bytes of Poppler diagnostics. Low-text review flags: 1, 12 and 4 pages respectively. All 995 CFI blocks were conservatively classified as layout; this preserves spacing but is not an accepted paragraph/section segmentation. This was a pipeline/provenance check, not full visual review, source correctness, tokenizer validation or retrieval evaluation. Layout acceptance and model-ready freezing remain open. No embedding, database, model-fidelity or downstream application validation was performed.

Final checks passed: formatting, Cargo check, all-targets Clippy and **11 preparation tests**; 27 saved artifact checksums and all 2,405 raw-fragment mappings/character bounds/content hashes were verified. The unchanged root separately passed formatting/check/Clippy and its 26 tests; no fresh live tutor run was needed or performed. Most recent validated private run: `day_trading/embeddings/pdf-draft-1791787419-968281896/`. Earlier draft runs are retained rather than overwritten.


## Reviewed beginners-guide cleanup — 2026-10-12

The instructor rejected cover/branding and running margin chunks in the DayTrading.com guide. `cleanup.rs` now applies `verified-margins-v1` only to inspected source SHA-256 `3e32d4176274c8ffcabd432235bdac313160ee77afbc15592e4cf5f9124793ec`. It excludes all nonblank lines on cover page 1, the exact `DAYTRADING.COM` first nonblank line on pages 2–18, and the exact running footer label with matching `page - 1 / 17` counter as the last nonblank line. Whitespace differences are allowed in the counter, not arbitrary footer content. Same text in the body is not removed; changed sources and other documents get no cleanup. These exclusions are reviewed extraction cleanup, not full layout acceptance.

Raw `pages.jsonl`, `extracted.txt` and `source.pdf` remain untouched. Removed lines are blanked in `cleaned-pages.jsonl`, not deleted, so surviving block line numbers still refer to original extraction. Every removal is auditable in `excluded-lines.jsonl`. The manifest is now format `pdf-corpus-draft-v2`, records `cleanup_version`, and each document records `excluded_lines`. Chunks are renumbered after cleanup; old chunk numbers are not stable identifiers. Original runs are retained.

Validated new run: `day_trading/embeddings/pdf-draft-1791823768-592081564/`. The beginners guide now has **56 chunks instead of 92**, with **38 excluded lines** (4 cover lines, 17 headers, 17 footers). The real “Keep A Journal” heading sharing a block with a brand header survives. Raw pages match the original run byte-for-byte; all surviving block line spans and chunk raw fragments map back to the raw extraction. Other books' chunk files are unchanged. Total draft chunks: **2,369**, still 407 pages. All artifact checksums verified; formatting/check/Clippy and **14 preparation tests** passed. No embeddings, database work or fresh tutor run performed. The earlier 2,405-chunk evidence above describes pre-cleanup drafts, not current accepted content.

Preview the cleaned guide from any working directory (`jq` and `less`):

```sh
RUN="/var/home/stpousty/git/gdk-sample-app-rust/day_trading/embeddings/pdf-draft-1791823768-592081564"
DOC=$(jq -r '.documents[] | select(.source_file == "DayTrading.com-PDF-Day-Trading-For-Beginners.pdf") | .id' "$RUN/manifest.json")
jq -r '"===== Chunk \(.order + 1) | PDF page \(.page_start) =====", .text, ""' "$RUN/$DOC/chunks.jsonl" | less
```

## Larger Heitkoetter chunks — 2026-10-12

The instructor found Heitkoetter passages too small. Indented prose/lists were classified as layout and isolated, producing a median of 82 characters. For inspected source SHA-256 `74296415db1e5d94b64352751df2a7b4d559b227fc0916f31d545e2ad35efeb0`, the new `page-block-groups-v1` policy groups adjacent paragraph and layout blocks in physical page order, preserving each layout block's spacing. Other documents keep isolated layout behavior. The document record now includes `chunk_policy`; grouped-policy IDs bind that policy explicitly. Layout blocks are never split or used as overlap; whole paragraph overlap is allowed only with new progress and sufficient room. The 3,000-character bound and page boundary remain unchanged. No content is dropped and no semantic section or cross-page join is guessed. Headers/page numbers remain in this book until separately reviewed cleanup; grouping is not margin removal or full layout acceptance.

Validated run: `day_trading/embeddings/pdf-draft-1791824631-737894742/`. Heitkoetter decreased from **1,318 to 273 chunks**, median **1,252 characters**, maximum **2,742**, with 26 chunks below 300 characters (including sparse/front-matter pages). All original blocks remain represented, with exact raw-fragment provenance and unchanged page extraction. The beginners guide and CFI chunk files are byte-identical to the previous cleaned run (56 and 995). Total draft chunks now 1,324. Formatting/check/Clippy and **15 preparation tests** passed; actual artifact hashes, complete block coverage, page boundaries and size bounds verified. No embeddings or root application changes.

Preview the larger Heitkoetter chunks:

```sh
RUN="/var/home/stpousty/git/gdk-sample-app-rust/day_trading/embeddings/pdf-draft-1791824631-737894742"
DOC=$(jq -r '.documents[] | select(.source_file == "The-Complete-Guide-To-Day-Trading.pdf") | .id' "$RUN/manifest.json")
jq -r '"===== Chunk \(.order + 1) | PDF page \(.page_start) =====", .text, ""' "$RUN/$DOC/chunks.jsonl" | less
```

## CFI running-margin cleanup — 2026-10-12

Instructor requested CFI header/footer removal. `verified-margins-v2` retains the beginners rules and adds a rule only for CFI source SHA-256 `50f6e16003682e96f7bd5fc05d5947fcbcc3efba5e8c37afdca908f802251049`. On physical pages 1–116, remove the first nonblank line only when whitespace-normalized text exactly matches `The Corporate Finance Institute The Complete Guide to Trading`, and the last nonblank line only when it exactly matches `corporatefinanceinstitute.com <physical-page-number>`. The same strings in body positions and mismatched counters remain. This does not remove CFI's cover title, section titles or front matter; it removes running margins only. Cleaned pages retain original line numbers; raw evidence is unchanged and all 232 removals are recorded in excluded-lines JSONL.

Validated run: `day_trading/embeddings/pdf-draft-1791824722-337094533/`. CFI decreased **995 to 765 draft chunks**, with 116 headers and 116 footers removed. Other books' chunks are byte-identical to the prior run (56 beginners, 273 Heitkoetter), giving 1,094 total drafts. The decrease is 230 chunks rather than 232 because two removed lines had shared blocks with other content. All artifact hashes, raw-page preservation, surviving original-line locations, chunk raw-fragment mappings and size bounds were verified. Formatting/check/Clippy and **16 tests** passed. Full CFI layout/visual review and tokenizer/freeze gates remain open; no embeddings or root/dependency changes.

```sh
RUN="/var/home/stpousty/git/gdk-sample-app-rust/day_trading/embeddings/pdf-draft-1791824722-337094533"
DOC=$(jq -r '.documents[] | select(.source_file == "The-Complete-Guide-to-Trading.pdf") | .id' "$RUN/manifest.json")
jq -r '"===== Chunk \(.order + 1) | PDF page \(.page_start) =====", .text, ""' "$RUN/$DOC/chunks.jsonl" | less
```

## Larger CFI chunks — 2026-10-12

Instructor found cleaned CFI chunks too small. Its inspected source checksum now also selects the existing `page-block-groups-v1` policy: adjacent blocks are grouped within physical pages, preserving layout text and the 3,000-character cap. No new cross-page, inferred-section or tokenizer behavior is introduced. Reviewed header/footer cleanup remains unchanged (232 exclusions); no cleaned blocks are dropped.

Validated run: `day_trading/embeddings/pdf-draft-1791825122-21376647/`. CFI decreased **765 to 182 draft chunks**, median **369 to 1,634 characters**, maximum **2,991**, with 11 below 300 characters (sparse pages or size-bound remainders). Every cleaned block is represented, raw fragments/content hashes/page bounds verified, and raw/cleaned pages, blocks and exclusion records are byte-identical to the preceding run. Beginners and Heitkoetter chunks remain unchanged (56 and 273). Total now **511 draft chunks**. All 16 tests plus formatting/check/Clippy passed; saved artifact hashes verified. Full visual/section review, candidate-tokenizer limits and corpus freezing remain pending. No embedding calls, database work or root/dependency changes.

```sh
RUN="/var/home/stpousty/git/gdk-sample-app-rust/day_trading/embeddings/pdf-draft-1791825122-21376647"
DOC=$(jq -r '.documents[] | select(.source_file == "The-Complete-Guide-to-Trading.pdf") | .id' "$RUN/manifest.json")
jq -r '"===== Chunk \(.order + 1) | PDF page \(.page_start) =====", .text, ""' "$RUN/$DOC/chunks.jsonl" | less
```

## Read-only word-count comparison

`compare_chunk_words.py` supplies three independent what-if reports without changing Rust chunking or saved chunks. Run with the system Python 3 and installed `pdftohtml` (Poppler):

```sh
PYTHONDONTWRITEBYTECODE=1 python3 embedding-code/compare_chunk_words.py --run \
  /var/home/stpousty/git/gdk-sample-app-rust/day_trading/embeddings/pdf-draft-1791825122-21376647
(cd embedding-code && PYTHONDONTWRITEBYTECODE=1 python3 -m unittest test_compare_chunk_words -v)
```

The script verifies cleaned-page/source hashes, reads current cleaned text, and uses PDF font size/color/family to identify **candidate** major headings and finer subsection headings, matching typography back to physical page/line positions. Major sections mean prominent chapter/topic headings (including part introductions), not only top-level Parts. Subsections include major boundaries plus smaller heading styles. This is an exploratory estimate, NOT a verified section map: emphasized prose can be misidentified, some headings missed, and CFI sidebar text sharing a heading line stays with that line. No semantic reading-order repair is claimed. Leading/front matter and tiny part introductions remain counted; there is no hidden minimum-size filter, character cap, section overlap or automatic acceptance. Inspect the saved heading-candidates JSON before trusting section results; source checksums bind these book-specific typography heuristics.

Words are whitespace-separated items, including standalone punctuation/symbols, NOT model tokens or dictionary words. For page chunks, add `ceil(previous physical page base words / 10)` words from the preceding page; totals include overlap, which is not recursively inherited. Empty pages create no chunk and reset carry-over. This count-only script does not materialize alternative passage text. Current Heitkoetter headers/page numbers/front matter remain included; beginners/CFI use their reviewed cleanup.

Each new protected `word-count-comparison-<timestamp>/` directory under the selected private run contains per-book **major_sections.csv**, **subsections.csv**, **pages_10pct_overlap.csv** (filenames prefixed by document ID), source typography XML/diagnostics, heading-candidates JSON and summary.json. CSVs list every chunk's title, page range, base words, overlap words and total words. Newly created files request 0600 and report directory 0700. Reports contain private headings/text in typography XML and must stay ignored. Three synthetic Python tests passed; real reports assert section base counts cover all cleaned words exactly once. Existing 511 Rust draft chunks are untouched; no embeddings.

Validated report directory: `day_trading/embeddings/pdf-draft-1791825122-21376647/word-count-comparison-1791826576105244339/`. Candidate counts (chunks / median words / max words): beginners major 14 / 281 / 628, subsections 44 / 77.5 / 330, pages+overlap 17 / 291 / 360; Heitkoetter major 38 / 678 / 7,903, subsections 114 / 291.5 / 4,316, pages+overlap 271 / 212 / 446; CFI major 27 / 983 / 2,873, subsections 161 / 175 / 960, pages+overlap 116 / 357 / 490. These inferred section results are not frozen corpus boundaries.


## Chunk-policy comparison decision — 2026-10-12

Instructor agreed to compare **one nonempty physical page plus 10% preceding-page word-tail overlap** against **section-aware paragraph groups targeting roughly 300–500 words**. Whole major sections/subsections are too variable for direct chunking; they remain diagnostic reports only. Review inferred headings and remaining Heitkoetter margins first. Page overlap uses ceil(10% of previous physical page base words), no recursive overlap, with empty pages resetting carry-over; structural overlap for section-aware groups must be explicitly settled/versioned within section boundaries. Word targets are not model-token limits. Use the same reviewed text/exclusions and source-span relevance questions; compare coverage/boundaries, size/overlap distributions, retrieval/evidence completeness and cost under fixed model settings and equal returned-token budgets. Freeze policy-specific collections and a common collection per policy across candidate models, then choose policy/model from evidence. See Section 6 and Section 9 of `lessons/12-specialized-retrieval/PLAN.md`. This request updates documentation only: current 511 draft chunks, Rust/Python code, root application and dependencies remain unchanged; no embedding calls, corpus freeze or database work authorized by this decision.


## Harrier test retrieval — 2026-10-12

The separately authorized [Harrier/PostgreSQL workflow](RETRIEVAL.md) now embeds and stores the saved **511 draft passages** (56/273/182) with preserved metadata and source hashes. Three live read-only search smoke queries and idempotent loading passed against local PostgreSQL 18.6/pgvector 0.8.7. The first validation used the official local checkpoint reference runtime; the later switch below uses llama.cpp on port 9999 in a separate collection. Drafts remain drafts. See that document for exact startup/load/query commands, private credentials/data handling, validation counts and open gates. Root remains Lesson 11.


### Harrier llama.cpp runtime switch — 2026-10-12

The instructor explicitly requested the existing **llama.cpp service on port 9999** instead of the Python reference runtime. The preparation Rust client now targets its OpenAI `/v1/embeddings` API, applies the query instruction in Rust and preflights tokenizer/context counts. The active collection is **`harrier-llamacpp-test-511-v1`**, re-embedded with the pinned F16 GGUF/runtime profile; the original `harrier-test-511-v1` Transformers collection is preserved and must not be mixed. Follow `embedding-code/RETRIEVAL.md` and Lesson 12 `DATABASE-SETUP.md` for current startup/load/query instructions. This changes only preparation, not root Lesson 11 or learner Lesson 12 integration. The existing user's server is bound to 0.0.0.0; instructions use 127.0.0.1, but no automatic restart occurs. Cross-runtime parity and final retrieval quality are not claimed.
