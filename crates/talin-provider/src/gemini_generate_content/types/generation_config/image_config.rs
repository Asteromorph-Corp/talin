use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// Config for image generation features.
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ImageConfig {
    /// Optional. The aspect ratio of the image to generate.
    /// Supported aspect ratios: 1:1, 2:3, 3:2, 3:4, 4:3, 9:16, 16:9, 21:9.
    pub aspect_ratio: Option<AspectRatio>,

    /// Optional. Specifies the size of generated images.
    /// Supported values are `"1K"`, `"2K"`, `"4K"`.
    pub image_size: Option<ImageSize>,
}

impl ImageConfig {
    /// Creates a new ImageConfig with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the aspect ratio.
    pub fn with_aspect_ratio(mut self, aspect_ratio: AspectRatio) -> Self {
        self.aspect_ratio = Some(aspect_ratio);
        self
    }

    /// Sets the image size.
    pub fn with_image_size(mut self, image_size: ImageSize) -> Self {
        self.image_size = Some(image_size);
        self
    }
}

/// The aspect ratios supported for image generation.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum AspectRatio {
    /// `"1:1"`
    #[serde(rename = "1:1")]
    R1x1,
    /// `"2:3"`
    #[serde(rename = "2:3")]
    R2x3,
    /// `"3:2"`
    #[serde(rename = "3:2")]
    R3x2,
    /// `"3:4"`
    #[serde(rename = "3:4")]
    R3x4,
    /// `"4:3"`
    #[serde(rename = "4:3")]
    R4x3,
    /// `"9:16"`
    #[serde(rename = "9:16")]
    R9x16,
    /// `"16:9"`
    #[serde(rename = "16:9")]
    R16x9,
    /// `"21:9"`
    #[serde(rename = "21:9")]
    R21x9,
}

/// Specifies the size of generated images.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ImageSize {
    /// `"1K"`
    #[serde(rename = "1K")]
    Size1K,
    /// `"2K"`
    #[serde(rename = "2K")]
    Size2K,
    /// `"4K"`
    #[serde(rename = "4K")]
    Size4K,
}
