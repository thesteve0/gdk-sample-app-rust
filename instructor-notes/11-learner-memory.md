# Instructor notes — Lesson 11: Learner-approved memory

## Objective, authorization, and release gates

Teach the distinction between conversation continuity and intentional cross-launch memory. Learners should explain why selected notes need source inspection, consent, per-addition approval, correction, and separate categories. A learner-agreed next activity is not tutor-assigned homework or a mastery assessment.

The instructor authorized the Lesson 11 Markdown record and root continuation. **Implementation validation is recorded below; instructor review remains pending.** Do not infer success from Lesson 10's historical checks. Lessons 8–9 review and Lesson 10 teaching/content/UI review remain separate gates; Lesson 9's live 100-share tweak passed independently on 2026-10-22.

Supply the complete source and helpers from the outset. Use **predict → run → explain → tweak → run again**. The approved tweak edits an agreed activity by hand between launches, then observes a question about resuming it. No Rust fill-ins, new dependencies, memory-management commands, or deliberate failure-path classroom exercises are authorized.

## Preparation

Read the [lesson](../lessons/11-learner-memory/LESSON.md), complete reference, root working copy, three Lesson 11 content files, README/AGENTS, shared manifest/lockfile/toolchain, and provider JSON. Preserve unrelated edits. Reference and root use one manifest and the root-relative Lesson 11 content directory; there is no lesson-specific Cargo package.

Use pinned Rust `1.94.1`, GDK `0.1.0-alpha.11`, and RMCP `3.4.1`; existing crossterm selection support remains. No dependency additions are needed. Start or supply a live streaming/tool-capable provider; do not silently substitute mocks. Provider override/first-model selection and `.env` loading remain unchanged.

Rehearse with private, non-sensitive records. Do not project an unreviewed `learner-record.md` or `session-output.md`. Both can contain personal information. Git ignore is hygiene, not security. Warn that consented record text goes to the configured provider and that diagnostics continue even with memory off.

Inspect record permissions before promising privacy: new files request Linux 0600; existing permissions are unchanged; neither file is encrypted. Avoid concurrent processes or editors while the application runs. Preserve needed diagnostic evidence privately before restarting, which truncates it.

## Classroom sequence

### 1. Predict a memory-off conversation

Ask: “If I decline memory but a file exists, should the tutor read it?” Select the module and press Enter at the `[y/N]` memory prompt. Explain that selection consent is local and renewed at each selection. Only trimmed case-insensitive `y`/`yes` approves; other input and EOF do not.

Show that ordinary teaching and diagnostics still occur. Memory-off is not offline/private inference. Replies keep the Session; `/finish` returns to selection, `/quit`/reply EOF exit, and blank replies are ignored. Selection creates a fresh Session; no old transcript is reloaded.

### 2. Establish category distinctions before code

Use the five-category table in the lesson. Ask attendees to classify examples:

- “We discussed stop orders” is discussion, not demonstrated understanding.
- “I want to understand limit orders” is a learner-stated goal.
- A learner's specific explanation may be limited evidence; the tutor's own explanation is not.
- A lingering question is unresolved even if a tutor gave an unhelpful answer.
- “You could compare order types next time” is a suggestion; “I agree to try that” can support an agreed activity.

No record entry certifies mastery or readiness to trade. A source quote and learner approval improve inspectability but do not automatically establish semantic truth. Keep “maximum planned loss,” never “maximum expected loss.”

### 3. Run a proposal and inspect the source

Consent to memory and use the non-sensitive example in the lesson. Predict the final-answer-before-approval order. Do not promise the model will select the tool or infer a successful proposal from its prose.

Explain **record proposal**, **tool execution boundary**, and **correlation ID** before showing `RecordProposal` and the quotation snippet. `propose_learning_record` validates category/shape/text and returns a correlated acknowledgement. It does not write. Its successful saved request/response is the pending work inspected after the final answer; there is no durable background approval queue.

`parse_record_proposal` requires nonempty single-line note and quotation, each at most 600 bytes without controls, rejects unknown fields, and verifies an exact substring of a user reply. It excludes the first application-owned opening input and tool responses. It can quote an earlier reply in the same Session, not necessarily the latest reply. **Occurrence is not semantic validation**: a real quotation may accompany a false interpretation, stale goal, unsuitable category, or unagreed activity.

Ask: “Does this note say more than your words support?” Reject an unsuitable proposal. Approve only a genuinely agreed note. If no suitable proposal appears, report it as actual model behavior, not a structural success. Multiple proposals get separate prompts and share the tool budget; do not use a blind piped sequence of `y` inputs that might become learner messages instead.

### 4. Explain who can write and when

`run_turn` drains Events concurrently with `StateMachine::run`, records available state, and inspects final saved conversation. Only a successful run with a valid ordinary final answer proceeds to `approve_record_proposals`. Successful current-turn proposal responses are matched by ID; failed/external proposals are not approved through this path.

The application displays category, note, and quotation, then asks `Save this note? [y/N] >`. Neither model text nor the tool acknowledgement is approval. The decision is logged before the approved append. Rejection does not append. The write happens outside the engine after the answer; it is not an added manual agent loop or an Effect pretending to be consent.

`append_approved_note` rereads, builds an escaped Markdown addition, checks the total size, appends, synchronizes, and rereads. It preserves existing bytes and updates the current Session's record context for later turns. Repeated category headings are deliberate chronological blocks. There is no automatic consolidation, completion detection, correction, or deletion tool.

### 5. Tweak between launches and observe resumption

Quit, inspect a genuinely agreed activity, and change it by hand to a new activity the learner now agrees to. Correct/remove obsolete source support rather than implying the earlier learner said the edited words. Relaunch, select, consent, and predict a question about resuming.

Confirm the edited record in diagnostic context, then assess the actual welcome. Asking about resume is qualitative model behavior, not guaranteed by loading a file. Assuming continuation or claiming mastery is a review issue. The record is resumption **data**, not instruction authority or a transcript. A fresh Session still has a new application-owned opening input.

Deleting/correcting the file affects future loads; it does not erase existing diagnostics or provider-side copies. Do not edit during a running process or treat the file as live reloaded context. There are no `/memory`, `/forget`, or activity-management commands to teach.

## Source navigation after modularization

Activate all eight `.rs` files using the lesson's complete-source command. Root `src/` is ephemeral: no backup/restoration is required and reference sources are never overwritten from root. Explain local modules before showing the `mod` declarations; all Rust sources begin with `// Lesson 11`. No shared course crate or new runtime abstraction was introduced.

Start with the content, then stay in `main.rs` for the authority story: selection consent → context loading/contribution → proposal-only handler → actual learner-quotation validation → successful run/final-answer inspection → per-addition default-no approval → decision log → append → current-Session refresh. Consent to use memory is not approval of an addition. Blank or unrecognized approval input means no. Model text cannot answer the local save prompt.

`learning_record.rs` supplies record types/schema, fixed-path bounded storage/escaping and confirmation-input mechanics. It does not decide whether to save. `parse_record_proposal` and `approve_record_proposals` remain main functions; schema/record fields use narrow parent visibility rather than getters or blanket public APIs. Context assembly also remains main by the instructor's current decision.

Follow the concurrent run/Event consumer and runtime/Effects in main before inspecting support modules. `presentation.rs` owns `TerminalProse`, not Event consumption; `chooser.rs` owns terminal input/raw-mode mechanics; `diagnostics.rs` owns complete plain rendering/logging; `provider_setup.rs` owns path/first-model helpers, not construction; `calculator.rs` holds the previously taught calculator. Application tests live in `tests.rs`; private storage, log, styling and calculator tests stay with their owners. No original test was removed.

## Supplied helper map

| Supplied boundary | Explain why, not Rust scaffolding |
|---|---|
| `LessonContext`, `prompt_parts` | Load teaching context at selection; add record only with consent; mark record as untrusted notes, not mastery/instructions |
| `confirmation`, `approval_value` | Two distinct default-no decisions owned by the learner |
| `read_record`, `record_path` | Fixed manifest-root path; missing means empty; regular UTF-8 file, no symlink, 32 KiB cap |
| `RecordCategory`, `RecordProposal`, `parse_record_proposal` | Separate categories, strict untrusted arguments, text bounds, exact source occurrence |
| `record_tool_definition`, `LossToolProvider`, `SafeTools` | Consent-gated preparation capability, no path/write argument, correlated results/errors |
| `approve_record_proposals` | Post-successful-answer approval, decision logging, approved Session-context refresh |
| `markdown_note`, `append_approved_note` | Escape only new payload, preserve old bytes, repeated headings, bounded append |
| `ChatRuntime`, effect handler, `display_events_to` | Saving and display remain separate; Events are drained during the run |
| `TerminalProse` | Streaming common-markup suppression, presentation only, not general rendering |
| `DiagnosticLog`, `session_fields`, `markdown_block` | Complete plain diagnostics and available partial state, not future learner memory |

The record read has pre/post size checks; append enforces 32,768 bytes including the header, formatting, and escaped payload. New files request 0600 on Linux; `.mode` does not alter existing permissions. `sync_all` does not make this a transactional database. This design assumes one process: no concurrent-writer locking, crash recovery, rollback, or exactly-once guarantee. A write can succeed before a later failure; restart is not permission to replay approval automatically.

## Unchanged safeguards and new shared budget

Keep the GDK-owned run loop, RunLimit → safe tools → inference order, eight applied work passes plus a possible terminal halt batch, four aggregate requests per turn, and 180-second run timeout. Calculator and proposal requests share the four-request limit, including unknown/malformed calls. Do not describe these as SDK defaults.

The adapter still scans all current-turn request blocks. Unknown/excess names get correlated errors; empty/duplicate IDs halt; answered and externally executed requests retain the pinned operation's exclusion. A memory-disabled proposal name is not authorized. Strict schema/runtime validation remains; no arbitrary paths or file actions are accepted.

Calculator validation retains exact decimal strings, at most four fractional places, positive shares/prices, entry above stop, and checked arithmetic. Optional `51.20`/`50.70`/`200` should yield `$100.00` with exclusions. No brokerage, shell, retrieval, browsing, or market-data tools are added.

Tool rejections can be nonfatal if a valid final answer follows. Run/inspection/logging/storage failures stop; no save prompt follows a failed run. Available partial state is diagnostic evidence, not a fabricated completion. Cancellation does not guarantee remote work stopped or undo an append.

## UI and diagnostics: explicit Lesson 11 exception

Root continuation now uses clean application labels and compact activity/results. Do not copy Lesson 10's `++++++++`, protocol actor arrows, or request IDs into this learner UI. `Tutor:`, `Learner >`, memory/save prompts, and category/source display suffice. The supplied `TerminalProse` helper suppresses common Markdown markers across streaming deltas without changing saved/provider text. It is not a general renderer; unsupported formatting may remain. Normal course color roles and independent stdout/stderr terminal gating remain; no application-added ANSI in plain/NO_COLOR/dumb output.

Diagnostics stay separate: `session-output.md` truncates at launch, appends within it, records complete before/after Session Debug values and nested/default fields, selected context/record, correlated tools, assembled saved tutor text, decisions, warnings/failures, and post-approval state. Markdown fences remain in this developer file and safely enclose embedded fences; no Session syntax-color promise or ANSI is made. Context output is not an exact serialized provider request. The log is never loaded as learner memory.

Logging failure is fatal, and a missing/unwritable log cannot promise to record its own error. No provider config/credential dump is deliberate, but arbitrary learner/model text may contain sensitive information. The approved record is not secured merely because it is ignored.

## Validation plan and pending evidence

Run the lesson's root format/check/Clippy/test commands and documented live run. Inspect deterministic consent gating, strict proposal parsing/source checks, all-block/unknown/correlation/request/pass safeguards, preservation/escaping/cap/permissions, and failure handling. Tests cannot certify agreement, category accuracy, mastery, teaching quality, or actual model tool selection.

Live rehearsal should cover memory off with an existing record, memory on with missing/existing record, proposal acknowledgement without writes, default-no/rejection/approval, successful final-answer ordering, multiple proposals sharing the budget, fresh selection/relaunch, optional calculator, arrow and plain/NO_COLOR/dumb modes, command/EOF behavior, complete diagnostics, and the approved hand-edit/resume question. Maintenance failure probes are not learner exercises. Record observed structure and qualitative concerns separately; never claim exact generated wording is guaranteed.

### Validation record — 2026-10-11

- `cargo fmt --check`, `cargo check`, `cargo clippy --all-targets`, and `cargo test` passed: **26 deterministic tests**. Root and supplied reference are byte-identical and reference rustfmt passed. No dependency, lockfile, provider JSON, or toolchain changes. Tests cover prior safeguards plus default-no decisions, consent-gated tool advertisement, proposal acknowledgement without writes, exact learner-source checks (excluding tutor/tool/opening), strict categories/fields/control/byte bounds, unknown names alongside calculator/proposals, preserved hand edits, escaped Markdown, 32 KiB bounds, UTF-8/regular-file/symlink rejection, new 0600/existing unchanged permissions, and streaming common-markup suppression. Tests do not establish semantic accuracy, agreement, or teaching quality.
- Against the bundled local `qwen3.6-35b-a3b` provider, a learner-agreed next activity produced one correlated proposal, an acknowledgement without saving, an ordinary final tutor answer, then local approval. The approved record was created with 0600 permissions. A manual edit replaced the activity; a fresh launch loaded it and asked whether to resume or change focus. This validates the documented tweak structurally, not exact welcome wording.
- Declining memory with that existing record advertised no proposal tool and left the record unchanged. An optional numeric turn made one calculator call with `51.20`/`50.70`/`200`, returned the correlated `$100.00`, and gave exclusions. No model selection was forced. A successful proposal-rejection retry displayed the note after the final answer and left existing bytes unchanged.
- **Initial rejection attempt failed honestly:** after a successful proposal acknowledgement the provider follow-up exceeded the 180-second bound. No approval prompt or write occurred; complete available partial Session and tool Events plus timeout diagnostics survived. A separate retry passed. This is reliability evidence needing review, not a hidden success or a new learner failure exercise.
- Redirected/plain and empty-`NO_COLOR` live output contained no application ANSI or protocol fences/actor arrows/IDs. Model bold markers in an initial calculator answer prompted the supplied `TerminalProse` helper; a later live calculator capture had plain prose. The helper only suppresses common markers and is not a complete renderer. `TERM=dumb` was used in the timed-out attempt; a complete successful colored arrow-key Lesson 11 scenario was **not revalidated**. Existing chooser/color deterministic checks pass; human UI/contrast review remains pending.
- Diagnostic captures retained selected record/context, complete nested/default Session Debug values, correlated requests/results, assembled answers, post-approval state, and failures without ANSI. Git ignore covers both root record and diagnostics. Proposal definition logging was checked in the final live run. Test record was removed; private diagnostic captures stayed outside tracked files.
- Content/teaching/UI and instructor review remain pending. One proposed activity broadened the learner wording, and generated explanations deserve educational review even when structural checks pass. Earlier Lessons 8–10 review and Lesson 9's live 100-share tweak remain independent gates. No Lesson 12 was scaffolded.

## Source evidence and orientation

The parent implementation review verified the exact published `goose-agent = 0.1.0-alpha.11` source. Its evidence map remains:

- `src/machine.rs`: ordered first-applicable steps, application of Effects, reload/re-evaluation, and run return behavior; application-owned limits remain separate.
- `src/inference.rs`: prompt-part preparation, streaming Events, reconstructed messages/usage as Effects.
- `src/operation.rs`: current-turn slicing, ordinary completion, and Emitter separation from saving.
- `src/tool.rs`: provider definitions/handlers, all-block pending filtering, sequential dispatch, correlated responses, unknown-name filtering with no application request cap.

This is pinned-source evidence supplied by the implementation owner, not a new independent SDK verification by this documentation task. Application code establishes approval, append, consent, and file behavior; the SDK does not provide those policies. Matching locked provider types/RMCP establish message and tool shapes; retain complete Debug representations rather than serializers that omit defaults.

The [official context-engineering guide](https://goose-docs.ai/docs/guides/context-engineering/index) is orientation only. It does not establish the Rust interfaces, persistence guarantees, or a requirement to reproduce Goose's full application assembly. No later lesson implementation is authorized by this handoff.

## Teaching-material completion and deferred review — 2026-10-11

The instructor requested implementation of the content and teaching material and will review once the full course has been implemented. **Review is deferred, not a stop-work gate for separately authorized lessons and not release approval.** Continue recording validation limits and qualitative concerns. This request does not itself authorize another lesson or expand this module beyond introductory day trading.

### Short teaching script

- **Motivate:** “History lets this process continue a conversation. A learning record lets a new process remember a few things the learner deliberately chose.” Point out that keeping everything is not the goal.
- **Classify:** use the lesson's five authored examples. Ask attendees to identify a suitable category and an unjustified inference. Contrast “discussed,” “learner explained,” and “mastered.”
- **Predict:** narrate the lifecycle through the proposal acknowledgement, then pause. Ask whether the file has changed. Repeat before local approval. Emphasize that the model cannot grant permission.
- **Run:** use a non-sensitive goal and a genuinely agreed small next activity. Let the actual model outcome determine whether a proposal appears; do not hide a missing or unsuitable proposal. Inspect note and quotation before approval.
- **Explain:** compare the live learner UI, approved Markdown, and private diagnostics. Only show diagnostics prepared for projection. Identify presentation Events, recorded messages, and the separate approved append without assigning infrastructure code.
- **Tweak:** exit, hand-edit the agreed activity and obsolete supporting text, relaunch, consent, and observe whether the tutor asks to resume. Show the fresh history and loaded edited context privately if needed.
- **Close:** use the exit questions in the lesson. Ask attendees which boundary is enforced by code and which outcome still depends on model behavior and human judgment.

### Feedback on learner answers

For “a limit order guarantees a fill,” say that the price constraint and execution outcome are separate, then ask for the distinction again. For a correct short answer, acknowledge that specific distinction without calling it mastery. For “a stop caps my actual loss,” connect the planned calculator result to its fees/slippage/gap exclusions. A proposed learning note should describe the learner's own contribution, not the instructor's correction.

Teach the optional next-activity list as a set of learner choices, not a completion checklist. Saving a goal does not make it a requirement; an old activity can be changed by hand or declined at resumption. Do not imply persistent notes make generated explanations accurate.

### End-of-course review queue

Review category accuracy and faithful wording, genuine activity agreement, no mastery inference, beginner-appropriate explanations, risk language, resumption choices, privacy clarity, terminal readability, and recovery from the observed provider timeout. Keep historical colored-UI limitations and earlier lesson live/review limitations visible; distinguish any fresh evidence in the dated modularization record below. The new authored examples and expanded runtime guidance have not yet undergone instructor review or a fresh live teaching evaluation; historical implementation checks are not evidence for their educational quality.

## Modularization validation — 2026-10-22

- Instructor approved the eight-file local split after clarifying that default-no means a blank/unrecognized response or EOF does not approve storage. `main.rs` retains context loading/prompt assembly, selection consent, proposal-only handler, `parse_record_proposal` (including actual-learner-source checks), successful-run inspection, `approve_record_proposals`, decision logging, append call and Session-context refresh. It also retains runtime/Effects, concurrent Event consumption, dispatch, limits, machine assembly/run/shutdown and partial-state handling. `learning_record.rs` supplies types/schema/storage/confirmation mechanics, not permission to save. `TerminalProse` moved to presentation, not the Event consumer.
- All **105 original function/test bodies** match after whitespace/optional-trailing-comma normalization; all **26 original test names** and all constant values are retained. Coverage is 18 application tests and two each in calculator, diagnostics, learning-record and presentation modules. Private filesystem fixtures remain with their owners without exposing diagnostic file fields or adding getters. `cargo fmt --check`, `cargo check`, `cargo clippy --all-targets`, `cargo test` and `cargo build` passed without warnings after removing unused test imports. Reference/root match and every Rust source begins with `// Lesson 11`. No dependency, lock/toolchain/provider, prompt or content change; no shared course library, new trait or framework. Lessons 12–14, preparation and dataset seeder are untouched.
- Fresh live colored down/up/Enter selection reached proposal activity → correlated acknowledgement saying NOT saved → ordinary saved/displayed final answer → local approval. Before each first decision the record was absent. **The first proposed activity was rejected** because it added “check for understanding regarding orders and execution” beyond the learner's accepted wording; no file was created. Its tutor answer also said “I've noted that for our future sessions” before approval, a misleading generated claim despite the unchanged storage boundary. A separate retry proposed the narrower activity “Next time I will explain why a limit order might remain unfilled.” with the actual learner quotation; it was inspected and approved. One correlated request/result, post-approval Session context and a new 0600 record were verified. These are separate attempts, not a silently replaced failure.
- Between launches the test record was hand-edited to comparing market and limit orders in a hypothetical example, with obsolete quotation support removed. A fresh consented `TERM=dumb` launch loaded the edited text in diagnostics **but its welcome ignored the activity and repeated experience/goals questions**. A subsequent empty-`NO_COLOR` launch loaded the same record and explicitly offered resumption or a different focus. Resumption behavior is therefore mixed, not guaranteed. That second launch produced a suitable slippage-goal proposal; it was explicitly rejected after the ordinary final answer and existing record bytes remained unchanged by SHA-256.
- A fresh colored arrow-key launch declined memory by blank input while the record existed: diagnostics contained neither the record marker nor an advertised proposal definition, and the record bytes stayed unchanged. The numerical turn made one request with decimal-string `51.20`/`50.70` and 200 shares, one matching `$100.00` result and an ordinary final answer explaining fees/slippage/gaps and no guaranteed realized-loss ceiling. `/finish` returned to the chooser, `q` exited, raw mode restored before line input, and cyan/blue/magenta/green roles plus activity → result → answer order were checked.
- Redirected/plain continuing selection, blank replies, `/finish`/fresh selection, `/quit`, blank/EOF, empty-`NO_COLOR` and dumb-terminal modes passed. There was no application ANSI in plain/NO_COLOR/dumb captures; clean dialogue omitted protocol fences/arrows/IDs. Independent stdout/stderr error gating passed. Running from `/tmp` without a provider override confirmed manifest-root paths. Complete plain before/after and post-approval diagnostics retained nested/default Session/usage/message data, selected context, correlated tools and approval decisions. The original deterministic `/dev/full`, permissions, symlink/UTF-8/size and presentation-failure tests passed. Test record was removed afterward; captures remain private outside tracked files.
- No fresh unavailable-provider/log-destination failure probe, IDE build, projector review, service restart, prompt correction or release approval is claimed. Historical timeout/partial-state evidence remains separate. Actual agreement/category/wording and teaching/resumption quality still require instructor review; preserved bodies and passing tests do not certify them. Lesson 9's live 100-share tweak passed separately on 2026-10-22; earlier reviews remain independent. Stop at the Lesson 11 review checkpoint and ask questions before Lesson 12.
