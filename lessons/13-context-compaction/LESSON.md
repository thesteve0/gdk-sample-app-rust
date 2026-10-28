# Lesson 13 — Continue learning with bounded working context

## Goal and supplied implementation

Continue a learning conversation without sending an ever-growing transcript to the tutor. Explain what may be shortened, what must remain authoritative, and why a generated summary cannot become learner evidence or permission to write memory.

Use the complete supplied Lesson 13 reference and root working copy, including the compaction, retrieval, concurrency and presentation helpers. **No missing Rust scaffolding is assigned.** The classroom rhythm is predict → run → explain → tweak → run again. Day trading remains introductory education, not a trading recommendation or competence certification.

**Implementation supplied; instructor review pending.** The complete source and policy are supplied; the validation record below distinguishes checks from remaining gaps. Historical planning-only wording in [PLAN.md](PLAN.md) and [the handoff](../final-lessons-handoff.md) precedes this task, not a claim that validation has occurred. This preserved Lesson 13 reference does not include Lesson 14 instrumentation; [Lesson 14](../14-agent-evaluation/LESSON.md) is separately authorized and implemented. The combined modularization pass finishes with root activated from Lesson 14, not this reference.

## Concepts introduced

A **local learning declaration** is the learner telling the application that they consider a named concept learned. Here `/learned <concept>` records that declaration when memory is enabled. It is **not** evidence of mastery, a test result, completion of a required progression, or readiness to trade. The command is handled locally, not interpreted as a model tool request.

Keep **three state domains** distinct:

| Domain | Purpose and authority |
| --- | --- |
| Authoritative application state | Actual message identities/provenance, tool correlation, approval decisions, revisions and limits. A generated summary cannot change these facts. |
| Learner-authorized persistent record | Selected notes in `learner-record.md`, used only with per-selection consent. Ordinary model proposals still need separate local approval. |
| Model working context | Instructions, consented record context, tools and conversation information supplied for inference. It can be shortened without silently changing the other domains. |

A **context window** is the bounded input a model can consider in one inference. **Working context** is the information our application supplies for that inference. **Lossy compaction** replaces selected older conversation material with a shorter generated summary: details can be omitted or distorted. It is neither an exact archive nor a guarantee of true claims.

**Genuine learner provenance** means a statement came from an actual learner reply, with its original identity and role. A summary quoting that reply is derived text, not a second genuine reply. Book excerpts, tool output, application opening messages and local commands are not eligible learner quotations.

A **safe boundary** is a point where an older complete exchange can be replaced without splitting pending tool work or disrupting current-turn inspection and local approvals. An **exchange** includes a genuine learner reply and the tutor's completed response, including any intervening correlated tool requests/results. Requests and responses must remain together.

An **Operation** is a GDK decision unit that examines state and proposes work. An **Effect** describes a change that the application runtime applies. The compaction Operation prepares a replacement; its replacement Effect changes working history only after validation. An **Event** is live presentation activity, not that saved change. GDK still owns re-evaluation through `StateMachine::run`; this application supplies the policy, not full Goose's automatic compaction assembly.

A **revision** identifies the state version used to prepare a replacement. Checking it before applying prevents a stale summary from overwriting intervening changes. A **budget** limits resources the application will use; this lesson uses a conservative byte estimate, not a verified model-token capacity.

## Prerequisites and run

Complete Lesson 12's retrieval, citation, calculator and consent/approval boundaries. Keep its prepared local tutor, matching Harrier/llama.cpp service and read-only PostgreSQL connection. Follow [Lesson 12 DATABASE-SETUP.md](../12-specialized-retrieval/DATABASE-SETUP.md); no database reload, model conversion or new corpus preparation is a learner exercise.

Use the supplied local service defaults or the existing ignored `.env`/`dotenvy` overrides. Do not print credentials or enable shell tracing. Root `src/` is an ephemeral exercise copy, not an authoritative reference. From the repository root, activate the **entire recursive source tree**, including child test directories:

```bash
rm -rf src
mkdir src
cp -R lessons/13-context-compaction/src/. src/
```

This intentionally replaces the exercise copy. No backup/restore or final hello-world reset is required. Preserve unrelated work outside `src/`; the combined Lessons 12–14 pass ultimately leaves root matching Lesson 14. To run this activated Lesson 13 copy:

```bash
umask 077
cargo run -- custom_aa_llama_qwen3_6-35b.json
```

The three local classroom defaults are `const` values at the top of `src/main.rs`: `RETRIEVAL_DATABASE_URL` uses the instructor-supplied `retrieval_reader` connection to `127.0.0.1:55432/harrier`, `HARRIER_URL` is `http://127.0.0.1:9999`, and `TUTOR_URL` is `http://127.0.0.1:8080`. No exports or private connection file are required for these defaults. Existing environment variables (including values loaded by `dotenvy` from `.env`) override them. For another prepared database, set `RETRIEVAL_DATABASE_URL` to its reader connection, never the loader/admin connection. The supplied password is **plaintext in source and compiled binaries**; this is an explicit local classroom convenience, not secret storage or a production credential pattern. Rotate/replace it for any other deployment and do not project the credential-bearing source. Defaults do not start services or bypass runtime/profile/reader validation. `TUTOR_URL` selects the tokenizer origin and must match the tutor provider JSON; it does not override generation configuration.


The tutor origin must match the provider JSON and its tokenizer service. Lesson 12's literal-loopback, proxy/redirect restrictions, compatible collection/profile and reader checks remain. Client loopback URLs do not secure an existing server bound to all interfaces. If services or authorized private books are unavailable, ask the instructor; mocks are not a silent substitute for a live run.

Maintenance checks, not classroom fill-ins:

```bash
cargo fmt --check
cargo check
cargo clippy --all-targets
cargo test
cargo build
```

The lesson sources share the root manifest; they are not a separate Cargo package. Copying only `main.rs` or only top-level `.rs` files is incomplete: the child test directories belong to the reference too. Use this Lesson 13 reference, not an earlier or later lesson's source, when repeating it.

## Supplied module map and teaching reading path

A **Rust module** groups related code under a local name. Here modules keep previously taught mechanics and supplied support out of the main teaching path; they do not create a shared course library or a new application framework. A declaration such as `mod calculator;` loads the sibling `calculator.rs`. The first support declarations in [main.rs](src/main.rs) are:

```rust
mod calculator;
mod chooser;
mod diagnostics;
mod learning_record;
mod presentation;
mod provider_setup;
```

The same main file also declares the cohesive `compaction` and `retrieval` modules and a test-only `tests` module. A **child test module** keeps tests beside the production module they exercise without splitting that module's algorithm. For example, `compaction.rs` ends with `#[cfg(test)] mod tests;`, which loads `compaction/tests.rs` only for tests. All supplied Rust files start with `// Lesson 13`.

| Complete source map | Responsibility |
| --- | --- |
| [main.rs](src/main.rs) | Context loading/prompt assembly; selection and commands; Session, authoritative state, provenance and revisions; preflight/maintenance call sites; runtime/Effects; concurrent Events; dispatch/allowlists/limits; evidence budget, tool registration and current-turn citation inspection; successful-run inspection and local approval order. |
| [compaction.rs](src/compaction.rs) | Cohesive declaration, ordinary-content projection, partition/budget policy, isolated summary run, revision-checked replacement and maintenance implementation. |
| [compaction/tests.rs](src/compaction/tests.rs) | Child tests for compaction, declarations, provenance, budgets, revision and failure boundaries. |
| [retrieval.rs](src/retrieval.rs) | Cohesive local read-only retrieval, transport/profile/provenance validation, token accounting and cancellation/Drop guards. |
| [retrieval/tests.rs](src/retrieval/tests.rs) | Child retrieval tests, including the explicitly opt-in actual-service test. |
| [calculator.rs](src/calculator.rs) | Inherited exact-decimal calculator/schema/validation and local tests. |
| [chooser.rs](src/chooser.rs) | Inherited arrow/numbered chooser, line input and raw-mode cleanup. |
| [diagnostics.rs](src/diagnostics.rs) | Inherited plain Markdown diagnostics, complete Session formatting and local tests; never learner memory. |
| [learning_record.rs](src/learning_record.rs) | Inherited record types/schema, bounded read/append and confirmation-input mechanics, with local tests; no model write authority. |
| [presentation.rs](src/presentation.rs) | Inherited terminal styles/gates and `TerminalProse`, with local tests; not Event consumption or saved-text rewriting. |
| [provider_setup.rs](src/provider_setup.rs) | Inherited provider path and first-configured-model selection. |
| [tests.rs](src/tests.rs) | Main's test-only application/runtime/dispatch/approval/Event/citation tests, including the opt-in same-snapshot live search test. |

**Teaching reading path:**

1. Begin in `main.rs`: `LessonContext::load`, `run_learning_partner` and `reply_action`. Trace consent, fresh selection, `/learned`, declaration-before-maintenance and automatic preflight-before-genuine-reply append. Context assembly stays here, not in support modules.
2. Read `ChatSession`, `ChatRuntime`, its append/policy methods and Effect handler. Identify genuine reply IDs, settled approvals, revisions and application authority before visiting compaction internals.
3. Follow the `compaction::BudgetPreparer`/maintenance wiring from main into `compaction.rs`: projection → partition → isolated summary → validation → `apply_outcome` revision check. Return to main's `ChatEffect::Maintenance` application boundary. Compaction stays cohesive, not scattered into generic helpers.
4. Follow `run_turn`, `cooperative_run` and `display_events_bounded`: GDK run and Event consumption execute concurrently; sender closure, draining, saved partial-state diagnostics, successful run and `inspect_turn` precede approval. Event consumption and cancellation ordering remain visible in main, not `presentation.rs`.
5. Read `EvidenceBudget`, `RetrievalToolProvider`, `LossToolProvider`, `SafeTools`, `RunLimit` and `inspect_citations` in main. Visit `retrieval.rs` only for its supplied cohesive implementation; visit `calculator.rs` to recall the already-taught decimal tool. Current-turn citation/evidence decisions and tool registration remain in main.
6. Finish in main with `parse_record_proposal`, `approval_pending` and `approve_record_proposals`: retained genuine-learner-source validation → independent default-no decision after successful inspection → decision log → approved append call → Session record refresh/revision → settled decision/revision → complete post-approval diagnostics. Storage/confirmation mechanics are support, not permission. Read the three separate test files and module-local inline tests as boundary evidence, not class fill-ins.

The six inherited support modules are supplied unchanged in behavior. Modularization changes reading locations and visibility/import boundaries, not prompts, policy, limits, authority, privacy or runtime behavior.

## 13.1 — Predict the three domains

Select **Getting started in day trading** and make the default-no memory decision yourself. Before running, predict:

- Which facts are genuine learner statements, which are tutor explanations, and which are tool results?
- Which may become summary data? Which may authorize a persistent write?
- Should shortening history restart the lesson or produce a new welcome? **No: it retains the same Session.**

Use [the rendered before/after visual](visuals/compaction-boundary.png), [its editable SVG](visuals/compaction-boundary.svg) and [its text equivalent](visuals/compaction-boundary.md). It is a conceptual trace, not captured model output.

Begin a short conversation about planned stops and execution uncertainty. State a small goal, give your own explanation, and ask a clarification. Do not script a claim of understanding that you do not hold. Observe normal Tutor/Learner dialogue and brief activity notices, not raw Session dumps.

## 13.2 — Run a local declaration, not a mastery assessment

After discussing a concept, decide whether you want to make a local declaration. Example command syntax:

```text
/learned stop price versus execution price
```

This narrowly authorizes saving **that declaration** only when this selection has memory consent. It is not blanket approval of other notes; it does not replace default-no approval for ordinary model proposals. The application creates a deterministic completion/declaration category that the model cannot propose. It must not rename the declaration as assessed mastery.

With memory consent disabled, the declaration is ephemeral in the current Session only: no record is read or written. A concept must be a valid nonblank single line of at most **200 UTF-8 bytes**, not 200 characters, using plain words and simple punctuation without control characters. Invalid input is rejected locally. Repeating the same declaration in the same Session does not save it again; that is not an exactly-once guarantee across launches.

Expect a short local status and the next learner prompt, **not a new Session, welcome, unsolicited tutor answer or model inference merely to interpret the command**. Check the record privately only if you consented. A successful declaration stays saved even if later summary generation fails; compaction failure is not rollback of an already authorized note.

Ordinary note proposals remain separate: strict actual-learner quotation, successful run/inspection, then a local default-no decision. Both accepted and declined decisions are settled so history replacement cannot replay an old approval prompt. Exact quotation occurrence still does not certify faithful interpretation or agreement.

## 13.3 — Explain the replacement boundary

The application retains the original opening and the **latest two complete genuine learner exchanges**, including their tool pairs, original message IDs and metadata. It summarizes only eligible older history. At later compactions, the previous summary and newly eligible history produce **one replacement summary**, not an accumulating stack of summaries.

The summary is assistant-derived, fallible **data**, not a genuine learner message or higher-priority prompt instruction. Common instructions, teaching guidance and consented record context stay separate. Summary generation uses a **tool-free isolated inference coordinated through the GDK**: it cannot search, calculate, propose notes or authorize writes. Its generated text is checked before the replacement Effect is applied.

The summarizer now receives an **ordinary-content projection**: a temporary input view with original message IDs, speaker roles and explicit provenance labels. Genuine learner replies are identified using application-owned reply IDs, not user role alone. Tutor text is labeled as explanation, prior summaries as fallible derived data, and tool requests/results as correlated historical data. Provider reasoning, redacted reasoning and internal message metadata are excluded from this view; originals remain unchanged in stored history and diagnostics. The projected request plus source-boundary instructions and editable guidance is checked against the same input allowance.

Summary guidance requires unresolved uncertainty to remain last expressed uncertainty unless a later genuine learner explicitly confirms resolution or withdraws it. Tutor answers alone cannot close it. Accepted activities preserve the learner's exact accepted scope; tutor reply length and hypothetical example prices cannot become activity requirements. These labels and checks enforce input provenance, not the semantic truth of every generated sentence.

The application sets a **pending maintenance request**: recorded intent to do context maintenance instead of tutor work. This separate run contains only the compaction Operation, so completion yields to the learner prompt without an unsolicited teaching response. A **replacement candidate** is prepared history that has not yet changed the store; the runtime applies it only at the expected revision. These are supplied application types, not new SDK APIs. The focused effect boundary is:

```rust
Maintenance(compaction::Outcome),
```

The outcome carries the expected revision, optional replacement conversation and an honest report. A no-op or failed attempt has no replacement; the report and available usage are still recorded. Locate `apply_outcome` in the supplied compaction module to see the revision check before conversation replacement. The application-owned completion/approval ledgers and cumulative usage remain outside generated summary authority.

The SDK exposes conversation replacement, but this custom runtime must explicitly implement its own safe replacement behavior. After explaining Effect, inspect this focused enum variant from pinned `goose-agent = 0.1.0-alpha.11`:

```rust
ReplaceConversation(Conversation),
```

This is an SDK capability, not a complete compaction policy or an instruction to insert the variant into an arbitrary application enum. In the supplied source, locate the compaction Operation, isolated inference, revision check and runtime replacement together. Narrate: choose eligible history → prepare bounded summary → validate replacement and revision → apply Effect → GDK reloads/re-evaluates. Supplied async machinery is infrastructure to explain, not reconstruct in class.

If preparation, validation or revision checking fails, the original working history remains intact. A successful replacement must reduce the measured working input; shape/size checks cannot prove that its meaning is faithful. Pending work is not split or discarded to make it fit.

**Quotation boundary:** a model proposal may quote only retained real learner replies. It may not quote the summary, a tool response, the opening or a command. Once an old real reply is no longer retained, its summary quotation does not restore eligibility. This bounded provenance rule trades older quotation availability for safe shortening.

**Citation boundary:** historical passage IDs in a summary are reminders of past evidence, not current citation support. If a later answer needs book evidence, search again during that learner turn. A valid old ID cannot substitute for the actual returned excerpt, its clipping and source qualifications.

## 13.4 — Observe the conservative budget

The supplied application policy is:

| Boundary | Supplied policy |
| --- | --- |
| Conservative total byte-estimate allowance | 100,000 bytes, including 16,000 reserved headroom (84,000 estimated-input allowance) |
| Default automatic compaction trigger | 60,000 bytes; adjustable by `COMPACTION_TRIGGER_BYTES` |
| Generated summary | At most 6,000 UTF-8 bytes; model output request capped at 4,096 tokens |
| One compaction attempt | 180-second deadline, up to 5-second cooperative cancellation grace; no repeated attempt loop |
| Existing ordinary tutor run | Four aggregate tool requests, eight applied work passes plus possible halt batch; 180-second deadline plus ≤5-second cancellation grace |

The trigger must be a positive integer at most 84,000. Eligible older serialized history must contain at least 8,000 bytes, with more than two genuine complete learner exchanges overall. A candidate must save at least 1,024 bytes and reduce total measured input by at least 10%, while still fitting the input allowance. Otherwise history remains unchanged.

The conservative estimate counts native serialized static instructions, consented record/declaration context, tool definitions and messages including metadata. It adds a JSON-string allowance for each tool-call argument object (OpenAI serializes these again), conservative thinking replay, 512 bytes per message/content block/tool, and a 4,096-byte model/options/stream envelope allowance. Ordinary escaped text is not duplicated wholesale. A fixture compares it with the pinned OpenAI formatter on escape-heavy calls. This is not an exact wire capture, a proof of an upper bound for arbitrary providers, or tokenizer accounting; preserve the local provider restrictions and reserved headroom. Guard **every inference**, including the isolated summarization request; do not count only conversation prose. Tutor requests also cap output at 4,096 tokens, with 65,536 cumulative streamed text bytes and 2,000,000 serialized Event bytes per ordinary turn. The summary has the same Event-byte cap but a 6,000-byte text cap. Truncated/cancelled answers cannot authorize proposal approval or summary replacement. Provider reasoning metadata is allowed for inspection but only ordinary text becomes summary data; reasoning-only output fails. Per-turn retrieval tokens remain a separate Lesson 12 cap. Serialized bytes are not model tokens, an exact context-window size or a promise that every provider accepts that amount. The unchanged provider JSON declares no context limit. The local tutor reported 131,072 tokens and one slot during validation, but that observation is not a portable capacity promise or exact full-request count. Declared model request parameters that could override output limits are rejected; this is not a general configurable-provider lesson.

Automatic compaction happens at a safe boundary when eligible history and the trigger permit it. It does not ask the model to keep compacting until something fits. If static content, retained exchanges or summary input cannot fit safely, the application stops rather than sending oversized input or silently dropping protected messages. At deadline the application cancels cooperatively and polls for at most five more seconds to let the pinned SDK save available partial Effects/usage. If hard-aborted, received usage may be unavailable and is reported honestly. Timeout/cancellation is not proof that a service stopped doing work.

For a classroom demonstration, the instructor rehearses a lower valid trigger against the actual static-context baseline, leaving the hard budget unchanged. Do not invent a universally safe low value, intentionally overflow a provider or remove guards. Continue ordinary short replies until eligible older exchanges exist. A brief compaction notice should be enough in the learner terminal; inspect before/after sizes and summary privately in diagnostics. If no compaction is eligible yet, record that rather than pretending it occurred.

## 13.5 — Check continuation and fidelity

After actual compaction, ask: “What question was I still unsure about, and what small activity did I agree to?” Compare the reply with your actual earlier words. A tutor suggestion is not an agreed activity. If no activity was agreed, the tutor should not invent one. Correct any strengthened or missing claim and inspect whether a later summary keeps the correction.

Optionally ask for maximum planned loss at entry `51.20`, stop `50.70`, `200` shares. Expect the preserved exact-decimal calculator to return **$100.00**, excluding fees, slippage and gaps. This is not statistical expected loss or a guaranteed realized ceiling. Inspect actual tool selection/correlation; do not force a request or hide a wrong result. If that exchange later becomes eligible for summary, verify that the qualifications survive.

Ask a fresh source-grounded question such as “What do the supplied books say about liquidity and the bid-ask spread?” Predict a new search before citations, not reuse of summary IDs. Inspect current returned passages and adjacent claims separately: citation membership does not prove semantic support.

## 13.6 — Tweak and run again

Choose one focused change supplied by the instructor:

- Adjust `COMPACTION_TRIGGER_BYTES` to the rehearsed valid demonstration value, leaving the hard budget and all safeguards unchanged. Predict when compaction becomes eligible, then repeat from a fresh selection.
- In the supplied editable `compaction-guidance.md`, emphasize preserving learner corrections and distinguishing agreed activities from tutor suggestions. Start a fresh selection to load guidance, repeat a short conversation, and compare the actual summary.

Guidance is model input, not a hard enforcement mechanism. Do not relax source/provenance checks, increase caps, inject failures or rewrite the summarizer as a classroom exercise. Restore the default after rehearsal as appropriate. An inconclusive or inaccurate result remains a finding.

## Diagnostics, privacy and unchanged boundaries

`session-output.md` is private plain Markdown overwritten at launch, never read as learner memory. It should retain complete Session Debug fields/defaults/nested metadata and usage, loaded guidance/context, compaction reason and counting method, before/after sizes, eligible history/source coverage, summary, replacement/revision decisions, approvals, correlated tools/results, assembled answers and available partial failures. Diagnostics can contain learner words and private book excerpts. Git ignore is not encryption or access control; obtain permission and sanitize before projecting.

The learner UI stays simple: Tutor/Learner labels, brief status, ordinary streamed text and readable source markers. No application-added fences, arrows, raw arguments, IDs, summaries or Session dumps. Course colors are presentation-only, with independent stdout/stderr gating and plain fallback for redirection, any `NO_COLOR` and `TERM=dumb`. Saved/provider content stays unchanged. Log/storage failures stop the application; compaction adds no transactional or crash-recovery guarantee.

Application bookkeeping is bounded: 128 distinct completion declarations and 256 retained genuine-reply identities and settled approval decisions per Session. Reaching a bound refuses more affected work rather than forgetting authority; a fresh selection remains available. Successful compaction prunes removed reply identities. These in-process ledgers do not promise deduplication or recovery across launches.

Memory remains fixed-path, hand-editable between launches, size/UTF-8/regular-file/symlink checked at pathname inspection. Hand edits are supported between launches only; append is not secure against a concurrent malicious pathname swap between inspection and open. No filesystem transaction or hardened concurrent-writer claim is made. `/finish` starts selection again; `/quit`/EOF exit; blank replies are ignored. This lesson adds no brokerage, arbitrary file/SQL/network capability, browsing, market data, simulator or model-weight updates.

Keep Lesson 12's limitations: provisional three-PDF 511-passage collection, unfinished corpus review/freeze and runtime-parity evidence, no expert-authored assessment rubric, observed source-scope/feedback wording defects, unsupported-price refusal without search, incomplete memory-aware welcome, and a prior source run without a final answer. Compaction does not repair these findings by itself. Rights, disclosure and server exposure remain operator responsibilities.

## Success criteria and validation status

Structural success means a bounded attempt at a safe boundary reduces measured context, preserves opening/recent genuine exchanges and tool pairs/IDs/metadata, excludes derived text from quotation eligibility, settles approvals without replay, guards every inference and retains original history on failure. `/learned` requires consent for persistence (otherwise it remains ephemeral), saves only its narrow declaration without model authority, and does not restart or solicit an answer.

Human/live success means the summary faithfully distinguishes learner evidence from explanations, corrections from older misconceptions, agreed activities from suggestions, and calculator/source claims from their qualifications. The same Session supports a relevant follow-up and fresh source search. Fluent summaries and passing shape tests do **not** certify generated meaning or learner mastery.

### Validation record — 2026-10-12

- Published exact alpha.11 registry dependencies and matching provider types restored; Cargo.lock again includes registry sources/checksums, not mutable absolute paths. Rust 1.94.1, RMCP, TLS, provider JSON and preparation dependencies are unchanged. Format/check/Clippy across all targets and IDE build passed. **62 deterministic tests passed; two existing real-service tests were explicitly enabled and passed.** The latter verify actual Harrier/reader/tutor-tokenizer retrieval and shared same-snapshot search budgets. Root and all three Lesson 13 reference modules match byte-for-byte; the Lesson 12 reference and retrieval module were preserved.
- A declined-memory command run with a rehearsed **84,000-byte trigger** completed successfully: `/learned` compacted eligible history **68,885 → 53,213 estimated bytes**, preserving the Session, opening and latest two complete exchanges. The follow-up recovered the pending hypothetical comparison and relevant gap uncertainty without claiming mastery. A differently cased repeated declaration did not append again. `/finish` produced a new selection; a new source-grounded question then performed a real search, returned three draft excerpts and cited actual returned CFI IDs. Source-specific stock-versus-forex scope was visible in this answer; this does not certify corpus quality or all explanations.
- A consented completion-and-compaction run completed: deterministic declaration append, **74,730 → 57,191 estimated bytes**, no unsolicited command answer, then same-Session follow-up naming the declared concept and pending exercise without mastery/readiness certification. The inspected summary distinguished actual learner answers, tutor examples, self-reported acknowledgement and unassessed proficiency. No local note proposal was needed or forced for this command.
- A separate consented no-op/dedup run saved exactly one declaration; a new file was mode **0600**. Declined consent in a later launch left the existing file bytes unchanged and did not load it as context. Disposable test records were removed. A live ordinary-proposal regression prepared two correlated proposals: one independently approved, one declined; only the approved note was saved, and post-approval diagnostics recorded both settled decisions. Successful command-save followed by failed maintenance is covered deterministically, not claimed as a new injected live outage test.
- In a longer automatic-trigger run, real compactions **67,590 → 57,074** and **71,494 → 57,167 estimated bytes** occurred. A correlated calculator returned **$100.00** with fees/slippage/gap exclusions, followed by a grounded ordinary answer. The later source turn returned evidence but its next inference was refused at **95,262 > 84,000 estimated bytes**; no over-budget call or local approval occurred. Another earlier run stopped after calculator output at **91,445 > 84,000**. Protected recent exchanges and new evidence can exceed the conservative allowance even after safe compaction; this currently requires a fresh selection or smaller input/content, not mid-turn truncation/automatic retry. **A successful long retrieval continuation after these stops is not claimed.** The fresh-selection source check above is separate.
- The first real summary attempts failed because the reasoning-capable tutor supplied Thinking metadata, initially rejected as non-text, and some attempts reached the former 2,000-token cap. History remained unchanged, partial data/available usage were recorded, and a later budget refusal stopped safely. Validation now accepts provider Thinking/RedactedThinking metadata without including it in summary text; unexpected tools/non-text protocol content still fails. The output request cap is 4,096 tokens, while ordinary summary text remains bounded at 6,000 bytes. Passing retries do not erase initial failures.
- **Historical teaching/fidelity defects observed before the 2026-10-13 fix:** the tutor sometimes described a stop as ensuring exit or execution, contrary to the fixed execution-uncertainty contract. One automatic summary said open questions had been addressed based on tutor clarification, not a subsequent learner confirmation. A follow-up introduced a “two-sentence” exercise detail rather than simply preserving the accepted comparison. Record these attribution/strengthening issues; schema/size tests and citation membership cannot certify semantic accuracy. The successful consented summary still reflects only limited inspected evidence, not a reliability benchmark.
- Redirected, empty-NO_COLOR and TERM=dumb transcripts and complete private Markdown diagnostics contained no application ANSI. A real PTY arrow/Enter selection, colored ephemeral declaration/no-op status and quit passed (cyan headings, green status, plain model text); colored successful compaction and colored citation search were not separately rerun. Deterministic style/stream tests passed. Diagnostics include full new bookkeeping fields, before/after history, guidance, coverage IDs, summary, disposition, available usage/partial messages, tool data and post-approval state. Ignored captures remain private under the existing preparation directory, not public course examples.
- The unchanged local tutor reported build `b11429-d81235049`, one slot and 131,072 context tokens; clients did not restart services or infer capacity from the model name. No corpus reload/freeze, runtime parity, expert benchmark, network-isolation test, filesystem-race hardening, exhaustive cancellation/failure matrix, or generated-summary reliability claim is made. Cooperative grace/usage and hard-abort behavior are deterministic fixtures, not new real-timeout evidence. Existing source rights/exposure and prior lesson review gaps remain. **Content/teaching/UI and instructor release review are deferred to end-of-course review, not completed.**


### Targeted fidelity correction — 2026-10-13

The instructor requested correcting execution guarantees, unjustified closure of learner uncertainty and invented accepted-activity details. Only Lesson13 source/guidance/materials and its reference/documentation were changed; published SDK/toolchain/provider pins, predecessor references and preparation remain unchanged.

- Tutor contract and teaching material now explicitly distinguish seeking execution from guaranteed fill/exit/timing/completeness, stop-market from stop-limit, and conditional limit-price protection from fill certainty. Guidance separates a gap from an exclusively overnight event. Prior tutor mistakes are fallible, not authoritative knowledge.
- Summary input is projected ordinary content with original IDs/speakers/provenance; provider thinking and internal metadata are not summary evidence. Exact historical tool calls/results and correlation IDs remain available. Summary boundary and contrastive policy examples distinguish tutor explanations from genuine learner confirmation and reply-format instructions from activity acceptance. Accepted activities retain a short exact acceptance span rather than inheriting tutor example prices or sentence counts.
- Format/check/Clippy and IDE build passed; **66 deterministic tests** and **two explicitly enabled real-service tests** passed. Four new projection tests verify provenance, reasoning exclusion, unchanged originals, actual tool correlation/data and guarded projected request size, including pinned formatter comparison. They do not assert that keyword matching certifies model meaning. Root/reference compaction modules match.
- Four real-provider fresh-Session replays completed with actual compaction and same-Session follow-ups: **73,804→58,011**, **71,803→56,338**, **87,915→63,176**, and **78,910→58,974 estimated bytes**. The third pre-maintenance total includes saved provider output above the input allowance; replacement fit before the next inference—no allowance was raised and no over-budget inference was attempted. No memory was enabled or written. Tests included explicit nonconfirmation and the less-leading case of a learner asking a question, receiving explanations, and moving on without confirmation.
- Inspected summaries kept uncertainty unconfirmed rather than resolving it from tutor answers. Follow-ups did not impose a two-sentence exercise, example prices or deadline, and distinguished requests for short tutor replies from activity requirements. The first implicit replay still added the adjective “brief” to the recalled comparison, so guidance was tightened to prefer exact accepted wording; the subsequent implicit replay recalled only the accepted hypothetical comparison. These findings and retries are retained privately, not concealed.
- Targeted execution questions no longer asserted that stops ensure exit or that market/stop-market orders guarantee a complete immediate fill. The final answers distinguished stop-limit nonfill risk. **Other wording defects still appeared:** a tutor generalized limit orders as a price ceiling without specifying buy versus sell, described a stop-limit as matching an exact level instead of price-or-better, and one costs answer overstated spread costs as applying identically to every trade. These were not silently counted as correct domain teaching. This focused correction is not a general domain-quality benchmark or a proof that prompt guidance eliminates inaccuracies.
- All checked plain/NO_COLOR/dumb captures and diagnostics remained ANSI-free. Complete original messages/metadata remain diagnostic data; summaries are still fallible and never eligible learner quotes or current-turn citation evidence. No fresh colored UI, consented-save replay, live source/calc dialogue or broad summary benchmark was run in this correction; the two real-service tests and prior regression evidence are separate. Earlier long-input limits, source support defects and deferred instructor review remain.

### Behavior-preserving modularization validation — 2026-10-22

- The activated complete Lesson 13 source tree passed `cargo fmt --check`, `cargo check`, `cargo clippy --all-targets`, `cargo test` and `cargo build`: **66 deterministic tests passed; two live tests remained ignored in the default run**. The source audit preserved all **225 original function/test bodies**, test names and constants. Strings, types and attributes were preserved apart from necessary visibility/import/conditional-compilation boundaries. The missing test-only `DEFAULT_PROVIDER_CONFIG` import and rustfmt ordering were corrected; neither was a behavior fix.
- Both opt-in actual-service tests were explicitly attempted and **failed because `RETRIEVAL_DATABASE_URL` was missing**. The documented `cargo run -- custom_aa_llama_qwen3_6-35b.json` also failed for the same prerequisite before the chooser. The private `day_trading/embeddings/postgres-local/connection.env` was absent; Harrier on port 9999 was unavailable; the tutor health endpoint on port 8080 returned HTTP 200. Tutor health alone does not validate retrieval or a tutor exchange. No service was restarted or replaced with a mock.
- Parent validation checked plain, empty-`NO_COLOR` and `TERM=dumb` launch-error output plus independent stderr terminal gates. This checks failure presentation only, not successful chooser/dialogue styling. **No fresh live compaction, declaration, same-Session continuation, approval, search, calculator, chooser or teaching result is claimed in this pass.** The earlier live evidence and semantic corrections above remain historical, not rerun successes.
- Context/commands/provenance/authoritative state, preflight, revisions, maintenance wiring, runtime/Effects, concurrent Event consumption, dispatch/inspection and approval ordering remain visible in main. Compaction and retrieval remain cohesive with child test files; the six inherited support modules carry supplied mechanics. No prompt/content, dependency/provider, corpus, authority, budget or behavior fix is included. Historical semantic defects, long-input stops and validation/review gaps remain open; structural preservation is not teaching approval.
- This checkpoint describes the preserved Lesson 13 reference; the separately authorized Lesson 14 continuation is not absent or unauthorized. Root is ephemeral and ultimately matches the complete Lesson 14 tree after the combined pass. No backup/restore, hello-world reset, service restart, new release approval or fresh IDE/projector validation is claimed.

## Exit questions and next step

- What can `/learned` authorize, and why is it not mastery evidence?
- Which of the three domains changes during compaction?
- Why does a summary quotation not qualify as a real learner reply?
- Why preserve a complete tool pair, its IDs and metadata rather than its result alone?
- Why must an old source ID be re-searched for a new answer?
- What can byte/shape tests prove, and what still needs human inspection?

Next, Lesson 14 evaluates enforced boundaries and fallible retrieval, compaction and teaching behavior together. That is the conceptual next step; its separately authorized implementation lives in the Lesson 14 reference, not in this preserved Lesson 13 tree. Supplemental [Goose smart context management](https://goose-docs.ai/docs/guides/sessions/smart-context-management) describes full Goose; it does not automatically configure this pinned custom GDK application.


### Service-default validation — 2026-10-28

Added the instructor-supplied reader connection and Harrier/tutor origins as constants at the top of main. Retrieval uses them only when the corresponding environment variable is absent; the main tokenizer-origin comparison uses the same tutor default. Invalid/non-Unicode retrieval environment values still fail instead of silently selecting a default. No process environment is rewritten, and no credential is deliberately added to diagnostics or telemetry. Provider JSON, compatibility checks, reader checks, dependencies and services are unchanged. The plaintext default is a classroom-only exception, not secure credential management.

Locked format/check/all-target Clippy/all-target tests/build passed in temporary staging for each complete reference: Lesson 12 **41**, Lesson 13 **67**, Lesson 14 **69** deterministic application tests, plus seven unchanged seeder tests each. One new test per lesson validates the reader host/role/database/port, local service origins and absent-variable fallback. The active Lesson 12 root independently passed the same checks and byte-matches its reference. Existing staged changes were preserved; no lesson activation replaced the user's root.

With all three override variables unset, both opt-in actual-service tests and the documented launch were attempted for each reference. All stopped with **`embedding runtime identity mismatch`** before the chooser: the existing Harrier service does not satisfy the pinned runtime identity check. This is blocked live validation, not a passed database connection, retrieval, tutor exchange, approval, compaction or telemetry run. No service restart, compatibility relaxation, corpus/catalog write or mock substitution occurred. No fresh IDE/projector/UI or teaching-quality validation is claimed; historical findings and review gates remain.


## Embedding server build flexibility — 2026-10-28

At the instructor's request, the learner application no longer requires an exact llama.cpp build string. A different or absent `/props` `build_info` is accepted. Model alias, F16 format, GGUF checksum, dimensions, context capacity, pooling, normalized vectors, query formatting, stored collection/profile identity, provenance and reader restrictions remain checked. The stored profile's build suffix records the original embedding preparation; it is not a requirement that the current query server report that build. Accepting another build does not establish numerical equivalence or retrieval-quality parity. The standalone preparation loader/client retains its separate pinned-build policy. No service restart, re-embedding or collection mutation is needed solely to pass the removed build check.

Validation for this change: formatting, locked check, all-target Clippy, all-target tests and build passed (67 deterministic application tests plus seven unchanged seeder tests). Both opt-in actual-service retrieval tests passed against the currently running services, including server build `b11669-620d2225c`; the documented launch reached the chooser and exited on EOF. Lesson 12 was checked in the active root; Lessons 13–14 were checked in temporary staging without replacing root. Regression coverage accepts changed/absent build metadata and still rejects incorrect model alias/format, sleeping/no-slot runtime and mismatched context. No full tutor conversation, approval/compaction/telemetry workflow, numerical cross-build parity benchmark, fresh UI/IDE/projector or teaching-quality review was performed. No services restarted, corpus/catalog modified, preparation client changed or dependencies updated. Existing historical defects/review gates remain.
## Learner-led topic scope — 2026-10-28

The four supplied teaching units are optional background examples, not a topic whitelist or required progression. Learners may ask open-ended day-trading questions outside those examples and choose the depth of discussion. Offer topic suggestions only when asked or when the learner wants help choosing a direction. General educational discussion of broker-comparison criteria is allowed; personalized trading recommendations and unsupported claims about current fees, regulations or broker terms are not. Limited source coverage is an evidence limitation, not a ban on the topic.

The three editable content files retain this lesson's own capability and authority instructions; do not replace later-lesson contracts with Lesson 10's calculator-only contract. Content changes take effect on a fresh selection, not in an already-running selected conversation. Historical teaching findings remain evidence about earlier content, not proof that this revision fixes model behavior.

Validation for this content-only revision: static scope/capability/original-section checks passed across Lessons 10–14. The active Lesson 12 context-loading test and all-target tests passed (41 application tests and seven unchanged dataset-seeder tests; two opt-in real-service tests were not run). No new live teaching, approval, retrieval, compaction or telemetry validation is claimed.
