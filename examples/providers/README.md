# `providers`

This example runs an agent against any of the supported LLM providers.
The agent and the code that runs it are the same for all of them. Only the model entry in `configs/models.toml` changes.

```sh
OPENAI_API_KEY=...    cargo run -- openai_responses
ANTHROPIC_API_KEY=... cargo run -- claude_messages
GEMINI_API_KEY=...    cargo run -- gemini_interactions
XAI_API_KEY=...       cargo run -- xai_responses
                      cargo run -- generic_chat           # a local server, see below
```

The agent is given the tool `current_time`, and asked what date today is.
The agent are expected to call the tool and return the answer, which is then printed to the console along with the token usage information.

## Model selection

`include_models!()` generates a `Models::<model_name>` constructor for each entry in `models.toml`.
If you use this to initialize the model directly (e.g. `AssistantAgent::new(Models::claude_messages())`), the agent keeps the provider type as a generic parameter `Agent::<ClaudeMessages>`.
This can be useful if you would like to perform operations specific to the provider.

Alternatively, you can use `Models::any(name)` to pick a model by its name at run time, and `AssistantAgent::any(model)` to create the agent using it.
This lets you write code that is agnostic to the provider type:

```rust
let model = Models::any(&name)?;
let agent = AssistantAgent::any(model).await;
```

## Provider

The `model_type` of a model entry chooses the API it speaks:

| `model_type` | API | Notes |
| --- | --- | --- |
| `openai_responses` | OpenAI Responses | |
| `openai_chat` | OpenAI chat completions | Has options for OpenAI-compatible servers: `flavor = "unstrict"`, `flavor = "final_answer_tool"`. |
| `claude_messages` | Anthropic Messages | Also runs on Vertex AI with `backend = "vertex"`. |
| `gemini_interactions` | Gemini Interactions | |
| `gemini_generate_content` | Gemini generateContent | |
| `xai_responses` | xAI Responses | |
| `generic_chat` | OpenAI chat completions, without extras | The default. For Ollama, vLLM, OpenRouter and similar servers. Tool instructions go into the system prompt. |
| `sglang_generate` | SGLang `/generate` | |

[Models](/docs/models.md) describes the options of each type.

## Cost

A model with a `pricing` table has its cost reported next to the token counts.
`openai_responses` in `models.toml` has one, so its usage includes a cost, and the others do not.
