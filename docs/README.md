# Documentation

Documentation for `talin`'s features.

If you are new, it is recommended to read through the guides below in order.
The [runnable examples](/examples/README.md) are also a good way to learn by doing.

If you are already familiar with the concepts, you can jump to the [cheatsheet](cheatsheet.md) for complete `agents.toml`, `tools.toml` and `models.toml` examples.

## 1. Getting started

| Guide | What it covers |
| --- | --- |
| [Configuration files](configs.md) | Where to place different configuration files, and how to organize them. |
| [Agents](agents.md) | Defining an agent with `agents.toml`: prompts and typed variables, model, tools, skills, structured output, attachments, and agent options. |
| [Running agents](agents_api.md) | Using the generated agent: constructors, the `Agent` type, sessions and multi-turn conversations. |

## 2. Models and tools

| Guide | What it covers |
| --- | --- |
| [Models](models.md) | Configuring LLMs in `models.toml`: providers, model types, and call options such as temperature and reasoning. |
| [Tools](tools.md) | Giving agents function tools, MCP tools and provider built-in tools, implementing your own, and configuring them per agent. |

## 3. Advanced Configuration

| Guide | What it covers |
| --- | --- |
| [Agent inheritance](advanced/agent_inheritance.md) | Sharing properties between agents with parent and abstract agents. |
| [Custom structs](advanced/custom_struct.md) | Replacing a generated tool input, tool initializer or agent output struct with your own type. |
| [Per-agent tool configuration](advanced/tool_config.md) | Passing agent-specific options to a function tool through `from_config`. |

## 4. Controlling agent runs

| Guide | What it covers |
| --- | --- |
| [Strategy](advanced/strategy.md) | Hooks controlling the agent's behavior during a run. |
| [Transcript editing](advanced/transcript.md) | Reading and editing the conversation state directly. |
| [Compaction](advanced/compaction.md) | Replacing old history with a summary in long-running sessions, automatically or on demand. |

## Debugging

Set `RUST_LOG=talin=debug` (or `talin=trace`) to see internal agent and LLM behavior.
