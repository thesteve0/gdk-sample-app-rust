# Lesson 5: Define a Deterministic Trading Tool and Inspect Its Raw Request

## Goal

Define and advertise **one** deterministic tool, `maximum_planned_loss`, to raw provider inference. Reconstruct the streamed assistant message, inspect every content block, and identify a pending structured tool request without executing it.

This lesson ends at the request boundary. Advertising a tool does not authorize it, receiving a request does not execute it, and no result is returned to the model yet.

## Concepts introduced

- Deterministic computation versus probabilistic model reasoning.
- A tool definition: name, description, and advertised JSON input schema.
- **Structured assistant output:** typed fields the application can inspect without guessing from prose.
- A **tool request:** structured assistant output that asks the application to use an advertised tool.
- A **correlation ID:** the request ID that links a later tool response to one specific tool request.
- Passing a tool definition to provider inference through the `tools` slice.
- Reconstructing streamed deltas into complete messages with `Conversation::push`.
- Inspecting every reconstructed message and content block.
- Reading a request's parsed call through its `Result`, without blindly unwrapping it.
- Retaining the complete assistant message, including its request ID, **in memory during this run**.
- Advertising a tool versus authorizing and executing it.

## Where this fits in the agentic application

Lessons 3 and 4 established the first half of the application flow: the application sends inference requests and retains ordinary user/assistant discussion history. This lesson adds an advertised capability, but the application remains in control at every boundary:

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
- The selected provider supports the GDK streaming call used here, including its `tools` argument.
- The provider JSON declares the first model to select.
- Before class, the instructor has verified that the classroom provider/model can emit a native structured request. The bundled local endpoint did so in the recorded post-Lesson-4 spike, but a request is not guaranteed on every provider, model, or run.

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
Assistant ToolRequest (id, name, arguments)  <-- LESSON 5 STOPS HERE
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
| Tool-request block | Structured assistant output: a named tool, arguments, and an ID that ask the application to consider a capability. |
| Request ID | A future correlation ID: Lesson 6 must attach it to the corresponding tool response so the provider can match result to request. |
| `Conversation` | The GDK container that reconstructs and retains this call's received assistant message(s) for the next raw-protocol step. It is not persistent storage or the complete input history. |

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

At the provider call site, the new input from Lesson 4 is a non-empty `tools` slice:

```rust
let tools = [tool_definition()];
let messages = vec![Message::user().with_text(USER_PROMPT)];

let conversation = stream_and_collect(provider.as_ref(), &model, &messages, &tools).await?;
//               │          │             │          │          └── one advertised tool
//               │          │             │          └── ordered user/assistant history
//               │          │             └── separate system instruction
//               │          └── configured model
//               └── provider
```

Advertising the definition does not force the model to call it and does not run it. A provider/model may return plain text, a structured request, a mixture of blocks, or no usable request. Observe and report that structural outcome in the live run.

The next two steps also change what the program retains from the stream: instead of collecting only text, it reconstructs complete messages so it can inspect structured blocks.

### Why this lesson needs a `Conversation`, not just concatenated text

Lessons 3 and 4 received ordinary text answers.

- In **Lesson 3**, the program only needed to display the assistant's streamed text.
- In **Lesson 4**, it needed the first answer's text so it could create a new `Message::assistant()` entry before sending a follow-up user question.

For those purposes, collecting text fragments into a `String` was enough.

This lesson begins an **agentic discussion**, where an assistant response can ask the application to do work rather than—or in addition to—writing text. That means the application must preserve more than the visible words.

### The pieces of the discussion

The application owns the discussion history. Its main pieces are:

| Term | Meaning |
| --- | --- |
| **`Conversation`** | The GDK container this lesson uses to reconstruct and retain complete assistant message(s) received in this streamed call. It does not contain the original user input here, and it exists only in memory. |
| **Message** | One participant's contribution to the discussion, such as a user question or an assistant response. A message has a role and can contain one or more content blocks. |
| **Content block** | One typed piece of a message, such as text or a structured tool request. One assistant message may contain multiple blocks. |
| **Stream delta** | A partial update received while the provider is streaming a message. A delta is not necessarily a complete message or complete content block. |
| **Tool-request block** | Structured assistant output that names a requested tool, supplies arguments, and carries a request ID. The request ID must later be matched with the tool response. |

**Structured assistant output** is data with a known shape that the program can examine by fields, rather than prose the program would have to guess how to interpret. Here, a tool-request block separately carries a tool name, its arguments, and an ID. The model produces this request, but the application must still treat every field as untrusted input; the shape does not authorize execution or prove the arguments are valid.

The flow in this lesson is:

```text
User message
    |
    v
Provider streams partial assistant-message deltas
    |
    v
Application prints any text immediately
    |
    v
Conversation::push merges the deltas into a complete assistant message
    |
    v
Application inspects every completed content block
    |
    v
If present: retain the structured tool request and its ID for Lesson 6
```

A text-only reconstruction would lose the information needed for the next step. For example, `as_concat_text()` can display text, but it does not retain a tool request's structured name, arguments, or correlation ID.

A **correlation ID** lets the application link one specific tool response to the request that caused it. It matters when an assistant message contains multiple requests, when requests have the same tool name, or when later protocol steps need to identify the exact request being answered. In Lesson 6, the application will include the retained request ID in the corresponding tool response so the provider can associate the calculation result with the correct request.

`Conversation::push(message)` is therefore not just string concatenation. It merges the assistant-message deltas received during this stream, preserving their structured content and relevant metadata until the program inspects the reconstructed message after streaming ends. This lesson's `Conversation` contains received assistant message(s), not the original user input. Lesson 6 will carry the retained assistant request message forward with the original user message(s) when it continues the raw protocol. This lesson still does **not** execute the tool. It only preserves and inspects the assistant's request so that the application can validate and handle it safely in the next lesson.

## Step 5.5: Reconstruct complete messages from stream deltas

A streamed response is **not** one stream item, and one response is **not** necessarily one content block. The helper prints text blocks as they arrive, then places every streamed `Message` delta into a `Conversation`. `Conversation::push` reconstructs complete messages while preserving structured blocks and metadata, including a future tool request's ID and arguments:

```rust
async fn stream_and_collect(
    provider: &dyn Provider,
    model: &ModelConfig,
    messages: &[Message],
    tools: &[Tool],
) -> Result<Conversation, Box<dyn Error>> {
    let mut stream = provider
        .stream(model, SYSTEM_INSTRUCTION, messages, tools)
        .await?;

    let mut conversation = Conversation::empty();
    while let Some((message, usage)) = stream.next().await.transpose()? {
        if let Some(message) = message {
            let text = message.as_concat_text();
            if !text.is_empty() {
                print!("{text}");
            }
            conversation.push(message);
        }
        if let Some(usage) = usage {
            eprintln!("\nusage: {usage:#?}");
        }
    }

    Ok(conversation)
}
```

This retains the `Some`/`None` handling from Lesson 4. The important change is `conversation.push(message)`: inspect reconstructed complete messages later, rather than flattening a partial stream delta into text.

`as_concat_text()` prints text blocks only. A provider may also return a `Thinking`, image, error, tool-request, or other block; the next step inspects those blocks after reconstruction rather than promising that they appear in streamed text output.

## Step 5.6: Inspect every reconstructed content block safely

Label this as a second, reconstructed view of the response. It may repeat text printed while streaming; that repetition demonstrates that the program is now examining the complete reconstructed message rather than an individual delta.

```rust
println!("\nReconstructed content blocks:");
let mut saw_tool_request = false;
for message in conversation.messages() {
    for block in &message.content {
        if let Some(request) = block.as_tool_request() {
            match &request.tool_call {
                Ok(call) => {
                    saw_tool_request = true;
                    println!("  tool request id={}", request.id);
                    println!("  tool name={}", call.name);
                    println!("  tool arguments={:?}", call.arguments);
                }
                Err(error) => {
                    saw_tool_request = true;
                    println!(
                        "  tool request id={} has an unparseable call: {}",
                        request.id, error
                    );
                }
            }
        } else if let Some(text) = block.as_text() {
            if !text.trim().is_empty() {
                println!("  text: {text}");
            }
        } else {
            println!("  {block}");
        }
    }
}
```

The double loop is deliberate: a response can contain multiple messages, each message can contain several blocks, and one message can contain more than one tool request. A `Thinking` block is one possible non-text block that the final `else` renders when the provider supplies it.

A tool-request block has an ID even when its enclosed call cannot be parsed. For a **parseable** request, print its ID, name, and arguments. For an **unparseable** one, print its ID and parse error without panicking. Do not deserialize, validate, allowlist, or execute the arguments yet.

## Step 5.7: Stop at the request boundary

After inspection, make the boundary visible:

```rust
println!();
if !saw_tool_request {
    println!("No structured maximum_planned_loss request was received this run.");
    println!(
        "Advertising or receiving a request does not execute the tool. Lesson 6 \
         validates the untrusted arguments and dispatches the calculation."
    );
}

println!(
    "\nRetained {} reconstructed assistant message(s) in memory for the next raw-protocol step.",
    conversation.messages().len()
);
```

The `Conversation` value contains the complete assistant message, including any request ID, while this program is running. This reference then exits; it does **not** save conversation history across program runs. Lesson 6 will extend the same raw-protocol flow by carrying the assistant request message forward to its correlated response.

## Expected structural behavior

- Text blocks stream to stdout as they arrive; non-text blocks are inspected later from reconstructed messages.
- Every content block in every reconstructed message is inspected, not only the first block.
- For a parseable `maximum_planned_loss` request, the program prints its ID, name, and arguments. The example arguments are normally shaped like `{"entry_price": "51.20", "stop_price": "50.70", "share_count": 200}`.
- For an unparseable request, the program prints its ID and parse error without panicking.
- The application performs no calculation, creates no tool response, and makes no follow-up inference.
- The complete assistant message is retained in memory only for the duration of this run.
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

Record whether the run produced a native structured request, plain text only, an unparseable request, or another block shape. The recorded post-Lesson-4 spike observed native `maximum_planned_loss` requests with the bundled endpoint, but the provider JSON does not guarantee tool-call behavior and this program does not inspect `finish_reason`. Do not treat a text-only run as proof that the program executed a tool or necessarily as a code defect.

## Next

Lesson 6 takes a parseable pending request and treats the model-provided name and arguments as untrusted input. It validates and allowlists the request, parses decimal strings exactly, executes the deterministic calculation, returns a user-role tool response with the retained request ID, and invokes the provider again for an educational explanation of the result and its exclusions.
