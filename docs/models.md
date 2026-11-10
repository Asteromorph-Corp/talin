# LLMs

LLMs are the engines powering your [agents](/docs/agents.md).

## Quickstart

Models are generated based on the TOML configuration and can then be imported into your code using the `include_models!` macro.

### Step 1: Define Models in `models.toml`

Create `configs/models.toml` in your project root and define the models, the specific fields will be explained in the later sections.

> For the sake of simplicy, this document uses the monolithic TOML setup. However, projects with a larger number of models may distribute them across multiple TOML files. See [configuration options](/docs/configs.md#-configuration-options) for more information.

```toml
# models.toml
[gpt_4o_mini]
api_base = "https://api.openai.com/v1"
api_key.env = "OPENAI_API_KEY"
model = "gpt-4o-mini"
```

### Step 2: Configure Build Script

Call `build_artifacts` in `build.rs`, which will generate the model struct based on your TOML configuration.

```rust
// build.rs
use talin_build::build_artifacts;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    build_artifacts()?;
    Ok(())
}
```

> ⚠️ `build.rs` must be placed in your **project root** (alongside `Cargo.toml`) and not within `src`.

### Step 3: Import the Models

```rust
// src/agents.rs
talin::include_models!();
```

This will give you a unit-like struct named `Models`.

The `Models` struct has a method `Models::model_name()` for each model defined in `models.toml`.
With the above example, this method would be `Models::gpt_4o_mini()`.

### Step 4: Use the Models to Initialize Agents

Now you can use the model to initialize your agents.

```rust
use crate::agents::{SomeAgent, Models};

#[tokio::main]
async fn main() -> String {
    let agent = SomeAgent::new(Models::gpt4o_mini());
    agent.run(SomeAgentInput::new()).await
}
```

## Basic Configuration

For the most basic settings, provide the fields below:

```toml
# models.toml
[gpt4o_mini]
api_base = "https://api.openai.com/v1"
api_key.env = "OPENAI_API_KEY"
model = "gpt-4o-mini"
```

- `api_base`: The base URL for the model's API endpoint

  - You can either hardcode it or provide an environment variable name (e.g. `"LLM_API_BASE"`)

- `model`: The name of the model to use

- `api_key_env_var`: Name of the environment variable that stores your API key. (Optional)

- `headers`: Extra HTTP headers sent with every request, e.g. for LLM gateways. (Optional)

  - Values are inline strings, or `{ env = "VAR" }` to read an environment variable when the model is created.
  - Headers managed by the library (`authorization`, `x-api-key`, `x-goog-api-key`, `content-type`, `content-length`, `host`, `anthropic-version`) are rejected. Use `api_key` for authentication.

> 🔐 **Security Tip**: For self-hosted models, use environment variable for `api_base` instead of hardcoding the API endpoint directly.

## Model Type

Different LLM providers require distinct request/response formats. You can select which protocol to use by setting the `model_type` field.

```toml
# models.toml
[gpt_4o_mini]
api_base = "https://api.openai.com/v1"
api_key.env = "OPENAI_API_KEY"
model = "gpt-4o-mini"
model_type = "openai_responses"
```

### Available options for `model_type`:

Every `model_type` is named `<provider>_<api>`, after the provider and the specific API it speaks.

- `"generic_chat"` (Default): Uses the OpenAI chat completions API with no native support for advanced features.
  
  - Tool instructions are appended to the system prompt.
    
  - Output structure must be enforced manually in the system prompt.
  
  - Used when `model_type` is unspecified.

- `"openai_chat"`: Uses the OpenAI chat completions API with advanced features.

- `"openai_responses"`: Uses the OpenAI responses API.

- `"claude_messages"`: Uses the Claude messages API.
 
- `"gemini_generate_content"`: Uses the Gemini generateContent API.
- 
- `"gemini_interactions"`: Uses the Gemini Interactions API, the recommended surface for new
  Gemini applications.

- `"xai_responses"`: Uses the xAI responses API.

- `"sglang_generate"`: Uses the SGLang generate API.

### Modality Support

Which media kinds a model accepts depends on the API it speaks, so support is listed per `model_type`.
Supplying an unsupported kind fails before the request is sent.

> ⚠️ The table below is not extensively tested and may change over time.
> A kind listed as supported may still fail depending on the mime type. 
> (e.g. `gemini_interactions` support `.mp4` / `.mpeg` and `.webm` videos, but not `.ogg` videos)

#### As Input

| `model_type`              | text | image | audio | video | document |
| ------------------------- | ---- | ----- | ----- | ----- | -------- |
| `generic_chat`            | ✅   | ✅    | ✅    | ❌    | ✅       |
| `openai_chat`             | ✅   | ✅    | ✅    | ❌    | ✅       |
| `openai_responses`        | ✅   | ✅    | ❌    | ❌    | ✅       |
| `xai_responses`           | ✅   | ✅    | ❌    | ❌    | ✅       |
| `claude_messages`         | ✅   | ✅    | ❌    | ❌    | ✅       |
| `gemini_generate_content` | ✅   | ✅    | ✅    | ✅    | ✅       |
| `gemini_interactions`     | ✅   | ✅    | ✅    | ✅    | ✅       |
| `sglang_generate`         | ✅   | ❌    | ❌    | ❌    | ❌       |

#### As Tool Result

Tools may also return non-text content. See [tool output](/docs/tools.md#custom-tool-output) for how to build one.

| `model_type`              | text | image | audio | video | document |
| ------------------------- | ---- | ----- | ----- | ----- | -------- |
| `generic_chat`            | ✅   | ✅    | ✅    | ❌    | ✅       |
| `openai_chat`             | ✅   | ❌    | ❌    | ❌    | ❌       |
| `openai_responses`        | ✅   | ✅    | ❌    | ❌    | ✅       |
| `xai_responses`           | ✅   | ✅    | ❌    | ❌    | ✅       |
| `claude_messages`         | ✅   | ✅    | ❌    | ❌    | ✅       |
| `gemini_generate_content` | ✅   | ⚠️    | ⚠️    | ⚠️    | ⚠️       |
| `gemini_interactions`     | ✅   | ✅    | ❌    | ❌    | ❌       |
| `sglang_generate`         | ✅   | ❌    | ❌    | ❌    | ❌       |

`gemini_generate_content` accept an arbitrary JSON as a tool result, so the serialized `Vec<ContentPart>` is sent as-is.
The request will succeed, but the model is very likely to not understand the serialized attachment.

### `model_type`-specific Configurations

On top of the basic fields, each `model_type` accepts its own set of options. Options that the
selected `model_type` does not recognize are rejected at build time.

#### `generic_chat`

```toml
# models.toml
[gemma3]
api_base = "https://generativelanguage.googleapis.com/v1beta/models"
model = "gemma-3-4b-it"
flavor = ["qwen3", "system_is_assistant"]
drop_thought = false
pricing = { input_tokens = 0.1, output_tokens = 0.4 }
```

- `flavor`: Behavioral tweaks applied to the requests. Accepts a single string or an array of them.

  - `"qwen3"`: Enables the Qwen3 instructor, which coaxes structured output out of Qwen3 models.

  - `"system_is_assistant"`: Treats the system message as an assistant message. Equivalent to
    `system_is_assistant = true`, see [below](#-system-message-as-assistant-message).

  - `"none"`: No flavor.

  - Default: `[]`

- `drop_thought`: See [Preserving the 'thought' part of LLM responses](#-preserving-the-thought-part-of-llm-responses).

- `pricing`: See [Pricing](#-pricing).

#### `openai_chat`

```toml
# models.toml
[gpt_4o_mini]
api_base = "https://api.openai.com/v1"
api_key.env = "OPENAI_API_KEY"
model = "gpt-4o-mini"
flavor = "unstrict"
drop_thought = false
pricing = { input_tokens = 0.15, cached_input_tokens = 0.075, output_tokens = 0.6 }
```

- `flavor`: Behavioral tweaks applied to the requests. Accepts a single string or an array of them.

  - `"unstrict"`: Removes the `strict` flag from tool definitions, for OpenAI-compatible providers
    that reject strict structured outputs.

  - `"final_answer_tool"`: Makes the model submit its final answer as a tool call. Useful for models with tool call support but without structured output support.

  - `"none"`: No flavor.

  - Default: `[]`

- `drop_thought`: See [Preserving the 'thought' part of LLM responses](#-preserving-the-thought-part-of-llm-responses).

- `pricing`: See [Pricing](#-pricing).

#### `claude_messages`

```toml
# models.toml
[claude_opus_4_5]
api_base = "https://api.anthropic.com/v1"
api_key.env = "ANTHROPIC_API_KEY"
model = "claude-opus-4-5"
backend = "anthropic"
anthropic_version = "2023-06-01"
anthropic_beta = ["context-1m-2025-08-07"]
```

- `backend`: Which service hosts the Claude messages API.

  - `"anthropic"`: The Anthropic API.

  - `"vertex"`: Vertex AI. Requires `project_id`, and accepts an optional `location`. Credentials
    are resolved lazily from the ambient Google Cloud environment.

  - Default: `"anthropic"`

- `anthropic_version`: The value sent in the `anthropic-version` header.

- `anthropic_beta`: A list of beta feature names sent in the `anthropic-beta` header.

- `pricing`: See [Pricing](#-pricing).

For the Vertex backend, point `api_base` at the Vertex endpoint and supply the project:

```toml
# models.toml
[claude_opus_4_5_vertex]
model = "claude-opus-4-5"
backend = "vertex"
project_id.env = "GOOGLE_CLOUD_PROJECT"
location = "us-east5"
```

- `project_id`: The Google Cloud project ID. Hardcode it or provide an environment variable name
  with `project_id.env`.

- `location`: The Vertex AI region. Hardcode it or provide an environment variable name with
  `location.env`.

#### `openai_responses`, `gemini_generate_content`, `gemini_interactions`, `xai_responses`

These types take no options beyond the basic fields and `pricing`. See [Pricing](#-pricing).

#### `sglang_generate`

```toml
# models.toml
[qwen3_local]
api_base.env = "SGLANG_API_BASE"
model = "Qwen3-32B"
drop_thought = false
```

- `drop_thought`: See [Preserving the 'thought' part of LLM responses](#-preserving-the-thought-part-of-llm-responses).

- `pricing`: See [Pricing](#-pricing).

#### System Message as Assistant Message

Some models (e.g. DeepSeek-R1) do not support system message. For these models, set
`system_is_assistant = true` in your configuration.

```toml
# models.toml
[gemma3]
api_base = "https://generativelanguage.googleapis.com/v1beta/models"
model = "gemma-3-4b-it"
system_is_assistant = true
```

With this option enabled, system prompts in your `agents.toml` is treated as assistant messages.

> ✅ Tip: Make sure to write prompts for these models using first-person pronouns.
>
> 🔁 Instead of: `"You are an helpful assistant, ..."`
>
> ✅ Use: `"I am an helpful assistant, ..."`

> ℹ️ Available for `model_type = "generic_chat"` only.

#### Preserving the 'thought' part of LLM responses.

For models without native support for reasoning, the model's reasoning are emitted as part of the response. For example,

```
The user asked me to do X, so I will do Y.
</think>
{
  "name": "some_tool",
  "arguments": {
    "arg1": "value1",
    "arg2": "value2"
  }
}
```

By default, these "thought" portions of an LLM response is dropped for subsequent requests.
However, if you want to preserve the thought process along with the tool call, you can set `drop_thought = false` in your model configuration.

```toml
# models.toml
[gemma3]
api_base = "https://generativelanguage.googleapis.com/v1beta/models"
model = "gemma-3-4b-it"
drop_thought = false
```

> ℹ️ Available for `model_type = "generic_chat"`, `"openai_chat"` and `"sglang_generate"`.
> The other types preserve the reasoning natively.

#### Pricing

Every `model_type` accepts a `pricing` table, which is used to report the cost of a run. All prices
are in USD per million tokens unless noted otherwise. When it is omitted, no cost is reported.

Most types share the same shape:

```toml
# models.toml
[gpt_4o_mini]
api_base = "https://api.openai.com/v1"
api_key.env = "OPENAI_API_KEY"
model = "gpt-4o-mini"

[gpt_4o_mini.pricing]
input_tokens = 0.15
cached_input_tokens = 0.075 # Optional, defaults to `input_tokens`
output_tokens = 0.6
```

`claude_messages` prices the prompt cache instead of a single cached input rate:

```toml
[claude_opus_4_5.pricing]
input_tokens = 5.0
cache_writes_5m = 6.25       # Optional, defaults to 1.25x `input_tokens`
cache_writes_1h = 10.0       # Optional, defaults to 2x `input_tokens`
cache_hits_refreshes = 0.5   # Optional, defaults to 0.1x `input_tokens`
output_tokens = 25.0
web_search = 0.01            # Optional, USD per web search request
```

`gemini_interactions` prices each input modality separately:

```toml
[gemini_3_pro.pricing]
input_tokens = 2.0
cached_input_tokens = 0.2           # Optional, defaults to `input_tokens`
image_input_tokens = 2.0            # Optional, defaults to `input_tokens`
cached_image_input_tokens = 0.2     # Optional, defaults to `image_input_tokens`
video_input_tokens = 2.0            # Optional, defaults to `input_tokens`
cached_video_input_tokens = 0.2     # Optional, defaults to `video_input_tokens`
audio_input_tokens = 4.0            # Optional, defaults to `input_tokens`
cached_audio_input_tokens = 0.4     # Optional, defaults to `audio_input_tokens`
output_tokens = 12.0                # Also used for thinking tokens
```

## Call Options

You can fine-tune the behavior of the LLM by specifying call options. The options are converted to the appropriate fields based on the model provider. A warning log is printed when an option is not supported by the selected model.

The options below apply to the model they are defined on. These can also be defined at `agents.toml`, which applies the options to all models used by the agent. See [`call options`](/docs/agents.md#call-options).

```toml
# models.toml
[<model_name>]
# ... Basic configuration fields ...
stream = true
tool_choice = "required"
parallel_tool_calls = true
n = 2
reasoning_effort = "high"

temperature = 0.0
top_p = 0.2
top_k = 3

frequency_penalty = 1.2
presence_penalty = 1.3
repetition_penalty = 0.8

max_completion_tokens = 1024
stop_words = ["Bird"]

extra_params.<param_name> = <value>
```

- [`stream`](https://platform.openai.com/docs/api-reference/chat/create#chat-create-stream): If set to true, the model response data will be streamed to the client as it is generated using server-sent events.

  - Default: `false`

- [`tool_choice`](https://platform.openai.com/docs/api-reference/chat/create#chat-create-tool_choice): Controls which (if any) tool is called by the model.

  - `tool_choice = "none"`: The model will not call any tool and instead generates a message.
  
  - `tool_choice = "auto"`: The model can pick between generating a message or calling one or more tools.
  
  - `tool_choice = "required"`: The model must call one or more tools.
  
  - `tool_choice = "builtin_tool_name"`: The model must call the provider's built-in tool. The available built-in tool depends on the model provider.

  - `tool_choice.function = "function_name"`: The model must call a specific function.

  - Default: `"auto"` if tools, `"none"` if no tools.

- [`parallel_tool_calls`](https://platform.openai.com/docs/api-reference/chat/create#chat-create-parallel_tool_calls): Whether to enable parallel function calling during tool use. Only supported for models with native support for structured tool call (`tool_call_type = "OpenAI"`)

  - Default: `true`

- [`reasoning_effort`](https://platform.openai.com/docs/api-reference/chat/create#chat-create-reasoning_effort): Constrains effort on reasoning for reasoning models. Only supported for certain models (gpt-5 and o-series models only).

  - Supported values: `"none"` `"minimal"`, `"low"`, `"medium"`, `"high"`.

  - Default: `"none"`

- [`temperature`](https://platform.openai.com/docs/api-reference/chat/create#chat-create-temperature): Sampling temperature. Controls randomness.

  - Range: `0` (deterministic) to `2` (very random)

  - Default: `1`

- [`top_p`](https://platform.openai.com/docs/api-reference/chat/create#chat-create-top_p): Top-p (nucleus) sampling parameter. Chooses from the smallest possible set of tokens whose cumulative probability exceeds p.

  - Range: `0` (deterministic) -`1`.

  - Default: `1`.


- `top_k`: Top-k sampling. Chooses from the top k most likely tokens. Often used with or instead of top_p.

- [`frequency_penalty`](https://platform.openai.com/docs/api-reference/chat/create#chat-create-frequency_penalty): Positive values penalize new tokens based on their existing frequency in the text so far, decreasing the model's likelihood to repeat the same line verbatim.

  - Range: `-2.0` to `2.0`

  - Default: `0`

- [`presence_penalty`](https://platform.openai.com/docs/api-reference/chat/create#chat-create-presence_penalty): Positive values penalize new tokens based on whether they appear in the text so far, increasing the model's likelihood to talk about new topics.

  - Range: `-2.0` to `2.0`.

  - Default: `0`

- `repetition_penalty`: Positive values discourage the model from repeating the same line verbatim.

  - Range: `0.0` to `2.0`.


- [`max_completion_tokens`](https://platform.openai.com/docs/api-reference/chat/create#chat-create-max_completion_tokens): The maximum number of tokens to generate in the completion.

  - Default: `None`


- [`stop_words`](https://platform.openai.com/docs/api-reference/chat/create#chat-create-stop): Up to 4 sequences where the API will stop generating further tokens. The returned text will not contain the stop sequence. Not supported by latest reasoning models.

  - Default: `None`

- `extra_params`: A table of options that are not explicitly supported.

  - Any parameters placed under this table will be directly inserted into the request body sent to the LLM provider.

## Full `models.toml` Example

```toml
# models.toml
[model_name]
api_base = "https://api.openai.com/v1"
api_key.env = "OPENAI_API_KEY"
model = "gpt-4o-mini"

model_type = "openai_resposes" # Optional
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

# extra_params.reasoning.enabled = true # Optional
```
