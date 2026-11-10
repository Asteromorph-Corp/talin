# Transcript Editing

The [transcript](/docs/agents_api.md#sessions) is the conversation state an agent exchanges with the LLM.
Editing a transcript directly grants you full control over the agent's memory, reasoning, and behavior.

Every edit ultimately goes through the same view → edit → modify cycle described below.
Writing that cycle out at each site is verbose, so an edit scheme packages it into a reusable value.
This document covers both the raw cycle and the schemes built on top of it.

## The `AgentTranscript`

`session.transcript` is an `AgentTranscript`, which exposes:

- `entries() -> &[Entry<B>]`

  Returns all stored records and markers, including typed native content.

- `full_view() -> Vec<EntryView>`

  Returns a read-only, ordered view of every entry in the transcript.

- `view(scope: &Scope) -> Vec<EntryView>`

  Returns a read-only, ordered view of the entries within `scope`.

- `modify(edits: Vec<EntryEdit>, scope: &Scope) -> Result<(), ModifyError>`

  Replaces every entry within `scope` with the sequence described by `edits` (see below).

A `Scope` picks the window these methods operate on. See [Scopes](#scopes) below.

### Model records

Each completed model response is stored as one record: its final canonical answer or tool-call
batch is paired with the whole provider-native response. `LlmStep` is the original enum with
`Finish` and `ToolCall` variants. Its optional canonical `thought` joins readable reasoning strings
without a separator; native blocks retain their original text, boundaries, and metadata.
All client calls in that response belong to its final `ToolCallMessage`; completing it executes
the calls and records their results. A final `Finish` supplies the answer. Errors use `LlmError`.

`full_view()` and `view()` expose final canonical messages, native-entry UUIDs, and transcript markers. Use
`entries()` to inspect each record's `canonical()` and typed `native()` content, including
reasoning signatures, compaction blocks, and server tools. Formatting the transcript with
`{:#?}` also shows the native records and UUIDs. `RecordStrategy` captures the canonical messages.

Truncated text and its continuation share a completed record. Structured providers preserve their
text parts and native blocks in order; text-based instructors parse the combined raw completion.
`modify_step` edits the final canonical message of the latest model record. Unchanged final
messages retain their native content. Rebuilding or migrating a coarse record can lose native details that
its final canonical message does not represent.

### Compaction

Every compaction is preceded by a `CompactionMarker { boundary }`. Markers are advisory: native
compaction items decide what requests omit, except after a boundary marker, which is used for
summaries that are not a native compaction. The entry after the marker holds the compaction
either inside the native content of an ordinary model record (automatic compaction) or as a
native-only `Entry::Native` (on-demand compaction). Original entries are never removed. See
[Compaction](/docs/advanced/compaction.md) for how each case is stored and rendered.

### The view → edit → modify cycle

`modify` replaces the entries within `scope` using `edits` as the complete, ordered specification of the new entries.
Entries outside `scope` are left untouched.

The workflow is therefore:

1. `view` the scope to get the current entries, each tagged with a stable `uuid`.
2. Map each `EntryView` to an `EntryEdit`, dropping, replacing, inserting, or reordering as you go.
   `EntryEdit::from(view)` turns an entry into the edit that re-emits it unchanged.
3. `modify` the same scope with the resulting `Vec<EntryEdit>`.

The `EntryEdit` variants are:

| Variant | Effect |
| --- | --- |
| `Keep(uuid)` | Re-emit the existing entry with that `uuid`, unchanged. |
| `Replace(uuid, message, retain)` | Replace the entry at `uuid` with `message`. If `retain` is `true`, the original provider-specific item is reused as a base. Ignored for tool-error targets. |
| `Insert(message)` | Insert `message` as a brand-new entry with a fresh `uuid`. |
| `StepMarker` | Place a step marker at this position. |
| `TurnMarker` | Place a turn marker at this position. |
| `CompactionMarker { boundary }` | Record a compaction; with `boundary: true`, requests start after it. |

`Keep` and `Replace` reference entries by the `uuid` you read from the `view`.

This resembles a `git rebase --interactive`: `edits` is a todo list over the entries in `scope`, `Keep` is `pick`, `Replace` is `edit`, and an entry you leave out of the list is `drop`ped.

Since the scope is fully replaced, anything you don't include in `edits` is removed.
The order of the entries in the new scope also matches the order of `edits` exactly.
You can reorder the entries by changing the order of the `EntryEdit`s, or drop them by omitting them from `edits`.

Note that even the step and turn markers are part of the scope, and omitting them from the `edits` drops them from the transcript.
This affects subsequent scopes anchored on those markers (e.g. `Scope::from(Last::step())`), so be sure to re-emit them if you want to preserve the boundaries.

Below is an example that drops all tool errors from the transcript:

```rust
let scope = Scope::full();
let view = session.transcript.view(&scope);
let edits = view
    .into_iter()
    .filter_map(|entry| match entry {
        EntryView::ToolError(_, _) => None,
        other => Some(EntryEdit::from(other)),
    })
    .collect::<Vec<_>>();
session.transcript.modify(edits, &scope)?;
```

## Scopes

A `Scope` selects a range of entries by two anchors: `start` (inclusive) and `end` (exclusive).

| Constructor | Range |
| --- | --- |
| `Scope::full()` | The whole transcript. |
| `Scope::to(end)` | From the start of the transcript to `end`. |
| `Scope::from(start)` | From `start` to the end of the transcript. |
| `Scope::new(start, end)` | From `start` to `end`. |

Anchors are clamped to the bounds of the transcript, and if `start` resolves past `end`, `end` is set to `start`.

An `Anchor` names a position in the transcript:

| Anchor | Position |
| --- | --- |
| `Start` / `End` | The first entry / one past the last. |
| `First::step()` / `Last::step()` | The first / last step marker. |
| `First::turn()` / `Last::turn()` | The first / last turn marker. |
| `First::matching(pred)` / `Last::matching(pred)` | The first / last entry satisfying the predicate. |
| `Fn(&[EntryView<'_>]) -> usize` | A position computed from the views. |

`First` and `Last` can be refined with `.skip(n)` (step `n` further matches) and `.from(anchor)` (restrict the scan to entries after / before another anchor).

```rust
// Scope everything since the last step marker.
let scope = Scope::from(Last::step());

// Scope everything up to, but not including, the second-to-last step.
let scope = Scope::to(Last::step().skip(1));
```

## Edit schemes

Writing the entire view → edit → modify cycle inline can be verbose and difficult to comprehend.
An edit scheme packages the cycle into a reusable value, so you can apply it to any transcript with a single method call.
This is useful whether you are editing the transcript directly, or from one of the [strategy hooks](/docs/advanced/strategy.md#transcript-editing).

A scheme implements one or both of the following traits:

- `EntryEditScheme` rewrites the entries.
- `SystemEditScheme` rewrites the system prompt contents.

and it is applied to a transcript with:

| Method | Applies |
| --- | --- |
| `apply_entry_edit(&scheme)` | An `EntryEditScheme`, over the scheme's own `scope`. |
| `apply_system_edit(&scheme)` | A `SystemEditScheme`. |
| `apply_edits(&scheme)` | A scheme implementing both, system pass first. |

## Built-in schemes

### `RemoveToolCalls`

Removes tool calls together with each call's paired result or error.

| Constructor | Selects |
| --- | --- |
| `RemoveToolCalls::matching(filter)` | Removes every tool call that matches the filter. |
| `RemoveToolCalls::errored()` | Removes every tool call that resulted in an error. |

You can also implement a custom selector and use it with `RemoveToolCalls(selector)`.
The selector must implement `ToolCallSelector`, which inspects the transcript and returns the list of tool call ids to remove.

Callling `.except_last()` on any selector spares the final match. 
This lets you keep only the most recent result of a repeatedly-called tool:

```rust
// Keep only the latest `check_status` call and its result; drop the earlier rounds.
let scheme = RemoveToolCalls::matching("check_status").except_last();
session.transcript.apply_entry_edit(&scheme).await?;
```

### `Interpolate`

Renders every `{{key}}` placeholder in the transcript against a `PlaceholderResolver`.
Unknown keys are left intact, and each entry's provider-specific base is preserved.

A resolver is anything that maps a key to its current value: a `HashMap<String, String>`, or a `Fn(&str) -> Result<Option<String>, E>` that reads live state.

```rust
let values = HashMap::from([("job_status".to_string(), "complete".to_string())]);
session.transcript.apply_edits(&Interpolate(&values)).await?;
```

> ⚠️ `Interpolate` is meant to be applied **ephemerally**, as a projection.
> Applying it permanently substitutes the value into the stored transcript, so the placeholder is gone and cannot be re-rendered against a newer value.

When implementing a custom `EntryEditScheme`, override `scope()` to restrict which entries `edit` sees; entries outside it pass through unchanged. It defaults to `Scope::full()`.

```rust
// Never touch the last two steps.
impl EntryEditScheme for Summarize {
    fn scope(&self) -> Scope {
        Scope::to(Last::step().skip(1))
    }

    // ...
}
```

## Applying a scheme during a run

You can also convert an edit scheme into a strategy.
The following strategies applies the provided edit scheme from one of the [strategy hooks](/docs/advanced/strategy.md#transcript-editing):

| Scheme | Permanent | Ephemeral |
| --- | --- | --- |
| `EntryEditScheme` | `ApplyEntryEditStrategy` | `ApplyEntryProjectionStrategy` |
| `SystemEditScheme` | `ApplySystemEditStrategy` | `ApplySystemProjectionStrategy` |
| Both | `ApplyEditsStrategy` | `ApplyProjectionsStrategy` |

`*EditStrategy` applies the scheme permanently (via `edit_transcript`), while `*ProjectionStrategy` applies it ephemerally to a throwaway clone (via `project_transcript`).

```rust
// Re-render the placeholders against current values before every request.
let strategy = ApplyProjectionsStrategy::new(Interpolate(&values));
let output = agent.run_with_strategy(strategy, input).await?;
```
