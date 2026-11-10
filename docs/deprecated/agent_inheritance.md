# Agent Inheritance

If you have multiple agents that share common properties, you can define a single parent agent for the shared properties, and extend it in child agents.

Child agents inherit all properties from the parent agent. Any properties defined in a child agent will override the corresponding property from its parent.

This can be useful when you need agents that have access to built-in tools from different providers, as each agent needs to be defined separately.

```toml
[search_agent]
system_prompt = "You are a research agent, aiding users with their queries."
user_prompt = "Research about {{topic}} and give me a detailed summary."
model = "gpt-4o"
call_options.tool_choice = "required"
call_options.temperature = 0.7

[search_agent_child]
extends = "search_agent"
model = "gpt-4o-mini"
tools = ["builtin.openai_responses/web_search"]

[claude_search_agent]
extends = "search_agent"
model = "claude_haiku_45"
tools = ["builtin.claude_messages/web_search"]
```

## Abstract Agents

Abstract agents are agents that only serves as a template for other agents. They cannot be instantiated directly and can only be extended by other agents.

Abstract agents can omit required properties. However, all required properties must be resolved in the concrete child agents.

```toml
[search_agent]
abstract = true
# system prompt may be omitted in abstract agents
user_prompt = "Research about {{topic}} and give me a detailed summary."

call_options.tool_choice = "required"
call_options.temperature = 0.7

[search_agent_child]
extends = "search_agent"
system_prompt = "You are a research agent, aiding users with their queries."
model = "gpt-4o-mini"
tools = ["builtin.openai_responses/web_search"]

[claude_search_agent]
extends = "search_agent"
system_prompt = "You are Claude, a research agent aiding users with their queries."
model = "claude_haiku_45"
tools = ["builtin.claude_messages/web_search"]
```
