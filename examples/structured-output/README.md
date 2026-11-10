# `structured-output`

This example shows how to have an agent return a typed Rust struct instead of a `String`.

```sh
OPENAI_API_KEY=... cargo run
```

The agent receives a brief introduction from the user, and returns a structured response by extracting a name, location and email address from the input.

## Output Schema

The output schema can be defined in `agents.toml` with `response_format`, or in Rust with `#[chain(derive_schema)]` on the output struct.

The model is shown the schema, and its answer is deserialized into the struct.
If the answer does not match, the run retries, and then fails with a deserialization error.

### A schema in `agents.toml`

Add a `response_format` to the agent:

```toml
# agents.toml
[structured_output_agent]
# ...

[structured_output_agent.response_format]
name = "User Info"
description = "Extracted user information"
fields.name = { type = "String", description = "The user's full name" }
fields.location = { type = "String", description = "The user's location" }
fields.email = { type = "String", description = "The user's email address" }
```

The build script generates a `StructuredOutputAgentOutput` struct with those fields, and the agent returns it from `run`:

```rust
let result: StructuredOutputAgentOutput = agent.run_immed(input).await?;
println!("{}", result.name);
```

## Custom Struct

When the generated struct is not enough, set `response_format = "custom"` and write the struct yourself:

```toml
[custom_output_agent]
# ...
response_format = "custom"
```

```rust
// src/types.rs
#[derive(Debug, ChainOutput, Ctor)]
#[chain(derive_schema)]
#[serde(deny_unknown_fields)]
pub struct CustomOutputAgentOutput {
    /// The user's full name
    pub name: String,
    // ...
}
```

The struct must be named `[AgentName]AgentOutput`.
The `#[derive(Ctor)]` generates the constructor type `[AgentName]AgentOutputCtor`, and this must be imported where `include_agents!()` is called, e.g. `src/agents.rs`.
See [custom struct guide](/docs/advanced/custom_struct.md#agent-output) for more details.

## Validating the answer

A strategy can access the output in its typed form.
For example, `ValidationStrategy` in `src/strategy.rs` rejects a name longer than 20 characters:

```rust
async fn process_output(&mut self, output: PartialTarget<CustomOutputAgentOutputCtor>)
    -> Result<PartialTarget<CustomOutputAgentOutputCtor>, Self::Error>
{
    if output.name.len() > 20 {
        return Err(ValidationError::NameTooLong);
    }
    Ok(output)
}
```

It is run with `agent.run_with_strategy_immed(ValidationStrategy, input)`.
See [`custom-strategy`](../custom-strategy) for more details.
