# Building Agentic Applications with the Goose GDK

An evolving, instructor-led course for learning how to build a custom agentic **Rust** command-line application with the Goose Development Kit (GDK). The course incrementally builds a day-trading teaching assistant that can also analyze hypothetical and user-proposed trades, present evidence and alternatives, and reach a clearly explained conclusion. It will not place, modify, or cancel trades.

This repository consumes the public [`goose-providers`](https://crates.io/crates/goose-providers) and [`goose-agent`](https://crates.io/crates/goose-agent) crates directly. Learners first see the raw provider tool-request/tool-result protocol and then assemble the same behavior with the GDK's state-machine agent loop. It does **not** expose this application to Kotlin or Python, extend the Goose application, or wrap the Goose CLI. The goal is to understand and assemble an agentic application one concept at a time.

> [!IMPORTANT]
> The GDK libraries are alpha. This course pins their exact versions and the Rust toolchain; review the official API reference and pinned source before upgrading them. Another coordinated GDK upgrade is expected before the course is taught.

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
- Define deterministic, read-only tools and complete the raw tool-request/tool-result round trip.
- Assemble tools and inference into the GDK state-machine agent loop.
- Distinguish model reasoning from deterministic calculations, time-sensitive market observations, and retrieved educational evidence.
- Enforce the application's no-trade-execution boundary through available capabilities rather than relying only on a prompt.
- Evaluate an agent's use of tools, evidence, uncertainty, alternatives, and conclusions.
- Add retrieval over a private, user-supplied educational corpus without committing copyrighted source material.
- Wrap the application in a usable command-line interface.

Rust concepts such as `Result`, `match`, ownership of conversation history, and Tokio async execution are introduced only when a GDK use case needs them.

## Course roadmap

| Lesson | Topic | Status | Materials |
|---:|---|---|---|
| 1 | Project bootstrap with Cargo and `src/main.rs` | **Complete** | [`lessons/01-bootstrap/`](lessons/01-bootstrap/) |
| 2 | Provider configuration and model discovery smoke test | **Complete** | [`lessons/02-provider-smoke-test/`](lessons/02-provider-smoke-test/) |
| 3 | First streaming model call | **Complete** | [`lessons/03-streaming-call/`](lessons/03-streaming-call/) |
| 4 | System instructions, message roles, and multi-turn conversation | **Active** | [`lessons/04-conversation/`](lessons/04-conversation/) |
| 5 | Define a deterministic trading calculator tool and inspect its raw request | Planned | — |
| 6 | Execute the tool and return its result through the raw provider protocol | Planned | — |
| 7 | Introduce state-machine operations, effects, sessions, and a minimal runtime | Planned | — |
| 8 | Assemble provider inference and the typed tool operation | Planned | — |
| 9 | Add domain instructions and application-enforced safety boundaries | Planned | — |
| 10 | Establish an evaluation baseline for tool use and grounded conclusions | Planned | — |
| 11 | Define fixture-backed read-only market observations | Planned | — |
| 12 | Combine observations and deterministic calculations in trade analysis | Planned | — |
| 13 | Ingest private educational documents while preserving provenance | Planned | — |
| 14 | Add educational retrieval as a read-only tool with citations | Planned | — |
| 15 | Evaluate retrieval quality, citations, and untrusted document content | Planned | — |
| 16 | Combine teaching, retrieval, market observations, and trade analysis | Planned | — |
| 17 | Command-line workflow, explicit model selection, and operational errors | Planned | — |
| Optional capstone | Open-weights model comparison or fine-tuning experiment | Planned | — |

**Status meanings:** **Complete** is ready to teach and validated; **Active** is the root exercise; **Draft** exists but needs independent Rust validation; **Planned** is expected direction only.

The post-Lesson 4 roadmap is accepted based on the isolated technical spike in [`spikes/post-lesson4/`](spikes/post-lesson4/). The spike confirmed native structured tool calls with the configured local provider, a complete raw request/result round trip, and the equivalent typed tool through `goose-agent`. It also confirmed that the state-machine transition needs two focused lessons and that model-facing prices should be decimal strings parsed into exact decimal arithmetic. [`post-lesson4-plan.md`](post-lesson4-plan.md) is the durable architecture and curriculum record; spike code is exploratory evidence, not lesson source.

### Immediate Lesson 5 boundary

Lesson 5 introduces exactly one domain-specific deterministic tool: maximum planned loss for a hypothetical long position. Its example uses entry `51.20`, stop `50.70`, and 200 shares, producing `100.00` before fees, slippage, or a gap through the stop. Learners define its name, description, and JSON input schema; pass that definition to raw provider inference; inspect all returned content blocks; identify a structured tool request; and preserve its request ID. They do **not** execute the tool or return a result until Lesson 6. Early tool lessons continue selecting the first configured model; explicit model selection remains deferred.

## Repository organization

```text
.
├── src/main.rs                 # current exercise; changes as the course advances
├── lessons/                    # explanation plus complete reference source
├── instructor-notes/           # instructor-only pacing and teaching guidance
├── spikes/                     # isolated engineering evidence, not lesson solutions
├── post-lesson4-plan.md        # accepted post-Lesson 4 curriculum/architecture context
├── Cargo.toml
├── Cargo.lock
├── rust-toolchain.toml
├── custom_aa_llama_qwen3_6-35b.json
└── .env.example
```

### Root source

Students write code in `src/main.rs`. It represents the lesson currently being developed, not a stable or production-ready application. The root is currently at the Lesson 4 conversation checkpoint: it reconstructs a streamed assistant reply and sends an ordered user/assistant/user history with a follow-up question, keeping the system instruction separate from the conversation.

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

The manifest pins `goose-providers` and `goose-agent` to `0.1.0-alpha.11`; the provider crate enables its `rustls-tls` transport feature. `goose-providers` supplies native provider construction, messages, streaming, and the raw tool protocol. `goose-agent` supplies the GDK state-machine agent loop and typed tool operations introduced after learners understand that protocol. The application does **not** use the `goose-sdk` foreign-language binding surface. `futures` supplies stream consumption and `dotenvy` loads a local `.env` before a provider is constructed.

## Provider configuration

The application uses a declarative provider JSON file as the source of truth for the provider engine and endpoint, authentication behavior, configured models, and capabilities. The bundled [`custom_aa_llama_qwen3_6-35b.json`](custom_aa_llama_qwen3_6-35b.json) is a replaceable local OpenAI-compatible example, not a course requirement.

Lesson 2 constructs a Rust provider with `goose_providers::declarative::from_json` and queries its available models with `fetch_supported_models`. Later lessons use that same native provider to perform inference. Environment-variable placeholders in the JSON are resolved when a provider is constructed. Learners copy an existing valid JSON configuration rather than authoring the complete schema from scratch.

Before the dedicated CLI lesson, examples use the bundled configuration by default and accept one optional positional override:

```bash
cargo run -- path/to/provider.json
```

Early inference and tool lessons select the first model declared in the JSON to focus on GDK fundamentals. A later lesson adds named model selection and validates whether a selected model has the capabilities required by the application.

## Target application and capability boundary

The target application is first and foremost a **day-trading teaching assistant**. It may also analyze hypothetical or user-proposed trades and reach a final conclusion, but a useful conclusion must show its evidence, assumptions, uncertainty, and reasonable alternatives. Model-generated prose is not treated as evidence by itself.

The course separates three kinds of grounded capability:

- deterministic tools for calculations such as percentage change, profit and loss, and position risk;
- read-only tools for time-sensitive observations such as market prices, always accompanied by provenance and freshness information;
- retrieval from educational material supplied privately by the learner.

The application has no trade-execution capability. It will not receive brokerage credentials or expose tools that submit, modify, or cancel orders. This is an architectural boundary, not merely a sentence in the system instruction.

### Private educational corpus

The planned retrieval corpus consists primarily of EPUB or PDF books legally owned by the learner. Those books, extracted text, retrieval indexes, and other derived local artifacts are private inputs and must not be committed to this repository. Lessons will use a small redistributable or course-authored sample corpus for reproducible exercises and document how a learner points the application at private local material.

Retrieval supplies relatively stable educational evidence; it is not a substitute for a market-data tool. Fine-tuning is optional and late in the course, after a baseline, tools, retrieval, and an evaluation set exist.

## Planned architecture diagrams

The tool lessons will include two diagrams at the point each abstraction is introduced:

1. a raw provider sequence showing inference, a structured tool request, application validation and execution, a correlated tool result, and follow-up inference;
2. a GDK state-machine diagram showing how inference and tool operations repeatedly derive and persist effects from conversation state until the machine yields or no operation applies.

The diagrams are intentionally sequenced this way so learners understand the protocol before the state machine that coordinates it.

### API keys and `.env`

The bundled local provider requires no key. For a provider that does:

1. its JSON identifies the environment variable;
2. place that variable in a local `.env` file copied from `.env.example`;
3. never commit `.env` or real credentials.

## Run the active exercise

Run the current Lesson 4 conversation exercise from the repository root:

```bash
cargo run
```

It loads an optional local `.env` through `dotenvy`, makes two streaming requests against the first configured model, prints each assistant reply to stdout, and prints completion usage metadata to stderr. The first reply is reconstructed in memory and re-sent as an assistant turn so the second request carries an ordered user/assistant/user history. Lessons 5 onward are planned. Provider-calling lessons are validated live here, but their release status remains Draft until the instructor independently approves them.

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
- [`goose-providers` 0.1.0-alpha.11](https://crates.io/crates/goose-providers/0.1.0-alpha.11)
- [`goose-agent` 0.1.0-alpha.11](https://crates.io/crates/goose-agent/0.1.0-alpha.11)

## License

Licensed under the [Apache License 2.0](LICENSE).