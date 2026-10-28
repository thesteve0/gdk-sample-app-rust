# AGENTS.md

Guidance for coding agents and for Goose when assisting learners in **Building Agentic Applications with the Goose GDK**.

## Project purpose

This repository is an evolving, instructor-led course for building a custom Rust agentic command-line application with the Goose Development Kit (GDK). It uses the exact pinned public `goose-providers` crate directly through its native Rust API; it is not a Goose application extension, a Kotlin/Python binding, or a wrapper around the Goose CLI.

Optimize first for a live class taught by an instructor. Goose-assisted self-study is useful, but secondary.

## Read before changing anything

1. Read `README.md`.
2. Inspect `git status` and preserve unrelated staged and unstaged work.
3. Read `Cargo.toml`, `Cargo.lock`, and `rust-toolchain.toml`.
4. Read the active root implementation in `src/main.rs`.
5. Read the relevant `lessons/<number>-<topic>/LESSON.md` and its complete reference solution (`src/main.rs` and any lesson-local modules).
6. Read the corresponding `instructor-notes/` file when one exists.
7. Read the provider JSON used by the active lesson.
8. For any GDK API, provider-schema field, type, command, or behavior, consult the official documentation and the exact pinned SDK source. Do not rely on memory.

## Repository model

- Root `src/` is always an ephemeral exercise working copy and may be incomplete. Activate a lesson by replacing it with all reference `.rs` files, not main alone; no backup or restoration is required. Preserve authoritative lesson references and unrelated work elsewhere. The final hello-world root reset occurs only after all review, not now.
- `lessons/` contains numbered instructional units.
- A lesson's `LESSON.md` contains explanations and important snippets.
- A coding lesson's `src/` supplies its complete reference solution from the outset (`main.rs` plus any lesson-local modules). It is not an independently packaged application. Replacement Lesson 7 is intentionally no-code and has no `src/main.rs`.
- Do **not** copy root source into a lesson directory at lesson completion.
- `instructor-notes/` contains one instructor guide per implemented lesson.
- Keep numeric prefixes on lesson directories.

## Pedagogy and scope

- Make small, concept-focused changes.
- Introduce only concepts required by the active lesson.
- Explain why a GDK abstraction is needed, not merely how to type it.
- Build the conceptual foundation progressively: establish the application-level mental model and define each domain concept before the lesson uses its code-level form.
- Do not scaffold the final application or future lessons prematurely.
- Do not introduce ownership workarounds, traits, generic abstractions, workspaces, or async machinery before the active lesson needs them.
- Ask the instructor before making uncertain curriculum, sequencing, or architecture decisions.
- Keep the roadmap in `README.md` current when lesson state or order changes.

**`README.md` is the authoritative curriculum plan**, including replacement Lessons 7–9 and their implementation handoff. `post-lesson4-plan.md` supplies supporting architecture context and historical evidence; it must not override the README. The final instructor decision (2026-10-11) fixes the course at 14 lessons: Lesson 12 PostgreSQL/pgvector retrieval, Lesson 13 context compaction, Lesson 14 evaluation. Earlier market-data/ingestion/CLI/fine-tuning sequencing is superseded. Read `lessons/final-lessons-handoff.md` and the numbered PLAN.md files before implementation.

### Progressive conceptual foundation and editorial rules

Apply these rules to every lesson and to curriculum changes:

1. **Define a new domain term in prose before it appears in a code snippet.**
2. Give the term a plain-language purpose: explain not only *what it is*, but *why the application needs it now*.
3. Reuse the exact term consistently after defining it.
4. Do not introduce a library type, method, field, or protocol value before its conceptual counterpart is established. For example:
   - define *conversation history* before `Conversation`;
   - define *structured output* before `as_tool_request()`;
   - define *correlation ID* before `request.id`;
   - define the *tool execution boundary* before dispatch code.
5. Future-state diagrams are encouraged when they help learners form a mental model, but clearly label them as future behavior and distinguish them from what the current lesson implements.

Use concise source comments to point out what code is doing and why it is needed at that point. Keep the longer explanation, terminology, diagrams, and instructor discussion in the lesson prose and `instructor-notes/`, rather than turning source comments into the lesson.

### Lesson-code Rust style: unwrap in the open

Attendees are not assumed to be proficient in Rust, and neither is the instructor. Reference solutions, lesson snippets, and tests favor explicit, verbose Rust over idiomatic compactness whenever the compact form hides what the type system is doing:

1. **Destructure in the open.** Unwrap a `Result` or `Option` with an explicit `match` whose only job is "extract the value or bail out": give the extracted value a named `let`, and bail with an early `return Err(...)`.
2. **No match guards** (`Ok(call) if cond`). Extract first into a named binding, then branch with plain `if`/`else`.
3. **No `|_|` closures and no `.map_err(...)?` chains.** Converting an error is its own named step: `let failure = format!(...)` followed by `return Err(failure);`.
4. **Keep `?` for the one obvious thing.** Propagating an already-formed error is the sanctioned single-concept use: `let entry = parse_price("entry_price", &args.entry_price)?;`.
5. **Name every intermediate value** (`parse_result`, `parsed_call`, `requested_name`) so each line teaches one thing.
6. **Positional format arguments.** Prefer `format!("{} ...", field, raw)` over inline captures (`{field}`) so formatting reads like the `String.format` style attendees already know from other languages.
7. One-line comments label each boundary or step; longer explanations stay in lesson prose and `instructor-notes/`, per the editorial rules above.

Behavior never changes under these rules; only structure and naming do.

### Lesson terminal output: payload delimiter and actor labels

**Lesson 8 exception (instructor decision, 2026-10-05):** do not use `++++++++` fences or repeated commentary in Lesson 8. Show Session before/after with every field name and its complete value, including nested message metadata and usage metadata, two blank lines after each raw Rust Session dump, an `application → provider:` label before the exchange, and one short actor label for the live stream. Do not omit absent/default fields. The earlier Lessons 5–6 retain their existing output convention; do not rewrite them as part of this change.

Lesson terminal output mixes the application's explanatory text with **protocol payload** — the actual values that traveled to or from the provider. Two output conventions make that boundary visible; Lessons 5 and 6 already follow them, and new lessons that print provider interaction must reuse them.

1. **Payload delimiter.** Every block of payload is printed between two identical `++++++++` lines through the `PAYLOAD_DELIMITER` constant and `print_delimiter()` helper; everything outside those lines is the application's own commentary. Streamed model output is payload even though individual deltas cannot each carry a fence: open the delimiter at the first text delta and close it after the stream ends. Define "payload delimiter" in lesson prose before the code that shows it, per the editorial rules above.
2. **Actor labels.** The line immediately before each payload block names the sender and the receiver — `application → provider` on outbound payloads, `provider → application` on streamed and reconstructed output, and `application, as the tool → provider` on dispatched tool responses (the tool execution boundary lives inside the application). With multiple actors in the round trip, every message must show who sent it and who receives it.
3. **Keep them uniform.** Use identical fences, labels, and phrasing across lesson code, `LESSON.md` snippets, and sample-output blocks, so learners see one convention. Fence and label any new payload site when a lesson introduces one.

### Terminal styling rule for all application code

**Instructor decision (2026-10-06):** reuse Lesson 9's terminal color scheme whenever creating or modifying code that produces terminal output. This applies to every active coding exercise and future application code, not only Lesson 9. Earlier exercises (Lessons 1–6 and 8) now follow it; Lesson 7 has no terminal code. This presentation-only update supersedes earlier instructions to leave Lesson 8's styling unchanged; it does not authorize curriculum or behavior changes. Archived drafts remain historical.

| Output role | Style |
| --- | --- |
| Headings and ordinary actor labels | Bold cyan (`1;36`) |
| Current human input, when separately displayed | Blue (`34`) |
| Reconstructed tool requests, including their actor labels | Magenta (`35`) |
| Tool responses, including their actor labels, and success checks/status | Green (`32`) |
| Warnings and nonfatal diagnostics | Yellow (`33`) |
| Errors on stderr | Bold red (`1;31`) |
| Live model-generated text | Default foreground, unbolded |
| Every Session attribute name, including nested metadata/usage fields and quoted object keys | Gold `#FFD700` (`38;2;255;215;0`) |
| Session attribute values | Default foreground, unbolded |
| Only the top-level Session `usage` section | Dim (`2`), with its attribute names still gold |

- Apply styles only when printing; never insert ANSI codes into messages, tool arguments/results, Events, Effects, recorded state, or provider input. Do not color the system instruction or entire history blue merely because it is outbound.
- Use the Lesson 9 `TextStyle`, `styled`, and terminal-detection pattern, supplying only the helpers/styles each standalone lesson needs. Preserve self-contained reference sources and avoid a new styling dependency or premature shared abstraction.
- Disable styling for a redirected/non-terminal stream, presence of `NO_COLOR` (even an empty value), or `TERM=dumb`. Check stdout and stderr independently with `IsTerminal`; plain transcripts must contain no application-added ANSI escapes.
- Reset foreground after gold field names with `39`, not a full reset, so usage dimming survives. Reset the complete style after each colored block so the next label/value/model fragment does not inherit it. Do not bold Session attribute values.
- **Do not dim any other output**, including raw-provider usage diagnostics, conversation/message metadata (even nested usage), instructions, or capability definitions.
- Preserve complete Session Debug values, absent/default fields, existing actor labels, payload delimiters, spacing, output order, and Lesson 8's no-fence exception. Do not add payload sites solely to show a color. Text labels must still distinguish roles without color.
- Keep lesson snippets, plain sample transcripts, and instructor notes consistent with the code. Supply and explain presentation helpers; do not turn them into Rust scaffolding exercises. Test style mappings, terminal gating, plain fallback, and complete Session rendering where applicable; validate live-provider output separately.

## Accepted post-Lesson 4 sequence

Isolated technical validation supported the direction recorded in `post-lesson4-plan.md`. Preserve these decisions unless the instructor explicitly revises them:

- Lesson 5 defines one deterministic maximum-planned-loss tool, advertises it to raw provider inference, and inspects structured request content. It stops before dispatch or a tool response.
- Lesson 6 validates and executes that request, returns a correlated user-role tool response, and asks the provider for a final educational explanation.
- On 2026-10-05 the instructor withdrew old Lessons 7–8; unchanged material is in `drafts/lessons/` and `drafts/instructor-notes/`. The README now records the replacement plan for instructor review. Lessons 7 and 8 implementation and the Lesson 8 root update were subsequently authorized; Lesson 9 was authorized on 2026-10-06 and is implemented; live-provider validation and instructor review remain pending. The old state-machine sequence is not binding.
- Replacement Lesson 7 establishes Session, Operation, StateMachine, Effect, and re-evaluation without code or pseudocode. An Operation may consult inputs outside the Session (Steer is an example). Do not equate recorded history with all live decision inputs or teach Goose's full assembly as required GDK infrastructure.
- The instructor narrowed Lesson 7 on 2026-10-05 to one simple question/answer round trip, with concise prose. Its progressive lecture diagrams/images cover ownership and external inputs, ordered pass/re-evaluation, and the simple exchange with Session snapshots. Defer the correlated planned-loss trace and its visuals to Lesson 9. Supply editable sources and projector-readable rendered images with captions/text equivalents; instructor notes explain reveal order and prediction prompts. Follow the README visual requirements, not a dense diagram of Goose's full assembly.
- Replacement Lesson 8 supplies complete code for one streamed exchange via `StateMachine::run`, displays the Session before/after, and distinguishes Events/Emitter from Effects/effect handler. Consume Events while the run executes, not after buffering a full run. Runtime traits and concurrency code are supplied and explained, not fill-ins.
- Replacement Lesson 9 combines successive user turns and tool use in one Session: the planned-loss tool is available on both turns; first ask “What is stop-loss referring to in day trading?” (expect no tool request), then request maximum planned loss for entry `51.20`, stop `50.70`, and `200` shares (expect a correlated tool round trip and `100.00`). Inspect and report incorrect tool selection; never hide it or force the expected outcome. Use “maximum planned loss,” not “maximum expected loss.”
- Give learners the entire coding-lesson implementation from the outset. Classroom rhythm is predict → run → explain → tweak → run again; do not assign missing Rust scaffolding. Approved tweaks are changing Lesson 8's question and changing Lesson 9's share count to `100` (planned loss `50.00`).
- Keep the GDK-owned run loop in the replacement lessons. Instructor decision (2026-10-05): Lesson 8 has no loop safeguard; add a concise comment beside `StateMachine::run` noting that production users may want a safeguard to prevent a runaway loop. Do not introduce a load budget or restore the manual loop. Separately settle Lesson 9 execution bounds and verify unknown-name handling and all-request/request-limit behavior in the tool adapter.
- Price arguments at the model/tool boundary are decimal strings. When execution begins in Lesson 6, parse and validate them immediately using exact decimal arithmetic. Do not teach binary floating point for currency. Validation used `rust_decimal`: at most four decimal places, dollar results formatted to two places.
- Keep tool names allowlisted, treat names and arguments as untrusted model output, reject unknown fields, preserve request IDs, handle all content blocks/tool requests, and impose explicit round/request or state-step bounds.
- Keep exploratory spike code out of lesson directories; derive the smallest concept-focused lesson code.
- Explicit model selection remains deferred to the later CLI lesson.

Before starting a tool lesson, read `post-lesson4-plan.md` and the exact pinned GDK/RMCP source. Lessons 1–7 are complete. The instructor clarified on 2026-10-07 that Lessons 8 and 9 still need review, superseding earlier completion records for Lesson 8. The root previously contained the authorized replacement Lesson 8. The instructor authorized Lesson 9 on 2026-10-06; root now contains the separately authorized Lesson 11 learning partner. Keep Lesson 7 and Lesson 8’s workflow unchanged; Lesson 8’s terminal styling now follows the all-application rule above. Lessons 8 and 9 still need instructor review; Lesson 9’s default live scenario passed, but its live 100-share tweak is pending. Lesson 10 was authorized on 2026-10-07; its implementation does not release Lessons 8–9 or authorize Lesson 11.

## Lesson 10 implementation — 2026-10-07

The instructor authorized Lesson 10 and replacing root with its interactive day-trading learning partner. Follow README for scope and the complete `lessons/10-day-trading-learning-partner/` reference and notes. Only Getting started in day trading is available. Separate editable Markdown supplies common contract, educational material/priorities, and teaching guidance, loaded at selection and fixed for that Session. Arrow/Enter selection has a numbered fallback for redirection, NO_COLOR, and TERM=dumb; /finish returns to selection, /quit/EOF exit, blank replies are ignored. Selection creates a fresh Session; replies retain it. GDK owns the within-run loop. Preserve Lesson 9 calculator validation/correlation/all-block safeguards and per-turn limits. Calculator use is optional, not forced.

Lesson 10's presentation exception: learner terminal shows brief activity, framed tool results, and framed tutor streams; complete Session snapshots and internal diagnostics move to root `session-output.md`. Truncate at launch, append every turn/selection, preserve all fields/defaults/nested metadata and available partial state with Rust Debug fences; record loaded context, correlated tools, assembled saved tutor answers, warnings/failures. Markdown contains no ANSI, no promise of gold syntax colors, and is never read as learner memory. Ignore /session-output.md; ignored is not secured and conversations can contain personal data. Log errors stop the application. Never deliberately dump provider configuration or credentials. Keep course terminal palette/gating for normal output; no Session formatting/color helpers are needed when not printing Sessions.

Supplied crossterm 0.29.0 events-only supports selection; GDK/provider/RMCP/toolchain versions and provider JSON remain unchanged. Approved classroom tweak edits the guidance pacing sentence to use a short hypothetical example before a check-for-understanding question, then starts a fresh selection. Instructor content/teaching review remains pending. Lessons 8–9 review and Lesson 9's live 100-share tweak remain separate pending gates.

Lesson 10 validation: 20 tests and format/check/Clippy passed; live conversation/reselection, optional correlated $100.00, arrow/color and plain/NO_COLOR/dumb modes, commands/EOF, complete Markdown diagnostics, and fatal logging/provider partial-state handling checked. Guidance tweak loaded but example-before-check ordering was inconclusive; default restored. Keep teaching/content/UI review pending. Exact evidence is in Lesson 10 and instructor notes.

Lesson 11 now prioritizes a persistent learning record and learner-agreed to-do list, superseding evaluation-baseline-primary scope. Distinguish discussion, goals, demonstrated understanding, unresolved questions, and agreed next activities; do not infer mastery from tutor explanations. The instructor authorized Lesson 11 on 2026-10-11: editable ignored Markdown, consent to use memory per selection, proposal-only tool, default-no per-addition local approval after successful run, append preserving hand edits, corrections/deletion by hand between launches. Root/reference are supplied; no later lesson is authorized.

## Lesson requirements

Every implemented coding `LESSON.md` should include a goal, concepts introduced, prerequisites, incremental walkthrough instructions, focused snippets, a run or validation command, expected structural behavior, success criteria, and the conceptual next step. Lesson 7 instead uses narrated traces/diagrams and instructor-led conceptual validation; no Rust snippets, source file, or Cargo run requirement. Consult the README for planned paths and handoff gates before implementation.

Keep lesson prose, reference source, root workflow, and instructor notes consistent. If code behavior changes, update every affected explanation in the same change. Do not add deliberate failure-path exercises without instructor approval.

## Provider and model conventions

- Provider JSON is the source of truth for endpoint, authentication, models, and capabilities.
- The bundled JSON is a replaceable default, not a required service or model.
- Learners copy and modify an existing valid configuration; they do not author its complete schema from scratch.
- Before the CLI lesson, use a default provider path and an optional positional override. Later replace it with a named `--provider` option after deliberately selecting a Rust parser.
- Initially select the first configured model. Introduce `--model` later and validate it against JSON-declared models.
- Load `.env` with `dotenvy` before constructing the provider. Never commit `.env`, credentials, or secret-bearing provider files.

## Development environment

- Target Linux for now.
- Use the toolchain pinned in `rust-toolchain.toml`.
- Use Cargo for dependency changes, locking, building, running, and tests.
- Commit `Cargo.lock` for this application/course repository; update it with `Cargo.toml`.
- Pin an exact alpha GDK release. Do not use moving branches or internal Goose crates without an explicit architecture decision.
- Do not call `cargo` with `--all-features` by default.
- Keep the dotenv approach consistent: use `dotenvy`.

## Validation

- Prefer live-provider validation whenever a lesson makes provider calls.
- If a provider is unavailable, ask the instructor to start or supply one; do not silently replace live validation with mocks.
- Assert structural outcomes rather than exact model text.
- After changing Rust source, manifests, lockfiles, or configuration, run the narrowest relevant checks and the lesson's documented command:

  1. `cargo fmt --check`
  2. `cargo check`
  3. `cargo clippy --all-targets`
  4. `cargo test`
  5. `cargo run -- [lesson arguments]`

- Report exactly what was and was not validated.

## Documentation and hygiene

- `README.md` is the course landing page; lesson detail belongs in `LESSON.md`; instructor-only guidance belongs in `instructor-notes/`.
- Call this material a course or workshop.
- External links are supplemental; core lesson instructions must stand on their own after setup.
- Never overwrite unrelated user changes or edit generated build output.
- Keep `/target/`, `.env`, `.idea/`, and `out/` untracked.
- Do not add contribution guidance unless asked.

## Goose-assisted self-study mode

Use this mode only when a learner asks Goose to guide them through the course.

1. Ask which lesson the learner is taking and confirm provider setup.
2. Read the lesson, its solution, and the active root state without revealing the complete solution prematurely.
3. Explain or assign one small step at a time.
4. Ask the learner to write or run it, then wait for their result or approval.
5. Diagnose observed output using the lesson's success criteria.
6. Prefer hints and focused snippets over replacing the learner's entire file.
7. Do not skip ahead into later concepts unless needed to unblock the current lesson.

## Instructor authority

The instructor owns curriculum scope, sequencing, and release decisions. When this file conflicts with an explicit instructor request, follow the request and update durable documentation when relevant.


## Lesson 9 implementation handoff (2026-10-06)

Lesson 9 source, prose, notes, and narrated SVG/PNG frames are supplied. At that handoff root matched its reference source; root now runs Lesson 11. `SHARE_COUNT` is the approved 200→100 tweak, shared by the prompt and structural check. The application uses a `ToolProvider` with typed `LossArguments` and a safe wrapper around the pinned `ToolOperation`; the SDK alone filters unknown names and supplies no request cap. Keep correlated unknown/excess errors and all-block handling. Bounds are eight applied work passes plus a possible terminal halt batch, four aggregate requests per user turn, and a 180-second run timeout; the engine owns the loop. Full partial state is shown on failure.

Eleven deterministic tests and build/lint checks passed; the local live endpoint was unavailable. Neither real selective-tool-use outcome nor the live 100-share tweak is validated. Start/supply a provider and record actual structure/explanation before requesting release review. Do not mark Lesson 9 Complete or implement future curriculum based on these tests.


### Lesson 9 output-order follow-up — 2026-10-06

The reconstructed request now displays through the shared Event queue after saving and before the correlated tool result/final answer, rather than being reprinted by post-run validation. A presentation-only marker is never persisted or included in provider input. All 12 tests and build/lint checks pass. The provider became available and the default live two-turn scenario passed: no first-turn call; one correlated second-turn calculator request/result, `$100.00`, then a grounded final explanation with exclusions. Initial endpoint-unavailable records above are historical. The live 100-share tweak and instructor review remain pending; do not mark Lesson 9 Complete. Lesson 10 was separately authorized on 2026-10-07.

### Lesson 9 terminal color follow-up — 2026-10-06

Instructor approved bold cyan headings, blue current human input, magenta reconstructed requests, green tool responses/success checks, yellow warnings, and bold red errors. Model stream stays normal color. **Only Session `usage` is dimmed**; no other section may be dimmed. Preserve complete snapshots, actor labels, fences, and output order. ANSI styling is presentation-only and disabled for redirected streams, `NO_COLOR`, and `TERM=dumb`. Root/reference and lesson/notes match; 14 tests, checks, IDE build, and default live colored/plain/NO_COLOR runs passed. Live 100-share tweak and instructor release review remain pending; Lessons 7–8 unchanged.

### Lesson 9 Session attribute styling — 2026-10-06

Bold every Session attribute name (including nested metadata, usage fields, and quoted object keys); leave values unbolded. Only usage remains dimmed. Preserve complete Debug values, plain-output fallback, and existing color/order conventions. Root/reference match; 15 tests, checks, IDE build, and default colored live run passed. Live 100-share tweak and release review remain pending.

### Gold Session attributes — 2026-10-06

Instructor found bold difficult to see and replaced it with gold (`#FFD700`) for every Session attribute name, including nested fields and quoted object keys. Values use the default foreground and are unbolded; only usage remains dimmed. Foreground-only reset preserves usage dimming. Root/reference match; 15 tests, formatting/check/Clippy, IDE build, and default colored live run passed. Plain fallback and complete snapshots remain intact; Lessons 7–8 unchanged. Live 100-share tweak and release review remain pending.


### Earlier-exercise terminal styling — 2026-10-06

Instructor requested Lesson 9's scheme for all earlier active coding exercises and made it the rule for future terminal-output code. Lessons 1–6 and 8 now use self-contained presentation helpers; prose, snippets, instructor notes, and README agree. Lesson 7 has no terminal code; root, Lesson 9, archived drafts, dependencies, and provider JSON are unchanged. Behavior, payload delimiters/labels/order, full Session values, and Lesson 8's no-fence exception remain intact.

Each earlier coding reference passed formatting, check, Clippy, tests (33 in aggregate), and its documented run using the shared pinned manifest in temporary staging. Provider-calling references passed against the live local model in colored, redirected/plain, and empty-NO_COLOR modes. TERM=dumb and independent stdout/stderr gating were also checked. Lesson 6 returned a correlated `$100.00` and grounded final explanation; Lesson 8 retained complete gold-field Session snapshots with only top-level usage dimmed. The unchanged root separately passed its 15 tests, build/lint checks, and default live two-turn scenario. Lesson 9's live 100-share tweak and instructor release review remain pending; this styling update does not change curriculum status.


## Lesson 11 implementation — 2026-10-11

Follow README and `lessons/11-learner-memory/` as current root. Preserve Lesson 10 and its pending UI notes. `learner-record.md` is fixed-path Markdown, read only with default-no consent per selection; approved note appends preserve existing bytes and use repeated category headings. No in-app correction/review commands; edit/delete between launches. Proposal tool has strict category/note/learner_quote fields, single-line 600-byte limits, exact actual-learner-quote check, and no path argument. Source occurrence is not semantic validation. The tool acknowledges WITHOUT writing; only local default-no approval after successful run/inspection saves. Notes are separate from diagnostics and old transcripts; new selection is fresh Session with current record context. Existing memory updates in current Session after approval; disk hand edits are for between launches. No mastery/completion inference.

Lesson 11's authorized UI exception overrides payload/actor logging conventions for its learner dialogue: simple Tutor/Learner labels, brief activity/results, no application-added fences/arrows/IDs/raw arguments/Session dumps. A supplied streaming `TerminalProse` helper suppresses common Markdown markers; no general renderer. Keep course colors/gating and untouched saved/provider text. Full developer data remains in plain overwritten `session-output.md`, never read as memory. Include complete post-approval Session diagnostics and proposal definition when available.

Record is capped at 32 KiB, regular UTF-8 only, rejects symlinks/nonregular paths; new Linux file requests 0600, existing permissions unchanged. Ignore is not encryption/access security; consented notes are sent to provider and can appear in diagnostics. Single-process append+sync has no transactional/crash/exactly-once guarantee; log/storage failures stop. Preserve calculator validation, allowlists/all blocks/correlation and shared four-request/eight-pass/180-second per-turn bounds. No dependencies/provider/toolchain upgrades.

26 tests and formatting/check/Clippy passed. Live approved append, manual-edit/relaunch resumption, declined memory, rejected addition, and correlated $100.00 calculator passed. An initial proposal-followup timed out at 180 seconds; no approval/write occurred and full partial diagnostics survived, then a separate rejection retry passed. Plain/NO_COLOR live output checked; colored arrow UI not revalidated. Test record removed. Lesson11 content/teaching/UI and instructor review pending; Lessons8–10 review and Lesson9 live100-share tweak remain independent gates. No Lesson12 scaffolding.


### Instructor review schedule — 2026-10-11

The instructor requested that content and teaching materials be implemented now and will review once the full course is implemented. Pending content/teaching/UI review is therefore deferred to end-of-course review, not a blocker for separately authorized implementation. This is not release approval and does not erase validation gaps or authorize later lessons implicitly. Lesson 11 now includes memory-aware runtime teaching guidance, authored category examples, a narrated approval lifecycle, exit questions, and a supplied instructor teaching script. These editorial additions still need instructor review and fresh live teaching evaluation.


## Final curriculum planning handoff — 2026-10-11

The instructor authorized documentation only for the final Lessons 12–14; **do not implement them until separately requested**. Root remains Lesson 11. Plans live in `lessons/12-specialized-retrieval/PLAN.md`, `lessons/13-context-compaction/PLAN.md`, and `lessons/14-agent-evaluation/PLAN.md`; `lessons/final-lessons-handoff.md` is the fresh-session entry point after README. No source belongs in these planned units yet.

Lesson 12 integrates a supplied PostgreSQL/pgvector datastore, not ingestion. Schema and embedding model/service are unresolved prerequisites. The instructor has private EPUB/PDF books, no expert-authored rubrics and no domain-expertise claim. Do not assume a prepared assessment corpus, silently invent schema or embedding compatibility, or represent source-grounded feedback as mastery assessment. Settle source rights, disclosure, provenance and a redistributable test corpus before coding.

Lesson 13 is long-conversation context compaction, not simulation. Verify pinned Operation/effect/inference APIs; full Goose compaction is not automatic GDK functionality. Preserve actual learner quotation provenance, kickoff/current-turn handling, request/result pairs and pending local approvals. Current `.skip(1)` quotation search is a compatibility trap when replacing history. Model summaries never authorize writes or substitute for authoritative state.

Lesson 14 evaluates the full application, separating deterministic invariants from live evidence use and fallible teaching judgments. Preserve prior validation gaps and deferred end-of-course review. Do not implement former Lessons 15–18 or a general CLI as implicit requirements.

Root `potential-lesson-15/` is an explicitly approved **documentation-only draft**, not a committed lesson or implementation permission. Preserve its plain-language terminology, reproducible probabilistic engine boundary, unresolved data/calibration choices and no-source status. Do not combine it with compaction.

## PDF preparation subset - 2026-10-11

The instructor separately authorized model/container downloads and supported GGUF conversion, then Rust **PDF ingestion and draft chunking only**. `embedding-code/` is a standalone preparation package with its own Cargo manifest/lockfile and README; root Lesson 11 and root dependencies remain unchanged. Installed Poppler extracts the initial three plan PDFs locally; private snapshots/pages/blocks/chunks/checksums stay in ignored `day_trading/embeddings/`. Default run produced 407 pages and 2,405 draft chunks, not an accepted/frozen corpus. CFI layout segmentation, full visual/reading-order review, verified header/hyphen cleanup, section relationships and shared candidate-tokenizer safeguards remain open. No embedding calls, PostgreSQL work, services, EPUB support or learner Lesson 12 integration were implemented or authorized in this phase. Preserve the Nemotron unsupported-conversion blocker and deferred fidelity validation. Follow `embedding-code/README.md` for exact commands/artifacts; do not treat a completed draft manifest as permission to embed. Eleven preparation tests and format/check/Clippy passed; the unchanged root passed its 26 tests and build/lint checks, without a fresh live tutor run.


### Reviewed PDF margin cleanup - 2026-10-12

The instructor rejected beginners-guide cover/branding/margin chunks. `embedding-code/src/cleanup.rs` now uses a reviewed source-checksum-bound rule, not generic repetition removal: cover page 1, exact first-line brand headers and matching last-line footers on pages 2-18. Raw pages remain intact; cleaned pages preserve line numbers and excluded-lines JSONL records every removal. Beginners chunks decreased 92 to 56 (38 lines excluded); other books are unchanged, total 2,369 draft chunks over 407 pages. New run and preview instructions are in `embedding-code/README.md`. Fourteen preparation tests plus formatting/check/Clippy and real artifact/provenance checks passed. Full layout acceptance, other-book cleanup, tokenizer/freeze gates and embedding calls remain deferred; no root or dependency changes.

### Larger Heitkoetter draft chunks - 2026-10-12

Instructor rejected tiny Heitkoetter chunks. For its inspected source checksum only, `page-block-groups-v1` groups adjacent paragraph/layout blocks within physical pages, retaining formatting and the 3,000-character cap. Layout blocks are not overlap candidates; raw provenance and all block coverage remain intact. Document records include chunk_policy and grouped IDs bind it. Heitkoetter decreased 1,318 to 273 chunks, median 82 to 1,252 characters, maximum 2,742. Headers/page numbers still require reviewed cleanup; no cross-page/semantic section claims. Beginners and CFI chunks unchanged; total 1,324 draft chunks. Fifteen preparation tests and format/check/Clippy plus actual hashes/provenance/coverage/bounds passed. New run/preview in embedding-code/README.md. No embeddings, dependency or root changes.

### CFI reviewed margins - 2026-10-12

Instructor requested CFI headers/footers removal. Cleanup v2 retains beginners rules and adds CFI-checksum-bound first-nonblank exact publisher/title header and last-nonblank website/matching physical-page footer for pages 1-116. Body occurrences, cover title and section headings remain; raw pages and original line provenance preserved, 232 exclusions audited. CFI chunks 995 to 765; other books unchanged (56 beginners, 273 Heitkoetter), 1,094 drafts total. Sixteen tests and format/check/Clippy plus actual hash/provenance/bounds checks passed. Latest run/preview in embedding-code/README.md. No embeddings, database, dependencies or root changes; full layout/tokenizer acceptance deferred.

### Larger CFI draft chunks - 2026-10-12

Instructor rejected small cleaned CFI chunks. Its inspected checksum now selects existing page-block-groups-v1, preserving cleaned layout text/page boundaries and 3,000-character cap. CFI decreased 765 to 182 chunks, median 369 to 1,634 characters, max 2,991. All cleaned blocks covered, all 232 margin exclusions unchanged, other books unchanged (56 beginners, 273 Heitkoetter), total 511 drafts. Sixteen tests and format/check/Clippy plus actual hashes/provenance/coverage/bounds checks passed. Latest preview/run in embedding-code/README.md. No embeddings, database, dependency or root changes; visual/section/tokenizer/freeze acceptance remains open.


## Chunk-policy comparison decision — 2026-10-12

Instructor agreed to compare **one nonempty physical page plus 10% preceding-page word-tail overlap** against **section-aware paragraph groups targeting roughly 300–500 words**. Whole major sections/subsections are too variable for direct chunking; they remain diagnostic reports only. Review inferred headings and remaining Heitkoetter margins first. Page overlap uses ceil(10% of previous physical page base words), no recursive overlap, with empty pages resetting carry-over; structural overlap for section-aware groups must be explicitly settled/versioned within section boundaries. Word targets are not model-token limits. Use the same reviewed text/exclusions and source-span relevance questions; compare coverage/boundaries, size/overlap distributions, retrieval/evidence completeness and cost under fixed model settings and equal returned-token budgets. Freeze policy-specific collections and a common collection per policy across candidate models, then choose policy/model from evidence. See Section 6 and Section 9 of `lessons/12-specialized-retrieval/PLAN.md`. This request updates documentation only: current 511 draft chunks, Rust/Python code, root application and dependencies remain unchanged; no embedding calls, corpus freeze or database work authorized by this decision.


### Harrier test embeddings and PostgreSQL — 2026-10-12

Instructor selected the downloaded Microsoft Harrier OSS v1 0.6B and authorized local test-set embedding, PostgreSQL loading, Docker-compatible startup, DDL/SQL and working similarity query code in `embedding-code/`. Follow `embedding-code/RETRIEVAL.md`. This supersedes prior preparation-only no-embedding/database restrictions for this subset only; root remains Lesson 11, not authorized Lesson 12 integration. Preserve saved 511-chunk draft status and provenance; no final freeze or alternative policy implementation. Official-checkpoint CPU reference runtime avoids claiming unvalidated GGUF fidelity; learner llama.cpp compatibility remains open. Use pinned model/profile for both passage/query embeddings, halfvec(1024), cosine HNSW, loopback-only services, ignored private credentials/data and reader/admin separation. Do not mix incompatible model profiles or upload private text.

Validation: collection `harrier-test-511-v1` contains 511 vectors (56/273/182); actual tokens 3–522, full original metadata/draft flags and SHA-256 retained. Local PostgreSQL18.6/pgvector0.8.7 startup/rerun, read-only three-topic query smoke checks, duplicate-load avoidance, permission rejection and HNSW plan verified. 26 automatic preparation Rust tests + enabled real-corpus test, 22 server tests +3 comparison tests and format/check/Clippy/IDE build passed; unchanged root26 tests/checks passed without fresh tutor run. Docker-compatible script was run with Podman, not separately Docker. Official CPU bfloat16 inference/float32 normalization; selected service identity/profile is pinned. No expert retrieval acceptance, GGUF parity, corpus freeze or full-context inference claimed. Services remain loopback running; shutdown instructions are in RETRIEVAL.md.


### Harrier llama.cpp runtime switch — 2026-10-12

The instructor explicitly requested the existing **llama.cpp service on port 9999** instead of the Python reference runtime. The preparation Rust client now targets its OpenAI `/v1/embeddings` API, applies the query instruction in Rust and preflights tokenizer/context counts. The active collection is **`harrier-llamacpp-test-511-v1`**, re-embedded with the pinned F16 GGUF/runtime profile; the original `harrier-test-511-v1` Transformers collection is preserved and must not be mixed. Follow `embedding-code/RETRIEVAL.md` and Lesson 12 `DATABASE-SETUP.md` for current startup/load/query instructions. This changes only preparation, not root Lesson 11 or learner Lesson 12 integration. The existing user's server is bound to 0.0.0.0; instructions use 127.0.0.1, but no automatic restart occurs. Cross-runtime parity and final retrieval quality are not claimed.

llama.cpp switch validation: new collection511 entries (56/273/182), actual tokens3–522 with per-input usage equality, norms0.999964–1.000039, no content-hash mismatches/all draft flags preserved. Original HF collection511 preserved and mismatch query rejected; repeat new load idempotent; three real reader topic queries passed. 32 automatic Rust tests plus2 explicitly enabled corpus/live tests, format/check/Clippy/IDE build passed. Initial combined opt-in invocation omitted required run path and failed; corrected invocation passed both. No root change/fresh tutor run, server restart, parity benchmark or freeze. Runtime file identity uses content hash, not mandatory HF-cache directory name, so copied verified local GGUF startup works.


## Superseding Lesson 12 implementation authorization — 2026-10-12

The instructor explicitly authorized implementing Lesson 12, its complete lesson/instructor materials, and root replacement after confirming the tutor service was running. Root now advances from the preserved Lesson 11 reference to Lesson 12. Use the existing provisional three-PDF `harrier-llamacpp-test-511-v1` collection; no final corpus freeze, full-source conversion, runtime parity claim, or Lesson 13–14 implementation is implied. The complete Lesson 12 and its validation record supersede earlier learner-integration restrictions only for this scope. Keep all earlier references and unrelated preparation work intact. Private-data access/distribution rights remain the operator's responsibility; do not distribute the books or excerpts with the public course.


### Lesson 12 integrated validation — 2026-10-12

Root/reference are supplied and synchronized. Format/check/Clippy/IDE build,40 deterministic tests and2 explicitly enabled real-service tests passed. Live source search/citations and correlated$100 calculator, memory approved/rejected additions, plain/NO_COLOR/dumb and colored arrow UI retry passed. Keep teaching defects visible: paraphrased/feedback answers confused source scope or relationship wording despite valid IDs; unsupported-price refusal made no search; consented hand-edited memory loaded but welcome did not offer its activity. Initial colored search had no final ordinary answer; partial diagnostics survived and no approval occurred. Reader PUBLIC TEMP was revoked and startup now preserves that restriction. No corpus reload/freeze, server restart, parity/expert benchmark or offline network-isolation test. See Lesson12's detailed validation record; instructor review remains pending.


## Lesson 13 authorization and pinned SDK decision — 2026-10-12

Instructor authorized full Lesson 13 application changes, lesson and instructor documentation. Root now continues with Lesson 13; preserve Lesson 12 reference and preparation. Published `goose-agent`, `goose-providers` and matching provider types remain exactly alpha.11; mutable local checkout dependencies are withdrawn. Review SDK migration only after the class is finished and verified. Earlier historical root/planning restrictions above are superseded for Lesson 13 only, not Lesson 14.

`/learned <concept name>` is a local learner declaration, not a model tool or mastery assessment. With memory consent enabled it narrowly authorizes its deterministic completion append; without consent it is ephemeral. Model-proposed notes still require independent default-no approval. Retain separate authoritative state, approved memory and lossy working context; never treat generated summaries as genuine learner quotation evidence, saved-note authorization or support for current-turn citations. Continue the same Session after safe-boundary compaction; failure must preserve history and report any already-saved declaration without automatically repeating storage. Follow Lesson 13 source/materials for exact limits, provenance, settled-approval rules and honest validation. No later lesson is implicitly authorized.


### Lesson 13 implementation evidence — 2026-10-12

Root/reference match;62 deterministic tests,2 explicitly enabled real-service tests,format/check/Clippy/IDE build passed. Live command compaction with consented deterministic save reduced74730→57191 estimated bytes; ephemeral command reduced68885→53213 and follow-ups retained same Session. Ordinary approved/rejected proposals, fresh source search, correlated$100 calculator and colored declaration/plain modes checked. Initial reasoning-metadata summary rejection and former2000-token truncation retained history; now provider thinking metadata remains diagnostics-only, summary ordinary-text cap6000, generation cap4096. Conservative input estimate includes native JSON plus argument/thinking replay/protocol margins, total100000/headroom16000, trigger60000; not tokens or exact wire capacity. Tutor output4096 tokens/65536 streamed text/2000000 Event bytes; deadline180 seconds plus5 cooperative-cancel grace, unavailable hard-abort usage reported. Long source/calculator runs safely stopped when protected input exceeded84000; no mid-turn rescue claimed. Teaching execution guarantees and summary open-question attribution defects remain. See LESSON.md for full evidence/gaps; no Lesson14 authority or completed release review.


### Lesson 13 targeted semantic correction — 2026-10-13

Instructor requested fixing observed execution guarantees, tutor answers falsely closing learner questions and invented agreed-activity details. Lesson13 now uses speaker/ID/provenance-labeled ordinary-content summary input, excluding provider reasoning/internal metadata while preserving originals and actual correlated tool data. Tutor/summary guidance distinguishes execution priority from certainty, learner confirmation from tutor explanation, and reply length from accepted activity scope.66 deterministic and2 explicitly enabled real-service tests,format/check/Clippy/IDE build passed; four live compaction/follow-up replays retained uncertainty and accepted comparison scope without stop-execution guarantees in targeted answers. An initial “brief” embellishment prompted exact-acceptance guidance tightening; retry passed. Other buy/sell limit, price-or-better and spread wording defects remain, so no general semantic reliability claim. Detailed evidence in Lesson13; pins/predecessors/preparation unchanged, no Lesson14 authorization or release review.


## Lesson 14 implementation authorization — 2026-10-13

Instructor authorized Lesson14 only and minimal root continuation. Root now uses `lessons/14-agent-evaluation/content` and synchronized complete main/retrieval/compaction/telemetry modules. Lesson13 and all earlier references/preparation are preserved. Follow Lesson14 LESSON, MLFLOW-SETUP, RUBRIC, SCENARIOS, HELD-BACK, COMPARISON and VALIDATION. Published alpha.11, RMCP, toolchain/provider pins unchanged; telemetry crates explicitly selected/locked. Official local minimal MLflow v3.17.1 image pinned by digest; SQLite and artifacts in ignored `.evaluation/mlflow`, never retrieval database. Podman tested; Docker-compatible, not separately Docker-tested.

Use native Rust tracing/OTLP and native MLflow individual human feedback, not full Goose CLI exporter assumptions, custom Python runner/package, evaluation engine, dashboard, hosted judge or summed quality score. Editable rubric v1 has ten application-quality criteria and separate unscored statuses; critical deterministic gates cannot be averaged away. Capture ordinary dialogue/results/evidence/summaries/approvals; do not evaluate provider reasoning or deliberately export credentials/configuration. `EVALUATION_STORAGE` isolates BOTH memory and diagnostics under repository `.evaluation/sessions/<simple-name>`; never move/overwrite real learning record. Keep consent, independent approval, local declaration, quotation provenance, GDK loop and all budgets. Actual evidence and incomplete/blocked branches stay visible. No potential Lesson15, SDK migration or release approval.

### Lesson 14 validation handoff

Root/reference synchronized;68 deterministic tests plus2 explicitly enabled actual-service tests and format/check/Clippy/Cargo build passed. IDE build attempted but blocked because only an unrelated project was open; no IDE success claimed. Official minimal MLflow native traces, individual numeric UI feedback(+1 exact calculator,−1 compaction scope), and SQLite/artifact restart persistence checked. Final tutor chat UI and persisted summary input/output/usage verified without duplicate application usage. Fresh60000/50000 comparison preserved hard84000 allowance; actual compaction/follow-up, isolated consent/rejected/approved memory and plain/dumb/colored-arrow UI checked. Keep early no-final-answer, fabricated calculator/budget stop, output-cap retry, source scope, strengthened/omitted activity and false learner-comfort findings. Final50000 replay compacted69501→55046 and69436→55018, retained uncertainty/scope in final answer but earlier false comfort remains. V04/V05 new semantic runs, V10 separate compatible synthetic fixture collection, held-back cases and expert quality remain incomplete/blocked. Full details in Lesson14 VALIDATION; Docker not separately tested, review/release pending, no Lesson15/SDK migration.


### Lesson 14 scope A — native dataset catalog/manual execution

Instructor authorized the native MLflow case catalog and supplied Rust seeder only, not an evaluation runner/judge or tutor/Goose API change. Follow `lessons/14-agent-evaluation/DATASETS.md` alongside LESSON, MLFLOW-SETUP, SCENARIOS, RUBRIC and actual VALIDATION. Default manifest `datasets/visible-v1.json` seeds native dataset `lesson14-visible-v1`, ten V01–V10 records; V04 conflict remains a branch of V04. Root-registered `seed_dataset.rs` runs as `cargo run --example seed_mlflow_dataset -- <active-numeric-experiment-id>`; append `--check` for read-only catalog verification. Fixed 127.0.0.1:5000, no proxies/redirects, pinned v3.17.1 publicly undocumented native API/3.0 dataset surface, no Python package/direct SQL writes. Protocol JSON-string tags/records and input-based dedup were source-inspected; do not assume upgrades are compatible.

Preserve manifest SHA-256/schema identity, refusal of duplicate names, same-name/different-manifest and edited/extra records; skip exact records and permit exact interrupted subsets to resume. No native immutable dataset versions: new definitions need a new manifest/name, not silent mutation of reviewed v1. The catalog is authored inputs/setup/inspection targets, not private excerpts, expected tutor transcripts, imported traces or completed reviews. All Q judgments remain separate and initially unreviewed; blocked/skipped/NA are not numerical zero/pass. Held-back cases unchanged/unseeded; V04 conflict/V10 blocked until separately prepared compatible fixtures/constrained selection, no changes to the private corpus or embedding service implied.

Execute manually with unique isolated EVALUATION_STORAGE for every condition/case/repetition, matched state and actual consent/approval decisions. A seeded expected approval branch authorizes nothing: inspect actual note/category/quotation after successful run, default no, approve only appropriate additions. Copy actual dataset ID/record ID/case/manifest/rubric/comparison/reviewer/evidence into trace Notes and individual feedback rationale. No automated trace-to-record linkage, native eval-run orchestration, summed score or judge. Seeding/checking is catalog evidence only; use VALIDATION for actual checks. Preserve main/reference sources, pins, services and earlier materials; no service restarts, Lesson15 or release approval implied.

### Scope A dataset catalog evidence — 2026-10-13

Native `lesson14-visible-v1` now stores10 authored V01–V10 records in existing experiment1 (`lesson14-compatibility`), dataset `d-c2a3478604b54bb795b2b342a6840f5a`; full API readback/association and repeat/check unchanged IDs/timestamps/content verified.7 catalog tests+68 unchanged application tests,format/check/Clippy/Cargo build passed;2 opt-in application live tests not rerun. Root/reference modules and lock unchanged. Plain/NO_COLOR/dumb plus actual PTY palette checked. IDE build blocked by unrelated open project. No dataset browser automation, new restart-persistence test/service restart, fresh tutor execution or rubric judgment; API catalog evidence is not completed evaluation. See Lesson14 VALIDATION. New versions need separate reviewed seeder/path/name changes; preserve v1. No Lesson15/SDK migration/release authority.


## Lesson 8 modularization — 2026-10-22

Instructor authorized behavior-preserving Lesson 8 modularization and corresponding documentation corrections. Keep Session/runtime/Effects, Event consumer, machine assembly/run/concurrency/shutdown, and validation in main (289 lines). Local presentation/provider modules and separate tests keep supporting details out of the instructional path; all eight original tests remain. Copy all four `.rs` files with the lesson's full-root-source backup/restore workflow, not main alone. No shared course library, dependency, root Lesson14, or other lesson change. This supersedes single-source wording for Lesson8 only; the instructor wants eventual modularization across lessons, but only Lesson8 is in this pass. Temporary-staging format/check/Clippy/test/build and fresh live streaming, colored/plain/NO_COLOR/dumb exchanges, independent stdout/stderr gating, and Japan tweak passed after the initially unavailable endpoint came online. France was restored and rerun; no service was restarted by the agent. Historical completion language was corrected to preserve pending instructor review. Do not infer release approval.


## Lesson 9 modularization checkpoint — 2026-10-22

Instructor authorized behavior-preserving Lesson 9 modularization only. Keep Session/runtime/Effects and handler, Event consumption, tool provider/safe dispatch, limits, machine assembly/run/concurrency/shutdown and inspection visible in main. Local `calculator.rs` holds the previously taught LossArguments/exact-decimal execution/schema; `presentation.rs` and `provider_setup.rs` hold supplied support; `tests.rs` and presentation-local tests retain coverage. Explain modules before declarations and supply the complete source map/reading path. Copy all five reference `.rs` files into the ephemeral root; no backup/restore is required. This current root decision supersedes earlier claims that root must remain a particular later lesson and the earlier Lesson 8 backup/restore convention, without changing that lesson's files. Later lessons, dependencies and provider behavior remain unchanged; stop at the Lesson 9 checkpoint. Fresh formatting/check/Clippy/build and all15 tests passed; all62 original function/test bodies match after whitespace normalization. Fresh default and100-share live runs passed, resolving the live-tweak gap; default200 restored. Colored/plain/empty-NO_COLOR/dumb and independent stderr gating passed. Conceptual first-turn “caps” wording remains a teaching defect; instructor review pending. No IDE build, projector review or service restart; see Lesson9 validation. Root/reference match; no later lesson/seeder/dependency changes.


### Lesson 9 outbound-preview clarification — 2026-10-22

Instructor approved explicit snapshot timing after input append/before machine run, a complete retained conversation-history preview (not a final serialized request), and separate instruction/advertised-definition labels distinguishing capability schema from an execution request. Root/reference presentation match; provider input/history/dispatch unchanged. Added no-state-mutation/history-order test;16 tests plus format/check/Clippy/build and fresh plain live default passed. Fresh100-share/PTY modes not rerun for this follow-up; prior evidence remains separate. Instructor review and first-turn semantic defects remain pending.


## Lesson 10 modularization checkpoint — 2026-10-22

Instructor authorized Lessons10–14 one at a time, with questions before each lesson and review checkpoints. Lesson10 boundaries were explicitly confirmed: extract calculator, presentation, provider_setup, chooser, diagnostics and tests; keep context loading/prompt assembly in main for now, alongside selection/reply lifecycle, runtime/Effects/Events/dispatch/limits/machine run/concurrency/shutdown and inspection. Later movement of context assembly needs a separate decision. Every Rust source starts `// Lesson 10`; root is ephemeral, activated from all seven reference files and matches them. No backup/restore or final hello-world reset now.

All86 original function/test bodies match after whitespace/optional-trailing-comma normalization; all20 original tests and fmt/check/Clippy/build pass. Fresh live conversation/reselection, optional correlated$100 calculator, blank/EOF/commands, colored arrow UI and plain/emptyNO_COLOR/dumb modes, independent stderr gating, complete plain diagnostics and manifest-root paths from /tmp passed. Initial unused test import removed; no behavior fix. Safety-net/closing-before-further-loss wording remains a teaching defect; prior guidance ordering experiment remains inconclusive and was not rerun. No IDE/projector review, service restart, new live failure probe or release approval. Lessons11–14/seeder/pins/content untouched; ask questions before Lesson11 and stop at Lesson10 review checkpoint. See Lesson10 dated validation for full scope and gaps.


## Lesson 11 modularization checkpoint — 2026-10-22

Instructor confirmed inherited support modules plus `learning_record.rs` for types/schema/storage/confirmation-input mechanics. Keep context assembly, consent, proposal-only tool handler, `parse_record_proposal` actual-learner-source validation, successful final/inspection gate, `approve_record_proposals`, decision log, append call and Session refresh visible in main, with runtime/Effects/Events/dispatch/limits/machine lifecycle. `TerminalProse` is presentation support, not Event consumption. All eight source files begin `// Lesson 11`; activate all into ephemeral root, no backup/restoration or final hello-world reset. Later context relocation needs a separate decision.

All105 original function/test bodies match after whitespace/optional-trailing-comma normalization; all26 original tests/constants and fmt/check/Clippy/build pass. Unused test imports removed; root/reference synchronized. Fresh colored arrow proposal/approval (new0600), rejected expanded activity then inspected narrow retry, rejected goal preserving bytes, consent-declined record exclusion, correlated$100 calculator, plain/emptyNO_COLOR/dumb, commands/blank/EOF/reselection, independent stderr and /tmp manifest-root paths passed structurally. Manual-edit resumption mixed: first welcome ignored loaded activity, subsequent welcome offered it. First tutor prematurely said “noted” and proposal added unagreed check scope; rejected without write, no prompt fixes. Preserve these defects and historical timeout. Complete plain diagnostics/post-approval context verified; test record removed. No IDE/projector review, fresh live failure probe, service restart or release approval. Lessons12–14/seeder/preparation/pins/content untouched. Stop for Lesson11 review; ask questions before Lesson12. See dated lesson/notes evidence.

## Superseding Lessons 12–14 refactor authorization — 2026-10-22

The instructor now authorizes completing the behavior-preserving modularization of Lessons 12, 13 and 14 autonomously after context compaction, with one combined review after all three. This supersedes the earlier requirement to ask questions and stop for review before each remaining lesson. Work sequentially, make conservative boundary choices following Lessons 10–11, and only ask if a genuine behavior/scope change or missing prerequisite cannot be resolved within that authorization. “Fix” here means instructional modularization and corresponding documentation, not prompt/semantic fixes or architecture redesign.

Keep context loading/prompt assembly, Session/runtime/Effects, concurrent Event consumption, dispatch/allowlists/limits, machine lifecycle and lesson-specific authority decisions visible in main. Carry forward local calculator/chooser/diagnostics/presentation/provider_setup/learning_record support where appropriate; keep genuine-learner-source validation and approval/log/append/Session-refresh sequence visible. Preserve existing cohesive retrieval/compaction/telemetry production modules; move their tests into module-local child files as useful rather than mechanically subdividing algorithms. Lesson12 evidence budget/registration/current-turn citation inspection, Lesson13 authoritative-state/provenance/command/preflight/revision/maintenance boundaries, and Lesson14 instrumentation/run_with_telemetry/isolation stay visible at their existing conceptual boundaries. Update Lesson14 telemetry's explicit source-hash inventory for all extracted production modules; preserve its metadata purpose/schema. Leave standalone dataset seeder and dataset assets untouched.

Every Rust file starts `// Lesson N`. Activate complete source trees into ephemeral root (including module-local test directories), preserve authoritative references and unrelated existing work, and finish with root matching Lesson14. Do not reset to hello world until all review is complete. No shared crate/framework, new traits/dependencies, upgrades, content/prompt changes, incidental behavior fixes, service restarts, corpus mutations or evaluation-catalog writes. Verify exact pinned SDK source for API claims. Preserve all tests, original bodies apart from necessary module paths/visibility and approved source inventory, terminal/privacy/approval/provenance/cancellation/Drop ordering, budgets and failure behavior. Validate each activated lesson with fmt/check/Clippy/tests/build and applicable documented/live scenarios; unavailable services require an honest blocked report, never mocks or claimed success. Retain historical semantic defects/gaps and separate structural checks from teaching judgments. Update each lesson/instructor guide and current README/AGENTS, recording exactly what was and was not validated. Preserve Lesson11's existing uncommitted work. No commits or release approval requested.


## Lessons 12–14 modularization combined checkpoint — 2026-10-22

Completed the authorized sequential behavior-preserving refactor; root now byte-matches the complete Lesson 14 source tree. Keep all six local support modules and module-local test directories when activating references: replace ephemeral `src/` recursively, not main alone. No final hello-world reset until review is complete. Main retains context, runtime/Effects, concurrent Event consumption, dispatch/limits/lifecycle, evidence/citation inspection, genuine learner-source validation and approval/log/append/Session refresh. Compaction retains authoritative/provenance/command/preflight/revision/maintenance boundaries; Lesson 14 retains instrumentation/run_with_telemetry/isolation. Retrieval/compaction/telemetry algorithms remain cohesive; their tests are child files. Telemetry explicitly hashes the six extracted production modules without changing its metadata schema/purpose. Seeder/assets and existing Lesson 11 work were preserved.

Original bodies preserved: Lesson12 158; Lesson13 225; Lesson14 242 except the approved source-inventory expansion. Independent test-name/constant and literal/type/attribute audits passed with only module imports/visibility/test boundaries and the approved paths changed. Every Rust file begins `// Lesson N`. Each activated lesson passed fmt/check/all-target Clippy/tests/build (40/66/68 deterministic tests respectively; two real-service tests ignored normally); Lesson14 all-target tests also passed seven unchanged seeder tests, no catalog calls. Initial Lesson13/14 missing test-only DEFAULT_PROVIDER_CONFIG import and formatting order were corrected; no behavior fix.

Both real-service tests were explicitly attempted in each lesson and failed because RETRIEVAL_DATABASE_URL is missing; documented launches fail before chooser for the same reason. The documented private connection.env is absent; Harrier9999 and MLflow5000 unavailable, tutor8080 health200. No live chat/tool/approval/compaction/export success is claimed. Plain/emptyNO_COLOR/dumb startup failures and independent stdout/stderr gating passed; complete failure diagnostics are plain and fatal missing-reader errors surfaced. No services restarted, credentials invented, corpus/catalog mutated or mocks substituted. Supply prerequisites before fresh conversation/teaching/telemetry validation. Historical semantic defects, incomplete fixture-dependent/held-back evaluation and review gates remain; no IDE/projector review, SDK/dependency/content/prompt changes, commits or release approval. Stop here for combined instructor review.


## Lessons 12–14 service defaults — 2026-10-28

The instructor explicitly requested top-of-main `const` defaults for the supplied local reader connection, Harrier9999 and tutor8080 in Lessons12–14. Environment overrides remain; the supplied reader password is intentionally plaintext in source/binaries, an exception to the earlier no-source-credentials convention for this local classroom default only. Do not replicate it in diagnostics/telemetry or treat it as secure/production configuration. Tokenizer/provider-origin equality and all compatibility/reader checks remain unchanged. Preserve the user-activated Lesson12 root and staged changes; do not automatically restore Lesson14.


### Learner embedding runtime build policy — 2026-10-28

Instructor requested removal of exact llama.cpp server-build equality in the active Lesson 12 and Lesson 12–14 references. `build_info` no longer gates learner startup; model/GGUF identity, pooling/normalization, dimensions/capacity, query formatting, collection/profile/provenance and reader checks remain. The stored profile build suffix describes original corpus preparation, not a current-query-server requirement; cross-build numerical parity is not established. Standalone preparation code remains unchanged and separately build-pinned. Root stays synchronized with Lesson 12; existing staged/unrelated changes are preserved.

Root Lesson 12 and staged Lessons 13–14 passed fmt/check/all-target Clippy/tests/build (41/67/69 deterministic application tests plus seven unchanged seeder tests each). Both actual-service tests passed for each lesson against current build `b11669-620d2225c`; documented launches reached chooser and exited on EOF. No fresh full tutor/approval/compaction/telemetry workflow, parity benchmark, UI/IDE/projector or teaching review; no service restart, corpus/catalog mutation, dependency change or release approval. Historical failures/gaps remain recorded.

## Learner-led topic scope — 2026-10-28

Instructor authorized correcting the three teaching-content files in Lessons10–14. Four example units are not a topic whitelist or required progression; follow learner-chosen day-trading questions/depth and offer topic suggestions only when requested or help choosing is wanted. General broker-comparison education is allowed; preserve no personalized financial recommendations, honest current-information limitations, and each lesson's capability/consent/approval/citation/provenance/compaction instructions. Later-lesson content restored from its own reference, not the calculator-only Lesson10 contract. No Rust, pins, dataset assets, services, records or corpus changes. Root remains the user-activated Lesson12. Static content checks and active Lesson12 context-loading test passed; no fresh live teaching/approval/retrieval/compaction/telemetry validation claimed. Existing semantic gaps/review gates remain.

Follow-up validation: active Lesson12 all-target tests passed (41 application tests plus seven unchanged seeder tests; two actual-service tests ignored). No catalog calls or live conversation were made.
