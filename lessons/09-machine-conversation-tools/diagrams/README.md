# Lesson 9 narrated planned-loss trace

Six progressive lecture frames connect the two user inputs to the machine's re-evaluation. **This is a narrated example, not an observed transcript or a guarantee of model behavior.** Inspect the actual saved content during the live exercise: report unnecessary first-turn requests, missing second-turn requests, invalid arguments, and other deviations honestly. Never remove the calculator on turn 1 or force a request to make the example come true.

A **Session snapshot** here is selected recorded data at a point in the story, not an exact serialized provider request or a complete runtime field dump. Earlier history is summarized in later frames for projector readability but is retained in the Session. Actual lesson output must show its complete saved fields. The example correlation ID `example-call-1` links the illustrated request and response; it is not a promised runtime ID or a message ID. Generated wording, additional content blocks, message counts, and usage metadata can differ.

An **Event** supports application presentation through the **Emitter**. An **Effect** proposes a recorded-state change; the application's **effect handler** applies it to the store. Seeing a fragment does not save a message. Each frame repeats separate display/save paths; these are responsibilities, not a promise about the exact timing or number of Events. The application saves user inputs before starting each run. Operations return Effects inside the run. Inference may also return usage Effects, omitted from these conceptual snapshots.

## Frames, captions, and text equivalents

### 1. First question

[Editable SVG](01-first-question.svg) · [Lecture PNG](01-first-question.png)

**Caption/text equivalent:** The application saves “What is stop-loss referring to in day trading?” in a Session and starts run 1. The calculator `maximum_planned_loss` is already available. The Tools Operation has no pending request, so inference asks the provider/model with history and the tool definition. Availability does not require execution.

**Reveal/predict:** Show the banner and snapshot first; point to the capability being available on **both** turns. Ask “Does availability mean the tool must run?” Then point to the pending-request check. Expected answer: no; the model may select a capability, while the application controls execution.

### 2. First answer without a tool

[Editable SVG](02-first-answer.svg) · [Lecture PNG](02-first-answer.png)

**Caption/text equivalent:** In this example the provider returns an ordinary conceptual assistant explanation, not a tool request. Inference reconstructs the answer and returns Effects; the effect handler saves it. The reloaded Session retains user question and assistant explanation. No pending request or unanswered input remains, so no step has work and run 1 returns to the application. The calculator was not removed.

**Reveal/predict:** Point to the two retained contributions, then contrast the display and save boxes. Ask “What changes when the next user input is saved?” Expected answer: the same Session grows; a new run has new work. A finished run does not wait for input by itself.

### 3. Append the numeric follow-up

[Editable SVG](03-follow-up.svg) · [Lecture PNG](03-follow-up.png)

**Caption/text equivalent:** The application appends a request for maximum planned loss for a hypothetical long position: entry `$51.20`, stop `$50.70`, `200` shares. It does not replace the earlier exchange or create a new Session. Run 2 begins. No request is pending yet; inference receives retained history and the same advertised calculator.

**Reveal/predict:** Trace the snapshot from top to bottom before revealing the right-hand narration. Ask “Which structured output would justify execution?” Expected answer: a request for the allowed tool with arguments; prose claiming a calculation happened is not execution evidence.

### 4. Save the assistant request

[Editable SVG](04-saved-request.svg) · [Lecture PNG](04-saved-request.png)

**Caption/text equivalent:** In the inference pass of run 2, the provider returns an assistant request named `maximum_planned_loss`, with correlation ID `example-call-1`, `entry_price: "51.20"`, `stop_price: "50.70"`, and `share_count: 200`. Inference reconstructs the message; Effects and the effect handler append it to the stored Session. Re-evaluation begins from the top with a request that has no matching response. Receiving/saving a request has not executed the calculator.

**Reveal/predict:** Point to name, quoted decimal-string prices, integer share count, and ID. Ask “Which Operation now has work, and why?” Expected answer: the Tools Operation, because a pending request has no correlated response. No Operation completion flag changed.

### 5. Validate, execute, and save the correlated response

[Editable SVG](05-saved-response.svg) · [Lecture PNG](05-saved-response.png)

**Caption/text equivalent:** The Tools Operation handles the pending request inside the application's tool execution boundary. Allowlisted dispatch and strict argument validation precede execution. Price strings are parsed immediately using exact decimal arithmetic; shape checks reject unknown fields, and domain checks require positive prices/shares, at most four decimal places, and entry above stop. The deterministic result is `(51.20 − 50.70) × 200 = 100.00`. Effects propose a user-role tool-response message; the effect handler saves it with the same `example-call-1` ID and `$100.00` result. That user-role message carries tool content, not another human question; its effective provider role is tool.

**Reveal/predict:** Cover the result while asking learners to calculate it. Reveal validation before arithmetic, then point between the two identical IDs. Ask “Is this response the final assistant answer?” Expected answer: no; it supplies evidence for another inference pass, within the same run. The actor is **application, as the tool → provider**, not a remotely executing model.

### 6. Final explanation, then no work

[Editable SVG](06-final-explanation.svg) · [Lecture PNG](06-final-explanation.png)

**Caption/text equivalent:** On reload the request has a correlated response, so the Tools Operation declines. Inference uses the retained conversation and result to produce an educational explanation of `$100.00` maximum planned loss, excluding fees, slippage, and gaps through the stop. Effects and the handler save that assistant answer. Re-evaluation now finds neither step has work; run 2 returns to the application. The first exchange, follow-up, request, response, and final explanation remain in the same Session. Planned loss is neither statistical expected loss nor a guaranteed ceiling on realized loss.

**Reveal/predict:** Point first to the matching IDs, then to the final assistant answer, then to the no-work exit. Ask “If only the share count changes to 100, what result should the tool return?” Expected answer: `$50.00`. Correlation and retention of the first exchange should still hold.

## Presentation and scope

- Show frames in numeric order. Each is a whole-frame reveal; covering a panel temporarily is optional, not an animation dependency.
- The GDK owns re-evaluation: load → check in order → first applicable step → apply Effects → reload. A new user input starts a **new run**; the request/result/explanation sequence contains multiple **passes within run 2**.
- “Tools Operation” and “inference” are conceptual names. The runtime can print SDK operation names such as `tools` and `llm`; `llm` is not a provider/model name.
- The snapshots omit unrelated metadata and summarize earlier entries deliberately; they do not claim a fixed number of saved messages for every model.
- The frames show the ordinary successful route only. Execution/request safeguards, unknown-name handling, and all-request handling belong to the complete implementation, not invented numeric bounds in the diagram. Validate those separately.
- The Session is retained by the in-memory application store across these two runs, not across process exit. These frames do not imply production disk persistence, Goose's full assembly, market data, or trade execution.
- Solid boxes indicate recorded-state paths; dashed boxes indicate presentation paths. Labels carry meaning without reliance on color. Arrows indicate flow/responsibility, not precise temporal order.

## Editing and exporting

The SVGs are editable source of truth: plain XML text, rectangles, and accessible title/description elements. No diagram-language plugin or generator is needed. PNGs are 1600×900, exported with librsvg (`rsvg-convert`) and DejaVu Sans. Use an ordinary browser/image viewer or slide deck.

From this directory, export after editing:

```bash
for source in *.svg; do
    rsvg-convert "$source" -o "${source%.svg}.png"
done
```

Keep each SVG title/description, PNG, and caption synchronized. Check rendering at full size and classroom distance. Asset verification is not live-provider validation; no actual model run or classroom/projector approval is claimed by these illustrations.
