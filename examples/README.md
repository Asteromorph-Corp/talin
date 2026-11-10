# Examples

Runnable examples for `talin`, ordered roughly from first steps to full applications.
Each directory is a crate in the `examples` Cargo workspace (separate from the main workspace, which excludes it), so run one with:

```sh
cd examples/<name>   # or: cargo run -p <name> from examples/
cargo run
```

Most examples require an API key to run, such as `OPENAI_API_KEY` or `ANTHROPIC_API_KEY`
You can provide the API keys from the environment (`ENV_VAR_NAME=<your-api-key> cargo run`), or copy [`.env.example`](.env.example) to `.env` in this directory.
The `.env` file is found when you run an example from its own directory.
Examples that need anything more (a server, a feature flag) say so in their README.

The reference documentation lives in [`docs/`](/docs); each section below links to the guide it accompanies.

## 1. Getting started

| Example | What it shows |
| --- | --- |
| [`hello-agent`](hello-agent) | The smallest useful agent: one `agents.toml`, one `build.rs`, one `run_immed` call. The README walks through what the build script generates. |
| [`function-tools`](function-tools) | Giving an agent function tools: defining them in `tools.toml`, implementing them with `#[tool]`, configuring them per agent, and forcing a tool call. |
| [`structured-output`](structured-output) | Having the model answer with a typed Rust struct instead of a `String`, and what to do for models without native support (`FinalAnswerToolStrategy`). |

Guides: [agents](/docs/agents.md), [running agents](/docs/agents_api.md), [tools](/docs/tools.md), [models](/docs/models.md).

## 2. Models and providers

| Example | What it shows |
| --- | --- |
| [`providers`](providers) | One agent run against any provider (Claude, OpenAI Chat and Responses, Gemini, xAI, generic chat-completions servers), with the configuration of each and what each accepts. Also covers pricing and what `stream = true` does. |
| [`reasoning`](reasoning) | Setting reasoning effort, reading the model's thoughts, and keeping or dropping them between steps. |
| [`multimodal`](multimodal) | Sending images, audio, video and documents as typed attachment slots. |
| [`builtin-tools`](builtin-tools) | Using a provider's server-side tools, such as web search, from `agents.toml`. |

Guide: [models](/docs/models.md).

## 3. Tools

| Example | What it shows |
| --- | --- |
| [`custom-tools`](custom-tools) | What the `#[tool]` macro can do beyond the basics: descriptions built from state, per-agent configuration, errors the model can recover from, rich outputs, and usage limits. |
| [`local-mcp`](local-mcp) | Using MCP tools through a local MCP client: this program connects to the server, over HTTP or stdio. Includes both example servers. |
| [`remote-mcp`](remote-mcp) | Using MCP tools through a remote MCP client: the provider connects to a public server and calls the tools itself. |
| [`skills`](skills) | Loading skills from disk, listing them, and letting the agent read a skill's references on demand. |

Guides: [tools](/docs/tools.md), [tool configuration](/docs/advanced/tool_config.md).

## 4. Organizing configuration

Techniques for keeping `agents.toml` and `tools.toml` manageable as the number of agents grows.

| Example | What it shows |
| --- | --- |
| [`shared-templates`](shared-templates) | Defining many agents that differ in a few attributes by sharing prompt structure. |
| [`custom-struct`](custom-struct) | Replacing a generated tool input or tool config struct with your own type by setting the field to `"custom"`, to use enums, optional fields and nested types. An agent's output struct can be replaced too, see [`structured-output`](structured-output). |

Guides: [agents](/docs/agents.md), [custom structs](/docs/advanced/custom_struct.md).

## 5. Sessions and control flow

| Example | What it shows |
| --- | --- |
| [`multi-turn`](multi-turn) | A conversation across turns with a session: stepping through a turn to approve tool calls, retrying failed steps, and cancelling a reply. |
| [`session-control`](session-control) | Editing the transcript between steps, and continuing a session on a different model. |
| [`agent-chaining`](agent-chaining) | Chaining agents so each one's output feeds the next, and carrying values forward through the chain. |

Guides: [running agents](/docs/agents_api.md), [transcript](/docs/advanced/transcript.md).

## 6. Strategies

A strategy hooks into an agent run and produces its own output, returned next to the agent's answer.
See the [strategy guide](/docs/advanced/strategy.md) for the hooks and the full list of built-in strategies.

### Simple strategies

These are small enough to be demonstrated in a few lines each, so they are shown in the strategy guide's own code samples and not in separate crates:
`RecordStrategy`, `UsageTrackStrategy`, `SharedUsageTrackStrategy`, `ToolStrategy`, and combining several strategies by passing a tuple to `run_with_strategy`.

### Advanced strategies

| Example | What it shows |
| --- | --- |
| [`custom-strategy`](custom-strategy) | Writing a strategy: one that adds a tool for the run and returns what the tool collected. Validation with `process_output` is shown in [`structured-output`](structured-output). |
| [`buffered_final_answer`](/crates/talin/examples/buffered_final_answer.rs) | Letting the model build a long answer in pieces and revise it before submitting. Needs the `buffered-final-answer` feature. |
| [`transcript-edit-strategies`](transcript-edit-strategies) | Applying an edit scheme on every step, either permanently (`*EditStrategy`) or only for the request being sent (`*ProjectionStrategy`). |

Compaction strategies are covered in section 7.

## 7. Long-running sessions

| Example | What it shows |
| --- | --- |
| [`automatic-compaction`](automatic-compaction) | Summarizing history automatically when a threshold is reached, on-demand compaction, and streaming, with replay tests that run without API keys. |

Guide: [compaction](/docs/advanced/compaction.md).

## 8. Full applications

| Example | What it shows |
| --- | --- |
| [`assistant-cli`](assistant-cli) | An interactive terminal assistant for a project directory: tool approval, custom tools, an MCP server, skills, compaction, typed reports, and usage and cost display. Its README maps each feature to the focused example it builds on. |

## Library examples

The `talin` crate also ships examples for the lower-level `Llm` and `Chain` APIs, for use without the TOML-based agent definitions.
Run them with `cargo run -p talin --example <name>`.

| Example | What it shows |
| --- | --- |
| [`llm`](/crates/talin/examples/llm.rs) | A single `Llm::invoke` call against any provider. |
| [`llm_chain`](/crates/talin/examples/llm_chain.rs) | An `LlmChain` with a system message and a user template. |
| [`llm_chain_multimodal`](/crates/talin/examples/llm_chain_multimodal.rs) | The same, with image input. |
| [`simple_chain`](/crates/talin/examples/simple_chain.rs) | A chain with a derived input type. |
| [`sequential_chain`](/crates/talin/examples/sequential_chain.rs) | Chaining without agents. |
| [`agent`](/crates/talin/examples/agent.rs) | A hand-built `Agent` with a tool, no code generation. |
| [`agent_builtin_tools`](/crates/talin/examples/agent_builtin_tools.rs), [`agent_multimodal`](/crates/talin/examples/agent_multimodal.rs), [`agent_skills`](/crates/talin/examples/agent_skills.rs) | The same feature examples as above, without code generation. |
