# Cheatsheet

This cheatsheet provides examples of how to configure agents, tools, and models using TOML files.

## Full `agents.toml` example

```toml
# agents.toml
[<agent_name>]
# Prompts
system_prompt = "System prompt" # Optional
user_prompt = "User prompt" # Optional

# Default model
model = "gpt_4o_mini"

# Attachments
attachments.images = ["images"]
attachments.videos = ["videos"]
attachments.audios = ["audios"]
attachments.documents = ["documents"]

# Agent Options
max_iterations = 10 # Optional
max_consecutive_fails = 5 # Optional
max_continuations = 0 # Optional

# Shorthand Tool Configuration
tools = [
    "tool_name",
    "mcp.server_name/tool_name"
    "builtin.provider_apiname/tool_name"
]

# Detailed Tool Configurations, mutually exclusive with shorthand above
[[<agent_name>.tools]]
kind = "local"
name = "tool_name"
config = { <param_name> = "value1", <param_name2> = "value2" } # Optional
[[<agent_name>.tools]]
kind = "mcp"
server = "server_name"
name = "tool_name"
[[<agent_name>.tools]]
kind = "builtin"
provider = "provider_apiname"
type = "tool_name"
<option_name> = <value> # Additional options specific to the tool

# Model call options applied to the agent.
[<agent_name>.call_options]
tool_choice = "required"

# Response format
[<agent_name>.response_format] # Optional
name = "schema_name"
description = "The schema description."
fields.<field_name> = { type = "u32", description = "The field description." }

# Agent-specific Config Variables
[<agent_name>.prompts]
<var_name> = "Some text"

[<agent_name>.if.model_type.<model_type>]
system_prompt = "{{base.system_prompt}}"
user_prompt = "{{base.user_prompt}}"
max_iterations = 10
max_consecutive_fails = 5
tools = [] # same format as [<agent_name>.tools]
prompts = {} # same format as [<agent_name>.prompts]
```

## Full `tools.toml` Example

```toml
# tools.toml
[<tool_name>]
name = "tool_name"
description = "Brief description of the tool"
default = false

[<tool_name>.input]
<field_name> = "i64"
<field_name2> = { type = "String", description = "Brief description of the field" }

[<tool_name>.config]
<param_name> = "u32"
<param_name2> = "String"
```

## Full `models.toml` Example

```toml
# models.toml
[<model_name>]
api_base = "https://api.openai.com/v1"
api_key.env = "OPENAI_API_KEY"
model = "gpt-4o-mini"

model_type = "openai_responses" # Optional
pricing = { input_tokens = 0.15, cached_input_tokens = 0.075, output_tokens = 0.6 } # Optional

stream = true # Optional
tool_choice = "required" # Optional
parallel_tool_calls = true # Optional
reasoning_effort = "high" # Optional

temperature = 0.7 # Optional
top_p = 0.9 # Optional
top_k = 5 # Optional

frequency_penalty = 0.5 # Optional
presence_penalty = 0.5 # Optional
repetition_penalty = 0.5 # Optional

max_completion_tokens = 1000 # Optional
stop_words = ["stop", "words"] # Optional
drop_thought = false # Optional

# extra_params.reasoning.enabled = true # Optional
```
