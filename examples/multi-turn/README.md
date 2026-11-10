# `multi-turn`

This example shows a conversation that continues across turns, with control over each turn as it runs.
It is an email assistant in the terminal: it drafts an email with you and asks for your approval before it sends it.

```sh
OPENAI_API_KEY=... cargo run
```

```text
> Write a short email to ana@example.com asking to move our meeting to Friday.
...draft...
> Looks good, send it.
  The assistant wants to call `send_email` with {"body":"...","subject":"...","to":"ana@example.com"}
  Allow? [y/N]
```

## Sessions

If you run an agent directly, each conversation starts from scratch without any memories from earlier turns.
If you wish to keep the conversation going, you can create a session, which keeps a [transcript](/docs/advanced/transcript.md) of the conversation across turns.

```rust
let mut session = AssistantAgent::new(Models::gpt_4o_mini()).await.session();
```

## Running a turn

A turn represents one complete user–model interaction. 
It starts with a user message, may include zero or more tool calls, and ends when the model produces its final response.
The agent options such as `max_consecutive_fails` and `max_tool_calls` are applied per turn.

You can run a turn to its completion with `session.run_turn(input)`.
Alternatively, you can step through it yourself with `session.turn(strategy, input)`.
`session.turn(strategy, input)` returns an `AgentRunner` that advances one step at a time:

| Call | What it does |
| --- | --- |
| `turn.step()` | Calls the model and runs its tool calls with the agent's tools. |
| `turn.half_step()` | Calls the model and stops before running its tool calls, returning a `PendingStep`. |
| `pending.complete()` | Runs the tool calls with the agent's tools. |
| `pending.complete_with(results)` | Supplies the tool results yourself, without running the tools. |
| `pending.discard()` | Rolls back the step, so the model is asked again. |
| `turn.finalize(partial)` | Turns the final partial answer into the output. |

Here, `half_step` is used to ask for approval.
If the user declines, we `complete_with` each call with an error so the model can try again or report failure itself.

## Errors and retries

When using `run_turn`, the agent automatically retries failed steps up to `max_consecutive_fails` times if the error indicates that recovery may be possible (e.g., a rate limit error).
When stepping through a turn manually, retries are not handled automatically.
You must decide whether and when to retry a failed step.

## Next

- [`session-control`](../session-control) edits the transcript between steps and moves a session to another model.
- [`automatic-compaction`](../automatic-compaction) keeps long sessions within the context window.
