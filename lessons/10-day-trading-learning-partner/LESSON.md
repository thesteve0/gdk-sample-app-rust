# Lesson 10 — Build a day-trading learning partner

## Goal

Turn the supplied GDK application into a customized educational partner: the learner selects **Getting started in day trading**, the application loads its own material and teaching instructions, and the learner asks questions in a focused, continuing conversation.

**Status:** implemented; structural checks and live scenarios passed; instructor review pending. This lesson does not release Lessons 8 or 9. Their instructor reviews, and Lesson 9's live 100-share tweak, remain separate gates.

The new idea is **application-owned teaching context**, not another agent loop. We reuse the state machine, streaming, saved history, and safe optional calculator from Lesson 9.

## Concepts introduced

- A **learning module** is a named body of educational material and guidance selected by the learner. This implementation has one module, not an automatically discovered catalogue.
- **Educational material** supplies the concepts, definitions, examples, and source attribution the tutor can use. **Teaching guidance** tells it how to teach that material. Separating them lets us change the teaching approach without changing Rust or the domain explanations.
- The **tutor contract** is the common instruction describing the educational role, conversational behavior, and limits. Instructions influence generated behavior; they are not a substitute for application-enforced capability boundaries.
- **Runtime-loaded content** is text read from files when a module is selected, rather than compiled into the executable. The selected content stays fixed for that conversation.
- **Prompt parts** are named pieces of application instruction contributed before inference. The supplied GDK inference preparer combines their text into the system instruction. They are not learner replies.
- A **lesson selection boundary** starts a fresh Session with the selected context. A **reply boundary** appends a learner message to that same Session and starts another machine run.
- A **diagnostic log** is a developer-facing record for inspecting context, saved state, tool exchanges, and failures. It is not a persistent learner profile or input to future conversations.

Reuse the earlier meanings of Session (recorded conversation and related data), Operation (a rule for work), Effect (a proposed saved-state change), and Event (information for presentation). The application owns input and output; the GDK owns the passes within each run.

## Prerequisites and complete supplied implementation

Use the pinned Linux/Rust setup, a valid provider JSON, and a reachable model supporting streaming and tool calls. Understand Lessons 7–9, especially Events versus Effects, successive runs in one Session, and correlated tool responses. Their review status is not changed here.

The entire [reference source](src/main.rs), supporting runtime, chooser, reply handling, diagnostics, and safeguards are supplied. There are no missing-code exercises. Lesson sources share the root `Cargo.toml` and lockfile; this directory is **not** an independent Cargo package.

The chooser adds one exact dependency, already supplied in the root manifest:

```toml
crossterm = { version = "=0.29.0", default-features = false, features = ["events"] }
```

It provides terminal key events and raw-mode support for selection, not a full CLI framework. The GDK crates remain exactly `0.1.0-alpha.11`; model selection still uses the first model declared in the provider JSON.

Run from the repository root:

```bash
cargo run -- custom_aa_llama_qwen3_6-35b.json
```

With no positional argument, the bundled provider path resolves against the shared manifest directory. An explicit relative provider path resolves against the **current working directory**. `.env` is loaded before provider construction. You may use your own valid JSON; this course does not require the bundled endpoint or model.

Content and `session-output.md` resolve against the shared manifest directory regardless of the working directory. The executable requires the checkout's content files to remain present: it does not embed them or fetch them online. Running the reference through the shared root manifest uses the **same** content directory, not a separate copy beside the reference source. If restoring the reference to root, save your own edits first:

```bash
cp lessons/10-day-trading-learning-partner/src/main.rs src/main.rs
```

## 10.1 — Predict, select, and run

Predict the opening before running: should an educational partner immediately deliver a lecture, or first ask what the learner wants to understand?

1. Launch the application. On a normal terminal, **Getting started in day trading** is the first, initially selected entry. Up/down switches between that entry and **Quit**; Enter selects. Esc or `q` exits the chooser.
2. If input or output is redirected, or `TERM=dumb`/`NO_COLOR`, use the numbered chooser: `1` selects the module; `q` quits. `NO_COLOR` also selects the numbered fallback so there are no application-added cursor-control escapes.
3. Select the module. The application starts the conversation automatically; no learner message is required to trigger the welcome.
4. Reply, for example: “I am new to this. I want to understand orders, not choose a stock.” Then ask a follow-up such as “Why might a limit order remain unfilled?”
5. Observe whether the tutor adapts to those interests and answers the follow-up using the existing conversation.

The **opening input** is an application-created user-role message that requests the welcome. It is not a quotation of something the learner typed:

```rust
const OPENING_INPUT: &str =
    "Begin the selected learning conversation by asking about my experience and learning goals.";
```

Each selection seeds this input in a fresh Session. Subsequent learner replies are appended to that Session; they do not start new modules. The model is instructed to ask about experience and goals, but the wording and adherence must be observed, not assumed.

| Learner action | Application behavior |
|---|---|
| Nonempty ordinary reply | Save it in the current Session and run the machine again |
| Empty/whitespace-only reply | Ignore it; do not call the provider |
| `/finish` | End this conversation and return to the chooser |
| Select the module again | Reload files and create a fresh Session, without the prior conversation |
| `/quit` | Exit the application |
| End of input (EOF) in line input | Exit, rather than keep asking for input |

`/finish` and `/quit` are application commands, not provider prompts. Returning to the chooser does not certify learning completion.

## 10.2 — Explain what customizes the tutor

Open the three content files before navigating Rust:

| File | Purpose |
|---|---|
| [tutor-contract.md](content/tutor-contract.md) | Educational role, respectful teaching behavior, source honesty, calculator use, and capability limits |
| [getting-started-material.md](content/getting-started-material.md) | Course-authored explanations and examples, with source attribution |
| [getting-started-guidance.md](content/getting-started-guidance.md) | Welcome, flexible conversation, beginner priorities, and pacing |

The material offers four teaching units: day trading versus longer-term investing; orders and execution; costs and execution conditions; risk and simulated practice. These are available topics, **not** a compulsory sequence or mastery checklist. Listening to an explanation does not prove understanding or readiness to trade.

The application reads all three files at selection and stores them as **lesson context**, the selected content attached to the Session. Missing, unreadable, or empty content causes an error, not an invented substitute. An Operation contributes the context as prompt parts before inference:

```rust
    fn prompt_parts(&self) -> Vec<(String, String)> {
        vec![
            ("tutor-contract".to_string(), self.contract.clone()),
            ("educational-material".to_string(), self.material.clone()),
            ("teaching-guidance".to_string(), self.guidance.clone()),
        ]
    }
```

The supplied adapter contributes those selected parts through the same Operation boundary used in Lesson 9:

```rust
    async fn prompt_parts(
        &self,
        session: &ChatSession,
        _conversation: &Conversation,
    ) -> anyhow::Result<Vec<(String, String)>> {
        Ok(session.lesson.prompt_parts())
    }
```

This is deliberate application assembly: the learner selects; the application loads; inference receives the selected context alongside conversation history and the calculator definition. There is no model-driven skill discovery, retrieval index, or browsing. Source links are attribution, not URLs fetched by the tutor. Goose's tutorial extension is an orientation pattern, not a runtime dependency.

## 10.3 — Explain continuity and the optional calculator

A machine run finishes before the application waits for the next reply. Saving a new reply creates work for another run; it retains the earlier messages and selected lesson context. Only per-turn safeguard state resets. `/finish` ends that continuity; a new selection is a new Session. Nothing reloads old conversation from disk at launch.

The reply boundary first records the learner's input, then starts the next run. These lines execute only after the previous run has finished:

```rust
            runtime.append_user(&session_id, &input).await?;
            turn_number += 1;
            run_turn(
                &machine,
                &runtime,
                &cancel,
                &session_id,
                turn_number,
                diagnostics,
            )
            .await?;
```

The supplied assembly keeps the safeguard Operation first, the safe tool adapter next, and inference last. `StateMachine::run` still owns loading, first-applicable work, Effect application, and re-evaluation. The chooser/reply loop coordinates **between runs**; it is not a replacement manual agent loop.

The **maximum planned loss** calculator measures the supplied entry-to-stop difference times a positive whole-number share count for a hypothetical long stock position. It excludes fees, slippage, and gaps through the stop; it is not statistical expected loss or a guaranteed ceiling on actual loss.

Tool availability does not require tool use. Predict no request for “What does stop-loss mean?” For a numerical follow-up, try:

> For a hypothetical long stock position, calculate maximum planned loss with entry 51.20, stop 50.70, and 200 shares. Explain what that number excludes.

Predict `(51.20 − 50.70) × 200 = $100.00`. Observe the actual request and result; do not infer execution from generated prose alone. The **tool execution boundary** remains inside the application: names and arguments are untrusted. Only the calculator is allowlisted. Its strict argument shape rejects unknown fields and incorrect types; decimal-string prices are parsed immediately using exact decimal arithmetic, must be positive, and allow at most four fractional places. Shares are a positive whole number; entry must exceed stop. Checked arithmetic rejects out-of-range calculations and dollar output uses two places.

A **correlation ID** connects each request with its saved response. The adapter examines all request blocks, not just the first. Unknown, malformed, or excess requests receive correlated errors rather than unauthorized execution. Empty or duplicate IDs halt because safe correlation is impossible. Nonfatal calculator rejections are reported; missing final answers, inference failures, safeguards, and logging failures stop the application rather than silently continuing.

The unchanged supplied bounds are **eight applied work passes per turn**, plus a possible ninth terminal halt batch; **four aggregate requests per turn**, including unknown or malformed requests; and **180 seconds per run**. A timeout/pass bound can leave partial state with unanswered requests. Diagnostics retain available state, not a fabricated success. Timeout cancellation is not rollback or proof that a remote server stopped.

There are no brokerage, order-entry, shell, browsing, retrieval, or market-data tools. A prompt guides appropriate teaching; the absence of execution capabilities enforces the no-trade-execution boundary.

## 10.4 — Explain learner output versus diagnostics

The learner interface intentionally omits Session snapshots, complete instructions, and raw argument dumps. It shows the chooser, reply prompt, tutor stream, compact calculator activity notice and framed result, and relevant warnings/errors.

A **payload delimiter** distinguishes live provider text from application commentary: the tutor stream opens and closes with `++++++++`. Its actor label is `Tutor (provider → application):`. Calculator activity notices and framed results retain `provider → application` and `application, as the tool → provider` plus the correlation ID. The result is fenced as payload; the request notice is not a complete request dump; full details are in diagnostics. No example transcript here claims a generated welcome or tool decision actually occurred.

Presentation follows the course palette: bold cyan headings/ordinary actor labels, blue learner prompt, magenta request notices, green results/status, yellow warnings, bold red stderr errors, and normal-foreground model text. Styling is printing-only, disabled for redirected streams, any `NO_COLOR`, or `TERM=dumb`, with stdout/stderr checked separately.

Open the root **`session-output.md`** after a run. It is truncated once at launch, then appended across turns and fresh selections within that launch. It contains:

- complete selected prompt parts and calculator definition, explicitly **not an exact serialized provider request**;
- the application-owned opening input or learner input and complete Session before/after each run;
- every Session field, including nested message/usage metadata, absent/default values, counters, halt reason, and the full lesson context;
- saved request names, arguments, IDs, results/errors, assembled tutor text, tool presentation Events, warnings, and failure details when available.

The Markdown file is plain: no ANSI styling, gold-field rendering, or deterministic Session colorization. It records assembled saved responses, not a replay of every live delta. Displaying Events does not save messages; the effect handler saves them. Presentation-only request markers never enter saved conversation or provider input.

**Privacy and failure rule:** this ignored file may contain personal conversation. Git ignore is not access control, encryption, or permission to share it. The application does not deliberately log provider configuration or credentials, but user/model text can still contain sensitive information. File creation/write/flush errors are fatal; the application stops. If the file itself is unavailable, it cannot guarantee recording its own logging failure. This file is not loaded as future learner memory.

## 10.5 — Tweak guidance, then run again

The approved customization changes **Markdown guidance**, not Rust. In `content/getting-started-guidance.md`, replace only this sentence:

> Use short explanations, with an example when helpful, then invite participation.

with:

> Use a short hypothetical example before asking a check-for-understanding question.

Predict what should change and what should stay the same. Restart the application, or use `/finish` and select the module again. Either path reloads the files into a fresh Session; editing the file does not change an already selected conversation. A restart also overwrites `session-output.md`, so preserve any comparison evidence privately before restarting.

Use the same stated learning goal and a comparable question. Compare whether an example now precedes a check-for-understanding question. This is a qualitative observation, not a guaranteed template or proof that one instruction caused every difference. Verify the new guidance in diagnostic context as structural evidence that the file was actually reloaded. Restore the default sentence after the comparison.

## Validation and success criteria

Repository maintenance checks, run from root:

```bash
cargo fmt --check
cargo check
cargo clippy --all-targets
cargo test
cargo run -- custom_aa_llama_qwen3_6-35b.json
```

For reproducible line-input validation against a **live** provider, the following supplies real learner inputs, not expected generated output. Redirection activates the numbered chooser. Inspect each capture and the diagnostic file before launching the next command, which overwrites diagnostics:

```bash
printf '1\nI am new to day trading and want to understand order types.\nWhy might a limit order remain unfilled?\n/finish\n1\n/quit\n' \
  | cargo run -- custom_aa_llama_qwen3_6-35b.json > /tmp/l10-conversation.txt

printf '1\nWhat does stop-loss mean in day trading?\nFor a hypothetical long stock position, calculate maximum planned loss with entry 51.20, stop 50.70, and 200 shares. Explain its exclusions.\n/quit\n' \
  | cargo run -- custom_aa_llama_qwen3_6-35b.json > /tmp/l10-calculator.txt

# Empty reply is ignored; EOF after the welcome exits without another exchange.
printf '1\n\n' \
  | TERM=dumb cargo run -- custom_aa_llama_qwen3_6-35b.json > /tmp/l10-eof.txt
```

**Structural success:** selection loads the three files; the first input is application-owned; ordinary replies preserve the same Session/history/context; `/finish` followed by selection creates a new Session; commands/empty input do not become provider messages; streaming is consumed during the run; full saved state is available in plain diagnostics; limits and correlation remain intact. The numerical scenario should show the allowlisted request, matching response ID, exact `$100.00`, and an ordinary saved final tutor answer. Wrong selection or missing calculation is an observed scenario failure, not something to hide or force.

**Qualitative observations:** does the tutor ask about goals, define unfamiliar terms, address interruptions, stay educational, distinguish a stop plan from guaranteed execution, and accurately explain tool exclusions? Does it avoid personalized recommendations, invented sources, claims of mastery, or persistent memory? Did the guidance tweak produce the intended teaching pattern? Automatic structural checks do not grade these judgments or prove displayed and saved prose are identical.

If the provider is unavailable, ask the instructor to start or supply one. Deterministic tests are useful for boundaries, but are not live teaching or tool-selection evidence.

## Validation record — 2026-10-07

- `cargo fmt --check`, `cargo check`, `cargo clippy --all-targets`, and `cargo test` passed: **20 deterministic tests**. The reference source also passed standalone rustfmt checking and is byte-identical to root. Tests cover existing decimal/domain/allowlist/correlation/all-block/request/pass safeguards, state continuity and fresh selection, fixed selected prompt parts, command classification, full Debug fields/default metadata, diagnostic truncation/append/embedded fences/control removal, failing log creation/write, framed ordered presentation and fallible output, and concurrent capacity-one Event draining without saving by display. `/dev/full` covers write failure on the Linux target. Tests are not teaching-quality or live model evidence.
- The lesson's documented live conversation, calculator, and blank/EOF pipelines passed against the bundled local **qwen3.6-35b-a3b** provider. The conversation asked about experience/goals, answered order-type questions, retained follow-up history, and returned to a fresh welcome after `/finish`/reselection. Empty input caused no extra run; EOF exited. A chooser `q` exited without inference. Default manifest-relative paths were also exercised from `/tmp`.
- The stop-loss conceptual turn made **no tool request**. The numeric turn made **one** `maximum_planned_loss` request with decimal-string `51.20`/`50.70`, `200` shares, and one matching saved result ID, returning **`$100.00`**, followed by an ordinary answer explaining fees, slippage, gaps, and lack of a guaranteed realized-loss ceiling. This was observed again in the final root live run, not forced by application logic. These outcomes do not validate Lesson 9's separate live 100-share tweak.
- A colored pseudo-terminal run exercised down/up/Enter, raw-mode restoration, continuing input, and the live calculator. Bold cyan/blue/magenta/green presentation, default-color tutor fragments, framed result, and request-activity → result → final-answer order were captured. An empty-`NO_COLOR` pseudo-terminal used numbered selection and added no ANSI. Redirected/plain runs and `TERM=dumb` blank/EOF flow also had no application-added ANSI. Stderr-only terminal error and dumb-terminal probes verified independent stderr styling/gating. Human projector/contrast review is still pending.
- Private diagnostic captures retained complete Session fields and nested/default message/usage metadata, selected content, correlated requests/results, assembled saved tutor text, multiple selections, and no ANSI. Git ignore was verified. A temporary unavailable-provider configuration exited with status 1 and retained complete available partial state plus failure diagnostics; this was a maintenance probe, not a new classroom exercise. A temporarily unavailable log destination caused exit 1 before learner UI/provider calls. Files were restored. The application cannot record to a log whose storage is unavailable.
- The approved guidance tweak was loaded into a fresh Session and confirmed in diagnostics, then the default sentence was restored. **Its intended teaching ordering was not clearly validated:** the model placed the hypothetical scenario inside its check question, rather than unambiguously presenting a short example first. This is mixed qualitative evidence, not a guaranteed prompt effect. Default conversations sometimes also used examples. Do not treat the paired observation as a controlled evaluation.
- Teaching review remains necessary. Some observed responses used reassurance such as “safety net,” described adverse fills as “slightly” worse, or asked whether planning made the learner more comfortable with trade parameters. These were accompanied by risk caveats, but their framing, precision, pacing, and suitability need instructor review; structural success does not certify educational accuracy or readiness to trade. Domain files are course-authored with verified FINRA/SEC attribution, not released instructional evidence merely because sources were consulted.

**Remaining gates:** instructor review of Lesson 10 content, teaching behavior, UI, and classroom tweak. Lessons 8–9 still require their own reviews; Lesson 9's live 100-share tweak remains pending. No Lesson 11 persistence design or implementation is supplied.

## Next conceptual step

Lesson 11's agreed primary direction is **learner persistence**: an application-owned learning record and learner-agreed to-do items, rather than the previously planned evaluation-baseline lesson. Its detailed design remains deferred. That future feature must distinguish evidence of understanding from topics merely discussed, and purposeful learner records from this lesson's disposable diagnostics. This lesson does not implement profiles, learning records, to-do storage, or cross-launch memory.

## Supplemental official orientation

- [Goose context engineering](https://goose-docs.ai/docs/guides/context-engineering/index) — orientation for deciding which context an application supplies, not a requirement to reproduce Goose's assembly.
- [Configure LLM Provider](https://goose-docs.ai/docs/getting-started/providers) — provider setup orientation.

The exact pinned Rust source, not these general Goose guides, establishes the state-machine interfaces and behavior used here. See the instructor notes for the source evidence map.
