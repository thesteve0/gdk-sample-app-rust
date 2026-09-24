# Lesson 4 Instructor Notes: System Instructions and Conversation History

## Teaching objective

Make the request boundary explicit. Learners should understand that a system instruction and ordered history are inputs to each provider call, and streamed assistant text must be reconstructed before it becomes a later assistant turn.

## Suggested pacing

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

- What behavior belongs in a system instruction rather than a user message?
- Why does the second call include the original user question and assistant reply?
- What information is lost if only the follow-up is sent?
- Why should the helper both print and return text?
- Why validate message ordering rather than exact words?

## Live-demo cautions

- Models may ignore requested length limits; treat them as prompt effects, not deterministic assertions.
- The second call re-sends text and therefore consumes more input tokens.
- The provider object does not hold application conversation history.
- Keep tool roles conceptual until the tool-result lesson.
- Avoid introducing an interactive input loop here; two fixed turns keep message ordering visible.

## Checkpoint

Learners distinguish system instructions from history, reconstruct the first response, append it as an assistant turn, and make a context-dependent follow-up call.
