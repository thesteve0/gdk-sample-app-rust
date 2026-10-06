# Lesson 7 Instructor Notes: GDK State-Machine Mental Model

## Objective and scope

Teach only enough to explain **one question/answer round trip**. Learners identify Session, Operation, Effect, StateMachine, and re-evaluation. No code, provider demo, or tool trace. The planned-loss round trip is deferred to Lesson 9 by the instructor's 2026-10-05 scope revision.

Use the [lesson](../lessons/07-state-machine-mental-model/LESSON.md) and five numbered images. Do not open the root draft or Goose's full operation list.

## Suggested pacing (25–30 minutes)

- Motivate “what work is needed now?”: 3 minutes.
- Define the five ideas and follow ownership: 7 minutes.
- Introduce the three layers with the section 7.3 table: 2 minutes.
- Explain run/pass and ordered checks: 5 minutes.
- Predict and narrate the simple exchange: 8 minutes.
- Check understanding and preview Lesson 8: 5 minutes.

## Before class

Open the five PNGs in numeric order, full screen. Test readability on the classroom projector. The editable SVGs and rendered PNGs are 1600×900; color is not required to understand them. Lesson captions supply text equivalents.

No provider or build is needed. Asset/source review is not classroom validation or release approval.

## Reveal order and prompts

| Frame | Point at / reveal | Prediction prompt | Expected answer |
|---|---|---|---|
| **01 ownership** | Store and loaded Session first; then Operation, Effects, and handler. Mention the external-input path last. | Who saves the reply? | The application's effect handler saves it to the store. An Operation can also consult live inputs; Steer is just an example. |
| **02 pass** | Follow checks on declines, then the apply/reload path. Briefly name the exits. | Two steps could act. Which wins? Where does checking restart? | First applicable step in configured order; restart at the top after saving. |
| **03 question** | User message beside inference and proposed append Effect. | What must change before the next check sees an answer? | The handler must record the assistant reply. Receiving or displaying it is not enough. |
| **04 saved reply** | New assistant message in the same Session; no completion-status field. | Will inference automatically act again? | No. The machine reloads and checks the updated conversation. |
| **05 stop** | Completed history, inference declining, return to application. | How many provider calls and passes? | One provider round trip; two passes. The second finds no work. |

Between frames 01 and 02, use section 7.3's table to distinguish **Types (vocabulary), Protocol (engine), and Assembly (application choices)**. The takeaway is that we build our own assembly, not Goose's full agent. Keep this to two minutes; skip generics, platform markers, and deployment examples.

Keep the narration concrete: **question saved → inference acts → reply Effect → handler saves → reload → inference declines**.

## Checkpoint answers

1. **Where does conversation live?** In the Session recorded in the application-owned store.
2. **What acts on pass 1?** Inference. **What does it propose?** An Effect adding the assistant reply.
3. **Who saves it?** The application effect handler, called by the machine.
4. **Why stop on pass 2?** The ordinary assistant answer is recorded; inference no longer applies, and this example has no other steps.
5. **What if several steps could act?** The earliest applicable step wins; later steps do not act that pass.

Success means learners can narrate the exchange and separate work from recording. Ask a learner to explain the transition between frames 03 and 05, rather than memorize definitions.

## Keep qualifications short

- Today's machine has only inference. In later assemblies, we put it last as a fallback; the engine does not require that order.
- No-work stopping differs from explicit yield. Do not explore cancellation, limits, or failure paths today.
- Steer's live queue shows that Session history is not every possible decision input. No Steer implementation or full Goose assembly is required.
- In-memory state survives runs only while the application keeps its store alive; it is not disk durability or exactly-once crash recovery.
- Frames are narrated examples and selected Session fields, not observed provider transcripts.

## Source verification and handoff

Curriculum follows the [README](../README.md#replacement-state-machine-lesson-plan) and [Working Mental Model](../reference-material/Goose%20GDK%20State%20Machine%20%E2%80%94%20Working%20Mental%20Model.md), with the instructor's narrowing of Lesson 7 to one simple exchange.

Verified against published `goose-agent` **0.1.0-alpha.11**:

- `src/machine.rs`: application loading/handler interfaces, ordered first-applicable check, apply/reload loop, no-work and yield exits.
- `src/operation.rs`: Operation checking/acting and Effects as data changes.
- `src/inference.rs`: ordinary user-message applicability and no inference after the saved ordinary assistant answer.

The [versioned machine API](https://docs.rs/goose-agent/0.1.0-alpha.11/goose_agent/machine/struct.StateMachine.html) is supplemental. Steer's external-input example was checked in the local Goose checkout's `crates/goose/src/agents/state_machine/ops_steer.rs`; it is not required alpha.11 infrastructure. The official documentation map had no GDK engine page at review time.

Lesson 8 supplies the complete inference-only runtime, streamed Events, and saved Effects. Its supplied implementation consumes Events concurrently and drains on normal channel closure. The instructor chose no loop safeguard, only a production-caution comment beside the machine run call. Lesson 9 introduces the correlated tool trace and its validation/limits. Do not implement either here or reset root Rust.

Instructor-led trace validation, actual projector review, and release approval remain pending.
