use serde::{Deserialize, Serialize};
use serde_with::base64::Base64;
use serde_with::{serde_as, skip_serializing_none};

/// The result of a function call.
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct FunctionResponse {
    /// The id of the function that was called.
    pub id: Option<String>,

    /// The name of the function that was called.
    pub name: String,

    /// The function response as a JSON object.
    pub response: serde_json::Value,

    /// Optional. Ordered [parts](FunctionResponsePart) that constitute a function response. Parts
    /// may have different IANA MIME types.
    pub parts: Option<Vec<FunctionResponsePart>>,

    /// Optional. Signals that function call continues, and more responses will be returned,
    /// turning the function call into a generator.
    pub will_continue: Option<bool>,

    /// Optional. Specifies how the response should be scheduled in the conversation.
    pub scheduling: Option<Scheduling>,
}

impl FunctionResponse {
    /// Creates a new [`FunctionResponse`].
    pub fn new(name: impl Into<String>, response: serde_json::Value) -> Self {
        Self {
            id: None,
            name: name.into(),
            response,
            parts: None,
            will_continue: None,
            scheduling: None,
        }
    }

    /// Sets the `id` field.
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Sets the `parts` field.
    pub fn with_parts(mut self, parts: Vec<FunctionResponsePart>) -> Self {
        self.parts = Some(parts);
        self
    }

    /// Sets the `will_continue` field.
    pub fn with_will_continue(mut self, will_continue: bool) -> Self {
        self.will_continue = Some(will_continue);
        self
    }

    /// Sets the `scheduling` field.
    pub fn with_scheduling(mut self, scheduling: Scheduling) -> Self {
        self.scheduling = Some(scheduling);
        self
    }
}

/// A datatype containing media that is part of a FunctionResponse message.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub enum FunctionResponsePart {
    /// Inline media bytes.
    InlineData(FunctionResponseBlob),
}

/// The result of a function call part containing raw media bytes.
#[skip_serializing_none]
#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct FunctionResponseBlob {
    /// The IANA standard MIME type of the source data. Examples: - image/png - image/jpeg If an
    /// unsupported MIME type is provided, an error will be returned. For a complete list of
    /// supported types, see Supported file formats.
    pub mime_type: String,
    /// Raw bytes for media formats. A base64-encoded string.
    #[serde_as(as = "Base64")]
    pub data: Vec<u8>,
}

impl FunctionResponseBlob {
    /// Creates a new [`FunctionResponseBlob`].
    pub fn new(mime_type: impl Into<String>, data: Vec<u8>) -> Self {
        Self {
            mime_type: mime_type.into(),
            data,
        }
    }
}

/// Specifies how the response should be scheduled in the conversation.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum Scheduling {
    /// This value is unused.
    SchedulingUnspecified,
    /// Only add the result to the conversation context, do not interrupt or trigger generation.
    Silent,
    /// Add the result to the conversation context, and prompt to generate output without
    /// interrupting ongoing generation.
    WhenIdle,
    /// Add the result to the conversation context, interrupt ongoing generation and prompt to
    /// generate output.
    Interrupt,
}
