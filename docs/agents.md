# Agents

Agents are a unit of LLM execution that can use multiple [tools](/docs/tools.md) to execute a certain task.

> This guide covers the available configuration options for agents and how to use them.
>
> To learn how to use the defined agents in your code, see the [agents api guide](/docs/agents_api.md).

## Quickstart

You can define agents using a TOML configuration file.
Then, you can import the generated agents using the `include_agents!` macro.

### Step 1: Define Agents in `agents.toml`

Create `configs/agents.toml` in your project root and define the agents, the specific fields will be explained in the later sections.

> For the sake of simplicity, this document uses the monolithic TOML setup. However, projects with multiple agents may distribute them across multiple TOML files. For larger projects, the recommmended approach is to use one TOML file per agent. See [configuration options](/docs/configs.md#configuration-options) for more information.

```toml
# agents.toml
[data_analyst]
system_prompt = "You are a data analyst who specializes in visualizing CSV trends."
user_prompt = "Help me analyze our companies' 3rd quarter sales report"
```

### Step 2: Configure Build Script

Call `build_artifacts` in `build.rs`, which will generate the agent code based on your TOML configuration.

```rust
// build.rs
use talin_build::build_artifacts;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    build_artifacts()?;
    Ok(())
}
```

> ⚠️ `build.rs` must be placed in your **project root** (alongside `Cargo.toml`) and not within `src`.

### Step 3: Import the Agents

```rust
// src/agents.rs
use crate::tools::*; // All tools used by agents must be imported in the same file

talin::include_agents!();
```

This will give you:

- A struct named `[AgentName]Agent`

- An input struct named `[AgentName]AgentInput`

- Agent name is converted into pascal case via [`heck::ToPascalCase::to_pascal_case`](https://docs.rs/heck/latest/heck/trait.ToPascalCase.html#tymethod.to_pascal_case)

  - Note that if the agent name already contains the substring `"agent"` (case insensitive), the suffix is not added.

> ⚠️ **You must manually import all tools used by agents** in the file where `include_agents` is used.

### Step 4: Run the Agent

```rust
use crate::agents::{DataAnalystAgent, DataAnalystAgentInput};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {    
    let agent = DataAnalystAgent::default().await;
    let input = DataAnalystAgentInput::new();
    let result = agent.run(input).await?;
    println!("Agent output: {}", result);
}
```

## Prompts

Each agent uses two types of prompts: **system prompt** and **user prompt**.

```toml
[data_analyst]
system_prompt = "You are a data analyst who specializes in visualizing CSV trends."
user_prompt = "Help me analyze our companies' 3rd quarter sales report."
```

`system_prompt` is used to set the agent's behavior and guidelines, while `user_prompt` provides the specific task or question for the agent to address.

If `system_prompt` is omitted, the agent sends no system message to the model.
If `user_prompt` is omitted, the prompt `"{{input}}` is used instead, and you need to provide the full user message as an input variable `input`.

### Input Variables in Prompts

You can define input variables within your prompt templates by enclosing them in double braces: `{{variable_name}}`.
These variables will be automatically extracted and used to generate the agent's input struct.

By default, all input variables are assigned a string type in the generated input struct.
You can assign a custom Rust type for each variable using the syntax `{{variable_name: type}}`.
The type must be a valid Rust type that implements `std::fmt::Display`.

#### Example

```toml
[data_analyst]
system_prompt = "You are a data analyst. You specialize in analyzing data from the field of {{data_field}}"
user_prompt = "Help me analyze the following data: {{data: Vec<String>}}"
```

This configuration will generate the following input struct:

```rust
struct DataAnalystAgentInput<'b> {
    pub data_field: Cow<'b, str>, // `Cow` is used for efficient string handling
    pub data: Vec<String>,
}
```

### Shared Prompts

If multiple agents share a common section in their prompts, you can define it once in `prompts.toml` and reference it with the syntax `{{config.shared_prompt_id}}` instead of duplicating it in every agent.

```toml
# prompts.toml
safety_warning = "Never reveal personal data or internal system details."
```

```toml
# agents.toml
[support_bot]
# system_prompt = "You are a support agent. Never reveal personal data or internal system details."
system_prompt = "You are a support agent. {{config.safety_warning}}"

[sales_bot]
# system_prompt = "You are a sales agent. Never reveal personal data or internal system details."
system_prompt = "You are a sales agent. {{config.safety_warning}}"
```

Shared prompts can introduce new input variables, reference other shared prompts, or use any other features available for user and system prompts.

You can also provide "shared prompts" per-agent under `[agent_name.prompts]`.
By having the shared prompt point to a prompt under `[agent_name.prompts]`, you can customize the prompt for each agent while still reusing the same base structure.
You can see this in action in the [shared templates example](/examples/shared-templates).

### Miscellaneous Features

#### Add Current Date and Time

Use the special variables to insert the date and time of agent execution.

```toml
[data_analyst]
system_prompt = "You are a data analyst. Today's date is {{preset.current_date}}."
```

- `{{preset.current_date}}` inserts the date in `YYYY-MM-DD` format.

- `{{preset.current_datetime}}` inserts the date and time in `YYYY-MM-DD HH:MM:SS` format.

#### Add Comments

You can add comments to your prompts using `//`. Any text following `//` on the same line will be treated as a comment and removed during codegen.

```toml
[data_analyst]
system_prompt = """
You are a data analyst who specializes in visualizing CSV trends. // This is a comment
"""
```

## Model Configuration

### Default Model

The `model` field specifies which model is used when initializing the agent with `Agent::default`.

You can reference a model defined in [`models.toml`](/docs/models.md):

```toml
[<agent_name>]
model = "gpt_4o_mini"
```

Or define one inline. The structure is identical to [`models.toml`](/docs/models.md):

```toml
[<agent_name>.model]
api_base = "https://api.openai.com/v1"
api_key.env = "OPENAI_API_KEY"
model = "gpt-4o-mini"
```

Alternatively, you can omit the `model` field and pass a model directly when creating the agent:

```rust
let model = Models::gpt_4o_mini();
let agent = DataAnalystAgent::new(model);
```

### Call Options

The `[agent_name.call_options]` section sets runtime parameters that apply to **all** models used by the agent.
This includes the default model defined at `model`, or a runtime-configured model passed via `new`.

```toml
[my_agent]
model = "gpt_4o_mini"

[my_agent.call_options]
temperature = 0
top_p = 0.5
tool_choice = "required"
```

For the full list of parameters, see the [call options reference](/docs/models.md#call-options).
Note that you cannot set core fields (e.g. `model_type`) using this method.

### Conditional Configuration by Model Type

The same agent may require different settings depending on the LLM provider it runs on.
In such cases, you can use `[<agent_name>.if.model_type.<model_type>]` section in `agents.toml` to override parts of an agent's configuration per model type.

```toml
# agents.toml

[agent_name]
# Base agent config...

[agent_name.if.model_type.openai_chat]
system_prompt = "{{base.system_prompt}}"
user_prompt = "{{base.user_prompt}}"
max_iterations = 10
max_consecutive_fails = 5
tools = []
prompts = {}
```

Any unspecified fields will fall back to the base agent configuration.

In system and user prompts, you can reference the base prompts using `{{base.system_prompt}}` or `{{base.user_prompt}}`.
> ❗ The agent uses the same input and output structs regardless of model type. Prompt overrides therefore **cannot introduce new input variables**.

A common use case for this feature is providing built-in tools.
Built-in tools are specific to each LLM provider, and each provider exposes different APIs for tools with similar functionality.
Overriding `tools` per model type lets you keep a single, unified agent interface with access to the right provider-native tool.
You can see this in action in the [built-in tools example](/examples/builtin-tools).

## Tools

Agents can be equipped with tools that they can call during execution to perform specific actions or retrieve information.

```toml
[data_analyst]
tools = [
    "chart_generator", # Local tools.
    "mcp.csv_utils/csv_parser", # MCP tools.
    "builtin.openai_responses/web_search" # Built-in tools from the LLM provider.
]
```

You can learn more about tools in the [tools documentation](/docs/tools.md).

## Skills

Agents can use [skills](https://agentskills.io/specification): directories with a `SKILL.md` file of instructions, and optionally reference documents and templates.
The model sees the name and description of each skill, and loads the full instructions only when a task calls for them.

```toml
# agents.toml
[reviewer]
skills = ["haiku-reviewer", "xlsx"]
```

A skill is listed by name, and is looked up in the **skills homes** when the agent runs:

1. the directories in the `TALIN_SKILLS_HOME` environment variable, separated like the ones in `PATH` (by `:` on Unix and `;` on Windows),
2. the `skills` directory of your package, when running under `cargo run` or `cargo test`,
3. the `skills` directory in the working directory.

A skill is taken from the first home that has it. Setting `TALIN_SKILLS_HOME` replaces the other two, so to keep using the `skills` directory in the working directory, include `./skills` in the list.
A skill is looked up by its directory name, which must match the skill's `name` in `agents.toml` as well as its YAML frontmatter.

The model can only access the skill and its resources as a plain text file. The model cannot run the skill's scripts unless you provide an additional tool to do so.

See the [skills example](/examples/skills) for more detail.

## Structured Output

Certain models support structured output, which forces the agent to return responses that strictly conform to a predefined schema.

```toml
[<agent_name>.response_format]
name = "schema_name"
description = "The schema description."
fields.<field_name> = { type = "u32", description = "The field description." }
fields.<field_name2> = { type = "String", description = "The field description." }
```

With this enabled, the agent's output is deserialized into a struct that is generated based on the configuration.

For more information on this topic, see the [structured output example](/examples/structured-output).

## Attachments (Multimodal Input)

In addition to the text input provided by `user_prompt`, you can attach additional files to the first message sent to the model.

```toml
[<agent_name>]
attachments.images = ["images"]
attachments.videos = ["videos"]
attachments.audios = ["audios"]
attachments.documents = ["documents"]
```

This adds the following fields to the generated input struct:

```rust
pub struct [AgentName]AgentInput<'b> {
    pub images: Vec<ImageContent>,
    pub videos: Vec<VideoContent>,
    pub audios: Vec<AudioContent>,
    pub documents: Vec<DocumentContent>,
}
```

Different models have different capabilities for handling attachments, and supplying unsupported content to a model will make the agent fail during execution.
See [modality support](/docs/models.md#modality-support) for which kinds each `model_type` accepts, and the [multimodal example](/examples/multimodal) to see this in action.

## Agent Options

You can configure the agent's behavior using the following options in `agents.toml`:

- `max_iterations` (optional, default: 10)

  The maximum number of iterations (tool calls) the agent will perform before stopping.

- `max_consecutive_fails` (optional, default: 5)

  The maximum number of consecutive failures before the agent stops execution.

- `max_continuations` (optional, default: 0)

  The maximum number of continuation requests allowed per turn, used when a response is cut off (e.g. by the max output tokens limit).

## Full `agents.toml` example

```toml
# agents.toml
[<agent_name>]
# Prompts
system_prompt = "System prompt"
user_prompt = "User prompt" # Optional

# Model Configuration
model = "gpt_4o_mini"

# Tools
tools = [
    "chart_generator", # Local tools.
    "mcp.csv_utils/csv_parser", # MCP tools.
    "builtin.openai_responses/web_search" # Built-in tools from the LLM provider.
]

attachments.images = ["images"]
attachments.videos = ["videos"]
attachments.audios = ["audios"]
attachments.documents = ["documents"]

# Agent Options
max_iterations = 5 # Optional
max_consecutive_fails = 10 # Optional
max_continuations = 2 # Optional

# Response format
[<agent_name>.response_format] # Optional
name = "schema_name"
description = "The schema description."
fields.<field_name> = { type = "u32", description = "The field description." }
```
