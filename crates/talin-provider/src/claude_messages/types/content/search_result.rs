use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::claude_messages::types::{CacheControlEphemeral, CitationsConfigParam, TextBlockParam};
use crate::utils::tagged_enum;

/// A search result item. Used to provide results from custom search tools.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct SearchResultBlockParam {
    /// The title of the search result.
    pub title: String,

    /// The contents of the search result.
    pub content: Vec<SearchResultContentBlockParam>,

    /// Source URL or identifier.
    pub source: String,

    /// The citation configuration for the search result.
    #[cfg_attr(test, schema(nullable = false))]
    pub citations: Option<CitationsConfigParam>,

    /// Create a cache control breakpoint at this content block.
    pub cache_control: Option<CacheControlEphemeral>,
}

tagged_enum! {
    /// A content block within a search result item. Can only be a text item.
    #[derive(Serialize, Deserialize, Debug, Clone)]
    #[serde(tag = "type", rename_all = "snake_case")]
    pub enum SearchResultContentBlockParam {
        /// A text item.
        Text(TextBlockParam),
    }
}

impl SearchResultBlockParam {
    /// Creates a new [`SearchResultBlockParam`] from the given title, content, and source.
    pub fn new(
        title: impl Into<String>,
        content: Vec<SearchResultContentBlockParam>,
        source: impl Into<String>,
    ) -> Self {
        Self {
            title: title.into(),
            content,
            source: source.into(),
            citations: None,
            cache_control: None,
        }
    }

    /// Sets the [`citations`](Self::citations) configuration for the search result.
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
