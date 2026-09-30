# Lesson 1: Project Bootstrap

## Goal

Create a clean Rust project layout for the command-line application used throughout this GDK course, using Cargo, a binary entry point, and safe environment configuration.

## How this agentic application works

This course builds a Rust command-line **agentic application**: an application that combines model inference with application-controlled state and capabilities. The target is a day-trading teaching assistant. It can explain hypothetical trades and later calculate their planned risk, but it will not receive brokerage credentials or place, modify, or cancel trades.

The Rust application—not the model—owns the workflow. It reads configuration, keeps the discussion history, decides which capabilities exist, validates untrusted inputs, performs permitted work, and presents the result. The Goose Development Kit (GDK) libraries give the application native Rust types and provider APIs; they do not take control away from the application.

| Term | Meaning in this course |
| --- | --- |
| **Application** | The Rust CLI learners are building. It controls the workflow, configuration, data, safety boundaries, and available capabilities. |
| **Provider** | The configured service interface the application uses to communicate with a model. |
| **Model** | The probabilistic language model that generates text and, in later lessons, structured requests. |
| **Inference** | One request from the application to the model and the model's response. |
| **Message** | A contribution sent to or received from the model, such as a user question or assistant response. |
| **Conversation history** | The ordered messages that the application retains and sends again when later inference needs prior context. |
| **Tool** | An application-provided capability with a defined input shape. Later lessons use tools for deterministic calculations; a model may request one, but cannot run it itself. |
| **Agentic loop** | The later repeating workflow in which the application asks the model for the next step, handles a permitted tool request if one arrives, and gives the result back to the model. |

The complete destination of the course looks like this. This is an architectural preview, not behavior implemented in Lesson 1:

```text
User
  |
  v
Rust application
  |-- keeps conversation state
  |-- sends an inference request to the provider/model
  |-- advertises only allowed tools
  |-- validates and executes permitted tool requests
  |-- returns tool results to the model
  v
Provider / model
  |-- produces text
  |-- may later request a tool
  v
Rust application presents the final response
```

The model does not directly run Rust code, access credentials, retain the application's state, or place trades. The application controls each boundary. Lessons 2–5 build the first parts of this picture; Lessons 6–8 complete the raw request/result loop and then introduce the GDK state-machine runtime.

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

`src/main.rs` is where the application begins. It is the future orchestrator described above; this bootstrap version only prepares safe local configuration and proves that Cargo can run the binary.

Create `src/main.rs`:

```rust
fn main() {
    // A missing `.env` is normal during bootstrap; dotenvy leaves the process
    // environment unchanged in that case.
    dotenvy::dotenv().ok();

    println!("Hello from gdk_hello");
}
```

Cargo recognizes `src/main.rs` as the default binary entry point. Calling `dotenvy::dotenv()` exercises the `dotenvy` dependency now, while the application has no provider or secrets to configure yet. Ignoring its error makes a missing local `.env` normal and leaves the existing process environment unchanged. Lesson 2 relies on the same call before it constructs a provider.

The root source is the current exercise; the source under this lesson directory is a complete reference snapshot.

## Step 1.4: Build and run

From the repository root:

```bash
cargo check
cargo run
```

`Cargo.lock` records the resolved dependency set for this application/course repository.

## Expected structural behavior

The program invokes `dotenvy::dotenv()` and then prints its greeting. It does not construct a provider, read provider JSON, or make a network request.

## Success criteria

- Cargo uses the pinned Rust toolchain and successfully checks the project.
- `cargo run` succeeds with no `.env` file and prints `Hello from gdk_hello`.
- `src/main.rs` invokes the `dotenvy` dependency before the greeting.
- You can identify the roles of `Cargo.toml`, `Cargo.lock`, `src/main.rs`, lesson material, and `.env.example`.

## Next

Lesson 2 makes the overview concrete by constructing the configured provider—the application's connection to a model service—and discovering the models it makes available. It does not perform inference yet.
