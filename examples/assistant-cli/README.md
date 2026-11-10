# `assistant-cli`

A terminal assistant for a project directory, built on `talin`.
It answers questions by reading files, keeps notes, asks before it reads, looks things up through an MCP server, and writes reports.

```sh
ANTHROPIC_API_KEY=... cargo run
```

```text
> What does configs/servers.toml set up?
  Allow `read_file` {"path":"configs/servers.toml"}? [y/N] y

It defines one MCP server, `deepwiki`...

[this turn]
- claude-haiku-4-5-20251001 (...)
  - Input tokens: 1520 tokens x $1 per 1M tokens = $1.5e-3
  - Output tokens: 214 tokens x $5 per 1M tokens = $1.1e-3
  - Subtotal: $2.6e-3

> /report what we found
```

## Commands

| Command | |
| --- | --- |
| *(any text)* | Asks the assistant. |
| `/report [focus]` | Writes a report of the conversation. |
| `/notes` | Shows the notes the assistant took with `take_note`. |
| `/usage` | Shows the tokens used so far, and their cost. |
| Ctrl-C | Cancels a reply that is being generated, or a question waiting for approval. At the prompt it quits. |
| *(empty line)* | Quits. |

## Feature Usage

| Feature | In this example | Focused example |
| --- | --- | --- |
| A conversation across turns | `assistant.session()` in `main.rs` keeps the history. | [`multi-turn`](../multi-turn) |
| Approval before a tool runs | `turn.rs` stops each step before its tool calls with `half_step`, and answers a refused call itself with `complete_with`. | [`multi-turn`](../multi-turn) |
| Cancelling | `main.rs` races the turn against Ctrl-C. | [`multi-turn`](../multi-turn) |
| Tools with configuration | `read_file` and `take_note` get their directory and file from `agents.toml`. | [`custom-tools`](../custom-tools) |
| Skills | `note-style` is loaded from `skills/` when the assistant is about to take a note. | [`skills`](../skills) |
| An MCP server | `deepwiki` in `servers.toml`, which Claude connects to itself because the `mcp-client` beta is enabled. | [`remote-mcp`](../remote-mcp) |
| Compaction | `with_compaction` summarizes older messages once the conversation is long. | [`automatic-compaction`](../automatic-compaction) |
| A typed answer from another agent | `/report` hands the session to the `reporter` agent, which answers with a struct, and takes the session back. | [`structured-output`](../structured-output), [`session-control`](../session-control) |
| Usage and cost | `SharedUsageTrackStrategy` adds each turn to a total. The model has `pricing` in `models.toml`. | [`providers`](../providers) |

## Organization

```text
configs/             the agents, models, tools and MCP server, in TOML
skills/note-style/   a skill the assistant can load
src/main.rs          the command loop, compaction, and the report
src/turn.rs          one turn: stepping, approval, retries
src/tools/           the two tools, each in its own module
```

## Choices

- **One provider.** The assistant uses Claude, chosen when the program is compiled.
  Stepping through a turn by hand and handing a session to another agent need a concrete model type, which `Models::any` does not give.
  To use another provider, change the model in `main.rs` and the `AssistantSession` type in `agents.rs`.
- **Compaction by message count.** `InstructionCompactor` asks the model to summarize the older messages, and works with any provider.
  The native compaction strategies need extra betas and token counting.
  Compaction and moving the session to another provider do not mix: a session with a summary cannot be migrated, which is why there is no model switching here.
- **Only `read_file` asks for approval.** The list is `NEEDS_APPROVAL` in `tools.rs`. Taking a note is harmless.
- **The tools are tested without a model.** `cargo test` checks that `read_file` stays inside the project directory and that `take_note` appends.

## Extending it

- Add a tool: a new table in `tools.toml`, a module under `src/tools/`, and an entry in the agent's `tools`.
- Add a skill: a directory under `skills/` with a `SKILL.md`, and its name in the agent's `skills`.
- Change what is asked before running: edit `NEEDS_APPROVAL`, or replace `approved` in `turn.rs`.
