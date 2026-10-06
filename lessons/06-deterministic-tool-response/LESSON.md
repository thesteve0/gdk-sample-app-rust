# Lesson 6: Execute the Tool and Return Its Result Through the Raw Provider Protocol

## Goal

Take the pending Lesson 5 request through its complete raw round trip. Because the user gave us unstructured text for the input 
parameters to the tool call, we need the model to parse out the arguments to send to the tool. This gives us the following steps
to complete:
1. Validate the model's untrusted arguments
2. Dispatch only an allowlisted tool 
3. Execute the deterministic calculation with exact decimal arithmetic, 
4. Return one correlated user-role tool response per request
5. Ask the provider for a final educational explanation of the result.

The lesson still contains no agent loop. The program performs **exactly two inference rounds by construction**: one that receives the request, and one follow-up call that returns the explanation.

## Concepts introduced

- **Untrusted input**, deepened: every field of a tool request — name, arguments, parse status — is model-produced data the application must validate before use. Lesson 5 defined this term; this lesson enforces it.
- **Deserialization** (shape validation): converting untrusted JSON arguments into a typed Rust value, which fails on wrong types, missing required fields, and unknown fields.
- **Domain validation** (value validation): application rules the advertised schema cannot enforce, such as at most four decimal places, a positive price, and an entry above the stop.
- **Exact decimal arithmetic**: computing money amounts with a decimal type, never binary floating point.
- **Allowlisted dispatch**: executing only a tool whose name is on an application-owned list. The application decides what is executable; the model only proposes.
- **Tool response**: a message carrying a tool's result back to the provider, correlated to one request by the request ID.
- **Effective role**: the role a message plays on the wire. A user-role message carrying a tool response has effective role `tool`.
- **Tool-level failure versus routing failure**: which boundary rejected a request — a request the application could not route (unparseable call, unknown tool name, missing arguments) versus a request that reached the tool but failed domain validation.
- **Round bound**: an explicit limit on how many inference rounds and requests one run processes. This program performs two rounds and dispatches only the requests received in round 1.
- **Payload delimiter**: a printed `++++++++` fence around every block of protocol payload — the outbound prompts and tool definition, the model's output, and the tool responses sent back. Everything between two delimiter lines is a value that traveled to or from the provider; everything outside them is this application's explanatory text.
- Reusing Lesson 5's terms: structured model output, tool request, correlation ID, deterministic tool, finish reason, content block, and `Conversation`.

## Where this fits in the agentic application

Lesson 5 stopped between steps 2 and 3 of the cycle, with the assistant tool-request message retained in memory. This lesson implements steps 3, 4, and 5:

```text
User message (entry 51.20, stop 50.70, 200 shares)
    |
    v
Provider inference + advertised tool definition (maximum_planned_loss)
    |
    v
Model returns an assistant-role ToolRequest (id, name, arguments)      <-- Lesson 5 stopped here
    |
    v
[Lesson 6] Application validates and allowlists the request            <-- implemented now
    |
    v
[Lesson 6] Deterministic Rust calculation with exact decimal arithmetic <-- implemented now
    |
    v
[Lesson 6] User ToolResponse with the matching request ID              <-- implemented now
    |
    v
[Lesson 6] Follow-up provider inference -> educational response        <-- implemented now
    |
    v
Application returns the final answer to the user
```

The model may request a capability, but the application authorizes and executes it. Nothing changed about that boundary in this lesson; what changed is that the application now completes the cycle instead of stopping at the request.

### Keep these six things separate

| Concept | Meaning in this lesson |
| --- | --- |
| Pending tool request | The Lesson 5 structured request, retained in the in-memory `Conversation` with its ID. |
| Deserialization | The shape boundary: untrusted JSON arguments become a typed Rust value or fail. |
| Domain validation | The value boundary: exact decimal parsing, decimal-place limits, entry above stop, positive shares. |
| Allowlisted dispatch | The authorization boundary: only a known tool name executes. |
| Tool response | One user-role message per request, carrying the result and the matching request ID. |
| Round bound | Two inference rounds and round-1 requests only. It is not an open agent loop. |

## Prerequisites

- Lessons 1–5 are complete, including a live provider that emitted a structured `maximum_planned_loss` request in Lesson 5.
- This lesson adds two requirements to the root `Cargo.toml` that Lessons 1–5 do not use yet:
  - `serde = { version = "1", features = ["derive"] }` — the derive feature deserializes untrusted arguments into a domain struct;
  - `rust_decimal = "1.43.0"` — exact decimal arithmetic for money.
- The provider JSON declares the first model to select, as in Lessons 3–5.
- The system instruction, user prompt, and advertised tool definition are unchanged from Lesson 5.

## Step 6.1: From a pending request to a result

A **tool response** is the application's answer to one tool request. It carries the result the model needs and the request ID that links it to the request that caused it. Until this lesson, the application could only inspect a request; now it answers one.

The round trip has a state fact the application must handle: the provider is stateless. It remembers nothing between calls, so the follow-up call that asks for the final explanation must resend **the entire history** — the original user message, the assistant tool-request message, and the tool response.

That is why this lesson's history is built in three stages:

```text
1. the original user message                        (already in hand)
2. the assistant tool-request message(s)            (Lesson 5's reconstructed Conversation)
3. one user-role tool-response message per request  (created in this lesson)
```

The program performs this cycle **twice as two inference rounds** and then stops. There is no loop that keeps dispatching new requests; if the model asks again in round 2, the run reports the bound and stops. Lesson 7 introduces the state-machine mental model through one simple exchange; Lesson 9 will show how it coordinates this raw tool work.

## Step 6.2: Model arguments are untrusted input, twice over

Lesson 5 established that tool names and arguments are untrusted model output: they arrive as text, their order is not guaranteed, and matching an advertised schema does not authorize anything. This lesson enforces that with **two separate checks**, in order:

1. **Deserialization** checks the argument *shape*: does the JSON have exactly the fields the application expects, with the expected types?
2. **Domain validation** checks the *values*: are the prices parseable exact decimals with at most four decimal places? Is the entry above the stop? Is the share count positive?

Keeping the two checks separate matters because they fail for different reasons. Deserialization fails on structure — a missing field, a string where a number was expected, an extra field the application never defined. Domain validation fails on business rules — a negative price, an inverted stop. Knowing which check rejected a request tells you whether the model misunderstood the schema or the application's domain rules are firing.

Deserialization is expressed with a domain struct. A **domain struct** is a Rust type whose fields describe one application concept — here, the arguments of one trading calculation:

```rust
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LossArguments {
    entry_price: String,
    stop_price: String,
    share_count: u32,
}
```

- `deny_unknown_fields` rejects extra fields. Lesson 5 noted that `additionalProperties: false` in the advertised schema does not prevent a model from sending extras; this attribute is the enforcement.
- Wrong types fail: a JSON number where `entry_price` declares a string is a deserialization error.
- **`entry_price` and `stop_price` stay strings on purpose.** serde_json would otherwise deserialize a bare JSON number into a binary floating-point type, which Step 6.3 forbids for money. Keeping the wire representation a string forces parsing to be an explicit, exact domain decision.

The call site treats missing arguments as a shape failure too — a request with no arguments is not dispatched.

## Step 6.3: Exact decimal arithmetic, never binary floating point

**Exact decimal arithmetic** computes money amounts with a decimal type whose base-10 representation matches the way prices are written. Binary floating point (`f64`) cannot represent values such as `0.1` exactly, so sums and differences of dollar amounts can drift. For a teaching application about money, that drift is unacceptable even when it is small.

The application therefore parses each price string immediately into `rust_decimal::Decimal`:

```rust
fn parse_price(field: &str, raw: &str) -> Result<Decimal, String> {
    let parse_result = Decimal::from_str_exact(raw);
    let value = match parse_result {
        Ok(parsed_price) => parsed_price,
        Err(_parse_error) => {
            let failure = format!("{} '{}' is not a plain decimal number", field, raw);
            return Err(failure);
        }
    };
    if value.scale() > 4 {
        return Err(format!("{} '{}' has more than four decimal places", field, raw));
    }
    if value <= Decimal::ZERO {
        return Err(format!("{} must be a positive dollar amount", field));
    }
    Ok(value)
}
```

`Result<T, E>` is the standard library's enum for "one of two outcomes": `Ok(T)` carries the success value, and `Err(E)` carries the failure reason. This lesson uses it with `T = Decimal` (the parsed price) and `E = String` (the reason validation failed). The function never returns a bare `Decimal` — `Ok(value)` wraps the parsed price and every failure path wraps a readable reason, so a caller cannot reach the price without handling the failure case first.

- The `match` on `parse_result` is the extract-or-bail shape: bind the success value, or return early with the readable failure. This lesson unwraps every `Result` this way, so the type-system work is visible in the code.
- `_parse_error` begins with an underscore to mark the parse library's own error type as deliberately unused — the `String` reason is what the caller and the model see.
- `Decimal::from_str_exact` parses a plain decimal string exactly. `"51.20"` keeps its two decimal places.
- `scale()` is the number of decimal places. The domain rule is at most four — the same limit the advertised pattern describes, now enforced in code.
- Positive values are a domain rule. The advertised schema cannot express "greater than zero" in the pattern; the application enforces it here.

The calculation itself is ordinary decimal arithmetic with the Lesson 5 scenario:

```rust
fn execute_maximum_planned_loss(arguments: &JsonObject) -> Result<String, String> {
    let deserialized = serde_json::from_value(Value::Object(arguments.clone()));
    let args: LossArguments = match deserialized {
        Ok(parsed) => parsed,
        Err(error) => {
            let failure = format!("arguments do not match the advertised schema: {}", error);
            return Err(failure);
        }
    };

    let entry = parse_price("entry_price", &args.entry_price)?;
    let stop = parse_price("stop_price", &args.stop_price)?;
    if args.share_count == 0 {
        return Err("share_count must be a positive whole number".to_string());
    }
    if entry <= stop {
        return Err(format!(
            "entry_price {} must be above stop_price {} for a long position",
            entry, stop
        ));
    }

    let shares = Decimal::from(args.share_count);
    let loss = (entry - stop) * shares;
    Ok(format!("${:.2}", loss))
}
```

For entry `51.20`, stop `50.70`, and 200 shares, the result is `"$100.00"` — the exact decimal answer, formatted to two places. The entry-above-stop rule rejects an inverted trade instead of returning a meaningless negative loss. As in Lesson 5, the tool's description states what the calculation excludes: fees, slippage, and a gap through the planned stop. The final educational answer is expected to repeat those exclusions.

## Step 6.4: Allowlisted dispatch through three boundaries

**Allowlisted dispatch** means the application executes only requests whose tool name is on an application-owned list. An **allowlist** is that list: everything not on it is rejected by default. The model proposes; the application disposes.

One function puts the three boundaries in order:

```rust
fn execute_allowed_tool(
    tool_call: &ToolResult<CallToolRequestParams>,
) -> ToolResult<CallToolResult> {
    let parsed_call = match tool_call {
        Ok(call) => call,
        Err(error) => {
            let failure = format!("the tool call could not be parsed: {}", error.message);
            return Err(ErrorData::invalid_params(failure, None));
        }
    };

    let requested_name = parsed_call.name.as_ref();
    if requested_name != TOOL_NAME {
        let failure = format!(
            "tool '{}' is not on this application's allowlist; '{}' is the only executable tool",
            requested_name, TOOL_NAME
        );
        return Err(ErrorData::invalid_params(failure, None));
    }

    let arguments = match &parsed_call.arguments {
        Some(arguments) => arguments,
        None => {
            return Err(ErrorData::invalid_params(
                "the tool call carried no arguments".to_string(),
                None,
            ));
        }
    };

    match execute_maximum_planned_loss(arguments) {
        Ok(loss) => Ok(CallToolResult::success(vec![ContentBlock::text(loss)])),
        Err(failure) => Err(ErrorData::invalid_params(failure, None)),
    }
}
```

Read the boundaries top to bottom:

1. **Parse boundary.** `tool_call` is a `Result`, as Lesson 5 established: tool-call parsing can fail inside a request. An unparseable call is never unwrapped; it becomes an error response.
2. **Allowlist boundary.** Only `maximum_planned_loss` dispatches. A model that names any other tool — even a plausible one — is refused without executing anything.
3. **Domain boundary.** Deserialization, value validation, and the deterministic calculation run only after the first two boundaries pass.

Each boundary first extracts what it needs into a named value — `parsed_call`, `requested_name`, `arguments` — and returns an error response when its check fails. The allowlist check is a plain `if`, not a match guard, so all three boundaries read the same way.

This distinction is worth naming because it is how you debug a misbehaving turn:

- A **routing failure** is a request the application could not route at all: unparseable call, unknown tool name, or missing arguments. Nothing about the tool's domain ran.
- A **tool-level failure** is a request that reached the tool but failed domain validation: unparseable price, too many decimal places, entry at or below the stop, zero shares.

Both become the same kind of protocol-valid answer — an error result with a reason — but the reason tells you which boundary rejected the request.

Every failure becomes `Err(...)` rather than a panic or a dropped request. The model receives a readable explanation and can respond; the program continues.

## Step 6.5: One correlated user-role response per request

The response is built with the GDK message API. A tool response rides on a **user-role** message:

```rust
history.push(Message::user().with_tool_response(request.id.clone(), result));
```

Two facts matter here:

1. **The role is user, but the effective role is tool.** **Effective role** is the role a message plays on the wire, as distinct from the role field stored on the message. The GDK convention is that tool results ride in user-role messages, and the provider serializes them as `role=tool` messages with a `tool_call_id`. The lesson's outbound view prints both roles so the convention is visible.
2. **The request ID is the correlation key.** Lesson 5 defined the correlation ID as the request ID that links a later response to one specific request. The response attaches `request.id` unchanged. The provider matches results to requests through IDs — never by order, and never by tool name. That is why the program answers every request, in order, each with its own correlated response, and why a response must never be built from anything but the request's own ID.

A successful response carries the deterministic text — `"$100.00"` — as its content. A failed request carries an error result with a reason. The main loop prints which happened for each request ID — labeled `application, as the tool → provider` — and prints the value itself between payload delimiters, so the dispatch and its direction are observable in the terminal.

## Step 6.6: The stateless follow-up call and the round bound

The follow-up call is the same streaming helper from Lesson 5, invoked a second time with the full history:

```rust
let mut history = vec![Message::user().with_text(USER_PROMPT)];
history.extend(conversation.messages().iter().cloned()); // assistant tool-request message(s)
// ... one tool-response message per pending request is pushed in the dispatch loop ...
```

Before the call, the program prints the outbound history it is about to send — each message's role, effective role, and blocks — because the stateless provider sees exactly that list and nothing else:

```text
outbound (application → provider): 1 advertised tool(s) and 3 message(s):
  1. role=User, effective role=user, block(s): text
  2. role=Assistant, effective role=assistant, block(s): thinking, tool request (id call_...)
  3. role=User, effective role=tool, block(s): tool response to id call_...
```

(One request plus its response produces three messages; the tool definition is still advertised but does not add a message. The message count grows by one per additional request.)

A **round bound** is an explicit limit on how many inference rounds and requests one run processes. The principle from the raw-protocol exploration carries into every tool lesson: the application, not the model, decides when the cycle ends. This program:

- performs **exactly two inference rounds** — round 1 receives requests, round 2 produces the final answer;
- dispatches **only the requests received in round 1**;
- never loops, even if round 2's response contains another tool request. That case is detected and reported as the bound doing its job.

Lesson 9 will coordinate these same responsibilities through the GDK state machine; Lesson 7 first introduces its mental model without tools.

## Step 6.7: The final educational answer

Round 2 streams the model's educational explanation, built on the tool result the history now contains. The program prints streamed text exactly as in Lessons 3–5, between payload delimiters: the model's answer is payload, not commentary. The expected answer uses the deterministic `$100.00` from the tool response — not a model-computed number — and identifies what the calculation excludes: fees, slippage, and a gap through the planned stop.

If no tool response were in the history, the model would have nothing grounded to explain. The correlation in Step 6.5 is what makes the explanation trustworthy: the model reasons over an application-computed result, not its own arithmetic.

## Step 6.8: Where the run ends

The closing section prints the same five-step cycle Lesson 5 listed, and names the step this run reached:

- With a streamed final answer and no re-request, the run completed steps 3–5: the streamed answer above is step 5, the application returning the final answer to the user.
- If round 2's response contains another tool request, the run stops at the two-round bound and says so — an explicit application limit, not a crash and not an agent loop.
- With no tool request in round 1 at all, the program reports that the round trip could not begin, executes nothing, and makes no follow-up call — the same outcome Lesson 5 reported, for the same reason.

## Reading the terminal: payload versus commentary

The terminal output of this lesson mixes two kinds of lines: explanatory text this application prints, and **protocol payload** — the actual values that traveled to or from the provider. A **payload delimiter** separates them. Every block of payload is printed between two identical `++++++++` lines, and the line immediately before each block names the **sender and the receiver** of that message:

- the outbound payload this application sends, labeled `application → provider`: system instruction, user message, and the advertised tool definition;
- the model's streamed output, labeled `provider → application`, fenced from its first text delta to the end of the stream;
- each reconstructed round-1 content block, labeled `provider → application`: thinking, tool request, and text;
- the tool response values this application sends back, labeled `application, as the tool → provider` — in this lesson the tool is this application — one fence per request.

Everything outside those lines is the application's own commentary. The delimiter matters because model output is untrusted data: the terminal must make visible exactly which lines are the provider's, so a learner can point at the boundary between the application's words and the model's. The actor labels exist for the same reason: the round trip has multiple actors — this application, the provider, and this application acting as the tool — and every message must show who sent it and who receives it.

In `main.rs` the convention is one constant and one helper: `PAYLOAD_DELIMITER` holds the fence, and `print_delimiter()` prints one line of it. The streaming helper opens the fence at the first text delta and closes it after the stream ends, because individual deltas cannot each carry a fence of their own.

## Expected structural behavior

- The run prints seven labeled phases: sending, round 1 streaming, round 1 metadata, round 1 reconstructed messages, validate/dispatch/respond, round 2, and the closing where-the-run-ends summary.
- The sending phase prints the same outbound payload view as Lesson 5 — system instruction, user message, and the advertised tool definition — between payload delimiters, labeled `application → provider`.
- Round 1 streams and reconstructs exactly as in Lesson 5, including the compact usage line and full inspection of every content block, with each content block between payload delimiters, labeled `provider → application`.
- For each pending request, the dispatch phase prints one line per request ID naming the sender and receiver (`application, as the tool → provider`), then the value itself between payload delimiters: a deterministic result on success, or the error reason returned to the model.
- One user-role tool-response message is created per request, each carrying that request's own ID.
- The round 2 header prints the full outbound history with roles, effective roles, and blocks — including at least one message whose effective role is `tool`.
- Round 2 streams the final educational answer between payload delimiters; the compact usage line prints for the second call.
- Everything between two `++++++++` delimiter lines is protocol payload; everything outside them is the application's commentary, and the line before each fence names who sent that message and who receives it.
- The closing summary names the reached step: cycle complete, stopped at the two-round bound on a re-request, or no round trip possible without a request.
- Nothing is ever executed without passing the parse, allowlist, and domain boundaries; no request is dropped; no panic occurs on model output.

## Success criteria

- Untrusted arguments are deserialized into a domain struct with `deny_unknown_fields`, and unknown fields, wrong types, and missing arguments are rejected before any calculation.
- Prices are parsed with exact decimal arithmetic (`rust_decimal`), never binary floating point; at most four decimal places, positive values, entry strictly above stop, and a positive share count are enforced as domain validation.
- Dispatch is allowlisted: a request naming any other tool is refused without executing.
- Every failure — parse, allowlist, or domain — becomes a protocol-valid error result with a reason; no panic, no dropped request.
- Each request receives exactly one user-role tool response carrying that request's ID; the provider can correlate result to request by ID alone.
- The follow-up call resends the entire history — user message, assistant tool-request message(s), tool response(s) — to the stateless provider.
- The program performs exactly two inference rounds and reports the bound when the model re-requests in round 2.
- Unit tests cover the exact scenario result (`$100.00`), single-decimal inputs, unknown fields, non-string prices, more than four decimal places, entry at or below stop, non-positive prices and shares, non-allowlisted names, and calls with no arguments — all without a live provider.

## Live validation

Run from the repository root:

```bash
cargo run
```

Or choose another existing provider configuration:

```bash
cargo run -- path/to/provider.json
```

Record the structural outcome:

- Did round 1 produce a parseable `maximum_planned_loss` request, and what did the dispatch line print?
- Did the round 2 outbound view show a message with effective role `tool`?
- Did the final educational answer use the deterministic `$100.00` and mention fees, slippage, or gap risk?
- Did the run end with the cycle complete, or at the two-round bound?

A text-only round 1 remains a provider/model outcome to record, not automatically a code defect — the program reports it and stops, exactly as Lesson 5 did.

## Next

[Lesson 7](../07-state-machine-mental-model/LESSON.md) uses this manual implementation as motivation for the GDK state machine, with no code: Session, Operation, StateMachine, Effect, and re-evaluation, narrated through one simple question/answer exchange. The planned-loss state-machine trace is deferred to Lesson 9. Lesson 8 supplies the smallest inference-only in-memory runtime; Lesson 9 combines continuing conversation and tools.