# Lesson 5: Define a Deterministic Trading Tool and Inspect Its Raw Request

## Goal

Define and advertise **one** deterministic tool, `maximum_planned_loss`, to raw provider inference. Reconstruct the streamed assistant-role message, inspect every content block, and identify a pending structured tool request without executing it.

This lesson ends at the request boundary. Advertising a tool does not authorize it, receiving a request does not execute it, and no result is returned to the model yet.

## Concepts introduced

- Deterministic computation versus probabilistic model reasoning.
- A tool definition: name, description, and advertised JSON input schema.
- **Structured model output:** typed fields the application can inspect without guessing from prose.
- A **tool request:** structured model output that asks the application to use an advertised tool.
- A **correlation ID:** the request ID that links a later tool response to one specific tool request.
- Passing a tool definition to provider inference through the `tools` slice.
- A **finish reason**: provider telemetry describing why the model's turn ended — `tool_calls` means the model ended its turn expecting a tool result. This program displays it but never acts on it.
- Reconstructing streamed deltas into complete messages with `Conversation::push`.
- Inspecting every reconstructed message and content block.
- Reading a request's parsed call through its `Result`, without blindly unwrapping it.
- Retaining the complete assistant-role message, including its request ID, **in memory during this run**.
- Advertising a tool versus authorizing and executing it.

## Where this fits in the agentic application

Lessons 3 and 4 established the first half of the application flow: the application sends inference requests and retains ordinary user/assistant-role discussion history. This lesson adds an advertised capability, but the application remains in control at every boundary:

```text
Application advertises one allowed tool
    |
    v
Model may produce text or a structured tool request
    |
    v
Application reconstructs and inspects the complete response
    |
    v
Lesson 5 stops: it retains the request but does not authorize or execute it
```

A tool request is not a command the model can carry out. It is a proposal for the application to consider. Lesson 6 completes the next steps: validate the proposal, execute only an allowed calculation, and return the result with the matching correlation ID.

## Prerequisites

- Lessons 1–4 are complete.
- This lesson adds one requirement to the root `Cargo.toml` that Lessons 1–4 do not use yet: `rmcp = "=3.4.1"`, which supplies the MCP `Tool` type and `JsonObject` schema used to advertise the deterministic tool.
- The selected provider supports the GDK streaming call used here, including its `tools` argument.
- The provider JSON declares the first model to select.
- Before class, the instructor has verified that the classroom provider/model can emit a native structured request, but a request is not guaranteed on every provider, model, or run.

## Step 5.1: Why give the application a deterministic tool?

A model predicts useful text. It should not be the authority for money-related arithmetic. A **deterministic tool** is a capability the application implements in Rust: after the application validates its inputs, the same inputs always produce the same result.

For this first example, the user describes a hypothetical long position:

| Input | Tool field | Value |
| --- | --- | --- |
| Entry price | `entry_price` | `"51.20"` |
| Stop price | `stop_price` | `"50.70"` |
| Shares | `share_count` | `200` |

For a long position, the calculation that **Lesson 6** will validate and execute is:

```text
maximum planned loss = (entry_price - stop_price) × share_count
```

This scenario will produce `$100.00` before fees, slippage, or a gap through the stop. In this lesson, however, the application does **not** calculate or return `$100.00`. It only gives the model a way to request that later calculation.

Maximum planned loss is the amount a trader intends to risk if a long position exits at its planned stop price. It is a planning figure rather than a guarantee; for further background on stop-loss orders, see [Investopedia's Stop-Loss Order primer](https://www.investopedia.com/terms/s/stop-lossorder.asp).

> **Domain note.** A planned loss uses the intended stop price as a planning reference, not as a guarantee of an actual fill. Fees, slippage, and a price gap through a stop can make an actual loss higher. Lesson 6 returns those limitations with the calculated result.

## Raw request sequence: the boundary for this lesson

Before writing code, keep this entire protocol in view:

```text
User message (entry 51.20, stop 50.70, 200 shares)
    |
    v
Provider inference + advertised tool definition (maximum_planned_loss)
    |
    v
Model returns an assistant-role ToolRequest (id, name, arguments)  <-- LESSON 5 STOPS HERE
    |
    v
[Lesson 6] Application validates and allowlists the request
    |
    v
[Lesson 6] Deterministic Rust calculation
    |
    v
[Lesson 6] User ToolResponse with the matching request ID
    |
    v
[Lesson 6] Follow-up provider inference -> educational response
```

The model may request a capability, but the application authorizes and executes it. The vertical boundary above is why a request is useful even though this lesson does not yet calculate anything.

### Keep these five things separate

| Concept | Meaning in this lesson |
| --- | --- |
| `Tool` definition | The application advertises a capability contract to the provider and model. |
| JSON input schema | Describes the expected argument shape; it does **not** validate received model output. |
| Tool-request block | Structured model output: a named tool, arguments, and an ID that ask the application to consider a capability. |
| Request ID | A future correlation ID: Lesson 6 must attach it to the corresponding tool response so the provider can match result to request. |
| `Conversation` | The GDK container that reconstructs and retains this call's received assistant-role message(s) for the next raw-protocol step. It is not persistent storage or the complete input history. |

## Step 5.2: Name the prompt for a request, not a result

```rust
const SYSTEM_INSTRUCTION: &str =
    "You are a day-trading teaching assistant. For the supplied hypothetical trade, use the \
     maximum_planned_loss tool. Never claim to place, modify, or cancel a trade.";
const USER_PROMPT: &str =
    "A hypothetical long trade enters at $51.20, uses a stop at $50.70, and has 200 shares. \
     What is the maximum planned loss?";
```

The system instruction asks the model to request the deterministic capability. It remains the separate `system` argument from Lesson 4, not an entry in `messages`.

Do not ask this first call to explain a tool result. The model has not received one. Lesson 6 will send a correlated response and then ask for the educational explanation, including the excluded fees, slippage, and gap risk.

## Step 5.3: Define one narrowly advertised schema

A tool is an MCP `Tool`: a name, a human description, and a JSON input schema. The schema below advertises the expected request shape; it does **not** validate provider output or authorize execution. Lesson 6 performs that enforcement.

```rust
fn tool_definition() -> Tool {
    let schema: JsonObject = serde_json::from_value(json!({
        "type": "object",
        "properties": {
            "entry_price": {"type": "string", "pattern": "^[0-9]+(\\.[0-9]{1,4})?$"},
            "stop_price": {"type": "string", "pattern": "^[0-9]+(\\.[0-9]{1,4})?$"},
            "share_count": {"type": "integer", "minimum": 1}
        },
        "required": ["entry_price", "stop_price", "share_count"],
        "additionalProperties": false
    }))
    .expect("static input schema must be a JSON object");

    Tool::new(
        "maximum_planned_loss",
        "Calculate the maximum planned loss for a hypothetical long stock position from \
         decimal-dollar entry and stop prices and a whole-number share count. This excludes \
         fees, slippage, and a gap through the stop.",
        Arc::new(schema),
    )
}
```

- **`entry_price` and `stop_price`** are decimal strings. At the model/tool boundary, text such as `"51.20"` is natural, and Lesson 6 will parse it with exact decimal arithmetic rather than binary floating point.
- The pattern allows zero to four decimal places. It is advertised guidance, not runtime validation.
- **`share_count`** is advertised as a whole number of at least one.
- **`additionalProperties: false`** declares extra fields invalid. It does not prevent a model from sending them; Lesson 6 must reject them.

Tool names and arguments are untrusted model output even when they appear to match this advertised schema.

## Step 5.4: Advertise the tool in the existing stream call

A **payload delimiter** separates explanatory text from **protocol payload** — the actual values that traveled to or from the provider. Every block of payload is printed between two identical `++++++++` lines, and everything outside them is this application's own commentary. The line immediately before each block also names the **sender and the receiver** of that message — `application → provider` on the way out, `provider → application` on the way back — because every message in the round trip must show both of its ends. The delimiter matters because model output is untrusted data: the terminal must make visible exactly which lines are the provider's, so a learner can point at the boundary between the application's words and the model's. Lesson 6 reuses the same delimiter for the tool responses sent back.

At the provider call site, the new input from Lesson 4 is a non-empty `tools` slice:

```rust
let tools = [tool_definition()];
let messages = vec![Message::user().with_text(USER_PROMPT)];

println!("── Sending ─────────────────────────────────────────────");
println!("1 user message; advertising 1 tool: {}", TOOL_NAME);
println!("  (the model may now request it)");

// The full outbound payload is these prompts plus the advertised tool
// definition: the entire context the model sees in the first call.
// Printing all of it makes it visible — the model answers only what this
// call contains, and nothing here executes anything.
println!("\nOutbound payload sent with this call (application → provider), quoted in full:");
print_delimiter();
println!("  system instruction:");
print_indented(SYSTEM_INSTRUCTION, "    > ");
println!("  user message:");
print_indented(USER_PROMPT, "    > ");
print_tool_definition(&tools[0])?;
print_delimiter();

println!("\n── Streaming response deltas (provider → application) ──");
let (conversation, usage) =
    stream_and_collect(provider.as_ref(), &model, &messages, &tools).await?;
//  └── the four inputs are unchanged: provider, model, system+user history, tools
print_usage_summary(&usage);

println!("\n── All the response messages (provider → application, after Conversation::push) ──");
let saw_tool_request = inspect_reconstructed(&conversation)?;
```

Right after the announcement, the program prints the entire outbound payload it sends: the `system` instruction from Step 5.2, the single user message, and the advertised tool definition from Step 5.3 — its name, description, and input schema. All three travel in the same inference call, so this is the complete context the model sees before any model output arrives. Note where the request to use the tool lives: in the system instruction text, not in the tool definition itself.

Advertising the definition does not force the model to call it and does not run it. A provider/model may return plain text, a structured request, a mixture of blocks, or no usable request. Observe and report that structural outcome in the live run.

The next two steps also change what the program retains from the stream: instead of collecting only text, it reconstructs complete messages so it can inspect structured blocks.

### Why this lesson needs a `Conversation`, not just concatenated text

Lessons 3 and 4 received ordinary text answers.

- In **Lesson 3**, the program only needed to display the model's streamed assistant-role text.
- In **Lesson 4**, it needed the first answer's text so it could create a new `Message::assistant()` entry before sending a follow-up user question.

For those purposes, collecting text fragments into a `String` was enough.

This lesson begins an **agentic discussion**, where a model response can ask the application to do work rather than—or in addition to—writing text. That means the application must preserve more than the visible words.

### The pieces of the discussion

The application owns the discussion history. Its main pieces are:

| Term | Meaning |
| --- | --- |
| **`Conversation`** | The GDK container this lesson uses to reconstruct and retain complete assistant-role message(s) received in this streamed call. It does not contain the original user input here, and it exists only in memory. |
| **Message** | One participant's contribution to the discussion, such as a user question or an assistant-role response. A message has a role and can contain one or more content blocks. |
| **Content block** | One typed piece of a message, such as text or a structured tool request. One assistant-role message may contain multiple blocks. |
| **Stream delta** | A partial update received while the provider is streaming a message. A delta is not necessarily a complete message or complete content block. |
| **Tool-request block** | Structured model output that names a requested tool, supplies arguments, and carries a request ID. The request ID must later be matched with the tool response. |

**Structured model output** is data with a known shape that the program can examine by fields, rather than prose the program would have to guess how to interpret. Here, a tool-request block separately carries a tool name, its arguments, and an ID. The model produces this request, but the application must still treat every field as untrusted input; the shape does not authorize execution or prove the arguments are valid.

The flow in this lesson is:

```text
User message
    |
    v
Application prints the outbound payload (system instruction + user message + tool definition)
    |
    v
Provider streams partial assistant-role message deltas
    |
    v
Application prints any text immediately and records usage telemetry
    |
    v
Conversation::push merges the deltas into a complete assistant-role message
    |
    v
Application inspects every completed content block
    |
    v
If present: retain the structured tool request and its ID for Lesson 6
```

A text-only reconstruction would lose the information needed for the next step. For example, `as_concat_text()` can display text, but it does not retain a tool request's structured name, arguments, or correlation ID.

A **correlation ID** lets the application link one specific tool response to the request that caused it. It matters when an assistant-role message contains multiple requests, when requests have the same tool name, or when later protocol steps need to identify the exact request being answered. In Lesson 6, the application will include the retained request ID in the corresponding tool response so the provider can associate the calculation result with the correct request.

`Conversation::push(message)` is therefore not just string concatenation. It merges the assistant-role message deltas received during this stream, preserving their structured content and relevant metadata until the program inspects the reconstructed message after streaming ends. This lesson's `Conversation` contains received assistant-role message(s), not the original user input. Lesson 6 will carry the retained assistant-role request message forward with the original user message(s) when it continues the raw protocol. This lesson still does **not** execute the tool. It only preserves and inspects the model's request so that the application can validate and handle it safely in the next lesson.

## Step 5.5: Reconstruct complete messages from stream deltas

A streamed response is **not** one stream item, and one response is **not** necessarily one content block. The helper prints text blocks as they arrive, then places every streamed `Message` delta into a `Conversation`. `Conversation::push` reconstructs complete messages while preserving structured blocks and metadata, including a future tool request's ID and arguments:

```rust
async fn stream_and_collect(
    provider: &dyn Provider,
    model: &ModelConfig,
    messages: &[Message],
    tools: &[Tool],
) -> Result<(Conversation, Option<ProviderUsage>), Box<dyn Error>> {
    let mut stream = provider
        .stream(model, SYSTEM_INSTRUCTION, messages, tools)
        .await?;

    // A streamed response is not one stream item and one response. Conversation::push
    // coalesces the partial deltas that share a message back into complete messages.
    let mut conversation = Conversation::empty();
    let mut streamed_text = false;
    let mut usage = None;
    while let Some((message, item_usage)) = stream.next().await.transpose()? {
        if let Some(message) = message {
            let text = message.as_concat_text();
            if !text.is_empty() {
                // Streamed text is the model's output, not commentary. Open the
                // delimiter at the first delta and close it after the stream ends;
                // individual deltas cannot each carry a fence of their own.
                if !streamed_text {
                    print_delimiter();
                }
                print!("{}", text);
                streamed_text = true;
            }
            // Text alone is enough for Lessons 3 and 4, but a tool-capable model
            // response can also contain structured requests, arguments, and correlation
            // IDs. Preserve the complete assistant-role message for the next agentic step.
            conversation.push(message);
        }
        if let Some(item_usage) = item_usage {
            usage = Some(item_usage);
        }
    }

    if streamed_text {
        // Streamed deltas do not guarantee a trailing newline, so start the
        // closing delimiter on a fresh line.
        println!();
        print_delimiter();
    } else {
        println!(
            "(no text blocks streamed; the response arrived as non-text blocks — see the reconstructed view below)"
        );
    }

    Ok((conversation, usage))
}
```

This retains the `Some`/`None` handling from Lesson 4. The two changes are `conversation.push(message)` and collecting the call's final `ProviderUsage` for a compact summary. `as_concat_text()` prints text blocks only. When no text block arrives at all, the helper says so under the streaming header and defers to the reconstructed view instead of leaving an unexplained gap in the output.

A **finish reason** is provider telemetry that describes why the model's turn ended. A finish reason of `tool_calls` means the model stopped its turn expecting the application to run a tool and return a result — it is evidence of the request boundary, not permission to act. This program only displays it:

```rust
/// Print one compact usage line. A finish reason of `tool_calls` means the
/// model ended its turn expecting a tool result: display it, never act on it.
fn print_usage_summary(usage: &Option<ProviderUsage>) {
    println!("\n── Usage ───────────────────────────────────────────────");
    let Some(usage) = usage else {
        println!("(no usage telemetry was provided by the provider)");
        return;
    };
    let input_tokens = match usage.usage.input_tokens {
        Some(count) => count.to_string(),
        None => "-".to_string(),
    };
    let output_tokens = match usage.usage.output_tokens {
        Some(count) => count.to_string(),
        None => "-".to_string(),
    };
    println!(
        "model={}  tokens in={} out={}",
        usage.model, input_tokens, output_tokens
    );
    if let Some(reasons) = &usage.finish_reasons {
        println!("finish_reasons={:?}", reasons);
        if reasons.iter().any(|reason| reason == "tool_calls") {
            println!("  ← the model ended its turn expecting a tool result.");
            println!("    The application, not the finish reason, decides what happens next.");
        }
    }
}
```

The compact line replaces a full struct dump: `model`, token counts, and the finish reason carry the teaching, while the remaining `ProviderUsage` fields (`stats`, `cost`, `response_id`, `additional_data`) are provider internals this lesson does not use.

## Step 5.6: Inspect every reconstructed content block safely

Label this as a second, reconstructed view of the response. Each phase prints under a labeled header, so streamed text and the reconstructed text block may both appear; that repetition is intentional. It demonstrates two concerns: responsive display while streaming and structured protocol inspection after reconstruction.

```rust
fn inspect_reconstructed(conversation: &Conversation) -> Result<bool, Box<dyn Error>> {
    let mut saw_tool_request = false;
    for (message_index, message) in conversation.messages().iter().enumerate() {
        println!(
            "message {}: role={:?}, {} content block(s)",
            message_index + 1,
            message.role,
            message.content.len()
        );

        for (block_index, block) in message.content.iter().enumerate() {
            let label = format!("block {}/{}:", block_index + 1, message.content.len());

            if let Some(request) = block.as_tool_request() {
                saw_tool_request = true;
                println!(
                    "  {} ToolRequest — the model asks the application to run a tool",
                    label
                );
                print_delimiter();
                // The request ID is the correlation ID: Lesson 6 must attach it to
                // the tool response so the provider can match result to request.
                println!("    request id (correlation ID): {}", request.id);
                // `tool_call` is a Result: tool-call parsing can fail inside a
                // request, so never blindly unwrap it.
                match &request.tool_call {
                    Ok(call) => {
                        println!("    tool name: {}", call.name);
                        println!(
                            "    arguments (untrusted model output; JSON object, order not guaranteed):"
                        );
                        match &call.arguments {
                            Some(arguments) => {
                                print_indented(&serde_json::to_string_pretty(arguments)?, "      ")
                            }
                            None => println!("      (none)"),
                        }
                    }
                    Err(error) => {
                        println!(
                            "    unparseable call (reported without panicking): {}",
                            error
                        );
                    }
                }
                print_delimiter();
            } else if let Some(thinking) = block.as_thinking() {
                println!("  {} Thinking block — Not ground truth", label);
                print_delimiter();
                print_indented(&thinking.thinking, "             ");
                print_delimiter();
            } else if let Some(text) = block.as_text() {
                if !text.trim().is_empty() {
                    println!("  {} Text", label);
                    print_delimiter();
                    print_indented(text, "    ");
                    print_delimiter();
                }
            } else {
                println!("  {} other non-text block: {}", label, block);
            }
        }
    }

    Ok(saw_tool_request)
}

/// Print multi-line model text with a fixed indent so streamed prose, private
/// reasoning, and structured arguments stay visually inside their labeled block.
fn print_indented(text: &str, indent: &str) {
    for line in text.lines() {
        println!("{}{}", indent, line);
    }
}
```

The double loop is deliberate: a response can contain multiple messages, each message can contain several blocks, and one message can contain more than one tool request. The output makes that structure visible instead of flattening it: every message prints its role and block count, and every block prints its index and kind.

A tool-request block has an ID even when its enclosed call cannot be parsed. For a **parseable** request, print its ID labeled as the correlation ID, its tool name, and its arguments as pretty JSON — the same shape the advertised schema describes, in whatever order the model sent them. For an **unparseable** one, print its ID and parse error without panicking. A `Thinking` block is framed as the model's private reasoning: it may narrate tool calls and results that never happened, so it is never ground truth. Do not deserialize, validate, allowlist, or execute the arguments yet.

## Step 5.7: Stop at the request boundary

After inspection, show where the run stops inside the cycle:

```rust
println!("\n── We stopped here ────────────────────────────────────");
println!("The full request/response cycle for a tool-using turn:");
println!("  1. send system instruction + user message (+ the advertised tool)");
println!("  2. model responds with thinking + a tool request");
println!("  3. application validates the request and executes the tool");
println!("  4. send the full history + tool response; model writes the final answer");
println!("  5. application returns the final answer to the user");

if saw_tool_request {
    println!("\nThis run stopped between steps 2 and 3 — an artificial stop for teaching.");
    println!("The tool request above was received, but nothing executed: the application");
    println!("alone authorizes, validates, and runs the calculation. Lesson 6 resumes");
    println!("the cycle at step 3.");
} else {
    println!("\nThis run stopped after step 2: no tool request arrived, so there was");
    println!("nothing to execute. Advertising a tool permits a request; it does not");
    println!("cause one. Lesson 6 validates and dispatches when one does arrive.");
}
```

The section shows the learner **where** the run stopped inside the cycle instead of summarizing leftover state. The diagram lists the full request/response flow for a tool-using turn, and the branch statement then names the exact stopping point. With a request received, the run stops between steps 2 and 3 — an artificial stop for teaching: the request arrived but nothing executed, and Lesson 6 resumes the cycle at step 3. With no request, the run stops after step 2: advertising a tool permits a request but does not cause one. Step 4 is the resubmission point worth naming aloud: the provider is stateless, so the follow-up call resends the entire history — system instruction, user message, assistant tool request, and the tool response. One state fact stays here in prose rather than in the terminal: the `Conversation` is in-process state only, so this program does **not** save conversation history across program runs. Persistence comes later.

## Expected structural behavior

- The run prints five labeled phases: sending (what is advertised), streaming, usage, reconstructed messages, and the closing stop summary.
- Under the sending header, the program prints the entire outbound payload before any model output: both prompts quoted in full (labeled as the system instruction and the user message) and the advertised tool definition with its name, description, and pretty-printed input schema.
- Text blocks stream to stdout as they arrive; if no text block streams, the run says so under the streaming header and points to the reconstructed view.
- Usage prints as one compact line: model, token counts, and the finish reason when the provider supplies one. A `tool_calls` finish reason is displayed as evidence of the request boundary and never used to change behavior.
- Every content block in every reconstructed message is inspected, not only the first block; every message prints its role and block count, and every block prints its index and kind.
- A `Thinking` block is printed in full under a framing label: the model's private reasoning that may narrate tool calls and results that never happened.
- For a parseable `maximum_planned_loss` request, the program prints its ID labeled as the correlation ID, its tool name, and its arguments as pretty JSON in the order the model sent them.
- For an unparseable request, the program prints its ID and parse error without panicking.
- The application performs no calculation, creates no tool response, and makes no follow-up inference; the cycle-position statement runs on every path.
- The closing section prints the full request/response cycle for a tool-using turn and names the exact step where the run stopped — between steps 2 and 3 with a request received, after step 2 without one.
- The complete assistant-role message is retained in memory only for the duration of this run; the terminal prints the cycle position rather than a state summary, and the lesson text carries the in-process-state note.
- A text-only response or no structured request is a provider/model outcome to record, not automatically a code defect.

## Success criteria

- One narrowly advertised `maximum_planned_loss` schema declares three required fields, decimal-string price patterns, positive whole-number shares, and no additional properties.
- One inference call advertises that definition through the `tools` slice.
- The program reconstructs stream deltas with `Conversation::push` before inspecting them.
- Every content block across every reconstructed message is inspected.
- A parseable request is read without blindly unwrapping its tool call; an unparseable request is reported safely.
- Nothing is executed: no decimal calculation, `CallToolResult`, tool response, or follow-up inference occurs.
- The unit test checks the advertised tool name, closed-object declaration, and required-field list without a live provider.

## Live validation

Run from the repository root:

```bash
cargo run
```

Or choose another existing provider configuration:

```bash
cargo run -- path/to/provider.json
```

Record whether the run produced a native structured request, plain text only, an unparseable request, or another block shape. The bundled endpoint has emitted native `maximum_planned_loss` requests, but the provider JSON does not guarantee tool-call behavior. The program displays the finish reason when the provider supplies one, but it never changes protocol behavior because of it. Do not treat a text-only run as proof that the program executed a tool or necessarily as a code defect.

## Next

Lesson 6 takes a parseable pending request and treats the model-provided name and arguments as untrusted input. It validates and allowlists the request, parses decimal strings exactly, executes the deterministic calculation, returns a user-role tool response with the retained request ID, and invokes the provider again for an educational explanation of the result and its exclusions.