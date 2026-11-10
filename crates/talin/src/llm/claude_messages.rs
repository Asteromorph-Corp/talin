//! Claude messages API client.

use async_trait::async_trait;
use talin_provider::claude_messages;
use talin_provider::claude_messages::beta_headers::MCP_CLIENT;
use talin_provider::claude_messages::types::{
    ContentBlockParam, MessageParam, MessagesRequest, MessagesResponse, OutputConfig, StopReason,
    SystemBlockParam, TextBlockParam, TextOrItems, ToolChoice, ToolResultBlockParam, Usage,
};

use crate::llm::claude_messages::helper::{
    check_stop_reason, discard_compacted, into_items, is_usable_compaction, map_tools,
    text_user_message,
};
use crate::llm::error::ContinuationError;
use crate::llm::{
    Llm, LlmBuiltinTool, LlmError, LlmRequest, LlmRequestItem, LlmRequestOptions, LlmRequestSystem,
    LlmResponse, Truncation, UnsupportedContent,
};
use crate::mcp::McpServerHttp;
use crate::schemas::{BodyMessage, LlmStep, ModelKey, ToolErrorMessage};
use crate::strategy::CompactionModel;
use crate::text::prompts::CONTINUE_PARTIAL;
use crate::tools::ToolSpec;
use crate::transcript::{ModelRecord, TruncatedRecord};

mod builder;
mod conversion;
mod helper;
mod options;
mod pricing;

pub use builder::{ClaudeMessagesBuilder, VertexClaudeMessagesBuilder};
pub use options::MessagesOptions;
pub use pricing::MessagesPricing;
pub use talin_provider::claude_messages::ClientConfig as MessagesConfig;
pub use talin_provider::claude_messages::types::BuiltinTool as MessagesBuiltinTool;

/// Native [`claude_messages`] types used in requests, responses, and options.
///
/// Re-exported so that request/response manipulation (e.g. in custom strategies) and
/// [`MessagesOptions`] configuration can name these types without depending on `claude_messages`
/// directly.
pub mod types {
    pub use talin_provider::claude_messages::types::*;
}

/// The client for Anthropic Claude models.
#[derive(Debug, Clone)]
pub struct ClaudeMessages {
    /// The HTTP client to use for requests.
    client: claude_messages::Client,
    /// The Anthropic model to use.
    model: String,
    /// The default call options for the LLM.
    default_options: MessagesOptions,
    /// The pricing for the configured model.
    pricing: Option<MessagesPricing>,
}

const DEFAULT_MAX_TOKENS: u32 = 32000;
pub(crate) type ClaudeSystemMessage = Option<TextOrItems<SystemBlockParam>>;

impl ClaudeMessages {
    /// Creates a [`ClaudeMessagesBuilder`] to configure a [`ClaudeMessages`].
    ///
    /// This is the same as calling [`ClaudeMessagesBuilder::default`].
    #[must_use]
    pub fn builder() -> ClaudeMessagesBuilder {
        ClaudeMessagesBuilder::default()
    }

    /// Creates a new [`ClaudeMessages`] instance.
    ///
    /// It is recommended to use [`ClaudeMessages::builder`] instead of calling the constructor
    /// directly.
    #[must_use]
    pub fn new(
        client: claude_messages::Client,
        model: String,
        options: MessagesOptions,
        pricing: Option<MessagesPricing>,
    ) -> Self {
        Self {
            client,
            model,
            default_options: options,
            pricing,
        }
    }
}

#[async_trait]
impl Llm for ClaudeMessages {
    type Request = MessagesRequest;
    type Response = MessagesResponse;

    type System = ClaudeSystemMessage;
    type Item = MessageParam;
    type BuiltinTool = MessagesBuiltinTool;
    type Options = MessagesOptions;

    type Pricing = MessagesPricing;
    type Usage = Usage;

    fn build_request(
        &self,
        system: Self::System,
        mut items: Vec<Self::Item>,
        tools: Vec<ToolSpec>,
        builtin_tools: Vec<Self::BuiltinTool>,
        mcp_servers: Vec<McpServerHttp>,
        options: Self::Options,
    ) -> Self::Request {
        let model = self.model.clone();
        let options = self.default_options.clone().merge(options);
        let mcp_enabled = self.client.is_beta_enabled(MCP_CLIENT);
        let (tools, mcp_servers) = map_tools(tools, mcp_servers, builtin_tools, mcp_enabled);
        let max_tokens = options.max_tokens.unwrap_or(DEFAULT_MAX_TOKENS);

        discard_compacted(&mut items);
        let mut request = MessagesRequest::new(model, items, max_tokens);
        request.system = system;
        request.tools = tools;
        request.mcp_servers = mcp_servers;
        request.stream = options.stream;
        request.tool_choice = options.tool_choice;
        request.thinking = options.thinking;
        request.stop_sequences = options.stop_sequences;
        let mut output_config = OutputConfig::new();
        output_config.effort = options.output_effort;
        output_config.format = options.output_format;
        request.output_config = Some(output_config);
        request.cache_control = options.cache_control;

        request
    }

    async fn send_request(&self, request: Self::Request) -> Result<Self::Response, LlmError> {
        let response = self.client.messages().generate(&request).await?;

        log::trace!("Received response: {response:?}");
        Ok(response)
    }

    fn parse_response(
        &self,
        response: Self::Response,
    ) -> Result<ModelRecord<Self::Item>, LlmError> {
        if let Some(reason) = response.stop_reason {
            check_stop_reason(reason, response.stop_details)?;
        }

        let canonical = LlmStep::from(response.content.clone());

        let native_items = response
            .content
            .into_iter()
            .map(ContentBlockParam::from)
            .collect::<Vec<_>>();
        let native = MessageParam::assistant(TextOrItems::Items(native_items));

        Ok(ModelRecord { canonical, native })
    }

    fn build_continuation(
        &self,
        mut request: Self::Request,
        partial: TruncatedRecord<Self::Item>,
    ) -> Result<Self::Request, ContinuationError> {
        // Continuation pointless for structured output.
        if request
            .output_config
            .as_ref()
            .is_some_and(|c| c.format.is_some())
        {
            return Err(ContinuationError::StructuredOutput);
        }

        let prompt = text_user_message(CONTINUE_PARTIAL);
        request.messages.extend([partial.native, prompt]);
        Ok(request)
    }

    fn supports_mcp(&self) -> bool {
        self.client.is_beta_enabled(MCP_CLIENT)
    }

    fn model_key(&self) -> ModelKey {
        ModelKey::new(self.model.clone(), "Anthropic")
    }

    fn pricing(&self) -> Option<Self::Pricing> {
        self.pricing
    }

    fn with_default_options(mut self, options: impl Into<Self::Options>) -> Self {
        self.default_options = self.default_options.merge(options.into());
        self
    }
}

impl LlmRequest for MessagesRequest {
    fn clear_tool_choice(mut self) -> Self {
        self.tool_choice = None;
        self
    }

    fn force_final(mut self) -> Self {
        self.tool_choice = Some(ToolChoice::None);
        self
    }
}

impl LlmResponse for MessagesResponse {
    type Usage = Usage;

    fn get_usage(&self) -> Option<Self::Usage> {
        Some(self.usage.clone())
    }

    fn truncation(&self) -> Option<Truncation> {
        match self.stop_reason {
            Some(StopReason::MaxTokens) => Some(Truncation::max_tokens()),
            Some(StopReason::PauseTurn) => Some(Truncation::new(
                "server-side sampling loop reaches its iteration limit",
            )),
            _ => None,
        }
    }
}

impl LlmRequestSystem for ClaudeSystemMessage {
    fn from_contents(contents: Vec<String>) -> Self {
        if contents.is_empty() {
            return None;
        } else if contents.len() == 1 {
            let content = contents.into_iter().next().expect("First element exists");
            return Some(TextOrItems::Text(content));
        }

        let items = contents
            .into_iter()
            .map(|text| SystemBlockParam::Text(TextBlockParam::new(text)))
            .collect::<Vec<_>>();
        Some(TextOrItems::Items(items))
    }

    fn from_contents_with(contents: Vec<String>, base: Self) -> Self {
        let Some(TextOrItems::Items(base)) = base else {
            return Self::from_contents(contents);
        };

        // Replace text parts with the new contents; preserve everything else (e.g. media).
        let base_items = base
            .into_iter()
            .filter(|p| !matches!(p, SystemBlockParam::Text(_)));
        let new_items = contents
            .into_iter()
            .map(|text| SystemBlockParam::Text(TextBlockParam::new(text)));
        let items = base_items.chain(new_items).collect::<Vec<_>>();
        Some(TextOrItems::Items(items))
    }
}

impl LlmRequestItem for MessageParam {
    fn contains_native_compaction(&self) -> bool {
        matches!(self.content(), Some(TextOrItems::Items(blocks)) if blocks.iter().any(is_usable_compaction))
    }

    fn concat(self, continuation: Self) -> Result<Self, UnsupportedContent> {
        let is_user = match &self {
            MessageParam::User(_) => true,
            MessageParam::Assistant(_) => false,
            _ => {
                return Err(UnsupportedContent::new(
                    "Claude `/messages`",
                    "non-block continuation",
                ));
            }
        };
        let (Some(TextOrItems::Items(mut items)), Some(TextOrItems::Items(next))) =
            (self.into_content(), continuation.into_content())
        else {
            return Err(UnsupportedContent::new(
                "Claude `/messages`",
                "non-block continuation",
            ));
        };
        items.extend(next);

        let content = TextOrItems::Items(items);
        if is_user {
            Ok(MessageParam::user(content))
        } else {
            Ok(MessageParam::assistant(content))
        }
    }

    fn from_message(message: BodyMessage) -> Result<Self, UnsupportedContent> {
        // Claude accepts every media source the canonical schema can express.
        MessageParam::try_from(message)
    }

    fn from_message_with(
        message: BodyMessage,
        base: Self,
        retain_thought: bool,
    ) -> Result<Self, UnsupportedContent> {
        // A plain-text baseline has nothing rich to preserve; rebuild from scratch.
        let is_user = matches!(message, BodyMessage::User(_) | BodyMessage::Tool(_));
        match (&message, &base) {
            (BodyMessage::User(_) | BodyMessage::Tool(_), MessageParam::User(_)) => {}
            (BodyMessage::Assistant(_) | BodyMessage::ToolCall(_), MessageParam::Assistant(_)) => {}
            _ => return Self::from_message(message),
        };

        let Some(TextOrItems::Items(items)) = base.into_content() else {
            return Self::from_message(message);
        };

        let filter_fn = |item: &ContentBlockParam| match item {
            ContentBlockParam::Text(_)
            | ContentBlockParam::ToolUse(_)
            | ContentBlockParam::ToolResult(_) => false,
            ContentBlockParam::Thinking(_) | ContentBlockParam::RedactedThinking(_) => {
                retain_thought
            }
            _ => true,
        };

        let mut base_items = items.into_iter().filter(filter_fn).peekable();

        if base_items.peek().is_none() {
            return Self::from_message(message);
        }

        let new_items = Self::from_message(message)?
            .into_content()
            .into_iter()
            .flat_map(into_items);
        let content = TextOrItems::Items(base_items.chain(new_items).collect::<Vec<_>>());

        Ok(if is_user {
            MessageParam::user(content)
        } else {
            MessageParam::assistant(content)
        })
    }

    fn from_tool_error(error: &ToolErrorMessage) -> Option<Self> {
        let result = ToolResultBlockParam::new(error.call_id.clone())
            .with_content(TextOrItems::Text(error.feedback.to_string()))
            .with_is_error(true);
        let content = TextOrItems::Items(vec![ContentBlockParam::ToolResult(result)]);
        Some(MessageParam::user(content))
    }
}

impl LlmBuiltinTool for MessagesBuiltinTool {}

impl Default for ClaudeMessages {
    fn default() -> Self {
        ClaudeMessages::builder().build()
    }
}

#[async_trait]
impl CompactionModel for ClaudeMessages {
    async fn count_tokens(&self, request: &MessagesRequest) -> Result<u32, LlmError> {
        Ok(self
            .client
            .messages()
            .count_tokens(&request.into())
            .await?
            .input_tokens)
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use super::*;
    use crate::schemas::UserMessage;

    #[test]
    fn completed_responses_keep_compaction_native_payload() {
        let client = ClaudeMessages::builder().with_api_key("test").build();
        for stop_reason in ["end_turn"] {
            for block in [
                serde_json::json!({"type": "compaction", "content": "summary", "signature": "sig", "tool_changes": null}),
                serde_json::json!({"type": "compaction", "content": null, "encrypted_content": "opaque", "signature": "sig", "tool_changes": []}),
                serde_json::json!({"type": "compaction", "content": null, "signature": "sig"}),
            ] {
                let response: MessagesResponse = serde_json::from_value(serde_json::json!({
                    "id": "msg_test", "type": "message", "role": "assistant", "model": "test",
                    "content": [block, {"type":"text", "text":"answer"}], "stop_reason": stop_reason,
                    "usage": {"input_tokens": 10, "output_tokens": 2},
                }))
                .unwrap();
                let record = client.parse_response(response).unwrap();
                assert_eq!(
                    serde_json::to_value(record.native()).unwrap(),
                    serde_json::json!({
                        "role": "assistant", "content": [block, {"type":"text", "text":"answer"}],
                    })
                );
            }
        }
    }

    #[tokio::test]
    #[ignore = "Claude API key is required."]
    async fn test_claude_generate() -> Result<(), Box<dyn Error>> {
        let api_key = std::env::var("CLAUDE_API_KEY").expect("missing env var: `CLAUDE_API_KEY`");

        let http_client = reqwest::Client::builder()
            .connection_verbose(true)
            .build()
            .unwrap();
        let client = ClaudeMessages::builder()
            .with_api_key(api_key)
            .with_http_client(http_client)
            .build();

        let system = vec!["You are number guesser, here to guess numbers.".to_owned()];
        let user = BodyMessage::User(UserMessage::new(
            "Guess the number I am thinking of. It's between 1 and 10.",
        ));

        let request = client.build_request(
            ClaudeSystemMessage::from_contents(system),
            vec![MessageParam::try_from(user)?],
            vec![],
            vec![],
            vec![],
            MessagesOptions::default(),
        );
        let response = client.send_request(request).await?;
        println!("Content: {:?}", response.content);
        println!("{:?}", response.usage);

        Ok(())
    }
}
