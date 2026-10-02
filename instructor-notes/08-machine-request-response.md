# Lesson 8 Instructor Notes: One Request and One Response through the GDK State Machine

## Teaching objective

Walk learners through the GDK state machine end to end with the smallest possible scenario: one seeded question ("What is the capital of France?"), one registered step (the GDK-shipped `InferenceRunner`), one provider call, one reply, one stop. The lesson opens with a jargon-free plain-words section (the state, the pass loop, where the state lives, what happens to it when the machine finishes) — start there and read it aloud; it is the on-ramp before any vocabulary. This lesson is the introductory walkthrough for the state machine — learners who found Lesson 7's deterministic trace abstract should be able to narrate one pass out loud (reload → ask `applies()` → provider call → effects → stop checks) before reading the code. Define inference step, InferenceRunner, kickoff, provider turn, streaming event/drain, usage effect, and the effect-type requirement in prose before showing their code. The lesson contacts a real provider inside `machine.step`; that is deliberate — the provider call is the machine's, not the application's.

## Suggested pacing

- Framing: the plain-words section (state, pass loop, where the state lives, what happens when the machine finishes) read aloud first, then Lesson 7's deterministic machine versus a provider inside it, then the walkthrough of one pass in prose: 15 minutes
- Diagrams 1–3 (pass loop, who holds what, the two-pass trace): 10 minutes
- The seed, the provider construction, and registering the inference step: 10 minutes
- The effect vocabulary — the three requirements and `ChatEffect`: 15 minutes
- The bounded pass loop and the three stop conditions: 10 minutes
- Payload delimiter, actor labels, and the streamed fence: 10 minutes
- Run live against the provider, walk both passes, review: 15 minutes

## Before class

- Run `cargo fmt --check`, `cargo check`, `cargo clippy --all-targets`, and `cargo test`; all five tests pass without a provider.
- Start or confirm the model endpoint declared in `custom_aa_llama_qwen3_6-35b.json` is reachable. The bundled default needs no key. This lesson's `cargo run` contacts the provider — the first state-machine lesson that does.
- Run `cargo run` yourself first and walk the trace: pass 1 reloads 1 message, `llm` applies, the outbound payload prints fenced (`application → provider`), the reply streams fenced (`provider → application`), the effects print in order (record usage, then append message); pass 2 reloads 2 messages and stops with "No step applies"; the final persisted conversation holds 2 messages.
- Confirm the pinned `goose-agent = "=0.1.0-alpha.11"` ships `InferenceRunner` and `Step::Inference`, and that this lesson adds no dependencies. Re-verify `applies()` semantics against the pinned source before teaching: kickoff found, no trailing error, projected last message non-empty, and the projection ends in a provider turn (`User` or `Tool` effective role).
- Expect the reply text and token counts to differ from the lesson's sample output on every run; the structure (fences, labels, effect order, stop reason) is what to grade.

## Discussion prompts

- Walk one pass out loud: what happens between `machine.step` and `machine.apply`, and what is in the store before and after? Where exactly does the provider get called?
- Why does the same inference step apply on pass 1 and decline on pass 2? Point at the exact history entry — the persisted reply — that changes its answer.
- Who decided the run should stop: the application, the model, or the history? (The history, through re-derivation — the same answer Lesson 7's operations gave.)
- What is the kickoff, and why does the runner re-derive from it instead of remembering that it already replied?
- What is the difference between an event and an effect? If the client never drained the channel, what would the store contain? (Exactly the same — events are presentation, effects are state.)
- Why is token usage an effect rather than a print inside the runner? What does that say about where cross-cutting records should travel?
- Name the three requirements the effect type must satisfy and which component requires each. What fails to compile if one is missing? Why does the default `ConversationEffect` not suffice in this pinned release?
- Why does the machine call `ensure_message_ids` when the emitter already generated ids for streamed messages?
- Why is the event channel 4096? What would go wrong with a small buffer while the step is still running?
- Why does the provider receive no system prompt here? What collects prompt parts and tool definitions, and what does that predict about Lesson 9?
- Which of the three stop conditions fired? Under what conversation state would each of the other two fire?
- What happens to the state when the machine finishes? Where does the conversation sit while the application waits to see whether the user asks a follow-up, and who starts the machine again if one arrives? (The store, under the session id; the application appends and re-runs.)
- Where does the payload fence open and close for the streamed response, and why can each delta not carry its own fence?

## Live-demo cautions

- The provider must be reachable before class. If it is down, the deterministic unit tests still pass, but do not demo the trace without a live provider and do not mock the call — ask to start or supply one.
- Model text varies run to run. Do not grade wording; grade structure: two fenced payload blocks with the correct actor labels, effects persisted in usage-then-reply order, "No step applies" on pass 2, and 2 messages persisted.
- The outbound payload is printed after the step returns, quoting the snapshot reloaded at the start of the pass — point at the pass header to show the reload, then the payload to show what the stateless provider was sent (one message, no system prompt, no tools).
- The streamed fence opens at the first text delta and closes after the stream ends — one fenced block per response. Point at it when discussing why individual deltas cannot carry fences.
- The effect order (usage, then reply) is fixed by the shipped runner's return order; do not present it as configurable.
- `MAX_STATE_STEPS` is 4 and never fires here. Changing it to 1 makes the bound fire after pass 1's persist — a useful one-off demo of the bound, but restore 4 afterward so the default stop reason stays "no step applies".
- Do not demo edits mid-class: registering a second step, seeding a tool request, or removing the persist changes which stop condition fires. Use the committed tests, which cover the declining step and the bound deterministically.

## What this lesson must not do

- No tool calls, no tool definitions, no `ToolOperation`, no second step, and no `StateMachine::run` — all of that is Lesson 9.
- No multi-turn follow-up: letting the user ask another question and watching a second provider turn was considered and deliberately deferred by instructor decision. The lesson stays exactly one request and one response.
- No system prompt or domain instructions — the provider receives only the conversation; instructions are Lesson 10.
- No usage persistence beyond printing: the usage effect travels through the effect list, but a later lesson decides where it is stored.
- No production storage: the store stays an in-memory map; the session type and effect vocabulary extend Lesson 7's, they do not replace it.
- No mocks in place of the live provider call; if the endpoint is unreachable, ask for one rather than substituting a fake stream.

## Checkpoint

Learners can narrate one pass end to end (reload → ask `applies()` → provider call → effects → stop checks), point at the re-derivation that makes the same inference step apply on pass 1 and decline on pass 2, name the three effect-type requirements and which component needs each, distinguish streamed events (presentation) from effects (state), and read both payload fences with their actor labels — including why the streamed fence opens at the first delta and closes after the stream.

## Transition to Lesson 9

Lesson 9 extends the step list and the protocol at once: a typed deterministic tool registered with the machine alongside inference, so one machine run asks the provider, receives a structured tool request, executes the tool through the machine (`ToolOperation`; in the current checkout the typed sync tool is invoked through `tokio::task::spawn_blocking` — re-verify against the exact pinned source), and returns the correlated result; the lesson also hands the pass loop to the GDK's own `StateMachine::run`, which loads sessions and applies steps without this lesson's hand-written bound loop. Keep the store, the session type, `ChatEffect`, and the pass-loop shape as they stand — Lesson 9 grows the step list and moves the loop, not the persistence contract.
