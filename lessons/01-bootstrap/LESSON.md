# Lesson 1: Project Bootstrap

## Goal

Create a clean Rust project layout for a GDK agentic application using Cargo, a binary entry point, and safe environment configuration.

## Directory structure

Students work in the repository root:

```text
gdk-sample-app-rust/
├── Cargo.toml
├── Cargo.lock
├── rust-toolchain.toml
├── .env.example
├── .gitignore
├── src/main.rs
└── lessons/01-bootstrap/     # explanation and reference source
```

## Step 1.1: Create the manifest

`Cargo.toml` identifies this binary package and its exact provider-library dependency:

```toml
[package]
name = "gdk-hello"
version = "0.1.0"
edition = "2021"
rust-version = "1.94.1"

[dependencies]
dotenvy = "0.15.7"
futures = "0.3"
goose-providers = { version = "=0.1.0-alpha.10", features = ["rustls-tls"] }
```

`goose-providers` is alpha, so the course pins an exact release. It is the native Rust library used by this standalone CLI to construct providers, send requests, and receive streams. `futures` supplies the `StreamExt` trait used to consume those streams in later lessons. `dotenvy` loads a local `.env` before provider construction in later lessons.

This CLI does **not** enable `goose-sdk`'s `uniffi` feature. That feature builds the SDK's foreign-language binding surface for Python and Kotlin; it is not the native Rust API for this application.

## Step 1.2: Add safe local configuration

Create `.env.example` with placeholder key documentation and ignore the real `.env`. The Rust-focused `.gitignore` includes:

```gitignore
/target/
.env
/.idea/
/out/
```

## Step 1.3: Add the binary entry point

Create `src/main.rs`:

```rust
fn main() {
    println!("Hello from gdk_hello");
}
```

Cargo recognizes `src/main.rs` as the default binary entry point. The root source is the current exercise; the source under this lesson directory is a complete reference snapshot.

## Step 1.4: Build and run

From the repository root:

```bash
cargo check
cargo run
```

`Cargo.lock` records the resolved dependency set for this application/course repository.

## Success criteria

- Cargo uses the pinned Rust toolchain and successfully checks the project.
- `cargo run` prints `Hello from gdk_hello`.
- You can identify the roles of `Cargo.toml`, `Cargo.lock`, `src/main.rs`, lesson material, and `.env.example`.

## Next

Lesson 2 parses the declarative provider JSON and performs a connectivity/model-discovery smoke test.
