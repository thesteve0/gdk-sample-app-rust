# Lesson 12 — Retrieve specialized educational evidence

## Goal and boundary

Extend the supplied learning partner with a narrow, read-only search of locally prepared book passages. Explain why an agent needs external evidence, observe a source-grounded answer, and inspect whether its citations actually support its claims. Day trading remains the teaching vehicle, not a trading recommendation or permission to build order execution.

The instructor has authorized this integration using the **provisional three-PDF, 511-passage test collection**. This is not a final corpus freeze, a three-model comparison, EPUB integration, or proof of retrieval quality. Historical planning/preparation-only restrictions in [PLAN.md](PLAN.md) and [the handoff](../final-lessons-handoff.md) describe earlier phases; this lesson covers the separately authorized runtime continuation. Read [README](../../README.md) for the current roadmap.

The instructor supplies the [complete reference source tree](src/), retrieval support, and [three editable teaching files](content/) before class. Root `src/` is an ephemeral exercise copy, not the authoritative Lesson 12 reference; the combined Lessons 12–14 modularization finishes with Lesson 14 active at root. Activate this lesson explicitly before running it. Predict → run → explain → tweak → run again. There are **no missing Rust scaffolding exercises** and no requirement to write ingestion, database, async, or presentation machinery during class.

## Concepts introduced

An **embedding** is a model-produced numeric representation of text. The application embeds the learner's search question so it can compare that question with already embedded passages. This does not retrain the tutor.

An **embedding space** is the complete compatible model and preprocessing contract, not merely a vector length. Passage and query embeddings must use the same selected model, runtime profile, formatting, dimensions, and normalization. Two 1,024-number vectors are not automatically comparable.

**Retrieval** selects candidate source passages to accompany the tutor's next inference. A **passage** or **chunk** is a bounded portion of extracted source text. A **collection** groups passages belonging to one preparation/profile. Retrieval adds evidence to the existing conversation; it does not replace the learner's consented learning record.

**Provenance** describes where evidence came from: stable passage identity, document/source identity, physical page locations, preparation/profile and available metadata. A **citation** is a reference to that evidence. Provenance lets an instructor inspect a claim instead of trusting fluent prose.

**Cosine distance** measures the difference in direction between embeddings; lower distance means closer vectors. It is not a probability of relevance, correctness, or mastery. **HNSW** is an approximate nearest-neighbor indexing algorithm used to find candidates efficiently. It is the index, not the distance metric; a small database may still be scanned by the PostgreSQL planner.

The **read-only retrieval boundary** is application code that accepts a narrow search query, uses approved local services and parameters, and returns bounded evidence. It is needed because a model request is untrusted input, not permission to execute arbitrary SQL or network requests.

A **source-grounded answer** connects a specific claim to an actually returned excerpt. **Source-grounded feedback** compares a learner's particular answer with that evidence and explains a supported distinction. Neither is a validated assessment of trading competence. There are no expert-authored rubrics here; seeing a tutor explanation is not evidence of learner understanding.

## Prerequisites and supplied setup

Complete Lesson 11's consent, proposal, approval and fresh-Session model. Keep its calculator, streamed Events, effect handling and GDK-owned loop. Be able to distinguish displayed activity from saved messages and model-generated proposals from local approval.

Before class, the instructor supplies and verifies:

- The pinned Rust toolchain and locked root dependencies, and the complete Lesson 12 source/support files.
- A working local tutor provider. The bundled JSON remains a replaceable default; under the existing limited CLI convention, the application selects the first declared model and accepts an optional positional provider path.
- PostgreSQL 18/pgvector 0.8.7 with the populated `harrier-llamacpp-test-511-v1` collection and a separate reader role.
- The matching local Microsoft Harrier OSS v1 0.6B F16 GGUF/llama.cpp embedding service on port 9999. It is separate from the tutor service on the bundled configuration's port 8080.
- Authorized access to the private source material and artifacts. A Git clone does not contain the books, extracted passages, embeddings, database contents, or private credentials.

The instructor follows [DATABASE-SETUP.md](DATABASE-SETUP.md) for startup, loading, identity checks, reader query and shutdown, and [RETRIEVAL.md](../../embedding-code/RETRIEVAL.md) for preparation evidence. **Learners do not install PostgreSQL, convert models, extract books, design chunks, or tune indexes in this lesson.** The three PDFs are sufficient for this provisional integration; EPUB preparation is future corpus work, not a blocker requiring learners to implement it now.

Activate the **complete source tree**, including the retrieval test subdirectory, from a repository-root Bash shell. This replaces the ephemeral exercise; preserve unrelated work outside `src/`. No backup/restore or final hello-world reset is required:

```bash
rm -rf src
mkdir src
cp -R lessons/12-specialized-retrieval/src/. src/
```

Then, from a prepared repository-root Bash shell:

```bash
umask 077
source day_trading/embeddings/postgres-local/connection.env
export RETRIEVAL_DATABASE_URL="$READER_DATABASE_URL"
export HARRIER_URL=http://127.0.0.1:9999
export TUTOR_URL=http://127.0.0.1:8080
cargo run -- custom_aa_llama_qwen3_6-35b.json
```

The tutor JSON must select a literal-loopback OpenAI-compatible service. The application rejects nonempty proxy environment variables and constrains provider redirects to the same local origin; the retrieval HTTP client disables proxies and redirects. `TUTOR_URL` must exactly match the provider origin (trailing slash aside) and expose the llama.cpp `/tokenize` API. This is a deliberate Lesson 12 local-service restriction, not general remote-provider support.

If the reader readiness check rejects inherited temporary-table privileges on an older prepared database, the instructor applies `REVOKE TEMPORARY ON DATABASE harrier FROM PUBLIC;` using the admin preparation shell. The supplied `embedding-code/postgres.sh up` now includes that restriction. Do not weaken the reader check or use admin credentials in the learner application. No corpus reload is required.

The private sourced file supplies `READER_DATABASE_URL`; the launch exports it as `RETRIEVAL_DATABASE_URL`, the application's **reader** connection, never the loader's admin `DATABASE_URL`. Do not invent a password, print the file, enable shell tracing, or project secret-bearing environment values. `TUTOR_URL` identifies the actual local tutor's tokenizer service and must match the tutor selected by provider JSON; it is not a second generation provider. The embedding client verifies the prepared GGUF/profile using the local model path reported by the running server; no extra `HARRIER_GGUF` variable is assumed. The preparation guide describes the verified file/hash checks.

Run checks from root as maintenance, not Rust fill-ins:

```bash
cargo fmt --check
cargo check
cargo clippy --all-targets
cargo test
```

These commands check deterministic code boundaries; actual integration outcomes are recorded below. Live datastore, embedding and tutor evidence must be recorded separately. If a service or the authorized private corpus is unavailable, ask the instructor; synthetic tests do not silently substitute for this live run.

To reactivate the supplied code after a classroom tweak, repeat the complete recursive activation above. Copying only `main.rs` and `retrieval.rs` leaves out required support and tests.

The shared manifest now adds async `reqwest` 0.13.5 with JSON/rustls/stream support, `tokio-postgres` 0.7 with JSON integration, and `sha2` 0.10. Cargo.lock fixes the actual resolved versions. The GDK, RMCP, toolchain, provider JSON and earlier references are unchanged. The separate preparation package still uses its own blocking clients; those are not called inside the agent tool handler.

## Supplied source map and teaching reading path

A **module** is a named part of this Rust application that groups related code. Local modules let us keep the agent's decisions visible while reading already-taught calculation and supplied UI/storage mechanics separately. They are files in this lesson, not a shared course library, another agent, or a new execution loop. Every Rust source starts with `// Lesson 12`.

| Source | Responsibility |
| --- | --- |
| [main.rs](src/main.rs) | Context loading/prompt assembly and consent; Session/runtime/Effects; concurrent Event consumption; machine assembly/run/cancellation; tool registration/dispatch/allowlists/limits; shared evidence ledger; current-turn citation inspection; genuine-learner-source validation and post-run approval sequence. |
| [retrieval.rs](src/retrieval.rs) | Cohesive local read-only adapter: strict query, endpoint/profile/privilege/provenance checks, embedding/SQL, tokenizer accounting, evidence clipping and connection-driver cancellation/Drop guards. |
| [calculator.rs](src/calculator.rs) | Previously taught exact-decimal calculation, strict arguments and capability schema; two local tests. |
| [learning_record.rs](src/learning_record.rs) | Record categories/proposal shape/schema, bounded file reading/appending and default-no confirmation-input mechanics; two local tests. It does not decide learner-source validity or when approval may happen. |
| [chooser.rs](src/chooser.rs) | Supplied arrow/Enter chooser, numbered fallback, input and raw-mode cleanup. |
| [presentation.rs](src/presentation.rs) | Palette, independent terminal gating and streaming `TerminalProse` formatting; two local tests. Event consumption stays in main. |
| [provider_setup.rs](src/provider_setup.rs) | Positional provider-path handling and first configured model selection. Local tutor policy/transport checks stay in main. |
| [diagnostics.rs](src/diagnostics.rs) | Plain Markdown log writing, complete Session rendering and presentation sanitization; two local tests. It is never learner memory. |
| [tests.rs](src/tests.rs) | Application/runtime/Event/dispatch/memory/citation/evidence-ledger tests and shared test helpers; includes one opt-in same-snapshot live search-batch test. |
| [retrieval/tests.rs](src/retrieval/tests.rs) | Adapter-local query/URL/database/vector/profile/provenance/serialization/cancellation tests, plus one opt-in actual-service smoke test. |

These declarations in main name supplied local files; learners do not reconstruct them:

```rust
mod calculator;
mod chooser;
mod diagnostics;
mod learning_record;
mod presentation;
mod provider_setup;
mod retrieval;
#[cfg(test)]
mod tests;
```

**Teaching reading path:**

1. In main, read `LessonContext::load`/`prompt_parts` and `run_learning_partner`: editable context, explicit consent, fresh selection versus retained replies, and local-service startup policy.
2. Stay in main for `ChatSession`, `ChatRuntime`, `ChatEffect`, `run_turn` and `display_events`: saving Effects is distinct from consuming Events concurrently. Follow timeout/cancellation, sender closure, draining and partial diagnostics before inspection; module extraction changes none of that order.
3. Trace main's two ToolProvider registrations, `SafeTools`, `RunLimit`, `RetrievalToolProvider` and `EvidenceBudget`. The budget resets before each run and remains shared across same-snapshot calls. Then open retrieval's cohesive adapter for the bounded service/SQL/provenance path; do not split that algorithm into classroom scaffolding.
4. Return to `inspect_turn`/`inspect_citations` in main: only successful correlated search results from this learner turn authorize citation IDs. Membership warnings do not certify semantic support.
5. Read `LossToolProvider`, `parse_record_proposal` and `approve_record_proposals` in main. The handler only prepares a proposal; its quotation must come from an actual learner reply, not kickoff/tutor/tool/source text. A successful final answer and inspection precede local default-no approval, decision logging, the append call, current Session record refresh and complete post-approval diagnostics. Storage mechanics in `learning_record` cannot authorize that sequence.
6. Consult calculator/chooser/presentation/provider_setup/diagnostics support only as needed. Finish with application and retrieval child tests plus inline support tests: default execution has 40 deterministic tests and two ignored live tests. Synthetic fixtures do not replace real services or human teaching review.

## 12.1 — Predict which question needs evidence

Select **Getting started in day trading**. Decide separately whether to use optional memory; blank consent means no. The opening and any consented resumption remain a normal short learning conversation.

Contrast two questions before running:

1. “What is a bid-ask spread?” Ordinary course-authored context may be enough; retrieval is not forced on every turn.
2. “What do the supplied books say about liquidity and the bid-ask spread?” This requests source-grounded support; predict a search and citations beside supported claims.

Ask the second question. Observe the **actual** selection pattern. A missing or unnecessary search is a finding, not something to hide by forcing a request. The learner terminal should stay simple: `Tutor:`, `Learner >`, brief **Searching educational sources** activity, a returned-passage count/status, and short tutor prose. No application-added protocol fences, actor arrows, correlation IDs, raw arguments, vectors or Session dumps belong in this dialogue. An evidence marker is a source citation, not a tool request ID.

Do not promise exact model wording, a specific winning book, or an exact result count. Search may return at most five candidates; fewer can fit the remaining evidence budget. A nearest-neighbor result is still only a candidate.

## 12.2 — Explain the bounded search round trip

A **tool request** is structured model output asking for a capability. A **correlation ID** connects that request with its response; it does not establish whether the response supports the answer. Both concepts carry forward from the earlier tool lessons.

The application-owned tool name is `search_educational_passages`. Its sole model-controlled input is a focused `query`. Here is an illustrative request argument object, not captured live output:

```json
{"query":"How does liquidity affect the bid-ask spread?"}
```

Unknown fields are rejected. The query must be nonblank and no more than **2,000 UTF-8 bytes**, not 2,000 characters. The model cannot supply SQL, a database identifier, a path, an endpoint, credentials, a collection name, or top-k.

Narrate the current flow:

1. The local tutor asks to search; SafeTools checks all request blocks and the shared allowlist/request limits.
2. The retrieval adapter validates the query and selected collection/runtime contract.
3. Rust prepares Harrier's query instruction and obtains a matching local query embedding. PostgreSQL does not embed text.
4. A parameterized, read-only PostgreSQL query searches the fixed collection using cosine distance and the supplied HNSW index.
5. The adapter returns bounded excerpts with provenance, citation IDs, cosine similarity (`1 - distance`) and clipping information, preserving the correlation ID.
6. The GDK re-evaluates saved state and the tutor streams a final answer using returned evidence.
7. Only afterward can a successful memory proposal receive a separate local save decision.

A **ToolProvider** supplies tool definitions and implementations to the GDK. Lesson 12 adds a separate retrieval provider beside the existing calculator/consented proposal provider. The pinned API permits multiple providers; this is not a second agent loop. In `goose-agent = 0.1.0-alpha.11`, the registration method is:

```rust
pub fn with_provider(mut self, provider: Arc<dyn ToolProvider<S>>) -> Self
```

This is a focused SDK signature, not a line to fill in. Read the supplied application's registration and SafeTools wrapper. The SDK advertises definitions, dispatches known requests and constructs correlated responses; the application owns validation, privileges, evidence budgets and rejection policy. In this exact release the SDK filters unknown names and does not impose the course's request cap, so removing SafeTools would lose safeguards.

`StateMachine::run` still owns within-turn iteration: load state, select the first applicable step, apply Effects, and reload/re-evaluate. **Effects** describe saved changes; **Events** carry live presentation activity through the Emitter while the run executes. A search activity Event is not evidence saved by itself. Supplied async HTTP/database support keeps service waits inside that existing machinery; attendees need not write concurrency infrastructure.

### Bounds and what they mean

A **token budget** limits model-token contribution, not character or word count. The application counts actual tokens for the serialized evidence, including provenance JSON, rather than estimating from excerpt length. The supplied adapter uses the **actual tutor tokenizer** through its local `/tokenize` endpoint for excerpts and the complete compact response JSON, including the `used_tokens` field itself. Harrier's tokenizer separately preflights the embedding input. Neither counting step is a declaration of the tutor's full context capacity.

| Boundary | Application-owned limit |
| --- | --- |
| Search query | Nonblank; 2,000 UTF-8 bytes maximum |
| Candidate results | At most five, fixed by the application |
| Each returned excerpt | At most 1,000 tokens; clipping labeled |
| Evidence across all searches in one learner turn | 4,000 actual tokens, including provenance/JSON serialization |
| Each embedding/tokenizer HTTP request | 15 seconds |
| SQL wait | 5 seconds, including server-side statement timeout |
| All tools together | Four requests per turn, not four per provider |
| Within-run work | Eight applied work passes, plus a possible terminal halt batch |
| Entire run | 180 seconds |

Calculator, proposal and retrieval requests share the aggregate limits. A supplied async-mutex ledger outside conversation shares the allowance across calls, including a batch whose handlers receive the same Session snapshot; it resets only before a new learner run. The serialized canonical tool-result JSON is counted before returning and charged to that ledger. Failed calls return correlated sanitized errors rather than evidence; they do not charge successful-evidence tokens. Commands cannot overlap runs in this single-process workflow. Multiple searches do not each receive a fresh 4,000-token allowance. These bounds limit new evidence in one turn; they do not compact accumulated history or guarantee that an arbitrarily long conversation fits the tutor. HTTP requests have a 15-second deadline each; several tokenizer calls can occur within the unchanged 180-second run bound. SQL also has a server-side statement timeout. An owned connection-driver task is aborted on drop, SQL timeout, or cancellation during an outstanding SQL query. The startup GGUF checksum uses one supplied blocking file task; cancellation cannot terminate that file task once running. Timeout/cancellation is not rollback or proof a service stopped executing. Do not assign deliberate service-breakage exercises.

## 12.3 — Inspect citations and support

A **stable passage ID** identifies the saved draft passage under its preparation identity. The tutor uses the exact returned marker **`[source:<chunk_id>]`**, substituting only an actually returned chunk ID. The angle-bracket form here is explanatory, not a real citation to paste into an answer. Changing/finalizing a collection can change identities; stable does not mean eternal across corpus revisions.

The tutor must cite only passages returned for the current learner turn, beside the claims they support. If evidence from an older turn is needed again, search again; do not invent or recycle a citation outside the current returned set. Titles/authors/publication information are used only when verified and available. **Physical PDF pages** count from the first PDF page, including front matter; they are not the book's printed page labels. No inferred chapter, EPUB page, fabricated edition, or chart interpretation is allowed.

An **excerpt clipping label** tells us that only a prefix of a stored passage was returned. The supplied token-bound prefix can end within a sentence; do not present a clipped prefix as the complete explanation or invent the missing continuation. The full stored content hash and raw fragment mapping identify the original passage, not a new uncropped quotation.

The instructor privately opens `session-output.md` after the run and checks:

- Which search query actually ran and which collection/profile it used.
- The returned IDs, exact excerpts, source identities, physical pages and clipping labels.
- Whether each cited ID belongs to returned evidence and whether that evidence supports the adjacent claim.
- Whether the tutor separated source claims from course-authored background and acknowledged dated or incomplete support.

**Membership is not semantic support.** A valid ID can be attached to an unsupported claim. A high similarity score is not a certified relevance threshold. No universal threshold is used to declare evidence sufficient or absent.

Current collection reality:

| PDF | Physical pages | Draft passages |
| --- | ---: | ---: |
| DayTrading.com beginners guide | 18 | 56 |
| Markus Heitkoetter, The Complete Guide to Day Trading | 273 | 273 |
| Corporate Finance Institute, The Complete Guide to Trading | 116 | 182 |
| **Total** | **407** | **511** |

Full layout/reading-order acceptance, remaining cleanup and alternative chunk-policy comparisons remain open. Retrieval can recover an oversimplified claim: the beginners guide's stop-loss wording is a known example. Do not endorse guaranteed realized-loss ceilings merely because a passage says so. Ask what the returned source actually says, what qualification is missing, and which caution comes from the fixed course material.

## 12.4 — Give feedback without declaring mastery

Try a source-grounded learning question: “Compare my statement with the supplied sources: a stop means I cannot lose more than the planned loss. Which qualification am I missing?”

Predict a focused comparison, not a readiness score. Inspect the actual excerpts before deciding the answer is grounded. Good feedback identifies the learner's specific claim, describes supported or missing qualifications, and invites a short explanation in the learner's own words. The calculator remains optional for complete hypothetical numeric inputs: entry `51.20`, stop `50.70`, `200` shares means maximum **planned** loss `$100.00`, excluding fees, slippage and gaps. That value is not statistical expected loss or a guaranteed realized-loss ceiling.

If excerpts do not support a requested detail, the tutor should say the returned evidence is insufficient and ask a narrower question. If the embedding/database service is unavailable, it should say search was unavailable—not claim the books contain no answer. This distinction is a teaching topic, not a prescribed failure exercise. Show a source disagreement only if actual evidence permits it; do not manufacture one for class.

## 12.5 — Preserve consent and distrust source instructions

**Untrusted retrieved text** is evidence that may include instruction-like content; it cannot override application instructions or expand capabilities. Approved learning notes are also data, not instructions. Framing tells the tutor how to treat them but is not proof of prompt-injection immunity.

Memory consent remains default-no **per selection**. It gates reading the fixed-path record and advertising proposals, independently of source retrieval. Search neither enables memory nor grants approval. A quotation from a book, a tool response, or the tutor is not an actual learner quotation. The strict proposal check still requires actual learner words; source occurrence is not semantic agreement or mastery.

A proposal only acknowledges preparation, without writing. After a successful ordinary final answer and inspection, each addition requires separate default-no local approval. Approved appends preserve hand edits; corrections/deletions happen between launches. The 32 KiB, regular UTF-8/no-symlink, new-file 0600, existing-permissions-unchanged and single-process durability limits from Lesson 11 remain. No crash-recovery or exactly-once promise is added by retrieval.

Selection starts a fresh Session; replies retain that Session. `/finish` returns to selection; `/quit` and EOF exit; blank replies are ignored. Three content files are loaded at selection and fixed for that Session. Old diagnostics are never restored as memory. No compaction, new CLI, browsing, arbitrary SQL/file/network capability, market data or trade execution is introduced.

## 12.6 — Approved tweak and second run

Choose **one** small tweak:

- Ask the same source-grounded question differently, for example “How do the supplied books connect ease of trading with the gap between buying and selling quotes?” Predict whether evidence remains relevant; compare actual support, not identical rankings or wording.
- Edit the pacing sentence in `content/getting-started-guidance.md` to “Use a short hypothetical example before a check-for-understanding question.” Start a **fresh selection** to load the edit, ask a supported question, and inspect pacing and citations separately. Restore the default after rehearsal if desired.

Do not change model profiles, collection identities, SQL, service timeouts or safeguards as a classroom tweak. Do not introduce infrastructure-failure exercises. Record inconclusive behavior honestly; fluent output is not automatic success.

## Privacy, diagnostics and source rights

Queries can contain learner text, and excerpts return to the local tutor. Use approved local endpoints only, no external inference fallback. Loopback client URLs do not fix a server listening on all interfaces: the instructor must inspect exposure and protect any explicitly approved LAN topology. The existing embedding process was reported bound to `0.0.0.0`; setup guidance uses loopback and does not automatically restart it.

Private PDFs, extracted text, embeddings, exports and learner conversations are not public course assets. Ownership is not redistribution permission. Confirm processing/disclosure and authorized learner access before class. Git ignore is not encryption or access security. Do not project private diagnostics without permission; prepare sanitized examples instead.

`session-output.md` remains plain Markdown, overwritten at launch and appended during selections/turns. It contains loaded context, complete before/after/post-approval Session Debug values including nested/default fields and usage, correlated requests/results, retrieval provenance/profile, assembled answers, decisions, warnings and available partial failures. It contains no application ANSI, deliberate credential/configuration dump, secret-bearing database URL or embedding-vector dump. It is never read as learner memory. Diagnostic/storage failures stop the application; unavailable logging cannot guarantee recording its own failure.

The learner UI keeps Lesson 11's Tutor/Learner simplicity, presentation-only course colors and independent stdout/stderr gating, with plain fallback for redirected output, any `NO_COLOR`, and `TERM=dumb`. Common generated Markdown suppression is not a full renderer; verify exact citation markers survive it. Saved/provider text remains unchanged.

## Success criteria and validation record

**Structural success:** the actual prepared reader datastore and matching local embeddings are exercised; strict query shape and application-owned scope hold; results are bounded with traceable provenance; all tools share limits/correlation safeguards; citations belong to returned passages; learner UI stays simple; complete diagnostics remain plain/separate; memory consent and post-run local approval remain unchanged.

**Human/live review:** the chosen evidence supports the adjacent claim; limited/no support is honestly explained; outages are distinguished from absence; source qualifications are preserved; feedback describes a particular learner answer without certifying mastery; actual tool selection, privacy and readability are reported rather than forced.

Preparation already has recorded real 511-vector loading, idempotence, reader queries and profile-isolation evidence in RETRIEVAL.md. The older `harrier-test-511-v1` Transformers collection is preserved and **incompatible** with the active llama.cpp profile; do not mix them. Preparation smoke checks do not validate this new tutor integration, cross-runtime parity, expert source accuracy, or a final corpus freeze. The implementation outcomes below distinguish structural checks from fallible teaching behavior. End-of-course content/teaching/UI review remains deferred, not completed; previous lessons' validation gaps remain independent.

### Implementation validation — 2026-10-12

- Root formatting, locked Cargo check, Clippy across all targets, IDE build and **40 deterministic tests** passed; **two opt-in local integration tests** also passed explicitly. They exercise actual query embeddings/SQL, source integrity, tutor-token counting including canonical JSON/provenance, excerpt clipping bounds, budget-limited omission, and two searches in one SDK batch sharing a single 4,000-token ledger. Public fixtures are course-authored inline test data, not book excerpts. Preparation regression tests also passed (32 automatic tests; its two separately opted-in corpus tests were not rerun in this integration pass).
- The real local liquidity/spread question selected search, returned three candidate excerpts using **2,555 tutor tokens**, and cited the two relevant CFI passages at physical pages 41 and 23. The adjacent explanation was inspected against those exact returned excerpts. The third candidate was irrelevant; top-k is not a relevance guarantee. The optional calculator then returned a correlated **$100.00** with exclusions. No record was read or proposed with declined consent.
- A rephrased source question searched and cited returned IDs, but blurred the forex passage into stock/share language. A source-grounded misconception-feedback run corrected the low-liquidity/wide-spread relationship without a mastery claim, yet its opening sentence incorrectly called the relationship “direct” and reversed the learner's stated relationship; its subsequent explanation was not fully supported by the cited excerpts. **These are teaching/support defects, not deterministic citation failures.** Citation membership cannot certify explanation quality.
- A dated AAPL price question declined unsupported market-data claims and gave no fabricated price, but did **not** search despite the requested search. Its blanket claim about what the books contain was not established by retrieval. Honest refusal was observed; source-absence validation was not.
- Live consented proposal preparation did not write before local approval; approved append succeeded; an independently declined second addition preserved bytes. A hand edit was loaded into a fresh launch's prompt context, but the welcome did **not** explicitly offer the edited activity. Memory transport/storage passed; memory-aware resumption teaching remains incomplete. The disposable test record was removed. `/finish` reselection, `/quit`, plain/empty-NO_COLOR/TERM=dumb dialogue and diagnostics were exercised; deterministic EOF/blank-input tests passed.
- Colored arrow/Enter selection and source-marker display passed on retry. The first colored run returned evidence but no ordinary final tutor answer; the application stopped, recorded complete available partial state, and offered no approval. Do not hide that reliability failure. Redirected/plain logs and all checked Markdown diagnostics contained no application ANSI; citations survived the streaming presentation helper.
- Startup rejected the reader's inherited PUBLIC temporary-table privilege. The instructor-side startup script now revokes TEMPORARY on this dedicated database; the same restriction was applied to the existing database without changing its corpus. The stricter reader check then passed. Added response model/profile provenance made a former 150-token empty-result test allowance too small; the corrected test verifies budget-limited output at 400 and an explicit error at 150. No silent over-budget output was permitted.

The inspected embedding runtime has eight slots and **2,048 tokens per slot**; the documented 8,192/8 startup example has 1,024 per slot. Runtime preflight uses the actual reported capacity, never training context. Existing services were not restarted; the user's Harrier process still listens on all interfaces. Clients use loopback and do not fall back externally, but operators must restrict server exposure. No offline-with-network-disabled test, full injected-outage/malformed-service matrix, exhaustive citation support benchmark, cross-runtime parity, index recall/latency benchmark, or corpus freeze is claimed. Rights and authorized learner access remain the operator's responsibility. Content/teaching/UI and instructor release review remain deferred to end-of-course review; prior lesson gaps are independent. No Lesson 13/14 code was introduced.

Private raw evidence remains in ignored `day_trading/embeddings/postgres-local/lesson12-*` captures, not public lesson examples. Diagnostics can contain private excerpts and learner text; Git ignore is not access control.

### Modularization validation — 2026-10-22

- Behavior-preserving local extraction passed `cargo fmt --check`, `cargo check`, `cargo clippy --all-targets`, `cargo test` and `cargo build`: **40 deterministic tests passed; two live tests were ignored by default**. The source audit preserved all **158 original function/test bodies**, test names and constants. Literals, types and attributes were preserved apart from necessary visibility/import changes and test `cfg` placement. No prompt, content, dependency, budget, authority or semantic fix was made.
- Both ignored real-service tests were explicitly attempted and failed because **`RETRIEVAL_DATABASE_URL` was missing**. `cargo run -- custom_aa_llama_qwen3_6-35b.json` failed at the same prerequisite **before the chooser**. The private `connection.env` was absent; Harrier on port 9999 was unavailable; the tutor on port 8080 returned health HTTP 200. Tutor health alone is not retrieval readiness. No mocks were substituted, services restarted or corpus mutated.
- Plain, empty-`NO_COLOR` and `TERM=dumb` launch-error output and independent stderr terminal gating were checked. This is error-path presentation evidence, **not** a fresh conversation, chooser, consent/storage, search, calculator or teaching validation. The historical successful integration and semantic defects above remain historical evidence, not fresh modularized live success. No fresh IDE/projector or release review is claimed.
- Root activation uses the entire recursive source tree, not a main/retrieval pair; the combined refactor's final root is Lesson 14. Lesson 9's default and 100-share live runs passed on 2026-10-22, resolving that earlier validation gap independently. Other historical teaching defects and deferred instructor review remain; this pass grants no release approval.

## Exit questions and next step

- Why must a query embedding match the passage embedding space, not just its dimension?
- What differs between HNSW, cosine distance, citation membership and semantic support?
- Why can a retrieved passage be relevant but still oversimplified or outdated?
- Which information can authorize a memory addition? Can a source passage supply learner agreement?
- What does the per-turn evidence cap limit, and what long-history problem does it not solve?

Next, Lesson 13 will manage growing working context while preserving provenance, request/result pairing and local approvals. That is future behavior, not automatic GDK compaction or part of this lesson.

Supplemental official orientation: [GDK SDK overview](https://goose-docs.ai/docs/gdk/sdk), [SDK API reference](https://goose-docs.ai/docs/gdk/sdk/api-reference). These pages orient the provider layer; exact state-machine/tool behavior here was checked against the pinned `goose-agent` source, not inferred from full Goose application features.
