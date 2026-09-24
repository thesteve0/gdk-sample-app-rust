# Building Agentic Applications with the Goose GDK

An evolving, instructor-led course for learning how to build a custom agentic **Rust** command-line application with the Goose Development Kit (GDK).

This repository consumes the public [`goose-providers`](https://crates.io/crates/goose-providers) crate directly for native Rust provider construction, messages, streamed responses, and—later in the course—tools. It does **not** expose this application to Kotlin or Python, extend the Goose application, or wrap the Goose CLI. The goal is to understand and assemble an agentic application one concept at a time.

> [!IMPORTANT]
> The GDK provider library is alpha. This course pins its exact version and Rust toolchain; review the official API reference and the pinned source before upgrading either.

> [!IMPORTANT]
> This course is under active development. The root source is the exercise currently being worked on and may be incomplete. The numbered lesson directories contain the accompanying explanations and complete reference implementations.

## Who this is for

The primary format is an **in-person course or workshop** led by an instructor. Lessons are small conceptual units, so several may be taught in one class session.

You can also use the repository for self-study. Goose can act as a tutor by following [`AGENTS.md`](AGENTS.md), presenting one step at a time, and waiting for you before moving ahead. Self-study support is secondary to the instructor-led experience.

## Learning goals

By the end of the course, learners should understand how to:

- Structure a Rust GDK application with Cargo.
- Load a declarative provider configuration.
- Construct a configured provider and discover its available models through Goose's native Rust provider API.
- Select a configured model for an individual request.
- Construct provider messages and system instructions.
- Process streamed text, completion metadata, and errors.
- Define tools and complete the tool-request/tool-result round trip.
- Wrap the application in a usable command-line interface.

Rust concepts such as `Result`, `match`, ownership of conversation history, and Tokio async execution are introduced only when a GDK use case needs them.

## Course roadmap

| Lesson | Topic | Status | Materials |
|---:|---|---|---|
| 1 | Project bootstrap with Cargo and `src/main.rs` | **Active** | [`lessons/01-bootstrap/`](lessons/01-bootstrap/) |
| 2 | Provider configuration and model discovery smoke test | Draft | [`lessons/02-provider-smoke-test/`](lessons/02-provider-smoke-test/) |
| 3 | First streaming model call | Draft | [`lessons/03-streaming-call/`](lessons/03-streaming-call/) |
| 4 | System instructions, message roles, and multi-turn conversation | Draft | [`lessons/04-conversation/`](lessons/04-conversation/) |
| 5 | Explicit model selection with `--model` | Planned | — |
| 6 | Defining a provider tool | Planned | — |
| 7 | Executing tools and returning results | Planned | — |
| 8 | Command-line interface, configuration, and error handling | Planned | — |

**Status meanings:** **Complete** is ready to teach and validated; **Active** is the root exercise; **Draft** exists but needs independent Rust validation; **Planned** is expected direction only.

## Repository organization

```text
.
├── src/main.rs                 # current exercise; changes as the course advances
├── lessons/                    # explanation plus complete reference source
├── instructor-notes/           # instructor-only pacing and teaching guidance
├── Cargo.toml
├── Cargo.lock
├── rust-toolchain.toml
├── custom_aa_llama_qwen3_6-35b.json
└── .env.example
```

### Root source

Students write code in `src/main.rs`. It represents the lesson currently being developed, not a stable or production-ready application. The root is currently at the Lesson 1 bootstrap checkpoint.

### Lesson directories

Each numbered lesson is both an instructional reference and a solution:

- `LESSON.md` explains the lesson and highlights important code snippets.
- `src/main.rs` contains the complete reference implementation for that lesson.

Lesson source is reference material, not an independently packaged application. The root manifest and locked dependencies are shared. The complete solution is already present in each lesson directory; it is not copied from the root at the end of class.

## Prerequisites and setup

The course currently targets **Linux**. Install the Rust toolchain pinned in [`rust-toolchain.toml`](rust-toolchain.toml), which initially specifies Rust `1.94.1`, plus Git and access to a compatible model provider.

From the repository root, let Cargo fetch and build the exact dependency resolution recorded in `Cargo.lock`:

```bash
cargo check
```

The manifest pins `goose-providers` to `0.1.0-alpha.10` with its `rustls-tls` transport feature. The application is a native Rust CLI: it uses the provider crate's native API and does **not** depend on `goose-sdk` or enable its `uniffi` feature. `uniffi` is the SDK's foreign-language binding surface for Python and Kotlin, neither of which calls this application. `futures` supplies stream consumption and `dotenvy` loads a local `.env` before a provider is constructed.

## Provider configuration

The application uses a declarative provider JSON file as the source of truth for the provider engine and endpoint, authentication behavior, configured models, and capabilities. The bundled [`custom_aa_llama_qwen3_6-35b.json`](custom_aa_llama_qwen3_6-35b.json) is a replaceable local OpenAI-compatible example, not a course requirement.

Lesson 2 constructs a Rust provider with `goose_providers::declarative::from_json` and queries its available models with `fetch_supported_models`. Later lessons use that same native provider to perform inference. Environment-variable placeholders in the JSON are resolved when a provider is constructed. Learners copy an existing valid JSON configuration rather than authoring the complete schema from scratch.

Before the dedicated CLI lesson, examples use the bundled configuration by default and accept one optional positional override:

```bash
cargo run -- path/to/provider.json
```

Early inference lessons select the first model declared in the JSON to focus on GDK fundamentals. A later lesson adds named model selection and validation.

### API keys and `.env`

The bundled local provider requires no key. For a provider that does:

1. its JSON identifies the environment variable;
2. place that variable in a local `.env` file copied from `.env.example`;
3. never commit `.env` or real credentials.

## Run the active exercise

Run the current Lesson 1 placeholder from the repository root:

```bash
cargo run
```

It prints `Hello from gdk_hello`. Lessons 2–4 are Draft reference implementations. Their provider behavior has been exercised during migration, but their release status remains Draft until the instructor independently approves them.

## Validation philosophy

Every implemented lesson defines prerequisites, a validation command, expected structural behavior, and explicit success criteria. Live-provider validation is the priority for provider lessons; model text is nondeterministic, so validate behavior such as receiving text fragments and completion metadata rather than exact wording.

For repository maintenance, use the narrowest relevant checks:

```bash
cargo fmt --check
cargo check
cargo clippy --all-targets
cargo test
```

Do not use `--all-features` by default: this CLI enables only the `rustls-tls` provider transport it needs.

## Troubleshooting

### Provider configuration is not found

Run Cargo from the repository root, or pass a valid path after `--`. Relative paths are resolved from the current working directory.

### Provider cannot be reached

Confirm that the local server or shared endpoint is running, `base_url` is correct, and local firewall or network policy permits the connection.

### Configured model is missing

Compare the model IDs printed by Lesson 2 with the configured model names in the selected provider JSON. For an OpenAI-compatible configuration with dynamic discovery enabled, the provider queries its configured models endpoint.

### Authentication fails

Confirm that the provider JSON identifies the right environment variable, `.env` is in the repository root, and the variable is loaded before provider construction. Do not put credentials in committed provider JSON.

### Cargo build fails

Confirm that the pinned Rust toolchain is active, `Cargo.toml` and `Cargo.lock` changed together, and required native build tools are installed. Re-check the exact pinned provider version and enabled transport feature before changing dependencies.

## For maintainers and coding agents

Read [`AGENTS.md`](AGENTS.md) before changing the repository. Preserve the instructor-led small-step pedagogy, keep lesson prose and reference source synchronized, preserve unrelated work, and validate provider lessons live where applicable.

## Official references

- [GDK SDK overview](https://goose-docs.ai/docs/gdk/sdk)
- [GDK SDK API reference](https://goose-docs.ai/docs/gdk/sdk/api-reference)
- [Configure LLM Provider](https://goose-docs.ai/docs/getting-started/providers)
- [`goose-providers` 0.1.0-alpha.10](https://crates.io/crates/goose-providers/0.1.0-alpha.10)

## License

Licensed under the [Apache License 2.0](LICENSE).
