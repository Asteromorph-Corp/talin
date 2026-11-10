use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// A set of the feedback metadata the prompt specified in GenerateContentRequest.content.
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct PromptFeedback {
    /// Optional. If set, the prompt was blocked and no candidates are returned.
    pub block_reason: Option<BlockReason>,

    /// Ratings for safety of the prompt.
    pub safety_ratings: Option<Vec<super::SafetyRating>>,
}

/// Specifies the reason why the prompt was blocked.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum BlockReason {
    /// Default value. This value is unused.
    BlockReasonUnspecified,
    /// Prompt was blocked due to safety reasons.
    Safety,
    /// Prompt was blocked due to unknown reasons.
    Other,
    /// Prompt was blocked due to the terms which are included from the terminology blocklist.
    Blocklist,
    /// Prompt was blocked due to prohibited content.
    ProhibitedContent,
    /// Candidates blocked due to unsafe image generation content.
    ImageSafety,
    /// Unknown reason not represented by this enum.
    #[serde(other)]
    Unknown,
}
