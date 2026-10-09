# Instructor notes — Lesson 10: Day-trading learning partner

## Teaching objective and status

Learners should explain how an application becomes a specialized educational partner by choosing and supplying its own content, keeping a focused conversation, and enforcing a limited capability boundary. The lesson is not about writing another Rust loop or reproducing Goose's application assembly.

**Implemented; structural checks/live scenarios passed; instructor review pending.** Lessons 8 and 9 still need their separate instructor reviews; Lesson 9's live 100-share tweak is still pending. Lesson 10 work does not approve either lesson for release.

Supply the entire implementation from the outset. Teach **predict → run → explain → tweak → run again**. The only customization exercise edits a sentence in the Markdown teaching guidance. Chooser events, raw-mode restoration, runtime traits, synchronization, logging, validation, and concurrency are supplied support, not attendee scaffolding assignments.

## Prepare before class

- Read the [lesson](../lessons/10-day-trading-learning-partner/LESSON.md), its complete reference source, and all three content files. Review the root working copy, shared manifest/lockfile, and provider JSON.
- Use the pinned Rust `1.94.1`, `goose-agent`/`goose-providers` `0.1.0-alpha.11`, and RMCP `3.4.1`. The new chooser dependency is `crossterm = { version = "=0.29.0", default-features = false, features = ["events"] }`.
- Start or supply a live streaming/tool-capable provider. Do not quietly replace it with mocks if unavailable.
- Rehearse in a real terminal and with redirected input/output. Arrow selection needs terminal stdin/stdout and non-dumb `TERM`; `NO_COLOR` uses the numbered fallback to avoid cursor-control escapes as well as color. Verify terminal restoration before learner line input.
- Inspect `session-output.md` privately. It is overwritten at every launch and can contain personal conversation. Git ignore does not secure it. Do not project an unreviewed file or ask attendees for sensitive financial information.
- The root is the runnable working copy. Reference source shares the root manifest and the same root-relative content path. No independent lesson Cargo package, embedded content, or network content loading exists.
- Save unrelated working-copy edits before any restore command. Do not copy a finished root back into lesson references.

Provider construction/configuration occurs before selection. With no override, the provider JSON path is rooted at the manifest directory; an explicit relative override is current-directory-relative. Content and diagnostics are always manifest-root-relative. `.env` loading and first-configured-model selection are retained; full CLI/model-selection work remains later.

## Suggested classroom rhythm

### 1. Predict the welcome and select

Ask: “What makes this a learning partner rather than a one-shot answer generator?” Invite predictions about the first message. The expected teaching intention is a brief welcome asking experience and goals, not a full lecture; actual generated behavior must be reviewed.

Run `cargo run -- custom_aa_llama_qwen3_6-35b.json`. Show the first/only module **Getting started in day trading**, then the Quit entry. Select using arrows/Enter. Mention Esc/`q`; do not dwell on key-event APIs.

Explain before showing the constant that the opening is **application-owned input**, represented as a user-role message, not a learner quotation. The selection triggers the run automatically. Ask: “Who starts this exchange? Who waits for the next learner reply?” The application does both; `StateMachine::run` only handles the work inside each run.

### 2. Run a focused conversation

Use a non-sensitive goal such as “I am new to this and want to understand order types.” Follow with “Why might a limit order remain unfilled?” Observe whether the answer stays focused and refers appropriately to prior discussion. Invite an interruption to test conversational flexibility, not a rigid four-unit tour.

Explain that the four educational units are options. Discussed topics are not completed learning, and exposure to material is not mastery or readiness to trade. A short learner answer is limited evidence, not an authoritative readiness assessment.

Show `/finish`, then select the same module again. Predict a fresh welcome without the prior history. Explain that commands are intercepted by the application, not passed to the provider. `/quit` and EOF exit; empty replies are ignored. No completion or learning record is saved by `/finish`.

### 3. Explain the content boundary before Rust

Open the files in this order:

1. **Tutor contract:** educational role, respectful adaptation, honesty about sources and capability limits.
2. **Educational material:** definitions, course-authored examples, beginner priorities, source attribution.
3. **Teaching guidance:** choose a small relevant concept, respond to questions, adjust explanations, and invite participation.

Ask: “Which file changes what we teach? Which changes how we teach?” Then show `LessonContext::load` and `prompt_parts` only as focused evidence of application assembly. The Session records the selected context; the safe tool Operation contributes it before inference; the default preparer joins prompt-part text into the system instruction.

This is not skill discovery or RAG. The learner chooses one module; files load at selection. Links are supplemental attribution, not browsing instructions. Goose's tutorial extension is a reference pattern, not a dependency or something this application calls.

The material is course-authored, with attribution distinguishing sourced explanations from application-defined planned-loss arithmetic and simulation caution. Source review does not imply instructor release approval or that the model will cite correctly. No current quotes, broker-specific guidance, or regulatory threshold exercise is included.

### 4. Observe optional tool use without forcing it

First ask “What does stop-loss mean in day trading?” Predict a conceptual explanation without a calculator request. Then use the lesson's explicit hypothetical entry `51.20`, stop `50.70`, and `200` shares. Predict `$100.00` and exclusions: fees, slippage, gaps through the stop.

The capability remains available throughout. Observe actual names, arguments, IDs, results, and final explanation in diagnostics. Generated prose saying “I calculated” is not execution evidence. Wrong selection, fabricated arithmetic, or missing grounding is a failed scenario observation, not a reason to remove the tool or force the expected result.

“Maximum planned loss” is not statistical expected loss or a guarantee about realized loss. Prices are decimal strings, parsed with exact decimal arithmetic; never teach floating-point currency. Missing inputs should prompt clarification, not invention.

### 5. Tweak guidance, reload, compare

Replace only the default-pacing sentence in `getting-started-guidance.md` with:

> Use a short hypothetical example before asking a check-for-understanding question.

Ask learners to predict ordering, not exact model words. Use `/finish` and fresh selection, or restart, then repeat comparable goals/questions. Confirm the modified guidance appears in diagnostic context. Existing selections keep their original context; a file edit does not live-update that Session. Restart truncates diagnostics, so preserve comparison evidence privately first.

Discuss instruction influence versus guarantees. A single paired run is an observation, not a controlled evaluation of teaching effectiveness. Restore the default sentence afterward. No Rust modifications or deliberate failure-path exercises are assigned.

## Supplied support: explain its purpose, not its implementation as an exercise

| Boundary | Supplied code to navigate | Teaching point |
|---|---|---|
| Selection/context | `choose_lesson`, `arrow_chooser`, `LessonContext::load` | Selection starts a fresh Session and reloads all three files; missing/empty content fails |
| Learner input | `read_line`, `reply_action`, `run_learning_partner` | Application owns commands and reply waiting outside machine runs |
| Recorded state | `ChatSession`, `ChatRuntime`, `append_user`, Effect handler | Same selected conversation retains history/context; counters/halt state reset per reply |
| Inference input | `SafeTools::prompt_parts`, `inference_tools` | Application contributes selected context and the one tool definition |
| Execution | `LossToolProvider`, `SafeTools`, decimal validation | Schema advertisement does not replace strict runtime validation |
| Display/saving | `run_turn`, `display_events`, request presentation copy | Receive Events while the run executes; Effects separately save complete state |
| Diagnosis | `DiagnosticLog`, `session_fields`, `markdown_block` | Plain complete diagnostic record, not a provider wire dump or learner memory |

`tokio::join!` consumes Events while `StateMachine::run` executes. Closing the sole Emitter after success/error lets the receiver drain before post-run inspection. Request presentation follows saving through the same Event queue, keeping complete request notice → tool result → final answer order. The presentation-only marker is not persisted or included in provider input. The consumer does not replace saving, and the log records assembled responses rather than replaying streaming deltas.

The chooser confines raw mode to selection, restores it before provider calls/line input, and supplies a numbered fallback for redirected/dumb streams. The interface is intentionally limited: one module plus Quit, ordinary reply lines, `/finish`, `/quit`, EOF, and ignored empty input. This is not the later named-option/model-selector CLI.

## Safeguards and capability boundary

Retain Lesson 9's application choices, not supposed SDK guarantees:

- Eight applied work batches per turn; a recorded terminal halt batch can make the counter nine. Ordinary completion at the bound is allowed.
- Four aggregate requests per turn, including unknown/malformed requests. Every excess pending request receives a correlated error rather than execution; once answered, exceeding the bound halts rather than continuing inference.
- A 180-second timeout per machine run. Local cancellation/drop is not rollback or a guarantee the remote server stopped.

`RunLimit` precedes the safe tool adapter and inference. The GDK still owns re-evaluation. All current-turn request blocks are examined. The shipped tool operation filters unknown names and has no application request cap; the wrapper supplies correlated unknown/excess errors. Recognized malformed requests receive SDK/validator errors. Empty/duplicate IDs halt. The SDK's externally executed request exclusion is retained; this application supplies no external execution mechanism.

Validation rejects unknown fields, missing/wrongly typed arguments, nonpositive prices/shares, more than four decimal places, entry at/below stop, and arithmetic range overflow. Tool rejections can be nonfatal if a proper final answer follows; unmatched responses, missing ordinary final answers, inference diagnostics, run safeguards, and log failures are fatal. A prior timeout/pass bound can leave unanswered requests; inspect partial state honestly. Do not automatically resume a failed run.

The contract discourages personalized recommendations and unsupported claims. The executable capability set enforces the stronger boundary: no trading, brokerage credentials, shell, arbitrary networking, live market data, browsing, or retrieval tool. Provider transport itself is not a general model-callable network capability.

## Output and diagnostic review

Learner-facing output intentionally removes Lesson 9's raw Session snapshots, instruction blocks, and raw argument dumps. Preserve its actor labels and live-stream `++++++++` fences. The compact calculator activity notice is application commentary; results are framed with payload delimiters. These are not complete protocol dumps; diagnostics contain full details.

Use bold cyan headings/ordinary labels, blue learner prompt, magenta request notices, green results/status, yellow warnings, bold red stderr errors, and normal model text. stdout/stderr are gated independently; redirection, `NO_COLOR` (including empty), and `TERM=dumb` disable application styling. No gold Session rendering or dimming is needed because Sessions are not printed to the learner terminal.

`session-output.md` lives at the shared manifest root, is truncated once at launch, and appends across all selections/turns in that process. Review context/prompt parts, calculator definition, opening/learner inputs, complete before/after Sessions including the lesson field and all nested/default data, request names/arguments/IDs, correlated results/errors, assembled tutor text, tool Events, warnings, and failures. Prompt-part output is not an exact serialized provider request. Markdown blocks safely enclose text that itself includes fences; file presentation strips terminal controls without mutating saved state. The file is plain, without ANSI or deterministic gold rendering.

Writes are flushed so file errors surface promptly. Creation/write/flush failure stops the application; do not promise a file can record its own unavailable-storage error. Available partial state is logged before ordinary run/inspection failures. The file is ignored, not access-controlled, and is never reloaded into inference. No deliberate provider-config/credential dump exists, but user/model content can still disclose sensitive information.

## Validation plan and evidence discipline

Use the exact root commands and piped live inputs in the lesson. Also rehearse interactive arrow selection, fresh selection after `/finish`, plain fallback, empty replies/EOF, guidance reload, and private diagnostic review. Check full context, complete nested/default Session fields, lack of ANSI in file/plain captures, and fatal logging behavior through maintenance checks—not new classroom failure exercises.

Keep **structural success** separate from **qualitative observations**. Saved same-Session history, new IDs on selection, matching request/result IDs, exact `$100.00`, bounds, command interception, and actual loaded guidance are inspectable structure. Welcome quality, respectful adaptation, accurate exclusions, source honesty, educational focus, and example-before-check behavior need human review. Structural checks do not prove teaching quality, intended selective tool use, or byte-for-byte equality of live/saved prose.

If the provider is unavailable, ask for one and record the limitation. Tests can cover deterministic handling but cannot stand in for a live model interaction. Do not assert exact wording, token counts, or one universal number of messages/passes.

## Pinned-source evidence versus official orientation

Evidence map for the published **`goose-agent = 0.1.0-alpha.11`** source used by this assembly:

- `src/machine.rs`, `step`/`apply`/`run`: ordered first-applicable scan, operation-contributed inference tools/prompt parts, Effect application, reload/re-evaluation, and yield/no-work return. The engine supplies no application pass bound; our Operation and timeout supply bounds.
- `src/inference.rs`, default request preparation and `InferenceRunner`: joins prompt-part text into the system instruction, consumes provider streaming, emits display messages, and returns reconstructed messages/usage as Effects. The diagnostic context view must not be labeled an exact serialized request.
- `src/operation.rs`, `messages_since_kickoff`, `ends_turn`, and `Emitter`: current-turn slicing excludes tool-response user messages as kickoff, ordinary assistant completion controls no-work behavior, and Events flow through the Emitter separately from Effects.
- `src/tool.rs`, `ToolProvider`, pending-request filtering, and `ToolOperation::run`: provider definitions/handlers, scanning all blocks, excluding answered/external requests, filtering unknown names, sequential dispatch, and correlated user-role responses with request metadata. Application wrappers add unknown-name errors and request bounds.

Matching `goose-provider-types` `0.1.0-alpha.11` message/conversation/usage Debug representations and RMCP `3.4.1` tool request/result types establish the saved data shapes. Keep complete Debug fields rather than relying on serialization that may omit absent/default values. Do not assume a moving upstream checkout is interchangeable with the locked published crates.

The [Goose context-engineering guide](https://goose-docs.ai/docs/guides/context-engineering/index) and [provider setup guide](https://goose-docs.ai/docs/getting-started/providers) are supplemental **orientation**. They are not source evidence for the exact pinned Rust state-machine APIs and do not require Goose's full assembly, tutorial extension, discovery, or RAG.

## Validation record — 2026-10-07

- `cargo fmt --check`, `cargo check`, `cargo clippy --all-targets`, and `cargo test` passed: **20 deterministic tests**. The reference source also passed standalone rustfmt checking and is byte-identical to root. Tests cover existing decimal/domain/allowlist/correlation/all-block/request/pass safeguards, state continuity and fresh selection, fixed selected prompt parts, command classification, full Debug fields/default metadata, diagnostic truncation/append/embedded fences/control removal, failing log creation/write, framed ordered presentation and fallible output, and concurrent capacity-one Event draining without saving by display. `/dev/full` covers write failure on the Linux target. Tests are not teaching-quality or live model evidence.
- The lesson's documented live conversation, calculator, and blank/EOF pipelines passed against the bundled local **qwen3.6-35b-a3b** provider. The conversation asked about experience/goals, answered order-type questions, retained follow-up history, and returned to a fresh welcome after `/finish`/reselection. Empty input caused no extra run; EOF exited. A chooser `q` exited without inference. Default manifest-relative paths were also exercised from `/tmp`.
- The stop-loss conceptual turn made **no tool request**. The numeric turn made **one** `maximum_planned_loss` request with decimal-string `51.20`/`50.70`, `200` shares, and one matching saved result ID, returning **`$100.00`**, followed by an ordinary answer explaining fees, slippage, gaps, and lack of a guaranteed realized-loss ceiling. This was observed again in the final root live run, not forced by application logic. These outcomes do not validate Lesson 9's separate live 100-share tweak.
- A colored pseudo-terminal run exercised down/up/Enter, raw-mode restoration, continuing input, and the live calculator. Bold cyan/blue/magenta/green presentation, default-color tutor fragments, framed result, and request-activity → result → final-answer order were captured. An empty-`NO_COLOR` pseudo-terminal used numbered selection and added no ANSI. Redirected/plain runs and `TERM=dumb` blank/EOF flow also had no application-added ANSI. Stderr-only terminal error and dumb-terminal probes verified independent stderr styling/gating. Human projector/contrast review is still pending.
- Private diagnostic captures retained complete Session fields and nested/default message/usage metadata, selected content, correlated requests/results, assembled saved tutor text, multiple selections, and no ANSI. Git ignore was verified. A temporary unavailable-provider configuration exited with status 1 and retained complete available partial state plus failure diagnostics; this was a maintenance probe, not a new classroom exercise. A temporarily unavailable log destination caused exit 1 before learner UI/provider calls. Files were restored. The application cannot record to a log whose storage is unavailable.
- The approved guidance tweak was loaded into a fresh Session and confirmed in diagnostics, then the default sentence was restored. **Its intended teaching ordering was not clearly validated:** the model placed the hypothetical scenario inside its check question, rather than unambiguously presenting a short example first. This is mixed qualitative evidence, not a guaranteed prompt effect. Default conversations sometimes also used examples. Do not treat the paired observation as a controlled evaluation.
- Teaching review remains necessary. Some observed responses used reassurance such as “safety net,” described adverse fills as “slightly” worse, or asked whether planning made the learner more comfortable with trade parameters. These were accompanied by risk caveats, but their framing, precision, pacing, and suitability need instructor review; structural success does not certify educational accuracy or readiness to trade. Domain files are course-authored with verified FINRA/SEC attribution, not released instructional evidence merely because sources were consulted.

**Remaining gates:** instructor review of Lesson 10 content, teaching behavior, UI, and classroom tweak. Lessons 8–9 still require their own reviews; Lesson 9's live 100-share tweak remains pending. No Lesson 11 persistence design or implementation is supplied.

## Handoff to Lesson 11

The instructor's revised primary direction for Lesson 11 is learner persistence: an application-owned learning record plus learner-agreed to-do items. It replaces the earlier evaluation-baseline focus; detailed design is deferred. End with: “What should a learning partner deliberately remember, and what evidence would justify that record?”

Do not implement a profile, mastery tracker, to-do workflow, persistence format, or context-resumption policy here. Distinguish that future purposeful learner record from today's overwritten developer diagnostic log. Keep Lessons 8–9 review gates and Lesson 10's own pending review explicit.
