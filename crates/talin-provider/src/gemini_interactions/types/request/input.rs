//! An input for
//! [`CreateInteractionRequest`](crate::gemini_interactions::types::CreateInteractionRequest).

use serde::{Deserialize, Serialize};

use crate::gemini_interactions::types::{Content, Step};

/// The input for an interaction.
///
/// The API accepts a bare string, a single [`Content`] block, a list of [`Content`] blocks, or a
/// list of [`Step`]s.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
#[non_exhaustive]
pub enum InteractionsInput {
    /// A single content block.
    Content(Content),
    /// An ordered list of steps (for multi-turn conversations).
    #[cfg_attr(test, schema(title = "StepList"))]
    Steps(Vec<Step>),
    /// A list of content blocks.
    #[cfg_attr(test, schema(title = "ContentList"))]
    Contents(Vec<Content>),
    #[cfg_attr(test, allow(missing_docs))] // Documentation messes with schema generation.
    #[cfg_attr(not(test), doc = "A single string input.")]
    Text(String),
}

impl From<Content> for InteractionsInput {
    fn from(value: Content) -> Self {
        InteractionsInput::Content(value)
    }
}

impl From<Vec<Step>> for InteractionsInput {
    fn from(value: Vec<Step>) -> Self {
        InteractionsInput::Steps(value)
    }
}

impl From<Vec<Content>> for InteractionsInput {
    fn from(value: Vec<Content>) -> Self {
        InteractionsInput::Contents(value)
    }
}

impl From<String> for InteractionsInput {
    fn from(value: String) -> Self {
        InteractionsInput::Text(value)
    }
}

impl From<&str> for InteractionsInput {
    fn from(value: &str) -> Self {
        InteractionsInput::Text(value.to_string())
    }
}
