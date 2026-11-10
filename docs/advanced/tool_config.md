# Per-Agent Tool Configuration for Function Tools

This guide explains how to set up per-agent tool configuration for function tools.

Sometimes, you may want a tool to behave slightly differently depending on the agent using it — for example, by limiting the number of results (`max_results`) or changing the language used in a search (`lang`). Instead of creating multiple tools for each variation, you can configure tools per agent.

## Step 1: Define Tool Configuration in `tools.toml`

Use the `config` field to define configuration structure for the tool:

```toml
# tools.toml
[search]
name = "tool_name"
description = "Performs a web search based on user input"

config = { lang = "String", max_results = "u32" }
```

### Required fields

A field must be supplied by the agent unless its type says it can be left out:

- `Option<T>`, `Vec<T>`, maps, sets and `()` are **optional**.
- Everything else (`String`, numbers, `bool`, enums, `Duration`, your own types, ...) is
  **required**. If an agent lists the tool without supplying one (or without any `config`), the
  build fails with a compile error pointing at the tool.

Unset optional fields keep their `Default` value (`None`, empty), so use `Option<T>` for a field
with a default and apply the real default in `from_config` (e.g.
`config.lang.unwrap_or("en".into())`).

## Step 2: Implement the `from_config` Function

Implement the function `from_config` in your tool impl block.

```rs
// src/tools.rs
use talin::tool;

#[tool]
impl SearchTool {
    // Basic tool implementation...

    fn from_config(config: _) -> Self {
        Self {
            lang: config.lang,
            max_results: config.max_results,
        }
    }
}
```

## Step 3: Provide Tool Configuration in `agents.toml`

```toml
# agents.toml
[english_search_expert]
# other agent configuration...

[[english_search_expert.tools]]
name = "search"
config = { lang = "en", max_results = 10 }
```

Then, the agent `english_search_expert` will use the `search` tool with the specified configuration.

## Custom configuration structs

A custom config struct (`config = "custom"`) opts into the same check by deriving `ConfigSchema`.
Each field takes the schema of its type, so a field whose type also derives `ConfigSchema` may be
left out as a whole, but has its required fields checked when the agent supplies some of them:

```rs
use talin::tools::ConfigSchema;

#[derive(Default, Deserialize, ConfigSchema)]
struct SearchToolConfig {
    api_key: String,        // required
    lang: Option<String>,   // optional
    retry: RetryConfig,     // optional, but checked inside if supplied (derives `ConfigSchema`)
}
```

To change what a type requires, implement `ConfigSchema` for it by hand. A struct that does not
derive `ConfigSchema` is a required field. Custom configs that do not derive `ConfigSchema` are not
checked at all. The derive does not support field types that refer to the struct's generic
parameters.
