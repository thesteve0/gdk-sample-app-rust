# Lesson 1 Instructor Notes: Project Bootstrap

## Teaching objective

Give learners a clean, understandable Rust project and a mental model before provider or inference code. They should identify the Rust application as the workflow owner; distinguish application, provider, model, inference, message, conversation history, tool, and future agentic loop; and identify the Cargo manifest, lockfile, binary entry point, toolchain file, environment template, and run command.

## Suggested pacing

- Course orientation, target use case, and application-versus-model boundary: 10 minutes
- Walk through the future agentic-application diagram and vocabulary: 10 minutes
- Walk through `Cargo.toml`, the exact provider-library pin, and the native-Rust decision: 10 minutes
- Create and explain `src/main.rs`, including the harmless dotenv bootstrap call: 10 minutes
- Build and run the dependency-backed greeting; review and questions: 10 minutes

## Before class

- Verify the pinned Rust toolchain on classroom Linux systems.
- Run `cargo check` and `cargo run` from a clean checkout.
- Confirm the committed `Cargo.lock` resolves with the manifest.
- Decide whether learners arrive with the toolchain and dependencies prepared.

## Discussion prompts

- Which responsibilities belong to the Rust application, and which belong to the model?
- Why can a model request a tool without being able to execute it?
- Why is the no-trade-execution boundary an application capability decision rather than only a prompt instruction?
- What is the difference between a provider and a model?
- What is the future agentic loop, and which portions are deliberately absent from Lesson 1?
- What is Cargo responsible for in this project?
- Why commit `Cargo.lock` for an application/course repository?
- Why is `goose-providers` pinned to an exact alpha release?
- Why does a standalone Rust CLI use the native provider API instead of a Python/Kotlin binding layer?
- Why can this lesson load `.env` even when no `.env` file exists?
- Which files may contain secrets, and why is `.env` ignored?

## Live-demo cautions

- Run Cargo commands from the repository root.
- Keep the architecture diagram at the conceptual level: do not introduce provider calls, tool schemas, raw protocol fields, or state-machine code yet.
- Do not call the bootstrap application a Goose harness. At this stage the Rust application is the orchestrator; the GDK state-machine runtime arrives later.
- Do not turn the lesson into a general Rust, package-management, linting, or async course.
- The repository may be newer than this snapshot; verify pinned versions and commands before teaching.

## Checkpoint

Learners can explain at a high level how the application, provider, model, conversation history, tools, and later agentic loop relate; identify the application's safety boundary; run the documented command; observe that it succeeds without a local `.env`; and explain why the bootstrap program calls `dotenvy::dotenv().ok()`. They can also explain `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `src/main.rs`, and `.env.example`.
