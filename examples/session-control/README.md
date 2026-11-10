# `session-control`

This example shows how to take control of a session: move it to another model, edit its transcript while a turn runs, and hand it to a different agent.

```sh
OPENAI_API_KEY=... ANTHROPIC_API_KEY=... cargo run
```

It runs one conversation in three stages:

1. **GPT-4o mini** plays the number-guessing game from [`function-tools`](../function-tools).
2. **Claude Haiku** takes over the same conversation, and plays again.
3. A **summarizer** agent takes over and critiques the game.

## Switching models

A session's transcript can be handed to a different agent of the same provider (`continue_session`) or a different provider (`continue_session_migrated`).
The new session keeps the entire history, but adopts the new agent's prompt, tools and output type.

```rust
let agent = GuesserAgent::new(Models::claude_haiku_45()).await;
let mut session = agent.continue_session_migrated(session)?;
```

If the new agent uses a different provider, the session's history is converted to the new provider's format.
This can lose provider-specific details, or outright fail if the history includes media the new provider does not accept.

## Editing the transcript

`session.turn(..)` starts a turn without running it.
Calling `step()` runs one model call, and the transcript can be edited before the next one.
Here every failed tool call is removed, so the model does not see its earlier mistakes:

```rust
let edits = view
    .into_iter()
    .filter_map(|entry| match entry {
        EntryView::ToolError(_, _) => None,
        other => Some(EntryEdit::from(other)),
    })
    .collect::<Vec<_>>();
turn.session.transcript.modify(edits, &scope)?;
```

See the [transcript guide](/docs/advanced/transcript.md) to learn more about editing the transcript, and [`transcript-edit-strategies`](../transcript-edit-strategies) for applying edits automatically.

## Next

- Running a session turn by turn with a user: [`multi-turn`](../multi-turn).
