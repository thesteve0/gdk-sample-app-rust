# Goose GDK State Machine — Working Mental Model

Oct 5, 2026 · @Steve Pousty

## How to use this doc

This is a high-level mental model of the goose GDK state machine, written as Steve's understanding with corrections folded in. It deliberately skips the internals of Session, Operation, and Effect; each section can seed a deeper follow-up conversation.

**Provenance.** Claims were checked against source in [aaif-goose/goose](https://github.com/aaif-goose/goose), upstream `main` as of 2026-10-05. A fork checkout (`thesteve0/goose`) was at a different commit, but the files below were identical to upstream.

**Confidence labels.** Unmarked statements were read directly in the code. Statements marked *(unverified)* are reasoned or inferred and are listed again under Open questions.

**Pasting into a new chat.** Paste this whole doc, then name the section to dig into. The source map at the end gives file paths to start from.

## The big picture

The state machine is an if/else-if chain inside a loop, evaluated against a persisted conversation log. Each pass, it asks an ordered list of Operations, one by one, "do you have work?" The first that says yes does its work and returns Effects (proposed changes). The machine persists those Effects, reloads the session, and starts again from the top.

When nothing has work, or an Operation says "yield," the run ends and control returns to the application. Calling the model (Inference) is the last branch: it happens only when nothing else needs doing.

**Best analogy:** event sourcing, or a Redux-style reducer. The history is the source of truth, Operations are rules that read it, and Effects are the changes they propose.

**Behavior depends on history plus live state, not history alone.** Operations are built holding references to application state that is never persisted: the steer queue, the current goose mode, the extension manager, hooks, and config values. So the next decision depends on two inputs, the persisted history and whatever that live state says right now.

What this does and does not weaken:

- **The past stays fixed.** Whatever an Operation decided using live state is recorded as effects, and operation notes record past actions. Re-reading the history after a stop reproduces every decision already made.
- **The next decision can change.** After a stop and resume, the same history can produce a different next pass if live state changed in between. Examples: the user switched goose mode, an extension was removed, or the steer queue is now empty.
- **Live-only data can be lost.** The steer queue is held only in memory. A message typed mid-run that Steer has not yet drained is not in the session, so a process crash at that moment loses it.

Restartability between passes still holds, but in a precise form: a resumed run picks up from the same history, not from the same complete state.

## The four pieces

| Piece | What it is | What it is NOT |
| --- | --- | --- |
| Session | The persisted state of a user's session: the conversation (an append-only log of messages) plus metadata such as recipe, usage, and working directory. Lives in a store owned by the application. | Not a registry of Operations or their statuses. The machine never holds it between passes. |
| Operation | A rule. Its `run()` checks the conversation and either returns `NotApplicable` or does its work and returns `Applied(effects)`. Check and act are one call. | Not a queued task. Not something whose status is stored anywhere. |
| Effect | A value describing a proposed data change: append a message, replace or compact the conversation, patch tool-request metadata, change message visibility, set the recipe, or record usage. | Not a status change on an Operation. Operations never mutate the Session directly. |
| StateMachine | Built once with an ordered list of steps (Operations, then Inference last). `run(session_id)` loops: load, step, apply. | Not given the Operations in `run()`, and not given the Session object, only its ID. |

**Where "status" really lives.** It is implied by the shape of the history. ToolExecution has work when a tool request has no matching tool response; MaxTurns when a turn count hits its limit; Compaction when the conversation crosses a size threshold. The one stored hint is *operation notes*: small metadata an Operation stamps on a message (for example, which tools it advertised), recording past actions only.

**Operations have a second role.** Before the model is called (the Inference Operation), every Operation in the list may contribute tools, prompt parts, and extra context to the request. They both act on the history and shape what the model sees.

## The loop

One call to `run()` is many passes (rounds). Each pass applies exactly one Operation's effects, then everything is re-evaluated from the top.

```
machine = StateMachine.new(ordered_steps, cancel)   # Operations fixed here

run(session_id):
  loop:
    session = load(session_id)                 # reload every pass; never cached
    if   entry_hook.has_work(session):    effects = entry_hook.act()
    elif slash_command.has_work(session): effects = slash_command.act()
    ...
    elif tool_exec.has_work(session):     effects = tool_exec.act()
    ...
    elif inference.applies(session):      effects = inference.act()   # last branch
    else: break                           # nothing had work -> run ends
    apply_and_persist(effects)            # the store commits the changes
    if effects.yield_to_client: break     # an Operation said "stop"
```

In the real code, each `has_work` and `act` pair is a single `run()` call returning `NotApplicable` or `Applied(effects)`.

**A run ends when:**

- no step has work, or
- an applied result sets `yield_to_client`, or
- the run is cancelled (cancelled steps report `NotApplicable`, get a `cancel()` hook, and force a yield).

*(partly verified)* For an ordinary final answer, the log ends on the model's own message, so Inference no longer applies. If no other step has work, the run ends because nothing applied, not because of `yield_to_client`. Not yet ruled out: StopHook or Retry acting at that point.

**Effects are applied in three phases.** First, message IDs are assigned and usage is filled in. Second, each effect is persisted in order. Third, events are emitted to the app. Some events wait until after persistence, so a fast response can never arrive before the request it answers is saved.

## The Operation order

Goose evaluates 18 steps in this fixed order, assembled in `crates/goose/src/agents/agent.rs` (around lines 1740–1827). Earlier steps get first say every pass: ToolApproval runs before ToolExecution, and MaxTurns is checked before the model is called again.

| # | Step | Role | Basis for role |
| --- | --- | --- | --- |
| 1 | EntryHook | Runs entry hooks at the start of a turn | Name and constructor |
| 2 | SlashCommand | Handles `/commands`; dispatches to every Operation below plus Status | Code read |
| 3 | Steer | Injects queued steering input; reads a queue given at construction, not only the session | Constructor |
| 4 | MaxTurns | Enforces the turn limit | Code and issue #12582 |
| 5 | BangShell | Likely handles `!` shell commands | Name only |
| 6 | Compaction | Compacts the conversation past a size threshold; included only if the provider does not manage its own context | Code read |
| 7 | ToolPairCompaction | Summarizes older tool request/response pairs past a cutoff | Constructor |
| 8 | ToolApproval | Gates tool calls needing approval | Code read |
| 9 | Doctor | Unknown | Name only |
| 10 | Project | Unknown | Name only |
| 11 | Skill | Skill handling | Name and constructor |
| 12 | Recipe | Recipe handling (likely the source of SetRecipe effects) | Name and constructor |
| 13 | ToolExecution | Runs all unanswered tool requests in one pass | Code read |
| 14 | UnknownTool | Answers requests for tools no extension provides | Inferred from ToolExecution |
| 15 | Retry | Retry and goal logic with timeouts | Constructor |
| 16 | StopHook | Runs stop hooks, with a cap on blocks | Constructor |
| 17 | ExitOnError | Ends the run on error | Name only |
| 18 | Inference | Calls the model; the final fallback branch | Code read |

"Name only" and "Constructor" rows are educated guesses; each is a candidate for a deeper dive.

## Worked example: the model asks for four tools

Four tool calls become one Operation pass, not four Operations. No status is flipped anywhere: appending a message is what creates the next pass's work.

1. **Pass N: Inference.** Nothing earlier had work, so the model is called. It replies with one assistant message containing four tool requests. Inference returns one effect, `AppendMessage`, which is persisted.
2. **Pass N+1: walk from the top.** ToolApproval may act first if any call needs approval. Otherwise ToolExecution acts. Its check reads more than the last message:
   - It scans the whole current turn (everything since the user's kickoff message) for tool requests with no matching response.
   - It reads an operation note confirming the model was actually offered those tools.
   - It reads the current goose mode, which lives outside the session. In Chat mode it answers each request with "skipped" instead of running it.
3. **Inside that one ToolExecution pass:**
   - Each request gets a disposition computed from history: Execute, Decline, or ParseError.
   - Execute calls are dispatched; their results are merged and gathered concurrently by design.
   - All four results go into one new message with the user role. Declined, unparseable, or interrupted calls get a canned error, because the model must never see an unanswered request. Approval requests are the exception: they become separate messages.
   - One batch of effects is returned and persisted.
4. **Pass N+2: walk from the top again.** ToolExecution finds nothing unanswered and returns `NotApplicable`. Earlier steps get first say:
   - **Steer** (step 3) checks two things: the log is between turns (it now ends on tool results), and its in-memory queue holds a message the user typed while the tools ran. If both are true, it appends that message, and Inference sees it next pass alongside the tool results.
   - ToolPairCompaction, Retry, or StopHook may also act.
   - If none has work, Inference applies and sends the tool results to the model.
5. **Why Inference fires.** Nothing marks the message "unanswered." Inference's trigger is a rule about the end of the log: the last message the model would see is from the user or a tool, is not empty, and is not an error. It is the same rule that fires when a human types something.

Which tools are called, and how many, is always the model's choice. The machine only decides which Operation acts on the result.

## A multi-turn conversation

Each user turn is one call to `run()`, which contains one or more passes. Over a long conversation, `run()` is called many times against the same session.

1. The user sends input to the agentic application.
2. The input is added to the session in the store. *(unverified: exactly where and by whom.)*
3. The application builds a StateMachine with the allowed Operations and calls `run(session_id)`.
4. `run()` loops, reloading the session each pass, until a stop condition is reached.
5. Throughout the run, the Emitter streams events (messages, notifications, usage) to the application, so the user can see progress before the run ends.
6. `run()` returns the final session, and control goes back to the application.
7. The user sends new input.
8. Repeat from step 2. The session now holds everything from earlier turns plus the new input.

*(unverified)* Because the Operation list is given at construction, different turns could use different lists. Goose already varies it once (Compaction is conditional). Caution: operation notes left by earlier runs may interact with a changed list.

## Corrected misconceptions

These came up while building this model. Each is an intuitive wrong turn worth calling out when teaching.

| Intuition | Correction |
| --- | --- |
| The Session holds all Operations and their statuses. | It holds the conversation log plus metadata. Work is implied by the shape of the history. |
| The Session is a key-value store: an Operation key with no value means "run it." | True only for tool calls (request ID without a response ID). Other Operations use other triggers: counts, size thresholds, external queues. |
| The machine checks which Operations need action, then runs one. | Each Operation checks for itself inside its own `run()`. The machine only sees `NotApplicable` or `Applied`. |
| Each pass, the machine evaluates every Operation against the session metadata and the last appended message. | It stops at the first Operation with work. Each Operation reads whatever it needs: the whole current turn, message notes, session metadata, or live application state outside the session, such as Steer's queue or the current goose mode. |
| Effects change an Operation's status. | Effects change data (messages, conversation, metadata, usage). The next pass's work follows from that data. |
| Operations are passed into `run()`. | They are passed to `StateMachine::new()`. `run()` takes only a session ID. |
| The application holds the Session object for the whole session. | The application owns the store; the machine reloads the session from it every pass. |
| The Operation list is a palette, picked situationally. | It is an ordered priority list; the first Operation with work wins each pass. |
| Four tool calls become four Operations. | One ToolExecution pass handles all of them. |
| Operations only gate whether another round happens. | Tool dispatch is itself an Operation, and every Operation can shape the model request. |

## Open questions

- [ ] Partly answered: a plain final answer ends the run because nothing applies (`InferenceRunner::applies` in `crates/goose-agent/src/inference.rs` needs the log to end on a user or tool message). Still to check: does StopHook or Retry act first? Start with `rg "yield_to_client|yielded_with" crates/goose/src/agents/state_machine/`.
- [ ] Where is the user's message written to the session before `run()` is called?
- [ ] Is the StateMachine rebuilt on every reply? Trace the callers of the builder in `agent.rs`.
- [ ] Do tool calls really execute concurrently, or does any extension serialize internally?
- [ ] What do Doctor, Project, BangShell, and ExitOnError do?
- [ ] How do operation notes behave if the Operation list changes between turns?

## Source map

The three layers separate a shared vocabulary, a generic engine, and one specific application. Dependencies run one way only: `goose` depends on `goose-agent`, which depends on `goose-provider-types`. Nothing points back up (checked in each crate's `Cargo.toml`).

- **Types: the vocabulary.** Shared definitions such as Message and Conversation, plus the `MaybeSend`/`MaybeSync` markers that let the same code compile natively and for wasm. Like a dictionary: everyone uses the same words, but it does nothing on its own.
- **Protocol: the engine and its rules, with nothing goose-specific.** The `Operation` trait, `StateMachine`, and the plug-in points `EffectHandler` (how effects are saved) and `SessionLoader` (how a session is loaded). It is generic: `Operation<S, E>` does not know goose's Session type or any effects beyond the basic `ConversationEffect`, and it has no notion of recipes, MCP extensions, or goose's database. Like the rules of a board game, without the pieces.
- **Assembly: goose's specific agent.** Goose fills every blank the protocol leaves open: a concrete Session, `GooseEffect`, `SessionManager` for loading and saving, the concrete Operations, and their order in `agent.rs`. The actual pieces and setup for one particular game.

**Why the split matters: you can build a different assembly.** Two examples in the repo:

- **The wasm test** (`crates/goose-agent/tests/wasm.rs`) uses only the protocol crate and defines its own `Effect` enum: a tiny agent with no goose assembly at all.
- **Issue #12476** describes running `goose-agent` inside a Cloudflare-style Durable Object. The conversation lives in that object's own SQLite, with its own `SessionLoader` and `EffectHandler`: a different store and environment, the same engine.

So "GDK" in the sense of a kit mostly means the protocol layer. Goose is one application assembled from it, and you could assemble your own.

| Layer | Crate | What lives there |
| --- | --- | --- |
| Types | `goose-provider-types` | `MaybeSend` and `MaybeSync`, plus conversation and message types |
| Protocol | `goose-agent` | `Operation`, `Inference`, `StateMachine`, `EffectHandler`, `SessionLoader`, `Emitter`, `ConversationEffect` |
| Assembly | `goose` | `GooseEffect`, `SessionManager`, the concrete Operations, and the step order |

| Topic | File |
| --- | --- |
| Operation trait, ConversationEffect | `crates/goose-agent/src/operation.rs` |
| step(), apply(), run() | `crates/goose-agent/src/machine.rs` |
| GooseEffect | `crates/goose/src/agents/state_machine/effects.rs` |
| Effect application and session loading | `crates/goose/src/agents/state_machine/session.rs` |
| ToolExecutionOperation | `crates/goose/src/agents/state_machine/ops_toolcalling.rs` |
| Step order | `crates/goose/src/agents/agent.rs`, around lines 1740–1827 |
| Example: bring your own Effect type | `crates/goose-agent/tests/wasm.rs` |
| Example: why Replace and Compact differ from Append | `live_replacement_preserves_concurrent_transcript_and_hidden_handoff` in `session.rs` |
