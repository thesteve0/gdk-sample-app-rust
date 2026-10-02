# Research spike: teaching the State-machine approach in the GDK

Status: started 2026-10-01. Research-only. None of this material may modify
`lessons/`, `instructor-notes/`, `README.md`, `post-lesson4-plan.md`, or the root
source until the instructor declares the spike complete and orders the teaching
material updated.

## Charter (instructor decisions, 2026-10-01)

1. **Simplest first.** The teaching order starts from the simplest possible use of
   the machine — one request, one response — and introduces operations, effects,
   and sessions only after. Lessons 7 and 8 in `lessons/` are in reverse order as
   they stand; the fix is deferred until this spike concludes.
2. **Runnable experiments, but interactive.** Experiments exist to teach the
   instructor concepts and patterns. Goose explains one concept/step at a time,
   the instructor writes or runs it, and results are reviewed before moving on —
   goose does not build the whole thing start to finish.
3. **Deliverable is spike material only.** What happens to lessons 7/8 and the
   roadmap is decided separately, later.

## Working rules

- Target the pinned `goose-agent = 0.1.0-alpha.11`; verify behavior against its
  source, not memory. Do not call `cargo` with `--all-features`.
- One experiment at a time; each starts with the concept in plain words, then the
  smallest code that shows it, then a run and a look at what actually happened.
- Anything discovered here stays in this directory.

## Candidate experiments (order: simplest first)

1. **The machine runs one request and one response, by itself.** The GDK's own
   `StateMachine::run` drives the loop — no hand-written pass loop. One seeded
   question, one `Step::Inference` (`InferenceRunner`), one reply, one stop.
   Concept: what `run` does that our hand-written loop mirrored, and what the
   machine does on our behalf.
2. **Who holds what.** Store, session loader, effect handler, effect type.
   Concept: the state lives in the store; the machine re-reads it every pass.
3. **A second step.** Register a hand-written operation next to inference.
   Concept: the step list is the agent; `applies()` decides who acts.
4. **A typed tool through the machine.** `ToolOperation`, structured request,
   correlated tool response, `spawn_blocking` in the updated checkout (re-verify
   against pinned source). Concept: the tool execution boundary.
5. **The loop's safety nets.** Step bounds, cancellation, empty responses.
   Concept: what stops a run and why.
