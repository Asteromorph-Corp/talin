use serde::{Deserialize, Serialize};

use super::types::{InputItem, ResponseUsage};

/// The complete replacement history returned by `/responses/compact`.
/// Retained user messages are input items, alongside the generated compaction item.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompactResource {
    /// Unique identifier for the compaction response.
    pub id: String,
    /// Always `response.compaction`.
    pub object: String,
    /// Replacement history in provider order, including retained user messages.
    pub output: Vec<InputItem>,
    /// Creation timestamp in seconds since the Unix epoch.
    pub created_at: u64,
    /// Usage for the summarization call.
    pub usage: ResponseUsage,
}

impl From<CompactResource> for Vec<InputItem> {
    fn from(response: CompactResource) -> Self {
        response.output
    }
}
