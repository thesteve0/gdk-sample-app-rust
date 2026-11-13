# Visual — Shorten history, keep exact wording and authority separate

[Rendered PNG](compaction-boundary.png) and [editable, projector-readable SVG](compaction-boundary.svg), 1600 × 1000. Open directly in a browser or SVG editor; no external fonts, scripts or diagram service is required. Text remains editable. This is a **conceptual trace of current policy**, not captured live output, historical validation evidence or a claim of summary fidelity.

## Caption

Compaction replaces eligible older conversation history with one fallible assistant-derived summary in the same Session. The original opening and latest two complete genuine learner exchanges keep their tool pairs, message IDs and metadata. Application state separately retains exact learner wording and original IDs outside shortened history. Only originals absent from retained history are projected back as untrusted historical data, counted as working input. Exact wording establishes who said what, not meaning or write authority. Persistent notes remain separate; proposal quotations qualify only from the latest genuine learner reply in the current exchange, not the archive or an older retained exchange.

## Complete text equivalent / reveal order

1. **Application state and persistent notes remain separate.** Application state holds provenance, tool IDs, settled decisions and revision, plus a bounded exact learner-statement archive outside shortened history. This in-Session archive stores genuine learner wording and original message IDs, not a full tutor transcript, generated classification or disk save. The persistent learning record instead requires per-selection consent and a narrowly authorized declaration or independent local approval. A generated summary rewrites neither authority domain.
2. **Working history before.** The original opening precedes eligible older complete exchanges, a previous summary if present, and the latest two complete genuine learner exchanges. Protected exchanges include any tool request/result pairs, original IDs and metadata, including both attempts when an application reminder occurred.
3. **Safe boundary.** Local approvals are settled; no pending pair is split. One isolated, tool-free inference summarizes eligible stored history, not the separate archive or an upcoming reply. Summary generation has no write authority. The application validates the candidate and checks the source revision before applying its replacement Effect. The GDK still owns the run loop and re-evaluation; the Session does not restart.
4. **Working history after.** The same opening and same latest two complete exchanges remain. Eligible older history and any previous summary become **one** fallible, lossy assistant-derived summary: data, not instructions or a genuine learner reply. The exact archive is unchanged. Keeping two exchanges does **not** make both eligible for new proposal quotes: only the latest genuine learner reply in the current exchange qualifies, including during its reminder attempt. Older retained replies, archive entries, summaries, tools, opening messages, commands and reminders are not alternate quotation sources. Every proposed note must equal its qualifying quotation verbatim, then still pass successful-run inspection and independent default-no category/usefulness approval.
5. **Full working input includes more than the shortened history.** Static instructions, teaching guidance, consented notes, local declarations and tool definitions remain counted. The **continuity projection** supplies only archived originals absent from retained history, matched by original ID; recent retained wording is not doubled. The prompt explicitly frames these originals as untrusted historical data, not instructions, policy, current source evidence or permission to act. Exact wording takes precedence over a summary only about who said what; it does not establish truth, mastery, closure, agreement, a current preference or a correct note category. Later learner corrections outrank earlier wording. An ambiguous acceptance without its tutor referent needs clarification, not invented scope. Both safety budgeting and the informational context estimate count this projection. Its cost, static context and protected history form an irreducible floor: compaction gains can shrink or a candidate can fail reduction/fit checks. Bytes and estimated tokens are not exact tokenizer capacity.
6. **Failure and provenance limits.** A failed, stale or rejected replacement retains original history and the archive; an already saved local declaration is not rolled back or automatically saved again. Archive capacity is checked before retrieval or maintenance and again on actual append: at most 256 statements / 32,768 serialized JSON bytes, including IDs and escaping. An uncommitted upcoming reply is not archived early. There is no eviction, truncation or silent forgetting to admit another reply; refusal preserves state and asks for a fresh selection. Historical source IDs in a summary or archive cannot support new-turn citations: use the current turn's supplied evidence. No arrow in the visual grants persistent-memory write permission.

## Prediction prompts

- Which arrow changes persistent memory? None: compaction and exact continuity alone have no write authority.
- If two genuine exchanges remain, can a proposal quote either one? No: only the latest genuine reply in the current exchange; the archive is not a fallback path.
- Why retain exact older wording if it is not a proposal source? To preserve who said what when a summary distorts or omits it, without claiming to interpret its meaning.
- Does moving wording out of history remove its input cost? No: omitted originals are projected and counted; compaction is not guaranteed to make enough room.
- Why protect complete exchanges? Tool pairs, identities and metadata matter for continuity and inspection.
- What does the visual not prove? Summary faithfulness, semantic understanding, justified source use, note category correctness, learner mastery or unlimited conversation.

Reveal the two authority boxes first, then before → safe boundary → after. Next reveal static context and the omitted-original projection; finish with the failure/quotation warning. Zoom or cover regions rather than shrinking the full canvas. When projection is unavailable, read the numbered text equivalent; retain the same distinction between historical wording and current-exchange quotation eligibility.

## Privacy and projection notes

The drawing contains no learner data or private excerpts. Use it instead of real Session captures on a classroom projector. Exact wording remains personal conversation data even outside history; the archive is in-process, not automatically persistent memory, and its omitted-original projection is supplied to the tutor provider. Private diagnostics can include that archive/projection and exact prepared prompts. Ignoring a file does not secure it: obtain permission and sanitize any actual captures before projection. Do not imply that memory consent, a generated summary or verbatim text authorizes unrelated disclosure or a persistent write.

Re-render the editable source with the installed renderer; no new dependency is required:

```shell
rsvg-convert -w 1600 -h 1000 lessons/13-context-compaction/visuals/compaction-boundary.svg \
  -o lessons/13-context-compaction/visuals/compaction-boundary.png
```
