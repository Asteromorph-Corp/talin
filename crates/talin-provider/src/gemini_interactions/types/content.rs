use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_with::base64::Base64;
use serde_with::{serde_as, skip_serializing_none};

use crate::gemini_interactions::types::utils::impl_extra_accessors;
use crate::utils::{string_enum, tagged_enum};

tagged_enum! {
    /// A content block within a message.
    ///
    /// Content is discriminated by its `type` field.
    #[derive(Serialize, Deserialize, Debug, Clone)]
    #[serde(tag = "type", rename_all = "snake_case")]
    pub enum Content {
        /// A text content block.
        Text(TextContent),
        /// An image content block.
        Image(ImageContent),
        /// An audio content block.
        Audio(AudioContent),
        /// A video content block.
        Video(VideoContent),
        /// A document content block.
        Document(DocumentContent),
    }
}

tagged_enum! {
    /// A content block within a function result.
    #[derive(Serialize, Deserialize, Debug, Clone)]
    #[serde(tag = "type", rename_all = "snake_case")]
    pub enum FunctionResultSubContent {
        /// A text content block.
        Text(TextContent),
        /// An image content block.
        Image(ImageContent),
    }
}

tagged_enum! {
    /// A content block within a thought step.
    #[derive(Serialize, Deserialize, Debug, Clone)]
    #[serde(tag = "type", rename_all = "snake_case")]
    pub enum ThoughtContent {
        /// A text content block.
        Text(TextContent),
        /// An image content block.
        Image(ImageContent),
    }
}

/// A text content block.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct TextContent {
    /// Citation information for model-generated content.
    #[cfg_attr(test, schema(nullable = false))]
    pub annotations: Option<Vec<Annotation>>,

    /// The text content.
    pub text: String,

    /// Additional fields the server returned that this crate does not model; captured so
    /// they round-trip back to the API verbatim.
    #[serde(flatten)]
    #[cfg_attr(test, schema(ignore))]
    extra: BTreeMap<String, serde_json::Value>,
}

/// An image content block.
#[skip_serializing_none]
#[serde_as]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[non_exhaustive]
pub struct ImageContent {
    /// The image content bytes.
    #[serde_as(as = "Option<Base64>")]
    #[cfg_attr(test, schema(value_type = String, format = Byte))]
    pub data: Option<Vec<u8>>,

    /// The mime type of the image.
    #[cfg_attr(test, schema(nullable = false))]
    pub mime_type: Option<ImageMimeType>,

    /// The resolution of the media.
    #[cfg_attr(test, schema(nullable = false))]
    pub resolution: Option<MediaResolution>,

    /// The URI of the image.
    #[cfg_attr(test, schema(nullable = false))]
    pub uri: Option<String>,

    /// Additional fields the server returned that this crate does not model; captured so
    /// they round-trip back to the API verbatim.
    #[serde(flatten)]
    #[cfg_attr(test, schema(ignore))]
    extra: BTreeMap<String, serde_json::Value>,
}

/// An audio content block.
#[skip_serializing_none]
#[serde_as]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[non_exhaustive]
pub struct AudioContent {
    /// The number of audio channels.
    #[cfg_attr(test, schema(nullable = false))]
    pub channels: Option<i32>,

    /// The audio content bytes.
    #[serde_as(as = "Option<Base64>")]
    #[cfg_attr(test, schema(value_type = String, format = Byte))]
    pub data: Option<Vec<u8>>,

    /// The mime type of the audio.
    #[cfg_attr(test, schema(nullable = false))]
    pub mime_type: Option<AudioMimeType>,

    /// The sample rate of the audio.
    #[cfg_attr(test, schema(nullable = false))]
    pub sample_rate: Option<i32>,

    /// The URI of the audio.
    #[cfg_attr(test, schema(nullable = false))]
    pub uri: Option<String>,

    /// Additional fields the server returned that this crate does not model; captured so
    /// they round-trip back to the API verbatim.
    #[serde(flatten)]
    #[cfg_attr(test, schema(ignore))]
    extra: BTreeMap<String, serde_json::Value>,
}

/// A video content block.
#[skip_serializing_none]
#[serde_as]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[non_exhaustive]
pub struct VideoContent {
    /// The video content bytes.
    #[serde_as(as = "Option<Base64>")]
    #[cfg_attr(test, schema(value_type = String, format = Byte))]
    pub data: Option<Vec<u8>>,

    /// The mime type of the video.
    #[cfg_attr(test, schema(nullable = false))]
    pub mime_type: Option<VideoMimeType>,

    /// A user-defined name for this content block, which the model can reference in its final
    /// response.
    #[cfg_attr(test, schema(nullable = false))]
    pub name: Option<String>,

    /// How the model processes this video for understanding.
    #[cfg_attr(test, schema(nullable = false))]
    pub processing: Option<VideoProcessing>,

    /// The resolution of the media.
    #[cfg_attr(test, schema(nullable = false))]
    pub resolution: Option<MediaResolution>,

    /// The URI of the video.
    #[cfg_attr(test, schema(nullable = false))]
    pub uri: Option<String>,

    /// Additional fields the server returned that this crate does not model; captured so
    /// they round-trip back to the API verbatim.
    #[serde(flatten)]
    #[cfg_attr(test, schema(ignore))]
    extra: BTreeMap<String, serde_json::Value>,
}

/// How the model processes a video for understanding.
///
/// The API accepts either a bare [`VideoProcessingMode`] or a [`MediaProcessing`] configuration.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
#[non_exhaustive]
pub enum VideoProcessing {
    /// A processing mode, taking the defaults of that mode.
    Mode(VideoProcessingMode),
    /// A processing configuration.
    Config(MediaProcessing),
}

tagged_enum! {
    /// How the model processes a media block.
    ///
    /// Discriminated by its `type` field.
    #[derive(Serialize, Deserialize, Debug, Clone)]
    #[serde(tag = "type", rename_all = "snake_case")]
    pub enum MediaProcessing {
        /// Fixed-rate frame extraction.
        Static(StaticMediaProcessing),
    }
}

/// Fixed-rate frame extraction, optionally over a segment of the media.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[non_exhaustive]
pub struct StaticMediaProcessing {
    /// Segment end time, as a decimal number of seconds with an `s` suffix (e.g. `"30s"`).
    ///
    /// Must be non-negative, and greater than `start_offset` when that is set.
    #[cfg_attr(test, schema(nullable = false, format = "google-duration"))]
    pub end_offset: Option<String>,

    /// Video frame-rate sampling density.
    #[cfg_attr(test, schema(nullable = false))]
    pub fps: Option<f64>,

    /// Segment start time, as a decimal number of seconds with an `s` suffix (e.g. `"10.5s"`).
    ///
    /// Must be non-negative.
    #[cfg_attr(test, schema(nullable = false, format = "google-duration"))]
    pub start_offset: Option<String>,
}

string_enum! {
    /// A way the model processes a video for understanding.
    #[serde(rename_all = "snake_case")]
    pub enum VideoProcessingMode {
        /// Fixed-rate frame extraction. All frames placed in context.
        Static,

        /// Model-driven dynamic navigation.
        Agentic,
    }
}

/// A document content block.
#[skip_serializing_none]
#[serde_as]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[non_exhaustive]
pub struct DocumentContent {
    /// The document content bytes.
    #[serde_as(as = "Option<Base64>")]
    #[cfg_attr(test, schema(value_type = String, format = Byte))]
    pub data: Option<Vec<u8>>,

    /// The mime type of the document.
    #[cfg_attr(test, schema(nullable = false))]
    pub mime_type: Option<DocumentMimeType>,

    /// The URI of the document.
    #[cfg_attr(test, schema(nullable = false))]
    pub uri: Option<String>,

    /// Additional fields the server returned that this crate does not model; captured so
    /// they round-trip back to the API verbatim.
    #[serde(flatten)]
    #[cfg_attr(test, schema(ignore))]
    extra: BTreeMap<String, serde_json::Value>,
}

string_enum! {
    /// The mime type of an image.
    pub enum ImageMimeType {
        /// PNG image format.
        #[serde(rename = "image/png")]
        Png,

        /// JPEG image format.
        #[serde(rename = "image/jpeg")]
        Jpeg,

        /// WebP image format.
        #[serde(rename = "image/webp")]
        Webp,

        /// HEIC image format.
        #[serde(rename = "image/heic")]
        Heic,

        /// HEIF image format.
        #[serde(rename = "image/heif")]
        Heif,

        /// GIF image format.
        #[serde(rename = "image/gif")]
        Gif,

        /// BMP image format.
        #[serde(rename = "image/bmp")]
        Bmp,

        /// TIFF image format.
        #[serde(rename = "image/tiff")]
        Tiff,
    }
}

string_enum! {
    /// The mime type of an audio block.
    pub enum AudioMimeType {
        /// WAV audio format.
        #[serde(rename = "audio/wav")]
        Wav,

        /// MP3 audio format.
        #[serde(rename = "audio/mp3")]
        Mp3,

        /// AIFF audio format.
        #[serde(rename = "audio/aiff")]
        Aiff,

        /// AAC audio format.
        #[serde(rename = "audio/aac")]
        Aac,

        /// OGG audio format.
        #[serde(rename = "audio/ogg")]
        Ogg,

        /// FLAC audio format.
        #[serde(rename = "audio/flac")]
        Flac,

        /// MPEG audio format.
        #[serde(rename = "audio/mpeg")]
        Mpeg,

        /// M4A audio format.
        #[serde(rename = "audio/m4a")]
        M4a,

        /// L16 audio format.
        #[serde(rename = "audio/l16")]
        L16,

        /// OPUS audio format.
        #[serde(rename = "audio/opus")]
        Opus,

        /// ALAW audio format.
        #[serde(rename = "audio/alaw")]
        Alaw,

        /// MULAW audio format.
        #[serde(rename = "audio/mulaw")]
        Mulaw,

        /// WebM audio format.
        #[serde(rename = "audio/webm")]
        Webm,
    }
}

string_enum! {
    /// The mime type of a video block.
    pub enum VideoMimeType {
        /// MP4 video format.
        #[serde(rename = "video/mp4")]
        Mp4,

        /// MPEG video format.
        #[serde(rename = "video/mpeg")]
        Mpeg,

        /// MPG video format.
        #[serde(rename = "video/mpg")]
        Mpg,

        /// MOV video format.
        #[serde(rename = "video/mov")]
        Mov,

        /// AVI video format.
        #[serde(rename = "video/avi")]
        Avi,

        /// FLV video format.
        #[serde(rename = "video/x-flv")]
        Flv,

        /// WebM video format.
        #[serde(rename = "video/webm")]
        Webm,

        /// WMV video format.
        #[serde(rename = "video/wmv")]
        Wmv,

        /// 3GPP video format.
        #[serde(rename = "video/3gpp")]
        ThreeGpp,
    }
}

string_enum! {
    /// The mime type of a document block.
    pub enum DocumentMimeType {
        /// PDF document format.
        #[serde(rename = "application/pdf")]
        Pdf,

        /// CSV document format.
        #[serde(rename = "text/csv")]
        Csv,

        /// PowerPoint (OOXML) presentation format.
        #[serde(rename = "application/vnd.openxmlformats-officedocument.presentationml.presentation")]
        Pptx,

        /// Legacy PowerPoint presentation format.
        #[serde(rename = "application/vnd.ms-powerpoint")]
        Ppt,

        /// OpenDocument presentation format.
        #[serde(rename = "application/vnd.oasis.opendocument.presentation")]
        Odp,

        /// Word (OOXML) document format.
        #[serde(rename = "application/vnd.openxmlformats-officedocument.wordprocessingml.document")]
        Docx,

        /// Legacy Word document format.
        #[serde(rename = "application/msword")]
        Doc,

        /// Word (OOXML) template format.
        #[serde(rename = "application/vnd.openxmlformats-officedocument.wordprocessingml.template")]
        Dotx,

        /// OpenDocument text format.
        #[serde(rename = "application/vnd.oasis.opendocument.text")]
        Odt,

        /// Excel (OOXML) spreadsheet format.
        #[serde(rename = "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet")]
        Xlsx,

        /// Legacy Excel spreadsheet format.
        #[serde(rename = "application/vnd.ms-excel")]
        Xls,

        /// OpenDocument spreadsheet format.
        #[serde(rename = "application/vnd.oasis.opendocument.spreadsheet")]
        Ods,
    }
}

string_enum! {
    /// The resolution of media input.
    #[serde(rename_all = "snake_case")]
    pub enum MediaResolution {
        /// Low resolution.
        Low,

        /// Medium resolution.
        Medium,

        /// High resolution.
        High,

        /// Ultra high resolution.
        UltraHigh,
    }
}

tagged_enum! {
    /// Citation information for model-generated content.
    #[derive(Serialize, Deserialize, Debug, Clone)]
    #[serde(tag = "type", rename_all = "snake_case")]
    pub enum Annotation {
        /// A URL citation annotation.
        UrlCitation(UrlCitation),
        /// A file citation annotation.
        FileCitation(FileCitation),
        /// A place citation annotation.
        PlaceCitation(PlaceCitation),
        /// Word information annotation.
        WordInfo(WordInfo),
        /// Speech synthesis annotation.
        SpeechMetadata(SpeechAnnotation),
    }
}

/// A URL citation annotation.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[non_exhaustive]
pub struct UrlCitation {
    /// End of the attributed segment, exclusive.
    #[cfg_attr(test, schema(nullable = false))]
    pub end_index: Option<i32>,

    /// Start of segment of the response that is attributed to this source, in bytes.
    #[cfg_attr(test, schema(nullable = false))]
    pub start_index: Option<i32>,

    /// The title of the URL.
    #[cfg_attr(test, schema(nullable = false))]
    pub title: Option<String>,

    /// The URL.
    #[cfg_attr(test, schema(nullable = false))]
    pub url: Option<String>,
}

/// A file citation annotation.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[non_exhaustive]
pub struct FileCitation {
    /// User provided metadata about the retrieved context.
    #[cfg_attr(test, schema(schema_with = crate::gemini_interactions::spec::schema::object_schema))]
    pub custom_metadata: Option<serde_json::Value>,

    /// The URI of the file.
    #[cfg_attr(test, schema(nullable = false))]
    pub document_uri: Option<String>,

    /// End of the attributed segment, exclusive.
    #[cfg_attr(test, schema(nullable = false))]
    pub end_index: Option<i32>,

    /// The name of the file.
    #[cfg_attr(test, schema(nullable = false))]
    pub file_name: Option<String>,

    /// Media ID in-case of image citations, if applicable.
    #[cfg_attr(test, schema(nullable = false))]
    pub media_id: Option<String>,

    /// Page number of the cited document, if applicable.
    #[cfg_attr(test, schema(nullable = false))]
    pub page_number: Option<i32>,

    /// Source attributed for a portion of the text.
    #[cfg_attr(test, schema(nullable = false))]
    pub source: Option<String>,

    /// Start of segment of the response that is attributed to this source, in bytes.
    #[cfg_attr(test, schema(nullable = false))]
    pub start_index: Option<i32>,
}

/// A place citation annotation.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[non_exhaustive]
pub struct PlaceCitation {
    /// End of the attributed segment, exclusive.
    #[cfg_attr(test, schema(nullable = false))]
    pub end_index: Option<i32>,

    /// Title of the place.
    #[cfg_attr(test, schema(nullable = false))]
    pub name: Option<String>,

    /// The ID of the place, in `places/{place_id}` format.
    #[cfg_attr(test, schema(nullable = false))]
    pub place_id: Option<String>,

    /// Snippets of reviews used to generate answers about the features of a given place.
    #[cfg_attr(test, schema(nullable = false))]
    pub review_snippets: Option<Vec<ReviewSnippet>>,

    /// Start of segment of the response that is attributed to this source, in bytes.
    #[cfg_attr(test, schema(nullable = false))]
    pub start_index: Option<i32>,

    /// URI reference of the place.
    #[cfg_attr(test, schema(nullable = false))]
    pub url: Option<String>,
}

/// A snippet of a user review that answers a question about the features of a specific
/// place in Google Maps.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[non_exhaustive]
pub struct ReviewSnippet {
    /// The ID of the review snippet.
    #[cfg_attr(test, schema(nullable = false))]
    pub review_id: Option<String>,

    /// Title of the review.
    #[cfg_attr(test, schema(nullable = false))]
    pub title: Option<String>,

    /// A link that corresponds to the user review on Google Maps.
    #[cfg_attr(test, schema(nullable = false))]
    pub url: Option<String>,
}

/// Word-level timing and speaker information for a transcribed audio segment.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[non_exhaustive]
pub struct WordInfo {
    /// Start of segment of the response that is attributed to this source, in bytes.
    #[cfg_attr(test, schema(nullable = false))]
    pub start_index: Option<i32>,

    /// End of the attributed segment, exclusive.
    #[cfg_attr(test, schema(nullable = false))]
    pub end_index: Option<i32>,

    /// Start offset in time of the word relative to the start of the audio.
    ///
    /// Present when `timestamp_granularities` contains `"word"`.
    #[cfg_attr(test, schema(nullable = false, format = "google-duration"))]
    pub start_offset: Option<String>,

    /// End offset in time of the word relative to the start of the audio.
    ///
    /// Present when `timestamp_granularities` contains `"word"`.
    #[cfg_attr(test, schema(nullable = false, format = "google-duration"))]
    pub end_offset: Option<String>,

    /// Speaker label for this word (e.g. `"spk_1"`, `"spk_2"`).
    ///
    /// Present when `diarization_mode` is set in `TranscriptionConfig`.
    #[cfg_attr(test, schema(nullable = false))]
    pub speaker: Option<String>,

    /// The transcribed word.
    #[cfg_attr(test, schema(nullable = false))]
    pub text: Option<String>,
}

impl Content {
    /// Creates a text content block.
    pub fn text(text: impl Into<String>) -> Self {
        Content::Text(TextContent {
            annotations: None,
            text: text.into(),
            extra: BTreeMap::new(),
        })
    }

    /// Creates an image content block from raw bytes.
    pub fn image(mime_type: ImageMimeType, data: Vec<u8>) -> Self {
        Content::Image(ImageContent {
            data: Some(data),
            mime_type: Some(mime_type),
            resolution: None,
            uri: None,
            extra: BTreeMap::new(),
        })
    }

    /// Creates an audio content block from raw bytes.
    pub fn audio(mime_type: AudioMimeType, data: Vec<u8>) -> Self {
        Content::Audio(AudioContent {
            channels: None,
            data: Some(data),
            mime_type: Some(mime_type),
            sample_rate: None,
            uri: None,
            extra: BTreeMap::new(),
        })
    }

    /// Creates an image content block referencing a URI.
    pub fn image_uri(mime_type: ImageMimeType, uri: impl Into<String>) -> Self {
        Content::Image(ImageContent {
            data: None,
            mime_type: Some(mime_type),
            resolution: None,
            uri: Some(uri.into()),
            extra: BTreeMap::new(),
        })
    }

    /// Creates an audio content block referencing a URI.
    pub fn audio_uri(mime_type: AudioMimeType, uri: impl Into<String>) -> Self {
        Content::Audio(AudioContent {
            channels: None,
            data: None,
            mime_type: Some(mime_type),
            sample_rate: None,
            uri: Some(uri.into()),
            extra: BTreeMap::new(),
        })
    }

    /// Creates a video content block from raw bytes.
    pub fn video(mime_type: VideoMimeType, data: Vec<u8>) -> Self {
        Content::Video(VideoContent {
            data: Some(data),
            mime_type: Some(mime_type),
            name: None,
            processing: None,
            resolution: None,
            uri: None,
            extra: BTreeMap::new(),
        })
    }

    /// Creates a video content block referencing a URI.
    pub fn video_uri(mime_type: VideoMimeType, uri: impl Into<String>) -> Self {
        Content::Video(VideoContent {
            data: None,
            mime_type: Some(mime_type),
            name: None,
            processing: None,
            resolution: None,
            uri: Some(uri.into()),
            extra: BTreeMap::new(),
        })
    }

    /// Creates a document content block from raw bytes.
    pub fn document(mime_type: DocumentMimeType, data: Vec<u8>) -> Self {
        Content::Document(DocumentContent {
            data: Some(data),
            mime_type: Some(mime_type),
            uri: None,
            extra: BTreeMap::new(),
        })
    }

    /// Creates a document content block referencing a URI.
    pub fn document_uri(mime_type: DocumentMimeType, uri: impl Into<String>) -> Self {
        Content::Document(DocumentContent {
            data: None,
            mime_type: Some(mime_type),
            uri: Some(uri.into()),
            extra: BTreeMap::new(),
        })
    }

    /// Returns the text of this content block, if it is a text block.
    pub fn as_text(&self) -> Option<&str> {
        match self {
            Content::Text(t) => Some(&t.text),
            _ => None,
        }
    }
}

impl TextContent {
    /// Creates a new text content block.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            annotations: None,
            text: text.into(),
            extra: BTreeMap::new(),
        }
    }

    /// Sets the citation annotations for this text block.
    pub fn with_annotations(mut self, annotations: Vec<Annotation>) -> Self {
        self.annotations = Some(annotations);
        self
    }
}

impl ImageContent {
    /// Creates a new, empty image content block.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the inline image bytes.
    pub fn with_data(mut self, data: Vec<u8>) -> Self {
        self.data = Some(data);
        self
    }

    /// Sets the mime type of the image.
    pub fn with_mime_type(mut self, mime_type: ImageMimeType) -> Self {
        self.mime_type = Some(mime_type);
        self
    }

    /// Sets the media resolution.
    pub fn with_resolution(mut self, resolution: MediaResolution) -> Self {
        self.resolution = Some(resolution);
        self
    }

    /// Sets the URI of the image.
    pub fn with_uri(mut self, uri: impl Into<String>) -> Self {
        self.uri = Some(uri.into());
        self
    }
}

impl AudioContent {
    /// Creates a new, empty audio content block.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the number of audio channels.
    pub fn with_channels(mut self, channels: i32) -> Self {
        self.channels = Some(channels);
        self
    }

    /// Sets the inline audio bytes.
    pub fn with_data(mut self, data: Vec<u8>) -> Self {
        self.data = Some(data);
        self
    }

    /// Sets the mime type of the audio.
    pub fn with_mime_type(mut self, mime_type: AudioMimeType) -> Self {
        self.mime_type = Some(mime_type);
        self
    }

    /// Sets the sample rate of the audio.
    pub fn with_sample_rate(mut self, sample_rate: i32) -> Self {
        self.sample_rate = Some(sample_rate);
        self
    }

    /// Sets the URI of the audio.
    pub fn with_uri(mut self, uri: impl Into<String>) -> Self {
        self.uri = Some(uri.into());
        self
    }
}

impl VideoContent {
    /// Creates a new, empty video content block.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the inline video bytes.
    pub fn with_data(mut self, data: Vec<u8>) -> Self {
        self.data = Some(data);
        self
    }

    /// Sets the mime type of the video.
    pub fn with_mime_type(mut self, mime_type: VideoMimeType) -> Self {
        self.mime_type = Some(mime_type);
        self
    }

    /// Sets the user-defined name of the video.
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Sets how the model processes the video.
    pub fn with_processing(mut self, processing: impl Into<VideoProcessing>) -> Self {
        self.processing = Some(processing.into());
        self
    }

    /// Sets the media resolution.
    pub fn with_resolution(mut self, resolution: MediaResolution) -> Self {
        self.resolution = Some(resolution);
        self
    }

    /// Sets the URI of the video.
    pub fn with_uri(mut self, uri: impl Into<String>) -> Self {
        self.uri = Some(uri.into());
        self
    }
}

impl StaticMediaProcessing {
    /// Creates a new, empty static processing configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the segment end time.
    pub fn with_end_offset(mut self, end_offset: impl Into<String>) -> Self {
        self.end_offset = Some(end_offset.into());
        self
    }

    /// Sets the frame-rate sampling density.
    pub fn with_fps(mut self, fps: f64) -> Self {
        self.fps = Some(fps);
        self
    }

    /// Sets the segment start time.
    pub fn with_start_offset(mut self, start_offset: impl Into<String>) -> Self {
        self.start_offset = Some(start_offset.into());
        self
    }
}

impl DocumentContent {
    /// Creates a new, empty document content block.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the inline document bytes.
    pub fn with_data(mut self, data: Vec<u8>) -> Self {
        self.data = Some(data);
        self
    }

    /// Sets the mime type of the document.
    pub fn with_mime_type(mut self, mime_type: DocumentMimeType) -> Self {
        self.mime_type = Some(mime_type);
        self
    }

    /// Sets the URI of the document.
    pub fn with_uri(mut self, uri: impl Into<String>) -> Self {
        self.uri = Some(uri.into());
        self
    }
}

impl UrlCitation {
    /// Creates a new, empty URL citation.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the end of the attributed segment.
    pub fn with_end_index(mut self, end_index: i32) -> Self {
        self.end_index = Some(end_index);
        self
    }

    /// Sets the start of the attributed segment.
    pub fn with_start_index(mut self, start_index: i32) -> Self {
        self.start_index = Some(start_index);
        self
    }

    /// Sets the title of the URL.
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets the URL.
    pub fn with_url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }
}

impl FileCitation {
    /// Creates a new, empty file citation.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the metadata about the retrieved context.
    pub fn with_custom_metadata(mut self, custom_metadata: serde_json::Value) -> Self {
        self.custom_metadata = Some(custom_metadata);
        self
    }

    /// Sets the URI of the file.
    pub fn with_document_uri(mut self, document_uri: impl Into<String>) -> Self {
        self.document_uri = Some(document_uri.into());
        self
    }

    /// Sets the end of the attributed segment.
    pub fn with_end_index(mut self, end_index: i32) -> Self {
        self.end_index = Some(end_index);
        self
    }

    /// Sets the name of the file.
    pub fn with_file_name(mut self, file_name: impl Into<String>) -> Self {
        self.file_name = Some(file_name.into());
        self
    }

    /// Sets the media ID of the citation.
    pub fn with_media_id(mut self, media_id: impl Into<String>) -> Self {
        self.media_id = Some(media_id.into());
        self
    }

    /// Sets the page number of the cited document.
    pub fn with_page_number(mut self, page_number: i32) -> Self {
        self.page_number = Some(page_number);
        self
    }

    /// Sets the source attributed for the portion of the text.
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }

    /// Sets the start of the attributed segment.
    pub fn with_start_index(mut self, start_index: i32) -> Self {
        self.start_index = Some(start_index);
        self
    }
}

impl PlaceCitation {
    /// Creates a new, empty place citation.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the end of the attributed segment.
    pub fn with_end_index(mut self, end_index: i32) -> Self {
        self.end_index = Some(end_index);
        self
    }

    /// Sets the title of the place.
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Sets the ID of the place.
    pub fn with_place_id(mut self, place_id: impl Into<String>) -> Self {
        self.place_id = Some(place_id.into());
        self
    }

    /// Sets the review snippets for the place.
    pub fn with_review_snippets(mut self, review_snippets: Vec<ReviewSnippet>) -> Self {
        self.review_snippets = Some(review_snippets);
        self
    }

    /// Sets the start of the attributed segment.
    pub fn with_start_index(mut self, start_index: i32) -> Self {
        self.start_index = Some(start_index);
        self
    }

    /// Sets the URI reference of the place.
    pub fn with_url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }
}

impl ReviewSnippet {
    /// Creates a new, empty review snippet.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the ID of the review snippet.
    pub fn with_review_id(mut self, review_id: impl Into<String>) -> Self {
        self.review_id = Some(review_id.into());
        self
    }

    /// Sets the title of the review.
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets the link to the user review on Google Maps.
    pub fn with_url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }
}

impl WordInfo {
    /// Creates a new, empty word information annotation.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the start of the attributed segment.
    pub fn with_start_index(mut self, start_index: i32) -> Self {
        self.start_index = Some(start_index);
        self
    }

    /// Sets the end of the attributed segment.
    pub fn with_end_index(mut self, end_index: i32) -> Self {
        self.end_index = Some(end_index);
        self
    }

    /// Sets the start offset in time of the word.
    pub fn with_start_offset(mut self, start_offset: impl Into<String>) -> Self {
        self.start_offset = Some(start_offset.into());
        self
    }

    /// Sets the end offset in time of the word.
    pub fn with_end_offset(mut self, end_offset: impl Into<String>) -> Self {
        self.end_offset = Some(end_offset.into());
        self
    }

    /// Sets the speaker label for the word.
    pub fn with_speaker(mut self, speaker: impl Into<String>) -> Self {
        self.speaker = Some(speaker.into());
        self
    }

    /// Sets the transcribed word.
    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.text = Some(text.into());
        self
    }
}

/// Speech annotation for text content.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[non_exhaustive]
pub struct SpeechAnnotation {
    /// End of the attributed segment, exclusive.
    #[cfg_attr(test, schema(nullable = false))]
    pub end_index: Option<i32>,

    /// The speaker to associate with this turn.
    #[cfg_attr(test, schema(nullable = false))]
    pub speaker: Option<String>,

    /// Start of segment of the response that is attributed to this source, in bytes.
    #[cfg_attr(test, schema(nullable = false))]
    pub start_index: Option<i32>,

    /// Style instruction for the speech synthesis.
    #[cfg_attr(test, schema(nullable = false))]
    pub style: Option<String>,
}

impl SpeechAnnotation {
    /// Creates a new, empty speech annotation.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the start of the attributed segment.
    pub fn with_start_index(mut self, start_index: i32) -> Self {
        self.start_index = Some(start_index);
        self
    }

    /// Sets the end of the attributed segment.
    pub fn with_end_index(mut self, end_index: i32) -> Self {
        self.end_index = Some(end_index);
        self
    }

    /// Sets the speaker to associate with this turn.
    pub fn with_speaker(mut self, speaker: impl Into<String>) -> Self {
        self.speaker = Some(speaker.into());
        self
    }

    /// Sets the style instruction for the speech synthesis.
    pub fn with_style(mut self, style: impl Into<String>) -> Self {
        self.style = Some(style.into());
        self
    }
}

impl From<TextContent> for Content {
    fn from(value: TextContent) -> Self {
        Content::Text(value)
    }
}

impl From<ImageContent> for Content {
    fn from(value: ImageContent) -> Self {
        Content::Image(value)
    }
}

impl From<AudioContent> for Content {
    fn from(value: AudioContent) -> Self {
        Content::Audio(value)
    }
}

impl From<VideoContent> for Content {
    fn from(value: VideoContent) -> Self {
        Content::Video(value)
    }
}

impl From<DocumentContent> for Content {
    fn from(value: DocumentContent) -> Self {
        Content::Document(value)
    }
}

impl From<MediaProcessing> for VideoProcessing {
    fn from(value: MediaProcessing) -> Self {
        VideoProcessing::Config(value)
    }
}

impl From<StaticMediaProcessing> for VideoProcessing {
    fn from(value: StaticMediaProcessing) -> Self {
        VideoProcessing::Config(MediaProcessing::Static(value))
    }
}

impl From<VideoProcessingMode> for VideoProcessing {
    fn from(value: VideoProcessingMode) -> Self {
        VideoProcessing::Mode(value)
    }
}

impl_extra_accessors!(
    TextContent,
    ImageContent,
    AudioContent,
    VideoContent,
    DocumentContent,
);
