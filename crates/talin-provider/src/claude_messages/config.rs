use reqwest::header::HeaderMap;

use crate::claude_messages::types::{ApiErrorDetails, MessagesRequest};
use crate::error::Error;

mod anthropic;
#[cfg(feature = "vertex")]
mod vertex;

pub use anthropic::AnthropicConfig;
#[cfg(feature = "vertex")]
pub use vertex::VertexConfig;

const ANTHROPIC_BETA: &str = "anthropic-beta";
const ANTHROPIC_VERSION: &str = "anthropic-version";
const X_API_KEY: &str = "x-api-key";
#[cfg(feature = "vertex")]
const AUTHORIZATION: &str = "authorization";

/// Beta headers for the Claude API.
pub mod beta_headers {
    /// Beta flag for the `mcp-client-2025-11-20` feature.
    #[cfg(feature = "mcp-client-2025-11-20")]
    pub const MCP_CLIENT: &str = "mcp-client-2025-11-20";

    /// Beta flag for on-demand compaction.
    #[cfg(feature = "compact-2026-09-04")]
    pub const COMPACT_ON_DEMAND: &str = "compact-2026-09-04";

    /// Beta flag for automatic compaction.
    #[cfg(feature = "compact-2026-01-12")]
    pub const COMPACT_AUTO_SUMMARIZE: &str = "compact-2026-01-12";

    /// Beta flag for tool changes by reference.
    #[cfg(feature = "mid-conversation-tool-changes-2026-07-01")]
    pub const MID_CONVERSATION_TOOL_CHANGES: &str = "mid-conversation-tool-changes-2026-07-01";

    /// Beta flag for inline tool changes.
    #[cfg(feature = "inline-tools-2026-09-15")]
    pub const INLINE_TOOLS: &str = "inline-tools-2026-09-15";
}

/// Configuration for the Claude Messages API client.
///
/// Different variants represent different backends that the client can communicate with.
#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum ClientConfig {
    /// Anthropic's first-party API (`https://api.anthropic.com`).
    Anthropic(AnthropicConfig),
    /// Google Cloud's Vertex AI Model Garden endpoint. Requires the `vertex` feature.
    #[cfg(feature = "vertex")]
    Vertex(VertexConfig),
}

impl ClientConfig {
    /// Checks whether the specific beta flag is enabled.
    pub fn is_beta_enabled(&self, flag: &str) -> bool {
        match self {
            ClientConfig::Anthropic(config) => config.is_beta_enabled(flag),
            #[cfg(feature = "vertex")]
            ClientConfig::Vertex(config) => config.is_beta_enabled(flag),
        }
    }

    pub(super) fn url(&self, path: &str) -> String {
        match self {
            ClientConfig::Anthropic(config) => config.url(path),
            #[cfg(feature = "vertex")]
            ClientConfig::Vertex(config) => config.url(path),
        }
    }

    pub(super) async fn headers(&self) -> Result<HeaderMap, Error<ApiErrorDetails>> {
        match self {
            ClientConfig::Anthropic(config) => config.headers(),
            #[cfg(feature = "vertex")]
            ClientConfig::Vertex(config) => config.headers().await,
        }
    }

    #[allow(unused_variables)]
    pub(super) fn messages_path(&self, model: &str, stream: bool) -> String {
        match self {
            ClientConfig::Anthropic(_) => "/messages".to_owned(),
            #[cfg(feature = "vertex")]
            ClientConfig::Vertex(config) => config.messages_path(model, stream),
        }
    }

    pub(super) fn count_tokens_path(&self) -> String {
        match self {
            ClientConfig::Anthropic(_) => "/messages/count_tokens".into(),
            #[cfg(feature = "vertex")]
            ClientConfig::Vertex(config) => config.messages_path("count-tokens", false),
        }
    }

    pub(super) fn messages_body(
        &self,
        request: &MessagesRequest,
    ) -> Result<serde_json::Value, Error<ApiErrorDetails>> {
        match self {
            ClientConfig::Anthropic(_) => serde_json::to_value(request).map_err(|err| {
                Error::JSONDeserialize(err, "failed to serialize request body".into())
            }),
            #[cfg(feature = "vertex")]
            ClientConfig::Vertex(config) => config.messages_body(request),
        }
    }
}

/// The default implementation for the [`ClientConfig`].
///
/// Defaults to the [`Anthropic`](ClientConfig::Anthropic) backend, reading the API key from the
/// `ANTHROPIC_API_KEY` environment variable.
impl Default for ClientConfig {
    fn default() -> Self {
        ClientConfig::Anthropic(AnthropicConfig::default())
    }
}

impl From<AnthropicConfig> for ClientConfig {
    fn from(config: AnthropicConfig) -> Self {
        ClientConfig::Anthropic(config)
    }
}

#[cfg(feature = "vertex")]
impl From<VertexConfig> for ClientConfig {
    fn from(config: VertexConfig) -> Self {
        ClientConfig::Vertex(config)
    }
}
