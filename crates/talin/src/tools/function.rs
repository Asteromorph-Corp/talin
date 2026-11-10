use std::string::String;

use async_trait::async_trait;
use schemars::{JsonSchema, Schema, schema_for};
use serde::de::DeserializeOwned;
use serde_json::Value;

mod function_spec;
mod function_tool;
mod input;

pub use function_spec::FunctionSpec;
pub use function_tool::FunctionTool;
pub(crate) use function_tool::sealed;
pub use input::{DefaultFunctionInput, EmptyFunctionInput};

use crate::schemas::ToolOutput;
use crate::transcript::EntryView;

/// The core trait for implementing tools that can be used by LLM agents.
///
/// This trait defines the interface that all function-based tools must implement.
/// It provides methods for describing the tool's capabilities, defining its input/output
/// schema, and executing its functionality.
///
/// # Type Parameters
///
/// * `Input` - The input type for the tool. Must implement [`JsonSchema`] for schema generation and
///   [`DeserializeOwned`] for JSON parsing.
/// * `Output` - The output type for the tool. Must be convertible to
///   [`ContentPart`](crate::schemas::ContentPart).
///
/// # Required Methods
///
/// * [`name`](Self::name) - Returns the tool's name
/// * [`description`](Self::description) - Returns a description of what the tool does
/// * [`call`](Self::call) - Executes the tool's functionality
///
/// # Optional Methods
///
/// * [`strict`](Self::strict) - Whether to use strict input validation (default: false)
/// * [`custom_description`](Self::custom_description) - Custom description override
/// * [`available`](Self::available) - Whether the tool can be used given the transcript
/// * [`parse_input`](Self::parse_input) - Custom input parsing logic
/// * [`summarize_input`](Self::summarize_input) - Summarize tool input for context compression
/// * [`summarize_output`](Self::summarize_output) - Summarize tool output for context compression
#[async_trait]
pub trait Function: Send + Sync {
    /// The input type for the tool.
    ///
    /// `JsonSchema` must be implemented for the type,
    /// which can be done using the derive macro from the `schemars` crate.
    ///
    /// If the type has fields of non-primitive types,
    /// you should implement `inline_subschema` and return `true`.
    type Input: JsonSchema + DeserializeOwned + Send + Sync;

    /// The output type for the tool. Recommended to use `String` or `Vec<String>` for simple tools.
    type Output: Into<ToolOutput> + Send + Sync;

    /// Returns the name of the tool.
    fn name(&self) -> String;

    /// Provides a description of what the tool does and when to use it.
    fn description(&self) -> String;

    /// JSON schema for the tool input parameters.
    ///
    /// Used for OpenAI function call.
    ///
    /// You don't need to implement this method as it is automatically generated based on the
    /// `Input` type.
    fn parameters(&self) -> Schema {
        schema_for!(Self::Input)
    }

    /// Whether the tool should be strict in its input validation.
    ///
    /// Used for OpenAI function call.
    ///
    /// If not implemented, it will default to `false`.
    fn strict(&self) -> bool {
        false
    }

    /// Optionally provides a fully custom description used instaed of the auto-generated tool
    /// description.
    fn custom_description(&self) -> Option<String> {
        None
    }

    /// Whether the tool is currently available, given the conversation so far.
    ///
    /// When this returns `false`, the tool is left out of the tool list of the following requests,
    /// and any calls to it is rejected. Calls already in the transcript are left untouched.
    /// Additionally, if every tool would be removed while the transcript contains tool calls, the
    /// tools are kept, since providers reject such requests.
    ///
    /// Due to how availability is computed, editing the transcript may affect the tool's
    /// availability. For example, if a call cap is written with [`count_tool_calls`] (e.g.
    /// `count_tool_calls(transcript) < N`), dropping old calls from the transcript may make the
    /// tool available again even after `N` calls have been made. If a tool requires a strict call
    /// cap, it should be implemented in the tool itself, e.g. with an internal counter.
    ///
    /// If left unimplemented, the tool is always available.
    ///
    /// [`count_tool_calls`]: crate::transcript::query::count_tool_calls
    fn available(&self, _transcript: &[EntryView<'_>]) -> bool {
        true
    }

    /// Executes the core functionality of the tool.
    ///
    /// Example implementation:
    /// ```rust,ignore
    /// // type Input = (usize, usize);
    /// // type Output = String;
    ///
    /// async fn call(
    ///     &self,
    ///     input: Self::Input,
    /// ) -> Result<Self::Output, Box<dyn std::error::Error + Send + Sync>> {
    ///     let result = self.0 + self.1;
    ///     Ok(result.to_string())
    /// }
    /// ```
    async fn call(
        &self,
        input: Self::Input,
    ) -> Result<Self::Output, Box<dyn std::error::Error + Send + Sync>>;

    /// Parses the input JSON value.
    ///
    /// If not implemented, it will default to parsing the input as a JSON value.
    async fn parse_input(&self, input: Value) -> Result<Self::Input, serde_json::Error> {
        serde_json::from_value(input)
    }

    /// Returns a concise summary of the tool's input.
    ///
    /// The summary is used for long conversaions to reduce context length.
    fn summarize_input(&self, _input: &Self::Input) -> Option<String> {
        Some(format!("Used tool {}", self.name()))
    }

    /// Returns a concise summary of the tool's output.
    ///
    /// The summary is used for long conversations to reduce context length.
    fn summarize_output(&self, _result: &Self::Output) -> Option<String> {
        None
    }
}
