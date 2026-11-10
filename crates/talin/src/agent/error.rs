use std::error::Error;

use thiserror::Error;

use crate::llm::{LlmError, UnsupportedContent};
use crate::mcp::McpError;
use crate::skill::SkillError;
use crate::template::TemplateError;
use crate::text::parse::ParseError;

/// Errors that can occur during agent operations.
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum AgentError {
    /// Model error.
    #[error("model error: {0}")]
    Model(#[source] LlmError),

    /// [Strategy](crate::strategy::Strategy) error.
    #[error("`Strategy::{0}` failed: {1}")]
    Strategy(&'static str, #[source] Box<dyn Error + Send + Sync>),

    /// Failed to format prompt template (e.g. missing variable).
    #[error("failed to format prompt template: {0}")]
    Template(#[from] TemplateError),

    /// Failed to sync tool list for an MCP server.
    #[error(transparent)]
    McpSync(#[from] McpSyncError),

    /// A skill could not be read.
    #[error("failed to read a skill: {0}")]
    Skill(#[from] SkillError),

    /// Failed to parse model output.
    #[error(transparent)]
    Parse(#[from] ParseError),

    /// All tool executions failed for a step.
    #[error("all tool executions failed for a step")]
    AllToolsFailed,

    /// Model requested tool call after reaching maximum number of iterations.
    ///
    /// The model must return a final answer when it reaches the max iterations.
    #[error("model requested tool call after reaching max iteration")]
    ToolCallAfterMaxIteration,
}

/// MCP server sync failure.
#[derive(Debug, Error)]
#[error("failed to sync tool list for MCP server \"{server}\": {source}")]
pub struct McpSyncError {
    /// The name of the server that failed.
    pub server: String,
    /// The inner mcp error.
    #[source]
    pub source: McpError,
}

impl AgentError {
    /// Returns whether retrying the failed request may plausibly succeed.
    pub fn is_retryable(&self) -> bool {
        match self {
            AgentError::Model(e) => e.is_retryable(),
            // Retryability of user-defined strategy hooks is unknown; assume retryable.
            AgentError::Strategy(_, _) => true,
            AgentError::Template(_) => false,
            AgentError::McpSync(_) => true,
            AgentError::Skill(e) => e.is_retryable(),
            // The model would produce a different output on retry.
            AgentError::Parse(_) => true,
            // The model can use the tool error messages and adjust accordingly.
            AgentError::AllToolsFailed => true,
            // The model is prompted to return the final answer.
            AgentError::ToolCallAfterMaxIteration => true,
        }
    }
}

impl McpSyncError {
    /// Constructs a new [`McpSyncError`] for the named server.
    pub fn new(server: impl Into<String>, source: McpError) -> Self {
        McpSyncError {
            server: server.into(),
            source,
        }
    }
}

impl From<LlmError> for AgentError {
    fn from(error: LlmError) -> Self {
        match error {
            LlmError::Parse(e) => AgentError::Parse(e),
            other => AgentError::Model(other),
        }
    }
}

impl From<UnsupportedContent> for AgentError {
    fn from(error: UnsupportedContent) -> Self {
        AgentError::Model(LlmError::from(error))
    }
}
