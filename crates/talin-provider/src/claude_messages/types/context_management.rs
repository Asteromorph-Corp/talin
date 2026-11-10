//! Threshold compaction via the <https://platform.claude.com/docs/en/api/http/beta/messages#beta_context_management_config> field, enabled via the `compact-2026-01-12` header.

use serde::{Deserialize, Serialize};

/// The [`BetaContextManagementConfig`](https://platform.claude.com/docs/en/api/http/beta/messages#beta_context_management_config) request field.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Debug, Clone, Default)]
#[non_exhaustive]
pub struct ContextManagementConfig {
    /// Context management edits to apply to the request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edits: Option<Vec<ContextManagementEdit>>,
}

impl ContextManagementConfig {
    /// Creates a configuration with the supplied values and otherwise default settings.
    pub fn new(edits: Vec<ContextManagementEdit>) -> Self {
        Self { edits: Some(edits) }
    }

    /// Sets the context management edits.
    pub fn with_edits(mut self, edits: Vec<ContextManagementEdit>) -> Self {
        self.edits = Some(edits);
        self
    }
}

/// Supported context management edits.
/// See [threshold compaction](https://platform.claude.com/docs/en/build-with-claude/compaction-threshold).
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Debug, Clone)]
#[serde(tag = "type")]
#[non_exhaustive]
pub enum ContextManagementEdit {
    /// Compacts conversation context at the configured threshold.
    #[serde(rename = "compact_20260112")]
    Compact(CompactEdit),
}

/// The `compact_20260112` edit; configures server-side auto compaction at a token threshold.
/// Requires the `compact-2026-01-12` beta header.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Debug, Clone, Default)]
#[non_exhaustive]
pub struct CompactEdit {
    /// Optional instructions for generating the summary.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
    /// Whether to return immediately after generating a compaction summary.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pause_after_compaction: Option<bool>,
    /// The token threshold that triggers automatic compaction.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trigger: Option<InputTokensThreshold>,
}

impl CompactEdit {
    /// Creates a configuration with the supplied values and otherwise default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the instructions for generating the summary.
    pub fn with_instructions(mut self, instructions: impl Into<String>) -> Self {
        self.instructions = Some(instructions.into());
        self
    }

    /// Sets whether generation pauses after compaction.
    pub fn with_pause_after_compaction(mut self, pause: bool) -> Self {
        self.pause_after_compaction = Some(pause);
        self
    }

    /// Sets the input token threshold for automatic compaction.
    pub fn with_trigger(mut self, trigger: InputTokensThreshold) -> Self {
        self.trigger = Some(trigger);
        self
    }
}

/// An input token trigger for threshold compaction.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InputTokensThreshold {
    /// The kind of configuration or iteration represented by this object.
    #[serde(rename = "type")]
    pub type_: InputTokensThresholdType,
    /// The input token count at which compaction is triggered.
    pub value: u32,
}

/// The token count used to trigger automatic compaction.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "snake_case")]
pub enum InputTokensThresholdType {
    /// Triggers based on the input token count.
    #[default]
    InputTokens,
}

impl InputTokensThreshold {
    /// Creates a configuration with the supplied values and otherwise default settings.
    pub fn new(value: u32) -> Self {
        Self {
            type_: InputTokensThresholdType::InputTokens,
            value,
        }
    }
}
