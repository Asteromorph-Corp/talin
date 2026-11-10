//! Compaction via the <https://platform.claude.com/docs/en/api/http/beta/messages/create#create.compaction> field, enabled via the `compact-2026-09-04` header.

use serde::{Deserialize, Serialize};

/// The on-demand compaction operation.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum CompactionType {
    /// Summarizes the supplied conversation context.
    #[default]
    Summarize,
}

/// The [on-demand compaction configuration](https://platform.claude.com/docs/en/api/http/beta/messages#beta_compaction_config).
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[non_exhaustive]
pub struct CompactionConfig {
    /// The kind of configuration or iteration represented by this object.
    #[serde(rename = "type")]
    pub type_: CompactionType,
    /// Optional instructions for generating the summary.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
}

impl CompactionConfig {
    /// Creates a configuration with the supplied values and otherwise default settings.
    pub fn new() -> Self {
        Default::default()
    }

    /// Sets the instructions for generating the summary.
    pub fn with_instructions(mut self, instructions: impl Into<String>) -> Self {
        self.instructions = Some(instructions.into());
        self
    }
}
