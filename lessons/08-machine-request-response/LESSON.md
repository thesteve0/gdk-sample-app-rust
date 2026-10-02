OK # Lesson 8: One Request and One Response through the GDK State Machine

## Goal

Walk through the GDK state machine step by step with the simplest possible scenario: one seeded user question, one registered step, one provider call, one reply, one stop. **Seeded** means the question is placed into the store by hand before the machine starts, rather than arriving from a live user mid-run — the same setup Lesson 7 practiced. The question is the same one Lessons 3–4 used — **"What is the capital of France?"** — because nothing about the scenario should compete with the machinery for attention. There are no tool calls, no tool definitions, and no domain boundaries in this lesson; the entire run is one request and one response, coordinated by the machine.

Lesson 7 introduced the machine's vocabulary over a deterministic, provider-free seed. This lesson is where the machine is walked through end to end: the full pass loop, a real provider call inside it, and a real stop. Lesson 7's machine promised the provider for later; this lesson keeps that promise — the provider enters the machine as an **inference step**, and the run streams an actual reply.

## How the state machine works, in plain words

Before any of this lesson's vocabulary, here is the whole machine in ordinary language. Everything the rest of the lesson says is a precise version of this picture, so it is worth reading slowly.

**There is one state: the conversation so far.** One object holds every message exchanged so far. In this lesson it starts with a single message in it — the user's question — and nothing else in the run matters except what that object contains.

**The state lives outside the machine.** This detail matters more than any other, and it is easy to miss. The state is kept in a *store* — in this lesson, a map in the program's memory; in a real application, a database or a file. Each pass of the loop reads the current state out of the store, works on it, and writes changes back as it goes. The machine itself remembers nothing between passes; every pass starts from what the store holds.

**The loop runs passes, and every pass asks the same question.** A pass does three things, in the same order every time: look at the current state; ask whether anything is still owed — in this lesson there is exactly one thing that could be owed, a reply from the provider to the latest user message; and if something is owed, do it in a way that changes the state, then start the next pass. If nothing is owed, the machine finishes.

```text
        the state: the conversation so far
        (kept in the store; starts as one message:
         the user's question)
                        │
                        v
   ┌───────────────────────────────────────────────────┐
   │ PASS: look at the state. Is anything still owed?  │
   │ (here: does the provider owe a reply to the       │
   │ latest user message?)                             │
   └────────────┬────────────────────────────────┬─────┘
            yes │                                │ no
                v                                v
   ┌───────────────────────────┐   ┌─────────────────────────────┐
   │ DO IT: call the provider, │   │ FINISH: the machine stops.  │
   │ write the reply into the  │   │ The state stays in the      │
   │ state, then pass again.   │   │ store, in case a follow-up  │
   └───────────────────────────┘   │ message arrives later.      │
                                   └─────────────────────────────┘
```

**What happens to the state when the machine finishes?** The machine stops; the state does not. The saved conversation stays in the store under its session id (`lesson-08` here), exactly as the last pass left it. This lesson's program shows it directly: after the machine has finished, `main` reads the session out of the store one more time, just to print it. The machine was done; the state was still there to be read.

That is also the answer to a question the machine cannot answer itself: *will the user ask a follow-up?* The machine has no way to know, and it does not try. It finishes and leaves the conversation in the store. In this lesson the store lives in the program's memory, so when the process exits, the state goes with it — nothing actually waits. In a real application the store is durable: the conversation sits there, finished but not deleted, and if a follow-up message ever arrives, the application appends it to the same session and starts the machine again. The machine reloads the whole history and carries on as if it had never stopped. A later lesson in this course builds exactly that.

**The single request and response, in these terms:**

1. **Before the machine starts.** The state exists with one message in it — the user's question — already saved in the store.
2. **Pass 1, look and ask.** The machine reads the state: one message, ending with the user's question. Is anything owed? Yes — the provider has not replied.
3. **Pass 1, do it.** The machine calls the provider and sends the whole state with the call; the provider keeps no memory of its own, so the entire conversation travels on every request. The reply comes back a few words at a time.
4. **Pass 1, write it back.** The machine writes the reply into the state and saves it. The pass ends, and the loop starts over.
5. **Pass 2, look and ask.** The state now ends with the provider's reply. Is anything owed? No — the reply is already in the history. The machine finishes, and the state, now two messages, stays in the store.

One safety net sits around the loop, and this scenario never touches it: the application bounds how many passes may run — four here — so a step that kept applying forever could not spin the machine indefinitely.

**The same story in the lesson's terms.** When the words below appear later in the lesson, this is how they map back to the picture:

- the *state* — the persisted conversation, read fresh from the store at the top of every pass (*session loading*)
- *is anything still owed?* — a step is asked whether it *applies*; for the inference step, that means asking whether the conversation ends in a *provider turn*
- *do it* — the inference step calls the provider and returns *effects*
- *write it back* — the effect handler persists each effect into the store
- *the machine finishes* — no step applies (pass 2 of this lesson's run)
- *the state stays in the store* — the session id; the same conversation any future run would reload

## The machine, walked through one pass

The picture above gave the machine in plain words; this section retells the same run one pass at a time with the lesson's vocabulary attached. Everything in this section is what the run does; the steps after it show where each piece lives.

**Stage 1 — Reload.** A pass begins by loading the session from the store. The conversation in this lesson holds exactly one message: the user's question. The machine never caches state between passes; whatever the store holds is the entire state of the world. This is the *persisted conversation state* from Lesson 7, and the *session loading* that reads it.

**Stage 2 — Ask the steps.** The machine walks its registered steps in order. This lesson's list holds exactly one step, and it is a new kind: an **inference step** — the step that asks the provider for the next message. Asking an inference step means checking first whether it *applies*: the step re-derives its decision from the conversation it was handed, exactly the discipline Lesson 7's hand-written operations followed. The GDK's shipped inference step locates the **kickoff** of the current exchange — the most recent visible user message that is not a tool response — and re-derives from the history that starts there whether the provider owes a reply. The conversation, projected as the provider will see it, must end in a **provider turn**: its last message must be addressed to the provider — a user message or a tool response — rather than coming from the provider. On pass 1 the conversation ends in the user's question, so the step applies. Nothing about this decision is remembered from a previous pass; it is read from the history every time.

**Stage 3 — The provider call.** When the step applies, the machine collects what every registered operation contributes — prompt parts and tool definitions — and hands them to the inference step. This lesson registers nothing else, so the provider receives only the conversation: no system instruction, no tools. The provider is stateless, so the outbound payload is the whole conversation: one user message. The reply comes back as a stream — small chunks of text — and the inference step emits one event per chunk for the client to watch, while accumulating the chunks itself.

**Stage 4 — Apply the effects.** The step returns its effects, and the machine hands them to the runtime: the token usage first, then the assistant reply. The effect handler persists each one into the store. Two facts are worth holding onto. First, the streamed events the client watched are *presentation, not state* — if the client never drained them, the reply would still be persisted, because effects are the only way the world changes. Second, the machine asks every effect to guarantee a message id before persisting, so every stored message carries one.

**Stage 5 — Stop checks.** After applying, the pass checks two stop conditions: whether the step yielded control to the client, and whether the application's state-step bound was reached. In this lesson neither fires, so the loop begins pass 2.

**Pass 2 — the same steps, a different state.** Reload: the store now holds two messages. Ask: the same inference step re-derives from the reloaded history — and the conversation now ends in the assistant's reply, which is *not* a provider turn. The step declines. No other step is registered, so the machine reports that no step applies, and the run stops. One request produced exactly one response — not because the application counted rounds, but because the persisted history is what every decision reads.

## Concepts introduced

- **Inference step** (`Step::Inference`): the machine's second kind of step, next to the operations Lesson 7 used. An inference step asks the provider for the next message; the machine calls it through the same `step` half of the loop it uses for operations.
- **InferenceRunner**: the GDK's shipped inference step, registered like any other step. Its name is `llm`, and that is the name the run prints when it reports which step applied. It is constructed with a provider and a model configuration, and it owns the streaming loop.
- **Kickoff**: the most recent visible user message that is not a tool response — the message that starts the current exchange. The inference step re-derives its decision from the history that begins at the kickoff.
- **Provider turn**: the state in which the conversation's last message is addressed to the provider — a user message or a tool response — so the provider owes a reply. The inference step applies only in a provider turn. Once the assistant's reply is persisted, the conversation ends in a provider-produced message and the same step declines. The step also declines if the conversation ends in an errored message.
- **Applies (for inference)**: the re-derivation that gates the provider call. The machine asks it before calling the step, so the provider is never contacted blindly.
- **Streaming event and drain**: the inference step emits one event per streamed chunk while it runs; the client drains the channel after the step returns. Events are how the client watches the step; they are not state.
- **Usage effect**: the inference step reports token usage as an effect, in the same list as the reply — not on a side channel. This lesson's runtime prints it.
- **Effect type requirement**: the machine requires of any effect type that it can guarantee message ids (`MachineEffect`) and be built from a message (`From<Message>`); the shipped inference step additionally requires one that can carry a usage record (`InferenceEffect`). This lesson defines an effect vocabulary meeting all three.
- Reusing Lesson 7 terms unchanged: persisted conversation state, session, session loading, effect, effect application, re-derivation from persisted state, yield, state-step bound, and the payload delimiter.

## Where this fits in the agentic application

Lesson 7 practiced the same pass loop with two differences. Its registered steps were written by hand in the lesson's own code, and the provider was deliberately never contacted — the practice conversation was placed into the store by hand, too. This lesson changes only the step list: the GDK's shipped inference step is registered, so the *do it* beat of the pass is a real provider call, and the reply written back is a real model response. Even the stop differs for an interesting reason: Lesson 7's steps ended by *yielding* — deliberately handing control back to the client — while this run stops by itself, because no step applies to the finished conversation.

The machine's contract did not change between the lessons — reload, ask, apply, stop — only the step list did. That is the point of the abstraction: a different list is a different agent, and this list is the smallest agent that can ask a provider a question and persist the answer.

### Diagram 1: the pass loop with a real provider

```text
                    ┌────────────────────────────────────────────┐
                    │  reload the persisted session by id        │
                    │  (SessionLoader -> MachineSession)         │
                    └───────────────────────┬────────────────────┘
                                            v
                    ┌────────────────────────────────────────────┐
                    │  StateMachine::step: ask the steps,        │
                    │  in order, for the current conversation    │
                    └───────────────────────┬────────────────────┘
                                            v
              applies? ── no ──> no step applies ──> STOP  (this lesson's pass 2)
                 │
                 v (llm applies: provider turn)
                    ┌────────────────────────────────────────────┐
                    │  InferenceRunner::infer                    │
                    │    outbound: whole conversation (stateless │
                    │      provider); no prompt parts, no tools  │
                    │    inbound: streamed chunks, one event each│
                    └───────────────────────┬────────────────────┘
                                            v (effects: usage, then reply)
                    ┌────────────────────────────────────────────┐
                    │  StateMachine::apply: EffectHandler        │
                    │  persists each effect into the store       │
                    └───────────────────────┬────────────────────┘
                                            v
                              yield_to_client? ── yes ──> STOP (client decides next)
                                            │ no
                                            v
                              step bound reached? ── yes ──> STOP
                                            │ no
                                            └────> reload again (next pass)
```

The provider sits outside the machine. The machine never holds provider state; it holds conversation state, and the inference step is the only place the provider is contacted.

### Diagram 2: who holds what

```text
APPLICATION                            GDK (goose-agent crate)          PROVIDER
───────────────────────────────        ───────────────────────────      ────────────────
the store: persisted conversation      StateMachine: the pass loop      the model:
  state under a session id             Step::Inference(InferenceRunner): stateless — holds
ChatRuntime: SessionLoader +             asks applies(), makes the        no conversation;
  EffectHandler (persistence)            provider call, returns effects   streams chunks and
ChatEffect: the effect vocabulary      Emitter: one event per chunk        reports usage
the pass loop (MAX_STATE_STEPS)        machine: stamps the applied step,
                                         asks effects for message ids
```

The application owns state and persistence; the GDK owns sequencing and the shipped inference step; the provider owns nothing. Every pass re-reads the store, so a crash between passes loses nothing the store did not record.

### Diagram 3: the two-pass trace

```text
SEED  (built in code)                       session 'lesson-08'
  1. user text: "What is the capital of France?"

PASS 1
  reload 'lesson-08'                    -> 1 message
  llm: applies?
    kickoff: the user question          -> the turn starts there
    projected history ends in a User message (a provider turn) -> applies
  infer:
    outbound = whole conversation       -> 1 message, no system prompt, no tools
    provider streams the reply          -> chunk events; the client drains them
  effects: [record usage, append reply] -> persisted in that order
  store now holds 2 messages

PASS 2
  reload 'lesson-08'                    -> 2 messages
  llm: applies?
    projected history ends in the Assistant reply -> not a provider turn
  -> declines; no other step exists     -> "no step applies" -> the machine stops

FINAL PERSISTED CONVERSATION (what the next reload would see)
  1. user text: "What is the capital of France?"
  2. assistant text: the streamed reply
```

The run stops because of what the history now *contains*, not because a counter ran out. Delete the persist step in your head and re-run pass 2: the reply would be missing from the reloaded history, the conversation would still end in a provider turn, and the provider would be asked again. Persistence is what makes the stop.

## Prerequisites

- Lessons 1–7 are complete. Lesson 7's machine vocabulary — persisted conversation state, session, operation, effect, yield, state-step bound — carries over unchanged.
- **No new dependencies.** The GDK agent crate pinned in Lesson 7 (`goose-agent = "=0.1.0-alpha.11"`) already ships `InferenceRunner` and `Step::Inference`; the provider stack from Lessons 2–6 is unchanged.
- A reachable provider. The bundled provider JSON declares the endpoint, and the bundled default requires no API key. This lesson's `cargo run` contacts the provider — the first state-machine lesson that does.
- `dotenvy` loads a local `.env` before the provider is constructed, as in every provider lesson.

## Step 8.1: Seeding one request and registering the inference step

The seed is one user message. The provider is stateless, so this conversation is the whole context of the request — there is nothing else to send.

```rust
fn seed_conversation() -> Conversation {
    let mut conversation = Conversation::empty();
    conversation.push(Message::user().with_text(USER_PROMPT));
    conversation
}
```

The provider is constructed the same way Lessons 2–6 built it: read the declarative JSON, resolve it with `from_json`, and select the first configured model — the course convention until the later model-selection lesson. One conversion is new: `from_json` returns a `Box<dyn Provider>`, and the inference runner wants a shared, reference-counted provider, so the box is wrapped with `Arc::from`:

```rust
let boxed_provider: Box<dyn Provider> = from_json(&provider_json, None, EnvKeyResolver {})?;
let provider: Arc<dyn Provider> = Arc::from(boxed_provider);
let model = ModelConfig::new(first_configured_model(&provider_config)?);
```

Then the machine is assembled. The step list holds exactly one entry, and it is the GDK's shipped inference step:

```rust
let runner = InferenceRunner::new(provider, model);
let steps: Vec<Step<'_, ChatSession, ChatEffect>> = vec![Step::Inference(Arc::new(runner))];
let cancel = CancellationToken::new();
let machine = StateMachine::new(steps, cancel.clone());
```

`InferenceRunner` is the GDK's inference step. Its name is `llm` — the name the run prints when it reports which step applied. Registering it with `Step::Inference` is the machine's second kind of step, next to `Step::Operation` from Lesson 7. When the machine reaches this step it collects the prompt parts and tool definitions that registered operations contribute and hands them to the runner; this lesson registers nothing else, so the provider receives only the conversation.

One assembly detail is inherited from Lesson 7 but worth restating: the event channel is large (4096) because the inference step emits one event per streamed chunk *while it is still running*, and this program drains the channel only after the step returns. The buffer must hold a whole response so the emitter never blocks mid-stream.

## Step 8.2: The effect vocabulary the machine and the runner require

The machine imposes a contract on the effect type any application hands it, and the shipped inference step adds to it. Three requirements, each visible as a trait bound:

1. The machine must be able to **guarantee message ids** — before persisting, it asks every effect to ensure its message carries an id (`MachineEffect::ensure_message_ids`).
2. GDK code must be able to **build an effect from a message** — the inference step converts each streamed message into an effect (`From<Message>`).
3. The shipped inference step must be able to **hand the effect type a usage record** (`InferenceEffect::record_usage`).

The GDK's default `ConversationEffect` satisfies the first two but has no `InferenceEffect` implementation in this pinned release, so this lesson defines its own two-variant vocabulary:

```rust
enum ChatEffect {
    AppendMessage(Message),
    RecordUsage(ProviderUsage),
}
```

Two variants, one per thing this lesson's run persists or records. The three implementations are mechanical, but each exists for one of the requirements above:

```rust
impl MachineEffect for ChatEffect {
    fn ensure_message_ids(&mut self) {
        // The emitter already generated an id for every streamed message, but
        // the machine asks each effect to guarantee one before it persists.
        if let ChatEffect::AppendMessage(message) = self {
            let taken = std::mem::replace(message, Message::assistant());
            let with_id = taken.with_generated_id_if_missing();
            *message = with_id;
        }
    }
}

impl From<Message> for ChatEffect {
    fn from(message: Message) -> Self {
        ChatEffect::AppendMessage(message)
    }
}

impl InferenceEffect for ChatEffect {
    fn record_usage(usage: ProviderUsage) -> Self {
        ChatEffect::RecordUsage(usage)
    }
}
```

`ensure_message_ids` looks roundabout for a reason: `with_generated_id_if_missing` takes the message by value, so the implementation takes the message out of the variant, adds the id, and puts it back. The type system holds the whole contract — an effect type the machine cannot give ids to, or the runner cannot build from a message or hand usage to, does not compile.

The runtime's effect handler applies them in the order the runner returned them. The usage effect is printed — this lesson persists nothing for it — and the reply is written into the store:

```rust
ChatEffect::AppendMessage(message) => {
    let role = effective_role(message);
    println!("persisting effect: append message (effective role {})", role);
    let mut conversations = self.store.conversations.lock().await;
    let stored = match conversations.get_mut(session.id()) {
        Some(stored) => stored,
        None => {
            let failure = format!("no session '{}' exists in the store", session.id());
            return Err(anyhow::anyhow!(failure));
        }
    };
    stored.push(message.clone());
}
ChatEffect::RecordUsage(usage) => {
    println!(
        "persisting effect: record usage (model={}, tokens in={} out={})",
        usage.model, input_tokens, output_tokens
    );
}
```

A later lesson could persist usage for accounting; the effect list is where it already travels.

## Step 8.3: The bounded pass loop

The pass loop is Lesson 7's shape with a provider call in the middle. Every pass is the same: reload, ask the steps, print the round trip, apply the effects, check the stop conditions.

```rust
async fn run_state_machine(
    machine: &StateMachine<'_, ChatSession, ChatEffect>,
    runtime: &ChatRuntime,
    emit: &Emitter,
    event_rx: &mut mpsc::Receiver<AgentEvent>,
    max_state_steps: u32,
) -> Result<RunOutcome, Box<dyn Error>> {
    for state_step in 1..=max_state_steps {
        // 1. Reload. The machine never caches the session; the store is the
        // source of truth at the start of every pass.
        let session = runtime.load(SESSION_ID).await?;
        let conversation = match session.conversation() {
            Some(conversation) => conversation,
            None => return Err("the loaded session has no conversation".into()),
        };
        println!(
            "\n── Pass {}: reloaded the persisted conversation ({} message(s)) ──",
            state_step,
            conversation.len()
        );

        // 2. Ask the registered steps, in order. The inference step is asked
        // whether it applies; if it declines, the machine reports that no
        // step applies and this loop stops.
        let outcome = machine.step(&session, emit).await?;
        let mut result = match outcome {
            Some(result) => result,
            None => return Ok(RunOutcome::StoppedNoOperation),
        };

        // 3. A step applied and already called the provider. The provider is
        // stateless, so the outbound payload is the whole conversation; print
        // it, then the streamed response, so the round trip reads in order.
        let applied_step = result.applied_step.unwrap_or("unknown");
        println!("\nThe '{}' step applied — it asked the provider.", applied_step);
        // ... print the outbound payload between payload delimiters, then
        //     drain and print the streamed response (Step 8.4) ...

        // 4. Persist the effects. This is the only way the world changes:
        // the reply enters the conversation because the effect handler
        // writes it into the store.
        machine.apply(runtime, &session, &mut result, emit).await?;

        // 5. Drain anything the effect application emitted.

        // 6. The machine also stops when a step yields control to the client.
        if result.yield_to_client {
            return Ok(RunOutcome::YieldedToClient);
        }
    }

    Ok(RunOutcome::StepBoundReached)
}
```

Three properties of this loop carry the lesson:

- **Nothing is remembered between passes.** The next pass re-derives the decision from the reloaded conversation. That is why pass 2 stops: not because pass 1 was counted, but because the reply is in the history.
- **The bound is generous on purpose.** `MAX_STATE_STEPS` is 4, and the scenario needs one pass; pass 2 should stop via "no step applies". The bound is the application's backstop — the counterpart of Lesson 6's round bound — never the reason this run ends.
- **The loop is written by hand, as in Lesson 7**, so the bound and the stop reasons stay explicit. Handing the loop to the GDK's own `run` is Lesson 9.

## Step 8.4: Reading the terminal: payload delimiter, actor labels, and the streamed fence

The **payload delimiter** convention from Lessons 5–6 applies unchanged: every block of protocol payload is printed between two identical `++++++++` lines (`PAYLOAD_DELIMITER` and `print_delimiter()`), and the line immediately before each block names the sender and the receiver. Everything between two delimiter lines is a value that traveled to or from the provider; everything else is this application's commentary.

This lesson adds a new payload site — the streamed response — and with it one new wrinkle. Individual deltas cannot each carry a fence of their own, so the delimiter opens at the first text delta and closes after the stream ends: one streamed response is one fenced block. The two payload blocks this run prints, with their actor labels:

- the outbound conversation, labeled `application → provider` — the whole stateless request;
- the streamed reply, labeled `provider → application` — the response as it arrived, chunk by chunk.

```rust
fn drain_and_print_events(event_rx: &mut mpsc::Receiver<AgentEvent>) {
    let mut streamed_text = false;
    while let Ok(event) = event_rx.try_recv() {
        let AgentEvent::Message(message) = event else {
            // The GDK defines other event kinds; none are produced by this lesson.
            continue;
        };
        let text = message.as_concat_text();
        if text.is_empty() {
            continue;
        }
        if !streamed_text {
            print_delimiter();
        }
        print!("{}", text);
        streamed_text = true;
    }
    if streamed_text {
        println!();
        print_delimiter();
    }
}
```

Draining the channel changes only what the client sees, never what is persisted. The events are presentation; the effects are state. That division is the state machine's cleanest rule: a step can show the client anything, but it changes the world only through effects.

## Expected structural behavior

The reply text and token counts vary by provider and model; the structure is what this lesson validates. One real run:

```text
Provider 'custom_aa_llama_qwen3_6-35b' with model 'qwen3.6-35b-a3b'
Seeded session 'lesson-08' with 1 user message: What is the capital of France?

── Pass 1: reloaded the persisted conversation (1 message(s)) ──

The 'llm' step applied — it asked the provider.

Outbound payload sent with this call (application → provider):
++++++++
  [User] What is the capital of France?
++++++++

Streamed response (provider → application):
++++++++
The capital of France is Paris.
++++++++
persisting effect: record usage (model=qwen3.6-35b-a3b, tokens in=17 out=153)
persisting effect: append message (effective role assistant)

── Pass 2: reloaded the persisted conversation (2 message(s)) ──

No step applies to the final conversation — the machine stops.
Final persisted conversation: 2 message(s)
  [User] What is the capital of France?
  [Assistant] The capital of France is Paris.
```

- Pass 1 reloads 1 message; the `llm` step applies; the outbound payload prints between payload delimiters (one user message, no system prompt, no tools); the streamed reply prints between payload delimiters; the effects persist in order — record usage, then append message with effective role `assistant`.
- Pass 2 reloads 2 messages; the step prints nothing because the runner declines; the run reports `No step applies to the final conversation — the machine stops.`
- The final persisted conversation holds 2 messages — what a future pass would reload.
- The machine stops via "no step applies", never via the bound: `MAX_STATE_STEPS` is 4 and the scenario needs one pass.
- Exactly one provider call happens in the whole run.

## Success criteria

- One request, one response: exactly one provider call; pass 2's decline is re-derived from the persisted reply, not counted by the application.
- The provider call is gated: `machine.step` asks the inference step whether it applies before calling it; the provider is never contacted blindly.
- The conversation is persisted in a store keyed by session id and reloaded at the start of every pass; the machine never caches state across passes.
- The reply and the usage record arrive as effects — the only way state changes — and the effect handler applies them in the order the runner returned them (usage, then reply).
- The effect type satisfies all three requirements (`MachineEffect`, `From<Message>`, `InferenceEffect`), and the lesson explains which component requires each.
- The streamed response reaches the client only as drained events; draining them changes nothing in the store.
- The payload fences and actor labels are consistent with Lessons 5–6: `application → provider` for the outbound conversation, `provider → application` for the streamed reply, with the streamed fence opened at the first delta and closed after the stream.
- The stop reason is reported, and it is "no step applies" — the bound and the yield are present in the loop's contract but do not fire in this scenario.
- Unit tests cover the seed, the loader round trip, effect persistence, a declining step stopping the machine, and the state-step bound — all without a provider.

## Validation

This lesson contacts the provider, so the endpoint declared in the provider JSON must be reachable. The bundled default requires no API key.

```bash
cargo test
cargo run
```

Record the structural outcome:

- Did pass 1 print the outbound conversation and the streamed reply between payload delimiters, with the correct actor labels?
- Did the persisting lines print in order — record usage, then append message?
- Did pass 2 stop with `No step applies to the final conversation — the machine stops.`?
- Does the final persisted conversation hold exactly 2 messages?

If the provider is unreachable, ask the instructor to start or supply one; do not silently replace live validation with mocks. The unit tests check the seed, the loader, the effects, and the stop conditions deterministically, but the provider call through the machine is the point of this lesson.

## Next

Lesson 9 registers a second participant alongside inference: a typed deterministic tool, coordinated by the machine. One machine run will ask the provider, receive a structured tool request, execute the tool through the machine, and return the correlated result — the Lesson 5–6 scenario reassembled under the state machine — and the lesson hands the pass loop to the GDK's own `run`. The France question stays one request and one response; the step list is what grows.