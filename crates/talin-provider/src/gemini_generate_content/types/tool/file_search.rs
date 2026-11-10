use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// The FileSearch tool that retrieves knowledge from Semantic Retrieval corpora. Files are imported
/// to Semantic Retrieval corpora using the ImportFile API.
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Default, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct FileSearch {
    /// Required. The names of the fileSearchStores to retrieve from. Example:
    /// fileSearchStores/my-file-search-store-123
    pub file_search_store_names: Vec<String>,
    /// Optional. Metadata filter to apply to the semantic retrieval documents and chunks.
    pub metadata_filter: Option<String>,
    /// Optional. The number of semantic retrieval chunks to retrieve.
    pub top_k: Option<u32>,
}

impl FileSearch {
    /// Creates a new empty FileSearch.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the file search store names for this tool.
    pub fn with_file_search_store_names(mut self, file_search_store_names: Vec<String>) -> Self {
        self.file_search_store_names = file_search_store_names;
        self
    }

    /// Sets the metadata filter for this tool.
    pub fn with_metadata_filter(mut self, metadata_filter: impl Into<String>) -> Self {
        self.metadata_filter = Some(metadata_filter.into());
        self
    }

    /// Sets the number of semantic retrieval chunks to retrieve.
    pub fn with_top_k(mut self, top_k: u32) -> Self {
        self.top_k = Some(top_k);
        self
    }
}
