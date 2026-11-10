use std::string::String;
use std::sync::Arc;

use async_trait::async_trait;
use schemars::{Schema, schema_for};
use serde_json::Value;

use crate::schemas::{ToolOutput, ToolResult};
use crate::tools::{DefaultFunctionInput, Function, FunctionSpec, ToolError, normalize_tool_name};
use crate::transcript::EntryView;

pub(crate) mod sealed {
    /// A sealed trait to prevent external implementations of the `ToolInternal` trait.
    pub trait Sealed {}
}

/// A dyn-compatible, generic-less interface for tools.
///
/// This trait is sealed, and automatically implemented for all types that implement [`Function`].
#[async_trait]
pub trait FunctionTool: sealed::Sealed + Send + Sync {
    /// Returns the name of the tool.
    fn name(&self) -> String;

    /// Provides a description of what the tool does and when to use it.
    fn description(&self) -> String;

    /// JSON schema for the tool input parameters.
    ///
    /// Used for OpenAI function call.
    ///
    /// If not implemented, it will default to
    /// ```json
    /// {
    ///     "input": {
    ///         "type": "string",
    ///         "description": "The input for the tool"
    ///     },
    ///     required: ["input"]
    /// }
    /// ```
    fn parameters(&self) -> Schema {
        schema_for!(DefaultFunctionInput)
    }

    /// Value for `strict` in the OpenAI function call
    ///
    /// If not implemented, it will default to `false`
    fn strict(&self) -> bool {
        false
    }

    /// Optionally provides a fully custom description used instaed of the auto-generated tool
    /// description.
    fn custom_description(&self) -> Option<String> {
        None
    }

    /// Whether the tool is currently available given the transcript; see [`Function::available`].
    fn available(&self, _transcript: &[EntryView<'_>]) -> bool {
        true
    }

    /// Processes an input string and executes the tool's functionality, returning a `Result`.
    async fn call(&self, input: Value) -> ToolResult;

    /// Returns the [`FunctionSpec`] for this tool.
    fn get_spec(&self) -> FunctionSpec {
        FunctionSpec::new(
            normalize_tool_name(&self.name()),
            Some(self.description()),
            self.parameters(),
            self.strict(),
            self.custom_description(),
        )
    }
}

impl<T> sealed::Sealed for T where T: Function {}

/// Blanket implementation of `FunctionTool` for any type that implements `Function`.
#[async_trait]
impl<T> FunctionTool for T
where
    T: Function + sealed::Sealed,
{
    fn name(&self) -> String {
        self.name()
    }

    fn description(&self) -> String {
        self.description()
    }

    fn parameters(&self) -> Schema {
        self.parameters()
    }

    fn strict(&self) -> bool {
        self.strict()
    }

    fn custom_description(&self) -> Option<String> {
        self.custom_description()
    }

    fn available(&self, transcript: &[EntryView<'_>]) -> bool {
        Function::available(self, transcript)
    }

    async fn call(&self, input: Value) -> ToolResult {
        let input: T::Input = self
            .parse_input(input)
            .await
            .map_err(ToolError::InputDeserialize)?;
        let input_summary = self.summarize_input(&input);
        let output: T::Output = self.call(input).await.map_err(ToolError::Execution)?;
        let output_summary = self.summarize_output(&output);

        let ToolOutput { content, summary } = output.into();
        let summary = [input_summary, summary, output_summary]
            .into_iter()
            .flatten()
            .reduce(|mut combined, value| {
                combined.push_str(&value);
                combined
            });

        Ok(ToolOutput { content, summary })
    }
}

impl<T: FunctionTool> sealed::Sealed for Arc<T> {}

/// Blanket implementation of `FunctionTool` for `Arc<T>` where `T: FunctionTool`.
#[async_trait]
impl<T: FunctionTool> FunctionTool for Arc<T> {
    fn name(&self) -> String {
        self.as_ref().name()
    }

    fn description(&self) -> String {
        self.as_ref().description()
    }

    fn parameters(&self) -> Schema {
        self.as_ref().parameters()
    }

    fn strict(&self) -> bool {
        self.as_ref().strict()
    }

    async fn call(&self, input: Value) -> Result<ToolOutput, ToolError> {
        self.as_ref().call(input).await
    }

    fn available(&self, transcript: &[EntryView<'_>]) -> bool {
        self.as_ref().available(transcript)
    }
}

impl<'a, T> From<T> for Box<dyn FunctionTool + 'a>
where
    T: FunctionTool + 'a,
{
    fn from(val: T) -> Self {
        Box::new(val)
    }
}
