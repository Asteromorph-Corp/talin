use serde::{Deserialize, Serialize};
use serde_with::base64::Base64;
use serde_with::{serde_as, skip_serializing_none};

use super::{
    Annotation, AudioMimeType, CodeExecutionCallArguments, Content, DocumentMimeType,
    FunctionResultValue, GoogleMapsCallArguments, GoogleMapsResult, GoogleSearchCallArguments,
    GoogleSearchResult, ImageMimeType, MediaResolution, UrlContextCallArguments, UrlContextResult,
};
use crate::utils::{string_enum, tagged_enum};

tagged_enum! {
    /// The delta content data for a content block, streamed incrementally.
    ///
    /// Discriminated by its `type` field.
    #[derive(Serialize, Deserialize, Debug, Clone)]
    #[serde(tag = "type", rename_all = "snake_case")]
    pub enum ContentDeltaData {
        /// An incremental text update.
        Text(TextDelta),
        /// An incremental image update.
        Image(ImageDelta),
        /// An incremental audio update.
        Audio(AudioDelta),
        /// An incremental document update.
        Document(DocumentDelta),
        /// An incremental thought-summary update.
        ThoughtSummary(ThoughtSummaryDelta),
        /// A thought signature.
        ThoughtSignature(ThoughtSignatureDelta),
        /// A function call.
        FunctionCall(FunctionCallDelta),
        /// A code execution call.
        CodeExecutionCall(CodeExecutionCallDelta),
        /// A URL context call.
        UrlContextCall(UrlContextCallDelta),
        /// A Google Search call.
        GoogleSearchCall(GoogleSearchCallDelta),
        /// A file search call.
        FileSearchCall(FileSearchCallDelta),
        /// A Google Maps call.
        GoogleMapsCall(GoogleMapsCallDelta),
        /// A function result.
        FunctionResult(FunctionResultDelta),
        /// A code execution result.
        CodeExecutionResult(CodeExecutionResultDelta),
        /// A URL context result.
        UrlContextResult(UrlContextResultDelta),
        /// A Google Search result.
        GoogleSearchResult(GoogleSearchResultDelta),
        /// A file search result.
        FileSearchResult(FileSearchResultDelta),
        /// A Google Maps result.
        GoogleMapsResult(GoogleMapsResultDelta),
        /// A text annotation update.
        #[serde(rename = "text_annotation_delta")]
        TextAnnotation(TextAnnotationDelta),
    }
}

tagged_enum! {
    /// The delta data for a step, streamed incrementally.
    ///
    /// Discriminated by its `type` field.
    #[derive(Serialize, Deserialize, Debug, Clone)]
    #[serde(tag = "type", rename_all = "snake_case")]
    pub enum StepDeltaData {
        /// An incremental text update.
        Text(TextDelta),
        /// An incremental image update.
        Image(ImageDelta),
        /// An incremental audio update.
        Audio(AudioDelta),
        /// An incremental video update.
        Video(VideoDelta),
        /// An incremental document update.
        Document(DocumentDelta),
        /// An incremental thought-summary update.
        ThoughtSummary(ThoughtSummaryDelta),
        /// A thought signature.
        ThoughtSignature(ThoughtSignatureDelta),
        /// A text annotation update.
        #[serde(rename = "text_annotation_delta")]
        TextAnnotation(TextAnnotationDelta),
        /// An incremental tool-call arguments update.
        #[serde(rename = "arguments_delta")]
        Arguments(ArgumentsDelta),
        /// A code execution call.
        CodeExecutionCall(CodeExecutionCallDelta),
        /// A URL context call.
        UrlContextCall(UrlContextCallDelta),
        /// A Google Search call.
        GoogleSearchCall(GoogleSearchCallDelta),
        /// A file search call.
        FileSearchCall(FileSearchCallDelta),
        /// A Google Maps call.
        GoogleMapsCall(GoogleMapsCallDelta),
        /// An MCP server tool call.
        McpServerToolCall(McpServerToolCallDelta),
        /// A processing call.
        ProcessingCall(ProcessingCallDelta),
        /// A code execution result.
        CodeExecutionResult(CodeExecutionResultDelta),
        /// A URL context result.
        UrlContextResult(UrlContextResultDelta),
        /// A Google Search result.
        GoogleSearchResult(GoogleSearchResultDelta),
        /// A file search result.
        FileSearchResult(FileSearchResultDelta),
        /// A Google Maps result.
        GoogleMapsResult(GoogleMapsResultDelta),
        /// A function result.
        FunctionResult(FunctionResultDelta),
        /// An MCP server tool result.
        McpServerToolResult(McpServerToolResultDelta),
        /// A processing result.
        ProcessingResult(ProcessingResultDelta),
    }
}

/// An incremental text update.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct TextDelta {
    /// The incremental text.
    pub text: String,
}

/// An incremental image update.
#[skip_serializing_none]
#[serde_as]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ImageDelta {
    /// The image content bytes.
    #[serde_as(as = "Option<Base64>")]
    #[cfg_attr(test, schema(value_type = String, format = Byte, nullable = false))]
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
}

/// An incremental audio update.
#[skip_serializing_none]
#[serde_as]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct AudioDelta {
    /// The number of audio channels.
    #[cfg_attr(test, schema(nullable = false))]
    pub channels: Option<i32>,

    /// The audio content bytes.
    #[serde_as(as = "Option<Base64>")]
    #[cfg_attr(test, schema(value_type = String, format = Byte, nullable = false))]
    pub data: Option<Vec<u8>>,

    /// The mime type of the audio.
    #[cfg_attr(test, schema(nullable = false))]
    pub mime_type: Option<AudioMimeType>,

    /// The sample rate of the audio. The value is ignored.
    #[deprecated(note = "use `sample_rate` instead")]
    #[cfg_attr(test, schema(nullable = false))]
    pub rate: Option<i32>,

    /// The sample rate of the audio.
    #[cfg_attr(test, schema(nullable = false))]
    pub sample_rate: Option<i32>,

    /// The URI of the audio.
    #[cfg_attr(test, schema(nullable = false))]
    pub uri: Option<String>,
}

/// An incremental video update.
#[skip_serializing_none]
#[serde_as]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct VideoDelta {
    /// The video content bytes.
    #[serde_as(as = "Option<Base64>")]
    #[cfg_attr(test, schema(value_type = Option<String>, format = Byte, nullable = false))]
    pub data: Option<Vec<u8>>,

    /// The mime type of the video.
    #[cfg_attr(test, schema(nullable = false))]
    pub mime_type: Option<VideoDeltaMimeType>,

    /// The resolution of the media.
    #[cfg_attr(test, schema(nullable = false))]
    pub resolution: Option<MediaResolution>,

    /// The URI of the video.
    #[cfg_attr(test, schema(nullable = false))]
    pub uri: Option<String>,
}

string_enum! {
    /// The mime type of a video block.
    pub enum VideoDeltaMimeType {
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

        /// JPEG2000 video format.
        #[serde(rename = "video/jpeg2000")]
        Jpeg2000,
    }
}

/// An incremental document update.
#[skip_serializing_none]
#[serde_as]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct DocumentDelta {
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
}

/// An incremental thought-summary update.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ThoughtSummaryDelta {
    /// A new summary item to be added to the thought.
    #[cfg_attr(test, schema(nullable = false))]
    pub content: Option<Content>,
}

/// A thought signature.
#[skip_serializing_none]
#[serde_as]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ThoughtSignatureDelta {
    /// Signature to match the backend source to be part of the generation.
    #[serde_as(as = "Option<Base64>")]
    #[cfg_attr(test, schema(value_type = String, format = Byte, nullable = false))]
    pub signature: Option<Vec<u8>>,
}

/// A streamed function call.
#[skip_serializing_none]
#[serde_as]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct FunctionCallDelta {
    /// The arguments to pass to the function.
    #[cfg_attr(test, schema(value_type = Object))]
    pub arguments: serde_json::Value,

    /// A unique ID for this specific tool call.
    pub id: String,

    /// The name of the tool to call.
    pub name: String,

    /// An opaque signature the backend requires to be echoed back verbatim on subsequent
    /// requests to validate the tool-call chain.
    #[serde_as(as = "Option<Base64>")]
    #[cfg_attr(test, schema(value_type = String, format = Byte))]
    pub signature: Option<Vec<u8>>,
}

/// A streamed code execution call.
#[skip_serializing_none]
#[serde_as]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct CodeExecutionCallDelta {
    /// The arguments to pass to the code execution.
    pub arguments: CodeExecutionCallArguments,

    /// A signature hash for backend validation.
    #[serde_as(as = "Option<Base64>")]
    #[cfg_attr(test, schema(value_type = String, format = Byte))]
    pub signature: Option<Vec<u8>>,
}

/// A streamed URL context call.
#[skip_serializing_none]
#[serde_as]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct UrlContextCallDelta {
    /// The arguments to pass to the URL context tool.
    pub arguments: UrlContextCallArguments,

    /// A signature hash for backend validation.
    #[serde_as(as = "Option<Base64>")]
    #[cfg_attr(test, schema(value_type = String, format = Byte))]
    pub signature: Option<Vec<u8>>,
}

/// A streamed Google Search call.
#[skip_serializing_none]
#[serde_as]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct GoogleSearchCallDelta {
    /// The arguments to pass to Google Search.
    pub arguments: GoogleSearchCallArguments,

    /// A signature hash for backend validation.
    #[serde_as(as = "Option<Base64>")]
    #[cfg_attr(test, schema(value_type = String, format = Byte))]
    pub signature: Option<Vec<u8>>,
}

/// A streamed file search call.
#[skip_serializing_none]
#[serde_as]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct FileSearchCallDelta {
    /// A signature hash for backend validation.
    #[serde_as(as = "Option<Base64>")]
    #[cfg_attr(test, schema(value_type = String, format = Byte))]
    pub signature: Option<Vec<u8>>,
}

/// A streamed Google Maps call.
#[skip_serializing_none]
#[serde_as]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct GoogleMapsCallDelta {
    /// The arguments to pass to the Google Maps tool.
    #[cfg_attr(test, schema(nullable = false))]
    pub arguments: Option<GoogleMapsCallArguments>,

    /// A signature hash for backend validation.
    #[serde_as(as = "Option<Base64>")]
    #[cfg_attr(test, schema(value_type = String, format = Byte))]
    pub signature: Option<Vec<u8>>,
}

/// A streamed MCP server tool call.
#[skip_serializing_none]
#[serde_as]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct McpServerToolCallDelta {
    /// The arguments to pass to the MCP server tool.
    #[cfg_attr(test, schema(schema_with = crate::gemini_interactions::spec::schema::object_schema))]
    pub arguments: serde_json::Value,

    /// The name of the tool which was called.
    pub name: String,

    /// The name of the used MCP server.
    pub server_name: String,
}

/// A streamed function result.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct FunctionResultDelta {
    /// Whether the tool call resulted in an error.
    #[cfg_attr(test, schema(nullable = false))]
    pub is_error: Option<bool>,

    /// The name of the tool that was called.
    #[cfg_attr(test, schema(nullable = false))]
    pub name: Option<String>,

    /// The result of the tool call.
    pub result: FunctionResultValue,
}

/// A streamed code execution result.
#[skip_serializing_none]
#[serde_as]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct CodeExecutionResultDelta {
    /// Whether the code execution resulted in an error.
    #[cfg_attr(test, schema(nullable = false))]
    pub is_error: Option<bool>,

    /// The output of the code execution.
    pub result: String,

    /// A signature hash for backend validation.
    #[serde_as(as = "Option<Base64>")]
    #[cfg_attr(test, schema(value_type = String, format = Byte))]
    pub signature: Option<Vec<u8>>,
}

/// A streamed URL context result.
#[skip_serializing_none]
#[serde_as]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct UrlContextResultDelta {
    /// Whether the URL context resulted in an error.
    #[cfg_attr(test, schema(nullable = false))]
    pub is_error: Option<bool>,

    /// The results of the URL context.
    pub result: Vec<UrlContextResult>,

    /// A signature hash for backend validation.
    #[serde_as(as = "Option<Base64>")]
    #[cfg_attr(test, schema(value_type = String, format = Byte))]
    pub signature: Option<Vec<u8>>,
}

/// A streamed Google Search result.
#[skip_serializing_none]
#[serde_as]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct GoogleSearchResultDelta {
    /// Whether the Google Search resulted in an error.
    #[cfg_attr(test, schema(nullable = false))]
    pub is_error: Option<bool>,

    /// The results of the Google Search.
    pub result: Vec<GoogleSearchResult>,

    /// A signature hash for backend validation.
    #[serde_as(as = "Option<Base64>")]
    #[cfg_attr(test, schema(value_type = String, format = Byte))]
    pub signature: Option<Vec<u8>>,
}

/// A streamed file search result.
#[skip_serializing_none]
#[serde_as]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct FileSearchResultDelta {
    /// The results of the file search.
    #[cfg_attr(test, schema(value_type = Vec<Object>))]
    pub result: Vec<super::FileSearchResult>,

    /// A signature hash for backend validation.
    #[serde_as(as = "Option<Base64>")]
    #[cfg_attr(test, schema(value_type = String, format = Byte))]
    pub signature: Option<Vec<u8>>,
}

/// A streamed Google Maps result.
#[skip_serializing_none]
#[serde_as]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct GoogleMapsResultDelta {
    /// The results of the Google Maps call.
    #[cfg_attr(test, schema(nullable = false))]
    pub result: Option<Vec<GoogleMapsResult>>,

    /// A signature hash for backend validation.
    #[serde_as(as = "Option<Base64>")]
    #[cfg_attr(test, schema(value_type = String, format = Byte))]
    pub signature: Option<Vec<u8>>,
}

/// A streamed MCP server tool result.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct McpServerToolResultDelta {
    /// Name of the tool which is called for this specific tool call.
    #[cfg_attr(test, schema(nullable = false))]
    pub name: Option<String>,

    /// The output from the MCP server call.
    pub result: FunctionResultValue,

    /// The name of the used MCP server.
    #[cfg_attr(test, schema(nullable = false))]
    pub server_name: Option<String>,
}

/// A streamed server-initiated media processing step.
#[skip_serializing_none]
#[serde_as]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ProcessingCallDelta {
    /// A signature hash for backend validation.
    #[serde_as(as = "Option<Base64>")]
    #[cfg_attr(test, schema(value_type = String, format = Byte))]
    pub signature: Option<Vec<u8>>,
}

/// The streamed result of a server-initiated media processing step.
#[skip_serializing_none]
#[serde_as]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ProcessingResultDelta {
    /// A signature hash for backend validation.
    #[serde_as(as = "Option<Base64>")]
    #[cfg_attr(test, schema(value_type = String, format = Byte))]
    pub signature: Option<Vec<u8>>,
}

/// A streamed text annotation update.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct TextAnnotationDelta {
    /// Citation information for model-generated content.
    #[cfg_attr(test, schema(nullable = false))]
    pub annotations: Option<Vec<Annotation>>,
}

/// An incremental tool-call arguments update.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ArgumentsDelta {
    /// The incremental arguments string.
    #[cfg_attr(test, schema(nullable = false))]
    pub arguments: Option<String>,
}

/// A `content.delta` streaming event.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ContentDelta {
    /// The delta content data.
    pub delta: ContentDeltaData,

    /// The event ID token used to resume the interaction stream from this event.
    #[cfg_attr(test, schema(nullable = false))]
    pub event_id: Option<String>,

    /// The index of the content block.
    pub index: i32,
}

/// A `content.start` streaming event.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ContentStart {
    /// The content block that is starting.
    pub content: Content,

    /// The event ID token used to resume the interaction stream from this event.
    #[cfg_attr(test, schema(nullable = false))]
    pub event_id: Option<String>,

    /// The index of the content block.
    pub index: i32,
}

/// A `content.stop` streaming event.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ContentStop {
    /// The event ID token used to resume the interaction stream from this event.
    #[cfg_attr(test, schema(nullable = false))]
    pub event_id: Option<String>,

    /// The index of the content block.
    pub index: i32,
}
