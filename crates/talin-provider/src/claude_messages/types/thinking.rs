use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::utils::{string_enum, tagged_enum};

tagged_enum! {
    /// Configuration for enabling Claude's extended thinking.
    #[derive(Serialize, Deserialize, Debug, Clone)]
    #[serde(tag = "type", rename_all = "snake_case")]
    pub enum ThinkingConfigParam {
        /// Enable extended thinking with a specified token budget.
        Enabled(ThinkingConfigEnabled),
        /// Enable adaptive thinking.
        Adaptive(ThinkingConfigAdaptive),
        /// Disable extended thinking.
        Disabled,
        /// Enable thinking between tool calls.
        BetweenTools,
    }
}

string_enum! {
    /// Controls how thinking content appears in the response.
    #[serde(rename_all = "snake_case")]
    pub enum ThinkingDisplay {
        /// Response includes summarized thinking content.
        Summarized,

        /// Response omits thinking content.
        Omitted,
    }
}

/// Configuration for enabling extended thinking with a specified token budget.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ThinkingConfigEnabled {
    /// The maximum number of tokens Claude can use for its internal reasoning process.
    ///
    /// Must be >= 1024 and less than
    /// [`max_tokens`](crate::claude_messages::types::MessagesRequest::max_tokens).
    #[cfg_attr(test, schema(schema_with = crate::claude_messages::spec::schema::number_schema))]
    pub budget_tokens: u32,

    /// Controls how thinking content appears in the response.
    pub display: Option<ThinkingDisplay>,
}

/// Configuration for enabling adaptive thinking.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[non_exhaustive]
pub struct ThinkingConfigAdaptive {
    /// Controls how thinking content appears in the response.
    pub display: Option<ThinkingDisplay>,
}

impl ThinkingConfigEnabled {
    /// Creates a new [`ThinkingConfigEnabled`] with the given token budget.
    pub fn new(budget_tokens: u32) -> Self {
        Self {
            budget_tokens,
            display: None,
        }
    }

    /// Sets the [`display`](Self::display) mode for thinking content.
    pub fn with_display(mut self, display: ThinkingDisplay) -> Self {
        self.display = Some(display);
        self
    }
}

impl ThinkingConfigAdaptive {
    /// Creates a new [`ThinkingConfigAdaptive`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the [`display`](Self::display) mode for thinking content.
    pub fn with_display(mut self, display: ThinkingDisplay) -> Self {
        self.display = Some(display);
        self
    }
}
