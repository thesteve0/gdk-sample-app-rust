# Post-Lesson 4 Curriculum and Architecture Plan

## Status of this document

**README.md is the authoritative curriculum plan**, including the replacement Lessons 7–9 and their implementation handoff. This document supplies supporting architecture context and historical validation evidence, not a competing sequence or a released lesson. Where older evidence conflicts with the README or exact pinned source, follow the README for curriculum and pinned source for API behavior.

The post-Lesson 4 technical spike is complete and supports the sequence below. The instructor accepted the outline on 2026-09-29. Lesson numbers and boundaries should now remain stable unless later implementation evidence or an explicit instructor decision requires a change. The instructor still owns curriculum sequencing and release decisions.

On 2026-10-01 the instructor ordered exactly such a change: a new Lesson 8 — one request and one response through the GDK state machine, the introductory walkthrough for the machine — was inserted before the former Lesson 8, and every lesson from the former Lesson 8 onward was renumbered by one (former 8→9 through former 17→18). That numbering was used before the 2026-10-05 replacement decision below; this paragraph preserves its provenance.

Later on 2026-10-01, after Lesson 8 was written, the instructor judged Lessons 7 and 8 to be in reverse order and a mess as they now stand. Both lessons must be revisited and fixed, but not yet: the fix is deferred until a research spike on teaching the state-machine approach in the GDK concludes. The spike lives in `spike/state-machine/` and is explicitly research-only — none of its material may modify `lessons/`, `instructor-notes/`, `README.md`, this plan, or the root source until the instructor declares the spike complete and orders the teaching material updated.

## Superseding instructor decision — 2026-10-05

Lessons 1–6 are complete. The instructor withdrew the existing Lessons 7 and 8 and requested a complete rework. Their material is now preserved in `drafts/lessons/` and `drafts/instructor-notes/`, unchanged. The earlier deferral no longer prevents this explicitly requested archival move and status update; it does not authorize promoting spike material into new lessons.

The instructor subsequently authorized writing a detailed replacement plan for review. The previous conversation-only restriction is superseded for **planning documentation only**, not lesson implementation. The [replacement plan in README.md](README.md#replacement-state-machine-lesson-plan) is authoritative:

- **Lesson 7:** concise no-code grounding in Session, Operation, StateMachine, Effect, and re-evaluation through one simple question/answer round trip only. The instructor deferred the planned-loss trace to Lesson 9 on 2026-10-05. Operations may also use information outside the Session; Steer is the explanatory example, not a feature to implement.
- **Lesson 8:** complete supplied application for one streamed request/response using the GDK run loop. Show the Session before/after and distinguish Events sent through the Emitter for display from Effects applied to recorded state. Consume Events during execution, not after buffering an entire run.
- **Lesson 9:** reuse the foundation for two turns in one Session with the same planned-loss tool available throughout. First ask what stop-loss means and expect no tool call; then ask for maximum planned loss for entry `51.20`, stop `50.70`, and `200` shares, observing a correlated tool round trip and `100.00` result. Unexpected first-turn tool use must be observed and reported, not concealed.

The class uses complete code, not fill-in exercises: predict, run, explain, tweak, and compare. Lesson 7 has no source file. Lessons 8 and 9 have complete source in their numbered lesson directories plus prose and instructor notes. The former planned Lesson 9 is replaced by the combined conversation/tool lesson; Lessons 10–18 keep their numbers and later scope.

Root `src/main.rs` and dependencies remain unchanged and still contain the withdrawn Lesson 8 exercise. On 2026-10-05 the instructor subsequently authorized **Lesson 7 implementation only**, with README and the Working Mental Model as authoritative sources. The replacement no-code lesson, instructor notes, and editable SVG/PNG lecture frames are now authored and source/asset-reviewed; instructor-led trace validation, projector review, and release approval remain pending. Lessons 8–9 implementation and any root reset still require separate authorization. Archived drafts stay unchanged.

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

Local checkout note (recorded during the Lesson 7 session): the working copy at `/var/home/stpousty/git/goose` (branch `main` @ `1ce7de7a3`) was diffed against published `goose-agent = 0.1.0-alpha.11` and `goose-provider-types = 0.1.0-alpha.11`. All differences are a wasm32-compatibility refactor (`MaybeSend`/`MaybeSync` bounds, conditional `async_trait(?Send)`), transparent on native Linux targets, plus a historical reported difference in typed `SyncTool` invocation. That comparison is not a reliable description of the current pinned implementation: the installed alpha.11 `src/tool.rs` reviewed on 2026-10-05 itself uses `tokio::task::spawn_blocking` in `with_sync_tool`. Recheck the exact source when implementing; do not teach a claimed inline-versus-spawn difference based on this historical note. The provider-types conversation module, `events.rs`, and `lib.rs` are identical to alpha.11.

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

## Post-Lesson 4 sequence — supporting context

Lessons 5–6 below retain their completed boundaries. The replacement Lessons 7–9 are governed by the README plan; earlier constraints requiring a provider-free runtime lesson or a hand-written pass loop are withdrawn. Later lesson descriptions retain their direction and numbering.

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

### Lessons 7–9: Replacement state-machine progression

See [README.md — Replacement state-machine lesson plan](README.md#replacement-state-machine-lesson-plan) for the complete goals, scenarios, teaching boundaries, planned paths, success criteria, implementation questions, and review gates. Do not reconstruct these lessons from the archived draft or historical spike summaries.

Lesson 7's diagrams and images are required lecture deliverables, not optional supplements. The README specifies ownership, ordered passes, and one simple exchange with Session snapshots, the external-input distinction, editable/rendered assets, accessibility, and instructor walkthrough requirements. The planned-loss/correlation trace is deferred to Lesson 9. Create those assets only when lesson implementation is authorized; preserve unrelated research artwork.

Supporting technical cautions:

- `StateMachine::run` reloads between passes and returns a final loaded Session on success; the application can separately load the initial Session for comparison.
- `Emitter` sends Events to the application; inference separately accumulates complete messages into Effects. Concurrent consumption is required for honest live display and avoids blocking a producer on a full channel.
- The application supplies its own Session, loader, effect handler, and an effect vocabulary compatible with the pinned inference runner. All required implementations belong in the supplied code, not learner fill-ins.
- The pinned run loop has no application-specific step-limit parameter. Agree on a minimal source-verified safeguard while retaining the GDK-owned run loop; do not silently copy the old manual loop. State limits and tool-request limits must be explicit and validated.
- Inspect the exact tool adapter for unknown names, malformed arguments, correlated results, and multiple requests; do not assume registering a typed tool alone provides every Lesson 6 safety requirement.
- Operations may contribute tools and prompt parts before inference and may read live external inputs as well as recorded state. The custom application does not need Goose's entire operation assembly.

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
6. Application-specific bounds must remain explicit. Historically, the spike limited raw rounds and aggregate requests and wrapped `StateMachine::step`/`apply` with a maximum applied-step count because alpha.11 `StateMachine::run` has no application-specific step-limit parameter. That wrapper is evidence, not the replacement lesson design: the README requires the GDK run loop and an instructor-agreed safeguard.
7. Streamed raw messages must be reconstructed with GDK merge semantics. An initial implementation that retained deltas independently read only the final `.`; accumulating through `Conversation::push`, as `InferenceRunner` does, fixed it.
8. Decimal strings parsed into `rust_decimal` are the accepted Lesson 5 representation.
9. Historical recommendation, superseded by the README replacement plan: split the transition rather than introduce all infrastructure at once. The enduring finding is cognitive load; the new plan uses one no-code lesson followed by two complete-code lessons, with run/read/tweak rather than scaffold-writing activities.
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

## Replacement lesson implementation handoff

Lessons 1–6 are complete; **do not begin by implementing Lesson 5 again**. Start with the [README replacement plan and handoff](README.md#implementation-handoff-and-review-gates), then read `AGENTS.md` and the mental-model reference. Confirm instructor authorization and the target lesson before writing lesson material.

Read Git status/diffs, manifests/lock/toolchain/provider JSON, root source, and complete Lessons 3–6 plus instructor notes. Preserve unrelated changes. Archived Lessons 7–8 and `spike/state-machine/` are research/history only. Keep numeric prefixes and do not restore drafts wholesale.

Review exact pinned `goose-agent` sources (`README.md`, `src/machine.rs`, `src/operation.rs`, `src/inference.rs`, `src/events.rs`, `src/tool.rs`), the provider/conversation/message types, RMCP typed tool interfaces, and official documentation before asserting behavior. Upstream source in the mental-model document is not necessarily identical to the pinned release. Do not upgrade dependencies as part of this planning change or without a separate decision.

Implement one authorized lesson at a time. Lesson 7 is prose/diagrams/instructor notes only and leaves root Rust unchanged. For Lesson 8, confirm the root update and settle minimal runtime/effect compatibility, concurrent event consumption/shutdown, and bounds while keeping `StateMachine::run`. Lesson 9 reuses that foundation, adds successive user inputs and the existing planned-loss capability, and preserves argument validation/correlation/request bounds. Script two user turns; do not add an interactive CLI prematurely.

Complete coding-lesson source must be available from the outset. Document the shared-manifest/root run workflow, structural expected output, and small predict/run/tweak activity. After code changes run formatting, checking, linting, tests, and the lesson's live-provider command. Specifically validate live streaming in Lesson 8 and both selective tool-use outcomes in Lesson 9. Report unexpected or missing tool calls honestly. Unit tests and builds do not establish live-provider behavior, and successful validation does not replace instructor release approval.
