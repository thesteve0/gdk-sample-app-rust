# Lesson 7 Instructor Notes: The GDK State Machine over the Lesson 6 Conversation

## Teaching objective

Hand the coordination of the Lesson 6 conversation to the GDK state machine. Define persisted conversation state, session, operation, effect, session loading, effect application, yield, and state-step bound in prose before showing their code-level forms. Learners should be able to point at the reload on every pass, at the operation that declines because its answer is already in the history, and at the three ways the machine stops. The lesson runs with no provider and no live inference: the provider returns in Lesson 8 as an inference step.

## Suggested pacing

- The higher-level framing — state and rounds of message passing, Lesson 6's single state holder versus persisted session state: 15 minutes
- Diagrams 1–3 and the mapping table: 10 minutes
- The session, the store, and the seed conversation: 10 minutes
- Session loading and effect application as the machine's whole persistence interface: 15 minutes
- The re-deriving operation (`answer_pending_tools` declines on pass 2): 15 minutes
- Yield, termination, step registration, and the bounded pass loop: 15 minutes
- Run the deterministic trace, walk the two passes, and review: 10 minutes

## Before class

- Run `cargo fmt --check`, `cargo check`, `cargo clippy --all-targets`, and `cargo test` against the reference solution; all seven tests pass without a provider.
- Run `cargo run` and walk the expected trace: pass 1 answers `call_seed_001` with `$100.00` and persists one effect with effective role `Tool`; pass 2 declines and yields; the final conversation holds 3 messages.
- Confirm the lesson compiles with the pinned `goose-agent = "=0.1.0-alpha.11"` and that no `.env` or provider JSON is needed — this lesson runs anywhere, which makes it a good fallback if the classroom provider is down.
- Skim the GDK state-machine README (crates/goose-agent/README.md in the goose repository) for the canonical phrasing: "the whole agent's behavior is a function of the persisted conversation, not of in-memory loop state."

## Discussion prompts

- In Lesson 6, who held the conversation, and what happened to it when the program ended? What does "persisted" mean in this lesson's store?
- Why does the loader clone the conversation on every pass instead of the machine keeping one copy? What would go wrong if a pass mutated a cached session?
- Where is the decision "is this request answered?" made, and what data does it read? Why must it re-derive instead of remembering what happened on the last pass?
- Why does the same `answer_pending_tools` operation apply on pass 1 and decline on pass 2? Point at the exact history entry that changes its answer.
- What is an effect, and why can an operation not push to the conversation directly? Who decides whether an effect is persisted?
- Why does this minimal runtime reject `ReplaceConversation`, `PatchToolRequestMeta`, and `SetMessageVisibility` instead of implementing them? What does rejecting them demonstrate about the effect enum?
- What are the three ways this machine stops? Which one fires in the deterministic trace, and why is the bound (3) never the reason?
- A machine registered with only `answer_pending_tools` ends pass 2 with "no operation applies" — what does that say about step registration being data? What agent is that list, compared to the two-step list?
- Why is the yield step an operation with its own re-derivation (`last_effective_role`) rather than a flag the first step sets?
- Why does the lesson write the pass loop by hand instead of calling `run`? Where does the GDK's own `run` contain this same loop, and what does it take that this loop does not?
- The tool result text lives inside the `ToolResponse` content block — why can `as_concat_text()` not see it, and where would a naive reader lose the `$100.00`?
- Which Lesson 6 boundaries survive unchanged into the machine, and which piece of Lesson 6 code actually disappeared?

## Live-demo cautions

- This lesson contacts no provider. If someone expects streaming output, remind them the provider returns in Lesson 8; the deterministic trace is the demo, and it is identical on every run.
- The seeded tool request is replayed protocol payload — point at the `++++++++` fences and the actor labels (`application → provider, replayed`; `provider → application, replayed from Lesson 6`; `application, as the tool → provider`) so the payload/commentary boundary stays consistent with Lessons 5–6.
- Changing `MAX_STATE_STEPS` to 2 makes the bound fire and the run end after pass 1's persist — a useful demo of the bound, but run it once only; the default 3 keeps the yield as the stopping reason.
- The pass loop reloads the session before each `machine.step`; the printed `── Pass N: reload session 'lesson-07' ──` lines are the reload evidence. Point at them when discussing why the machine never caches state.
- The "persisting effect" line prints the message's effective role (`Tool`), not its stored role (`User`) — the same Lesson 6 convention, now observed at the persistence boundary.
- Do not demo edits mid-class: swapping step order, removing the yield step, or raising the bound changes which stop reason fires. If you want to show it, use the committed tests, which assert both step-list shapes.

## What this lesson must not do

- No typed `SyncTool`, no `ToolOperation`, and no `StateMachine::run` — the typed tool and the crate's run loop return in Lesson 9. The provider does arrive in Lesson 8 as a `Step::Inference` inference step, but only as the one-request/one-response walkthrough: the GDK-shipped `InferenceRunner` is the sole registered step, with no tool calls.
- No production storage: the store stays an in-memory map; files or a database are a later lesson's swap.
- No second tool and no allowlist growth: the tool path is Lesson 6's single deterministic calculation, unchanged.
- No market data, retrieval, model-selection CLI, or cancellation demonstrations beyond constructing the token.
- No binary floating point anywhere in the currency path; Lesson 6's exact-decimal discipline carries over.
- No new effect kinds implemented in the runtime; the lesson rejects everything but `AppendMessage` on purpose.

## Checkpoint

Learners can state the machine's pass shape (reload → ordered operations re-derive → first applicable emits effects → persist → repeat), point at the re-derivation that makes the same operation apply on pass 1 and decline on pass 2, name the three stop reasons and which one fired, and explain why the effect enum and the step list are data boundaries rather than conveniences.

## Transition to Lesson 8 and Lesson 9

Lesson 8 (now implemented) walks the machine through one simple request and response: the GDK-shipped `InferenceRunner` is registered as the sole `Step::Inference`, so the provider re-enters inside the machine exactly where `yield_to_client` fires today. The provider is called once, in pass 1; the streamed reply is persisted as effects; and pass 2 finds no step that applies and stops. Lesson 9 then registers the typed deterministic tool alongside inference (`ToolOperation`; its invocation runs through `tokio::task::spawn_blocking` in the updated checkout — re-verify against the pinned source) and hands the loop to `StateMachine::run`, which loads the session and applies steps without the hand-written bound loop this lesson mirrors. Keep the store and the session type as they stand — Lesson 8 keeps the in-memory persistence shape and the hand-written bounded pass loop; the step list gains an inference step and the lesson defines its own `ChatEffect` because `ConversationEffect` has no `InferenceEffect` impl in the pinned release.
