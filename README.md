# Talin

A minimal, type-safe framework for defining LLM agents using TOML.

## Overview

`talin` lets you build LLM agents with minimal configuration.
Instead of wiring prompts, tools, variables, and system messages through scattered code, you can define agents in a single TOML file and interact with them programmatically in a type-safe manner.

## Key Features

- Simplified agent definition via TOML
- Strongly-typed prompt variables mapped into Rust structs
- Composable system/user prompts

## Installation

This package consists of two crates:

1. **`talin`**: Used in your application code.
2. **`talin-build`**: Used in the `build.rs` build script.

Both crates need to be added to your project.

### Using `cargo add`

Run the following commands to add the dependencies

```sh
cargo add talin
cargo add --build talin-build
```

### Manual Configuration

Alternatively, add the dependencies manually in your `Cargo.toml`:

```toml
# Cargo.toml
[dependencies]
talin = "0.6"

[build-dependencies]
talin-build = "0.6"
```

## Example

### `agents.toml`

```toml
[guesser]
system_prompt = "You are a number guesser, a master of deduction and probability."
user_prompt = """
Guess the number that the user is thinking of using the provided tools.
The number is between 1 and {{max_number: u32}}
"""

[guesser.model]
api_base = "https://api.openai.com/v1"
model = "gpt-4o-mini"
api_key_env_var = "OPENAI_API_KEY"
```

### `build.rs`

Before writing any application code, call `talin_build::build_artifacts()` from `build.rs`.

```rs
// build.rs
use talin_build::build_artifacts;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    build_artifacts()?;
    Ok(())
}
```

> ⚠️ `build.rs` must be placed in your **project root** (alongside `Cargo.toml`) and not within `src`.

This parses the various TOML files and generates the corresponding Rust code in `OUT_DIR`, which consists of:

- The agent struct, e.g. `GuesserAgent`
- Input struct automatically derived from the prompt variables, e.g.`GuesserAgentInput`
- Additional codes discussed in [detailed guide](#-detailed-usage-guide)

The generated code must then be imported into your application code using the appropriate macro.

### Application code

```rs
use talin::include_agents;

// Import generated agent definitions
include_agents!();

#[tokio::main]
async fn main() {
    // Create an agent instance using the model config in agents.toml
    let agent = GuesserAgent::default().await;
   
    // Construct typed input
    let input = GuesserAgentInput { max_number: 10u32 };

    // Execute the agent
    let result = agent.call(input).await;

    match result {
        Ok(output) => println!("Agent response:\n{output}"),
        Err(e) => eprintln!("Error: {e}"),
    }
}
```

## Detailed Usage Guide

- [Setup config files](/docs/configs.md)

- [Define](/docs/agents.md) and [use](/docs/agents_api.md) agents

- [Implement tools for agents](/docs/tools.md)

- [Configure Model parameters](/docs/models.md)

## Debugging & Logging

To view internal agent/LLM behavior, set the environment variable `RUST_LOG`

- **Basic debugging**:

  ```sh
  RUST_LOG=talin=debug
  ```

- **Verbose tracing**:

  ```sh
  RUST_LOG=talin=trace
  ```
