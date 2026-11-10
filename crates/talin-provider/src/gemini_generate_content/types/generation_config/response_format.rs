use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// Configuration for the response output format.
///
/// Allows specifying output configuration per modality (text, audio, image) in a
/// flat structure.
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ResponseFormatConfig {
    /// Optional. Text output format configuration.
    pub text: Option<TextResponseFormat>,

    /// Optional. Image output format configuration.
    pub image: Option<ImageResponseFormat>,

    /// Optional. Audio output format configuration.
    pub audio: Option<AudioResponseFormat>,
}

impl ResponseFormatConfig {
    /// Creates a new ResponseFormatConfig with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the text output format configuration.
    pub fn with_text(mut self, text: TextResponseFormat) -> Self {
        self.text = Some(text);
        self
    }

    /// Sets the image output format configuration.
    pub fn with_image(mut self, image: ImageResponseFormat) -> Self {
        self.image = Some(image);
        self
    }

    /// Sets the audio output format configuration.
    pub fn with_audio(mut self, audio: AudioResponseFormat) -> Self {
        self.audio = Some(audio);
        self
    }
}

/// Text output format configuration.
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct TextResponseFormat {
    /// Optional. The MIME type of the text output.
    pub mime_type: Option<TextMimeType>,

    /// Optional. The JSON schema that the output should conform to.
    /// Only applicable when the MIME type is `APPLICATION_JSON`.
    pub schema: Option<serde_json::Value>,
}

impl TextResponseFormat {
    /// Creates a new TextResponseFormat with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the MIME type of the text output.
    pub fn with_mime_type(mut self, mime_type: TextMimeType) -> Self {
        self.mime_type = Some(mime_type);
        self
    }

    /// Sets the JSON schema that the output should conform to.
    pub fn with_schema(mut self, schema: serde_json::Value) -> Self {
        self.schema = Some(schema);
        self
    }
}

/// The MIME type of a text output.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum TextMimeType {
    /// Default value. Unspecified MIME type.
    MimeTypeUnspecified,
    /// JSON output (`application/json`).
    ApplicationJson,
    /// Plain text output (`text/plain`).
    TextPlain,
    /// Unknown value not represented by this enum.
    #[serde(other)]
    Unknown,
}

/// Image output format configuration.
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ImageResponseFormat {
    /// Optional. The MIME type of the image output.
    pub mime_type: Option<ImageMimeType>,

    /// Optional. The size of the image output.
    pub image_size: Option<ImageOutputSize>,

    /// Optional. The delivery mode for the image output.
    pub delivery: Option<MediaDelivery>,

    /// Optional. The aspect ratio for the image output.
    pub aspect_ratio: Option<ImageOutputAspectRatio>,
}

impl ImageResponseFormat {
    /// Creates a new ImageResponseFormat with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the MIME type of the image output.
    pub fn with_mime_type(mut self, mime_type: ImageMimeType) -> Self {
        self.mime_type = Some(mime_type);
        self
    }

    /// Sets the size of the image output.
    pub fn with_image_size(mut self, image_size: ImageOutputSize) -> Self {
        self.image_size = Some(image_size);
        self
    }

    /// Sets the delivery mode for the image output.
    pub fn with_delivery(mut self, delivery: MediaDelivery) -> Self {
        self.delivery = Some(delivery);
        self
    }

    /// Sets the aspect ratio for the image output.
    pub fn with_aspect_ratio(mut self, aspect_ratio: ImageOutputAspectRatio) -> Self {
        self.aspect_ratio = Some(aspect_ratio);
        self
    }
}

/// The MIME type of an image output.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum ImageMimeType {
    /// Default value. Unspecified MIME type.
    MimeTypeUnspecified,
    /// JPEG image output (`image/jpeg`).
    ImageJpeg,
    /// Unknown value not represented by this enum.
    #[serde(other)]
    Unknown,
}

/// The size of an image output.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum ImageOutputSize {
    /// Default value. Unspecified image size.
    ImageSizeUnspecified,
    /// 512px image size.
    ImageSizeFiveTwelve,
    /// 1K image size.
    ImageSizeOneK,
    /// 2K image size.
    ImageSizeTwoK,
    /// 4K image size.
    ImageSizeFourK,
    /// Unknown value not represented by this enum.
    #[serde(other)]
    Unknown,
}

/// The aspect ratio for an image output.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum ImageOutputAspectRatio {
    /// Default value. Unspecified aspect ratio.
    AspectRatioUnspecified,
    /// 1:1 aspect ratio.
    AspectRatioOneByOne,
    /// 2:3 aspect ratio.
    AspectRatioTwoByThree,
    /// 3:2 aspect ratio.
    AspectRatioThreeByTwo,
    /// 3:4 aspect ratio.
    AspectRatioThreeByFour,
    /// 4:3 aspect ratio.
    AspectRatioFourByThree,
    /// 4:5 aspect ratio.
    AspectRatioFourByFive,
    /// 5:4 aspect ratio.
    AspectRatioFiveByFour,
    /// 9:16 aspect ratio.
    AspectRatioNineBySixteen,
    /// 16:9 aspect ratio.
    AspectRatioSixteenByNine,
    /// 21:9 aspect ratio.
    AspectRatioTwentyOneByNine,
    /// 1:8 aspect ratio.
    AspectRatioOneByEight,
    /// 8:1 aspect ratio.
    AspectRatioEightByOne,
    /// 1:4 aspect ratio.
    AspectRatioOneByFour,
    /// 4:1 aspect ratio.
    AspectRatioFourByOne,
    /// Unknown value not represented by this enum.
    #[serde(other)]
    Unknown,
}

/// Audio output format configuration.
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct AudioResponseFormat {
    /// Optional. The MIME type of the audio output.
    pub mime_type: Option<AudioMimeType>,

    /// Optional. The delivery mode for the audio output.
    pub delivery: Option<MediaDelivery>,

    /// Optional. Sample rate in Hz.
    pub sample_rate: Option<i32>,

    /// Optional. Bit rate in bits per second (bps).
    /// Only applicable for compressed formats.
    pub bit_rate: Option<i32>,
}

/// The MIME type of an audio output.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum AudioMimeType {
    /// Default value. Unspecified MIME type.
    MimeTypeUnspecified,
    /// MP3 audio output.
    AudioMp3,
    /// Ogg Opus audio output.
    AudioOggOpus,
    /// Linear16 PCM audio output.
    AudioL16,
    /// WAV audio output.
    AudioWav,
    /// A-law audio output.
    AudioAlaw,
    /// mu-law audio output.
    AudioMulaw,
    /// Unknown value not represented by this enum.
    #[serde(other)]
    Unknown,
}

/// The delivery mode for a media output.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum MediaDelivery {
    /// Default value. Unspecified delivery mode.
    DeliveryUnspecified,
    /// The media is delivered inline in the response.
    Inline,
    /// The media is delivered as a URI reference.
    Uri,
    /// Unknown value not represented by this enum.
    #[serde(other)]
    Unknown,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wire<T: Serialize>(v: &T) -> String {
        serde_json::to_string(v).unwrap()
    }

    #[test]
    fn test_enum_wire_values() {
        assert_eq!(wire(&TextMimeType::ApplicationJson), "\"APPLICATION_JSON\"");
        assert_eq!(wire(&ImageMimeType::ImageJpeg), "\"IMAGE_JPEG\"");
        assert_eq!(wire(&AudioMimeType::AudioL16), "\"AUDIO_L16\"");
        assert_eq!(wire(&AudioMimeType::AudioOggOpus), "\"AUDIO_OGG_OPUS\"");
        assert_eq!(wire(&MediaDelivery::Uri), "\"URI\"");
        assert_eq!(
            wire(&ImageOutputSize::ImageSizeFiveTwelve),
            "\"IMAGE_SIZE_FIVE_TWELVE\""
        );
        assert_eq!(
            wire(&ImageOutputSize::ImageSizeOneK),
            "\"IMAGE_SIZE_ONE_K\""
        );
        assert_eq!(
            wire(&ImageOutputAspectRatio::AspectRatioTwentyOneByNine),
            "\"ASPECT_RATIO_TWENTY_ONE_BY_NINE\""
        );
        assert_eq!(
            wire(&ImageOutputAspectRatio::AspectRatioNineBySixteen),
            "\"ASPECT_RATIO_NINE_BY_SIXTEEN\""
        );
    }

    #[test]
    fn test_unknown_fallback() {
        // Unrecognized server values deserialize to the catch-all instead of erroring.
        let d: MediaDelivery = serde_json::from_str("\"SOMETHING_NEW\"").unwrap();
        assert_eq!(d, MediaDelivery::Unknown);
    }
}
