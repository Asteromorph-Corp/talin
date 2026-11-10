use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::claude_messages::types::{CacheControlEphemeral, TextCitation, TextCitationParam};

/// A text block within a response message.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct TextBlock {
    /// The text content.
    pub text: String,

    // Should be serialized as `null` if `None`.
    /// Citations supporting the text block.
    ///
    /// The type of citation returned depends on the type of document being cited: a PDF results
    /// in `page_location`, plain text in `char_location`, and a content document in
    /// `content_block_location`.
    #[cfg_attr(test, schema(required = true))]
    pub citations: Option<Vec<TextCitation>>,
}

/// A text block within a request message.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct TextBlockParam {
    /// The text content.
    pub text: String,

    /// Citations associated with the text content.
    pub citations: Option<Vec<TextCitationParam>>,

    /// Create a cache control breakpoint at this content block.
    pub cache_control: Option<CacheControlEphemeral>,
}

impl TextBlock {
    /// Creates a new [`TextBlock`] from the given text.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            citations: None,
        }
    }

    /// Sets the [`citations`](Self::citations) supporting the text block.
    pub fn with_citations(mut self, citations: Vec<TextCitation>) -> Self {
        self.citations = Some(citations);
        self
    }
}

impl TextBlockParam {
    /// Creates a new [`TextBlockParam`] from the given text.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            citations: None,
            cache_control: None,
        }
    }

    /// Sets the [`citations`](Self::citations) associated with the text content.
    pub fn with_citations(mut self, citations: Vec<TextCitationParam>) -> Self {
        self.citations = Some(citations);
        self
    }

    /// Sets the [`cache_control`](Self::cache_control) breakpoint for this content block.
    pub fn with_cache_control(mut self, cache_control: CacheControlEphemeral) -> Self {
        self.cache_control = Some(cache_control);
        self
    }
}

impl From<TextBlock> for TextBlockParam {
    fn from(value: TextBlock) -> Self {
        Self {
            text: value.text,
            citations: value
                .citations
                .map(|citations| citations.into_iter().map(Into::into).collect()),
            cache_control: None,
        }
    }
}
