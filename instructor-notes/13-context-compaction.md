# Instructor notes — Lesson 13: Context compaction

## Aim and handoff status

Teach one transferable boundary: shortening model working context must not silently change application facts, persistent-note authority or genuine learner evidence. Use [the lesson](../lessons/13-context-compaction/LESSON.md), complete supplied reference/helpers and editable summary guidance from the outset. Attendees predict → run → explain → tweak → run again; they do not rebuild async infrastructure or a summarizer.

The complete authorized implementation is supplied. See the lesson validation record for checked behavior and remaining gaps. The supplied limits are 100,000 serialized input bytes, 60,000-byte default trigger via `COMPACTION_TRIGGER_BYTES`, 6,000-byte summary with a 4,096-token generation cap and one 180-second deadline plus at most five seconds of cancellation grace. Do not treat these limits as evidence of summary fidelity or exact provider capacity. These notes describe the preserved Lesson 13 reference, not Lesson 14 instrumentation. Lesson 14 is separately authorized and implemented; the combined modularization pass ultimately leaves the ephemeral root matching its complete tree. Historical planning-only restrictions are not a current claim that Lesson 14 is unauthorized.

## Before class

1. Read README/AGENTS, Lesson 13 PLAN/handoff, complete Lesson 12 materials/content and the actual Lesson 13 source. Preserve unrelated edits. Confirm published alpha.11 SDK pins rather than an unreviewed local checkout.
2. Use [Lesson 12 DATABASE-SETUP](../lessons/12-specialized-retrieval/DATABASE-SETUP.md). Verify the existing local tutor, Harrier runtime/profile, provisional `harrier-llamacpp-test-511-v1` collection and reader role. No reload, new source corpus, model conversion or database lesson is needed. Keep source rights, disclosure and service exposure checks; a loopback client does not fix an all-interface server.
3. Activate the complete recursive reference as shown below, then launch with the lesson's root command and existing private `.env`/reader setup. Never project credentials, environment connection strings or shell tracing. Services unavailable? Ask for them; do not relabel mock output as integration evidence.
4. Confirm final code retains original opening/latest two complete genuine learner exchanges, tool pairs/IDs/metadata; isolated tool-free GDK summary inference; one assistant-derived summary; revision-checked replacement; real-reply-only proposal quotation; settled accepted/declined approvals; all-inference budget guard and no repeated compaction loop. Locate the full runtime application boundary, not just an enum variant.
5. Confirm `/learned` is local, consent-gated, nonblank single-line ≤200 UTF-8 bytes, duplicate-suppressed within the same Session, and ephemeral (no disk access) with memory disabled. Its deterministic completion category is not model-proposable. Confirm no new welcome/unsolicited answer. A saved declaration is not rolled back on a later compaction failure.
6. Rehearse a lower valid trigger above the actual static-context baseline, with enough room for safe protected history. Measure how many ordinary exchanges are needed; supply that value to class rather than a universal guessed threshold. Keep the 100,000-byte hard budget unchanged. Do not deliberately overflow providers or break services as a learner exercise.
7. Run actual checks and live continuation, inspect summary meaning, consent/storage, tool and UI regressions. Prepare a permissioned sanitized diagnostic view. Diagnostic files contain learner text/private excerpts and are overwritten at launch; preserve any approved evidence privately before a new rehearsal.

## Source organization and activation

A **Rust module** groups related code under a local name so already-taught mechanics and supplied support do not obscure the new boundary. Define it before showing a declaration. In `main.rs`, for example:

```rust
mod calculator;
mod chooser;
mod diagnostics;
mod learning_record;
mod presentation;
mod provider_setup;
```

These are the six inherited support modules, not a shared library or new framework. Main also declares the cohesive compaction/retrieval modules and test-only application tests. A **child test module** is a separate test source owned by its production module: `#[cfg(test)] mod tests;` at the end of `compaction.rs` loads `compaction/tests.rs` for tests only. Every Rust source starts `// Lesson 13`.

Use the lesson's [complete linked source map and teaching reading path](../lessons/13-context-compaction/LESSON.md#supplied-module-map-and-teaching-reading-path). For instructor navigation, the complete tree is:

- [main.rs](../lessons/13-context-compaction/src/main.rs): context assembly, commands, provenance/authoritative state, preflight/revisions/maintenance wiring, Session/runtime/Effects, concurrent Events, dispatch/limits, evidence/registration/citation inspection and approval lifecycle.
- [compaction.rs](../lessons/13-context-compaction/src/compaction.rs) and [compaction/tests.rs](../lessons/13-context-compaction/src/compaction/tests.rs): cohesive compaction/declaration policy and child tests.
- [retrieval.rs](../lessons/13-context-compaction/src/retrieval.rs) and [retrieval/tests.rs](../lessons/13-context-compaction/src/retrieval/tests.rs): cohesive retrieval and child tests, preserving cancellation/Drop guards.
- [calculator.rs](../lessons/13-context-compaction/src/calculator.rs): exact-decimal tool/schema/validation and local tests.
- [chooser.rs](../lessons/13-context-compaction/src/chooser.rs): selection/input/raw-mode cleanup.
- [diagnostics.rs](../lessons/13-context-compaction/src/diagnostics.rs): complete plain diagnostic rendering and local tests.
- [learning_record.rs](../lessons/13-context-compaction/src/learning_record.rs): record types/schema/storage/confirmation-input mechanics and local tests.
- [presentation.rs](../lessons/13-context-compaction/src/presentation.rs): terminal gates/styles/`TerminalProse` and local tests, not Event consumption.
- [provider_setup.rs](../lessons/13-context-compaction/src/provider_setup.rs): path/model selection support.
- [tests.rs](../lessons/13-context-compaction/src/tests.rs): main's application-boundary tests, including the opt-in same-snapshot live search test.

Read main's context/selection/reply flow first, then Session/runtime and genuine reply/revision ledgers. Follow the preflight and maintenance call sites into compaction's projection/partition/isolated inference/replacement checks and return to the runtime Effect boundary. Next trace `run_turn`/`cooperative_run` and concurrent `display_events_bounded`, including sender closure/draining and partial diagnostics. Then inspect main's tool registration, shared evidence budget, allowlists/limits and current-turn citations. Finish with `parse_record_proposal` and successful run/inspection → default-no approval → decision log → approved append call → Session refresh/revision → settled decision/revision → post-approval diagnostics. Those decisions remain visible in main; support modules do not acquire authority. Read tests as evidence, not scaffolding exercises.

From the repository root, replace only the ephemeral exercise tree with **all files recursively**, not `main.rs` alone:

```bash
rm -rf src
mkdir src
cp -R lessons/13-context-compaction/src/. src/
```

No backup/restore or final hello-world reset is required. Preserve unrelated work outside the exercise tree and all authoritative references. Child test directories are required. Root ultimately advances to the separately authorized Lesson 14 after the combined pass; do not mistake the active root for the preserved Lesson 13 reference.

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

Emphasize: original opening and latest two complete genuine learner exchanges remain with tool pairs/IDs/metadata. Older summary plus newly eligible history become one derived summary. Static instructions, teaching guidance, consented record and tools remain separately counted. Summary-generation tools are absent; generated text is data, not stronger instructions.

A state revision guards against stale application. Failure leaves working history intact; if `/learned` already saved, report that honestly rather than silently repeat storage. Local approved/declined proposals are settled; compaction must not reopen either. Storage is still single-process, not transactional or crash-exactly-once.

### 4. Observe compaction and continue (10 minutes)

Use the rehearsed trigger; continue ordinary short learning replies until older exchanges are eligible. Do not imply two replies must always trigger it. Watch for a brief learner-facing notice, then privately inspect counted before/after input, selected source history, generated summary and applied replacement. If it did not compact, explain the actual threshold/eligibility evidence.

Ask: “What question was I still unsure about, and what small activity did I agree to?” Check against actual words; a suggested activity is not accepted. Correct any summary strengthening. The conversation continues in the same Session, not a new welcome. The command itself should not elicit an unsolicited answer.

Optionally run entry 51.20/stop 50.70/200 shares: the exact calculator result is $100.00 maximum **planned** loss, excluding fees, slippage and gaps, not expected loss or guaranteed realized loss. Observe actual tool selection. When summarized later, keep the qualifications, not only the number.

Ask a new book-grounded liquidity/spread question. Predict fresh search before current citations. Historical IDs in a summary do not preserve current excerpt support. Inspect returned ID membership separately from whether each passage supports its adjacent claim.

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

The summary is ≤6,000 ordinary-text UTF-8 bytes with a 4,096-token output request cap; provider thinking metadata is inspected but never inserted as summary text. One attempt has a 180-second deadline plus ≤5-second cooperative cancellation grace; hard-aborted received usage can be unavailable. Ordinary tutor runs retain four shared requests/eight applied work passes plus possible halt, with the same180-second deadline/5-second grace and explicit4096-token/65536-text-byte/2000000-Event-byte output caps. Record compaction work/usage separately, do not hide it as a free second inference path. Automatic safe-boundary compaction is not an unlimited retry loop. Oversized static/protected context causes a controlled stop, not another forced summary or silent deletion. Cancellation is not rollback or proof the service stopped. Per-turn retrieval's 4,000 actual-token evidence cap remains separate from accumulated working context.

## Privacy, diagnostics and inherited limitations

Learner UI retains clean Tutor/Learner dialogue, readable citations, brief status and presentation-only course palette/gating. Raw summaries, IDs, requests, Session dumps and actor/fence protocols stay out of ordinary dialogue. Verify NO_COLOR/redirection/TERM=dumb and independent stderr fallback; common Markdown suppression is not a general renderer. Provider/saved text must remain unchanged.

`session-output.md` is overwritten plain private diagnostics, never memory. Include full fields/defaults/metadata/usage and loaded context, counted sizes, source coverage, summary/replacement/revision decisions, settled approvals, declarations, complete tools, answers and available partial failures. Exclude deliberate credential/configuration/secret-URL/vector dumps. Ignored files remain readable to anyone with filesystem access; sanitize and obtain consent before projection. Log/storage failure stops; unavailable logging cannot promise to record itself.

Memory remains optional per selection and hand-editable between launches. `/finish`, `/quit`, EOF and blank replies keep their meanings. No new file/network/SQL/order authority, corpus freeze, model parity, expert rubric or clinical mastery assessment appears here.

Do not erase Lesson 12 findings: source-scope and feedback relationship wording defects despite valid IDs; unsupported-price refusal without search; hand-edited activity transported but not offered in welcome; an initial search with no final ordinary answer. The 511 passages are provisional, not all books or an accepted corpus. Earlier review/validation gaps remain independent. Compaction does not certify those problems fixed.

## Source evidence and validation record

For documentation, checked the locally cached published `goose-agent 0.1.0-alpha.11` `operation.rs` (Operation, ConversationEffect/ReplaceConversation) and `machine.rs` (load → step → apply → reload run). Official [Goose smart context management](https://goose-docs.ai/docs/guides/sessions/smart-context-management) documents full Goose, not an automatically supplied policy for this application. A later source spot-check confirmed 16,000-byte headroom (84,000 estimated-input allowance), at least 8,000 eligible older serialized bytes, and replacement reduction of at least 1,024 bytes and 10%. It confirmed declined-consent declarations are ephemeral, not rejected; plain-word/simple-punctuation validation applies. Bounded completion/provenance/settled-decision ledgers require final review (currently 128 completions and 256 entries for each other ledger). Inspect the supplied main/compaction modules before class; bounded ledgers do not confer crash recovery or across-launch deduplication.

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
