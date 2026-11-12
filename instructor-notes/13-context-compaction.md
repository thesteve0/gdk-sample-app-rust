# Instructor notes — Lesson 13: Context compaction

## Aim and handoff status

Teach one transferable boundary: shortening model working context must not silently change application facts, persistent-note authority or genuine learner evidence. Use [the lesson](../lessons/13-context-compaction/LESSON.md), complete supplied reference/helpers and editable summary guidance from the outset. Attendees predict → run → explain → tweak → run again; they do not rebuild async infrastructure or a summarizer.

The complete authorized implementation is supplied. See the lesson validation record for checked behavior and remaining gaps. The supplied limits are 100,000 conservatively estimated input bytes (including reserved headroom), 60,000-byte default trigger via `COMPACTION_TRIGGER_BYTES`, 6,000-byte summary with a 4,096-token generation cap and a 180-second deadline plus at most five seconds of cancellation grace per attempt. One grounded genuine question may have an initial attempt and exactly one acknowledgment escalation, each independently bounded. Do not treat these limits as evidence of summary fidelity or exact provider capacity. These notes describe the preserved Lesson 13 reference, not Lesson 14 instrumentation. Lesson 12 is complete; this continuation inherits its final automatic retrieval and acknowledgment behavior. The approved RAG-only library and current activation/source map below supersede the earlier root layouts. Lesson 14 remains separately implemented and pending review, not the active reading path here. Historical planning-only restrictions are not a current claim that Lesson 14 is unauthorized.

## Before class

1. Read README/AGENTS, Lesson 13 PLAN/handoff, complete Lesson 12 materials/content and the actual Lesson 13 source. Preserve unrelated edits. Confirm published alpha.11 SDK pins rather than an unreviewed local checkout.
2. Use [Lesson 12 DATABASE-SETUP](../lessons/12-specialized-retrieval/DATABASE-SETUP.md). Verify the existing local tutor, Harrier runtime/profile, provisional `harrier-llamacpp-test-511-v1` collection and reader role. No reload, new source corpus, model conversion or database lesson is needed. Keep source rights, disclosure and service exposure checks; a loopback client does not fix an all-interface server.
3. Activate the complete recursive reference as shown below, then launch with the lesson's root command and supplied local defaults or private `.env`/reader overrides. Never project credentials, environment connection strings or shell tracing. Services unavailable? Ask for them; do not relabel mock output as integration evidence.
4. Confirm final code retains original opening/latest two complete genuine learner exchanges, both initial/escalated attempts and their tool pairs/IDs/metadata; isolated tool-free GDK summary inference; one assistant-derived summary; revision-checked replacement; real-reply-only proposal quotation; settled accepted/declined approvals; all-inference budget guard and no repeated compaction loop. Locate the full runtime application boundary, not just an enum variant.
5. Confirm `/learned` is local, consent-gated, nonblank single-line ≤200 UTF-8 bytes, duplicate-suppressed within the same Session, and ephemeral (no disk access) with memory disabled. Its deterministic completion category is not model-proposable. Confirm no new welcome/unsolicited answer. A saved declaration is not rolled back on a later compaction failure.
6. Rehearse a lower valid trigger above the actual static-context baseline, with enough room for safe protected history. Measure how many ordinary exchanges are needed; supply that value to class rather than a universal guessed threshold. Keep the 100,000-byte hard budget unchanged. Do not deliberately overflow providers or break services as a learner exercise.
7. Run actual checks and live continuation, inspect summary meaning, consent/storage, tool and UI regressions. Prepare a permissioned sanitized diagnostic view. Diagnostic files contain learner text/private excerpts and are overwritten at launch; preserve any approved evidence privately before a new rehearsal.

## Source organization and activation

A **Rust module** groups related code; a **library target** is a separate compiled crate within this same package. Explain that distinction before showing declarations. The library is only previously taught **RAG** (retrieval-augmented generation) support. It is not the withdrawn broad support-library extraction, a workspace or a framework.

`lib.rs` exposes:

```rust
pub mod rag;
```

The binary imports `gdk_hello::rag`. Main keeps policy/credentials/constants and passes them through `retrieval_defaults()` (`RetrievalDefaults` plus `RetrievalLimits`), `grounding_policy()` (`GroundingPolicy`), the grounding-contract argument and tutor model-name bound. A library cannot read binary constants. Fixed protocol/model/profile/SQL safety identities stay in the library. In contrast, `compaction` remains a binary-local module: main supplies `compaction_policy()` to `Policy::from_env`, and compaction can import binary-parent summary/ledger/safety constants.

Use the lesson's [complete linked source map and teaching reading path](../lessons/13-context-compaction/LESSON.md#supplied-module-map-and-teaching-reading-path). Instructor navigation:

| Sources | Read for |
| --- | --- |
| `main.rs` | Context assembly/selection/commands; policies; Session, authoritative/provenance/revision ledgers; automatic retrieval/preflight/atomic append; runtime/Effects/concurrent Events; dispatch/limits; attempts/escalation/inspection and approval lifecycle. |
| `lib.rs`, `rag/mod.rs` | RAG-only library/export boundary. |
| `rag/grounding.rs`, `rag/citations.rs`, `rag/transport.rs` | Evidence selection/formatting; lexical acknowledgment/membership/footer; local tutor transport/override restrictions. No approval authority. |
| `rag/retrieval.rs`, `rag/retrieval/tests.rs`, `rag/tests.rs` | Cohesive retrieval/profile/provenance/token accounting/cancellation/Drop plus child tests and opt-in `live_local_smoke`. |
| `compaction.rs`, `compaction/tests.rs` | Binary-local declarations, summary projection/partition/budgets, isolated inference, revision replacement/maintenance and tests. |
| `calculator.rs`, `chooser.rs`, `diagnostics.rs`, `learning_record.rs`, `presentation.rs`, `provider_setup.rs` | Six binary-local support modules: decimal tools, selection/input, plain diagnostics, storage/confirmation, terminal styling/prose, provider selection. None moved to the library. |
| `tests.rs` | Application, scripted real-StateMachine orchestration and opt-in `live_forced_grounding_obeys_budget_and_current_turn_selection` tests. |

Trace main first: `run_learning_partner` → `prepare_turn_grounding` → `prospective_bytes` → optional `maintain(PendingReply)` → `append_grounded_user`. Then Session/Effect/revision boundaries, compaction internals, `run_turn`/`run_attempt`/`cooperative_run` and concurrent `display_events_bounded`, including closure/draining/partial diagnostics. Finish with `LossToolProvider`/`SafeTools`/`RunLimit` and `parse_record_proposal`/`approve_record_proposals`: inspect successful run → genuine-source validation → independent default-no decision → log → approved append → Session refresh/revision → settled decision/revision → post-approval diagnostics.

There is **no `RetrievalToolProvider`, `EvidenceBudget` or model search tool**. One automatic retrieval precedes each genuine question, with default threshold **0.55**, at most **three excerpts**, **2,500 tutor-token serialized-response budget** and **1,000 tokens per excerpt**. Kickoff/blank/local commands do not retrieve. Grounding stays a separate current-question attachment, not learner history. Main owns the grounding-contract text, one-time reminder and all authority decisions.

Activate all files recursively; `lib.rs`, `rag/` and child test directories are required:

```bash
rm -rf src
mkdir src
cp -R lessons/13-context-compaction/src/. src/
```

No backup/restore, final hello-world reset or automatic advancement is required. Preserve unrelated work and authoritative references. Every supplied Rust source begins `// Lesson 13`. The instructor calibration example now imports this RAG library; it requires the library layout active. Do not claim all-target compatibility for earlier lessons lacking it.

## Reveal script and class rhythm (about 45–55 minutes)

### 1. Motivate, define and predict (5 minutes)

Say: “The tutor can keep talking, but we cannot keep sending an unlimited transcript. What must not change when we shorten it?” Define local learning declaration, three state domains, working context and lossy compaction **before** code.

Ask: “Is hearing a good explanation proof that you learned it?” No. “If you choose `/learned`, who makes that declaration?” The learner, not an assessor. “Can the model turn that into permission to save other notes?” No.

Open [the editable SVG](../lessons/13-context-compaction/visuals/compaction-boundary.svg). Its content is conceptual, not actual output. Reveal in this order (zoom or cover regions; no animation/dependency needed):

1. Top strip: authoritative application state and learner-authorized record stay separate from working context.
2. Left column: original opening, older complete exchanges, protected latest two genuine exchanges.
3. Center boundary: settled approvals, no pending split pair; isolated tool-free summary; revision/validation before applying.
4. Right column: opening + one derived summary + the same recent exchanges. Bottom warning: genuine quotations and new-turn citations cannot come from a summary.

Prediction: “Which arrows represent changing stored memory?” None. Read [the text equivalent](../lessons/13-context-compaction/visuals/compaction-boundary.md) aloud when projection is unavailable. The visual shows what the policy intends, not a claim that a particular model summary is faithful.

### 2. Run a normal learning conversation (7 minutes)

Select Getting started in day trading; let the learner choose default-no memory. Ask about stop price versus execution price, invite one real learner explanation and an unresolved question. Suggest at most one small optional activity and wait for agreement. Do not fabricate agreement to give the summary something to retain.

Have the learner make `/learned stop price versus execution price` only if they genuinely want that declaration. Predict a local status/next prompt, no tutor run or fresh welcome. With consent it narrowly authorizes the deterministic declaration append; without consent it does not save. Explain validation/repeat behavior without assigning invalid-input or storage-failure experiments. Other proposals still need independent local approval.

Say: “This is your declaration, not proof that you could execute a trade safely.” Do not mark a unit mastered or turn the learning priorities into a compulsory completion checklist.

### 3. Explain before opening supplied code (8 minutes)

Define genuine provenance, complete exchange, safe boundary, Operation, Effect, Event, revision and budget. Then trace the supplied implementation:

eligible older history → isolated bounded inference → validated replacement/revision → runtime Effect → GDK reload/re-evaluation.

Show the lesson's pinned `ReplaceConversation(Conversation)` variant only after defining replacement. It demonstrates SDK capability, not an automatic policy. The application has its own effect enum/runtime and must enforce the safeguards. GDK owns iteration; there is no new hand-written model/tool loop and no assumed full Goose compaction assembly.

Emphasize: original opening and latest two complete genuine learner exchanges remain with tool pairs/IDs/metadata. Older summary plus newly eligible history become one derived summary. Static instructions, teaching guidance, consented record and tools remain separately counted. Upcoming question/evidence are counted through optional `PendingReply` before actual append and on the replacement candidate, but never enter the isolated historical-summary input. `append_grounded_user` validates the candidate question plus attachment before atomically storing history/evidence/genuine ID and incrementing revision. Summary-generation tools are absent; generated text is data, not stronger instructions.

One genuine exchange may include two attempts: the same evidence, one application reminder and both sets of tool pairs. Pair validation resets at each SDK kickoff, permitting actual calculator ID reuse across attempts without splitting either pair. First-attempt approvals must settle **before** the reminder changes the protocol quotation window; `append_application` never adds a genuine learner ID. A successful silent ignore alone triggers one retry; errors, safeguards, incomplete output or invalid proposal cannot approve or escalate. Lexical `Cited` includes any non-sentinel marker; separate ID warnings and human support review remain necessary.

A state revision guards against stale application. Failure leaves working history intact; if `/learned` already saved, report that honestly rather than silently repeat storage. Local approved/declined proposals are settled; compaction must not reopen either. Storage is still single-process, not transactional or crash-exactly-once.

### 4. Observe compaction and continue (10 minutes)

Use the rehearsed trigger; continue ordinary short learning replies until older exchanges are eligible. Do not imply two replies must always trigger it. Watch for a brief learner-facing notice, then privately inspect counted before/after input, selected source history, generated summary and applied replacement. If it did not compact, explain the actual threshold/eligibility evidence.

Ask: “What question was I still unsure about, and what small activity did I agree to?” Check against actual words; a suggested activity is not accepted. Correct any summary strengthening. The conversation continues in the same Session, not a new welcome. The command itself should not elicit an unsolicited answer.

Optionally run entry 51.20/stop 50.70/200 shares: the exact calculator result is $100.00 maximum **planned** loss, excluding fees, slippage and gaps, not expected loss or guaranteed realized loss. Observe actual tool selection. When summarized later, keep the qualifications, not only the number.

Ask a new book-grounded liquidity/spread question. Predict automatic retrieval before question append and current citations, not an optional model search request. Historical IDs in a summary do not preserve current excerpt support. Inspect returned ID membership separately from whether each passage supports its adjacent claim.

### 5. Tweak, predict and repeat (7 minutes)

Choose one: adjust only the instructor-rehearsed valid trigger, or emphasize corrections and activity agreement in the supplied `compaction-guidance.md`. Start a fresh selection to load guidance and repeat a small conversation. Keep budget/safeguards unchanged. Model guidance can improve a summary but cannot enforce meaning.

Compare fidelity, not identical generated wording or rankings. Record inconclusive, lost or invented claims openly. Restore defaults as appropriate. Never secretly force tool calls, trim evidence to hide a defect or turn failure probes into unapproved exercises.

### 6. Exit discussion (5 minutes)

- Which of the three domains changed? Which did not?
- Why is `/learned` narrow write permission, not mastery evidence or model authority?
- Can an old learner quote copied into a summary authorize a new proposal? No: only retained genuine replies qualify.
- Why keep request/result pairs, IDs and metadata together?
- Can a historical source ID support a new-turn citation? Not without fresh returned evidence.
- Does a valid 6,000-byte summary prove that its claims are accurate? No.

Bridge to the separately authorized Lesson 14: evaluate deterministic boundaries separately from fallible evidence use and teaching judgments. Its instrumentation belongs in its own preserved reference, not this Lesson 13 teaching path.

## Summary fidelity review checklist

Compare actual eligible history and generated summary, not merely the next fluent response. Record specific omissions/strengthening rather than a global “looks good.”

| Check | Question |
| --- | --- |
| Claims and attribution | Does it distinguish learner statements, tutor explanations, book claims and tool facts? Were suggestions promoted into facts or policy? |
| Evidence and limits | Does reported understanding cite actual learner behavior narrowly, without mastery/readiness inference? |
| Corrections | Is the latest correction preserved, rather than reviving an older misconception or outdated goal? |
| Open questions | Are unresolved questions still open, not marked answered because the tutor explained them? |
| Activities | Was the activity explicitly agreed, merely offered, completed by learner declaration, or still pending? Are these distinct? |
| Calculator qualifications | Are exact inputs/result and exclusions retained when relevant? No guaranteed realized ceiling or statistical expected-loss relabeling? |
| Citation qualifications | Are source scope, physical-page/clipping/draft/dated limitations kept? Are IDs historical reminders, not new-turn evidence? |
| Instruction boundary | Is instruction-like learner/book/record text treated as data, not promoted into system policy? |
| Genuine provenance | Are summary/tool/opening/command text excluded from proposal quotation sources? Did removal of old replies restrict eligibility honestly? |
| Authority and continuity | No new write authority, duplicate local approval, fresh welcome or silent state rollback? |

Shape, size, correlation and revision tests enforce deterministic boundaries. They cannot certify that generated meaning is complete, true or pedagogically helpful. Summary fidelity needs live human inspection and, later, systematic evaluation. A follow-up can be sensible while the summary is wrong; inspect both.

## Budget explanation to say aloud

“100,000 estimated bytes is our conservative application budget, including 16,000 bytes reserved headroom; native JSON plus argument-string/thinking replay allowances, 512-byte per-envelope margins and a 4,096-byte request margin are estimated, not a wire capture; it is not an exact token window. 60,000 is a trigger that leaves headroom, not the model's capacity. We count static context, consented notes, tool definitions and history, and guard the separate summary input too.” Keep byte allowance distinct from model-token capacity.

The summary is ≤6,000 ordinary-text UTF-8 bytes with a 4,096-token output request cap; provider thinking metadata is inspected but never inserted as summary text. One attempt has a 180-second deadline plus ≤5-second cooperative cancellation grace; hard-aborted received usage can be unavailable. Each tutor attempt retains four shared requests/eight applied work passes plus possible halt, with the same180-second deadline/5-second grace and explicit4096-token/65536-text-byte/2000000-Event-byte output caps. Record compaction work/usage separately, do not hide it as a free second inference path. Automatic safe-boundary compaction is not an unlimited retry loop. Oversized static/protected context causes a controlled stop, not another forced summary or silent deletion. Cancellation is not rollback or proof the service stopped. The current retrieval response budget is 2,500 tutor tokens, with 1,000 tokens per excerpt and at most three threshold-selected excerpts, not the historical 4,000-token tool-evidence budget. Prospective estimates include the new question/attachment, replacing the previous attachment. These limits remain separate from accumulated working-context bytes. The single permitted escalation receives fresh attempt limits but not a larger 84,000-byte input allowance.

## Privacy, diagnostics and inherited limitations

Learner UI retains clean Tutor/Learner dialogue, readable citations, brief status and presentation-only course palette/gating. Raw summaries, IDs, requests, Session dumps and actor/fence protocols stay out of ordinary dialogue. Verify NO_COLOR/redirection/TERM=dumb and independent stderr fallback; common Markdown suppression is not a general renderer. Provider/saved text must remain unchanged.

`session-output.md` is overwritten plain private diagnostics, never memory. Include full fields/defaults/metadata/usage and loaded context, counted sizes, source coverage, summary/replacement/revision decisions, settled approvals, declarations, complete tools, answers and available partial failures. Include per-attempt grounding outcomes and exact system prompts captured only after `BudgetPreparer` accepts the full estimated request; this is exact prepared input, **not proof of delivery**. Exclude deliberate credential/configuration/secret-URL/vector dumps. Ignored files remain readable to anyone with filesystem access; sanitize and obtain consent before projection. Log/storage failure stops; unavailable logging cannot promise to record itself.

Memory remains optional per selection and hand-editable between launches. `/finish`, `/quit`, EOF and blank replies keep their meanings. No new file/network/SQL/order authority, corpus freeze, model parity, expert rubric or clinical mastery assessment appears here.

Do not erase Lesson 12 findings: source-scope and feedback relationship wording defects despite valid IDs; unsupported-price refusal without search; hand-edited activity transported but not offered in welcome; an initial search with no final ordinary answer. The 511 passages are provisional, not all books or an accepted corpus. Earlier review/validation gaps remain independent. Compaction does not certify those problems fixed.

## Source evidence and validation record

For documentation, checked the locally cached published `goose-agent 0.1.0-alpha.11` `operation.rs` (Operation, ConversationEffect/ReplaceConversation) and `machine.rs` (load → step → apply → reload run). Official [Goose smart context management](https://goose-docs.ai/docs/guides/sessions/smart-context-management) documents full Goose, not an automatically supplied policy for this application. A later source spot-check confirmed 16,000-byte headroom (84,000 estimated-input allowance), at least 8,000 eligible older serialized bytes, and replacement reduction of at least 1,024 bytes and 10%. The current verification also read alpha.11 `inference.rs` (`InferenceRequestPreparer`, `PreparedInferenceRequest`, `with_request_preparer`) and `operation.rs`'s `messages_since_kickoff`: the latest user-visible non-tool-response user message starts the SDK attempt window, whereas genuine exchange identity remains application-owned. Prompt capture occurs in the application preparer after its budget guard, not as provider-delivery confirmation. It confirmed declined-consent declarations are ephemeral, not rejected; plain-word/simple-punctuation validation applies. Bounded completion/provenance/settled-decision ledgers require final review (currently 128 completions and 256 entries for each other ledger). Inspect the supplied main/compaction modules before class; bounded ledgers do not confer crash recovery or across-launch deduplication.

## Implementation validation and review handoff — 2026-10-12

See the [Lesson 13 validation record](../lessons/13-context-compaction/LESSON.md#validation-record--2026-10-12) for the authoritative detailed evidence. Formatting/check/Clippy/IDE build and **62 deterministic tests** passed; **two explicitly enabled real-service tests** passed. Published alpha.11 registry pins are restored, root/reference match, predecessor/preparation preserved. No SDK migration until the class is finished and verified.

Actual command compaction and same-Session follow-up passed both without persistence (68,885→53,213 estimated bytes) and with consented deterministic save (74,730→57,191). Dedup/no-op behavior, new0600 record, declined consent preserving existing bytes, independent accepted/rejected model proposals, fresh source search, correlated$100 calculator, plain/NO_COLOR/dumb and colored arrow declaration UI were exercised. Disposable records removed; private captures are ignored, not projector assets.

Do not hide initial rejected reasoning-metadata summaries, former2000-token truncations, or later safe budget stops after calculator/retrieval. Reasoning metadata is now accepted for inspection, never inserted as summary text; cap4096 output tokens and6000 text bytes remain. Native JSON plus conservative argument/thinking/protocol allowances is an estimate, not an exact provider request upper-bound proof. Two long runs exceeded the protected input allowance and stopped without next inference/approval; a fresh-selection search passed separately. No automatic mid-turn compaction or truncation rescue is promised.

Inspect meaning: tutor execution guarantees were wrong; one summary treated tutor clarification as resolved learner uncertainty; a follow-up strengthened the accepted exercise into a two-sentence form. Successful summaries distinguished learner evidence and unassessed acknowledgement, but do not establish general fidelity. Preserve these cases for Lesson14 evaluation rather than rewriting them as successes.

Colored eligible compaction/citation search, real timeout/grace usage, injected post-save outage, comprehensive failure/network isolation, filesystem-race hardening and expert quality benchmark were not validated here. The pathname checks do not protect against malicious concurrent replacement between inspection/open; edits remain supported between launches only. No corpus freeze, service restart or cross-runtime parity. Deferred instructor review remains pending, not release approval.


## Fidelity correction rehearsal — 2026-10-13

Use the updated lesson's targeted correction record; do not project private captures. Explain ordinary-content projection before opening `summary_source`: originals keep their metadata/reasoning, but the summary request sees labeled ordinary speaker/provenance records and real correlated tool data. A tutor explanation is not learner confirmation; a short-answer preference is not an exercise constraint. Contrastive guidance is supplied, not a Rust exercise or proof of semantic validation.

Four live compaction/follow-up replays passed the targeted uncertainty/activity/stop-guarantee checks, including a less-leading case without explicit nonconfirmation. The first implicit follow-up added “brief”; exact-acceptance wording was tightened and a subsequent replay avoided it.66 deterministic and2 real-service tests plus format/check/Clippy/IDE build passed. Four new tests enforce input projection/provenance/budget, not mastery or generated truth.

Do not replace the original defect evidence with “hallucinations solved.” New tutor wording still confused buy/sell limit boundaries, exact-level versus price-or-better, and generalized spread costs. The requested fixes have targeted live evidence, not global domain correctness. Inspect accepted activity scope and unconfirmed questions after compaction. Old long-input stops and source-support/review gaps remain. No dependency migration, corpus change, later-lesson implementation or release approval.


## Modularization checkpoint — 2026-10-22

See the lesson's [dated modularization validation](../lessons/13-context-compaction/LESSON.md#behavior-preserving-modularization-validation--2026-10-22). The activated recursive tree passed format/check/Clippy/test/build: **66 deterministic tests passed, two live tests ignored by default**. The source audit preserved **225 original function/test bodies**, test names/constants, strings/types/attributes except necessary visibility/import/cfg boundaries. A missing test-only `DEFAULT_PROVIDER_CONFIG` import and rustfmt ordering were corrected; no behavior fix or prompt change was made. The six inherited support modules reduce reading noise; compaction/retrieval remain cohesive with child tests, and the main authority/runtime/inspection/approval path remains visible.

Both explicitly enabled live tests failed for missing `RETRIEVAL_DATABASE_URL`; the documented application launch failed at the same prerequisite **before the chooser**. Private `connection.env` was absent, Harrier9999 unavailable, tutor8080 health returned200. Do not confuse tutor health with a successful tutor/retrieval integration. No fresh compaction, declaration, continuation, approval, search, calculator, chooser or teaching outcome is established. Parent checks covered plain/empty-NO_COLOR/dumb launch errors and independent stderr gates only, not a new successful colored dialogue. No service restart, mock substitute, corpus/catalog write or fresh IDE/projector review occurred.

Preserve the historical live successes, semantic correction replays, residual domain defects, long-input stops and incomplete review/failure evidence above. This is behavior-preserving organization, not certification of semantic fidelity, release approval or permission to erase gaps. Lesson 14 is separately implemented/authorized, and root ultimately matches it after the combined pass; Lesson 13 remains a complete independent reference. No backup/reset is part of this checkpoint.


## Local service defaults — 2026-10-28

The three local classroom defaults are `const` values at the top of `src/main.rs`: `RETRIEVAL_DATABASE_URL` uses the instructor-supplied `retrieval_reader` connection to `127.0.0.1:55432/harrier`, `HARRIER_URL` is `http://127.0.0.1:9999`, and `TUTOR_URL` is `http://127.0.0.1:8080`. No exports or private connection file are required for these defaults. Existing environment variables (including values loaded by `dotenvy` from `.env`) override them. For another prepared database, set `RETRIEVAL_DATABASE_URL` to its reader connection, never the loader/admin connection. The supplied password is **plaintext in source and compiled binaries**; this is an explicit local classroom convenience, not secret storage or a production credential pattern. Rotate/replace it for any other deployment and do not project the credential-bearing source. Defaults do not start services or bypass runtime/profile/reader validation. `TUTOR_URL` selects the tokenizer origin and must match the tutor provider JSON; it does not override generation configuration.


### Service-default validation — 2026-10-28

Added the instructor-supplied reader connection and Harrier/tutor origins as constants at the top of main. Retrieval uses them only when the corresponding environment variable is absent; the main tokenizer-origin comparison uses the same tutor default. Invalid/non-Unicode retrieval environment values still fail instead of silently selecting a default. No process environment is rewritten, and no credential is deliberately added to diagnostics or telemetry. Provider JSON, compatibility checks, reader checks, dependencies and services are unchanged. The plaintext default is a classroom-only exception, not secure credential management.

Locked format/check/all-target Clippy/all-target tests/build passed in temporary staging for each complete reference: Lesson 12 **41**, Lesson 13 **67**, Lesson 14 **69** deterministic application tests, plus seven unchanged seeder tests each. One new test per lesson validates the reader host/role/database/port, local service origins and absent-variable fallback. The active Lesson 12 root independently passed the same checks and byte-matches its reference. Existing staged changes were preserved; no lesson activation replaced the user's root.

With all three override variables unset, both opt-in actual-service tests and the documented launch were attempted for each reference. All stopped with **`embedding runtime identity mismatch`** before the chooser: the existing Harrier service does not satisfy the pinned runtime identity check. This is blocked live validation, not a passed database connection, retrieval, tutor exchange, approval, compaction or telemetry run. No service restart, compatibility relaxation, corpus/catalog write or mock substitution occurred. No fresh IDE/projector/UI or teaching-quality validation is claimed; historical findings and review gates remain.


## Embedding server build flexibility — 2026-10-28

At the instructor's request, the learner application no longer requires an exact llama.cpp build string. A different or absent `/props` `build_info` is accepted. Model alias, F16 format, GGUF checksum, dimensions, context capacity, pooling, normalized vectors, query formatting, stored collection/profile identity, provenance and reader restrictions remain checked. The stored profile's build suffix records the original embedding preparation; it is not a requirement that the current query server report that build. Accepting another build does not establish numerical equivalence or retrieval-quality parity. The standalone preparation loader/client retains its separate pinned-build policy. No service restart, re-embedding or collection mutation is needed solely to pass the removed build check.

Validation for this change: formatting, locked check, all-target Clippy, all-target tests and build passed (67 deterministic application tests plus seven unchanged seeder tests). Both opt-in actual-service retrieval tests passed against the currently running services, including server build `b11669-620d2225c`; the documented launch reached the chooser and exited on EOF. Lesson 12 was checked in the active root; Lessons 13–14 were checked in temporary staging without replacing root. Regression coverage accepts changed/absent build metadata and still rejects incorrect model alias/format, sleeping/no-slot runtime and mismatched context. No full tutor conversation, approval/compaction/telemetry workflow, numerical cross-build parity benchmark, fresh UI/IDE/projector or teaching-quality review was performed. No services restarted, corpus/catalog modified, preparation client changed or dependencies updated. Existing historical defects/review gates remain.

## Learner-led scope follow-up — 2026-10-28

The content now explicitly accepts learner-chosen day-trading questions beyond the four supplied examples. Offer the menu only when requested or when the learner wants help choosing. Preserve lesson-specific capability, memory, citation and continuity instructions rather than copying a common calculator-only contract across lessons. For a fresh-selection rehearsal, ask “What should I consider when comparing brokers?” and then choose another topic outside the examples. Inspect whether the tutor answers educationally without redirecting to the preset units, inventing current facts or recommending a personalized financial decision. In retrieval lessons, separately ask what the supplied books support and inspect actual returned evidence/citations.

Validation of this edit: static content checks across Lessons 10–14 and the active Lesson 12 context-loading test passed. No new live tutor, memory approval, retrieval, compaction or telemetry behavior is established by these checks; fresh teaching validation remains pending.


## Historical root-only library split — 2026-11-11 (withdrawn)

> Historical record only. The subsequent Lesson 12 restoration withdrew this broad extraction. The approved RAG-only continuation below supersedes it. The statements about active layout/reference/telemetry in this section describe that earlier checkpoint, not the current source map.

The instructor approved a root-only extraction that separates previously taught support from the Lesson 13 focus, so the working copy now compiles a library target alongside the application. `src/lib.rs` declares library `gdk_hello`, owning `calculator`, `chooser`, `evidence`, `presentation`, `provider_setup` and `retrieval`; `main.rs` keeps everything this lesson teaches — context assembly, commands, compaction wiring, runtime/Effects, concurrent Events, dispatch/limits, evidence/citation inspection and the approval lifecycle — and imports the support modules with `use gdk_hello::…`. `RetrievalToolProvider` and `LossToolProvider` stay in main.

Explain the parameter-passing consequence in class: a library crate cannot read binary-crate constants, so the three classroom service defaults and the lesson title are passed into library calls (`Retrieval::from_env_with(RetrievalDefaults { … })`, `choose_lesson(LESSON_TITLE)`) and environment variables still override. The four RAG glue helpers and their tests now live in `evidence.rs`/`evidence/tests.rs`; the reading path and every boundary decision above are unchanged, and the 69-function test inventory is conserved.

Validation: formatting/check/Clippy/tests/build all pass (67 deterministic tests; lib 18 + bin 49), both opt-in actual-service tests passed against live services, and the documented launch reached the chooser and exited on EOF. The preserved reference tree under this lesson's `src/` is untouched and keeps its single-crate layout; reactivating from the reference reproduces that layout, not the split. Before trusting Lesson 14 telemetry after reactivation, update its source-hash inventory for the new lib/bin layout.


## Final Lesson 12 continuation / RAG-only library checkpoint — 2026-11-11

Use the [new dated lesson evidence](../lessons/13-context-compaction/LESSON.md#final-lesson-12-rag-continuation-and-rag-only-library--2026-11-11), not the withdrawn broad split or old model-search reading path. The instructor approved final Lesson 12 RAG behavior plus this RAG-only library; the checked root and complete reference source trees now match byte-for-byte. Lesson 12 remains complete, Lesson 13 remains **implemented; instructor review pending**.

**Checks reported for this implementation:** format, all-target check/Clippy/tests and build passed; **105 deterministic tests (25 library +80 binary), two service tests normally ignored**, plus **seven unchanged seeder tests without catalog calls**. Both actual-service tests explicitly passed again after the scoped-approval fix (`live_local_smoke`, `live_forced_grounding_obeys_budget_and_current_turn_selection`); the library test receives all three service environment variables privately from main defaults, never via projected credentials. Root launch reached chooser, `q` exited0. Scripted-Operation tests exercise the real StateMachine, not a live provider, for fresh two-attempt bounds/reused calculator IDs and refusal to approve/escalate after errors, safeguards, incomplete output or invalid first proposal; pending/settled/revision/atomic-append tests preserve authority.

**Live demonstrations:** temporary staging used byte-identical production sources; only Cargo's manifest-root location changed to isolate record/log, leaving root sources and real record untouched. Plain/empty-NO_COLOR/dumb/true colored-arrow, grounded source question, blank, ephemeral declaration, finish/fresh selection and quit passed structurally. Calculator-only request correlated `$100.00` with exclusions. Consented `/learned` created0600 storage; exact stated goal was inspected/approved; rejected goal preserved bytes; memory-declined fresh Session excluded saved text. An actual silent-ignore run declined its note locally **before** one reminder, then cited an exact source, with no repeated approval/write. Another escalated answer declined via `[source:none]`, no third retry. Do not say live escalation is untested now.

**Compaction evidence:** rehearsed trigger50000/default60000; successful prospective79546→64475 followed by a fresh grounded journal answer in the **same Session**, exit0; separate78338→62589 then valid decline, exit0. Gap-question/comparison recalls occurred **before** final compaction, not as new post-compaction fidelity validation. Initial75912→62372 then correlated100 calculator hit subsequent86314>84000 guard: no final answer/new approval, no truncation, partial diagnostics preserved. Other insufficient-8000-byte-history runs honestly no-op'd. Do not promise compaction after a fixed number of replies or mid-turn rescue.

**Teach the remaining defects openly:** unselected IDs and ungrounded `[source:none]` warnings were observed; lexical acknowledgment is not membership or semantic support. Execution “certainty of leaving,” source-scope/emotional-support embellishment, raw proposal acknowledgment echo and consented welcome ignoring memory persist alongside historical findings. No more retries repair those semantics. No new live outage/failure injection, network isolation, full-context/parity/corpus-quality benchmark, IDE/projector/teaching review, dependency/service change or catalog write. Instructor review/release and Lesson14 evaluation gaps remain independent.


### Final scoped-approval and validation follow-up — 2026-11-11

Settled note decisions now use the stable kickoff message ID together with the request ID, not a Session-global correlation ID. Five added regressions cover accepted/rejected prior decisions with reused IDs, independent new-proposal validation, and same-attempt pending-response scans. A reused ID cannot suppress a new local decision or make unsettled work compactable. The final inventory is 105 deterministic application/library tests plus seven unchanged seeder tests; both opt-in real-service tests passed again (17.05s library, 16.29s application). Formatting, all-target check/Clippy/tests and build remain warning-free. Complete root and Lesson 13 source trees match byte-for-byte.

A final memory-run retry approved the inspected exact goal and saved `/learned`, then rejected a fresh proposal before escalation. The second attempt proposed another note but hit the unchanged input guard at 96,721 > 84,000; there was no final answer or second local approval, and partial diagnostics survived. This is additional failure evidence, not a successful whole-workflow rerun. A validation-harness artifact-reuse mistake initially ran a root-bound binary instead of the isolated one, created only the known test record (the root record was absent initially), and overwrote diagnostics. That known test record was removed and the pre-test diagnostic file restored; a forced isolated compilation and manifest-root verification preceded the retry. No pre-existing learner record was read or modified. Earlier isolated successes remain separate evidence; no fresh final colored/compaction rerun is claimed.
