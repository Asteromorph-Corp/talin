# `reasoning`

This example shows how to control how much a model reasons, read what it reasoned, and decide whether that reasoning is sent back to it.

```sh
ANTHROPIC_API_KEY=... cargo run -- claude
OPENAI_API_KEY=...    cargo run -- openai
```

The agent answers a question that needs five exchange-rate lookups, a conversion fee, a ranking and a follow-up calculation.
The model is expected to reason about the results before returning the final answer.

## Set `reasoning_effort`

`reasoning_effort` is a [call option](/docs/models.md#call-options), so it can be set on a model in `models.toml` or on an agent in `agents.toml`:

```toml
# agents.toml
[solver]
# ...
call_options.reasoning_effort = "high"

[solver_quick]
# ...
call_options.reasoning_effort = "low"
```

The levels are `none`, `minimal`, `low`, `medium`, `high`, `xhigh` and `max`.
Providers support different subsets, and an unsupported level is mapped to the nearest supported one.

`solver` and `solver_quick` are the same agent apart from this option.
The example runs both agents on the same question and prints the token usage of each, so you can see what the extra effort costs.

## Read the model's reasoning

The model's reasoning is stored in the `thought` field of each assistant and tool-call message.
You can inspect the reasoning by using a `RecordStrategy` to collect the messages, and read the `thought` field of each message in the transcript.

```rust
let strategies = (RecordStrategy::new(), UsageTrackStrategy::default());
let output = agent.run_with_strategy_immed(strategies, QUESTION).await?;
let (record, usage) = output.extra_content;
```

Some providers return only parts of the reasoning block, or return it as a encrypted or opaque blob for security reasons.
These are often not human-readable, and they are not stored in the transcript,
Therefore, it is strongly discouraged to rely on the existence of `thought` field or lack thereof in your program logic.

## Next

- [`session-control`](../session-control) edits the transcript, including the thoughts, between steps.
