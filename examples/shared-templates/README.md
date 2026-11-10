# `shared-templates`

This example shows how to define **multiple agents with similar prompt structures** using *shared prompts* as described [here](/docs/agents.md#-shared-prompts).

Usually, shared prompts are used to extract a common section of a prompt, but in this example, we will show how to extract the entire prompt structure.

## Motivation

When defining multiple agents that differ only in a few attributes, writing full prompts for each agent quickly becomes verbose and hard to maintain.

For example, suppose we want to define two agents:

```toml
# agents.toml

[mit_cs]
system_prompt = "You are computer science researcher from MIT."
user_prompt = "As a computer science researcher, answer the student's question: {{question}}"

[stanford_physics]
system_prompt = "You are physics researcher from Stanford."
user_prompt = "As a physics researcher, answer the student's question: {{question}}"
```

As the number of agents grow, the prompts become repetitive and hard to maintain.
If we want to change the structure of the prompt, we would have to update each agent individually.

While we can extract each section of the prompt into a shared prompt (`system_header = "You are "`, `system_body = " researcher from "`, `system_footer = "."`), this makes the prompt structure harder to read and maintain.

## Extracting Shared Prompt Structure

The example instead extracts entire structures of the prompts into reusable shared prompts.

```toml
# prompts.toml

researcher_description = "You are {{config.field}} researcher from {{config.university}}."
researcher_instruction = "As a {{config.field}} researcher, answer the student's question: {{question}}"
```

## Agent Configuration with Prompt Variables

Each agent can now reference the shared prompts and only supply the values that differ.

```toml
# agents.toml

[mit_cs]
system_prompt = "{{config.researcher_description}}"
user_prompt = "{{config.researcher_instruction}}"

[mit_cs.prompts]
field = "computer science"
university = "MIT"

[stanford_physics]
system_prompt = "{{config.researcher_description}}"
user_prompt = "{{config.researcher_instruction}}"

[stanford_physics.prompts]
field = "physics"
university = "Stanford"
```

With this setup, `mit_cs` agent resolves to:

```text
System prompt:
"You are computer science researcher from MIT."

User prompt:
"As a computer science researcher, answer the student's question: {{question}}"
```

Notably, `{{question}}` is a placeholder for input variable, and not a reference to a shared prompt.
Therefore, it remains as-is in the final prompt. and can be filled in at runtime.
