# Post-Lesson 4 Technical Spike

This directory is exploratory engineering evidence, not a released lesson or the root learner exercise. It tests the exact pinned `goose-providers` and `goose-agent` alpha.11 APIs without changing Lesson 4 materials.

## Run

```bash
cargo run --example post-lesson4-spike -- --raw
cargo run --example post-lesson4-spike -- --state
cargo run --example post-lesson4-spike -- --all path/to/provider.json
```

The local provider must be running. Both paths ask the configured model to calculate a hypothetical long position's maximum planned loss for entry `51.20`, stop `50.70`, and 200 shares. The deterministic answer is `100.00` before fees, slippage, or a gap through the stop.

## What the spike exercises

### Raw protocol

- Advertise one allowlisted RMCP tool to provider inference.
- Inspect every streamed message and every tool-request content block.
- Reconstruct complete assistant messages from stream deltas and preserve them, rather than saving only the first tool call.
- Validate the tool name, deserialize with unknown-field rejection, and perform domain validation.
- Execute deterministic decimal arithmetic.
- append a user-role tool response with the matching request ID and provider metadata;
- Repeat inference until final text or a fixed round/request limit is reached.

### State machine

- Register the same calculator as a typed `SyncTool` in `ToolOperation`.
- Use provider-backed `InferenceRunner` and a prompt-contributing operation.
- Persist `ConversationEffect`s in a minimal in-memory `SessionLoader`/`EffectHandler`.
- Reload the session between steps, matching `StateMachine::run` semantics.
- Apply an explicit six-step bound around `step`/`apply` for the spike. The alpha.11 `StateMachine::run` method itself has no application-specific maximum-step argument.

## Explicit failure and termination policy

| Case | Spike behavior |
|---|---|
| Multiple requests in one response | Execute each recognized request, preserving each ID, up to the aggregate limit. |
| Malformed request representation | Return its structured protocol error as that request's tool response. |
| Unknown/disallowed tool name | Raw dispatch returns `invalid_params`; `ToolOperation` leaves unknown requests unhandled because only registered tools are executable. |
| Invalid/missing/extra arguments | Return `invalid_params`; `RiskInput` rejects unknown fields and the function validates values. |
| Deterministic tool error | Return a caller-visible tool error, retaining the request ID. |
| Repeated calls | Raw mode allows at most 3 inference rounds and 4 total requests. State mode allows at most 6 applied steps. |
| No request and no final text | Fail explicitly. |

The raw loop intentionally returns validation/tool failures to the model so it may correct a call within the bound. Provider transport/protocol failures abort instead. Production policy may choose to stop immediately after particular failures.

## Currency representation comparison

Both considered approaches are exact for the example:

- **Integer cents:** mechanically simple (`5120 - 5070`, then multiply by 200) and needs no decimal dependency, but exposes an unnatural model-facing unit. It cannot naturally represent sub-cent prices without choosing a different fixed scale.
- **Decimal strings plus `rust_decimal`:** natural tool arguments (`"51.20"`) and explicit exact decimal parsing, but adds a dependency and validation concepts. The spike caps prices at four decimal places and formats dollar outputs to two.

**Provisional recommendation:** decimal strings are clearer and safer at the model/tool boundary for a trading lesson. Parse and validate them immediately into a decimal type. Integer minor units remain attractive for an internal API whose fixed scale is already established, but naming fields `entry_cents` makes the first tool less natural and silently choosing cents excludes valid sub-dollar tick sizes.

## Source findings and curriculum implications

The implementation was checked against the exact published sources for:

- `goose-agent 0.1.0-alpha.11`: `machine.rs`, `operation.rs`, `inference.rs`, `tool.rs`, and `tests/tool_operation.rs`;
- `goose-provider-types 0.1.0-alpha.11`: conversation/message builders and content types;
- `goose-providers 0.1.0-alpha.11`: the provider surface re-exported by the project.

Notable findings:

1. `ToolOperation` already batches pending known requests and emits one user-role response message with correlated IDs.
2. Typed `SyncTool` registration requires RMCP `ToolBase`, Serde, and JSON Schema types. It executes through `spawn_blocking`.
3. `InferenceRunner` requires a custom effect type to retain `ProviderUsage`; `ConversationEffect` alone does not implement `InferenceEffect` in alpha.11.
4. A working runtime must understand conversation replacement and append effects. The minimal spike deliberately rejects metadata-patch effects it does not use.
5. State-machine setup introduces enough concepts—typed RMCP tools, effects, usage effects, sessions, loader/handler traits, events, cancellation, and bounds—that the transition should remain **two lessons**, as proposed in the planning document. Keep most runtime plumbing instructor-provided in the first state-machine lesson.
6. The raw implementation remains substantially easier to narrate as a protocol. It should precede this machinery.

## Live evidence record

On 2026-09-29, the configured local endpoint successfully completed:

- **Raw protocol:** 4/4 runs emitted a native structured request with the expected name and arguments, preserved a request ID, received the correlated deterministic result, and returned final prose containing `100.00` and the required caveats. One initial run exposed a spike bug: raw stream deltas were stored without GDK message merging and appeared as only `.` when reading the last delta. Reconstructing through `Conversation::push`, as `InferenceRunner` does, fixed it; the subsequent run plus three-run sample all passed.
- **State machine:** 2/2 runs applied `llm -> tools -> llm`, then stopped because no operation applied. Its final answer used `100.00`, explained the arithmetic and exclusions, and denied placing a trade.
- **Native tool calling:** no tool shim, prompt change, or model change was required for this sample. The configured alias was `qwen3.6-35b-a3b`; provider usage identified the serving model as `peculiar-ragdoll/Unsloth-Ornith-1.5-35B-A3B:UD-Q8_K_XL`.

This small sample establishes compatibility and encouraging repeatability, not broad reliability. Before lesson release, test malformed/unknown calls through structural fixtures and repeat live scenarios across the intended classroom model/server configuration. Do not silently substitute mocks when that provider is unavailable.
