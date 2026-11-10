use crate::schemas::{IntoWithUsage, Usage, WithUsage};

/// The final output from an agent turn.
///
/// Contains content, token usage, and extra content based on the execution strategy.
pub struct AgentOutput<O, S>
where
    O: Send + Sync,
    S: Send + Sync,
{
    /// The main content produced by the agent.
    pub content: O,
    /// Extra content returned by the strategy used during execution.
    pub extra_content: S,
}

impl<O, S> AgentOutput<O, S>
where
    O: Send + Sync,
    S: Send + Sync,
{
    /// Constructs a new [`AgentOutput`].
    pub fn new(content: O, extra_content: S) -> Self {
        Self {
            content,
            extra_content,
        }
    }
}

impl<O> AgentOutput<O, Usage>
where
    O: Send + Sync,
{
    /// Removes strategy-specific extra contents, leaving only the content and the token usage.
    pub fn into_with_usage(self) -> WithUsage<O> {
        self.content.with_usage(self.extra_content)
    }
}
