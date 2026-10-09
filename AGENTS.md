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
5. Read the relevant `lessons/<number>-<topic>/LESSON.md` and its complete `src/main.rs` reference solution.
6. Read the corresponding `instructor-notes/` file when one exists.
7. Read the provider JSON used by the active lesson.
8. For any GDK API, provider-schema field, type, command, or behavior, consult the official documentation and the exact pinned SDK source. Do not rely on memory.

## Repository model

- `src/main.rs` is the exercise currently being developed and may be incomplete.
- `lessons/` contains numbered instructional units.
- A lesson's `LESSON.md` contains explanations and important snippets.
- A coding lesson's `src/main.rs` is its complete reference solution from the outset. It is not an independently packaged application. Replacement Lesson 7 is intentionally no-code and has no `src/main.rs`.
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

**`README.md` is the authoritative curriculum plan**, including replacement Lessons 7–9 and their implementation handoff. `post-lesson4-plan.md` supplies supporting architecture context and historical evidence; it must not override the README. It prioritizes the raw tool protocol, then the GDK state machine, before later market data, evaluation, retrieval, and CLI/model-selection work. The roadmap is not constrained to a fixed number of lessons.

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

Before starting a tool lesson, read `post-lesson4-plan.md` and the exact pinned GDK/RMCP source. Lessons 1–7 are complete. The instructor clarified on 2026-10-07 that Lessons 8 and 9 still need review, superseding earlier completion records for Lesson 8. The root previously contained the authorized replacement Lesson 8. The instructor authorized Lesson 9 on 2026-10-06; root now contains the separately authorized Lesson 10 learning partner. Keep Lesson 7 and Lesson 8’s workflow unchanged; Lesson 8’s terminal styling now follows the all-application rule above. Lessons 8 and 9 still need instructor review; Lesson 9’s default live scenario passed, but its live 100-share tweak is pending. Lesson 10 was authorized on 2026-10-07; its implementation does not release Lessons 8–9 or authorize Lesson 11.

## Lesson 10 implementation — 2026-10-07

The instructor authorized Lesson 10 and replacing root with its interactive day-trading learning partner. Follow README for scope and the complete `lessons/10-day-trading-learning-partner/` reference and notes. Only Getting started in day trading is available. Separate editable Markdown supplies common contract, educational material/priorities, and teaching guidance, loaded at selection and fixed for that Session. Arrow/Enter selection has a numbered fallback for redirection, NO_COLOR, and TERM=dumb; /finish returns to selection, /quit/EOF exit, blank replies are ignored. Selection creates a fresh Session; replies retain it. GDK owns the within-run loop. Preserve Lesson 9 calculator validation/correlation/all-block safeguards and per-turn limits. Calculator use is optional, not forced.

Lesson 10's presentation exception: learner terminal shows brief activity, framed tool results, and framed tutor streams; complete Session snapshots and internal diagnostics move to root `session-output.md`. Truncate at launch, append every turn/selection, preserve all fields/defaults/nested metadata and available partial state with Rust Debug fences; record loaded context, correlated tools, assembled saved tutor answers, warnings/failures. Markdown contains no ANSI, no promise of gold syntax colors, and is never read as learner memory. Ignore /session-output.md; ignored is not secured and conversations can contain personal data. Log errors stop the application. Never deliberately dump provider configuration or credentials. Keep course terminal palette/gating for normal output; no Session formatting/color helpers are needed when not printing Sessions.

Supplied crossterm 0.29.0 events-only supports selection; GDK/provider/RMCP/toolchain versions and provider JSON remain unchanged. Approved classroom tweak edits the guidance pacing sentence to use a short hypothetical example before a check-for-understanding question, then starts a fresh selection. Instructor content/teaching review remains pending. Lessons 8–9 review and Lesson 9's live 100-share tweak remain separate pending gates.

Lesson 10 validation: 20 tests and format/check/Clippy passed; live conversation/reselection, optional correlated $100.00, arrow/color and plain/NO_COLOR/dumb modes, commands/EOF, complete Markdown diagnostics, and fatal logging/provider partial-state handling checked. Guidance tweak loaded but example-before-check ordering was inconclusive; default restored. Keep teaching/content/UI review pending. Exact evidence is in Lesson 10 and instructor notes.

Lesson 11 now prioritizes a persistent learning record and learner-agreed to-do list, superseding evaluation-baseline-primary scope. Distinguish discussion, goals, demonstrated understanding, unresolved questions, and agreed next activities; do not infer mastery from tutor explanations. Storage/consent/corrections/tools remain undesigned. Do not scaffold Lesson 11 or any later lesson without authorization.

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

Lesson 9 source, prose, notes, and narrated SVG/PNG frames are supplied. At that handoff root matched its reference source; root now runs Lesson 10. `SHARE_COUNT` is the approved 200→100 tweak, shared by the prompt and structural check. The application uses a `ToolProvider` with typed `LossArguments` and a safe wrapper around the pinned `ToolOperation`; the SDK alone filters unknown names and supplies no request cap. Keep correlated unknown/excess errors and all-block handling. Bounds are eight applied work passes plus a possible terminal halt batch, four aggregate requests per user turn, and a 180-second run timeout; the engine owns the loop. Full partial state is shown on failure.

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
