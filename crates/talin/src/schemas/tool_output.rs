use serde::{Deserialize, Serialize};

use crate::schemas::ContentPart;
use crate::tools::ToolError;

/// The result of a tool call.
pub type ToolResult = Result<ToolOutput, ToolError>;

/// The output of a successful tool execution.
///
/// Contains the actual data returned by the tool along with an optional summary
/// that can be used for context compression in long conversations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolOutput {
    /// The actual output data from the tool.
    pub content: Vec<ContentPart>,
    /// A summary of the tool execution for context compression.
    ///
    /// Currently unused, but may be used in the future for context compression in long
    /// conversations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
}

impl ToolOutput {
    /// Creates a new [`ToolOutput`] with the given contents.
    pub fn new(content: Vec<ContentPart>) -> Self {
        ToolOutput {
            content,
            summary: None,
        }
    }

    /// Sets the [`summary`](Self::summary) for the output.
    pub fn with_sumary(mut self, summary: String) -> Self {
        self.summary = Some(summary);
        self
    }

    /// Returns the output as text if it consists of exactly one text part. Returns `None`
    /// otherwise.
    pub fn as_text(&self) -> Option<&str> {
        if self.content.len() != 1 {
            return None;
        }
        let Some(ContentPart::Text(text)) = self.content.first() else {
            return None;
        };
        Some(text)
    }
}

impl<T> From<T> for ToolOutput
where
    ContentPart: From<T>,
{
    fn from(item: T) -> Self {
        ToolOutput {
            content: vec![ContentPart::from(item)],
            summary: None,
        }
    }
}

impl<T> From<Vec<T>> for ToolOutput
where
    ContentPart: From<T>,
{
    fn from(items: Vec<T>) -> Self {
        let content = items.into_iter().map(ContentPart::from).collect::<Vec<_>>();
        ToolOutput {
            content,
            summary: None,
        }
    }
}

impl std::fmt::Display for ToolOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, item) in self.content.iter().enumerate() {
            if i > 0 {
                writeln!(f, "\n---")?;
            }
            write!(f, "{item}")?;
        }
        Ok(())
    }
}
