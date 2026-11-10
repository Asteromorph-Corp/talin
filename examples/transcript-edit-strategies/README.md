# `transcript-edit-strategies`

This example shows how to edit the agent transcript with a strategy.

```sh
OPENAI_API_KEY=... cargo run
```

The agent polls a deployment with a `check_deploy` tool until it reports 100%.

For the polling tool, only the last result matters.
However, the transcript keeps all calls and results and wastes the context window with old information.
The example drops the older polls from the transcript by applying an edit scheme systemically with a strategy.

## Edit Scheme

An [edit scheme](/docs/advanced/transcript.md#edit-schemes) is a reusable change to a transcript.
`RemoveToolCalls` removes tool calls together with their results:

```rust
let scheme = RemoveToolCalls::matching("check_deploy").except_last();
```

This keeps only the most recent `check_deploy` call.

## Permanent / Ephemeral Edit

You can convert an edit scheme into a strategy which would apply the scheme before each step.
You can either apply the edits permanently, or ephemerally to a copy of the transcript that is discarded after the request is built.

| Strategy | Applied | The stored transcript |
| --- | --- | --- |
| `ApplyEntryEditStrategy::new(scheme)` | Before each step, to the transcript itself. | Updated. |
| `ApplyEntryProjectionStrategy::new(scheme)` | Before each step, to a copy that is discarded after the request is built. | Unaffected. |

In this example, a permanent edit is used to remove the older polls, because they are not needed for the next step.

An ephemeral edit would be more useful in scenarios such as replacing a placeholder with a value that is constantly updated.
Replacing the value permanently in the stored transcript would make it impossible to update the value in the future, while an ephemeral edit would keep the placeholder and allow it to be replaced with the current value for each request.

## Other schemes

| Scheme | What it does |
| --- | --- |
| `RemoveToolCalls::errored()` | Removes tool calls that failed, with their errors. |
| `Interpolate(&values)` | Replaces `{{key}}` placeholders in the transcript with current values. It is meant to be used as a projection, because a permanent edit would replace the placeholder with the value for good. |

`Interpolate` also rewrites the system prompt, so `ApplyProjectionsStrategy::new(Interpolate(&values))` renders the placeholders in the system prompt and the entries together.

You can also implement a `EntryEditScheme` for your own edits.
