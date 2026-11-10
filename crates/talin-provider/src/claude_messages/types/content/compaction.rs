use serde::{Deserialize, Serialize};

use super::tool_changes::ResponseToolChange;
use crate::claude_messages::types::CacheControlEphemeral;

/// A [compaction block](https://platform.claude.com/docs/en/api/http/beta/messages#beta_compaction_block) returned by the API.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct CompactionBlock {
    /// The plaintext summary, or `null` when no plaintext summary is available.
    pub content: Option<String>,
    /// Opaque encrypted summary returned by the provider.
    pub encrypted_content: Option<String>,
    /// Provider signature to preserve when sending the summary back.
    pub signature: Option<String>,
    /// The [API reference](https://platform.claude.com/docs/en/api/http/beta/messages#beta_compaction_block) distinguishes `[]` from absence.
    /// Preserve `null` too if returned.
    #[serde(default, with = "serde_with::rust::double_option")]
    pub tool_changes: Option<Option<Vec<ResponseToolChange>>>,
}

/// A [compaction block parameter](https://platform.claude.com/docs/en/api/http/beta/messages#beta_compaction_block_param) sent in a later request.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Debug, Clone)]
#[non_exhaustive]
pub struct CompactionBlockParam {
    /// Keep `null` for failed compactions when round-tripping the block.
    pub content: Option<String>,
    /// Opaque encrypted summary returned by the provider.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encrypted_content: Option<String>,
    /// Provider signature to preserve when sending the summary back.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
    /// Tool changes returned with the summary, preserving omitted, null, and empty values.
    #[serde(
        skip_serializing_if = "Option::is_none",
        with = "serde_with::rust::double_option"
    )]
    pub tool_changes: Option<Option<Vec<ResponseToolChange>>>,
    /// Optional cache breakpoint after this compaction block.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControlEphemeral>,
}

impl From<CompactionBlock> for CompactionBlockParam {
    fn from(block: CompactionBlock) -> Self {
        Self {
            content: block.content,
            encrypted_content: block.encrypted_content,
            signature: block.signature,
            tool_changes: block.tool_changes,
            cache_control: None,
        }
    }
}

impl CompactionBlockParam {
    /// Creates a new [`CompactionBlockParam`] from the given summary.
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: Some(content.into()),
            encrypted_content: None,
            signature: None,
            tool_changes: None,
            cache_control: None,
        }
    }

    /// Creates a new [`CompactionBlockParam`] from opaque encrypted content.
    pub fn new_encrypted(encrypted_content: impl Into<String>) -> Self {
        Self {
            content: None,
            encrypted_content: Some(encrypted_content.into()),
            signature: None,
            tool_changes: None,
            cache_control: None,
        }
    }

    /// Creates a block representing a failed compaction with `null` content.
    pub fn new_failed() -> Self {
        Self {
            content: None,
            encrypted_content: None,
            signature: None,
            tool_changes: None,
            cache_control: None,
        }
    }

    /// Sets the [`signature`](Self::signature) returned by the API.
    pub fn with_signature(mut self, signature: impl Into<String>) -> Self {
        self.signature = Some(signature.into());
        self
    }

    /// Sets the [`tool_changes`](Self::tool_changes) returned by the API.
    pub fn with_tool_changes(mut self, tool_changes: Vec<ResponseToolChange>) -> Self {
        self.tool_changes = Some(Some(tool_changes));
        self
    }

    /// Adds a cache breakpoint after the returned summary.
    pub fn with_cache_control(mut self, cache_control: CacheControlEphemeral) -> Self {
        self.cache_control = Some(cache_control);
        self
    }
}
