use serde::{Deserialize, Serialize};

/// Content part modality.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum Modality {
    /// Unspecified modality.
    ModalityUnspecified,
    /// Plain text.
    Text,
    /// Image.
    Image,
    /// Video.
    Video,
    /// Audio.
    Audio,
    /// Document, e.g. PDF.
    Document,
    /// Unknown modality not represented by this enum.
    #[serde(other)]
    Unknown,
}

/// Represents token counting info for a single modality.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ModalityTokenCount {
    /// The modality associated with this token count.
    pub modality: Modality,
    /// Number of tokens.
    pub token_count: u32,
}
