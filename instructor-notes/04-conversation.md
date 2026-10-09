# Lesson 4 Instructor Notes: System Instructions and Conversation History

## Teaching objective

Make the transition from a single inference exchange to an application-owned discussion explicit. Define turn, conversation history, and system instruction before using them. Learners should understand that a system instruction and ordered history are inputs to each provider call, the provider object does not retain this program's history, and model-generated assistant-role text must be reconstructed before the application creates a later assistant-role turn.

## Suggested pacing

- Trace how application-owned turns become ordered conversation history: 10 minutes
- Contrast system instructions and user messages: 10 minutes
- Refactor Lesson 3 into a text-returning helper: 15 minutes
- Add assistant-role and follow-up user turns: 15 minutes
- Run both requests and inspect ordered history: 10 minutes
- Review and questions: 5 minutes

## Before class

- Validate Lesson 3 against the planned provider.
- Run the Lesson 4 reference and confirm both calls complete promptly.
- Use short prompts and concise instructions for local inference.
- Confirm the provider preserves ordinary user/assistant-role history.

## Discussion prompts

- Why does the application—not the provider object—own the discussion history?
- What is a turn, and why must its ordering be preserved?
- Which state belongs to the application, and which component generates the next response?
- What makes an ordered list of messages history rather than a single prompt?
- Why is the system instruction not a user or assistant-role turn?
- What behavior belongs in a system instruction rather than a user message?
- Why does the second call include the original user question and assistant-role reply?
- What information is lost if only the follow-up is sent?
- Why should the helper both print and return text?
- Why validate message ordering rather than exact words?

## Live-demo cautions

- Models may ignore requested length limits; treat them as prompt effects, not deterministic assertions.
- The second call re-sends text and therefore consumes more input tokens.
- The provider object does not hold application conversation history. Do not imply that the model or provider object remembers earlier calls merely because the second response refers to the first.
- This lesson intentionally retains only plain text. Lesson 5 introduces structured model output and the need to reconstruct complete provider messages.
- Keep tool roles conceptual until the tool-result lesson.
- Avoid introducing an interactive input loop here; two fixed turns keep message ordering visible.

## Prompt caching (backend detail)

The lesson mentions prompt caching only in passing: some servers cache an identical system prefix, the application cannot rely on it, and the full instruction is re-sent every call. This section holds the backend-specific detail for questions, verified against llama.cpp's `tools/server/README.md` on 2026-09-30; other backends differ.

**llama.cpp's `server`** (the bundled default at `127.0.0.1:8080`) reuses a shared KV prefix across requests, but the knobs are easy to get wrong:

- **`--cache-prompt` is a boolean toggle, not a value-taking flag.** Write just `--cache-prompt` to enable, or `--no-cache-prompt` to disable — never `--cache-prompt true`. It is enabled by default, so the common case needs no flag at all.
- **The `true`/`false` value is a per-request body field**, `cache_prompt`, which overrides the server default for one request.
- **`--cache-reuse N`** is a separate knob: the minimum chunk size (in tokens) at which it reuses cached KV via "KV shifting"; it requires prompt caching enabled and defaults to `0`.
- **Caveats:** reuse is per-slot (a finished slot keeps its KV alive and matches the incoming prefix against it), so a follow-up turn must land on a slot whose cache shares the prefix — prefix-affinity selection helps but does not guarantee it; sliding-window-attention models can break reuse (servers log "forcing full prompt re-processing"); and the reused prefix is bounded by the slot's cache size, so a long conversation eventually evicts it.

This is a server-side optimization layered on top of a correct, simple contract: the application always re-sends the full instruction.

## Terminal presentation guidance

- The two response headings are bold cyan; streamed model text and usage metadata stay normal, with no dimming.
- The supplied `main` wrapper prints fatal errors once in bold red on stderr and exits 1; history and streaming behavior are unchanged.
- stdout and stderr are gated independently by `IsTerminal`; redirected output, a present `NO_COLOR` (including empty), or `TERM=dumb` disables styling. Plain output has the same strings and layout.
- Treat the local presentation helpers and error wrapper as supplied support, not a Rust fill-in exercise. There are no new printed user-input sites or payload fences; color is not protocol data or a validation criterion.

## Checkpoint

Learners distinguish system instructions from history, reconstruct the first response, append it as an assistant-role turn, and make a context-dependent follow-up call.

## Transition to Lesson 5

The next lesson does not add model-selection CLI plumbing. It changes to the day-trading teaching-assistant domain and introduces a deterministic maximum-planned-loss tool at the raw provider boundary. Use entry `51.20`, stop `50.70`, and 200 shares; the exact result is `100.00` before fees, slippage, or a gap through the stop.

Keep Lesson 5 deliberately narrow: define and advertise the tool, inspect all returned content blocks, recognize a structured request, and preserve its ID. Do not execute it yet. That pause lets learners see that the model proposes a capability call while the application remains responsible for authorization and dispatch. Lesson 6 performs validation, execution, the correlated user-role response, and follow-up inference.


## Terminal styling validation — 2026-10-06

- Using this lesson's complete source with the shared pinned manifest in temporary staging, `cargo fmt --check`, `cargo check`, `cargo clippy --all-targets`, and `cargo test` passed (2 tests). Root source and dependencies were not replaced.
- Successful terminal-colored, redirected/plain, and empty-`NO_COLOR` runs passed. `TERM=dumb` and independent stdout/stderr gating were also verified with error-output probes; errors retained exit status 1 and terminal stderr was bold red.
- Both live inference calls returned text and completion usage; the follow-up used the retained first exchange.
