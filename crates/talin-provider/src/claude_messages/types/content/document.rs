use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::claude_messages::types::{
    CacheControlEphemeral, CitationsConfig, CitationsConfigParam, ImageBlockParam, TextBlockParam,
    TextOrItems,
};
use crate::utils::{string_enum, tagged_enum};

/// A document block within a response message.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct DocumentBlock {
    /// The document title.
    #[cfg_attr(test, schema(required = true))]
    pub title: Option<String>,

    /// The document source.
    pub source: DocumentBlockSource,

    /// The citations configuration for the document.
    #[cfg_attr(test, schema(required = true))]
    pub citations: Option<CitationsConfig>,
}

/// A document block within a request message.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct DocumentBlockParam {
    /// The document title.
    pub title: Option<String>,

    /// The document source.
    pub source: DocumentBlockParamSource,

    /// Context about the document that will not be cited from, used to provide additional
    /// information about the document.
    pub context: Option<String>,

    /// The citations configuration for the document.
    pub citations: Option<CitationsConfigParam>,

    /// Create a cache control breakpoint at this content block.
    pub cache_control: Option<CacheControlEphemeral>,
}

tagged_enum! {
    /// The document source, as returned in a response.
    #[derive(Serialize, Deserialize, Debug, Clone)]
    #[serde(tag = "type", rename_all = "snake_case")]
    pub enum DocumentBlockSource {
        /// A base64-encoded PDF document.
        Base64(Base64PDFSource),
        /// A plain text document.
        Text(PlainTextSource),
    }
}

tagged_enum! {
    /// The document source, as passed in a request.
    #[derive(Serialize, Deserialize, Debug, Clone)]
    #[serde(tag = "type", rename_all = "snake_case")]
    pub enum DocumentBlockParamSource {
        /// A base64-encoded PDF document.
        Base64(Base64PDFSource),
        /// A plain text document.
        Text(PlainTextSource),
        /// A document represented as a list of content blocks.
        #[serde(rename = "content")]
        Content(ContentBlockSource),
        /// A URL pointing to a PDF document.
        Url(URLPDFSource),
        /// A document file id.
        File(FileDocumentSource),
    }
}

/// A base64-encoded PDF document.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct Base64PDFSource {
    /// The base64-encoded document data.
    pub data: String,

    /// The media type of the document. e.g. "application/pdf".
    pub media_type: Base64PDFType,
}

string_enum! {
    /// A media type of a base64-encoded PDF document.
    pub enum Base64PDFType {
        /// `"application/pdf"`
        #[serde(rename = "application/pdf")]
        Pdf,
    }
}

/// A plain text document.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct PlainTextSource {
    /// The document content.
    pub data: String,

    /// The media type of the document. e.g. `text/plain`.
    pub media_type: PlainTextType,
}

string_enum! {
    /// A media type of a plain text document.
    pub enum PlainTextType {
        /// `"text/plain"`
        #[serde(rename = "text/plain")]
        Plain,
    }
}

/// A document represented as a list of content blocks.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ContentBlockSource {
    /// The content of the document.
    #[cfg_attr(
        test,
        schema(schema_with = crate::claude_messages::spec::schema::text_or_content_block_source_contents_schema)
    )]
    pub content: TextOrItems<ContentBlockSourceContent>,
}

tagged_enum! {
    /// A content block within a document source. Can be either a text block or an image block.
    #[derive(Serialize, Deserialize, Debug, Clone)]
    #[serde(tag = "type", rename_all = "snake_case")]
    pub enum ContentBlockSourceContent {
        /// A text block.
        Text(TextBlockParam),
        /// An image block.
        Image(ImageBlockParam),
    }
}

/// A URL pointing to a PDF document.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct URLPDFSource {
    /// The URL of the document.
    pub url: String,
}

/// A document file ID.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct FileDocumentSource {
    /// The file ID of the document.
    pub file_id: String,
}

impl DocumentBlock {
    /// Creates a new [`DocumentBlock`] from the given source.
    pub fn new(source: DocumentBlockSource) -> Self {
        Self {
            title: None,
            source,
            citations: None,
        }
    }

    /// Sets the [`title`](Self::title) of the document.
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets the [`citations`](Self::citations) configuration for the document.
    pub fn with_citations(mut self, citations: CitationsConfig) -> Self {
        self.citations = Some(citations);
        self
    }
}

impl DocumentBlockParam {
    /// Creates a new [`DocumentBlockParam`] from the given source.
    pub fn new(source: DocumentBlockParamSource) -> Self {
        Self {
            title: None,
            source,
            context: None,
            citations: None,
            cache_control: None,
        }
    }

    /// Sets the [`title`](Self::title) of the document.
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets the [`context`](Self::context) about the document.
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }

    /// Sets the [`citations`](Self::citations) configuration for the document.
    pub fn with_citations(mut self, citations: CitationsConfigParam) -> Self {
        self.citations = Some(citations);
        self
    }

    /// Sets the [`cache_control`](Self::cache_control) breakpoint for this content block.
    pub fn with_cache_control(mut self, cache_control: CacheControlEphemeral) -> Self {
        self.cache_control = Some(cache_control);
        self
    }
}

impl Base64PDFSource {
    /// Creates a new [`Base64PDFSource`] from the given data and media type.
    pub fn new(data: impl Into<String>, media_type: Base64PDFType) -> Self {
        Self {
            data: data.into(),
            media_type,
        }
    }
}

impl PlainTextSource {
    /// Creates a new [`PlainTextSource`] from the given data and media type.
    pub fn new(data: impl Into<String>, media_type: PlainTextType) -> Self {
        Self {
            data: data.into(),
            media_type,
        }
    }
}

impl ContentBlockSource {
    /// Creates a new [`ContentBlockSource`] from the given content.
    pub fn new(content: TextOrItems<ContentBlockSourceContent>) -> Self {
        Self { content }
    }
}

impl URLPDFSource {
    /// Creates a new [`URLPDFSource`] from the given URL.
    pub fn new(url: impl Into<String>) -> Self {
        Self { url: url.into() }
    }
}

impl FileDocumentSource {
    /// Creates a new [`FileDocumentSource`] from the given file ID.
    pub fn new(file_id: impl Into<String>) -> Self {
        Self {
            file_id: file_id.into(),
        }
    }
}

impl From<DocumentBlock> for DocumentBlockParam {
    fn from(value: DocumentBlock) -> Self {
        DocumentBlockParam {
            title: value.title,
            source: value.source.into(),
            context: None,
            citations: value.citations.map(CitationsConfigParam::from),
            cache_control: None,
        }
    }
}

impl From<DocumentBlockSource> for DocumentBlockParamSource {
    fn from(value: DocumentBlockSource) -> Self {
        match value {
            DocumentBlockSource::Base64(base64) => DocumentBlockParamSource::Base64(base64),
            DocumentBlockSource::Text(text) => DocumentBlockParamSource::Text(text),
            DocumentBlockSource::Unknown(unknown) => DocumentBlockParamSource::Unknown(unknown),
        }
    }
}
