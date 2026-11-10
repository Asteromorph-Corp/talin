use serde::{Deserialize, Serialize};

/// Media resolution for the input media.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum MediaResolution {
    /// Media resolution has not been set.
    MediaResolutionUnspecified,
    /// Media resolution set to low (64 tokens).
    MediaResolutionLow,
    /// Media resolution set to medium (256 tokens).
    MediaResolutionMedium,
    /// Media resolution set to high (zoomed reframing with 256 tokens).
    MediaResolutionHigh,
}
