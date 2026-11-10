use thiserror::Error;

use crate::agent::AgentError;
use crate::llm::LlmError;
use crate::template::TemplateError;
use crate::text::parse::ParseError;

/// Errors that can occur during chain execution.
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum ChainError {
    /// Agent error.
    #[error("agent error: {0}")]
    Agent(#[source] AgentError),

    /// Model error.
    #[error("model error: {0}")]
    Model(#[source] LlmError),

    /// Failed to format prompt template (e.g. missing variable).
    #[error("failed to format prompt template: {0}")]
    Template(#[from] TemplateError),

    /// Failed to parse model output.
    #[error(transparent)]
    Parse(#[from] ParseError),

    /// Error not covered by other variants.
    #[error("{0}")]
    Other(String),
}

impl From<AgentError> for ChainError {
    fn from(error: AgentError) -> Self {
        match error {
            AgentError::Model(e) => ChainError::Model(e),
            AgentError::Template(e) => ChainError::Template(e),
            AgentError::Parse(e) => ChainError::Parse(e),
            other => ChainError::Agent(other),
        }
    }
}

impl From<LlmError> for ChainError {
    fn from(error: LlmError) -> Self {
        match error {
            LlmError::Parse(e) => ChainError::Parse(e),
            other => ChainError::Model(other),
        }
    }
}

impl<I> From<(I, ParseError)> for ChainError {
    fn from((_, err): (I, ParseError)) -> Self {
        ChainError::Parse(err)
    }
}
