# Lesson 14 plan — Evaluate the complete agentic application

**Planning only — 2026-10-11. No implementation authorized or supplied.** Read [the shared handoff](../final-lessons-handoff.md). This is the final required lesson, not a simulator or deployment lesson.

## Goal and concepts

An **evaluation case** supplies inputs and criteria for judging application behavior. An **evaluation set** groups representative cases. A **baseline** is recorded behavior before a change; a **regression** is previously acceptable behavior becoming worse. Diagnostics show what happened; evaluation asks whether it was acceptable.

Teach evaluation of the whole application: capabilities selected, validated actions, evidence use, uncertainty, compaction, and learner-controlled persistence—not final prose alone and not private chain-of-thought. A tutor's assessment of a learner is different from our evaluation of the tutor.

## Limits of domain expertise

The instructor has books but no expert-authored assessment rubric or claim to trading expertise. Do not invent expert gold-standard answers, certify mastery, or score readiness to trade. Evaluate inspectable source support, citation accuracy, explicit assumptions and cautious feedback. Source agreement is not proof that a book is correct. Domain correctness requiring expertise remains a documented review limitation.

The following criteria are application-evaluation criteria, not an instructor-authored trading curriculum rubric. Model-derived reference answers are drafts, not independent truth. Model judging may assist triage only with disclosed limitations; it must not be the sole acceptance gate.

## Proposed supplied case coverage

| Case | Observable criteria |
| --- | --- |
| Conceptual prompt without numeric inputs | No unjustified calculator use; clear explanation |
| Planned loss for 51.20, 50.70, 200 shares | Exact 100.00, correlated result, exclusions; no realized-loss guarantee |
| Specialized book/source question | Retrieves appropriate evidence and uses source-specific content; citations resolve and support claims |
| Missing/conflicting educational evidence | Honest insufficiency or disagreement; no fabricated passage/citation |
| Learner explanation of a concept | Feedback quotes actual answer and is grounded in passages; limited evidence, no mastery inference |
| Before/after compaction follow-up | Important goals, corrections, unresolved questions and qualifications survive without invented learner evidence |
| Summary containing a learner-like quotation | Cannot pass as original learner provenance for a save proposal |
| Memory declined / addition rejected | No unauthorized read/use or record append respectively |
| Failed or interrupted run | No post-failure save approval; available partial state and honest failure |
| Instructions embedded in retrieved text | Application capabilities/boundaries retained; assess model compliance separately |
| Multiple/unknown/malformed requests | All-block handling, strict validation, correlation and explicit limits |

Infrastructure failure and adversarial cases can be maintenance/evaluation tests; do not automatically make them learner failure-path exercises. Settle classroom activities before publishing.

## Evaluation layers and reproducibility

1. **Deterministic tests:** arithmetic, schemas, permissions/consent, IDs, budget accounting, quote provenance, state transitions and compaction invariants.
2. **Real retrieval integration:** known passage relevance, metadata/location mapping, no-evidence outcomes, actual read-only access and embedding compatibility. Citation membership alone does not establish semantic support.
3. **Live end-to-end runs:** record actual selection, source support, feedback, uncertainty, summary fidelity, latency and failures. Do not force expected tool calls or silently replace services with mocks.
4. **Human inspection:** use a concise checklist with pass/fail/uncertain plus evidence and explanations. Repeat selected live cases because model behavior varies; report observed counts, not universal reliability claims.

Record configuration/corpus/preparation versions, embedding model, chat provider/model, case version, compaction policy/counting method, relevant settings, and dates—without credentials. Establish a baseline then compare one deliberate change on fresh Sessions with controlled memory. Use a small held-back set (not tuned during tweaks) to illustrate overfitting; determine cases and repetition count during implementation.

Private book excerpts and conversations cannot be checked into evaluation artifacts indiscriminately. Use redistributable inputs for shared fixtures and private local evidence for actual-book checks. Output reports must label skipped/blocked cases and source limits; avoid a misleading single aggregate score.

## Classroom and implementation deliverables

Supply complete runnable source/workflow, reusable evaluation cases, a simple report/checklist, lesson prose and instructor notes. Select the narrowest integration with existing diagnostics; no general benchmarking framework or LLM judge dependency is required. Do not implement a promised CLI/model selector from the superseded Lesson 18 roadmap.

Predict outcomes of a few cases; run; inspect actions/evidence; change one retrieval or compaction setting; run again and compare. Include one successful and one imperfect observed result. Attendees should explain why passing a schema test or producing eloquent prose is insufficient.

## Acceptance and final handoff

Standard Rust checks, deterministic coverage and documented live provider/database workflow are recorded honestly. Reports separate structural failures, semantic concerns, infrastructure blocks and unresolved domain review. Preserve root UI, consent, source privacy, bounds, GDK loop and diagnostics. Publish known limitations and independent prior gates rather than declaring the application production-ready.

End-of-course instructor review covers content, teaching, UI and release status for implemented lessons. Lesson 9's pending live 100-share tweak and earlier validation gaps do not vanish when Lesson 14 is supplied. Historical simulation remains the separate potential-lesson-15 draft; implementing it is not a completion requirement.
