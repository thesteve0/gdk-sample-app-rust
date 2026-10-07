# Instructor notes — Lesson 9

## Teaching intent and status

Connect the raw protocol from Lessons 5–6 to the GDK foundation from Lessons 7–8: same Session across two human inputs, selective use of an always-available calculator, and request/result correlation within one machine run. The entire implementation is supplied. Use predict → run → explain → tweak → run again, not missing Rust scaffolding.

**Implemented; default live scenario validated, live 100-share tweak and instructor review pending.** Authoring/build success does not release the lesson. Lessons 7–8 remain complete and unchanged.

## Preparation

Read the [lesson](../lessons/09-machine-conversation-tools/LESSON.md), complete [source](../lessons/09-machine-conversation-tools/src/main.rs), and [six lecture frames with captions/text equivalents](../lessons/09-machine-conversation-tools/diagrams/README.md). Root contains Lesson 9; shared-manifest execution is documented in the lesson. The bundled provider is replaceable. Start a streaming/tool-capable provider before class.

Check both default and approved 100-share tweak live before teaching. Keep an actual transcript locally, not a secret-bearing configuration. Do not silently replace unavailable inference with a mock or force the desired tool-selection pattern.

## Suggested pacing

1. **Predict:** ask whether a conceptual stop-loss question needs the calculator. Emphasize that the tool is available from turn 1. Predict `$100.00` for the numeric follow-up.
2. **Run:** observe the first answer, retained first exchange in turn 2, actual tool name/arguments/ID, deterministic response, and final explanation. Inspect every saved block, not a prose assertion that a tool ran.
3. **Explain:** trace application append → new run; then inference → save request → reload → tools → save result → reload → inference → save final answer → no work. A new pass is not a new human turn.
4. **Tweak:** change only `SHARE_COUNT` to `100`; predict `$50.00`. The validator uses the same constant. Run and compare result/correlation/continuity; restore `200`.

If turn 1 requests a calculator or turn 2 does not, identify it as observed provider/model selection and a failed scenario check. Do not remove tools on turn 1, revise the code to force requests, or hide errors. Invalid arguments remain safely rejected. Generated prose still needs human review: automatic checks do not grade grounding or exclusions.

## Progressive visual reveal

Use frames 01–06 in numeric order; each SVG has a corresponding 1600×900 PNG. The [diagram guide](../lessons/09-machine-conversation-tools/diagrams/README.md) gives full text equivalents and per-frame prompts.

- **01, first question:** reveal Session and capability banner before narration. “Does availability require execution?” No.
- **02, first answer:** point to preserved question/answer and separate Event/Effect paths. “Who must save the next input?” The application.
- **03, follow-up:** show retained first exchange and numeric human input. “What structured output would justify execution?” A request with allowlisted name and arguments.
- **04, request:** highlight quoted price strings and `example-call-1`. “Which Operation now has work?” Tools, because the request has no matching response. Saving is not execution.
- **05, result:** predict arithmetic before revealing it. Follow validation → exact calculation → response with the same ID. “Is the response the final assistant answer?” No; inference uses it next pass.
- **06, explanation:** connect IDs, final answer, and no-work exit. “What changes for 100 shares?” Result becomes `$50.00`, not the machine structure.

These are narrated examples, not an observed transcript or guaranteed model behavior. Snapshots select recorded data for readability; actual program output expands all fields. The diagrams omit the supplied safeguard step to focus the normal route. Do not teach Goose's full assembly as required infrastructure.

## Code navigation and application choices

Only show focused boundaries unless asked:

- `main` and `append_user`: same store/ID/machine/capability, new human input and per-run reset. In-memory retention is not disk durability.
- `SafeTools`, `LossToolProvider`: the SDK operation advertises and dispatches; the application provider reuses typed `LossArguments` and Lesson 6 validation. This is not the RMCP `SyncTool` route: it preserves text dollar output without new dependency/serialization scaffolding.
- `prompt_parts`: educational guidance reaches inference through the default preparer, which joins contributed text into the system instruction. Operations shape inference inputs as well as act on saved state.
- `execute_allowed_tool`/`execute_maximum_planned_loss`: allowlisted names, strict shape, immediate exact decimal parsing, positive values, four-place precision, entry above stop, checked arithmetic and two-place dollars. Unknown fields and excessive decimal range fail safely. The schema is not runtime validation.
- `ChatEffect`/handler: message and usage retention from Lesson 8 plus halt reason; the handler counts applied batches. Do not claim only the step list changed.
- `run_turn`/`display_events`: concurrent receiver, sender close/drain, Session view even after a failed run. Tool-response Events are application/tool content; display does not save it. The handler now emits a presentation-only request copy after saving the complete reconstructed message, before the next machine pass. The same FIFO orders request → result → final stream. The display marker exists only on that copy, not in stored history or provider input. Validation no longer reprints requests at the end.

Explain supporting traits/locking/shared ownership only as needed. They are supplied integration contracts, not attendee implementation exercises.

## Output interpretation

Lesson 9 returns to Lessons 5–6 payload fences and sender/receiver actor labels. Do not alter Lesson 8's no-fence exception. Instruction/tool-definition output shows contributed values, not the complete prepared/serialized request. Raw Session snapshots show every field including absent/default nested metadata and all usage records, with two blank lines afterward. Long reasoning content is not silently truncated.

The instructor-approved color scheme adds bold cyan headings, blue current human input, magenta request blocks, green tool responses/success checks, yellow warnings, and bold red errors. Streamed answers use normal terminal color. **Dim only the Session `usage` section**, never conversation history or other metadata. Every Session attribute name, including nested fields and quoted object keys, is gold (`#FFD700`); values and Debug type names are unbolded. Usage values remain dim. Styling resets before `applied_passes`; snapshots remain complete. Color supplements actor labels/fences and changes no stored/provider data. It is automatically disabled on redirected streams, with `TERM=dumb`, or when `NO_COLOR` is present. Compare in a real terminal and check projector contrast; do not remove fields to simplify output. The extra human-input block highlights the current question, not a full serialized provider request.

The live consumer displays text and tool responses; saved Session views expose all content. The same answer appearing in Events and state is intentional. The pinned inference runner records final usage through Effects, so do not promise a usage Event per exchange.

`inspect_turn` checks some displayed text, an ordinary saved final assistant answer, request inputs, exact deterministic result, matching IDs, and first-turn selection. It does not prove final live text equals saved final text or that the final explanation faithfully uses `$100.00`. Review both yourself, including fees, slippage, gaps, and the distinction from statistical expected loss/guaranteed realized loss. Never call a wrong selection pattern a successful classroom validation.

## Safeguard semantics

Application choices: eight applied work batches per user turn; four aggregate requests per user turn, including unknown/malformed calls; 180-second run timeout. `RunLimit` is before tools/inference. A terminal halt Effect may produce a displayed pass count of nine. Ordinary no-work completion at the bound remains allowed.

The shipped SDK tool operation filters unknown names and has no request limit. `SafeTools` answers unknown/excess requests first. Re-evaluation lets the shipped operation handle recognized and malformed requests next. Excess requests are not executed; once all responses exist, exceeding the request bound halts without further inference. The timeout/pass bound can interrupt before all requests are answered; show partial state honestly and do not resume automatically.

Empty/duplicate IDs yield a recorded halt rather than fabricate correlation. The SDK excludes externally executed requests; this application exposes no such mechanism. A timeout cancels/drops local execution, not rollback or exactly-once remote behavior. These supplied paths are maintenance safety coverage, not deliberate classroom failure exercises.

## Pinned-source verification

Reviewed published `goose-agent` **0.1.0-alpha.11**:

- `machine.rs`: ordered first-applicable scan, operation contributions before inference, effect application/reload, explicit yield, and no built-in application pass bound on `run`.
- `operation.rs`: current-turn kickoff selection excludes tool-response user messages; Events await sends through Emitter; Effects are separate; `ends_turn` checks ordinary assistant output.
- `tool.rs`: all current-turn blocks scanned; answered IDs/external calls excluded; unknown names filtered rather than answered; sequential pending dispatch; parse errors returned; user-role response content retains IDs/provider metadata; provider registration stable across both turns.
- `inference.rs`: default preparer joins prompt parts, streams Events and reconstructs message Effects, saves usage, checks applicability, and yields diagnostics on empty/error output.
- `events.rs`: variants consumed by display.

Reviewed matching provider-types message/conversation/usage and RMCP **3.4.1** request/result interfaces, along with exact provider declarative construction. The [official provider documentation](https://goose-docs.ai/docs/getting-started/providers) supplies provider orientation; exact pinned source is authoritative for the state-machine interfaces. Do not substitute the full Goose operation assembly.

## Validation record — 2026-10-06

- `cargo fmt --check`, `cargo check`, `cargo clippy --all-targets`, and `cargo test` passed. Eleven tests passed, covering exact `$100.00`/`$50.00` arithmetic and invalid shape/domain/range; same capability/history across turns; all-block/multiple-call correlated dispatch including unknown and parse errors; excess-request errors and halt; duplicate IDs/pass safeguard; message/usage Effects, snapshot independence, complete fields, no-work applicability, and concurrent channel draining without saving by display.
- IDE project build passed with no reported problems.
- Live `cargo run -- custom_aa_llama_qwen3_6-35b.json` was attempted. Connection to `127.0.0.1:8080` failed on turn 1; the saved inference diagnostic was displayed and the process returned an error. An independent endpoint probe also failed. No real first-turn selection or second-turn tool round trip was validated.
- The 100-share calculation is unit-tested, not live-provider validated. Request-limit/unknown/malformed tests construct protocol input locally; they are not model-behavior evidence.
- Local Markdown links, all six SVG XML files, root/reference-source equality, and `git diff --check` passed. All six rendered frames were visually inspected for layout/readability. These are narrated teaching assets, not evidence of live correctness or classroom/projector approval.

**Before release:** start/supply a provider; run both default and approved tweak; inspect actual tool-selection patterns, arguments, IDs, results, retained history, live Events, final explanation/exclusions, and termination. Record outcomes, then request instructor review. Do not mark Complete merely because deterministic checks pass.

### Output-order fix and live revalidation — 2026-10-06

The late request print in `inspect_turn` was removed. The handler sends reconstructed request presentation through the existing Event queue after saving, avoiding partial-argument dumps and keeping all output in one consumer. A regression test verifies complete request display → correlated tool result → final-answer Event ordering and that display metadata never enters saved history.

Formatting, checking, Clippy, and all **12 tests** passed. The default live run succeeded: turn 1 had no requests/responses; turn 2 had one calculator request and one matching response, `$100.00`, followed by the final explanation. Actual output placed the reconstructed request at line 371, tool-result Event at 398, and final answer stream at 449 in the captured local transcript. There was no late request block after the Session view. The final explanation used the correct amount and listed fees, slippage, and gaps as exclusions. These observed positions/counts are not universal output assertions. The approved 100-share tweak remains live-unvalidated; instructor release review remains pending.

### Terminal colors — 2026-10-06

Instructor-approved presentation change: use the color scheme above and dim **only usage**, preserving full snapshots, labels, fences, and request/result/answer order. Root and reference match; no dependency, provider, or Lessons 7–8 changes. Formatting, compilation, warning-free Clippy, all **14 tests**, and IDE build passed. Three default live runs passed: colored pseudo-terminal, redirected plain transcript, and `NO_COLOR=1` pseudo-terminal. Captured ANSI checks confirmed all normal palette categories, four usage-only dimmed sections with reset before other fields, no ANSI in plain modes, and complete request → result → final stream order. Human projector/contrast review and the live 100-share tweak remain pending; no release status change.

### Bold Session attributes — 2026-10-06

Instructor requested bold attribute names throughout Session snapshots, including nested fields and quoted object keys, with values unbolded. Only usage remains dimmed. Formatting preserves all raw Debug values; string values containing colons and Debug type names are not treated as labels. ANSI bold reset restores dim for usage values. All 15 tests, formatting/check/Clippy, IDE build, and the default colored live scenario passed. Root/reference match; previous lessons unchanged. Live 100-share tweak and instructor release review remain pending.

### Gold Session attributes — 2026-10-06

Instructor found bold difficult to see and replaced it with gold (`#FFD700`) for every Session attribute name, including nested fields and quoted object keys. Values use the default foreground and are unbolded; only usage remains dimmed. Foreground-only reset preserves usage dimming. Root/reference match; 15 tests, formatting/check/Clippy, IDE build, and default colored live run passed. Plain fallback and complete snapshots remain intact; Lessons 7–8 unchanged. Live 100-share tweak and release review remain pending.
