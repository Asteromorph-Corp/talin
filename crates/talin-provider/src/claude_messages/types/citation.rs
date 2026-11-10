//! Citations within a text block, and the request-side twin of each.
//!
//! A citation states every field on both sides, `null` included, so the two differ only in
//! `file_id`, which only a response carries.

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::utils::tagged_enum;

tagged_enum! {
    /// A citation within a response message.
    #[derive(Serialize, Deserialize, Debug, Clone)]
    #[serde(tag = "type", rename_all = "snake_case")]
    pub enum TextCitation {
        /// Reference to a specific text position in a plain text document.
        CharLocation(CitationCharLocation),
        /// Reference to a specific page range in a PDF document.
        PageLocation(CitationPageLocation),
        /// Reference to a specific content block in a custom content document.
        ContentBlockLocation(CitationContentBlockLocation),
        /// Reference to a result from an internal web search tool.
        WebSearchResultLocation(CitationsWebSearchResultLocation),
        /// Reference to a result from a custom search tool.
        SearchResultLocation(CitationsSearchResultLocation),
    }
}

tagged_enum! {
    /// A citation within a request message.
    #[derive(Serialize, Deserialize, Debug, Clone)]
    #[serde(tag = "type", rename_all = "snake_case")]
    pub enum TextCitationParam {
        /// Reference to a specific text position in a plain text document.
        CharLocation(CitationCharLocationParam),
        /// Reference to a specific page range in a PDF document.
        PageLocation(CitationPageLocationParam),
        /// Reference to a specific content block in a custom content document.
        ContentBlockLocation(CitationContentBlockLocationParam),
        /// Reference to a result from an internal web search tool.
        WebSearchResultLocation(CitationWebSearchResultLocationParam),
        /// Reference to a result from a custom search tool.
        SearchResultLocation(CitationSearchResultLocationParam),
    }
}

/// Reference to a specific text position in a plain text document, as returned in a response.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct CitationCharLocation {
    /// The index of the cited document in a list of documents.
    #[cfg_attr(test, schema(schema_with = crate::claude_messages::spec::schema::number_schema))]
    pub document_index: usize,

    // Should be serialized as `null` if `None`.
    /// The title of the cited document.
    #[cfg_attr(test, schema(required = true))]
    pub document_title: Option<String>,

    /// The exact text that is being cited.
    pub cited_text: String,

    /// The starting character index of the cited text.
    #[cfg_attr(test, schema(schema_with = crate::claude_messages::spec::schema::number_schema))]
    pub start_char_index: usize,

    /// The ending character index of the cited text.
    #[cfg_attr(test, schema(schema_with = crate::claude_messages::spec::schema::number_schema))]
    pub end_char_index: isize,

    // Should be serialized as `null` if `None`.
    /// The id of the file the cited document originates from.
    #[cfg_attr(test, schema(required = true))]
    pub file_id: Option<String>,
}

/// Reference to a specific text position in a plain text document, as passed back in a request.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct CitationCharLocationParam {
    /// The index of the cited document in a list of documents.
    #[cfg_attr(test, schema(schema_with = crate::claude_messages::spec::schema::number_schema))]
    pub document_index: usize,

    // Should be serialized as `null` if `None`.
    /// The title of the cited document.
    #[cfg_attr(test, schema(required = true))]
    pub document_title: Option<String>,

    /// The exact text that is being cited.
    pub cited_text: String,

    /// The starting character index of the cited text.
    #[cfg_attr(test, schema(schema_with = crate::claude_messages::spec::schema::number_schema))]
    pub start_char_index: usize,

    /// The ending character index of the cited text.
    #[cfg_attr(test, schema(schema_with = crate::claude_messages::spec::schema::number_schema))]
    pub end_char_index: isize,
}

/// Reference to a specific page range in a PDF document, as returned in a response.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct CitationPageLocation {
    /// The index of the cited document in a list of documents.
    #[cfg_attr(test, schema(schema_with = crate::claude_messages::spec::schema::number_schema))]
    pub document_index: usize,

    // Should be serialized as `null` if `None`.
    /// The title of the cited document.
    #[cfg_attr(test, schema(required = true))]
    pub document_title: Option<String>,

    /// The exact text that is being cited.
    pub cited_text: String,

    /// The starting page number of the cited text.
    #[cfg_attr(test, schema(schema_with = crate::claude_messages::spec::schema::number_schema))]
    pub start_page_number: usize,

    /// The ending page number of the cited text.
    #[cfg_attr(test, schema(schema_with = crate::claude_messages::spec::schema::number_schema))]
    pub end_page_number: isize,

    // Should be serialized as `null` if `None`.
    /// The id of the file the cited document originates from.
    #[cfg_attr(test, schema(required = true))]
    pub file_id: Option<String>,
}

/// Reference to a specific page range in a PDF document, as passed back in a request.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct CitationPageLocationParam {
    /// The index of the cited document in a list of documents.
    #[cfg_attr(test, schema(schema_with = crate::claude_messages::spec::schema::number_schema))]
    pub document_index: usize,

    // Should be serialized as `null` if `None`.
    /// The title of the cited document.
    #[cfg_attr(test, schema(required = true))]
    pub document_title: Option<String>,

    /// The exact text that is being cited.
    pub cited_text: String,

    /// The starting page number of the cited text.
    #[cfg_attr(test, schema(schema_with = crate::claude_messages::spec::schema::number_schema))]
    pub start_page_number: usize,

    /// The ending page number of the cited text.
    #[cfg_attr(test, schema(schema_with = crate::claude_messages::spec::schema::number_schema))]
    pub end_page_number: isize,
}

/// Reference to a specific content block in a custom content document, as returned in a response.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct CitationContentBlockLocation {
    /// The index of the cited document in a list of documents.
    #[cfg_attr(test, schema(schema_with = crate::claude_messages::spec::schema::number_schema))]
    pub document_index: usize,

    // Should be serialized as `null` if `None`.
    /// The title of the cited document.
    #[cfg_attr(test, schema(required = true))]
    pub document_title: Option<String>,

    /// The exact text that is being cited.
    pub cited_text: String,

    /// The starting content block index of the cited text.
    #[cfg_attr(test, schema(schema_with = crate::claude_messages::spec::schema::number_schema))]
    pub start_block_index: usize,

    /// The ending content block index of the cited text.
    #[cfg_attr(test, schema(schema_with = crate::claude_messages::spec::schema::number_schema))]
    pub end_block_index: isize,

    // Should be serialized as `null` if `None`.
    /// The id of the file the cited document originates from.
    #[cfg_attr(test, schema(required = true))]
    pub file_id: Option<String>,
}

/// Reference to a specific content block in a custom content document, as passed back in a
/// request.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct CitationContentBlockLocationParam {
    /// The index of the cited document in a list of documents.
    #[cfg_attr(test, schema(schema_with = crate::claude_messages::spec::schema::number_schema))]
    pub document_index: usize,

    // Should be serialized as `null` if `None`.
    /// The title of the cited document.
    #[cfg_attr(test, schema(required = true))]
    pub document_title: Option<String>,

    /// The exact text that is being cited.
    pub cited_text: String,

    /// The starting content block index of the cited text.
    #[cfg_attr(test, schema(schema_with = crate::claude_messages::spec::schema::number_schema))]
    pub start_block_index: usize,

    /// The ending content block index of the cited text.
    #[cfg_attr(test, schema(schema_with = crate::claude_messages::spec::schema::number_schema))]
    pub end_block_index: isize,
}

/// Reference to a result from the internal web search tool, as returned in a response.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct CitationsWebSearchResultLocation {
    /// The encrypted index of the cited search result.
    pub encrypted_index: String,

    // Should be serialized as `null` if `None`.
    /// The title of the cited search result.
    #[cfg_attr(test, schema(required = true))]
    pub title: Option<String>,

    /// The exact text that is being cited.
    pub cited_text: String,

    /// The URL of the cited search result.
    pub url: String,
}

/// Reference to a result from the internal web search tool, as passed back in a request.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct CitationWebSearchResultLocationParam {
    /// The encrypted index of the cited search result.
    pub encrypted_index: String,

    // Should be serialized as `null` if `None`.
    /// The title of the cited search result.
    #[cfg_attr(test, schema(required = true))]
    pub title: Option<String>,

    /// The exact text that is being cited.
    pub cited_text: String,

    /// The URL of the cited search result.
    pub url: String,
}

/// Reference to a result from a custom search tool, as returned in a response.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct CitationsSearchResultLocation {
    /// The index of the cited search result.
    #[cfg_attr(test, schema(schema_with = crate::claude_messages::spec::schema::number_schema))]
    pub search_result_index: usize,

    // Should be serialized as `null` if `None`.
    /// The title of the cited search result.
    #[cfg_attr(test, schema(required = true))]
    pub title: Option<String>,

    /// The exact text that is being cited.
    pub cited_text: String,

    /// The source of the cited search result.
    pub source: String,

    /// The starting content block index of the cited text.
    #[cfg_attr(test, schema(schema_with = crate::claude_messages::spec::schema::number_schema))]
    pub start_block_index: usize,

    /// The ending content block index of the cited text.
    #[cfg_attr(test, schema(schema_with = crate::claude_messages::spec::schema::number_schema))]
    pub end_block_index: isize,
}

/// Reference to a result from a custom search tool, as passed back in a request.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct CitationSearchResultLocationParam {
    /// The index of the cited search result.
    #[cfg_attr(test, schema(schema_with = crate::claude_messages::spec::schema::number_schema))]
    pub search_result_index: usize,

    // Should be serialized as `null` if `None`.
    /// The title of the cited search result.
    #[cfg_attr(test, schema(required = true))]
    pub title: Option<String>,

    /// The exact text that is being cited.
    pub cited_text: String,

    /// The source of the cited search result.
    pub source: String,

    /// The starting content block index of the cited text.
    #[cfg_attr(test, schema(schema_with = crate::claude_messages::spec::schema::number_schema))]
    pub start_block_index: usize,

    /// The ending content block index of the cited text.
    #[cfg_attr(test, schema(schema_with = crate::claude_messages::spec::schema::number_schema))]
    pub end_block_index: isize,
}

/// The citation configuration of a document or search result, as returned in a response.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct CitationsConfig {
    /// Whether citations are enabled.
    pub enabled: bool,
}

/// The citation configuration of a document or search result, as passed in a request.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[non_exhaustive]
pub struct CitationsConfigParam {
    /// Whether to enable citations.
    #[cfg_attr(test, schema(nullable = false))]
    pub enabled: Option<bool>,
}

impl CitationCharLocation {
    /// Creates a new [`CitationCharLocation`].
    pub fn new(
        document_index: usize,
        cited_text: impl Into<String>,
        start_char_index: usize,
        end_char_index: isize,
    ) -> Self {
        Self {
            document_index,
            document_title: None,
            cited_text: cited_text.into(),
            start_char_index,
            end_char_index,
            file_id: None,
        }
    }

    /// Sets the [`document_title`](Self::document_title) of the citation.
    pub fn with_document_title(mut self, document_title: impl Into<String>) -> Self {
        self.document_title = Some(document_title.into());
        self
    }

    /// Sets the [`file_id`](Self::file_id) the cited document originates from.
    pub fn with_file_id(mut self, file_id: impl Into<String>) -> Self {
        self.file_id = Some(file_id.into());
        self
    }
}

impl CitationCharLocationParam {
    /// Creates a new [`CitationCharLocationParam`].
    pub fn new(
        document_index: usize,
        cited_text: impl Into<String>,
        start_char_index: usize,
        end_char_index: isize,
    ) -> Self {
        Self {
            document_index,
            document_title: None,
            cited_text: cited_text.into(),
            start_char_index,
            end_char_index,
        }
    }

    /// Sets the [`document_title`](Self::document_title) of the citation.
    pub fn with_document_title(mut self, document_title: impl Into<String>) -> Self {
        self.document_title = Some(document_title.into());
        self
    }
}

impl CitationPageLocation {
    /// Creates a new [`CitationPageLocation`].
    pub fn new(
        document_index: usize,
        cited_text: impl Into<String>,
        start_page_number: usize,
        end_page_number: isize,
    ) -> Self {
        Self {
            document_index,
            document_title: None,
            cited_text: cited_text.into(),
            start_page_number,
            end_page_number,
            file_id: None,
        }
    }

    /// Sets the [`document_title`](Self::document_title) of the citation.
    pub fn with_document_title(mut self, document_title: impl Into<String>) -> Self {
        self.document_title = Some(document_title.into());
        self
    }

    /// Sets the [`file_id`](Self::file_id) the cited document originates from.
    pub fn with_file_id(mut self, file_id: impl Into<String>) -> Self {
        self.file_id = Some(file_id.into());
        self
    }
}

impl CitationPageLocationParam {
    /// Creates a new [`CitationPageLocationParam`].
    pub fn new(
        document_index: usize,
        cited_text: impl Into<String>,
        start_page_number: usize,
        end_page_number: isize,
    ) -> Self {
        Self {
            document_index,
            document_title: None,
            cited_text: cited_text.into(),
            start_page_number,
            end_page_number,
        }
    }

    /// Sets the [`document_title`](Self::document_title) of the citation.
    pub fn with_document_title(mut self, document_title: impl Into<String>) -> Self {
        self.document_title = Some(document_title.into());
        self
    }
}

impl CitationContentBlockLocation {
    /// Creates a new [`CitationContentBlockLocation`].
    pub fn new(
        document_index: usize,
        cited_text: impl Into<String>,
        start_block_index: usize,
        end_block_index: isize,
    ) -> Self {
        Self {
            document_index,
            document_title: None,
            cited_text: cited_text.into(),
            start_block_index,
            end_block_index,
            file_id: None,
        }
    }

    /// Sets the [`document_title`](Self::document_title) of the citation.
    pub fn with_document_title(mut self, document_title: impl Into<String>) -> Self {
        self.document_title = Some(document_title.into());
        self
    }

    /// Sets the [`file_id`](Self::file_id) the cited document originates from.
    pub fn with_file_id(mut self, file_id: impl Into<String>) -> Self {
        self.file_id = Some(file_id.into());
        self
    }
}

impl CitationContentBlockLocationParam {
    /// Creates a new [`CitationContentBlockLocationParam`].
    pub fn new(
        document_index: usize,
        cited_text: impl Into<String>,
        start_block_index: usize,
        end_block_index: isize,
    ) -> Self {
        Self {
            document_index,
            document_title: None,
            cited_text: cited_text.into(),
            start_block_index,
            end_block_index,
        }
    }

    /// Sets the [`document_title`](Self::document_title) of the citation.
    pub fn with_document_title(mut self, document_title: impl Into<String>) -> Self {
        self.document_title = Some(document_title.into());
        self
    }
}

impl CitationsWebSearchResultLocation {
    /// Creates a new [`CitationsWebSearchResultLocation`].
    pub fn new(
        encrypted_index: impl Into<String>,
        cited_text: impl Into<String>,
        url: impl Into<String>,
    ) -> Self {
        Self {
            encrypted_index: encrypted_index.into(),
            title: None,
            cited_text: cited_text.into(),
            url: url.into(),
        }
    }

    /// Sets the [`title`](Self::title) of the citation.
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }
}

impl CitationWebSearchResultLocationParam {
    /// Creates a new [`CitationWebSearchResultLocationParam`].
    pub fn new(
        encrypted_index: impl Into<String>,
        cited_text: impl Into<String>,
        url: impl Into<String>,
    ) -> Self {
        Self {
            encrypted_index: encrypted_index.into(),
            title: None,
            cited_text: cited_text.into(),
            url: url.into(),
        }
    }

    /// Sets the [`title`](Self::title) of the citation.
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }
}

impl CitationsSearchResultLocation {
    /// Creates a new [`CitationsSearchResultLocation`].
    pub fn new(
        search_result_index: usize,
        cited_text: impl Into<String>,
        source: impl Into<String>,
        start_block_index: usize,
        end_block_index: isize,
    ) -> Self {
        Self {
            search_result_index,
            title: None,
            cited_text: cited_text.into(),
            source: source.into(),
            start_block_index,
            end_block_index,
        }
    }

    /// Sets the [`title`](Self::title) of the citation.
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }
}

impl CitationSearchResultLocationParam {
    /// Creates a new [`CitationSearchResultLocationParam`].
    pub fn new(
        search_result_index: usize,
        cited_text: impl Into<String>,
        source: impl Into<String>,
        start_block_index: usize,
        end_block_index: isize,
    ) -> Self {
        Self {
            search_result_index,
            title: None,
            cited_text: cited_text.into(),
            source: source.into(),
            start_block_index,
            end_block_index,
        }
    }

    /// Sets the [`title`](Self::title) of the citation.
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }
}

impl CitationsConfig {
    /// Creates a new [`CitationsConfig`] with the given enabled state.
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }
}

impl CitationsConfigParam {
    /// Creates a new [`CitationsConfigParam`] with the given enabled state.
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled: Some(enabled),
        }
    }

    /// Sets whether citations are [`enabled`](Self::enabled).
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = Some(enabled);
        self
    }
}

impl From<TextCitation> for TextCitationParam {
    fn from(value: TextCitation) -> Self {
        match value {
            TextCitation::CharLocation(citation) => {
                TextCitationParam::CharLocation(citation.into())
            }
            TextCitation::PageLocation(citation) => {
                TextCitationParam::PageLocation(citation.into())
            }
            TextCitation::ContentBlockLocation(citation) => {
                TextCitationParam::ContentBlockLocation(citation.into())
            }
            TextCitation::WebSearchResultLocation(citation) => {
                TextCitationParam::WebSearchResultLocation(citation.into())
            }
            TextCitation::SearchResultLocation(citation) => {
                TextCitationParam::SearchResultLocation(citation.into())
            }
            TextCitation::Unknown(value) => TextCitationParam::Unknown(value),
        }
    }
}

/// Drops [`file_id`](CitationCharLocation::file_id), which a request does not carry.
impl From<CitationCharLocation> for CitationCharLocationParam {
    fn from(value: CitationCharLocation) -> Self {
        Self {
            document_index: value.document_index,
            document_title: value.document_title,
            cited_text: value.cited_text,
            start_char_index: value.start_char_index,
            end_char_index: value.end_char_index,
        }
    }
}

/// Drops [`file_id`](CitationPageLocation::file_id), which a request does not carry.
impl From<CitationPageLocation> for CitationPageLocationParam {
    fn from(value: CitationPageLocation) -> Self {
        Self {
            document_index: value.document_index,
            document_title: value.document_title,
            cited_text: value.cited_text,
            start_page_number: value.start_page_number,
            end_page_number: value.end_page_number,
        }
    }
}

/// Drops [`file_id`](CitationContentBlockLocation::file_id), which a request does not carry.
impl From<CitationContentBlockLocation> for CitationContentBlockLocationParam {
    fn from(value: CitationContentBlockLocation) -> Self {
        Self {
            document_index: value.document_index,
            document_title: value.document_title,
            cited_text: value.cited_text,
            start_block_index: value.start_block_index,
            end_block_index: value.end_block_index,
        }
    }
}

impl From<CitationsWebSearchResultLocation> for CitationWebSearchResultLocationParam {
    fn from(value: CitationsWebSearchResultLocation) -> Self {
        Self {
            encrypted_index: value.encrypted_index,
            title: value.title,
            cited_text: value.cited_text,
            url: value.url,
        }
    }
}

impl From<CitationsSearchResultLocation> for CitationSearchResultLocationParam {
    fn from(value: CitationsSearchResultLocation) -> Self {
        Self {
            search_result_index: value.search_result_index,
            title: value.title,
            cited_text: value.cited_text,
            source: value.source,
            start_block_index: value.start_block_index,
            end_block_index: value.end_block_index,
        }
    }
}

impl From<CitationsConfig> for CitationsConfigParam {
    fn from(value: CitationsConfig) -> Self {
        Self {
            enabled: Some(value.enabled),
        }
    }
}
