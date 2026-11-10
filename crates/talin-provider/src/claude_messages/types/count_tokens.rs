use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use super::{
    CacheControlEphemeral, ContextManagementConfig, MessageParam, MessagesRequest, Model,
    OutputConfig, SystemBlockParam, TextOrItems, ThinkingConfigParam, ToolChoice, ToolUnion,
};

/// Input to the [token counting endpoint](https://platform.claude.com/docs/en/api/beta/messages/count_tokens).
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize)]
#[non_exhaustive]
pub struct CountTokensRequest {
    /// The model whose tokenizer counts the prompt.
    pub model: Model,
    /// The native conversation to count.
    pub messages: Vec<MessageParam>,
    /// The system prompt included in the count.
    pub system: Option<TextOrItems<SystemBlockParam>>,
    /// Tool definitions included in the count.
    pub tools: Option<Vec<ToolUnion>>,
    /// Tool selection policy.
    pub tool_choice: Option<ToolChoice>,
    /// Thinking configuration for the request.
    pub thinking: Option<ThinkingConfigParam>,
    /// Output configuration, including structured output schemas.
    pub output_config: Option<OutputConfig>,
    /// Request-level cache control.
    pub cache_control: Option<CacheControlEphemeral>,
    /// Context edits to apply before counting.
    pub context_management: Option<ContextManagementConfig>,
    #[cfg(feature = "mcp-client-2025-11-20")]
    /// MCP servers available to the request.
    pub mcp_servers: Option<Vec<super::McpServer>>,
}

impl From<&MessagesRequest> for CountTokensRequest {
    fn from(request: &MessagesRequest) -> Self {
        Self {
            model: request.model.clone(),
            messages: request.messages.clone(),
            system: request.system.clone(),
            tools: request.tools.clone(),
            tool_choice: request.tool_choice.clone(),
            thinking: request.thinking.clone(),
            output_config: request.output_config.clone(),
            cache_control: request.cache_control.clone(),
            context_management: request.context_management.clone(),
            #[cfg(feature = "mcp-client-2025-11-20")]
            mcp_servers: request.mcp_servers.clone(),
        }
    }
}

/// Token count reported for the supplied prompt, including system and tool definitions.
#[derive(Debug, Clone, Deserialize)]
#[non_exhaustive]
pub struct MessageTokensCount {
    /// Input token count reported by the provider.
    pub input_tokens: u32,
}
