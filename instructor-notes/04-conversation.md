# Lesson 4 Instructor Notes: System Instructions and Conversation History

## Teaching objective

Make the transition from a single inference exchange to an application-owned discussion explicit. Define turn, conversation history, and system instruction before using them. Learners should understand that a system instruction and ordered history are inputs to each provider call, the provider object does not retain this program's history, and streamed assistant text must be reconstructed before the application creates a later assistant turn.

## Suggested pacing

- Trace how application-owned turns become ordered conversation history: 10 minutes
- Contrast system instructions and user messages: 10 minutes
- Refactor Lesson 3 into a text-returning helper: 15 minutes
- Add assistant and follow-up user turns: 15 minutes
- Run both requests and inspect ordered history: 10 minutes
- Review and questions: 5 minutes

## Before class

- Validate Lesson 3 against the planned provider.
- Run the Lesson 4 reference and confirm both calls complete promptly.
- Use short prompts and concise instructions for local inference.
- Confirm the provider preserves ordinary user/assistant history.

## Discussion prompts

- Why does the application—not the provider object—own the discussion history?
- What is a turn, and why must its ordering be preserved?
- Which state belongs to the application, and which component generates the next response?
- What makes an ordered list of messages history rather than a single prompt?
- Why is the system instruction not a user or assistant turn?
- What behavior belongs in a system instruction rather than a user message?
- Why does the second call include the original user question and assistant reply?
- What information is lost if only the follow-up is sent?
- Why should the helper both print and return text?
- Why validate message ordering rather than exact words?

## Live-demo cautions

- Models may ignore requested length limits; treat them as prompt effects, not deterministic assertions.
- The second call re-sends text and therefore consumes more input tokens.
- The provider object does not hold application conversation history. Do not imply that the model or provider object remembers earlier calls merely because the second response refers to the first.
- This lesson intentionally retains only plain text. Lesson 5 introduces structured assistant output and the need to reconstruct complete provider messages.
- Keep tool roles conceptual until the tool-result lesson.
- Avoid introducing an interactive input loop here; two fixed turns keep message ordering visible.

## Checkpoint

Learners distinguish system instructions from history, reconstruct the first response, append it as an assistant turn, and make a context-dependent follow-up call.

## Transition to Lesson 5

The next lesson does not add model-selection CLI plumbing. It changes to the day-trading teaching-assistant domain and introduces a deterministic maximum-planned-loss tool at the raw provider boundary. Use entry `51.20`, stop `50.70`, and 200 shares; the exact result is `100.00` before fees, slippage, or a gap through the stop.

Keep Lesson 5 deliberately narrow: define and advertise the tool, inspect all returned content blocks, recognize a structured request, and preserve its ID. Do not execute it yet. That pause lets learners see that the model proposes a capability call while the application remains responsible for authorization and dispatch. Lesson 6 performs validation, execution, the correlated user-role response, and follow-up inference.
