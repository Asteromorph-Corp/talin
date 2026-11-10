use serde::{Deserialize, Serialize};
use serde_with::base64::Base64;
use serde_with::{serde_as, skip_serializing_none};

use crate::claude_messages::types::CacheControlEphemeral;
use crate::utils::{string_enum, tagged_enum};

/// An image block within a request message.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ImageBlockParam {
    /// The image source.
    pub source: ImageSource,

    /// The transformations the server applies to this image before the model observes it.
    pub transformations: Option<ImageTransformationsParam>,

    /// Create a cache control breakpoint at this content block.
    pub cache_control: Option<CacheControlEphemeral>,
}

/// The transformations the server applies to an image before the model observes it.
///
/// Each field names a condition the server transforms images for, and selects the transformation
/// applied to it. An omitted field keeps its default behavior, so an empty value is equivalent to
/// omitting the transformations altogether.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[non_exhaustive]
pub struct ImageTransformationsParam {
    /// What the server does when this image exceeds the model's maximum image size.
    #[cfg_attr(test, schema(nullable = false))]
    pub oversized_image: Option<OversizedImageTransformation>,
}

string_enum! {
    /// What the server does with an image that exceeds the model's maximum image size.
    #[serde(rename_all = "snake_case")]
    pub enum OversizedImageTransformation {
        /// Scales the image down to fit, which changes the dimensions the model observes without
        /// saying so. The default.
        Downsize,

        /// Rejects the request with a 400 error naming the image's dimensions and the largest
        /// dimensions that fit, so that the image can be scaled deliberately instead.
        Error,
    }
}

tagged_enum! {
    /// The image source.
    #[derive(Serialize, Deserialize, Debug, Clone)]
    #[serde(tag = "type", rename_all = "snake_case")]
    pub enum ImageSource {
        /// A base64-encoded image.
        Base64(Base64ImageSource),
        /// A URL pointing to an image.
        Url(URLImageSource),
        /// An image file id.
        File(FileImageSource),
    }
}

/// A base64-encoded image.
#[serde_as]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct Base64ImageSource {
    /// The base64-encoded image data.
    #[serde_as(as = "Base64")]
    #[cfg_attr(test, schema(value_type = String))]
    pub data: Vec<u8>,

    /// The media type of the image. e.g. "image/png", "image/jpeg".
    pub media_type: Base64ImageType,
}

string_enum! {
    /// A media type of a base64-encoded image.
    pub enum Base64ImageType {
        /// `"image/jpeg"`
        #[serde(rename = "image/jpeg")]
        Jpeg,

        /// `"image/png"`
        #[serde(rename = "image/png")]
        Png,

        /// `"image/gif"`
        #[serde(rename = "image/gif")]
        Gif,

        /// `"image/webp"`
        #[serde(rename = "image/webp")]
        Webp,
    }
}

/// A URL pointing to an image.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct URLImageSource {
    /// The URL pointing to the image.
    pub url: String,
}

/// An image file id.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct FileImageSource {
    /// The file id of the image.
    pub file_id: String,
}

impl ImageBlockParam {
    /// Creates a new [`ImageBlockParam`] from the given source.
    pub fn new(source: ImageSource) -> Self {
        Self {
            source,
            transformations: None,
            cache_control: None,
        }
    }

    /// Sets the [`transformations`](Self::transformations) applied to this image.
    pub fn with_transformations(mut self, transformations: ImageTransformationsParam) -> Self {
        self.transformations = Some(transformations);
        self
    }

    /// Sets the [`cache_control`](Self::cache_control) breakpoint for this content block.
    pub fn with_cache_control(mut self, cache_control: CacheControlEphemeral) -> Self {
        self.cache_control = Some(cache_control);
        self
    }
}

impl ImageTransformationsParam {
    /// Creates a new [`ImageTransformationsParam`], applying no transformation.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets what the server does with an
    /// [`oversized_image`](Self::oversized_image).
    pub fn with_oversized_image(mut self, oversized_image: OversizedImageTransformation) -> Self {
        self.oversized_image = Some(oversized_image);
        self
    }
}

impl Base64ImageSource {
    /// Creates a new [`Base64ImageSource`] from the given data and media type.
    pub fn new(data: Vec<u8>, media_type: Base64ImageType) -> Self {
        Self { data, media_type }
    }
}

impl URLImageSource {
    /// Creates a new [`URLImageSource`] from the given URL.
    pub fn new(url: impl Into<String>) -> Self {
        Self { url: url.into() }
    }
}

impl FileImageSource {
    /// Creates a new [`FileImageSource`] from the given file id.
    pub fn new(file_id: impl Into<String>) -> Self {
        Self {
            file_id: file_id.into(),
        }
    }
}
