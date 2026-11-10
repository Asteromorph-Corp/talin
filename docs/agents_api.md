# Agents API

> This guide covers how to use the agents you defined in code.
> To learn how to define and configure agents, see the [agents guide](/docs/agents.md).

## Anatomy of a Generated Agent

For an agent named `guesser`, [`include_agents!`](/docs/agents.md#step-3-import-the-agents) generates:

- `[AgentName]Agent`: a unit struct that acts as a factory for the agent.
  Its associated functions build the runtime agent objects based on your TOML configuration.
- `[AgentName]AgentInput`: the input struct.
  The [input variables](/docs/agents.md#input-variables-in-prompts) you declared in `agents.toml` become fields on this struct.
- `[AgentName]AgentExt` / `[AgentName]AgentSessionExt`: extension traits with the convenience methods (see [Convenience Methods](#convenience-methods)).

The two runtime types you will work with can be found in `talin::agent`:

| Type | What it is | How to get one |
| --- | --- | --- |
| [`Agent`](#the-agent) | A stateless configuration(prompt, LLM, tools, options). | `[AgentName]Agent::new(..)` / `[AgentName]Agent::default()` |
| [`AgentSession`](#sessions) | A stateful conversation that owns a transcript. | `[AgentName]Agent::session(..)` or `agent.session()` |

## Constructors

```rust
use crate::agents::{GuesserAgent, Models};

// 1. With an explicit model.
let agent = GuesserAgent::new(Models::gpt_4o_mini()).await;

// 2. With the default model declared in `agents.toml`.
let agent = GuesserAgent::default().await;

// 3. Open a session skipping the intermediate agent object.
let mut session = GuesserAgent::session(Models::gpt_4o_mini()).await;
```

- `new<P>(profile: P) -> Agent<..>`

  Builds an agent with the given profile.
  The `profile` carries the model and default strategy for the agent.
  Typically, you would pass an `LlmPreset` returned by the [`Models::*` functions](/docs/models.md) as this argument.

- `default() -> Agent<..>`

  Builds an agent using the default model declared in `agents.toml`. 
  Equivalent to `[AgentName]Agent::new([AgentName]Agent::default_model())`.

- `session<P>(profile: P) -> AgentSession<..>`

  Opens a session skipping the intermediate agent object.
  Equivalent to `[AgentName]Agent::new(profile).await.session()`.

- `default_model() -> LlmPreset`

  Returns the default model for the agent.
  Useful if you want to inspect or tweak the preset before passing it to `new`.


## The `Agent`

An `Agent` is a stateless bundle of configuration.
Eacn call to it spawns a single session without retaining any history from previous calls.

> ⚠️ **Changed in v0.4.** Agents no longer remember previous turns across calls.
> If you need conversation history, open a [session](#sessions).

### Running

- `run(&self, input: AgentInput) -> Result<Output, AgentError>`

  Spawns a one-shot session, runs a single turn to completion without additional strategy, and returns the output.
  `Output` is `String` unless the agent uses [structured output](/docs/agents.md#structured-output).

  ```rust
  let result = agent.run(GuesserAgentInput {}).await?;
  ```

- `run_with_strategy(&self, strategy, input) -> Result<AgentOutput<Output, S::Output>, AgentError>`

  Like `run`, but with a custom [`Strategy`].
  Returns both the agent output and the strategy's output (e.g. usage tracking) in an [`AgentOutput`].

### Prompt Inspection

- `get_prompt(&self, input: AgentInput) -> Result<Prompt, TemplateError>`

  Returns the prompt formatted with `input`, without calling the model.
  Useful for debugging or auditing.
  Requires the trait `talin::chain::GetPrompt`.

## Sessions

A session is a persistent conversation that owns a [transcript](#editing-the-transcript). 
Unlike `Agent`s, a session carries the full message history forward across turns.

```rust
// Open a session, either from a constructed agent...
let mut session = agent.session();
// ...or directly from the factory.
let mut session = GuesserAgent::session(Models::gpt_4o_mini()).await;
```

Running a session requires a mutable reference to the session, since the transcript is updated in-place as the conversation progresses.

A session also exposes its moving parts as public fields, all of which are editable during and in-between turns.
This includes the model (`llm`), list of available tools (`tools`, `builtin_tools`), call options (`options`), and the conversation transcript (`transcript`).

### Turns and Steps

Two concepts structure a session:

- A **turn** is one round of interaction.
  A turn consists of a single user input, followed by zero or more steps, followed by the model's final output.
- A **step** is a single model call within a turn.
  Typically, a step consists of a batch of tool calls paired with their results.
  A turn runs as many steps as the model needs before producing its final answer.

You can either run the entire turn at once or work through it step by step.

#### ⏩ Running a turn to completion

- `run_turn(&mut self, input) -> Result<Output, AgentError>`

  Runs one turn to completion, returning the final output.

  ```rust
  let result = session.run_turn(GuesserAgentInput {}).await?;
  ```

- `run_turn_with_strategy(&mut self, strategy, input) -> Result<AgentOutput<..>, AgentError>`

  As above, but with a custom [`Strategy`], returning an [`AgentOutput`].

#### Step through a turn manually

`turn` begins a turn and returns an [`AgentRunner`].
You can inspect and edit the `AgentRunner` between turns for detailed control over the turn's execution.

- `turn(&mut self, strategy, input) -> Result<AgentRunner<..>, AgentError>`

  Begins a turn and returns the runner.
  Pass `()` for the default strategy.

On the returned `AgentRunner`:

- `step(&mut self) -> Result<Option<Output::Partial>, AgentError>`

  Advances the turn by one step.
  Returns `Some(partial)` when the model produces its final answer, or `None` when it took a tool-calling step and the turn should continue.

- `half_step(&mut self) -> Result<PendingStep<..>, AgentError>`

  Advances the turn by "half a step".
  This executes the model-side of the step but not the tool-calling side.
  You can call `complete()` on the returned `PendingStep` to handle tool calls with the defined tools, `complete_with(results)` to supply your own tool results, or `discard()` to revert to the previous state and try again.

- `finalize(self, partial) -> Result<AgentOutput<..>, AgentError>`

  Converts a final partial output (from `step`) into the turn's final output, merging in fields from the input.

- `run_to_finish(self) -> Result<AgentOutput<..>, AgentError>`

  Advances the turn until it finishes.

Use the following pattern to step through a turn manually and modify the session between steps:

```rust
let mut turn = session.turn((), GuesserAgentInput {}).await?;
let result = loop {
    if let Some(partial) = turn.step().await? {
        break partial;
    }
    // Modify the session between steps.
};
let result = turn.finalize(result).await?;
println!("{:#?}", result.content);
```

### Editing the Transcript

`session.transcript` is an `AgentTranscript`: the conversation state exchanged with the LLM.
It can be edited between steps or from a [strategy](/docs/advanced/strategy.md)'s transcript hooks.
The transcript modification is performed via view → edit → modify cycle, or through the reusable edit schemes built on top of it.

See the [transcript editing guide](/docs/advanced/transcript.md) for the full API.

### Continuing with a Different Agent

A session's transcript can be handed off to another agent.
This lets you switch the system prompt, tools, model, or output type mid-conversation while preserving conversation history.

- `agent.continue_session(prev_session) -> AgentSession<..>`

  Continues `prev_session` under `agent` with the same model provider.
  Adopts the new agent's prompt, tools, options, and output type.

  Equivalently, `prev_session.continue_with(&agent)` is available as a session method.

- `agent.continue_session_migrated(prev_session) -> Result<AgentSession<..>, UnsupportedContent>`

  Continues `prev_session` under `agent` with a different LLM provider.
  Migrates the transcript to the new provider's message format.

  Equivalently, `prev_session.continue_with_migrated(&agent)` is available as a session method.

  > ⚠️ Migration is lossy.
  > It drops any provider-specific fields that can't be expressed in the canonical message.
  > Furthermore, message/tool-call id formats may differ between providers, and you may need to edit the transcript afterward to fix inconsistencies.
  > The conversion may also fail outright if the transcript includes multimodal content (images, audio, etc.) that the new provider doesn't support.

```rust
// Start on GPT-4o-mini, then migrate the same conversation to Claude.
let mut session = GuesserAgent::session(Models::gpt_4o_mini()).await;
let result1 = session.run_turn(GuesserAgentInput {}).await?;

let agent2 = GuesserAgent::new(Models::claude_haiku_45()).await;
let mut session = agent2.continue_session_migrated(session)?;
let result2 = session.run_turn(GuesserAgentInput {}).await?;

// Hand the conversation to a different agent (same provider) for a summary.
let summarizer = SummarizerAgent::new(Models::claude_haiku_45()).await;
let mut session = summarizer.continue_session(session);
let summary = session.run_turn(SummarizerAgentInput {}).await?;
```

See the [session-control example](/examples/session-control) for the full structure.

## Convenience Methods

The generated extension traits (`[AgentName]AgentExt` for an `Agent`, `[AgentName]AgentSessionExt` for an `AgentSession`) add `*_immed` variants that take each input field as a positional argument instead of the input struct.
The traits are implemented identically for both types, so the same call site works whether you hold an agent or a session.

- `run_immed(&self, field1, field2, ...) -> Result<Output, AgentError>`

  Shortcut for `run` (on an agent) / `run_turn` (on a session) with direct arguments.

- `run_with_strategy_immed(&self, strategy, field1, field2, ...)`

  Shortcut for `run_with_strategy` / `run_turn_with_strategy` with direct arguments.

- `get_prompt_immed(&self, field1, field2, ...) -> Result<Prompt, TemplateError>`

  Shortcut for `get_prompt` with direct arguments.

## Choosing a Model at Runtime

Each backend has its own agent type (`Agent<OpenAIResponses, ..>`, `Agent<ClaudeMessages, ..>`), so the arms of a runtime `match` can't return them directly.
`AnyAgent` and `AnyAgentSession` are enums over the built-in backends that erase the model type.

Look up a preset by name with `Models::any`, then build the agent with the generated `[AgentName]::any`:

```rust
let preset = Models::any("gpt_4o_mini")?;
let agent = GuesserAgent::any(preset).await;
let result = agent.run_immed().await?;
```

`Models::any` accepts the model names in `Models::NAMES` (the `models.toml` ids in snake case).
For an unknown name it returns an `UnknownModel` error that lists the valid ones.

An `AnyPreset` is `Clone`, so one preset can be reused across agents and requests.
You can also convert a preset into `AnyPreset` (`AnyPreset::from(Models::gpt_4o_mini())`) or a concrete agent into `AnyAgent` (`AnyAgent::from(GuesserAgent::new(..).await.into())`) yourself.

The generated [extension traits](#convenience-methods) work on `AnyAgent` / `AnyAgentSession` too.
`AnyAgent` also has `run`, `run_with_strategy`, `session`, and `id` directly.

`session.continue_with(&agent)` / `session.continue_migrated_with(&agent)` is also available on `AnyAgent` / `AnyAgentSession`.
Like the methods on concrete agents, `continue_with` requires the same backend, while `continue_migrated_with` allows switching backends.
Notably, the same backend check in `continue_with` is performed at runtime for `AnyAgent` / `AnyAgentSession`, and the method fails with a `BackendMismatch` error if the backends don't match.
Also for `AnyAgent` / `AnyAgentSession` only, `continue_migrated_with` behaves exactly like `continue_with` when the backends match.

A strategy passed to `run_with_strategy*` must work with every backend (`UniversalStrategy` is automatically implemented for all strategies that do).

## Output Types

By default, an agent's output is a `String`.
When the agent declares a [`response_format`](/docs/agents.md#structured-output), the output struct is generated based on the schema.
The model's response is deserialized into this struct, and the struct is returned by the agent methods instead of a raw string.

Strategy-bearing methods (`run_with_strategy`, `run_turn_with_strategy`, `finalize`, `run_to_finish`) return an [`AgentOutput`].
The struct has two fields: `content` (the agent's output) and `extra_content` (the strategy's output, e.g. usage tracking info).
When the agent has no strategy, `strategy_output` is `()`.

[`Strategy`]: /docs/advanced/strategy.md
[`AgentOutput`]: #-output-types
[`AgentRunner`]: #-step-through-a-turn-manually
