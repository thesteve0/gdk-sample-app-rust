# Instructor notes — Lesson 12: Specialized educational retrieval

## Teaching aim and authorization

Teach a transferable agent boundary: the application retrieves external evidence through a narrow read-only boundary and injects it per turn, then we inspect how the tutor uses it. Day trading is the example domain, not a specialist credential or a trading application. Use the [lesson](../lessons/12-specialized-retrieval/LESSON.md), supplied complete reference/support and [editable content](../lessons/12-specialized-retrieval/content/) from the outset. Do not assign Rust scaffolding, database installation, model conversion or book ingestion.

The runtime continuation is separately authorized for the provisional three-PDF **511-passage** collection. Historical planning/preparation-only wording in PLAN/handoff/setup records earlier phases; it is not a reason to invent a different architecture. Keep the distinction between authorized integration and unclaimed final corpus freeze/model parity. Review is deferred to end-of-course, not completed or a blocker for this separately authorized work. The instructor subsequently marked Lessons 1–12 complete on 2026-11-11; the recorded validation limitations and open preparation gates below remain part of the lesson's record.

Core learning outcomes: explain compatible embedding spaces; distinguish retrieval/index/distance; trace the learner question, validated search, returned provenance and cited claim; distinguish support from relevance and assessment; preserve learner-controlled memory independently of retrieved evidence.

## Before class: instructor-only readiness checklist

1. Read README, the entire [PLAN and latest amendments](../lessons/12-specialized-retrieval/PLAN.md), [handoff](../lessons/final-lessons-handoff.md), Lesson 11's lesson/reference/notes/content and the active root. Preserve unrelated work. Root `src/` is ephemeral; activate the complete Lesson 12 tree before rehearsal. The combined Lessons 12–14 refactor historically finished with Lesson 14 active; on 2026-11-11 the instructor had the root restored to this lesson's cleaned tree, so the active root should already match — verify rather than assume.
2. Follow [DATABASE-SETUP](../lessons/12-specialized-retrieval/DATABASE-SETUP.md) and [RETRIEVAL](../embedding-code/RETRIEVAL.md) to start/verify existing local services and the loaded collection. Keep preparation outside classroom time. Private artifacts are not in Git; obtain authorized originals rather than silently substitute another corpus.
3. Confirm `harrier-llamacpp-test-511-v1`, three document identities, 511 draft rows, 1,024 dimensions, selected model/runtime profile, hashes and draft status. The preserved Transformers collection `harrier-test-511-v1` is incompatible with this query runtime even though dimensions match.
4. Verify the actual Harrier model/GGUF identity, pooling, normalization, query formatting and token preflight through the supplied client. The service is llama.cpp on port 9999, not a chat endpoint or the old Python service. Verify the local tutor provider independently; the bundled default uses port 8080.
5. Use the local reader default or override `RETRIEVAL_DATABASE_URL` in a protected shell for a different prepared instance; never use the admin `DATABASE_URL` used by loading. Confirm SELECT access and no write privileges. Do not print credentials, trace the shell, project connection strings or place secrets in example commands.
6. Verify actual exposure: client `127.0.0.1` does not make an existing `0.0.0.0` server private. Follow the approved topology/firewall policy; there is no external fallback. Do not stop unrelated services or promise an automatic restart.
7. Run format/check/Clippy/tests and an actual local tutor/datastore rehearsal. Record commands, actual tool selection, returned evidence and limits; deterministic tests and preparation reader-query smoke checks are not new tutor-validation evidence.
8. Prepare an authorized, sanitized projection view. `session-output.md` can contain learner words, consented record context and private excerpts; do not open it blindly on a projector. Use course-authored synthetic evidence when public redistribution is required and label it synthetic, not real-book validation.

Activate all source and child-test files from repository root; this intentionally replaces the ephemeral exercise, with no backup/restore or final hello-world reset:

```bash
rm -rf src
mkdir src
cp -R lessons/12-specialized-retrieval/src/. src/
```

Root launch/check commands are in LESSON.md. The three service constants in main supply defaults; export overrides only when the prepared instance differs. The latter is the actual tutor tokenizer endpoint, not an alternative generation provider, and must match the provider JSON. The supplied adapter verifies the GGUF path reported by the local embedding server; there is no assumed `HARRIER_GGUF` variable. Use the verified environment contract above; the application additionally requires literal-loopback provider origin equality and rejects ambient proxies, with same-origin provider redirects. Complete source and support must be present before handing out the lesson.

## Supplied local modules

A **module** is a named part of this Rust application that groups related code. Local modules let us keep the agent's decisions visible while reading already-taught calculation and supplied UI/storage mechanics separately. They are files in this lesson, not a shared course library, another agent, or a new execution loop. Every Rust source starts with `// Lesson 12`.

| Source | Responsibility |
| --- | --- |
| [main.rs](../lessons/12-specialized-retrieval/src/main.rs) | Context loading/prompt assembly and consent; Session/runtime/Effects; concurrent Event consumption; machine assembly/run/cancellation; tool registration/dispatch/allowlists/limits; forced per-turn source grounding; application-side acknowledgment classification with one escalated re-run; per-request exact system prompt capture via a request-preparer hook; current-turn citation inspection; genuine-learner-source validation and post-run approval sequence. |
| [retrieval.rs](../lessons/12-specialized-retrieval/src/retrieval.rs) | Cohesive local read-only adapter: strict query, endpoint/profile/privilege/provenance checks, embedding/SQL, tokenizer accounting, evidence clipping and connection-driver cancellation/Drop guards. |
| [calculator.rs](../lessons/12-specialized-retrieval/src/calculator.rs) | Previously taught exact-decimal calculation, strict arguments and capability schema; two local tests. |
| [learning_record.rs](../lessons/12-specialized-retrieval/src/learning_record.rs) | Record categories/proposal shape/schema, bounded file reading/appending and default-no confirmation-input mechanics; two local tests. It does not decide learner-source validity or when approval may happen. |
| [chooser.rs](../lessons/12-specialized-retrieval/src/chooser.rs) | Supplied arrow/Enter chooser, numbered fallback, input and raw-mode cleanup. |
| [presentation.rs](../lessons/12-specialized-retrieval/src/presentation.rs) | Palette, independent terminal gating and streaming `TerminalProse` formatting; two local tests. Event consumption stays in main. |
| [provider_setup.rs](../lessons/12-specialized-retrieval/src/provider_setup.rs) | Positional provider-path handling and first configured model selection. Local tutor policy/transport checks stay in main. |
| [diagnostics.rs](../lessons/12-specialized-retrieval/src/diagnostics.rs) | Plain Markdown log writing, complete Session rendering and presentation sanitization; two local tests. It is never learner memory. |
| [tests.rs](../lessons/12-specialized-retrieval/src/tests.rs) | Application/runtime/Event/dispatch/memory/citation/grounding tests and shared test helpers; includes one opt-in live test that forced per-turn grounding prepares turn state without touching conversation history. |
| [retrieval/tests.rs](../lessons/12-specialized-retrieval/src/retrieval/tests.rs) | Adapter-local query/URL/database/vector/profile/provenance/serialization/cancellation tests, plus one opt-in actual-service smoke test. |

### Teaching reading path: preserve the visible boundaries

Use [the lesson's detailed reading path](../lessons/12-specialized-retrieval/LESSON.md#supplied-source-map-and-teaching-reading-path), not a line-by-line tour of support helpers:

1. Main's `LessonContext` loading/prompt assembly and selection-time consent establish context authority before inference.
2. Main's Session/runtime/Effects, concurrent `display_events` and `run_turn` explain saving versus presentation, machine execution, timeout/cancellation, sender drop/drain and inspection. Keep that order visible.
3. Main's one provider registration, `SafeTools`, `RunLimit` and the forced per-turn grounding path (`prepare_turn_grounding` → `select_grounding` → per-turn prompt part) establish all-block dispatch and one 2,500-token evidence allowance per learner turn. Read the cohesive retrieval adapter afterward, including cancellation/Drop guards.
4. Main's `inspect_turn`/`inspect_citations` use only the current turn's `source-grounding` excerpts, cited by the tutor; valid IDs do not prove semantic support.
5. Main's proposal-only handler and `parse_record_proposal` retain genuine-learner-source validation. Follow successful final/inspection → `approve_record_proposals` → default-no decision → decision log → append call → Session record refresh → post-approval diagnostics. `learning_record` supplies types/schema/file/confirmation mechanics, not permission to save.
6. Open supporting modules only for an explanation learners need; finish with application tests, retrieval child tests and inline calculator/record/presentation/diagnostics tests. Forty-six deterministic tests and two default-ignored live tests remain; no new learner scaffolding or shared framework is introduced.

### Corpus facts to say aloud

“Today we use a provisional test collection, not all the books and not an expert-reviewed knowledge base.” It contains 56 beginners-guide passages, 273 Heitkoetter passages and 182 CFI passages, from PDFs of 18, 273 and 116 physical pages. The 511 saved passages preserve original draft/provenance information. Full visual/reading-order acceptance, remaining margins and alternative chunk-policy comparisons remain open. EPUB support is future corpus expansion, not a requirement to implement before this provisional integration.

The instructor has private books but no expert-authored assessment rubrics or domain-expertise claim. The Heitkoetter source is from 2008; do not treat old book claims as current regulation. Do not invent missing authors, editions, publication dates, chapter names or chart interpretation.

## Suggested live-class sequence

### 1. Motivate and define (5 minutes)

Ask: “If the tutor gives a plausible answer, how could we inspect its evidence?” Define embedding, embedding space, retrieval, passage, collection and provenance in ordinary language before opening any SDK code. Contrast the tutor's generated explanation, fixed course context, returned book evidence and learner-approved notes. They are different sources with different authority.

Then define cosine distance and HNSW: “One is the comparison measure; the other is an approximate index that helps find candidates.” A closer vector is not a truth probability. No universal score threshold decides whether these books support a claim.

### 2. Predict grounding and citation use (5 minutes)

Contrast “What is a bid-ask spread?” with “What do the supplied books say about liquidity and the bid-ask spread?” Both run the same forced per-turn grounding — the application retrieves before every genuine learner turn — so the prediction is about **citation use**, not about whether a search occurs. The model has no search tool and cannot request one; a simple question may simply be answered without markers when the excerpts do not support it. Because acknowledgment is enforced, the honest options for a grounded turn are: cite the excerpts used, decline with `[source:none]`, or — if the tutor tries to stay silent — one application-authored re-ask. Predict which of the three occurs and why an application, not the model, must decide.

Select Getting started in day trading. Let learners choose default-no memory separately; do not enable it for them. Predict a short welcome/resumption, then a short answer with `[source:<id>]` markers beside supported claims and a citation footer for the IDs actually cited. Do not guarantee exact generated wording or which book ranks first.

### 3. Run and inspect actual behavior (10 minutes)

Run the supplied root with approved local services. Look for simple Tutor/Learner dialogue, short tutor prose, a citation footer whenever the tutor actually cites IDs, a plain source note when the tutor declines with `[source:none]`, and a grounding warning when retrieval is unavailable **or** a grounded reply is silently ignored and re-asked. No raw arguments, protocol fences, arrows, correlation IDs, vectors or Session dumps belong in ordinary dialogue.

Grounding runs on every genuine learner turn; a `no_evidence` turn answers without markers and an `unavailable` turn shows a learner-visible warning with the failure recorded in diagnostics — record which occurred rather than hiding either. The acknowledgment classification (`cited`, `declined`, `silently ignored`) is recorded in diagnostics for every grounded turn; a silent ignore triggers exactly one application-authored re-ask, never two. If irrelevant evidence appears, inspect rather than hide it. Do not rewrite the question secretly or inject content to make the demonstration look successful. If services are unavailable, ask for them; a mock is not live integration evidence.

Open only the approved diagnostic view. Match the query, returned IDs/excerpts/provenance and final answer. Explain **citation membership** before semantic support: an ID can exist and still fail to support its adjacent claim. Show physical PDF pages, counting from the file's first page, not printed labels. An exact marker `[source:<chunk_id>]` identifies returned evidence; it is not an ID the tutor may invent. Placeholder notation in the lesson is not a real citation. Also show the per-request `Application → provider: exact system prompt` dumps and point out where the source-grounding contract sits — first — so learners see that enforcement text actually reached the tutor, and in what order.

### 4. Explain the supplied boundaries (10 minutes)

Narrate: learner question → strict validation (the learner input is the query) → compatible local query embedding → parameterized reader query → threshold/cap selection → per-turn `source-grounding` prompt part (inserted first) → GDK re-evaluation → streamed tutor answer → application acknowledgment classification (`cited`/`declined`/`silently ignored`) → one authored re-ask only on a silent ignore → citation footer and approval decisions. This is current behavior, not a future system diagram.

Open the retrieval adapter and grounding selection only after explaining why a read-only, application-controlled boundary is needed: the model cannot craft queries, choose result counts, or re-search mid-turn. The SDK still dispatches the calculator/proposal tools, but supplies neither the application SQL policy, the grounding threshold/budget, nor the shared request cap. Unknown-name filtering in the pinned SDK makes the wrapper important. GDK still owns iteration; the application waits for the next learner turn afterward.

Distinguish Events from Effects: grounding runs before the run and logs diagnostics directly rather than emitting tool Events, while saved conversation changes belong to the recorded history through effect handling. Supplied async waits and concurrent stream consumption are infrastructure to explain, not code to reconstruct in class.

Show application-owned limits and why they are separate:

- Grounding query is the learner input: nonblank, at most 2,000 UTF-8 bytes; oversize input degrades the turn to `unavailable` rather than truncating silently.
- At most five SQL candidates; at most three selected statements above the 0.55 similarity threshold; collection/top-k/SQL/endpoints are not model-controlled.
- At most 1,000 tokens per excerpt and one 2,500 actual-token caller-owned budget per turn for grounding evidence, **including provenance JSON**. There is one forced search per turn, so no cross-call ledger is needed. The actual local tutor `/tokenize` counts excerpts and complete compact evidence JSON, including its accounting field; Harrier `/tokenize` separately preflights embedding input. Do not call character counts tokens or treat this budget as verified tutor context capacity. A clipped prefix can end mid-sentence; its clipping label must survive and no missing continuation may be invented.
- Embedding wait 15 seconds; SQL wait 5 seconds including server statement timeout; grounding precedes the 180-second run bound.
- Calculator and memory proposal tools share four requests, eight applied work passes plus possible halt batch and a 180-second whole run bound.

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

Diagnostics remain overwritten-at-launch plain Markdown, not learner memory. Preserve complete nested/default Session Debug values, post-approval state, loaded context, tool definitions/requests/results, retrieval profile/provenance, assembled answers, warnings, per-request exact system prompts (initial and escalated attempts) and available partial failure state. The system-prompt dump is instructor-facing evidence of what instruction text reached the tutor, including the grounding contract's first position; it is not shown in learner dialogue. Do not deliberately dump credentials, secret-bearing URLs, entire provider configurations or vectors. Logging/storage failures stop; unavailable storage cannot guarantee recording its own failure.

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

The active application and Lesson 11 reference establish the existing consent/approval/quote checks, content loading and UI design. Harrier HTTP and PostgreSQL boundaries are application adapters, not new GDK APIs. The supplied `src/retrieval.rs` returns `SearchResponse` with `collection`, `model`, `revision`, `profile`, `status`, `notice`, `evidence` and `used_tokens`. Status is `evidence_candidates`, `no_evidence`, or `budget_limited`; sanitized errors distinguish execution failure from usable evidence. Grounding in main is caller-owned: `prepare_turn_grounding` passes one 2,500-token per-turn budget, while the calibration harness passes a large value; retrieval is not a tool and has no correlation ID. Token counting uses the actual local tutor tokenizer and canonical response JSON. See Lesson 12 for source/hash, privilege, cancellation and endpoint checks.

## Validation and review record

The exact implementation evidence follows; earlier preparation or Lesson 11 success is not substituted for these runs.

Preparation evidence remains in RETRIEVAL.md: the active llama.cpp collection contains 511 vectors, profile identity and full draft provenance; original HF collection remains isolated; real load/rerun/reader smoke checks were recorded. Podman startup was tested, not separately Docker. Cross-runtime parity, expert retrieval quality, accepted layout/chunk-policy freeze and full EPUB corpus coverage remain unclaimed.

Integration rehearsal must separately inspect source-specific support, citation membership versus semantic correctness, query/evidence bounds across turns, collection/profile checks, all-block/shared-request limits/correlation, local services/privacy, plain and colored UI/citation survival, post-approval diagnostics and unchanged calculator/memory behavior. Maintenance failure probes belong to testing, not newly assigned classroom exercises. Keep outages, weak evidence and inconclusive teaching behavior in the record.

End-of-course review should examine beginner pacing, source qualifications, precise feedback, no mastery inference, readable citations, genuine memory agreement, privacy/source rights and terminal readability. Earlier Lesson 8–10 review, Lesson 10's inconclusive pacing experiment and Lesson 11's recorded teaching gaps remain independent. Lesson 9's default and 100-share live runs passed on 2026-10-22; its earlier live-tweak gap is resolved, not a reason to rewrite dated historical records. Later lessons have separate authorizations; this lesson neither introduces their behavior nor grants a new CLI or final corpus/model acceptance.


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


## Local service defaults — 2026-10-28

The three local classroom defaults are `const` values at the top of `src/main.rs`: `RETRIEVAL_DATABASE_URL` uses the instructor-supplied `retrieval_reader` connection to `127.0.0.1:55432/harrier`, `HARRIER_URL` is `http://127.0.0.1:9999`, and `TUTOR_URL` is `http://127.0.0.1:8080`. No exports or private connection file are required for these defaults. Existing environment variables (including values loaded by `dotenvy` from `.env`) override them. For another prepared database, set `RETRIEVAL_DATABASE_URL` to its reader connection, never the loader/admin connection. The supplied password is **plaintext in source and compiled binaries**; this is an explicit local classroom convenience, not secret storage or a production credential pattern. Rotate/replace it for any other deployment and do not project the credential-bearing source. Defaults do not start services or bypass runtime/profile/reader validation. `TUTOR_URL` selects the tokenizer origin and must match the tutor provider JSON; it does not override generation configuration.


### Service-default validation — 2026-10-28

Added the instructor-supplied reader connection and Harrier/tutor origins as constants at the top of main. Retrieval uses them only when the corresponding environment variable is absent; the main tokenizer-origin comparison uses the same tutor default. Invalid/non-Unicode retrieval environment values still fail instead of silently selecting a default. No process environment is rewritten, and no credential is deliberately added to diagnostics or telemetry. Provider JSON, compatibility checks, reader checks, dependencies and services are unchanged. The plaintext default is a classroom-only exception, not secure credential management.

Locked format/check/all-target Clippy/all-target tests/build passed in temporary staging for each complete reference: Lesson 12 **41**, Lesson 13 **67**, Lesson 14 **69** deterministic application tests, plus seven unchanged seeder tests each. One new test per lesson validates the reader host/role/database/port, local service origins and absent-variable fallback. The active Lesson 12 root independently passed the same checks and byte-matches its reference. Existing staged changes were preserved; no lesson activation replaced the user's root.

With all three override variables unset, both opt-in actual-service tests and the documented launch were attempted for each reference. All stopped with **`embedding runtime identity mismatch`** before the chooser: the existing Harrier service does not satisfy the pinned runtime identity check. This is blocked live validation, not a passed database connection, retrieval, tutor exchange, approval, compaction or telemetry run. No service restart, compatibility relaxation, corpus/catalog write or mock substitution occurred. No fresh IDE/projector/UI or teaching-quality validation is claimed; historical findings and review gates remain.


## Embedding server build flexibility — 2026-10-28

At the instructor's request, the learner application no longer requires an exact llama.cpp build string. A different or absent `/props` `build_info` is accepted. Model alias, F16 format, GGUF checksum, dimensions, context capacity, pooling, normalized vectors, query formatting, stored collection/profile identity, provenance and reader restrictions remain checked. The stored profile's build suffix records the original embedding preparation; it is not a requirement that the current query server report that build. Accepting another build does not establish numerical equivalence or retrieval-quality parity. The standalone preparation loader/client retains its separate pinned-build policy. No service restart, re-embedding or collection mutation is needed solely to pass the removed build check.

Validation for this change: formatting, locked check, all-target Clippy, all-target tests and build passed (41 deterministic application tests plus seven unchanged seeder tests). Both opt-in actual-service retrieval tests passed against the currently running services, including server build `b11669-620d2225c`; the documented launch reached the chooser and exited on EOF. Lesson 12 was checked in the active root; Lessons 13–14 were checked in temporary staging without replacing root. Regression coverage accepts changed/absent build metadata and still rejects incorrect model alias/format, sleeping/no-slot runtime and mismatched context. No full tutor conversation, approval/compaction/telemetry workflow, numerical cross-build parity benchmark, fresh UI/IDE/projector or teaching-quality review was performed. No services restarted, corpus/catalog modified, preparation client changed or dependencies updated. Existing historical defects/review gates remain.

## Learner-led scope follow-up — 2026-10-28

The content now explicitly accepts learner-chosen day-trading questions beyond the four supplied examples. Offer the menu only when requested or when the learner wants help choosing. Preserve lesson-specific capability, memory, citation and continuity instructions rather than copying a common calculator-only contract across lessons. For a fresh-selection rehearsal, ask “What should I consider when comparing brokers?” and then choose another topic outside the examples. Inspect whether the tutor answers educationally without redirecting to the preset units, inventing current facts or recommending a personalized financial decision. In retrieval lessons, separately ask what the supplied books support and inspect actual returned evidence/citations.

Validation of this edit: static content checks across Lessons 10–14 and the active Lesson 12 context-loading test passed. No new live tutor, memory approval, retrieval, compaction or telemetry behavior is established by these checks; fresh teaching validation remains pending.

## Forced per-turn source grounding and reference restoration — 2026-11-11

The instructor had the root restored to this lesson's complete reference and had all lesson documentation brought in line with the actual restored design. A Lesson 13 shared-library split had briefly existed in the working copy; its orphaned files were removed and the root again byte-matches `lessons/12-specialized-retrieval/src/`. `examples/rag_calibration.rs` was reverted to the pre-split single-crate API and remains an uncommitted working-tree change; no commit was made.

Design correction recorded here and applied to the lesson body above: the reference implements **forced per-turn source grounding**, not a model-callable retrieval tool. There is no `search_educational_passages` tool, no `RetrievalToolProvider`, and no `EvidenceBudget` in the source. The application retrieves once per genuine learner turn before the machine runs (threshold 0.55 with a validated `GROUNDING_MIN_SIMILARITY` override, at most three statements, 2,500 actual caller-owned tokens); the learner input is the query; SQL returns at most five candidates; empty selection reports `no_evidence`; retrieval failure reports `unavailable` with full diagnostics and a learner-visible warning. Excerpts travel as a per-turn `source-grounding` prompt part, never saved into conversation history; the tool surface remains calculator plus the conditional proposal definition behind SafeTools/RunLimit. The class-level prediction therefore concerns citation use, not tool selection. Dated validation records above were left verbatim.

Validation for the restoration and this documentation pass: `cargo fmt --check`, `cargo check`, `cargo clippy --all-targets`, `cargo test` and `cargo build` passed. Deterministic tests now total **46** (up from the historical 40/41 counts), with the two `#[ignore]` live tests (`live_forced_grounding_prepares_turn_state_without_touching_history` and the retrieval smoke test) both passing explicitly in 18.93s against tutor :8080, Harrier :9999 and PostgreSQL :55432. The documented launch reached the chooser and exited 0, printing the grounding banner. No corpus mutation, dependency change or service restart occurred. No fresh teaching-quality, UI or IDE/projector review is claimed; historical semantic defects and deferred end-of-course review remain.

## Grounding acknowledgment enforcement — 2026-11-11

Instructor follow-up after a live turn delivered a qualifying excerpt (similarity 0.684 ≥ 0.55) and the tutor answered with no `[source:]` markers — a silent ignore. Root cause was tutor behavior; retrieval, selection, state and injection were verified correct. The instructor's framing governs the fix: models are overconfident, so they must **not** be trusted to judge what retrieved information is relevant. The application now enforces acknowledgment.

Teaching points for this change:

- **The enforcement contract leads the prompt.** The `source-grounding` prompt part is inserted first, before tutor contract/educational material/teaching guidance. Ask learners why position matters: a buried rule is more likely to be dropped by the model; the diagnostics system-prompt dump makes the position inspectable.
- **The model chooses cite-or-decline, the application judges compliance.** The tutor must either cite each excerpt used with its exact `[source:<id>]` marker or decline in one sentence ending with `[source:none]`. The application classifies with `GroundingOutcome` (`NotSupplied`/`Cited`/`Declined`/`SilentIgnore`) by scanning the reply's citations against this turn's selected excerpts — the model's self-report plays no role. Citing and declining together is a flagged contradiction, not an error swallowed.
- **Exactly one re-ask.** A silent ignore appends one authored reminder as a user message and re-runs the turn once under fresh per-attempt bounds; a second silent ignore after the re-ask is reported, not retried. This mirrors the course principle that safeguards must be bounded, not open-ended.
- **Escalation/provenance interaction.** Proposal provenance windows on the latest user message, so first-attempt proposals are approved before the escalation append, and the reminder is excluded from learner-quote provenance. Use this to revisit why an application-authored message is never learner evidence.
- **Diagnostics as instructor evidence.** Every genuine inference request's exact system prompt is captured via `PromptDiagnosticsPreparer` (wrapping the SDK's `IdentityInferenceRequestPreparer` through `InferenceRunner::with_request_preparer`) and dumped per attempt and request number. This is the mechanism that made the original silent-ignore diagnosable; present it before the escalation path.

Honest limits to state in class: the escalation, decline and contradiction branches have deterministic test coverage but no live-provider evidence yet — forcing a live silent-ignore is non-deterministic. A cited answer still only shows citation membership, not semantic support. Classification relies on exact marker syntax; a paraphrase without markers is, by design, non-compliant rather than semantically evaluated.

Validation for this pass (matching the lesson record): fmt/check/clippy (zero warnings)/test (49 deterministic, up from 46; three new: outcome classification, escalation message excluded from provenance, exact system-prompt capture)/build passed; both `#[ignore]` live tests passed in 18.46s against tutor :8080, Harrier :9999 and PostgreSQL :55432; the documented launch reached the chooser and exited 0; a live run of the previously silent-ignored journal/psychology question showed both excerpts cited with exact markers (outcome `cited`), diagnostics dumps leading with the grounding contract on the grounded turn, and complete `turn_grounding` state. `src/` was mirrored byte-identical into the lesson's `src/`. No corpus mutation, dependency change or service restart occurred. Historical semantic defects, teaching/UI review and the deferred end-of-course review remain.

Completion record (2026-11-11): the instructor verified the root `src/` tree and `lessons/12-specialized-retrieval/src/` as byte-identical and marked Lesson 12 complete, with the grounding acknowledgment-enforcement pass above as the lesson's final validated state. The honest limits above and the deferred end-of-course review remain part of the record; this completion does not approve Lessons 13–14.

## Pre-display citation gate backport — 2026-11-12

Lesson 13's citation gate is now part of this lesson's reference (new `src/citations.rs`; `main.rs` registry wiring, buffered display, inline `InvalidAcknowledgment` re-ask). Full behavior is in the lesson record, including this lesson's ordering subtlety: attempt-1 record proposals are approved **before** the escalation user message is appended, because that message moves the provenance window. Classroom framing: the application checks citation IDs the way it checks tool arguments — untrusted model output validated against what the application actually supplied. Re-validation note: with this lesson active, `cargo clippy --all-targets` fails on the `rag_calibration` example (it needs the Lesson 13 library); use `cargo clippy --bin gdk-hello` / `cargo clippy --tests` instead. Validation matches the lesson record: 51 + 2 ignored tests, fmt clean in place, clippy zero warnings in both sanctioned scopes; the live fabrication path remains deterministic-coverage only.
