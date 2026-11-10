use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;

/// A multimodal data input, which can be a single item, a flat list (one item per request in a
/// batch), or a nested list (multiple items per request).
///
/// Mirrors [`MultimodalDataInputFormat`] in the python source.
///
/// [`MultimodalDataInputFormat`]: https://github.com/sgl-project/sglang/blob/430418e2182606b5296b26c912df8f12ec9b380f/python/sglang/srt/managers/io_struct.py#L143-L148
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MultimodalData<T> {
    /// A single item for a single request.
    Single(T),
    /// One item per request in a batch.
    Batch(Vec<T>),
    /// Multiple items per request in a batch.
    NestedBatch(Vec<Vec<T>>),
}

/// Detail level for an image, as used by the OpenAI-style image schema.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImageDetail {
    /// Automatically choose the detail level.
    Auto,
    /// Low detail level.
    Low,
    /// High detail level.
    High,
}

/// A structured image reference.
///
/// Mirrors [`ImageData`] in the python source.
///
/// [`ImageData`]: https://github.com/sgl-project/sglang/blob/430418e2182606b5296b26c912df8f12ec9b380f/python/sglang/srt/utils/common.py#L922-L927
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageData {
    /// URL, file path, or base64 data URI for the image.
    pub url: String,
    /// Requested detail level (defaults to `auto` server-side).
    pub detail: Option<ImageDetail>,
    /// Maximum number of dynamic patches (tiles) to split the image into.
    pub max_dynamic_patch: Option<i64>,
    /// Free-form processor keyword arguments.
    pub preprocess_kwargs: Option<HashMap<String, Value>>,
}

/// A single image input: a bare URL/path/base64 string, a structured [`ImageData`], or a free-form
/// object.
///
/// Mirrors [`ImageDataInputItem`] in the python source.
///
/// [`ImageDataInputItem`]: https://github.com/sgl-project/sglang/blob/430418e2182606b5296b26c912df8f12ec9b380f/python/sglang/srt/managers/io_struct.py#L136
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ImageDataItem {
    /// A bare URL, file path, or base64 data URI.
    Url(String),
    /// A structured [`ImageData`] reference.
    Data(ImageData),
    /// A free-form object.
    Dict(HashMap<String, Value>),
}

/// A structured video reference.
///
/// Mirrors [`VideoData`] in the python source.
///
/// [`VideoData`]: https://github.com/sgl-project/sglang/blob/430418e2182606b5296b26c912df8f12ec9b380f/python/sglang/srt/utils/common.py#L930-L933
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoData {
    /// URL, file path, or base64 data URI for the video.
    pub url: String,
    /// Free-form processor keyword arguments.
    pub preprocess_kwargs: Option<HashMap<String, Value>>,
}

/// A single video input: a bare URL/path/base64 string, a structured [`VideoData`], or a free-form
/// object.
///
/// Mirrors [`VideoDataInputItem`] in the python source.
///
/// [`VideoDataInputItem`]: https://github.com/sgl-project/sglang/blob/430418e2182606b5296b26c912df8f12ec9b380f/python/sglang/srt/managers/io_struct.py#L138
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum VideoDataItem {
    /// A bare URL, file path, or base64 data URI.
    Url(String),
    /// A structured [`VideoData`] reference.
    Data(VideoData),
    /// A free-form object.
    Dict(HashMap<String, Value>),
}

/// A single audio input: a bare URL/path/base64 string or a free-form object.
///
/// Mirrors [`AudioDataInputItem`] in the python source.
///
/// [`AudioDataInputItem`]: https://github.com/sgl-project/sglang/blob/430418e2182606b5296b26c912df8f12ec9b380f/python/sglang/srt/managers/io_struct.py#L137
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AudioDataItem {
    /// A bare URL, file path, or base64 data URI.
    Url(String),
    /// A free-form object.
    Dict(HashMap<String, Value>),
}
