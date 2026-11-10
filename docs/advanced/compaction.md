# Compaction

Compaction replaces an older part of the conversation with a shorter summary, so later requests
send less history. The library never deletes stored history to do this: every original entry and
UUID stays in the [transcript](/docs/advanced/transcript.md). Compaction only changes which part
of the stored history is sent.

There are two kinds:

- **Automatic** compaction is performed by the provider inside an ordinary generation request
  once the input crosses a threshold. You enable it with a [`Strategy`](/docs/advanced/strategy.md).
- **On-demand** compaction is performed by the library before a step's request is built. A
  `CompactionStrategy` on the session selects a prefix, summarizes it, and stores the summary.

## Where compaction items live

Every compaction is announced by an `Entry::CompactionMarker { boundary }`
(`EntryView::CompactionMarker` in canonical views), so canonical code can always tell that a
compaction happened. Markers are advisory: native compaction items, not markers, decide what a
request omits. The one exception is a summary that is not a native compaction, such as the reply
stored by `InstructionCompactor`; nothing native can omit history for it, so its marker has
`boundary: true`. The entry directly after the marker holds the compaction itself, in one of two
forms. Code that inspects or edits the transcript must handle both.

### 1. Inside the native content of an ordinary canonical message (automatic)

A response that compacted is still a normal model response. The runner stores it as one
`Entry::Message`, like any other step, and places an advisory marker before it:

```text
Entry::StepMarker
Entry::CompactionMarker { boundary: false }
Entry::Message(uuid, record)
  record.canonical() -> BodyMessage::Assistant / ToolCall   (the step's answer or tool calls)
  record.native()    -> Claude:    { role: assistant, content: [compaction, thinking, text] }
                        Responses: [compaction, reasoning, message]
```

The canonical message is innocuous: it holds only the answer or the tool calls and says nothing
about compaction. The compaction block or item exists only in `record.native()`, and the native
record may also carry content generated before the compaction, such as earlier thinking, which is
not sent again. `LlmRequestItem::contains_native_compaction` reports whether a native item holds a
compaction; the runner adds the marker when it does. A Claude compaction whose summary is `null`
(a failed compaction) gets no marker.

### 2. As a separate native-only entry (on-demand)

On-demand compaction inserts the marker and a native-only entry immediately after the last
summarized entry:

```text
... original entries (kept, no longer sent) ...
Entry::CompactionMarker { boundary }
Entry::Native(uuid, summary)        (no canonical message)
... retained entries (sent verbatim) ...
```

`summary` is whatever the compactor returned, in the provider's native form. The marker is a
boundary exactly when `summary` is not a native compaction
(`!summary.contains_native_compaction()`):

| Compactor                 | `summary`                                                       | `boundary` |
| ------------------------- | --------------------------------------------------------------- | ---------- |
| `ClaudeMessagesCompactor` | an assistant message holding one compaction block               | `false`    |
| `OpenAIResponsesCompactor`| the `/responses/compact` output: user messages, then a compaction item | `false` |
| `InstructionCompactor`    | the model's ordinary reply to the summarization instructions    | `true`     |
| your own `Compactor`      | the `L::Item` it returns                                        | depends    |

`EntryView::Native(uuid)` stands for this entry in canonical views; `entries()` exposes its
typed native content. Because the marker is inserted right after the summarized prefix, a summary
created later can appear earlier in the entry sequence than entries recorded before it.

## How requests are rendered

1. `Llm::materialize` starts after the latest marker with `boundary: true`, if any, and keeps
   the system prompt. Advisory markers are ignored, so without a boundary marker the whole
   stored history is lowered.
2. Claude Messages and OpenAI Responses then omit everything before the latest native
   compaction in the lowered items (`discard_compacted` in their `build_request`):
   - Claude keeps the latest compaction block with a non-empty `content` or `encrypted_content`,
     and everything after it. Earlier blocks of the same message and earlier messages are dropped.
     A compaction block whose summary is `null` is not a boundary.
   - Responses keeps the latest record that contains a compaction item, whole, and every later
     record.

The native rule covers both kinds of compaction: an on-demand Claude or Responses summary is
itself a native compaction, and an automatic compaction in a later response moves the boundary
past an earlier on-demand summary. Selectors use the latest marker of either kind as the start of
the active history when choosing what to summarize next.

## Consequences for transcript code

- **Canonical messages do not describe compaction.** The marker says that a compaction happened,
  but the compacting record's canonical message is an ordinary answer, and an
  `EntryView::Native(uuid)` can stand for a whole summarized prefix.
- **Edits before the boundary do not reach requests.** Editing an original message that was
  summarized changes only stored history; the summary is what is sent.
- **On-demand summaries can duplicate messages.** `OpenAIResponsesCompactor` stores the user
  messages that `/responses/compact` returns, so each of them is stored twice: once as the
  original entry and once inside the summary. Requests contain only the summary's copy. Each
  later compaction stores another copy.
- **Editing a record keeps its compaction.** `modify_step` and `EntryEdit::Replace` rebuild the
  native content from the new canonical message on top of the old native content; compaction
  blocks and items are retained.
- **Migration.** `migrate` (and `continue_session_migrated`) rebuild native content from the
  canonical messages. A session compacted automatically migrates without error but loses the
  compaction, so the new provider receives the full history; `migrate` drops the markers of
  those records, which no longer hold a compaction. A session with an on-demand summary fails to
  migrate, because the native-only entry has no canonical message.

## Automatic compaction

`AutoCompactStrategy::new(threshold)` enables automatic compaction for `ClaudeMessages` or
`OpenAIResponses`. `ClaudeMessagesAutoCompactStrategy::new(threshold, instructions)` adds optional
summary instructions; `OpenAIResponsesAutoCompactStrategy::new(threshold)` is the Responses
variant. They are per-turn strategies and only configure the request.

The Claude strategy sets `pause_after_compaction = false`, so generation continues in the same
response. A paused response would hold only the compaction block and end the step with an empty
answer. Claude needs the `compact-2026-01-12` beta header on the client.

## On-demand compaction

Configure a policy on the session:

```rust,ignore
let mut session = agent.session().with_compaction(
    ClaudeMessagesCompactStrategy::new(24000, 8000),
);
```

The policy persists across turns and same-provider session continuation. Migrating to a different
provider clears it. Set `session.compaction = None` to disable it.

`ClaudeMessagesCompactStrategy` and `OpenAIResponsesCompactStrategy` pair the provider's native
compactor with `TokenBudget`. `CustomCompactStrategy::new(compactor, selector)` combines any
compactor with any selector.

Selectors choose the last entry to summarize. A cutoff never splits a client tool exchange.

- `TokenBudget { threshold, retain_tokens }` triggers when the active history reaches
  `threshold` input tokens and keeps a recent suffix within `retain_tokens`, including the
  system prompt and tools. It needs a token-count endpoint (`CompactionModel::count_tokens`),
  available for Claude Messages and OpenAI Responses, and makes one count request for the
  trigger plus one per candidate cutoff.
- `MessageCount { threshold, retain_messages }` triggers on the number of active messages and
  keeps at least `retain_messages`.
- `CompactionCutoff(uuid)` summarizes through an explicit entry.
- Implement `CompactionSelector<L>` for another policy.

Compactors produce the summary from the lowered prefix and the session's `RequestParts`:

- `ClaudeMessagesCompactor` sends the request with the on-demand `compaction` field (beta
  `compact-2026-09-04`) and keeps the last compaction block that carries a summary.
- `OpenAIResponsesCompactor` calls `/responses/compact` and keeps its output, minus the system
  prompt it echoes back.
- `InstructionCompactor::new(instructions)` works with every LLM: it appends the instructions as
  a user message, forbids tool calls, and keeps the reply. It uses the session's tools with
  default call options, so a structured response format does not shape the summary. It fails on a
  truncated, empty, or tool-calling reply.
- Implement `Compactor<L>` to build your own summarization request with `parts.build(llm,
  context)`. Return `None` to leave history unchanged; return summary context without unresolved
  client tool calls.

### When it runs

Compaction runs before a step's request is built, at most once per step. The selected prefix is
lowered and the summarization request is built from it; strategy projections and request hooks do
not apply to that request. The step's request is then built once from the updated transcript.
Compaction is skipped during truncated-output continuation, and a generation retry keeps a
committed compaction. Compaction usage reaches `inspect_usage` before the step's request is built,
including Claude's compaction iterations. A compactor error fails the step like any model error.

## Endpoint support

Live results for every strategy and endpoint, with traces, are in the
[compaction compatibility matrix](/examples/automatic-compaction/COMPATIBILITY.md). In short:

- Anthropic Messages and OpenAI Responses support every strategy.
- `InstructionCompactor` with `MessageCount` or `CompactionCutoff` worked on every endpoint
  tested, including Chat Completions, OpenRouter, and xAI.
- OpenRouter forwards automatic compaction only on its Messages endpoint and has no token
  counting or native on-demand compaction.
- xAI has no automatic compaction or token counting; its `/responses/compact` works through
  `OpenAIResponsesCompactor` but drops facts that appear only in tool output.

The [automatic-compaction example](/examples/automatic-compaction/README.md) runs these policies
with generated agents, a real tool, and recorded API captures.
