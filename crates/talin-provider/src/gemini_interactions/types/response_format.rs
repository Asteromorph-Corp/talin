use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::utils::string_enum;

/// Configuration for a single response output format.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
#[non_exhaustive]
pub enum ResponseFormat {
    /// Configuration for text output format.
    Text(TextResponseFormat),
    /// Configuration for image output format.
    Image(ImageResponseFormat),
    /// Configuration for audio output format.
    Audio(AudioResponseFormat),
    /// Configuration for video output format.
    Video(VideoResponseFormat),
    /// A response format specified by an arbitrary JSON object.
    #[serde(untagged)]
    #[cfg_attr(test, schema(title = "OtherResponseFormat"))]
    Other(#[cfg_attr(test, schema(inline = true))] OtherResponseFormat),
}

string_enum! {
    /// A response output modality.
    #[serde(rename_all = "snake_case")]
    pub enum ResponseModality {
        /// The model should return text.
        Text,

        /// The model should return images.
        Image,

        /// The model should return audio.
        Audio,

        /// The model should return video.
        Video,

        /// The model should return documents.
        Document,
    }
}

string_enum! {
    /// The delivery mode for a generated media output.
    #[serde(rename_all = "snake_case")]
    pub enum Delivery {
        /// Data is returned inline in the response.
        Inline,

        /// Data is returned as a URI.
        Uri,
    }
}

/// Configuration for text output format.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[non_exhaustive]
pub struct TextResponseFormat {
    /// The MIME type of the text output.
    #[cfg_attr(test, schema(nullable = false))]
    pub mime_type: Option<TextOutputMimeType>,

    /// The JSON schema that the output should conform to. Only applicable when `mime_type`
    /// is `application/json`.
    #[cfg_attr(test, schema(schema_with = crate::gemini_interactions::spec::schema::object_schema))]
    pub schema: Option<schemars::Schema>,
}

string_enum! {
    /// The MIME type of a text output.
    pub enum TextOutputMimeType {
        /// JSON output format.
        #[serde(rename = "application/json")]
        ApplicationJson,

        /// Plain text output format.
        #[serde(rename = "text/plain")]
        TextPlain,
    }
}

/// Configuration for image output format.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[non_exhaustive]
pub struct ImageResponseFormat {
    /// The aspect ratio for the image output.
    #[cfg_attr(test, schema(nullable = false))]
    pub aspect_ratio: Option<ImageAspectRatio>,

    /// The delivery mode for the image output.
    #[cfg_attr(test, schema(nullable = false))]
    pub delivery: Option<Delivery>,

    /// The size of the image output.
    #[cfg_attr(test, schema(nullable = false))]
    pub image_size: Option<ImageSize>,

    /// The MIME type of the image output.
    #[cfg_attr(test, schema(nullable = false))]
    pub mime_type: Option<ImageOutputMimeType>,
}

string_enum! {
    /// The aspect ratio for an image output.
    pub enum ImageAspectRatio {
        /// 1:1 aspect ratio.
        #[serde(rename = "1:1")]
        Ratio1x1,

        /// 2:3 aspect ratio.
        #[serde(rename = "2:3")]
        Ratio2x3,

        /// 3:2 aspect ratio.
        #[serde(rename = "3:2")]
        Ratio3x2,

        /// 3:4 aspect ratio.
        #[serde(rename = "3:4")]
        Ratio3x4,

        /// 4:3 aspect ratio.
        #[serde(rename = "4:3")]
        Ratio4x3,

        /// 4:5 aspect ratio.
        #[serde(rename = "4:5")]
        Ratio4x5,

        /// 5:4 aspect ratio.
        #[serde(rename = "5:4")]
        Ratio5x4,

        /// 9:16 aspect ratio.
        #[serde(rename = "9:16")]
        Ratio9x16,

        /// 16:9 aspect ratio.
        #[serde(rename = "16:9")]
        Ratio16x9,

        /// 21:9 aspect ratio.
        #[serde(rename = "21:9")]
        Ratio21x9,

        /// 1:8 aspect ratio.
        #[serde(rename = "1:8")]
        Ratio1x8,

        /// 8:1 aspect ratio.
        #[serde(rename = "8:1")]
        Ratio8x1,

        /// 1:4 aspect ratio.
        #[serde(rename = "1:4")]
        Ratio1x4,

        /// 4:1 aspect ratio.
        #[serde(rename = "4:1")]
        Ratio4x1,
    }
}

string_enum! {
    /// The size of an image output.
    pub enum ImageSize {
        /// 512px image size.
        #[serde(rename = "512")]
        Size512,

        /// 1K image size.
        #[serde(rename = "1K")]
        Size1K,

        /// 2K image size.
        #[serde(rename = "2K")]
        Size2K,

        /// 4K image size.
        #[serde(rename = "4K")]
        Size4K,
    }
}

string_enum! {
    /// The MIME type of an image output.
    pub enum ImageOutputMimeType {
        /// JPEG image format.
        #[serde(rename = "image/jpeg")]
        Jpeg,
    }
}

/// Configuration for audio output format.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[non_exhaustive]
pub struct AudioResponseFormat {
    /// Bit rate in bits per second (bps). Only applicable for compressed formats (MP3, Opus).
    #[cfg_attr(test, schema(nullable = false))]
    pub bit_rate: Option<i32>,

    /// The delivery mode for the audio output.
    #[cfg_attr(test, schema(nullable = false))]
    pub delivery: Option<Delivery>,

    /// The MIME type of the audio output.
    #[cfg_attr(test, schema(nullable = false))]
    pub mime_type: Option<AudioOutputMimeType>,

    /// Sample rate in Hz.
    #[cfg_attr(test, schema(nullable = false))]
    pub sample_rate: Option<i32>,
}

string_enum! {
    /// The MIME type of an audio output.
    pub enum AudioOutputMimeType {
        /// MP3 audio format.
        #[serde(rename = "audio/mp3")]
        Mp3,

        /// OGG Opus audio format.
        #[serde(rename = "audio/ogg_opus")]
        OggOpus,

        /// Raw PCM (L16) audio format.
        #[serde(rename = "audio/l16")]
        L16,

        /// WAV audio format.
        #[serde(rename = "audio/wav")]
        Wav,

        /// A-law audio format.
        #[serde(rename = "audio/alaw")]
        Alaw,

        /// Mu-law audio format.
        #[serde(rename = "audio/mulaw")]
        Mulaw,
    }
}

/// Configuration for video output format.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[non_exhaustive]
pub struct VideoResponseFormat {
    /// The aspect ratio for the video output.
    #[cfg_attr(test, schema(nullable = false))]
    pub aspect_ratio: Option<VideoAspectRatio>,

    /// The delivery mode for the video output.
    #[cfg_attr(test, schema(nullable = false))]
    pub delivery: Option<Delivery>,

    /// The duration for the video output.
    #[cfg_attr(test, schema(nullable = false, format = "google-duration"))]
    pub duration: Option<String>,

    /// The video output resolution. Defaults to 720p.
    #[cfg_attr(test, schema(nullable = false))]
    pub resolution: Option<VideoResolution>,
}

string_enum! {
    /// The aspect ratio for a video output.
    pub enum VideoAspectRatio {
        /// 16:9 aspect ratio.
        #[serde(rename = "16:9")]
        Ratio16x9,

        /// 9:16 aspect ratio.
        #[serde(rename = "9:16")]
        Ratio9x16,
    }
}

/// The resolution for a video output.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum VideoResolution {
    /// 360p resolution.
    #[serde(rename = "360p")]
    R360p,
    /// 720p resolution.
    #[serde(rename = "720p")]
    R720p,
    /// 1080p resolution.
    #[serde(rename = "1080p")]
    R1080p,
    /// 4K resolution.
    #[serde(rename = "4k")]
    R4K,
}

/// A response format specified by an arbitrary JSON object.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OtherResponseFormat(pub serde_json::Value);

impl ResponseFormat {
    /// Creates a text response format that constrains output to JSON matching the given
    /// JSON schema.
    pub fn json_schema(schema: schemars::Schema) -> Self {
        ResponseFormat::Text(TextResponseFormat {
            mime_type: Some(TextOutputMimeType::ApplicationJson),
            schema: Some(schema),
        })
    }

    /// Creates a text response format producing plain text.
    pub fn text_plain() -> Self {
        ResponseFormat::Text(TextResponseFormat {
            mime_type: Some(TextOutputMimeType::TextPlain),
            schema: None,
        })
    }
}

impl TextResponseFormat {
    /// Creates a new, empty text response format.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the MIME type of the text output.
    pub fn with_mime_type(mut self, mime_type: TextOutputMimeType) -> Self {
        self.mime_type = Some(mime_type);
        self
    }

    /// Sets the JSON schema that the output should conform to.
    pub fn with_schema(mut self, schema: schemars::Schema) -> Self {
        self.schema = Some(schema);
        self
    }
}

impl ImageResponseFormat {
    /// Creates a new, empty image response format.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the aspect ratio for the image output.
    pub fn with_aspect_ratio(mut self, aspect_ratio: ImageAspectRatio) -> Self {
        self.aspect_ratio = Some(aspect_ratio);
        self
    }

    /// Sets the delivery mode for the image output.
    pub fn with_delivery(mut self, delivery: Delivery) -> Self {
        self.delivery = Some(delivery);
        self
    }

    /// Sets the size of the image output.
    pub fn with_image_size(mut self, image_size: ImageSize) -> Self {
        self.image_size = Some(image_size);
        self
    }

    /// Sets the MIME type of the image output.
    pub fn with_mime_type(mut self, mime_type: ImageOutputMimeType) -> Self {
        self.mime_type = Some(mime_type);
        self
    }
}

impl AudioResponseFormat {
    /// Creates a new, empty audio response format.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the bit rate (bps) for compressed audio formats.
    pub fn with_bit_rate(mut self, bit_rate: i32) -> Self {
        self.bit_rate = Some(bit_rate);
        self
    }

    /// Sets the delivery mode for the audio output.
    pub fn with_delivery(mut self, delivery: Delivery) -> Self {
        self.delivery = Some(delivery);
        self
    }

    /// Sets the MIME type of the audio output.
    pub fn with_mime_type(mut self, mime_type: AudioOutputMimeType) -> Self {
        self.mime_type = Some(mime_type);
        self
    }

    /// Sets the sample rate (Hz) of the audio output.
    pub fn with_sample_rate(mut self, sample_rate: i32) -> Self {
        self.sample_rate = Some(sample_rate);
        self
    }
}

impl VideoResponseFormat {
    /// Creates a new, empty video response format.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the aspect ratio for the video output.
    pub fn with_aspect_ratio(mut self, aspect_ratio: VideoAspectRatio) -> Self {
        self.aspect_ratio = Some(aspect_ratio);
        self
    }

    /// Sets the delivery mode for the video output.
    pub fn with_delivery(mut self, delivery: Delivery) -> Self {
        self.delivery = Some(delivery);
        self
    }

    /// Sets the duration for the video output.
    pub fn with_duration(mut self, duration: impl Into<String>) -> Self {
        self.duration = Some(duration.into());
        self
    }
}

impl From<TextResponseFormat> for ResponseFormat {
    fn from(value: TextResponseFormat) -> Self {
        ResponseFormat::Text(value)
    }
}

impl From<ImageResponseFormat> for ResponseFormat {
    fn from(value: ImageResponseFormat) -> Self {
        ResponseFormat::Image(value)
    }
}

impl From<AudioResponseFormat> for ResponseFormat {
    fn from(value: AudioResponseFormat) -> Self {
        ResponseFormat::Audio(value)
    }
}

impl From<VideoResponseFormat> for ResponseFormat {
    fn from(value: VideoResponseFormat) -> Self {
        ResponseFormat::Video(value)
    }
}

#[cfg(test)]
impl utoipa::PartialSchema for OtherResponseFormat {
    fn schema() -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
        use utoipa::openapi::schema::AdditionalProperties;
        use utoipa::openapi::{ObjectBuilder, RefOr, Schema};

        RefOr::T(Schema::Object(
            ObjectBuilder::new()
                .additional_properties(Some(AdditionalProperties::FreeForm(true)))
                .build(),
        ))
    }
}

#[cfg(test)]
impl utoipa::ToSchema for OtherResponseFormat {}
