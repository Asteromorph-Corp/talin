use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// Result of executing code.
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct CodeExecutionResult {
    /// Optional. Identifier correlating this result with the originating
    /// `ExecutableCode` part.
    pub id: Option<String>,

    /// Outcome of the code execution.
    pub outcome: Outcome,

    /// Output from the code execution (stdout).
    pub output: Option<String>,
}

impl CodeExecutionResult {
    /// Creates a new [`CodeExecutionResult`].
    pub fn new(outcome: Outcome) -> Self {
        Self {
            id: None,
            outcome,
            output: None,
        }
    }

    /// Sets the `id` field.
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Sets the `output` field.
    pub fn with_output(mut self, output: impl Into<String>) -> Self {
        self.output = Some(output.into());
        self
    }
}

/// Outcome of code execution.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum Outcome {
    /// Outcome unspecified.
    OutcomeUnspecified,
    /// Code execution completed successfully.
    OutcomeOk,
    /// Code execution failed.
    OutcomeFailed,
    /// Code execution deadline exceeded.
    OutcomeDeadlineExceeded,
}
