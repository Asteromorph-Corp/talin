# `custom-strategy`

This example shows how to write your own [strategy](/docs/advanced/strategy.md).

```sh
OPENAI_API_KEY=... cargo run
```

The agent reads a short text and records each fact with a `jot` tool.
Its final answer is a one-sentence summary, and the notes arrive separately, as the strategy's output:

```rust
let output = agent
    .run_with_strategy_immed(NotebookStrategy::new(), TEXT)
    .await?;

output.content;        // the agent's answer: the summary
output.extra_content;  // the strategy's output: the `Notebook`
```

## Strategy

A strategy defines hooks that modify major phases of a run: building the request, reading the response, running tools, and producing the answer.
It may also yield an output of its own with `finalize`.

`NotebookStrategy` implements three hooks:

| Hook | Role |
| --- | --- |
| `additional_tools` | Adds the `jot` tool for the duration of the run. |
| `process_tool_result` | Records each tool call and whether it succeeded. |
| `finalize` | Returns the notes and the call log. |

While `NotebookStrategy`'s `process_tool_result` returns the tool result as-is without modification, it is possible to modify the items from the `process_*` hooks.

## Strategy Generics

```rust
impl<L: Llm, O: OutputCtor> Strategy<L, O> for NotebookStrategy
```

If the strategy only touches tools and tool results, you can implement `Strategy` trait fot it without specifying the model (`L`) or which output type (`O`).
Pin `O` only when you need to look at the agent's output, as the validation strategy in [`structured-output`](../structured-output) does with `process_output`.

## Composition

A tuple of strategies is itself a strategy.
When using a tuple strategy, the `process_*` hooks are called in order, and the output from one strategy is passed to the next as input.
The `finalize` hooks are also called individually for each strategy, and the outputs are collected into a tuple.

It is recommended to define modular strategies that does one job, and compose them together for more complex behavior.

```rust
let strategies = (NotebookStrategy::new(), UsageTrackStrategy::default());
let output = agent.run_with_strategy_immed(strategies, TEXT).await?;
let (notebook, usage) = output.extra_content;
```

## Errors

For most hooks, returning an `Err` makes the runner try the step again instead of fully failing.

Due to the inherent randomness of LLM responses, a retry may succeed even if the first attempt failed.

Refer to the [strategy guide](/docs/advanced/strategy.md#error-semantics) for exact error semantics of each hook.

## Next

- [`transcript-edit-strategies`](../transcript-edit-strategies) changes what the model sees on each step.
