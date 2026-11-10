use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// Config for thinking features.
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ThinkingConfig {
    /// Indicates whether to include thoughts in the response.
    /// If true, thoughts are returned only when available.
    pub include_thoughts: Option<bool>,

    /// The number of thoughts tokens that the model should generate.
    pub thinking_budget: Option<u32>,

    /// Optional. Controls the maximum depth of the model's internal reasoning process.
    pub thinking_level: Option<ThinkingLevel>,
}

impl ThinkingConfig {
    /// Creates a new ThinkingConfig with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets whether to include thoughts in the response.
    pub fn with_include_thoughts(mut self, include_thoughts: bool) -> Self {
        self.include_thoughts = Some(include_thoughts);
        self
    }

    /// Sets the number of thoughts tokens the model should generate.
    pub fn with_thinking_budget(mut self, thinking_budget: u32) -> Self {
        self.thinking_budget = Some(thinking_budget);
        self
    }

    /// Sets the depth of the model's internal reasoning process.
    pub fn with_thinking_level(mut self, thinking_level: ThinkingLevel) -> Self {
        self.thinking_level = Some(thinking_level);
        self
    }
}

/// Controls the depth of thinking.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum ThinkingLevel {
    /// Default value.
    ThinkingLevelUnspecified,
    /// Little to no thinking.
    Minimal,
    /// Low thinking level.
    Low,
    /// Medium thinking level.
    Medium,
    /// High thinking level.
    High,
}
