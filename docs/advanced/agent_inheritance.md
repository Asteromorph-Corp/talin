# Agent Inheritance

If you have multiple agents that share common properties, you can define a single parent agent for the shared properties, and extend it in child agents.

Child agents inherit all properties from the parent agent. Any properties defined in a child agent will override the corresponding property from its parent.

This is useful when several agents differ only in a few properties, such as the model or the user prompt.

```toml
[researcher]
system_prompt = "You are a research agent, aiding users with their queries."
user_prompt = "Research about {{topic}} and give me a detailed summary."
model = "gpt-4o"
call_options.tool_choice = "required"
call_options.temperature = 0.7

# Same agent, but cheaper and less random.
[researcher_mini]
extends = "researcher"
model = "gpt-4o-mini"
call_options.temperature = 0.2

# Same prompts, but a different task.
[researcher_brief]
extends = "researcher"
user_prompt = "Research about {{topic}} and answer in three sentences."
```

> [!WARNING]
> Avoid deep inheritance chains, as they make it harder to track where fields are defined and inherited. Although there is no strict limit on inheritance depth, aim for a single level whenever possible and avoid going over two levels deep.

## Abstract Agents

An abstract agent only serves as a template for other agents. Setting `abstract = true` skips code generation for it, so no agent type is generated and it cannot be used directly. It can only be extended by other agents.

This keeps your generated code free of agents that are never meant to be run.

```toml
[base_researcher]
abstract = true
system_prompt = "You are a research agent, aiding users with their queries."
call_options.tool_choice = "required"
call_options.temperature = 0.7

[summarizer]
extends = "base_researcher"
user_prompt = "Research about {{topic}} and give me a detailed summary."
model = "gpt-4o-mini"

[fact_checker]
extends = "base_researcher"
user_prompt = "Verify the following claim: {{claim}}"
model = "claude_haiku_45"
```
