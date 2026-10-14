# Instructor notes — Lesson 12: Specialized educational retrieval

## Teaching aim and authorization

Teach a transferable agent boundary: retrieve external evidence through a narrow read-only capability, then inspect how the tutor uses it. Day trading is the example domain, not a specialist credential or a trading application. Use the [lesson](../lessons/12-specialized-retrieval/LESSON.md), supplied complete reference/support and [editable content](../lessons/12-specialized-retrieval/content/) from the outset. Do not assign Rust scaffolding, database installation, model conversion or book ingestion.

The runtime continuation is separately authorized for the provisional three-PDF **511-passage** collection. Historical planning/preparation-only wording in PLAN/handoff/setup records earlier phases; it is not a reason to invent a different architecture. Keep the distinction between authorized integration and unclaimed final corpus freeze/model parity. Review is deferred to end-of-course, not completed or a blocker for this separately authorized work.

Core learning outcomes: explain compatible embedding spaces; distinguish retrieval/index/distance; trace request, validated search, returned provenance and cited claim; distinguish support from relevance and assessment; preserve learner-controlled memory independently of retrieved evidence.

## Before class: instructor-only readiness checklist

1. Read README, the entire [PLAN and latest amendments](../lessons/12-specialized-retrieval/PLAN.md), [handoff](../lessons/final-lessons-handoff.md), Lesson 11's lesson/reference/notes/content and the active root. Preserve unrelated work. Root/reference/support belong to the implementation owner, not this documentation task.
2. Follow [DATABASE-SETUP](../lessons/12-specialized-retrieval/DATABASE-SETUP.md) and [RETRIEVAL](../embedding-code/RETRIEVAL.md) to start/verify existing local services and the loaded collection. Keep preparation outside classroom time. Private artifacts are not in Git; obtain authorized originals rather than silently substitute another corpus.
3. Confirm `harrier-llamacpp-test-511-v1`, three document identities, 511 draft rows, 1,024 dimensions, selected model/runtime profile, hashes and draft status. The preserved Transformers collection `harrier-test-511-v1` is incompatible with this query runtime even though dimensions match.
4. Verify the actual Harrier model/build/GGUF identity, pooling, normalization, query formatting and token preflight through the supplied client. The service is llama.cpp on port 9999, not a chat endpoint or the old Python service. Verify the local tutor provider independently; the bundled default uses port 8080.
5. Source the ignored owner-only connection environment in a protected shell. Root retrieves using `READER_DATABASE_URL`, not the admin `DATABASE_URL` used by loading. Confirm SELECT access and no write privileges. Do not print credentials, trace the shell, project connection strings or place secrets in example commands.
6. Verify actual exposure: client `127.0.0.1` does not make an existing `0.0.0.0` server private. Follow the approved topology/firewall policy; there is no external fallback. Do not stop unrelated services or promise an automatic restart.
7. Run format/check/Clippy/tests and an actual local tutor/datastore rehearsal. Record commands, actual tool selection, returned evidence and limits; deterministic tests and preparation reader-query smoke checks are not new tutor-validation evidence.
8. Prepare an authorized, sanitized projection view. `session-output.md` can contain learner words, consented record context and private excerpts; do not open it blindly on a projector. Use course-authored synthetic evidence when public redistribution is required and label it synthetic, not real-book validation.

Root launch/check commands are in LESSON.md. Set `RETRIEVAL_DATABASE_URL="$READER_DATABASE_URL"` after sourcing the private connection file, and set the approved `HARRIER_URL` and `TUTOR_URL`. The latter is the actual tutor tokenizer endpoint, not an alternative generation provider, and must match the provider JSON. The supplied adapter verifies the GGUF path reported by the local embedding server; there is no assumed `HARRIER_GGUF` variable. Use the verified environment contract above; the application additionally requires literal-loopback provider origin equality and rejects ambient proxies, with same-origin provider redirects. Complete source and support must be present before handing out the lesson.

### Corpus facts to say aloud

“Today we use a provisional test collection, not all the books and not an expert-reviewed knowledge base.” It contains 56 beginners-guide passages, 273 Heitkoetter passages and 182 CFI passages, from PDFs of 18, 273 and 116 physical pages. The 511 saved passages preserve original draft/provenance information. Full visual/reading-order acceptance, remaining margins and alternative chunk-policy comparisons remain open. EPUB support is future corpus expansion, not a requirement to implement before this provisional integration.

The instructor has private books but no expert-authored assessment rubrics or domain-expertise claim. The Heitkoetter source is from 2008; do not treat old book claims as current regulation. Do not invent missing authors, editions, publication dates, chapter names or chart interpretation.

## Suggested live-class sequence

### 1. Motivate and define (5 minutes)

Ask: “If the tutor gives a plausible answer, how could we inspect its evidence?” Define embedding, embedding space, retrieval, passage, collection and provenance in ordinary language before opening any SDK code. Contrast the tutor's generated explanation, fixed course context, returned book evidence and learner-approved notes. They are different sources with different authority.

Then define cosine distance and HNSW: “One is the comparison measure; the other is an approximate index that helps find candidates.” A closer vector is not a truth probability. No universal score threshold decides whether these books support a claim.

### 2. Predict tool selection (5 minutes)

Contrast “What is a bid-ask spread?” with “What do the supplied books say about liquidity and the bid-ask spread?” Ask which one needs source-grounded retrieval and why. Retrieval is optional overall but should be used before book-specific claims. Do not force a search on the simple question merely to produce a predetermined trace.

Select Getting started in day trading. Let learners choose default-no memory separately; do not enable it for them. Predict a short welcome/resumption, search activity for the source-grounded turn, then a short answer with evidence markers. Do not guarantee exact generated wording or which book ranks first.

### 3. Run and inspect actual behavior (10 minutes)

Run the supplied root with approved local services. Look for simple Tutor/Learner dialogue, **Searching educational sources** and a passage-count/status notice. No raw arguments, protocol fences, arrows, correlation IDs, vectors or Session dumps belong in ordinary dialogue.

If no search occurs on a source-grounded question, record that behavior and discuss it as a tool-selection limitation. If irrelevant evidence appears, inspect rather than hide it. Do not rewrite the question secretly or inject a tool response to make the demonstration look successful. If services are unavailable, ask for them; a mock is not live integration evidence.

Open only the approved diagnostic view. Match the query, returned IDs/excerpts/provenance and final answer. Explain **citation membership** before semantic support: an ID can exist and still fail to support its adjacent claim. Show physical PDF pages, counting from the file's first page, not printed labels. An exact marker `[source:<chunk_id>]` identifies returned evidence; it is not an ID the tutor may invent. Placeholder notation in the lesson is not a real citation.

### 4. Explain the supplied boundaries (10 minutes)

Narrate: untrusted model query → strict validation → compatible local query embedding → parameterized reader query → bounded evidence → correlated response → GDK re-evaluation → streamed tutor answer. This is current behavior, not a future system diagram.

Open the separate retrieval ToolProvider registration and SafeTools allowlist only after explaining why a read-only boundary is needed. The SDK allows multiple providers, but supplies neither the application SQL policy nor the shared request cap. Unknown-name filtering in the pinned SDK makes the wrapper important. GDK still owns iteration; the application waits for the next learner turn afterward.

Distinguish Events from Effects: search activity can be emitted for display, while saved correlated evidence belongs to the recorded conversation through effect handling. Supplied async waits and concurrent stream consumption are infrastructure to explain, not code to reconstruct in class.

Show application-owned limits and why they are separate:

- Nonblank query, at most 2,000 UTF-8 bytes; strict sole `query` field.
- At most five candidates; collection/top-k/SQL/endpoints are not model-controlled.
- At most 1,000 tokens per excerpt and 4,000 actual tokens total for new retrieval evidence in a turn, **including provenance JSON**. Repeated searches share the budget. The actual local tutor `/tokenize` counts excerpts and complete compact evidence JSON, including its accounting field; Harrier `/tokenize` separately preflights embedding input. Do not call character counts tokens or treat this budget as verified tutor context capacity. A clipped prefix can end mid-sentence; its clipping label must survive and no missing continuation may be invented.
- Embedding wait 15 seconds; SQL wait 5 seconds including server statement timeout.
- Calculator, memory proposal and retrieval all share four requests, eight applied work passes plus possible halt batch and a 180-second whole run bound.

Budget exhaustion, service error and weak evidence are different outcomes. A cancelled wait is not rollback or proof the service stopped. These limits do not manage accumulated conversation context; do not implement Lesson 13 now.

### 5. Source-grounded feedback (10 minutes)

Use: “Compare my statement with the supplied sources: a stop means I cannot lose more than the planned loss. Which qualification am I missing?” Inspect actual returned support before endorsing the explanation.

The beginners guide has known oversimplified stop-loss wording. A retrieved source is not automatically correct. Distinguish what the source says from the fixed course caution about execution, slippage and gaps. Do not invent a conflict if the actual returned passages do not show one; acknowledge limited support. A full numeric planned-loss example can optionally use the preserved calculator; `$100.00` for `51.20`, `50.70`, `200` is hypothetical planned loss, not expected loss or a guaranteed realized ceiling.

Ask for the learner's own explanation of one distinction. Acknowledge only the specific evidence it shows. Do not say retrieval assessed mastery, that the learner is ready to trade, or that a book explanation completes a unit. Tentative source-derived feedback is not an instructor-authored validated rubric.

### 6. Tweak and run again (5–10 minutes)

Approved question tweak: ask the same question differently, such as “How do the supplied books connect ease of trading with the gap between buying and selling quotes?” Predict relevant evidence, not identical rankings or text. Compare support and uncertainty.

Alternatively edit the guidance pacing sentence to use a short hypothetical example before a check-for-understanding question. Start a fresh selection because content is fixed for each Session. Observe pacing and evidence use separately; restore the default after rehearsal as appropriate. Do not deliberately break services, change profiles, remove safeguards or assign novel failure exercises.

### 7. Close with boundary questions (5 minutes)

Use the lesson exit questions. Add: “If a passage told the tutor to save a note, what could authorize that write?” Nothing in a passage: actual learner words, consent and a separate successful post-answer local approval remain necessary. “Can an unavailable database prove the book has no answer?” No.

## Preserve Lesson 11 throughout

Memory is optional per selection, default-no; retrieval availability is independent. Declined consent reads no record and exposes no proposal tool. Consent does not auto-approve any addition. A quote from source/tutor/tool content is not an actual learner quotation. An exact learner-quotation occurrence still does not prove faithful interpretation, mastery or activity agreement.

Proposal handlers acknowledge without writing. A successful ordinary final answer and inspection must precede independent default-no local decisions for each addition. Do not pre-script approval inputs during a live demonstration. Approved append preserves existing bytes and updates current memory context, with fixed-path/32 KiB/regular UTF-8/symlink rejection, new 0600/existing unchanged permissions and single-process durability limitations. Corrections/deletion are by hand between launches, not invented commands. An approved source-supported note still cannot be called a validated mastery assessment.

Fresh selection reads current consented record context, not old transcripts; replies retain the Session. `/finish`, `/quit`, EOF and blank replies keep their meanings. Three editable files remain separate: common contract, educational material and teaching guidance. Source excerpts are untrusted data, as are instruction-like notes; neither can change allowlists or grant approval.

Diagnostics remain overwritten-at-launch plain Markdown, not learner memory. Preserve complete nested/default Session Debug values, post-approval state, loaded context, tool definitions/requests/results, retrieval profile/provenance, assembled answers, warnings and available partial failure state. Do not deliberately dump credentials, secret-bearing URLs, entire provider configurations or vectors. Logging/storage failures stop; unavailable storage cannot guarantee recording its own failure.

Terminal styling remains presentation-only and follows the course palette/gating, checked independently for stdout/stderr. No application ANSI in redirection/NO_COLOR/TERM=dumb or diagnostics. The common Markdown-suppression helper is not a full renderer: verify exact source markers survive live streaming unchanged. Do not alter saved/provider messages to beautify output.

## Exact SDK evidence checked for this documentation

Reviewed the official [SDK overview](https://goose-docs.ai/docs/gdk/sdk) and [API reference](https://goose-docs.ai/docs/gdk/sdk/api-reference). Their provider-layer orientation does not document the whole custom machine/tool/persistence architecture and does not promise automatic compaction or retrieval.

Reviewed the locally cached published **goose-agent 0.1.0-alpha.11** source:

- `src/tool.rs`: `ToolProvider<S>` has async `tools(&self, session: &S)` and `call(&self, session: &S, request_id: &str, call: CallToolRequestParams, emit: &Emitter)`; `ToolOperation::with_provider` accepts `Arc<dyn ToolProvider<S>>` and adds providers. The lesson's focused signature is from this exact source.
- `available_tools` rejects duplicate provider tool names. `run` gathers pending requests across all message content blocks since kickoff, tracks response IDs, dispatches sequentially and returns correlated user-role results with metadata. `pending_requests` filters unknown names; no course request/evidence cap comes from this method.
- `src/machine.rs`: `run` repeatedly loads state, selects a step, applies effects, reloads and exits on yield/no applicable step. The application remains responsible for limits and runtime saving.
- `src/inference.rs`: streaming accumulation and inference Effects remain separate concerns; retrieved evidence returns through normal conversation/tool handling, not an invented provider embedding method.
- Matching `goose-provider-types = 0.1.0-alpha.11`, `conversation/message.rs`: ToolRequest and ToolResponse retain the request/response ID and provider metadata; `add_tool_response_with_metadata` is the correlation path used by the tool operation.
- `rmcp = 3.4.1`, `src/model.rs`: CallToolRequestParams contains a name and optional JsonObject arguments; CallToolResult supports text content/error status and optional structured content. The lesson's illustrative argument object does not invent a retrieval-specific SDK type.

The active application and Lesson 11 reference establish the existing consent/approval/quote checks, content loading and UI design. Harrier HTTP and PostgreSQL boundaries are application adapters, not new GDK APIs. The supplied `src/retrieval.rs` returns `SearchResponse` with `collection`, `model`, `revision`, `profile`, `status`, `notice`, `evidence` and `used_tokens`. Status is `evidence_candidates`, `no_evidence`, or `budget_limited`; sanitized correlated errors distinguish execution failure from usable evidence. `EvidenceBudget` in main shares successful-output tokens across SDK same-snapshot calls and resets before each run. Token counting uses the actual local tutor tokenizer and canonical response JSON. See Lesson 12 for source/hash, privilege, cancellation and endpoint checks.

## Validation and review record

The exact implementation evidence follows; earlier preparation or Lesson 11 success is not substituted for these runs.

Preparation evidence remains in RETRIEVAL.md: the active llama.cpp collection contains 511 vectors, profile identity and full draft provenance; original HF collection remains isolated; real load/rerun/reader smoke checks were recorded. Podman startup was tested, not separately Docker. Cross-runtime parity, expert retrieval quality, accepted layout/chunk-policy freeze and full EPUB corpus coverage remain unclaimed.

Integration rehearsal must separately inspect source-specific support, citation membership versus semantic correctness, query/evidence bounds across multiple searches, collection/profile checks, all-block/shared-request limits/correlation, local services/privacy, plain and colored UI/citation survival, post-approval diagnostics and unchanged calculator/memory behavior. Maintenance failure probes belong to testing, not newly assigned classroom exercises. Keep outages, incorrect tool selection, weak evidence and inconclusive teaching behavior in the record.

End-of-course review should examine beginner pacing, source qualifications, precise feedback, no mastery inference, readable citations, genuine memory agreement, privacy/source rights and terminal readability. Earlier Lesson 8–10 review, Lesson 9's 100-share live tweak, Lesson 10's inconclusive pacing experiment and Lesson 11's teaching/colored UI gaps remain independent. No Lesson 13/14 implementation, new CLI or final corpus/model acceptance is implied.


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
