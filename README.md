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
| 7 | GDK state-machine mental model — no code | **Planned; review pending** | Plan below |
| 8 | One streamed request/response through the state machine | **Planned; review pending** | Plan below |
| 9 | One Session, successive user turns, and selective tool use | **Planned; review pending** | Plan below |
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

**Status meanings:** **Complete** is ready to teach and validated; **Active** is the root exercise; **Planned** is expected direction only; **Draft** is withdrawn material, not part of the teaching sequence.

Lessons 1–6 are complete. On 2026-10-05 the instructor withdrew the previous Lessons 7 and 8 for complete rework; their unchanged prose, reference source, and instructor notes are preserved under [`drafts/`](drafts/). They are not part of the teaching sequence.

**This README is the authoritative curriculum plan.** The replacement plan below records the subsequent instructor discussion and is ready for review, not yet implemented or released. It replaces the old state-machine boundaries, including the former planned Lesson 9. Lessons 10 onward retain their existing numbers and direction; this three-lesson replacement needs no further renumbering. [`post-lesson4-plan.md`](post-lesson4-plan.md) supplies supporting architecture context and implementation handoff guidance, not a competing curriculum sequence. If the two disagree, follow this README and reconcile the supporting document.

## Replacement state-machine lesson plan

### Teaching approach and common requirements

The progression is **mental model → one streamed exchange → a continuing conversation with tools**. Cover the conceptual ground of Lessons 3–6 through the state machine without forcing a one-to-one correspondence.

Class time and Rust familiarity are limited. Coding lessons supply the **entire implementation** in their `src/main.rs` from the outset, including all supporting Session, store, loader, effect-handler, effect-type, event-consumer, and safety code. There are no missing-code exercises or requirements to reconstruct Rust scaffolding. The classroom rhythm is **predict → run → explain → tweak → run again**. Success means learners can explain and change behavior, not reproduce the Rust implementation from memory.

Use the GDK's terms consistently; its names are part of the mental model. Define each concept and its purpose before showing its code-level form. Explain the supporting infrastructure, but do not turn these lessons into a Rust traits, generics, ownership, locking, or async-concurrency course. Keep source comments concise and put extended discussion in lesson prose and instructor notes.

The primary conceptual reference is [`Goose GDK State Machine — Working Mental Model`](reference-material/Goose%20GDK%20State%20Machine%20%E2%80%94%20Working%20Mental%20Model.md). It describes upstream Goose as of 2026-10-05, including explicitly unverified points. Distinguish the reusable GDK engine from Goose's full application assembly; verify any implementation claim against the exact pinned SDK. Do not teach Goose's full operation list as something this application must implement.

### Lesson 7 — GDK state-machine mental model (no code)

**Goal:** learners can narrate how the machine decides what work happens next, where recorded state lives, and why saving a result changes the next decision.

**Prerequisite:** completed Lessons 1–6, especially the raw request/result round trip. No provider or Rust execution is required for this lesson.

**Five ideas, using the GDK terms:**

1. **Session:** recorded conversation and relevant session information, loaded from a store owned by the application. It is not a registry of Operations and their completion statuses.
2. **Operation:** a rule that examines the Session **and any other information it has access to**, then declines or performs work. Checking and acting belong to the Operation. Steer illustrates the qualification: Goose's Steer Operation also consults a live queue of user guidance outside the Session. Do not implement Steer here.
3. **StateMachine:** evaluates an ordered list of steps. The first applicable step acts; after its Effects are applied, checking begins again from the top. In our teaching assembly, inference is the final fallback step.
4. **Effect:** a proposed change to recorded data. An Operation returns Effects; the application's effect handler applies them to its store. An Effect is not an Operation-status update.
5. **Re-evaluation:** each pass reloads the Session and decides what applies now. The newly recorded messages change the work that applies; the machine is not advancing a remembered program counter through Operations. Live external inputs can also change the next decision.

**Teaching sequence:** motivate coordination from Lesson 6; establish the five ideas; walk a simple question/answer story; then walk the familiar planned-loss tool round trip. Use plain-language diagrams and message/state tables, **not Rust or pseudocode**. Distinguish one user turn, one machine run, and the multiple passes that may occur within a run. Explain that no applicable step or an explicit yield returns control to the application; acknowledge cancellation/errors and application limits without a deep failure-path lesson. A finished run does not itself wait for new user input.

Explain store lifetime honestly: in-memory state can survive passes and successive runs in the same process, but is not durable across process exit. Recorded history is not necessarily the complete input needed to reproduce future decisions; do not claim crash recovery replays external actions exactly once.

**Required lecture visuals:** diagrams and rendered images are core teaching material, not optional decoration. Introduce them progressively rather than opening with one dense architecture diagram:

- **Who owns what:** show the application's store, a loaded Session, the StateMachine, Operations, and Effects. Draw a separate external-input path into an Operation so the Session is not pictured as its only possible input; use Steer's live queue as a clearly labeled Goose example.
- **One pass and re-evaluation:** show loading the Session, checking Operations in order, stopping at the first applicable step, applying its Effects, and returning to the top with newly loaded state. Include the no-applicable-step/yield exits without depicting Operations as queued tasks with completion statuses.
- **Simple exchange, frame by frame:** show the initial user message, inference's proposed reply Effect, the updated recorded conversation, and why the next pass has no work. Keep the Session before/after beside the relevant pass rather than hiding state inside an abstract loop.
- **Planned-loss round trip, frame by frame:** show inference → tool execution → inference, with Session/message snapshots at each boundary. Visually connect the tool request and response by the same correlation ID and show the deterministic `100.00` result. Label this as a narrated conceptual example, not observed model output or a guarantee of tool selection.

Use consistent GDK names, actor labels, arrow meanings, and a small visual legend across the images. Make them readable on a classroom projector; do not rely on color alone. Each image needs a caption/text equivalent, and instructor notes should specify its reveal order, what to point at, and a prediction question for learners. Supply editable diagram sources plus rendered images that can be viewed without a diagram-rendering plugin (for example SVG with PNG exports) under the lesson's `diagrams/` directory. Choose the simplest maintainable rendering tool during implementation; diagrams are conceptual illustrations, not Rust or pseudocode exercises. Do not copy the existing research diagrams wholesale or alter unrelated instructor artwork.

**Observable checkpoint:** given a narrated trace, learners identify the Session, applicable Operation, proposed Effect, store update, and reason the next pass differs. They explain why the first applicable step wins and why an Operation may read beyond the Session.

**Deliverables when authorized:** `lessons/07-state-machine-mental-model/LESSON.md` and `instructor-notes/07-state-machine-mental-model.md`; editable diagram sources and rendered lecture images in `lessons/07-state-machine-mental-model/diagrams/` are required alongside them. This no-code lesson intentionally has **no `src/main.rs`**, code snippets, or Cargo run requirement. Its validation is an instructor-led trace walkthrough and terminology/source review.

### Lesson 8 — One streamed request/response through the state machine

**Goal:** connect Lesson 7's mental model to a complete running application, with one provider exchange and two visible paths back to the application: display Events and saved Effects.

**Prerequisite:** Lesson 7 and the working provider setup from Lessons 2–6.

**Scenario and teaching sequence:**

1. Load provider configuration as before: `.env`, default JSON path with optional positional override, first configured model. Seed one Session with one simple user question. A default such as “What is the capital of France?” is sufficient; no tools, follow-up input, or new domain-policy layer.
2. Load and display the Session **before** the run: its ID and deliberately selected conversation fields, not a sprawling Rust debug dump.
3. Assemble the smallest inference-only machine using the shipped `InferenceRunner` and call `StateMachine::run`. Let the GDK own the pass loop; do not reintroduce the old learner-facing hand-written loop.
4. Receive and display Events **while the run is executing**. The inference step uses the **Emitter** to send **Events** to the application; the application owns their presentation. Explain this path before the channel/task code that supports it.
5. Explain the other path: the inference step reconstructs complete response messages and returns **Effects**, which the application applies to recorded state. Displaying streamed fragments does not save the response, and a streamed Event need not be a complete message.
6. Display the final Session returned by the successful run. In the ordinary text-only case, the before view has the user question and the after view has the user question plus the saved assistant reply. Explain why inference no longer applies and control returns to the application.

**Implementation requirements:** provide all code, including the small in-memory runtime and SDK-required effect implementations. Consume Events concurrently with the machine, not by draining a large buffer only after inference/run completes. Establish clean consumer shutdown and drain remaining Events before the final Session view. Keep persisted-state snapshots visually distinct from provider traffic; reuse `++++++++` payload delimiters and actor labels at actual payload sites. Do not falsely label a Session snapshot as the exact provider request or a later buffer drain as live streaming.

**Tweak:** change the user question, predict the new answer, and observe that the machine structure and before/after relationship remain the same. Do not require learners to implement loader/handler traits, message-ID plumbing, or concurrency machinery.

**Success criteria:** one successful provider exchange; visible response Events consumed during the run; complete reply retained in the store; understandable Session views before and after; normal run completion. Learners distinguish Events/Emitter (presentation) from Effects/effect handler (recorded state) and explain the stop. Validate structure, not exact model wording or token counts.

**Out of scope:** tools, multi-turn input, production storage, a general CLI, model selection, Goose's full assembly, or deliberate cancellation/failure exercises.

**Deliverables when authorized:** `lessons/08-machine-request-response/LESSON.md`, complete `lessons/08-machine-request-response/src/main.rs`, and matching `instructor-notes/08-machine-request-response.md`. The withdrawn draft with the same directory name remains under `drafts/`; it is reference only, not the implementation to restore wholesale.

### Lesson 9 — One Session, successive user turns, and selective tool use

**Goal:** demonstrate conversation continuity and tool selection through the same state-machine foundation, preserving the protocol and safety boundaries established in Lessons 5–6.

**Prerequisite:** Lesson 8. Reuse its Session/runtime, event-display path, and effect vocabulary wherever possible; explain any required changes instead of claiming only the step list changed.

**Scenario and teaching sequence:**

1. Register the existing `maximum_planned_loss` capability **from the first turn**, through the GDK tool operation alongside inference. The same capability remains available on both turns.
2. Ask: **“What is stop-loss referring to in day trading?”** Run the machine and display the answer and saved conversation. Expected behavior is a conceptual explanation **without a tool request**: there is no calculation to perform.
3. Append a follow-up to the **same Session** asking for the **maximum planned loss** for a hypothetical long position with entry price `$51.20`, stop price `$50.70`, and `200` shares. Start another machine run; do not discard the first exchange or start a new Session.
4. Observe the saved assistant tool request (name, arguments, correlation ID), validated deterministic execution, correlated tool response, and final educational explanation using `$100.00`. Identify fees, slippage, and gaps through the stop as exclusions; planned loss is not expected statistical loss or a guaranteed realized-loss ceiling.

Use two scripted user turns initially; an interactive input loop and input CLI are not needed. The application appends each new user message and starts a run. Inside the calculation run, the expected inference → tool execution → inference progression happens through multiple passes. Availability does not imply necessity: the model chooses whether to request the tool, while the application controls executable capabilities.

**Observation, not a guarantee:** explicitly inspect whether the model incorrectly requests the calculator for turn 1, and whether it correctly requests it for turn 2. Report unexpected behavior as a provider/model outcome; do not hide it, force tool selection, remove the tool on turn 1, silently substitute a mock, or claim the machine prevents unnecessary calls. Invalid or incomplete arguments must still be rejected safely. A run with the wrong tool-selection pattern is not evidence that the intended classroom scenario validated successfully.

**Safety and protocol continuity:** retain one allowlisted calculation, strict argument shape/unknown-field rejection, decimal strings parsed immediately with exact arithmetic, at most four decimal places, positive prices/share count, entry above stop, two-place dollar output, unchanged request/result IDs, handling of all blocks and requests, and explicit execution bounds. A typed adapter may organize these responsibilities but must not erase validation or observability. No trading, shell, arbitrary-network, market-data, or retrieval capability.

Map the manual responsibilities from Lessons 5–6 to the GDK assembly: advertise a definition, inspect/request, validate/execute, return a correlated response, save history, and infer again. Explain how Operations can contribute tools and prompt parts before inference as well as perform work. Carry forward the familiar educational instruction as needed, explaining how it reaches inference; reserve the fuller assistant contract for Lesson 10.

**Tweak:** change `200` shares to `100`, predict `$50.00`, and compare the deterministic result and final explanation while checking that correlation and conversation continuity still hold.

**Success criteria:** both runs use the same Session and retain the first exchange; turn 1 makes no tool request; turn 2 produces a valid request, matching response, exact `$100.00` result, and grounded explanation; Events and saved state remain distinguishable; runs terminate under explicit safeguards. Learners distinguish a new user turn/new run from another pass within the current run. Record actual structure rather than asserting an exact message count or generated wording.

**Deliverables when authorized:** `lessons/09-machine-conversation-tools/LESSON.md`, its complete `src/main.rs`, and matching `instructor-notes/09-machine-conversation-tools.md`.

### Implementation handoff and review gates

**Current authorization:** write/update this plan and supporting guidance only. No new lesson directories, Rust implementation, dependency changes, or root-source reset are authorized by the planning request. The instructor will review this plan before implementation begins.

For a new implementation chat:

1. Read this README first, then `AGENTS.md`, the current-decision/handoff sections of `post-lesson4-plan.md`, and the mental-model reference. Inspect Git status/diffs and preserve unrelated work.
2. Read the manifest, lockfile, toolchain, provider JSON, root source, and complete Lessons 3–6 with their instructor notes. Read archived drafts only to identify useful evidence and pitfalls, not as accepted curriculum.
3. Verify the exact pinned GDK/RMCP sources and official documentation: machine loading/run/application, inference streaming, Events/Emitter, effect requirements, tool registration/dispatch and correlation. The mental-model reference's upstream claims and open questions are not substitutes for this verification.
4. Implement **one authorized lesson at a time**, beginning with Lesson 7 unless the instructor chooses otherwise. Do not scaffold future lessons. Lesson 7 does not change root Rust; confirm the root update when starting a coding lesson, because the root currently contains the withdrawn Lesson 8 draft.
5. Before coding Lesson 8, resolve the streaming-consumer lifecycle and explicit execution-limit design. Pinned `StateMachine::run` has no application-specific step-limit argument. Keep the GDK run loop and choose the smallest source-verified safeguard; bring uncertain design/tradeoffs to the instructor rather than silently reinstating the old bounded manual loop or dropping bounds. Before Lesson 9, verify safe unknown-name handling, all-request handling, and request limits in the chosen tool adapter. Exact limit values are implementation decisions to document, not inherited guarantees from old spikes.
6. Supply complete source, explanations, expected traces, instructor pacing, and the small predict/run/tweak activities together. Publish a precise root-based run workflow for the shared manifest; lesson sources are not independent Cargo packages. Keep sample output honest about streamed versus persisted data.
7. For coding changes run `cargo fmt --check`, `cargo check`, `cargo clippy --all-targets`, `cargo test`, and the documented live-provider run. Unit tests cover deterministic validation, effect application, state continuity, and safeguards; they do not replace live streaming/tool-selection validation. If the provider is unavailable, ask the instructor to start or supply one. Record what passed, failed, or remains unvalidated before changing lesson status.

**Still to resolve during implementation:** exact minimal runtime/effect shapes, SDK-compatible typed tool adapter, event-task shutdown, source-verified bounds, and the final prompt wording for the numeric follow-up. These are not reasons to redesign the agreed teaching progression. Ask before changing scope or adding deliberate failure-path activities. Instructor review and release remain separate from a successful build or provider run.

## Repository organization

```text
.
├── src/main.rs                 # current exercise; changes as the course advances
├── lessons/                    # explanation plus complete reference source
├── instructor-notes/           # instructor-only pacing and teaching guidance
├── drafts/                     # withdrawn lessons and their instructor notes
├── post-lesson4-plan.md        # supporting architecture context and implementation handoff
├── reference-material/         # durable background references; not instructor-only (state machine mental model)
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

The root `src/main.rs` is the runnable classroom working copy, not a stable or production-ready application. Learners run and tweak complete supplied code rather than fill in missing implementation. The root currently retains the **withdrawn Lesson 8 implementation**; it has not been reset to Lesson 6 or advanced to a replacement lesson. It is draft code, not the active teaching checkpoint. Source and dependencies remain unchanged pending plan review and authorization to implement a coding lesson.

### Lesson directories

Coding lessons are instructional references with complete supplied applications:

- `LESSON.md` explains the lesson and highlights important code snippets.
- `src/main.rs` contains the entire reference implementation from the outset, ready to run and tweak.

Lesson 7 is intentionally conceptual: it has prose, diagrams, and instructor notes, but no Rust source.

Lesson source is reference material, not an independently packaged application. The root manifest and locked dependencies are shared. The complete solution is already present in each lesson directory; it is not copied from the root at the end of class.

## Prerequisites and setup

The course currently targets **Linux**. Install the Rust toolchain pinned in [`rust-toolchain.toml`](rust-toolchain.toml), which initially specifies Rust `1.94.1`, plus Git and access to a compatible model provider.

From the repository root, let Cargo fetch and build the exact dependency resolution recorded in `Cargo.lock`:

```bash
cargo check
```

The manifest pins `goose-providers` and `goose-agent` to exact `0.1.0-alpha.11` releases; the provider crate enables its `rustls-tls` transport feature. `goose-providers` supplies native provider construction, messages, streaming, and the raw tool protocol, and `rmcp` supplies the MCP tool type used to advertise deterministic tools. `goose-agent` supplies the GDK state machine, operations, effects, emitter, and the session/effect traits (retained from the withdrawn state-machine drafts). `serde`'s derive feature deserializes untrusted tool arguments into domain structs, and `rust_decimal` supplies exact decimal arithmetic for money-related tool arguments (both added with Lesson 6). `anyhow`, `async-trait`, and `tokio-util` support the state machine's error type, async traits, and cancellation token (retained from the withdrawn state-machine drafts). Each lesson that adds a new requirement instructs adding it to the root manifest. The application does **not** use the `goose-sdk` foreign-language binding surface. `futures` supplies stream consumption and `dotenvy` loads a local `.env` before a provider is constructed.

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

## Current root workflow

There is no active replacement state-machine exercise yet. `cargo run` still runs the withdrawn Lesson 8 code and contacts the configured provider; it does not run the completed Lesson 6 reference. Consult [`lessons/06-deterministic-tool-response/`](lessons/06-deterministic-tool-response/) for the last completed lesson. Resetting the root exercise is a separate instructor decision.

Provider-calling lessons require live-provider validation. If the endpoint is unreachable, ask the instructor to start or supply one. Marking a lesson Complete does not by itself approve it for teaching — that release decision remains with the instructor.

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