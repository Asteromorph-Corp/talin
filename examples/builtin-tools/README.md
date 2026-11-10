# `builtin-tools`

This example shows how to use the built-in tools provided by LLM backends.

## Basic Usage

Note that you must also specify the provider for the built-in tools.
Currently, the supported values for `provider` are `openai_responses`, `claude_messages`, `gemini_generate_content`, and `xai_responses`.

```toml
# agents.toml
[search_agent]
# Other agent configurations...

if.model_type.openai.tools = ["builtin.openai_responses/web_search"]
```

## Additional Parameters

Some built-in tools may require additional parameters. You can specify them in the `builtin_tools` section as follows:

```toml
# agents.toml
[search_agent]
# Other agent configurations...

[search_agent.if.model_type.openai]
[[tools]]
kind = "builtin"
provider = "openai_responses"
type = "web_search"
filters.allowed_domains = ["example.com"]
search_context_size = "low"
```

The specific parameter depends on the tool being used. Refer to the documentation of each provider for more details on the available parameters.

- [OpenAI Responses](https://platform.openai.com/docs/api-reference/responses/create#responses_create-tools)
- [Anthropic](https://platform.claude.com/docs/en/api/messages#tool_union)
- [Gemini](https://ai.google.dev/api/caching#Tool)
