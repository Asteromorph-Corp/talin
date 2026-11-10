# Strategy

`Strategy` is a trait that offers customization over an agent execution, and produce an output that is attached to the final agent output.
You can implement your custom strategy, or use one of the pre-implemented strategies.

## Usage

To run an agent with a strategy, use `.run_with_strategy()`:

```rust
use crate::agents::{SomeAgent, SomeAgentInput};
use crate::strategy::SomeStrategy;

#[tokio::main]
async fn main() {
    let agent = SomeAgent::default().await;

    let strategy = SomeStrategy::new();
    let input = SomeAgentInput::new();
    
    let output = agent.run_with_strategy(strategy, input).await.unwrap();
    
    println!("{}", output.content); // the final answer from the agent execution
    println!("{:?}", output.extra_content); // the output produced by the strategy
}
```

A strategy produces its output independently of the agent's output, and it is returned as `extra_content` in the final agent output.
The strategy's output is produced by `Strategy::finalize()` hook, and its type is defined by the `Output` associated type.
Strategies that have nothing to return use `type Output = ()`.

You can also apply multiple strategies to a single execution by passing a N-tuple of strategies to `.run_with_strategy()`.
Every hook is invoked on each strategy in tuple order, with each strategy receiving the value the previous one returned.
The outputs of all strategies are collected into a N-tuple as `extra_content` in the final agent output.

## Hooks

Every hook has a default no-op implementation, so you only implement the phases you care about.
The only required item is `finalize()`, along with the `Output` and `Error` associated types.

### Injection

| Method | Description |
| --- | --- |
| `additional_tools` | Extra function tools available during the run. |
| `additional_mcp_servers` | Extra MCP servers available during the run, added to the session's. A server with the same name as one of the session's replaces it. |
| `additional_builtin_tools` | Extra provider-specific built-in tools, appended to the session's. |
| `call_options` | The call options for the LLM, merged over the model's configured options. |

### Transcript hooks

| Method | Description |
| --- | --- |
| `edit_transcript` | Permanently edit the stored [transcript](/docs/advanced/transcript.md) before it is materialized. |
| `project_transcript` | Edit a throwaway clone of the transcript at materialize time. |

`edit_transcript` is the home for *idempotent* structural edits such as dropping a redundant tool call or merging messages.
The edit persists into the session transcript and is applied once per step.
`project_transcript` is the home for *ephemeral* edits that depend on live state, such as rendering placeholders against current values or conditionally hiding entries.

### Provider-specific (native) hooks

These operate on the provider's own types, so they can reach fields that the provider-agnostic layer erases at the cost of portability.

| Method | Description |
| --- | --- |
| `process_materialized` | Rewrite the materialized system prompt and items before the request is built. |
| `process_request` | Rewrite the provider-specific request before it is sent. |
| `process_response` | Rewrite the provider-specific response before it is lowered into the transcript. |

The `SystemOf`, `ItemOf`, `RequestOf`, and `ResponseOf` type aliases give you the provider-specific types for a given LLM.
A strategy that uses these hooks is typically implemented for one provider only, rather than for all `L: Llm`.

### Provider-agnostic hooks

| Method | Description |
| --- | --- |
| `process_initial_messages` | Inspect, validate, or rewrite the initial messages of a turn. |
| `inspect_usage` | Observe the token usage of a step. |
| `process_llm_step` | Inspect, validate, or rewrite the model-produced `LlmStep`. |
| `process_tool_result` | Inspect, validate, or rewrite a tool's `ToolResult`; you can turn a failure into a success, or vice versa. |
| `process_final_answer` | Inspect, validate, or rewrite the model's final answer before it is deserialized. |
| `process_output` | Inspect, validate, or rewrite the agent output after it is deserialized. |
| `finalize` | Consume the strategy and produce its `Output`. |

A response produces one `ModelRecord`: a structured canonical step paired with the whole native
response. `process_llm_step` runs once on the completed step; `inspect_usage` runs once per HTTP
response, including auxiliary compaction requests. The response's text and client calls are joined
into one `Finish` or `ToolCallMessage`. `LlmStep` retains the original `Finish` and `ToolCall`
variants and optional `thought` field. Multiple readable reasoning strings are joined without a separator
in that canonical field; the native response keeps them unchanged.

The stored transcript entry exposes the final canonical message; its native content retains all
blocks. Changes to the final message through `process_llm_step` are written back through
`modify_step`. Use `entries()` to inspect native history.

### Error semantics

Returning `Err` from a hook does not always fail the run — most hooks retry the phase they belong to.

| Hook | Effect of `Err` |
| --- | --- |
| `process_initial_messages` | Fails the run immediately. |
| `process_materialized` | Materializes the transcript again. |
| `process_request` | Builds the request again. |
| `process_response`, `process_llm_step`, `process_final_answer`, `process_output` | Retries the request. |
| `process_tool_result` | Fails the current step and prompts a retry. |
| `finalize` | Fails the run; avoid unless strictly necessary. |

## Implementing a strategy

Implement `Strategy<L, O>`, where `L` is the LLM type and `O` is the agent's output constructor (`SomeAgentOutputCtor`, generated alongside the agent).
Keep `L` generic to make the strategy work with any provider.
Likewise, keep `O` generic to make the strategy work with any output type, and only pin it if you need to inspect the structured output directly (via `process_output`).

```rust
use async_trait::async_trait;
use talin::chain::PartialTarget;
use talin::llm::Llm;
use talin::strategy::Strategy;

use crate::agents::SomeAgentOutputCtor;

pub struct ValidationStrategy;

#[async_trait]
impl<L: Llm> Strategy<L, SomeAgentOutputCtor> for ValidationStrategy {
    type Output = ();
    type Error = ValidationError;

    async fn process_output(
        &mut self,
        output: PartialTarget<SomeAgentOutputCtor>,
    ) -> Result<PartialTarget<SomeAgentOutputCtor>, Self::Error> {
        // Returning `Err` here asks the model for another answer.
        if output.name.len() > 20 {
            return Err(ValidationError::NameTooLong);
        }
        Ok(output)
    }

    async fn finalize(self) -> Result<Self::Output, Self::Error> {
        Ok(())
    }
}
```

## Pre-implemented strategies

All of these live in `talin::strategy`.

### Observation

- `RecordStrategy`: Records all messages and outputs during the agent execution.
  Returns the recorded messages.
- `UsageTrackStrategy`: Accumulates token usage across every LLM call in the turn.
  Returns the accumulated `Usage`.
  - `SharedUsageTrackStrategy`: A variant that also merges the turn's usage into a shared usage object, so usage can be tallied across multiple runs.

### Tools and output

- `ToolStrategy`: Provides an additional function tool to the agent.
  Returns the tool itself.
- `TagStrategy`: Tags tool output entries with unique, memorable identifiers, so the model can refer to an entry by tag instead of repeating its content.
  Returns the tag index.
  - `SharedTagStrategy`: A variant that shares the tag index across multiple agent executions, keeping tags consistent across a crew of collaborating agents.
- `BufferedFinalAnswerStrategy`: Lets the model build its final answer incrementally in a buffer instead of forcing it to produce the final answer in one go.
- `FinalAnswerToolStrategy`: Replaces a JSON-schema `response_format` with a `submit_final_answer` tool of the same shape.
  Useful for models that support tool calls but not structured output, and for models that are forced to answer as soon as `response_format` is set even when more tool calls are needed.
  Only compatible with OpenAI chat completions models.

### Compaction

`AutoCompactStrategy`, `ClaudeMessagesAutoCompactStrategy` and `OpenAIResponsesAutoCompactStrategy`
enable automatic compaction. On-demand policies such as `ClaudeMessagesCompactStrategy`,
`OpenAIResponsesCompactStrategy` and `CustomCompactStrategy` are configured on the session with
`with_compaction`. See [Compaction](/docs/advanced/compaction.md).

### Transcript editing

These strategies apply an [edit scheme](/docs/advanced/transcript.md#edit-schemes) to the transcript
on every step, saving you from writing the transcript hooks by hand.
`*EditStrategy` applies the scheme permanently (via `edit_transcript`), while `*ProjectionStrategy`
applies it ephemerally (via `project_transcript`).

- `ApplyEntryEditStrategy` / `ApplyEntryProjectionStrategy`: Apply an `EntryEditScheme` to the
  transcript's entries.
- `ApplySystemEditStrategy` / `ApplySystemProjectionStrategy`: Apply a `SystemEditScheme` to the
  system prompt.
- `ApplyEditsStrategy` / `ApplyProjectionsStrategy`: Apply a hybrid scheme that implements both.
