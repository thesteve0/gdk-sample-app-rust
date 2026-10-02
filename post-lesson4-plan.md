# Post-Lesson 4 Curriculum and Architecture Plan

## Status of this document

This document records the accepted direction for the course after Lesson 4. It is durable context for future curriculum discussions and implementation work, not a released lesson.

The post-Lesson 4 technical spike is complete and supports the sequence below. The instructor accepted the outline on 2026-09-29. Lesson numbers and boundaries should now remain stable unless later implementation evidence or an explicit instructor decision requires a change. The instructor still owns curriculum sequencing and release decisions.

On 2026-10-01 the instructor ordered exactly such a change: a new Lesson 8 — one request and one response through the GDK state machine, the introductory walkthrough for the machine — was inserted before the former Lesson 8, and every lesson from the former Lesson 8 onward was renumbered by one (former 8→9 through former 17→18). The sequence below reflects that renumbering; this paragraph is the explicit instructor decision the stability clause above requires.

Later on 2026-10-01, after Lesson 8 was written, the instructor judged Lessons 7 and 8 to be in reverse order and a mess as they now stand. Both lessons must be revisited and fixed, but not yet: the fix is deferred until a research spike on teaching the state-machine approach in the GDK concludes. The spike lives in `spike/state-machine/` and is explicitly research-only — none of its material may modify `lessons/`, `instructor-notes/`, `README.md`, this plan, or the root source until the instructor declares the spike complete and orders the teaching material updated.

## Course application goal

The course incrementally builds a **day-trading teaching assistant that can also analyze trades**.

The application should:

- teach day-trading concepts and risk-management principles;
- analyze hypothetical and user-proposed trades;
- use deterministic tools for arithmetic rather than trusting model calculations;
- use read-only tools for time-sensitive market observations;
- retrieve educational evidence from a private corpus;
- reach a final conclusion when asked;
- always present the evidence, assumptions, uncertainty, and reasonable alternatives behind that conclusion;
- explain what observations would weaken or change its conclusion.

The application must not:

- place, modify, or cancel trades;
- receive brokerage credentials;
- expose a broker or order-entry tool;
- gain an indirect execution path through a generic shell or arbitrary HTTP tool;
- claim to possess current market information unless a tool supplied timestamped evidence.

The no-trade-execution rule is an architectural capability boundary, not merely a disclaimer or system-prompt instruction.

## Agreed instructor decisions

1. The application is primarily a **teaching assistant that can also analyze trades**, not primarily a stock recommender.
2. It may give a final conclusion, but it must present evidence and alternatives.
3. No market-data provider has been selected yet.
4. The intended educational corpus consists mainly of privately owned EPUB or PDF books. The source books must not be committed to Git.
5. The course should use the latest published GDK generation that includes the state-machine agent loop. Another coordinated upgrade is likely before the class is taught.
6. Learners should understand the raw tool-request/tool-result protocol before using the state-machine abstraction.
7. The course needs diagrams for both the raw tool round trip and the GDK state machine.

## Current dependency and validation state

The repository has been upgraded from `goose-providers 0.1.0-alpha.10` to the latest published state-machine generation available during this planning session:

- `goose-providers = 0.1.0-alpha.11`
- `goose-agent = 0.1.0-alpha.11`
- transitive `goose-provider-types = 0.1.0-alpha.11`
- Rust `1.94.1`

The versions remain exactly pinned because these crates are alpha.

The existing Lesson 4 application was validated after the upgrade with:

- `cargo fmt --check`
- `cargo check`
- `cargo clippy --all-targets`
- `cargo test`
- IDE project build
- `cargo run` against the configured live local provider

The two-turn Lesson 4 conversation continued to work after the upgrade.

Before teaching the course, review the current public GDK release and perform another coordinated dependency/source/documentation review rather than changing one GDK crate independently.

Local checkout note (recorded during the Lesson 7 session): the working copy at `/var/home/stpousty/git/goose` (branch `main` @ `1ce7de7a3`) was diffed against published `goose-agent = 0.1.0-alpha.11` and `goose-provider-types = 0.1.0-alpha.11`. All differences are a wasm32-compatibility refactor (`MaybeSend`/`MaybeSync` bounds, conditional `async_trait(?Send)`), transparent on native Linux targets, plus one behavioral change noted in the Lesson 9 section below: typed `SyncTool` invocation now goes through `tokio::task::spawn_blocking`. The provider-types conversation module, `events.rs`, and `lib.rs` are identical to alpha.11.

## Relevant GDK findings

### Raw provider protocol

`goose-providers` exposes the provider-level protocol needed to teach the mechanism directly:

1. Send conversation messages and tool definitions to inference.
2. Receive an assistant message containing one or more structured tool requests.
3. Inspect and validate each request.
4. Dispatch only a known, allowed tool.
5. Preserve the assistant tool-request message in conversation history.
6. Add a correlated user-role tool response using the request ID.
7. Invoke inference again.
8. Continue until the assistant returns a final answer or an application limit is reached.

Important teaching details:

- Advertising a tool does not execute it.
- A response can contain multiple content blocks and multiple tool calls.
- Tool-call parsing can fail inside a request; do not blindly unwrap it.
- Tool names and arguments are model output and therefore untrusted input.
- Tool results must retain the request/result correlation ID.
- The application—not the model—decides what is executable.

### State-machine agent loop

The latest published `goose-agent` crate describes itself as the GDK agent loop assembled as a state machine. Relevant concepts include:

- `StateMachine`
- `Operation`
- `Inference`
- `ToolOperation`
- typed synchronous and asynchronous tools
- `ToolProvider`
- `MachineSession`
- `SessionLoader`
- `EffectHandler`
- persisted conversation effects
- emitted events and cancellation

The state machine repeatedly reloads persisted conversation state, checks ordered operations, applies and persists effects, and continues until an operation yields or no operation applies.

The course should not introduce all of these abstractions before learners see the protocol they coordinate. The raw round trip gives learners a concrete model for understanding why operations, effects, sessions, and repeated state derivation exist.

### Retrieval is separate

Neither the provider API nor the agent loop supplies a complete application-specific RAG pipeline. The course will need separate ingestion, extraction, chunking, embedding/indexing, storage, and retrieval choices.

Retrieval can be exposed to the agent as a read-only tool. That keeps the decision and evidence path observable:

1. The model determines that external educational evidence is needed.
2. It requests retrieval.
3. The application searches the private knowledge base.
4. The tool returns passages and provenance.
5. The model synthesizes an answer and cites the returned evidence.

A direct document attachment is not the same as RAG.

## Evidence model for the application

The course should keep three evidence-bearing capability categories separate.

### 1. Deterministic tools

Examples:

- percentage change;
- profit and loss;
- risk/reward ratio;
- maximum planned position loss;
- hypothetical position size from an explicitly supplied risk budget.

These are suitable for the first tools because they are deterministic, testable, independent of network availability, and clearly demonstrate why arithmetic should not be delegated to probabilistic generation.

### 2. Read-only time-sensitive tools

Examples:

- quotes;
- historical bars;
- market-session state;
- company events;
- eventually, carefully selected news sources.

A market observation should include enough metadata to evaluate its meaning:

- symbol and instrument identity;
- source;
- observed timestamp and timezone;
- price and currency;
- market/session state;
- whether it is historical, delayed, or real time;
- freshness or staleness status;
- explicit failure information.

Price data and news should not be the first tool lesson. They introduce authentication, rate limits, data licensing, ticker ambiguity, timestamps, market hours, stale data, network failures, and reproducibility concerns.

News should arrive later than structured price data because it also introduces source credibility, syndication, misinformation, publication/update semantics, licensing, and prompt injection in untrusted text.

### 3. Educational retrieval

Retrieval should ground teaching in relatively stable material. It is not a substitute for current market data.

Retrieved evidence should include provenance such as:

- document identity;
- title and author when available;
- edition or file fingerprint;
- PDF page or EPUB chapter/location;
- matched passage;
- retrieval score;
- document date/version when relevant.

The assistant must be able to say that the corpus contains insufficient evidence. A citation does not automatically make a conclusion current, correct, or suitable.

## First deterministic tool recommendation

The first tool should be a hypothetical position-risk or maximum-planned-loss calculator.

Example inputs:

- entry price: `$51.20`;
- stop price: `$50.70`;
- share count: `200`.

Expected result before fees, slippage, or a gap through the stop: `$100`.

This tool is preferred over a generic calculator because it teaches an actual day-trading risk concept while keeping the mechanism deterministic and easy to validate.

### Financial arithmetic decision

Do not casually imply that binary floating-point gives exact currency arithmetic.

The spike compared integer minor units with decimal strings parsed by `rust_decimal`. Both represented the example exactly, but the instructor-facing recommendation is now accepted: use **decimal strings at the model/tool boundary and parse them immediately into exact decimal arithmetic**. This keeps arguments natural (`"51.20"`, not `5120` cents), while making parsing and validation explicit. The spike permits at most four decimal places and formats dollar outputs to two places.

Integer minor units remain a valid internal design when a fixed scale has already been established, but cents are less natural in a first tool schema and cannot represent sub-cent prices without selecting another scale. Do not use binary floating point for Lesson 5 currency calculations.

The spike also validated an explicit failure/termination policy for raw-protocol exploration: at most 3 raw inference rounds, 4 tool requests, and 6 state-machine steps per run. The principle — imposing explicit round/request or state-step bounds — carries into every tool and state-machine lesson; exact limits may be retuned per lesson.

## Accepted post-Lesson 4 sequence

The completed spike established that the state-machine transition should be split across two lessons. The instructor later inserted a third (the new Lesson 8) so the machine is walked through a single provider request and response before tools re-enter. The sequence below is accepted; do not collapse the state-machine lessons (7, 8, and 9) merely to shorten the roadmap.

### Lesson 5: Define a deterministic trading tool and inspect its raw request

Introduce:

- deterministic computation versus model reasoning;
- MCP tool name, description, and input schema;
- passing the tool definition to provider inference;
- inspecting structured assistant content blocks;
- preserving the request ID;
- recognizing that a tool request is not a final answer and has not executed anything.

Stop before execution so the request boundary remains visible.

### Lesson 6: Execute and return the raw tool result

Introduce:

- model arguments as untrusted input;
- deserialization and domain validation;
- explicit allowlisted dispatch;
- deterministic Rust execution;
- assistant tool-request history;
- correlated user-role tool response;
- follow-up inference that explains the result educationally;
- tool-level versus routing/protocol failures.

The final explanation should identify what the calculation excludes, such as fees, slippage, and gaps through the planned stop.

### Lesson 7: State-machine concepts and minimal runtime

Use the manual implementation to motivate:

- repeated inference/tool cycles;
- persisted conversation state;
- operations;
- effects;
- session loading;
- effect application;
- yielding and termination.

Build only the smallest in-memory session/runtime needed for the existing deterministic scenario. Do not introduce production storage or multiple tools.

### Lesson 8: One request and one response through the state machine

Inserted by explicit instructor decision on 2026-10-01. The instructor asked for a walkthrough of the state machine with the simplest possible scenario before tools re-enter:

- one seeded user question ("What is the capital of France?"), no tool calls;
- the GDK-shipped `InferenceRunner` registered as the sole `Step::Inference`;
- the machine's pass shape — reload, ask `applies()`, provider call, effects, stop checks — narrated end to end;
- the lesson-owned effect vocabulary the machine and the shipped runner require (`MachineEffect`, `From<Message>`, `InferenceEffect`), since `ConversationEffect` has no `InferenceEffect` implementation in pinned alpha.11;
- the hand-written bounded pass loop kept from Lesson 7, so the state-step bound stays explicit;
- the provider called on pass 1 only; pass 2 re-derives from the persisted reply and stops with "no step applies".

This lesson is the introductory discussion home for the state machine: the full pass loop, the state-machine diagrams, and the payload-delimiter conventions (including the streamed fence) are established here. The instructor explicitly deferred a multi-turn follow-up (letting the user ask another question) — the lesson stays one request and one response.

### Lesson 9: Assemble inference and the typed tool operation

Refactor the same scenario using:

- provider-backed inference;
- a typed deterministic tool;
- `ToolOperation`;
- the in-memory runtime;
- ordered state-machine steps;
- `StateMachine::run` or the narrowest suitable state-machine execution surface.

The pedagogical point is to map each manual protocol responsibility to the GDK abstraction that now coordinates it.

A behavioral note observed while reviewing the local goose checkout against pinned alpha.11: the wasm32-compatibility refactor changed how a typed `SyncTool` is invoked. In `goose-agent = 0.1.0-alpha.11` the sync tool runs inline; in the updated checkout `invoke_sync` runs the tool through `tokio::task::spawn_blocking` (with an inline fallback under wasm32). On native targets the `MaybeSend`/`MaybeSync` bounds are identical to `Send`/`Sync`, so the lesson's behavior is otherwise unchanged, but the invocation is now off the async executor thread. Lesson 9 material should describe the spawn_blocking invocation when it presents the typed tool, and re-verify against the exact pinned source before teaching.

### Lesson 10: Domain instructions and capability boundary

Define the actual assistant contract:

- teach before or while analyzing;
- distinguish evidence, assumptions, and model judgment;
- present alternatives and uncertainty;
- reach a final conclusion when asked;
- explain what would change the conclusion;
- never imply current data exists without a tool result;
- never imply that an order was placed.

Teach that prompts influence behavior while capability design limits possible actions.

### Lesson 11: Evaluation baseline

Create scenario-based checks before introducing live external data. Evaluate:

- correct tool selection;
- not calling a calculator for conceptual questions;
- argument validation;
- deterministic correctness;
- consistency between tool output and prose;
- evidence and assumptions being visibly separated;
- alternatives accompanying conclusions;
- abstention when required data is absent;
- no invented prices or sources;
- no claim of order execution;
- boundary retention across multiple turns;
- latency, usage, and local-model constraints where useful.

Prefer structural assertions and rubrics over exact generated prose. Maintain development and holdout scenarios to reduce prompt overfitting.

### Lesson 12: Fixture-backed read-only market data

Define a stable market-observation contract using historical fixtures first. This keeps the class reproducible and isolates the tool contract from provider selection.

Do not select a live source until its licensing, delayed/real-time semantics, API stability, authentication, rate limits, symbol identity, timestamps, caching terms, historical availability, and cost are researched.

### Lesson 13: Evidence-based trade analysis

Combine:

- an educational principle;
- a historical market observation;
- a deterministic risk calculation;
- alternatives and risk discussion;
- a final conclusion.

Evaluate visible evidence and conclusions, not private chain-of-thought.

### Lesson 14: Private educational document ingestion

Introduce local, private document handling separately from retrieval:

1. load a private local EPUB or PDF;
2. extract text and source-location metadata;
3. normalize while preserving useful citation locations;
4. chunk;
5. embed or otherwise index;
6. store local retrieval artifacts.

PDF page references and EPUB chapter/location references need deliberate source-location modeling rather than arbitrary chunk IDs.

### Lesson 15: Retrieval tool and citations

Expose educational retrieval as a read-only tool. Return passages with provenance and allow an insufficient-evidence result.

### Lesson 16: Retrieval evaluation and hostile content

Test:

- irrelevant retrieval;
- missing evidence;
- conflicting sources;
- citation accuracy;
- unsupported synthesis;
- stale educational material;
- instructions embedded in retrieved text.

Retrieved text is untrusted data and must not override system/application policy.

### Lesson 17: Combined teaching and trade analysis

Bring together:

- conversation;
- deterministic calculations;
- read-only observations;
- educational retrieval;
- citations;
- alternatives;
- a final evidence-based conclusion.

### Lesson 18: CLI, model selection, and operational errors

Move explicit model selection later so it serves a meaningful purpose. Compare models based on:

- reliable structured tool calling;
- context requirements;
- streaming behavior;
- retrieval and citation behavior;
- boundary retention;
- latency and cost;
- local hardware constraints.

The CLI can then support provider/model selection, scenario inputs, private corpus paths, and clear error reporting.

### Optional capstone: Open-weights model comparison or fine-tuning

Fine-tuning is an optional experiment after the application has:

- a stable task definition;
- a baseline model;
- deterministic tools;
- retrieval;
- representative failure data;
- development and holdout evaluations.

Fine-tuning is not the solution for current prices, changing regulations, deterministic arithmetic, citation provenance, or the no-execution capability boundary.

A safer first experiment uses course-authored examples to improve teaching style, response structure, or tool-selection behavior. Directly training on copyrighted private books introduces materially different licensing, memorization, extraction, and model-distribution risks and should not be the default course path.

## Private corpus policy

The following private or derived artifacts must stay out of Git:

- original EPUB and PDF books;
- extracted text;
- rendered page images;
- chunks containing book text;
- embeddings;
- vector indexes or local retrieval databases;
- annotations that reproduce substantial protected content;
- fine-tuning datasets derived from the books;
- model adapters or weights that may memorize the corpus.

The repository may contain:

- course-authored or permissively licensed sample documents;
- ingestion and retrieval source code;
- configuration examples;
- documentation for selecting a private local directory;
- structural test fixtures that do not reproduce protected text.

Add explicit ignore rules when concrete local corpus/index paths are chosen. Do not invent paths before the ingestion design is selected.

Ownership of a book does not automatically imply permission to redistribute it or its derived textual artifacts. The course should teach a private local workflow and avoid making universal legal claims about indexing or training on copyrighted material.

## Required architecture diagrams

### Raw provider sequence diagram

The diagram should show:

```text
User message
    |
    v
Provider inference + tool definitions
    |
    v
Assistant ToolRequest (name, arguments, ID)
    |
    v
Application validates and dispatches
    |
    v
Deterministic Rust tool
    |
    v
User ToolResponse with matching ID
    |
    v
Follow-up provider inference
    |
    v
Final educational response
```

It must emphasize that the model requests a capability but the application authorizes and executes it.

### GDK state-machine diagram

The diagram should show:

```text
Reload persisted session/conversation
    |
    v
StateMachine checks ordered steps
    |-- ToolOperation handles a pending known request
    |-- Inference handles conversation requiring a model response
    v
Conversation effects
    |
    v
Runtime persists effects
    |
    +-- repeat from reloaded state
    +-- yield / stop when no operation applies
```

Include a mapping table:

| Raw responsibility | State-machine concept |
|---|---|
| Inspect a pending request | `ToolOperation` |
| Execute a known tool | Registered typed tool or provider |
| Call the model | `Inference` |
| Append request/result | Conversation effects |
| Retain history | Session plus effect handler |
| Repeat | `StateMachine` execution |
| Return control or stop | Yield/no applicable operation |

Use diagrams to explain behavior and responsibility, not merely list Rust type names.

## Completed technical spike and accepted findings

The isolated implementation and detailed evidence are in `spikes/post-lesson4/`. It is not a released lesson and must not prematurely establish final application architecture.

The spike demonstrated:

1. The configured local endpoint emitted native structured requests for the deterministic tool in 4/4 recorded raw runs; no tool shim, model change, or prompt workaround was required for this sample.
2. The raw `alpha.11` request/result round trip worked with complete streamed-message reconstruction, validated allowlisted dispatch, user-role responses, matching request IDs, and follow-up inference.
3. Final answers consistently used the deterministic `100.00` result and identified fees, slippage, and gap-through-stop risk.
4. The same capability worked as a typed RMCP `SyncTool` registered through `goose-agent::tool::ToolOperation`.
5. A minimal in-memory `MachineSession`, `SessionLoader`, and `EffectHandler` supported provider-backed `InferenceRunner` and the state machine. Two recorded runs applied `llm -> tools -> llm` and then stopped when no operation applied.
6. Application-specific bounds must remain explicit. The spike limits raw rounds and aggregate requests, and wraps `StateMachine::step`/`apply` with a maximum applied-step count because alpha.11 `StateMachine::run` has no application-specific step-limit parameter.
7. Streamed raw messages must be reconstructed with GDK merge semantics. An initial implementation that retained deltas independently read only the final `.`; accumulating through `Conversation::push`, as `InferenceRunner` does, fixed it.
8. Decimal strings parsed into `rust_decimal` are the accepted Lesson 5 representation.
9. The state-machine transition requires two lessons. Typed tools, schemas, effects, usage effects, sessions, loader/handler traits, events, cancellation, and termination are too much for one focused unit. Keep most runtime scaffolding instructor-provided when first introduced.
10. Structural tests can cover exact calculations and malformed/unknown arguments, while live-provider runs remain necessary for native structured-call behavior.

These runs establish compatibility and encouraging repeatability, not broad reliability across models and servers. Revalidate against the exact classroom provider before release. Do not turn spike code directly into a lesson without removing exploratory complexity.

## Topics deliberately deferred

Defer or exclude from the core path until the bounded teaching assistant works:

- broker integration and order APIs;
- autonomous scheduling or continuous market monitoring;
- generic shell or arbitrary network tools;
- live news and social sentiment;
- portfolio/account persistence;
- personalized suitability or tax profiling;
- complex backtesting engines;
- production vector infrastructure;
- book-derived fine-tuning.

Broker/order execution should remain outside the intended application, not be treated as a later advanced feature.

## Documentation and maintenance implications

- Keep the root README aligned with the application goal and provisional roadmap.
- Update lesson prose, reference source, root workflow, and instructor notes together when behavior changes.
- Keep exact alpha GDK versions coordinated across directly used crates.
- Consult official documentation and exact pinned source for every GDK API or behavior.
- Before the class, repeat source/API research and rerun all checks after the anticipated GDK upgrade.
- Keep domain examples focused on teaching and hypothetical analysis until grounded data tools exist.
- Do not imply that system instructions, citations, or model fine-tuning alone provide safety.

## Official and source references used during planning

Official documentation consulted:

- <https://goose-docs.ai/docs/getting-started/providers>
- <https://goose-docs.ai/docs/guides/managing-tools/index>
- <https://goose-docs.ai/docs/guides/managing-tools/tool-permissions>
- <https://goose-docs.ai/docs/guides/tool-shim>

Source/API research consulted:

- exact published `goose-providers 0.1.0-alpha.10` and companion type sources when evaluating the original plan;
- published `goose-providers 0.1.0-alpha.11` and `goose-agent 0.1.0-alpha.11`;
- the latest local Goose checkout, especially `crates/goose-agent/README.md`, `src/machine.rs`, `src/tool.rs`, and related provider/type sources.

The public `goose-agent` source and tests currently provide important implementation detail beyond the high-level official documentation, so future work must continue to verify behavior against the exact pinned crate source.

## Lesson 5 handoff for the next Goose session

Do not begin by redesigning the roadmap or promoting the spike wholesale. The next task is to design and implement the smallest teachable Lesson 5 that matches the accepted boundary.

Start by reading, in order:

1. `AGENTS.md`, `README.md`, and this document;
2. `git status` and diffs, because the working tree intentionally contains instructor work;
3. `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, and the provider JSON;
4. root `src/main.rs` plus all Lesson 4 prose, source, and instructor notes;
5. exact pinned `goose-providers`, `goose-provider-types`, RMCP, and (for later lessons) `goose-agent` source.

Lesson 5's required endpoint is an assistant response containing a structured request for `maximum_planned_loss`, not a completed calculation. Use the hypothetical long-position scenario with decimal-string entry `51.20`, decimal-string stop `50.70`, and integer share count `200`. Define one narrowly described tool with a strict object schema, pass it as the provider call's tools slice, inspect all streamed message/content blocks, print or otherwise expose the request name/arguments/ID, and preserve enough complete assistant state for Lesson 6. Reconstruct streaming deltas with GDK message merge semantics; do not assume one stream item is a complete response or one response has only one block/request.

Lesson 5 must explain:

- deterministic computation versus probabilistic model reasoning;
- tool name, description, and JSON input schema;
- decimal strings as an untrusted wire representation, with exact parsing deferred to execution in Lesson 6;
- advertising a tool versus authorizing/executing it;
- request IDs as future correlation keys;
- why every content block is inspected;
- the provider/model may fail to request a tool and that this is a structural behavior to validate live.

Lesson 5 must not yet introduce `rust_decimal` calculation code, dispatch, `CallToolResult`, tool responses, repeated agent loops, typed `SyncTool`, `ToolOperation`, sessions/effects, market data, retrieval, or model-selection CLI work. Add only dependencies genuinely required by Lesson 5; the spike's dependencies do not all belong in learner-facing code yet. Update root source, a new `lessons/05-.../LESSON.md` and complete reference `src/main.rs`, the matching instructor note, README status/root description, and affected documentation together. Include the required raw sequence diagram in Lesson 5, focused through the pending request boundary while previewing Lesson 6's result path.

Before calling Lesson 5 ready, run the repository checks and its documented live-provider command. Assert structure rather than exact prose, and report whether a native structured request was actually observed.