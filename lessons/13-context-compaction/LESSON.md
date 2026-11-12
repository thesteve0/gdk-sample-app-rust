# Lesson 13 — Continue learning with bounded working context

## Goal and supplied implementation

Continue a learning conversation without sending an ever-growing transcript to the tutor. Explain what may be shortened, what must remain authoritative, and why a generated summary cannot become learner evidence or permission to write memory.

Use the complete supplied Lesson 13 reference and root working copy, including the compaction, retrieval, concurrency and presentation helpers. **No missing Rust scaffolding is assigned.** The classroom rhythm is predict → run → explain → tweak → run again. Day trading remains introductory education, not a trading recommendation or competence certification.

**Implementation supplied; instructor review pending.** The complete source and policy are supplied; the validation record below distinguishes checks from remaining gaps. Historical planning-only wording in [PLAN.md](PLAN.md) and [the handoff](../final-lessons-handoff.md) precedes this task, not a claim that validation has occurred. This preserved Lesson 13 reference does not include Lesson 14 instrumentation; [Lesson 14](../14-agent-evaluation/LESSON.md) is separately authorized and implemented. Lesson 12 is complete. This continuation uses its final automatic retrieval and acknowledgment behavior plus a newly approved RAG-only Rust library; Lesson 13 remains pending instructor review. The current source map below supersedes earlier layouts, not their dated evidence.

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

A **safe boundary** is a point where an older complete exchange can be replaced without splitting pending tool work or disrupting current-turn inspection and local approvals. An **exchange** starts with one genuine learner reply and includes its completed tutor attempt(s), application reminder if needed, and correlated tool requests/results. Both attempts belong to that same genuine exchange. Pair validation uses each protocol kickoff window, so a calculator ID may be reused across attempts without splitting or confusing either pair. Requests and responses must remain together.

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

This intentionally replaces the exercise copy. No backup/restore or final hello-world reset is required. Preserve unrelated work outside `src/`. Activate Lesson 13 only when taking this lesson; do not reset or automatically advance to another lesson. To run this activated Lesson 13 copy:

```bash
umask 077
cargo run -- custom_aa_llama_qwen3_6-35b.json
```

The three local classroom defaults are `const` values at the top of `src/main.rs`: `RETRIEVAL_DATABASE_URL` uses the instructor-supplied `retrieval_reader` connection to `127.0.0.1:55432/harrier`, `HARRIER_URL` is `http://127.0.0.1:9999`, and `TUTOR_URL` is `http://127.0.0.1:8080`. No exports or private connection file are required for these defaults. Existing environment variables (including values loaded by `dotenvy` from `.env`) override them. For another prepared database, set `RETRIEVAL_DATABASE_URL` to its reader connection, never the loader/admin connection. The supplied password is **plaintext in source and compiled binaries**; this is an explicit local classroom convenience, not secret storage or a production credential pattern. Rotate/replace it for any other deployment and do not project the credential-bearing source. Defaults do not start services or bypass runtime/profile/reader validation. `TUTOR_URL` selects the tokenizer origin and must match the tutor provider JSON; it does not override generation configuration.


The tutor origin must match the provider JSON and its tokenizer service. Lesson 12's literal-loopback, proxy/redirect restrictions, compatible collection/profile and reader checks remain. Client loopback URLs do not secure an existing server bound to all interfaces. If services or authorized private books are unavailable, ask the instructor; mocks are not a silent substitute for a live run.

Maintenance checks, not classroom fill-ins:

```bash
cargo fmt --check
cargo check --all-targets
cargo clippy --all-targets
cargo test --all-targets
cargo build
```

The lesson sources share the root manifest; they are not a separate Cargo package. Copying only `main.rs` or only top-level `.rs` files is incomplete: the child test directories belong to the reference too. Use this Lesson 13 reference, not an earlier or later lesson's source, when repeating it.

## Supplied module map and teaching reading path

A **Rust module** groups related code under a local name. A **library target** compiles reusable code as a separate crate within this same Cargo package; the binary target owns the interactive application. This is not a new workspace, framework or shared course service. The library contains only previously taught retrieval-augmented generation (**RAG**) support: retrieving relevant text and supplying it as evidence for generation. Selection, context assembly and every Lesson 13 authority decision remain in the binary.

After defining that boundary, read [lib.rs](src/lib.rs):

```rust
pub mod rag;
```

The binary imports this library through `gdk_hello::rag`. A library cannot read binary-crate constants: main therefore supplies service defaults and retrieval limits through `retrieval_defaults()`/`RetrievalDefaults`, selection policy through `grounding_policy()`/`GroundingPolicy`, the authored contract to `TurnGrounding::prompt_part`, and the model-name bound to `validate_local_tutor`. Environment overrides still apply. Fixed protocol, collection/profile/model identities and SQL safety checks remain library implementation details, not adjustable classroom defaults.

Compaction is deliberately **binary-local**, not part of the reusable RAG library. Main's `compaction_policy()` supplies a `compaction::Policy` to `Policy::from_env`; `compaction.rs` can also import its binary parent's summary, ledger and safety constants. Do not confuse this local module access with library access to main.

A **grounding policy** is the application's choice of similarity cutoff, maximum grounded statements and retrieval-response token budget. Main constructs it, rather than making those teaching choices library defaults:

```rust
fn grounding_policy() -> GroundingPolicy {
    GroundingPolicy {
        min_similarity: grounding_threshold(),
        max_statements: GROUNDING_MAX_STATEMENTS,
        token_budget: GROUNDING_TOKEN_BUDGET,
    }
}
```

`retrieval_defaults()` similarly passes the service origins/reader connection and `RetrievalLimits` to `Retrieval::from_env_with`. Keep the credential-bearing constructor off the projector.

A **child test module** is a test file owned by its production module; `#[cfg(test)] mod tests;` in `compaction.rs` loads `compaction/tests.rs`. Every supplied Rust source starts `// Lesson 13`.

| Complete source map | Responsibility |
| --- | --- |
| [main.rs](src/main.rs) | Classroom constants/policy construction; context/prompt assembly; selection/commands; Session, authoritative state/provenance/revisions; retrieval/preflight/atomic append and maintenance call sites; runtime/Effects; concurrent Events; tools/limits; attempt/escalation/inspection and approval lifecycle. |
| [lib.rs](src/lib.rs), [rag/mod.rs](src/rag/mod.rs) | Library target and public RAG module/exports only. |
| [rag/grounding.rs](src/rag/grounding.rs) | Per-question retrieval preparation, threshold selection and ephemeral evidence/prompt formatting; no Session mutation or note authority. |
| [rag/citations.rs](src/rag/citations.rs) | Lexical acknowledgment classification, selected-ID warnings and citation footer; not semantic support verification. |
| [rag/transport.rs](src/rag/transport.rs) | Literal-loopback tutor, proxy/redirect and declared-output-override checks. |
| [rag/retrieval.rs](src/rag/retrieval.rs) | Cohesive read-only retrieval, runtime/profile/provenance checks, tokenizer accounting and cancellation/Drop guards. |
| [rag/tests.rs](src/rag/tests.rs), [rag/retrieval/tests.rs](src/rag/retrieval/tests.rs) | Library boundary and retrieval tests, including opt-in `live_local_smoke`. |
| [compaction.rs](src/compaction.rs), [compaction/tests.rs](src/compaction/tests.rs) | Binary-local declarations, projection/partition/budgets, isolated summary, revision-checked replacement, maintenance and child tests. |
| [calculator.rs](src/calculator.rs) | Binary-local exact-decimal calculator/schema/validation and local tests. |
| [chooser.rs](src/chooser.rs) | Binary-local selection/input/raw-mode cleanup. |
| [diagnostics.rs](src/diagnostics.rs) | Complete plain Markdown diagnostics and local tests; never learner memory. |
| [learning_record.rs](src/learning_record.rs) | Record types/schema/storage/confirmation mechanics and local tests; not permission to write. |
| [presentation.rs](src/presentation.rs) | Styles/gates/`TerminalProse` and local tests; not Event consumption or saved-text rewriting. |
| [provider_setup.rs](src/provider_setup.rs) | Provider path/first-model support, still binary-local. |
| [tests.rs](src/tests.rs) | Application/runtime/dispatch/approval/orchestration tests, including opt-in `live_forced_grounding_obeys_budget_and_current_turn_selection`. |

**Teaching reading path:**

1. Start in main: `LessonContext::load`, `run_learning_partner`, `reply_action`, and the three policy/default constructors. Trace fresh selection/consent and `/learned` before opening support files.
2. Trace `prepare_turn_grounding` → `compaction::prospective_bytes` → optional `compaction::maintain` with `PendingReply` → `ChatRuntime::append_grounded_user`. Then read `ChatSession::prompt_parts`, the runtime's revision/genuine-ID ledgers and separate `append_application` boundary.
3. Follow `BudgetPreparer` and maintenance into compaction's projection → partition → isolated summary → validation → `apply_outcome`; return to `ChatEffect::Maintenance`. Compaction/provenance stay application-owned.
4. Trace `run_turn` → `run_attempt` → `cooperative_run` with concurrent `display_events_bounded`, sender closure/draining, partial diagnostics and `inspect_turn`. A successful first attempt settles approvals before any one-time reminder; an unsuccessful/incomplete attempt cannot approve or escalate.
5. Read main's `LossToolProvider`, `SafeTools` and `RunLimit`. Retrieval is automatic before genuine questions, **not a model search tool**: there is no `RetrievalToolProvider` or `EvidenceBudget`. Visit the library only for the reusable retrieval/grounding/citation mechanics; decisions to append, inspect, retry and approve remain in main.
6. Finish with `parse_record_proposal`, `approval_pending`, `approve_record_proposals`: genuine-source validation → successful-run inspection → independent default-no decision → decision log → approved append → Session record refresh/revision → settled decision/revision → post-approval diagnostics. Read tests as boundary evidence, not class fill-ins.

Activation must copy the **complete recursive tree**, including `lib.rs`, `rag/` and child test directories, not just main or top-level files. The instructor calibration example now imports `gdk_hello::rag::retrieval` and needs this library layout active; these checks do not establish all-target compatibility when an older lesson without that library is activated.

## 13.0 — Carry forward Lesson 12 grounding

A **grounding attachment** is the selected book evidence for one genuine learner question, kept separately from conversation history. Before recording that question, the application performs one retrieval: default similarity threshold **0.55**, at most **three excerpts**, **2,500 tutor-token serialized search-response budget**, and **1,000 tutor tokens per excerpt**. These token limits are distinct from compaction's byte estimate. The threshold is provisional, not a relevance or truth guarantee. Kickoff, blank input and local commands do not trigger this question retrieval. The model cannot elect to skip it or request arbitrary SQL/search.

An **acknowledgment** is a visible citation or explicit decline of supplied excerpts. `ChatSession::prompt_parts` places the authored source-grounding contract first. A grounded answer must cite used excerpts with their exact `[source:<id>]` markers, or state that they did not support the answer with `[source:none]`. Evidence replaces the previous question's attachment and is not appended as a learner message.

An **attempt** is one bounded GDK run for the current question. `run_turn` classifies its successful final answer through `grounding_outcome`. A silent ignore receives **exactly one** application-authored reminder and a second attempt, with the same evidence and fresh attempt bounds; no third attempt occurs, even if the second answer silently ignores it too. Before appending that reminder, the application settles first-attempt proposals under the original learner quotation window. `append_application` does not add a genuine learner ID. Cite/decline contradictions and unselected source IDs are warnings, not more retries. Lexically, any non-sentinel citation counts as `Cited`; membership is checked separately, and neither check proves the adjacent claim is supported.

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

An optional **pending reply** is the upcoming genuine question and its selected grounding attachment, not yet recorded in history. `PendingReply` makes prospective maintenance count that question and replacement attachment before and after compaction. It does not make either item summary source material: isolated summary input contains only eligible historical messages, never the pending question or current evidence. `append_grounded_user` then validates a candidate containing both question and attachment before atomically changing stored history/evidence/genuine IDs and revision. A refused append leaves those unchanged. After defining the pending reply, inspect this focused argument from main's automatic `compaction::maintain` call:

```rust
Some(compaction::PendingReply {
    input: input.clone(),
    grounding: grounding.clone(),
}),
```

Then follow `append_grounded_user` to its candidate-budget check **before** any stored assignment. This order prevents a newly appended question from falsely becoming an incomplete historical exchange during maintenance.

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

**Citation boundary:** historical passage IDs in a summary are reminders of past evidence, not current citation support. Each later genuine learner question triggers a new retrieval before append; only that question's selected attachment can support its current citations. A valid old ID cannot substitute for the actual returned excerpt, its clipping and source qualifications.

## 13.4 — Observe the conservative budget

The supplied application policy is:

| Boundary | Supplied policy |
| --- | --- |
| Conservative total byte-estimate allowance | 100,000 bytes, including 16,000 reserved headroom (84,000 estimated-input allowance) |
| Default automatic compaction trigger | 60,000 bytes; adjustable by `COMPACTION_TRIGGER_BYTES` |
| Generated summary | At most 6,000 UTF-8 bytes; model output request capped at 4,096 tokens |
| One compaction attempt | 180-second deadline, up to 5-second cooperative cancellation grace; no repeated attempt loop |
| Each tutor attempt (at most two per genuine question) | Four aggregate tool requests, eight applied work passes plus possible halt batch; 180-second deadline plus ≤5-second cancellation grace; fresh bounds on the one permitted escalation |
| Per-question retrieval | One search before append; threshold 0.55, at most three excerpts, 2,500 serialized-response tokens and 1,000 tokens per excerpt |

The trigger must be a positive integer at most 84,000. Eligible older serialized history must contain at least 8,000 bytes, with more than two genuine complete learner exchanges overall. A candidate must save at least 1,024 bytes and reduce total measured input by at least 10%, while still fitting the input allowance. Otherwise history remains unchanged.

The conservative estimate counts native serialized static instructions, consented record/declaration context, tool definitions and messages including metadata. It adds a JSON-string allowance for each tool-call argument object (OpenAI serializes these again), conservative thinking replay, 512 bytes per message/content block/tool, and a 4,096-byte model/options/stream envelope allowance. Ordinary escaped text is not duplicated wholesale. A fixture compares it with the pinned OpenAI formatter on escape-heavy calls. This is not an exact wire capture, a proof of an upper bound for arbitrary providers, or tokenizer accounting; preserve the local provider restrictions and reserved headroom. Guard **every inference**, including the isolated summarization request; do not count only conversation prose. Tutor requests also cap output at 4,096 tokens, with 65,536 cumulative streamed text bytes and 2,000,000 serialized Event bytes per tutor attempt. The summary has the same Event-byte cap but a 6,000-byte text cap. Truncated/cancelled answers cannot authorize proposal approval or summary replacement. Provider reasoning metadata is allowed for inspection but only ordinary text becomes summary data; reasoning-only output fails. The 2,500-token serialized retrieval-response budget and 1,000-token excerpt cap remain separate; the upcoming selected attachment is included in prospective byte counting, not summarized as historical dialogue. Serialized bytes are not model tokens, an exact context-window size or a promise that every provider accepts that amount. The unchanged provider JSON declares no context limit. The local tutor reported 131,072 tokens and one slot during validation, but that observation is not a portable capacity promise or exact full-request count. Declared model request parameters that could override output limits are rejected; this is not a general configurable-provider lesson.

Automatic compaction happens at a safe boundary when eligible history and the trigger permit it. It does not ask the model to keep compacting until something fits. If static content, retained exchanges or summary input cannot fit safely, the application stops rather than sending oversized input or silently dropping protected messages. At deadline the application cancels cooperatively and polls for at most five more seconds to let the pinned SDK save available partial Effects/usage. If hard-aborted, received usage may be unavailable and is reported honestly. Timeout/cancellation is not proof that a service stopped doing work.

For a classroom demonstration, the instructor rehearses a lower valid trigger against the actual static-context baseline, leaving the hard budget unchanged. Do not invent a universally safe low value, intentionally overflow a provider or remove guards. Continue ordinary short replies until eligible older exchanges exist. A brief compaction notice should be enough in the learner terminal; inspect before/after sizes and summary privately in diagnostics. If no compaction is eligible yet, record that rather than pretending it occurred.

## 13.5 — Check continuation and fidelity

After actual compaction, ask: “What question was I still unsure about, and what small activity did I agree to?” Compare the reply with your actual earlier words. A tutor suggestion is not an agreed activity. If no activity was agreed, the tutor should not invent one. Correct any strengthened or missing claim and inspect whether a later summary keeps the correction.

Optionally ask for maximum planned loss at entry `51.20`, stop `50.70`, `200` shares. Expect the preserved exact-decimal calculator to return **$100.00**, excluding fees, slippage and gaps. This is not statistical expected loss or a guaranteed realized ceiling. Inspect actual tool selection/correlation; do not force a request or hide a wrong result. If that exchange later becomes eligible for summary, verify that the qualifications survive.

Ask a fresh source-grounded question such as “What do the supplied books say about liquidity and the bid-ask spread?” Predict one application-driven retrieval before the genuine question is appended, not a model search request or reuse of summary IDs. Inspect current returned passages and adjacent claims separately: citation membership does not prove semantic support.

## 13.6 — Tweak and run again

Choose one focused change supplied by the instructor:

- Adjust `COMPACTION_TRIGGER_BYTES` to the rehearsed valid demonstration value, leaving the hard budget and all safeguards unchanged. Predict when compaction becomes eligible, then repeat from a fresh selection.
- In the supplied editable `compaction-guidance.md`, emphasize preserving learner corrections and distinguishing agreed activities from tutor suggestions. Start a fresh selection to load guidance, repeat a short conversation, and compare the actual summary.

Guidance is model input, not a hard enforcement mechanism. Do not relax source/provenance checks, increase caps, inject failures or rewrite the summarizer as a classroom exercise. Restore the default after rehearsal as appropriate. An inconclusive or inaccurate result remains a finding.

## Diagnostics, privacy and unchanged boundaries

`session-output.md` is private plain Markdown overwritten at launch, never read as learner memory. It should retain complete Session Debug fields/defaults/nested metadata and usage, loaded guidance/context, selected grounding/candidates and outcome, each attempt's exact prepared system prompt, compaction reason and counting method, before/after sizes, eligible history/source coverage, summary, replacement/revision decisions, approvals, correlated tools/results, assembled answers and available partial failures. BudgetPreparer checks the full prepared input before capturing its exact system prompt; a capture is not proof the request reached the network/provider. Diagnostics can contain learner words and private book excerpts. Git ignore is not encryption or access control; obtain permission and sanitize before projecting.

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


## Historical root-only library split — 2026-11-11 (withdrawn)

> Historical record only. The Lesson 12 restoration withdrew this broad support-library layout. The approved RAG-only continuation below supersedes it; calculator/chooser/presentation/provider support remain binary-local. Its root/reference/telemetry statements describe that earlier checkpoint, not the current source map.

After this lesson's modularization, the instructor approved a root-only extraction that separates previously taught support from the Lesson 13 focus. The active root now declares a library target in `src/lib.rs` (`gdk_hello`) owning `calculator`, `chooser`, `evidence`, `presentation`, `provider_setup` and `retrieval`; `main.rs` keeps the Lesson 13 application (compaction, diagnostics, learning_record, providers, runtime, limits, evidence/citation decisions and approvals) and consumes the library through `use gdk_hello::…`. The `RetrievalToolProvider` and `LossToolProvider` remain in main.

A Rust **library target** compiles as its own crate inside the same package, so its modules cannot read binary-crate constants. The application therefore passes classroom defaults into library calls as parameters: `Retrieval::from_env_with(RetrievalDefaults { database_url, embedding_url, tutor_url })` (environment variables still override each default) and `choose_lesson(LESSON_TITLE)`. The four RAG glue helpers moved verbatim into `evidence.rs` with their tests in `evidence/tests.rs`; behavior, output, limits and the 69-function test inventory are unchanged.

Validation for this structural change: formatting, check, all-target Clippy, all-target tests and build passed; both opt-in actual-service tests passed against the live local services; the documented launch reached the chooser and exited on EOF. The preserved reference tree under this lesson's `src/` still shows the earlier single-crate module layout and is unchanged; activating this lesson from its reference reproduces that layout, not the library split. When the root later advances to Lesson 14, its telemetry source-hash inventory must be updated for the new lib/bin layout before telemetry evidence is trusted.


## Final Lesson 12 RAG continuation and RAG-only library — 2026-11-11

The instructor approved continuing from completed Lesson 12's final automatic grounding/acknowledgment implementation into Lesson 13, with a **RAG-only** library and main-owned policy passed as parameters. The current source map/reading path above describes the supplied continuation; the complete checked root source has been copied to the Lesson 13 reference and verified byte-identical. This supersedes the withdrawn broad root-only extraction, not the dated behavioral findings. Lesson 13 remains **implemented; instructor review pending**; Lesson 12's completion stands.

### Structural checks and actual-service evidence

- Formatting, all-target check/Clippy/tests and build passed: **105 deterministic application/library tests (25 library + 80 binary)**; two service tests ignored normally. **Seven unchanged dataset-seeder tests** also passed, with no catalog calls. New orchestration tests execute a real `StateMachine` with a scripted `Operation`, not a live provider: exactly two attempts/fresh bounds, reused actual calculator IDs, and no approval/escalation after error, halt, incomplete output or invalid first-attempt proposal. Pending/settled/revision/atomic-append boundaries are covered separately.
- Both opt-in actual-service tests explicitly passed again after the scoped-approval fix: `live_local_smoke` and `live_forced_grounding_obeys_budget_and_current_turn_selection`. Library live testing requires the three service variables supplied explicitly (privately from main's defaults); do not echo the reader connection. The documented root launch accepted `q`, reached the chooser and exited **0**.
- Live UI/conversation checks used a temporary staging package with byte-identical production sources; only Cargo's manifest-root location changed to isolate record/diagnostics. Root production sources were not altered for the runs. Plain, empty `NO_COLOR`, `TERM=dumb` and true colored-arrow modes, source question, blank input, ephemeral `/learned`, `/finish`/fresh selection and quit passed structurally. No real learner record was touched.
- A fresh calculator-only turn produced a correlated **$100.00** and final explanation with exclusions. Consented `/learned` saved to a new **0600** record; an inspected exact stated-goal proposal was approved; a rejected goal preserved bytes; a fresh memory-declined Session excluded the saved record.
- Live escalation **was exercised**: a rejected-note run first silently ignored supplied evidence; the local note decline occurred **before** the single reminder, then the second answer cited an exact source marker, without repeated approval/write. Another silent-ignore → `[source:none]` second answer was recorded as a decline; no third attempt occurred. A lexical acknowledgment still does not establish correct source support.

### Compaction, controlled stop and remaining teaching findings

With the rehearsed **50,000-byte trigger** (default remains **60,000**), a successful run reduced prospective input **79,546 → 64,475 estimated bytes**, then answered a fresh journal question with grounding in the **same Session**, exit **0**. Another run reduced **78,338 → 62,589** and ended with a valid decline, exit **0**. Uncertainty/activity follow-ups recalled the gap question/comparison **before** final compaction; they were not re-tested after that compaction. Do not claim a new post-compaction uncertainty/scope pass from the successful source follow-up alone.

An initial long run compacted prospective input **75,912 → 62,372**, then calculated **$100.00**, but a subsequent inference was refused at **86,314 > 84,000** estimated input bytes. There was **no final answer or new approval**; history was not silently truncated and partial diagnostics survived. Other runs honestly no-op'd because fewer than **8,000 eligible older serialized bytes** existed. Compaction is neither guaranteed after N turns nor a mid-attempt rescue or promise that protected context will fit.

Retain observed defects: wrong/unselected citation IDs warned; `[source:none]` on an ungrounded answer warned; execution wording included **“certainty of leaving”** despite guidance; source-scope/emotional-support claims were embellished; the tutor echoed raw proposal-acknowledgment wording; a consented welcome ignored loaded memory. Historical semantic findings and earlier failure evidence above remain relevant. Any non-sentinel marker still counts lexically as `Cited`; membership warnings are separate, with no semantic verification or extra retries.

No new live service-outage/failure injection, network-isolation, full-context/parity/corpus-quality benchmark, IDE/projector/teaching review, catalog write, dependency change or service restart is claimed. This evidence establishes specific structural/live outcomes, not semantic reliability, release approval or completed Lesson 14 evaluation.


### Final scoped-approval and validation follow-up — 2026-11-11

Settled note decisions now use the stable kickoff message ID together with the request ID, not a Session-global correlation ID. Five added regressions cover accepted/rejected prior decisions with reused IDs, independent new-proposal validation, and same-attempt pending-response scans. A reused ID cannot suppress a new local decision or make unsettled work compactable. The final inventory is 105 deterministic application/library tests plus seven unchanged seeder tests; both opt-in real-service tests passed again (17.05s library, 16.29s application). Formatting, all-target check/Clippy/tests and build remain warning-free. Complete root and Lesson 13 source trees match byte-for-byte.

A final memory-run retry approved the inspected exact goal and saved `/learned`, then rejected a fresh proposal before escalation. The second attempt proposed another note but hit the unchanged input guard at 96,721 > 84,000; there was no final answer or second local approval, and partial diagnostics survived. This is additional failure evidence, not a successful whole-workflow rerun. A validation-harness artifact-reuse mistake initially ran a root-bound binary instead of the isolated one, created only the known test record (the root record was absent initially), and overwrote diagnostics. That known test record was removed and the pre-test diagnostic file restored; a forced isolated compilation and manifest-root verification preceded the retry. No pre-existing learner record was read or modified. Earlier isolated successes remain separate evidence; no fresh final colored/compaction rerun is claimed.
