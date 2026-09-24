# Lesson 1 Instructor Notes: Project Bootstrap

## Teaching objective

Give learners a clean, understandable Rust project before provider or inference concepts. They should identify the Cargo manifest, lockfile, binary entry point, toolchain file, environment template, and run command.

## Suggested pacing

- Course orientation and GDK-versus-Goose distinction: 5 minutes
- Walk through `Cargo.toml`, the exact provider-library pin, and the native-Rust decision: 10 minutes
- Create and explain `src/main.rs`: 10 minutes
- Build and run the placeholder: 10 minutes
- Review and questions: 5 minutes

## Before class

- Verify the pinned Rust toolchain on classroom Linux systems.
- Run `cargo check` and `cargo run` from a clean checkout.
- Confirm the committed `Cargo.lock` resolves with the manifest.
- Decide whether learners arrive with the toolchain and dependencies prepared.

## Discussion prompts

- What is Cargo responsible for in this project?
- Why commit `Cargo.lock` for an application/course repository?
- Why is `goose-providers` pinned to an exact alpha release?
- Why does a standalone Rust CLI use the native provider API instead of a Python/Kotlin binding layer?
- Which files may contain secrets, and why is `.env` ignored?

## Live-demo cautions

- Run Cargo commands from the repository root.
- Do not turn the lesson into a general Rust, package-management, linting, or async course.
- The repository may be newer than this snapshot; verify pinned versions and commands before teaching.

## Checkpoint

Learners can run the documented command and explain `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `src/main.rs`, and `.env.example`.
