# Lesson 7: The GDK State Machine over the Lesson 6 Conversation

## Goal

Hand the coordination of the Lesson 6 conversation to the GDK state machine. Lessons 3–6 coordinated the agentic cycle with a hand-written loop: Lesson 6 performed **exactly two inference rounds by construction** and stopped. That works for teaching the raw protocol, but it is not how a real agent decides what to do next. This lesson introduces the GDK's state machine — an ordered list of operations the crate executes for you, each one re-deriving its decision from the persisted conversation on every pass.

The lesson contains **no provider call**. The session is seeded in code with the same conversation Lesson 6 reconstructed — the user message and the assistant's structured `maximum_planned_loss` request — so the machine's trace is deterministic and reproducible in class. The provider returns in Lesson 8, as an inference step in the machine.

## State and rounds of message passing

Before the state machine itself, it is worth naming what has been true all along. In Lessons 3–6 the application held exactly one piece of state: the in-memory `Conversation`. The provider held none — it is stateless, so every inference round resent the entire accumulated history, and "exactly two rounds" was control flow the application wrote by hand. When the program ended, the conversation vanished with the process.

The state machine keeps the same conversation, the same stateless provider, and the same resend-everything rounds, but changes who owns the sequencing. The conversation becomes **persisted conversation state** stored under a session id and reloaded from a store on every pass; a pass reloads the state, lets an ordered list of operations re-derive their decisions from it, applies the first applicable operation's effects, persists them, and repeats from the reloaded state. Nothing is decided from a loop counter or an in-memory flag — every decision is a function of what is in the persisted history — and Lesson 6's fixed two-round bound becomes an explicit limit on applied steps. The provider is still stateless and the wire traffic is unchanged; what changed is that the application's own state now lives somewhere a reload can find it, and sequencing is data (the ordered operations) instead of code.

## Concepts introduced

- **Persisted conversation state**: the conversation lives in a store keyed by a session id, and every pass begins by reloading it. This is the state holder Lessons 3–6 kept in a local variable.
- **Session**: one id plus the conversation it names. The `MachineSession` trait is what the machine reads: `id()` is the key the store reloads by, and `conversation()` is the state an operation reasons over.
- **Operation**: one step in the ordered list the machine executes. An operation re-derives its decision from the conversation it is handed and either applies or declines.
- **Declining (not applicable)**: an operation that has nothing to do in the current state returns "not applicable" and the machine moves to the next operation in the list.
- **Effect**: the only way an operation changes the world — a request to append a message (or otherwise modify the conversation) that the runtime persists. Operations never touch the conversation directly.
- **Session loading**: the store's `SessionLoader` implementation, which produces a fresh session view for every pass. The machine never caches the session.
- **Effect application**: the store's `EffectHandler` implementation, which writes effects back. Loading and applying are the machine's entire interface to persistence.
- **Re-derivation from persisted state**: deciding from what the history contains, not from loop counters or in-memory flags. This is the pedagogical heart of the lesson: the same operation is asked again on pass 2 and declines, because the answer it produced on pass 1 is now in the history.
- **Yield**: an operation can hand control back to the client — the thing driving the agent, which in this lesson is this program. The machine stops and the client decides what happens next.
- **State-step bound**: an explicit limit on how many steps one run applies. It is the counterpart of Lesson 6's round bound: the application, not the model and not an unbounded loop, decides when the machine stops.
- Reusing Lesson 5–6 terms: tool request, tool response, correlation ID, effective role, allowlisted dispatch, domain validation, exact decimal arithmetic, and the payload delimiter.

## Where this fits in the agentic application

Lesson 6 completed one raw round trip by hand: receive the request, validate and execute it, return the correlated tool response, ask for the final explanation. Lesson 7 takes the first half of that cycle — the tool response — and lets the GDK state machine coordinate it:

```text
User message (entry 51.20, stop 50.70, 200 shares)
    |
    v
Assistant structured ToolRequest (call_seed_001)                 <- Lesson 6 work, replayed in the seed
    |
    v
[Lesson 7] Pass 1: reload session -> answer_pending_tools applies
           -> executes the deterministic calculation
           -> emits AppendMessage effect (user tool response)
    |
    v
[Lesson 7] Runtime persists the effect into the store
    |
    v
[Lesson 7] Pass 2: reload session -> answer_pending_tools declines
           -> yield_to_client yields -> the machine stops
    |
    v
[Lesson 8] A new kind of step: provider-backed inference reads the
           conversation (including the tool response) and produces the
           next assistant message                                <- future
```

The boundaries from Lessons 5–6 are unchanged. Tool arguments are still untrusted input, dispatch is still allowlisted, prices are still exact decimals. The state machine coordinates the conversation; it does not relax a single guard.

### Diagram 1: who owns the sequencing

```text
Lesson 6 (hand-written control flow)        Lesson 7 (the GDK state machine)
─────────────────────────────────────       ─────────────────────────────────────
main() writes:                              The machine owns the loop:
                                            ┌──────────────────────────────────┐
  round 1: receive request                  │ every pass:                      │
    -> validate + allowlist                 │   1. reload the persisted session │
    -> execute deterministic tool           │   2. ask each operation in order  │
    -> push tool response                   │   3. first applicable one emits   │
  round 2: resend history, ask for answer   │      effects -> persist them      │
  stop (two rounds by construction)         │   4. repeat from the reload       │
                                            │ stop when: no operation applies,  │
                                            │   a step yields, or the bound     │
                                            │   on applied steps is reached     │
                                            └──────────────────────────────────┘
```

In Lesson 6 the sequencing lived in `main`'s code and ran exactly twice. In Lesson 7 the sequencing lives in the ordered list of operations, and the same operation is consulted on every pass — so what it decides must depend on what the history contains, not on how many times it has run.

### Diagram 2: the GDK state machine

This is the machine the GDK assembles, with the two halves of the loop the crate provides (`step` decides, `apply` persists). The second operation kind — **inference** — is how the provider re-enters in Lesson 8; it is labeled as future behavior so it is not confused with what this lesson implements.

```text
                    ┌────────────────────────────────────────────┐
                    │  reload the persisted session by id        │
                    │  (SessionLoader -> MachineSession)         │
                    └───────────────────────┬────────────────────┘
                                            v
                    ┌────────────────────────────────────────────┐
                    │  StateMachine::step: ask operations,       │
                    │  in order, for the current conversation    │
                    └───────────────────────┬────────────────────┘
                                            v
              applies? ── no ──> try the next operation ── none applies ──> STOP
                 │
                 v (effects)
                    ┌────────────────────────────────────────────┐
                    │  StateMachine::apply: hand the effects     │
                    │  to the runtime (EffectHandler persists)   │
                    └───────────────────────┬────────────────────┘
                                            v
                              yield_to_client? ── yes ──> STOP (client decides next)
                                            │ no
                                            v
                              step bound reached? ── yes ──> STOP
                                            │ no
                                            └────> reload again (next pass)

  Operation kinds the machine runs:
    - Step::Operation  (this lesson):  answer_pending_tools, yield_to_client
    - Step::Inference  (Lesson 8):     provider-backed model response        <- future
```

Two halves, two traits. `step` is the decision half: the machine hands the reloaded conversation to each operation in order. `apply` is the persistence half: the first applicable operation's effects go to the runtime, and nothing a step decided survives unless the runtime persisted it.

### Diagram 3: the deterministic two-pass trace

```text
SEED  (built in code, no provider)          session 'lesson-07'
  1. user text:  "A hypothetical long trade enters at $51.20, ... $100.00?"
  2. assistant tool request: id=call_seed_001, tool=maximum_planned_loss

PASS 1
  reload 'lesson-07'                    -> 2 messages, request unanswered
  answer_pending_tools:
    re-derive unanswered from history   -> call_seed_001 has no response
    execute_allowed_tool (L6 boundaries)-> deterministic "$100.00"
    emit AppendMessage effect           -> user tool response, id=call_seed_001
  apply: persist effect                 -> store now holds 3 messages

PASS 2
  reload 'lesson-07'                    -> 3 messages, request answered
  answer_pending_tools:
    re-derive unanswered from history   -> empty -> declines (not applicable)
  yield_to_client:
    last effective role is Tool         -> yields -> the machine stops

FINAL PERSISTED CONVERSATION (what the next reload sees)
  1. user text  2. assistant tool request  3. user tool response ($100.00)
```

## Prerequisites

- Lessons 1–6 are complete, including the Lesson 6 round trip that produced the `$100.00` tool response.
- This lesson adds the GDK agent crate and its support dependencies to the root `Cargo.toml`:
  - `goose-agent = "=0.1.0-alpha.11"` — the pinned GDK agent crate that provides the state machine, operations, effects, and emitter. An exact pin, per the course convention: no moving branches.
  - `anyhow` — the error type the machine's traits use;
  - `async-trait` — the machine's traits are async;
  - `tokio-util` — the machine uses a `CancellationToken` for cooperative cancellation;
  - `tokio` gains the `sync` feature — the emitter is an `mpsc` channel.
- The tool code from Lesson 6 — `LossArguments`, `parse_price`, `execute_maximum_planned_loss`, `execute_allowed_tool` — carries over unchanged.
- There is no provider in this lesson, so no provider JSON is read and no `.env` is loaded. Lesson 8 restores both.

## Step 7.1: The session and the store

A **session** is one conversation plus the id it is stored under. Until now the conversation lived in a local variable; here it lives in a store keyed by id, which is what makes it *persisted conversation state* — state a reload can find, not state that dies with a loop iteration.

```rust
struct TradingSession {
    id: String,
    conversation: Conversation,
}

impl MachineSession for TradingSession {
    fn id(&self) -> &str {
        &self.id
    }

    fn conversation(&self) -> Option<&Conversation> {
        Some(&self.conversation)
    }
}
```

`MachineSession` is the trait the machine reads. `id()` is the key the store reloads by; `conversation()` is the state an operation reasons over. This lesson's session type is hand-written so you can see the trait is just "an id and a conversation" — Lesson 8 builds on the same shape.

The store is an in-memory map from session id to conversation. It is seeded with the conversation Lesson 6 reconstructed:

```rust
fn seed_conversation() -> Conversation {
    let mut conversation = Conversation::empty();
    conversation.push(Message::user().with_text(USER_PROMPT));

    // The request the provider actually sent in Lesson 6, replayed in code.
    // The id is fixed so the trace is reproducible across runs.
    let raw_arguments = json!({
        "entry_price": "51.20",
        "stop_price": "50.70",
        "share_count": 200
    });
    let arguments = match raw_arguments {
        Value::Object(object) => object,
        _ => JsonObject::new(),
    };
    let params = CallToolRequestParams::new(TOOL_NAME).with_arguments(arguments);
    conversation.push(Message::assistant().with_tool_request("call_seed_001", Ok(params)));
    conversation
}
```

The seed is protocol payload — it is exactly what the provider sent in Lesson 6 — so it is built in code with a fixed correlation ID (`call_seed_001`) to keep the machine's trace deterministic. Lesson 8 replaces this seeding with a live provider call.

## Step 7.2: Loading and applying are the machine's whole persistence interface

**Session loading** is how the machine gets a fresh view of the state for every pass; **effect application** is how a step's decisions become persisted state. The machine defines both as traits, and the runtime implements them.

```rust
#[async_trait]
impl SessionLoader<TradingSession> for InMemoryStore {
    async fn load(&self, session_id: &str) -> anyhow::Result<TradingSession> {
        let loaded = self.conversations.lock().unwrap().get(session_id).cloned();
        let conversation = match loaded {
            Some(conversation) => conversation,
            None => {
                let failure = format!("session '{}' is not in the store", session_id);
                return Err(anyhow::anyhow!(failure));
            }
        };
        Ok(TradingSession {
            id: session_id.to_string(),
            conversation,
        })
    }
}
```

The loader clones the conversation out of the map. That clone is the point: each pass gets its own view of the state, and nothing the machine did on the previous pass is visible unless the effect handler wrote it back.

```rust
#[async_trait]
impl EffectHandler<TradingSession, ConversationEffect> for InMemoryStore {
    async fn apply_effects(
        &self,
        session: &TradingSession,
        effects: &mut [ConversationEffect],
        _emit: &Emitter,
    ) -> anyhow::Result<()> {
        let mut conversation = session.conversation().cloned().ok_or_else(|| {
            anyhow::anyhow!("session '{}' loaded without conversation", session.id())
        })?;
        for effect in effects {
            match effect {
                ConversationEffect::AppendMessage(message) => {
                    let role = effective_role(message);
                    println!(
                        "  persisting effect: append message (effective role {:?})",
                        role
                    );
                    conversation.push(message.clone());
                }
                // This minimal runtime applies only AppendMessage; the other
                // effect kinds are rejected so the lesson's surface stays small.
                ConversationEffect::ReplaceConversation(_) => {
                    return Err(anyhow::anyhow!(
                        "this runtime only applies AppendMessage effects"
                    ));
                }
                ConversationEffect::PatchToolRequestMeta { .. } => {
                    return Err(anyhow::anyhow!(
                        "this runtime only applies AppendMessage effects"
                    ));
                }
                ConversationEffect::SetMessageVisibility { .. } => {
                    return Err(anyhow::anyhow!(
                        "this runtime only applies AppendMessage effects"
                    ));
                }
            }
        }
        self.conversations
            .lock()
            .unwrap()
            .insert(session.id().to_string(), conversation);
        Ok(())
    }
}
```

An **effect** is the only way an operation changes the world. The operation asks — "append this message" — and the runtime decides whether and how to persist it. The GDK defines several effect kinds; this minimal runtime applies only `AppendMessage` and rejects the rest, so the lesson's surface stays small. Rejecting the others in code is also how you see that the effect enum is a real boundary, not a convenience.

## Step 7.3: An operation that re-derives from the persisted history

An **operation** is one step in the machine's ordered list. It receives the conversation as it exists right now and either applies (returns effects) or declines (returns "not applicable"). The critical discipline: an operation decides from the conversation it was handed, never from "have I run before".

```rust
struct AnswerPendingTools;

#[async_trait]
impl Operation<TradingSession> for AnswerPendingTools {
    fn name(&self) -> &'static str {
        "answer_pending_tools"
    }

    async fn run(
        &self,
        _session: &TradingSession,
        conversation: &Conversation,
        _emit: &Emitter,
    ) -> anyhow::Result<OperationResult<ConversationEffect>> {
        let unanswered = unanswered_tool_requests(conversation);
        if unanswered.is_empty() {
            return not_applicable();
        }
        let mut effects = Vec::new();
        for request in unanswered {
            let result = execute_allowed_tool(&request.tool_call);
            let id = request.id.clone();
            print_tool_result(&id, &result);
            effects.push(ConversationEffect::AppendMessage(
                Message::user().with_tool_response(id, result),
            ));
        }
        applied(effects)
    }
}
```

The decision lives in one helper, and it is **re-derived from the persisted state** on every call:

```rust
fn unanswered_tool_requests(conversation: &Conversation) -> Vec<&ToolRequest> {
    let mut answered = Vec::new();
    for message in conversation.messages() {
        for block in &message.content {
            if let Some(response) = block.as_tool_response() {
                answered.push(response.id.clone());
            }
        }
    }
    let mut requests = Vec::new();
    for message in conversation.messages() {
        for block in &message.content {
            if let Some(request) = block.as_tool_request() {
                if !answered.contains(&request.id) {
                    requests.push(request);
                }
            }
        }
    }
    requests
}
```

A request counts as unanswered only when no response in the history carries its ID. Lesson 6 collected every pending request because round 1 was the only round; this helper is asked on every pass, so it must re-check. On pass 1 the seeded request has no response, so the operation applies. On pass 2 the response is in the history — the ID `call_seed_001` appears in a tool response — so the same helper returns an empty list and the operation declines.

Inside the operation, everything from Lessons 5–6 is unchanged: `execute_allowed_tool` runs the parse, allowlist, and domain boundaries, and the response is correlated to the request by cloning its ID into `with_tool_response`. The machine changed who calls this code; it did not change what the code guards.

## Step 7.4: Yielding and termination

**Yield** is how an operation hands control back to the client — the thing driving the agent. After a tool response lands in the conversation, a real agent would run inference again; this lesson has no inference step, so the machine yields instead. The yield is itself an operation, so the same re-derivation discipline applies to it:

```rust
struct YieldToClient;

#[async_trait]
impl Operation<TradingSession> for YieldToClient {
    fn name(&self) -> &'static str {
        "yield_to_client"
    }

    async fn run(
        &self,
        _session: &TradingSession,
        conversation: &Conversation,
        _emit: &Emitter,
    ) -> anyhow::Result<OperationResult<ConversationEffect>> {
        let last_role = match last_effective_role(conversation.messages()) {
            Ok(role) => role,
            Err(_) => return not_applicable(),
        };
        if matches!(last_role, EffectiveRole::Tool) {
            return yielded();
        }
        not_applicable()
    }
}
```

The step fires only when the conversation ends in a tool response — Lesson 6's *effective role* `tool` is the condition, read back from the persisted history. `last_effective_role` is a helper the GDK ships for exactly this kind of read.

There are three ways this machine stops, and the lesson's run reports which one fired:

1. **No operation applies** — every step in the list declined for the current state.
2. **A step yielded** — an operation decided the client should decide next.
3. **The state-step bound was reached** — the application's limit on applied steps fired.

Registering steps matters. A machine whose list contains only `answer_pending_tools` ends pass 2 with "no operation applies" — nothing declines the yield, because no yield step exists. The same history with both steps registered ends with a yield. The list is data; a different list is a different agent.

## Step 7.5: Assembling the machine and the bounded pass loop

The machine is assembled from the ordered steps and a cancellation token, and driven by an explicit pass loop:

```rust
let steps: Vec<Step<'_, TradingSession>> = vec![
    Step::Operation(Arc::new(AnswerPendingTools)),
    Step::Operation(Arc::new(YieldToClient)),
];
let cancel = CancellationToken::new();
let machine = StateMachine::new(steps, cancel.clone());

let (event_tx, mut event_rx) = mpsc::channel::<AgentEvent>(64);
let emit = Emitter::new(event_tx, cancel);
```

Step order matters: tool requests must be answered before the yield step is even consulted. The emitter streams events to the client while a step runs; this program is the client, so it drains the channel between passes.

The pass loop mirrors the machine's own `run` — with one deliberate difference:

```rust
async fn run_state_machine(
    machine: &StateMachine<'_, TradingSession>,
    store: &InMemoryStore,
    emit: &Emitter,
    event_rx: &mut mpsc::Receiver<AgentEvent>,
    max_steps: u32,
) -> RunOutcome {
    let mut applied_steps = 0;
    loop {
        if applied_steps >= max_steps {
            println!(
                "\n  state-step bound reached after {} applied step(s); the machine stops",
                applied_steps
            );
            return RunOutcome::StepBoundReached;
        }

        let session = store.load(SESSION_ID).await.expect("session load");
        println!(
            "\n  ── Pass {}: reload session '{}' ──",
            applied_steps + 1,
            session.id()
        );

        let step_result = machine.step(&session, emit).await.expect("machine step");
        let mut result = match step_result {
            Some(result) => result,
            None => {
                println!("  no operation applies: the machine stops");
                return RunOutcome::StoppedNoOperation;
            }
        };
        match result.applied_step {
            Some(name) => println!("  step '{}' applied", name),
            None => println!("  a step applied"),
        }

        machine
            .apply(store, &session, &mut result, emit)
            .await
            .expect("apply effects");

        applied_steps += 1;

        // Events the step emitted while it ran: this application is the client.
        while let Ok(event) = event_rx.try_recv() {
            if let AgentEvent::Message(message) = event {
                println!("  event: message appended (id {:?})", message.id);
            }
        }

        if result.yield_to_client {
            println!("  a step yielded control to the client: the machine stops");
            return RunOutcome::YieldedToClient;
        }
    }
}
```

A **state-step bound** is an explicit limit on how many steps one run applies. It is the counterpart of Lesson 6's round bound: the application decides when the machine stops, not the model and not an unbounded loop. The GDK's own `run` method contains this same loop — reload, step, apply, stop on yield — but takes no step-limit parameter, so this lesson writes the loop by hand to keep the bound explicit and visible. Lesson 9 hands the loop to `run`.

Note the reload inside the loop: the session is loaded fresh on every pass, and `machine.step` is called with that fresh view. The machine never caches state between passes — persistence is the only memory.

## Step 7.6: Reading the terminal: replayed payload versus commentary

The lesson receives no provider traffic, but the seeded session is protocol payload — it is exactly what the provider sent in Lesson 6 — and the dispatched tool result is payload this application sends as the tool. The **payload delimiter** convention from Lessons 5–6 applies unchanged: every block of payload is printed between two identical `++++++++` lines, and the line immediately before each block names the sender and the receiver:

- the seeded system instruction, labeled `application → provider, replayed`;
- the seeded assistant tool request, labeled `provider → application, replayed from Lesson 6`;
- the dispatched tool result, labeled `application, as the tool → provider` — the tool execution boundary lives inside this application.

Everything outside those lines is the application's commentary: the pass headers, the applied-step names, the persisting-effect lines, and the yield report. The same one constant and one helper carry the convention: `PAYLOAD_DELIMITER` holds the fence, and `print_delimiter()` prints one line of it.

One payload fact the run makes visible: the tool result text lives **inside** the `ToolResponse` content block, not in a plain text block, so a reader that only concatenates text blocks sees the user prompt but not the `$100.00`. The lesson's tests extract the tool result by hand — walk the content blocks, take `as_tool_response()`, then walk the result's own content. Knowing where a payload lives inside a message is part of reading protocol state.

## Expected structural behavior

- The run prints the seeded session (system instruction and assistant tool request, both fenced and labeled as replayed payload), then the machine run, then the final persisted conversation summary.
- Pass 1: the session reloads with 2 messages; `answer_pending_tools` applies; the deterministic result `$100.00` prints between payload delimiters, labeled `application, as the tool → provider`; one `AppendMessage` effect is persisted with effective role `Tool`.
- Pass 2: the session reloads with 3 messages; `answer_pending_tools` declines (the request is answered); `yield_to_client` fires (the last effective role is `Tool`); the machine reports the yield and stops.
- The final persisted conversation holds 3 messages: user text, assistant tool request, user tool response. That is what a future pass would reload.
- The machine stops via yield, not via the bound: the deterministic scenario needs exactly two passes, and `MAX_STATE_STEPS` is 3 so the bound is never the reason the run ends.
- No provider is contacted, no `.env` is read, and no network call happens. The trace is identical on every run.
- Tool boundaries are unchanged: the parse, allowlist, and domain boundaries from Lessons 5–6 guard execution inside `execute_allowed_tool`.

## Success criteria

- The conversation is persisted in a store keyed by session id, and every pass begins by reloading it through `SessionLoader`; the machine never holds state across passes.
- An operation's decision is a function of the conversation it is handed: the same `answer_pending_tools` operation applies on pass 1 and declines on pass 2, with no loop counter or in-memory flag involved.
- Effects are the only way state changes: the operation returns `ConversationEffect::AppendMessage`, and only the runtime's `EffectHandler` writes it back.
- The machine yields to the client when the conversation ends in a tool response, and the run reports the yield as the reason it stopped.
- The state-step bound is explicit (`MAX_STATE_STEPS`) and checked before each pass; removing it would be an unbounded loop, which is why the lesson writes the bound by hand.
- Step registration is data: a machine registered with only `answer_pending_tools` ends with "no operation applies" instead of a yield — the test suite asserts both behaviors.
- The tool path is byte-for-byte the Lesson 6 discipline: untrusted arguments, deny-unknown-fields deserialization, exact decimal arithmetic, allowlisted dispatch, and a correlated user-role tool response.
- Unit tests cover the deterministic scenario (seed request answered, `$100.00` in the tool response payload), the state-step bound, "no operation applies" without a yield step, and the Lesson 6 rejection paths — all without a live provider.

## Validation

This lesson makes no provider calls, so `cargo run` is a deterministic trace you can run anywhere:

```bash
cargo test
cargo run
```

Record the structural outcome:

- Did pass 1 print the tool result `$100.00` between payload delimiters and persist one effect with effective role `Tool`?
- Did pass 2 print `answer_pending_tools` declining and `yield_to_client` firing?
- Did the run stop via yield with the final conversation holding 3 messages?
- Where in the output does the replayed tool request appear, and which actor labels name it?

There is no live-provider run to record in this lesson. If a provider is desired for demonstration, that is Lesson 8: the provider returns as an inference step inside the machine, not as hand-written rounds.

## Next

Lesson 8 walks the machine through one simple request and response: the GDK-shipped `InferenceRunner` is registered as the sole `Step::Inference`, so the provider re-enters inside the machine exactly where `yield_to_client` fires today. One provider call produces the streamed reply, which is persisted as effects; a second pass finds no step that applies and the machine stops. Lesson 9 then registers the typed tool alongside inference and hands the loop to the GDK's own `run` — which loads the session and applies steps without the hand-written bound loop this lesson mirrors.
