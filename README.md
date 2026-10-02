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
| 4 | System instructions, message roles, and multi-turn conversation | **Complete** | [`lessons/04-conversation/`](lessons/04-conversation/) |
| 5 | Define a deterministic trading calculator tool and inspect its raw request | **Complete** | [`lessons/05-deterministic-tool-request/`](lessons/05-deterministic-tool-request/) |
| 6 | Execute the tool and return its result through the raw provider protocol | **Complete** | [`lessons/06-deterministic-tool-response/`](lessons/06-deterministic-tool-response/) |
| 7 | Introduce state-machine operations, effects, sessions, and a minimal runtime | **Complete** | [`lessons/07-state-machine-runtime/`](lessons/07-state-machine-runtime/) |
| 8 | One request and one response through the GDK state machine | **Active** | [`lessons/08-machine-request-response/`](lessons/08-machine-request-response/) |
| 9 | Assemble provider inference and the typed tool operation | Planned | — |
| 10 | Add domain instructions and application-enforced safety boundaries | Planned | — |
| 11 | Establish an evaluation baseline for tool use and grounded conclusions | Planned | — |
| 12 | Define fixture-backed read-only market observations | Planned | — |
| 13 | Combine observations and deterministic calculations in trade analysis | Planned | — |
| 14 | Ingest private educational documents while preserving provenance | Planned | — |
| 15 | Add educational retrieval as a read-only tool with citations | Planned | — |
| 16 | Evaluate retrieval quality, citations, and untrusted document content | Planned | — |
| 17 | Combine teaching, retrieval, market observations, and trade analysis | Planned | — |
| 18 | Command-line workflow, explicit model selection, and operational errors | Planned | — |
| Optional capstone | Open-weights model comparison or fine-tuning experiment | Planned | — |

**Status meanings:** **Complete** is ready to teach and validated; **Active** is the root exercise; **Planned** is expected direction only.

The post-Lesson 4 roadmap is accepted based on isolated technical validation with the configured local provider. That validation confirmed native structured tool calls, a complete raw request/result round trip, the equivalent typed tool through `goose-agent`, a two-lesson state-machine transition, and decimal-string prices parsed into exact decimal arithmetic. [`post-lesson4-plan.md`](post-lesson4-plan.md) is the durable architecture and curriculum record.

On 2026-10-01 the instructor ordered a renumbering: a new Lesson 8 — one request and one response through the GDK state machine, the introductory walkthrough for the machine — was inserted before the former Lesson 8, and every lesson from the former Lesson 8 onward shifted by one (former 8→9 through former 17→18). [`post-lesson4-plan.md`](post-lesson4-plan.md) records this as the explicit instructor decision the plan's stability clause requires.

### Immediate Lesson 8 boundary

Lesson 7 completed the state-machine vocabulary over a deterministic, provider-free seed: it seeds the conversation Lesson 6 reconstructed (`call_seed_001`, entry `51.20`, stop `50.70`, 200 shares producing `100.00`), assembles the machine over two hand-written operations, and stops via `yield_to_client`. Lesson 8 — the active exercise — walks the machine through one request and one response with a real provider call inside `machine.step`: the seed is exactly one user question ("What is the capital of France?"), the sole registered step is the GDK-shipped `InferenceRunner` registered as `Step::Inference`, and the pass loop is written by hand with an explicit state-step bound (`MAX_STATE_STEPS = 4`). The effect vocabulary is the lesson-owned `ChatEffect` (`AppendMessage`, `RecordUsage`) because the GDK's default `ConversationEffect` has no `InferenceEffect` implementation in the pinned release. The machine calls the provider on pass 1 only; pass 2 re-derives from the persisted reply and stops with "no step applies". There are no tool calls or tool definitions, no system instruction, and no `StateMachine::run` — the typed tool, the ordered step list alongside inference, and the crate's run loop return in Lesson 9, and a multi-turn follow-up (letting the user ask another question) is deliberately deferred.

## Repository organization

```text
.
├── src/main.rs                 # current exercise; changes as the course advances
├── lessons/                    # explanation plus complete reference source
├── instructor-notes/           # instructor-only pacing and teaching guidance
├── post-lesson4-plan.md        # accepted post-Lesson 4 curriculum/architecture context
├── AGENTS.md                   # guidance for coding agents and Goose-assisted self-study
├── Cargo.toml
├── Cargo.lock
├── rust-toolchain.toml
├── custom_aa_llama_qwen3_6-35b.json
├── .env.example
├── .gitignore                  # keeps /target/, .env, .idea/, and out/ untracked
└── LICENSE                     # Apache License 2.0
```

### Root source

Students write code in `src/main.rs`. It represents the lesson currently being developed, not a stable or production-ready application. The root is currently at the Lesson 8 machine request/response exercise: it seeds an in-memory store with one user question ("What is the capital of France?"), assembles the GDK state machine over exactly one step — the GDK-shipped `InferenceRunner` registered as `Step::Inference` — implements `SessionLoader` and `EffectHandler` as the machine's persistence interface, and drives a hand-written pass loop with an explicit state-step bound. Pass 1 calls the provider and persists the streamed reply as effects (usage, then the message); pass 2 re-derives from the persisted reply and stops with "no step applies". It reads `.env` and the provider JSON as every provider lesson does, and defines the lesson-owned `ChatEffect` effect vocabulary because the GDK's default `ConversationEffect` has no `InferenceEffect` implementation in the pinned release.

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

The manifest pins `goose-providers` and `goose-agent` to exact `0.1.0-alpha.11` releases; the provider crate enables its `rustls-tls` transport feature. `goose-providers` supplies native provider construction, messages, streaming, and the raw tool protocol, and `rmcp` supplies the MCP tool type used to advertise deterministic tools. `goose-agent` supplies the GDK state machine, operations, effects, emitter, and the session/effect traits (added with Lesson 7, after learners understand the raw protocol it coordinates). `serde`'s derive feature deserializes untrusted tool arguments into domain structs, and `rust_decimal` supplies exact decimal arithmetic for money-related tool arguments (both added with Lesson 6). `anyhow`, `async-trait`, and `tokio-util` support the state machine's error type, async traits, and cancellation token (added with Lesson 7). Each lesson that adds a new requirement instructs adding it to the root manifest. The application does **not** use the `goose-sdk` foreign-language binding surface. `futures` supplies stream consumption and `dotenvy` loads a local `.env` before a provider is constructed.

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

Run the current Lesson 8 machine request/response exercise from the repository root:

```bash
cargo run
```

It contacts the provider: the store is seeded in code with one user question ("What is the capital of France?"), the GDK state machine runs over exactly one step — the GDK-shipped `InferenceRunner` — and the pass loop carries an explicit state-step bound. Pass 1 reloads the session, the inference step re-derives that the conversation ends in a provider turn, calls the provider with the whole conversation (no system prompt, no tools), and the streamed reply prints between payload delimiters; the effects persist in order (record usage, then append message). Pass 2 reloads the persisted state, the same step declines because the conversation now ends in the assistant's reply, and the run reports "no step applies" and prints the final persisted conversation (2 messages). The lesson also validates deterministically with `cargo test` (seed, loader, effects, a declining step, the state-step bound). Provider-calling lessons are validated live against a configured provider; if the endpoint is unreachable, ask the instructor to start or supply one. Marking a lesson Complete does not by itself approve it for teaching — that release decision remains with the instructor.

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
- [GDK state machine: `goose-agent` crate README](https://github.com/block/goose/blob/main/crates/goose-agent/README.md) — the canonical description of the state-machine approach ("the whole agent's behavior is a function of the persisted conversation, not of in-memory loop state"); the goose-docs.ai pages currently cover the provider layer only.

## License

Licensed under the [Apache License 2.0](LICENSE).