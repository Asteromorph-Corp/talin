# Configuration File

The talin framework uses TOML configuration files to generate necessary codes. Follow the guidelines below to organize and create your configuration files.

## Folder Structure

Create a `configs/` directory at the root level of your project to store configuration files (outside `src/` directory)

```
project-root/
├── configs/ # Configuration files go here
└── src/
    └── main.rs
```

### Configuration Options

You can choose between two structures for your configuration files:

1. **Monolithic Files**

```
configs/
├── agents.toml
├── tools.toml
├── models.toml
└── prompts.toml
```

2. **Modular Files**

```
configs/
├── agents/
│   ├── planner.toml
│   └── executor.toml
├── tools/
│   ├── embedder.toml
│   └── search.toml
├── models/
│   ├── openai.toml
│   └── anthropic.toml
└── prompts.toml
```

The format of the configuration files is the same for both monolithic and modular structures.

Generally, the modular file structure is recommended for larger projects with many agents, tools, or models.
For smaller projects, the monolithic structure may be more convenient.

## Configuration TOMLs

There are four types of configuration files:
- **Agent TOML**: Defines the agents, their prompts, and the tools they can use.
- **Tool TOML**: Defines the tools, their input parameters, and their descriptions.
- **Model TOML**: Defines the models, their API endpoints, and their authentication methods
- **Prompt TOML**: Defines the prompts shared by agents

The configuration files are written in TOML format.
Below are minimal examples for each configuration file.
For a more detailed specification, refer to the respective documentations.

### Agent TOML

For a more detailed specification, refer to the [agents documentation](/docs/agents.md)

```toml
[research_assistant]
system_prompt = "After years of research experience, you have developed a keen eye for..."
user_prompt = "Evaluate the research proposal: {{proposal}}"
tools = ["google_search"]

[research_assistant.model]
api_base = "https://api.openai.com/v1"
api_key.env = "OPENAI_API_KEY"
model = "gpt-4o-mini"
model_type = "openai_chat"
```

### Tool TOML

For a more detailed specification, refer to the [tools documentation](/docs/tools.md)

```toml
[addition]
name = "add_numbers"
description = "Add two numbers"

[addition.input]
num1 = { type = "i64", description = "The first number to add" }
num2 = { type = "i64", description = "The second number to add" }
```

### Model TOML

For a more detailed specification, refer to the [models documentation](/docs/models.md)

```toml
[openai_gpt4]
api_base = "https://api.openai.com/v1"
api_key.env = "OPENAI_API_KEY"
model = "gpt-4o-mini"
model_type = "openai_responses"
```

### Prompt TOML

There isn't a dedicated page for the Prompt TOML specification, as it follows a simple `key = "value"` format.

```toml
welcome_message = "Hello, I am {{config.name}}! How can I help you today?"
farewell_message = "Goodbye! Have a great day."
```

To learn how to use prompts defined in `prompts.toml`, refer to the [shared templates example](/examples/shared-templates).
