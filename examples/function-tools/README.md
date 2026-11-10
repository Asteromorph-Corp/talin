# `function-tools`

This example shows how to configure agents to use tools.
It also shows how to define and implement your own tools using the `tools.toml` configuration and `#[tool]` macro.

```sh
OPENAI_API_KEY=...    cargo run -- gpt_4o_mini
ANTHROPIC_API_KEY=... cargo run -- claude_haiku_45
```

The agent plays a number-guessing game against a tool that knows the answer.
Each turn it guesses a number, and the tool says whether the answer is higher or lower.

## Tool

The model sees the tool's name, description and input schema as they are provided in `configs/tools.toml`:

```toml
[up_or_down]
name = "Up or Down"
description = "Tells you whether your guess is higher or lower than the answer."

[up_or_down.input]
guess = { type = "u8", description = "The number you're guessing." }

[up_or_down.config]
answer = "u8"
```

Only the tool's structure and implementation need to be provided in code.
See `src/tools.rs`:

```rust
#[tool]
impl UpOrDownTool {
    type Output;
    async fn call(&self, input: Self::Input) -> Result<Self::Output, Box<dyn Error + Send + Sync>> { /* ... */ }
    fn from_config(config: _) -> Self { /* ... */ }
}
```

The input type for the tool is generated from the `tools.toml` file.
The output type is part of the tool implementation, and you can implement the `From` impl to `talin::schemas::ContentPart` on the output type to decide how the tool's output is rendered for the model.

## Configuring the tool per agent

`[up_or_down.config]` declares the possible configuration parameters for the tool.
This configuration can be set in `agents.toml` for each agent:

```toml
[[guesser.tools]]
name = "up_or_down"
config = { answer = 9 }
```

## Forcing a tool call

`call_options.tool_choice` makes the model use the tool instead of answering from memory:

```toml
[guesser.call_options]
tool_choice.function = "up_or_down"
```

The agent also has a `response_format`, so its final answer is a typed `GameResult` and not text.

## Next

- More of what `#[tool]` can do: [`custom-tools`](../custom-tools).
- The same agent on any provider: [`providers`](../providers).
