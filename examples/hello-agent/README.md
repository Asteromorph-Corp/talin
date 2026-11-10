# `hello-agent`

The example shows how to use `talin` to define and run a simple agent.

```sh
OPENAI_API_KEY=... cargo run
```

The agent is defined with prompts to translate provided text into the specified language.
We run the agent to translate "Hello, world!" into French.

## Files

| File | Role |
| --- | --- |
| `configs/models.toml` | The model endpoint, API key variable, and API type. |
| `configs/agents.toml` | The agent's prompts and the model it uses. |
| `build.rs` | Calls `build_artifacts()`, which reads the files above and generates Rust. |
| `src/main.rs` | Includes the generated code and runs the agent. |

## Build Artifacts

For the `[translator]` agent, `build.rs` generates:

- `TranslatorAgent`: construction helper for the `[translator]` agent, imported using `include_agents!`.
- `Models`: construction helper for the models, imported using `include_models!`.

The structs themselves are unit-like structs, and implements methods to construct a `talin::agent::Agent` or `talin::llm::LlmPreset`.
`TranslatorAgent::default()` uses the model named in the agent's `model` field, but you can use different models with it using `TranslatorAgent::new(Models::gpt_4o_mini()).await`.

Every `{{placeholder}}` in the prompts becomes a typed parameter of the run methods, in alphabetical order.
For the translator agent, this gives `run_immed(language, text)`.
If you update the prompt to require a different set of placeholders, the calls in `main.rs` would stop compiling until you update it.

## Next

- Give the agent tools: [`function-tools`](../function-tools).
- Get a typed answer instead of a `String`: [`structured-output`](../structured-output).
