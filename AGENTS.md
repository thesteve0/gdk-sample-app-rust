# AGENTS.md

Guidance for coding agents and for Goose when assisting learners in **Building Agentic Applications with the Goose GDK**.

## Project purpose

This repository is an evolving, instructor-led course for building a custom Rust agentic command-line application with the Goose Development Kit (GDK). It uses the exact pinned public `goose-providers` crate directly through its native Rust API; it is not a Goose application extension, a Kotlin/Python binding, or a wrapper around the Goose CLI.

Optimize first for a live class taught by an instructor. Goose-assisted self-study is useful, but secondary.

## Read before changing anything

1. Read `README.md`.
2. Inspect `git status` and preserve unrelated staged and unstaged work.
3. Read `Cargo.toml`, `Cargo.lock`, and `rust-toolchain.toml`.
4. Read the active root implementation in `src/main.rs`.
5. Read the relevant `lessons/<number>-<topic>/LESSON.md` and its complete `src/main.rs` reference solution.
6. Read the corresponding `instructor-notes/` file when one exists.
7. Read the provider JSON used by the active lesson.
8. For any GDK API, provider-schema field, type, command, or behavior, consult the official documentation and the exact pinned SDK source. Do not rely on memory.

## Repository model

- `src/main.rs` is the exercise currently being developed and may be incomplete.
- `lessons/` contains numbered instructional units.
- A lesson's `LESSON.md` contains explanations and important snippets.
- A lesson's `src/main.rs` is its complete reference solution from the outset. It is not an independently packaged application.
- Do **not** copy root source into a lesson directory at lesson completion.
- `instructor-notes/` contains one instructor guide per implemented lesson.
- Keep numeric prefixes on lesson directories.

## Pedagogy and scope

- Make small, concept-focused changes.
- Introduce only concepts required by the active lesson.
- Explain why a GDK abstraction is needed, not merely how to type it.
- Do not scaffold the final application or future lessons prematurely.
- Do not introduce ownership workarounds, traits, generic abstractions, workspaces, or async machinery before the active lesson needs them.
- Ask the instructor before making uncertain curriculum, sequencing, or architecture decisions.
- Keep the roadmap in `README.md` current when lesson state or order changes.

The direction includes providers, model selection, messages, streaming, system instructions, tools, the tool-result round trip, and a command-line interface. The roadmap is not constrained to a fixed number of lessons.

## Lesson requirements

Every implemented `LESSON.md` should include a goal, concepts introduced, prerequisites, incremental instructions, focused snippets, a run or validation command, expected structural behavior, success criteria, and the conceptual next step.

Keep lesson prose, reference source, root workflow, and instructor notes consistent. If code behavior changes, update every affected explanation in the same change. Do not add deliberate failure-path exercises without instructor approval.

## Provider and model conventions

- Provider JSON is the source of truth for endpoint, authentication, models, and capabilities.
- The bundled JSON is a replaceable default, not a required service or model.
- Learners copy and modify an existing valid configuration; they do not author its complete schema from scratch.
- Before the CLI lesson, use a default provider path and an optional positional override. Later replace it with a named `--provider` option after deliberately selecting a Rust parser.
- Initially select the first configured model. Introduce `--model` later and validate it against JSON-declared models.
- Load `.env` with `dotenvy` before constructing the provider. Never commit `.env`, credentials, or secret-bearing provider files.

## Development environment

- Target Linux for now.
- Use the toolchain pinned in `rust-toolchain.toml`.
- Use Cargo for dependency changes, locking, building, running, and tests.
- Commit `Cargo.lock` for this application/course repository; update it with `Cargo.toml`.
- Pin an exact alpha GDK release. Do not use moving branches or internal Goose crates without an explicit architecture decision.
- Do not call `cargo` with `--all-features` by default.
- Keep the dotenv approach consistent: use `dotenvy`.

## Validation

- Prefer live-provider validation whenever a lesson makes provider calls.
- If a provider is unavailable, ask the instructor to start or supply one; do not silently replace live validation with mocks.
- Assert structural outcomes rather than exact model text.
- After changing Rust source, manifests, lockfiles, or configuration, run the narrowest relevant checks and the lesson's documented command:

  1. `cargo fmt --check`
  2. `cargo check`
  3. `cargo clippy --all-targets`
  4. `cargo test`
  5. `cargo run -- [lesson arguments]`

- Report exactly what was and was not validated.

## Documentation and hygiene

- `README.md` is the course landing page; lesson detail belongs in `LESSON.md`; instructor-only guidance belongs in `instructor-notes/`.
- Call this material a course or workshop.
- External links are supplemental; core lesson instructions must stand on their own after setup.
- Never overwrite unrelated user changes or edit generated build output.
- Keep `/target/`, `.env`, `.idea/`, and `out/` untracked.
- Do not add contribution guidance unless asked.

## Goose-assisted self-study mode

Use this mode only when a learner asks Goose to guide them through the course.

1. Ask which lesson the learner is taking and confirm provider setup.
2. Read the lesson, its solution, and the active root state without revealing the complete solution prematurely.
3. Explain or assign one small step at a time.
4. Ask the learner to write or run it, then wait for their result or approval.
5. Diagnose observed output using the lesson's success criteria.
6. Prefer hints and focused snippets over replacing the learner's entire file.
7. Do not skip ahead into later concepts unless needed to unblock the current lesson.

## Instructor authority

The instructor owns curriculum scope, sequencing, and release decisions. When this file conflicts with an explicit instructor request, follow the request and update durable documentation when relevant.
