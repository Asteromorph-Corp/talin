# Tools

Agents can be equipped with tools to complete tasks more effectively.

This document explains how to attach different types of tools to your agents, as well as how to implement your own custom tools.

## Types of Tools

There are three types of tools:

1. **Function Tools** (Also referred to as **Local Tools**): Custom tools you implement in your codebase, or the tools pre-implemented in this crate.
2. **MCP Tools**: Tools exposed by [Model Context Protocol](https://modelcontextprotocol.io/) servers over streamable HTTP or stdio.
3. **Built-in Tools**: Tools provided natively by certain model providers.

## Custom Function Tools

You can define custom function tools using the `tools.toml` configuration file.

### Step 1: Define the Tool in `tools.toml`

Create `configs/tools.toml` in your project root and define the models, the specific fields will be explained in the later sections.

> For the sake of simplicy, this document uses the monolithic TOML setup. However, projects with a multiple tools may distribute them across multiple TOML files. See [configuration options](/docs/configs.md#-configuration-options) for more information.

  ```toml
  [addition]
  name = "add_numbers"
  description = "Add two numbers"
  
  [addition.input]
  num1 = { type = "i64", description = "The first number to add" }
  num2 = { type = "i64", description = "The second number to add" }
  ```

### Step 2. Configure build script

  - Call `build_artifacts` in `build.rs`

    ```rust
    // build.rs
    use talin_build::build_artifacts;

    fn main() -> Result<(), Box<dyn std::error::Error>> {
        build_artifacts()?;
        Ok(())
    }
    ```

### Step 3. Define a Struct Named `[ToolName]Tool`

`ToolName` must be the pascal case version of the name defined in `tools.toml`, the conversion uses [`heck::ToPascalCase::to_pascal_case`](https://docs.rs/heck/latest/heck/trait.ToPascalCase.html#tymethod.to_pascal_case). Note that if the tool name already contains the substring `"tool"` (case insensitive), the suffix is not added.

```rust
// src/tools.rs
struct AdditionTool {
    // You may want to add fields like api_key here
}
```

### Step 4. Implement the Tool Functionality with a `#[tool]` Macro

```rust
// src/tools.rs
use talin::tool;

#[tool]
impl AdditionTool {
    type Output = i64; // Optional, default `String`

    // `Self::Input` is generated from toml definition
    async fn run(&self, input: Self::Input) -> Result<Self::Output, Box<dyn Error>> {
        Ok(input.num1 + input.num2)
    }

    // Optional: if this returns false, the tool is left out of requests and calls to it are rejected.
    // Default: always available.
    fn available(&self, transcript: &[EntryView<'_>]) -> bool {
        count_tool_calls(transcript, "addition") < 5
    }
}
```

> 🔔 **Note**: Due to how availability is computed, editing the transcript may affect the tool's availability.
> For example, if a call cap is written with [`count_tool_calls`] (e.g. `count_tool_calls(transcript) < N`), dropping old calls from the transcript may make the tool available again even after `N` calls have been made.
> If a tool requires a strict call cap, it should be implemented in the tool itself, e.g. with an internal counter.

#### Dynamic `name` and `description`

By default, the tool's name and description come from `tools.toml`. If they should depend on the tool's own state or configuration (e.g. a search tool that mentions its domain or language), define `name` and/or `description` in the `#[tool]` block.
If a field is defined in the TOML file, it is also available as the constant `TOOL_NAME` or `TOOL_DESCRIPTION`, so you can use it as a base:

```rust
#[tool]
impl WebSearchTool {
    fn description(&self) -> String {
        format!(
            "{TOOL_DESCRIPTION} Searches for articles about {} written in {}.",
            self.domain, self.language,
        )
    }

    // `name` can be overridden the same way
    // fn name(&self) -> String { ... }

    // ...
}
```

Both methods take `&self`, and the results are evaluated whenever the tool spec is built.
You can choose to omit `name` and `description` from `tools.toml`, but if you do so, the `#[tool]` block for the tool has to define the `name` and `description` methods.

> 🔔 **Note**: You get a generated `[ToolName]ToolInput` struct with the fields specified in toml. 
> This is the type of `Self::Input`

#### The `Output` type

`Self::Output` may be any type implementing `Into<ToolOutput>`:

```rust
pub struct ToolOutput {
    pub content: Vec<ContentPart>,
    pub summary: Option<String>,
}

pub enum ContentPart {
    Text(String),
    Image(ImageContent),
    Audio(AudioContent),
    Video(VideoContent),
    Document(DocumentContent),
}
```

Blanket impls in the crate cover any type implementing `Into<ContentPart>` (such as the `i64` above) and `Vec<impl Into<ContentPart>>`, so you only need to build a full `ToolOutput` when you want to set a `summary` or mix content parts.

### Step 5. Use the Tool in `agents.toml`

```toml
[math_expert]
# other agent configuration...
tools = ["addition"]
```

Now, when you run your agent, it will be able to use the `addition` tool to add two numbers!

```rust
// src/agents.rs
use crate::tools::AdditionTool; // You need to import the tool struct where you call `include_agents`

talin::include_agents!();
```

## Custom Tool Output

The `Output` type of a custom tool may be any type that implements `Into<ToolOutput>`:

```rust
pub struct ToolOutput {
    pub content: Vec<ContentPart>,
    pub summary: Option<String>,
}

pub enum ContentPart {
    Text(String),
    Image(ImageContent),
    Audio(AudioContent),
    Video(VideoContent),
    Document(DocumentContent),
}
```

For any type `T` that implements `Into<ContentPart>`, blanket impls in the crate provide an implementation of `Into<ToolOutput>` for both `T` and `Vec<T>`, so these types can be used as an output type as well.
You may also return the full `ToolOutput` struct if you want to set a `summary` or mix different content parts.

> ⚠️ Most models only support text and image as tool result, and unsupported content will fail the agent loop at runtime.
> See [modality support](/docs/models.md#-modality-support) for which content types are supported by which models.

> `summary` is currently unused, but may be used in the future for compaction / summarization of tool outputs.

## MCP Tools

[Model Context Protocol](https://modelcontextprotocol.io/) servers expose tools over streamable HTTP or stdio.

By default, if the provider supports MCP tool calls natively, we trust it to connect to the server and call the tool on its own.
For providers without native support, we initialize a local MCP client and handle the tool calls ourselves while advertising the tool as a regular function tool to the model.

If the server is not reachable from provider server for any reason (e.g. `localhost` servers, firewalls), you can set `always_local = true` in the server definition to force the use of a local MCP client.

### Step 1: Define the MCP Server in `servers.toml`

Create `configs/servers.toml` in your project root and define the mcp servers.

```toml
# servers.toml
[counter]
url.env = "COUNTER_URL" # The URL of the MCP server
authorization.env = "COUNTER_API_KEY" # Optional, the authorization token for the MCP server
description = "Counter service with tools." # Optional, brief description of the server
tools = ["get_value", "increment", "decrement"] # Optional, list of tools exposed by the server
always_local = true # Optional, if true, a local MCP client will be used regardless of provider capabilities.
```

To run a local server as a child process over stdio instead, use `command` in place of `url`:

```toml
# servers.toml
[filesystem]
command = "npx" # The program to run (mutually exclusive with `url`)
args = ["-y", "@modelcontextprotocol/server-filesystem", "/tmp"] # Optional
env = { API_KEY.env = "FS_API_KEY", MODE = "fast" } # Optional, extra environment variables for the process
tools = ["read_file"]
```

`authorization` is only valid with `url`, and `args`/`env` only with `command`. Stdio servers always use the local MCP client, since a provider cannot reach a process on your machine.

### Step 2. Use the Tool in `agents.toml`

```toml
# agents.toml
[math_expert]
# other agent configuration...
tools = ["mcp.counter/increment", "mcp.counter/decrement"] # Use the tool in the format of "mcp.{server_name}/{tool_name}"
```

Now, when you run your agent, it will be able to use the `decrement` tool from the server.

For LLM providers that natively support MCP tool call, we send the server data to the provider directly. Otherwise, the calls to the tool are handled like a function tool call using the local MCP client.

## Built-in Tools

Some providers may offer built-in tools that can be used directly.

You can see this in action in the [builtin tools example](/examples/builtin-tools).

```toml
# agents.toml
[research_assistant]
# other agent configuration...
tools = ["builtin.openai_responses/web_search"] # Use the tool in the format of "builtin.{provider}_{api}/{tool_name}"
```

## Detailed Tool Configuration for Agents

In the previous sections, we used the shorthand syntax of using a simple string for the `tools` field in `agents.toml`.

Alternatively, you can use the following syntax:

```toml
# agents.toml
[<agent_name>]
# other agent configuration...
# 
[[agent_name.tools]] # Local Tool
kind = "local"
name = "local_tool_name"

[[agent_name.tools]] # MCP Tool
kind = "mcp"
server = "mcp_server_name"
name = "mcp_tool_name"

[[agent_name.tools]] # Built-in Tool
kind = "builtin"
provider = "provider_apiname" # e.g. "openai_responses", "claude_messages", "gemini_generate_content"
type = "builtin_tool_name" # This is `type` because most LLM providers use the field `type` for built-in tools
<option_name> = <value> # Additional options specific to the tool
```

This is useful for specifying additional options for each tool. This includes the [initialization parameters for function tools](/docs/advanced/tool_init.md) or the options for built-in tools which varies depending on the provider.

## Full `tools.toml` Example

```toml
# tools.toml
[<tool_name>]
name = "tool_name"
description = "Brief description of the tool"

[<tool_name>.input]
<field_name> = "i64" # Shorthand field definition
<field_name2> = { type = "String", description = "Brief description of the field" } # Detailed field definition

[<tool_name>.config] # Configuration parameters
<param_name> = "u32"
<param_name2> = "String"
```
