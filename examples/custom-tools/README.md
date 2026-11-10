# `custom-tools`

This example shows what you can do when writing your own function tools with the `#[tool]` macro.
For the basics of defining a tool, see [`function-tools`](../function-tools) and the [tools guide](/docs/tools.md).

```sh
OPENAI_API_KEY=... cargo run
```

A bookshop assistant checks the stock of two books and orders them.
One title in the request is misspelled ("Salt and Lantern" for "Salt and Lanterns").
The titles are made up, so the model cannot spot the typo itself: the tool call fails, and the model has to recover from the error.

## `#[tool]` Features and their Usage

| Feature | Where |
| --- | --- |
| A description built from the tool's own state | `check_stock`: `description()` extends `TOOL_DESCRIPTION` with the store name. |
| Per-agent configuration | `check_stock`: `config` in `tools.toml`, filled in by `shopper` and `uptown_shopper` in `agents.toml`, and read in `from_config`. |
| Errors the model can recover from | `check_stock`: an unknown title returns `Err` that lists the valid titles. |
| Several content parts and a summary | `place_order`: returns a `ToolOutput`. |
| A cap on calls | `place_order`: `available()` allows at most two calls in the transcript. |
| A tool with no configuration | `place_order`: still implements `from_config`, taking `()`. |
| Log summaries | `check_stock`: `summarize_input()`. |

## Errors

The run does not stop when `call` when the tool fails.
Instead, the error message is stored in the transcript 
In subsequent steps, the model can inspect the error message and try again with a revised call.
Write the message for the model: say what was wrong, and what would have worked.

`main.rs` records the run with `RecordStrategy` and prints every message, so you can see the failed call, the error, and the corrected call.

## One Module per Tool

When you have multiple tools, 
The code generated for a tool defines items such as `TOOL_NAME` and `TOOL_DESCRIPTION` in the module that contains the `#[tool]` impl, so two tools cannot share a module.
This is why `src/tools/` has a file per tool.

## Next

- Tools that live in another process: [`local-mcp`](../local-mcp).
