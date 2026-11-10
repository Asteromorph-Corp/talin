use async_trait::async_trait;
use talin_provider::openai_chat;
use talin_provider::openai_chat::types::{
    ChatCompletionRequestAssistantMessageContent, ChatCompletionRequestAssistantMessageContentPart,
    ChatCompletionRequestMessage, ChatCompletionRequestMessageContentPartText,
    ChatCompletionRequestToolMessage, ChatCompletionRequestToolMessageContent,
    ChatCompletionToolChoiceOption, ChatCompletionTools, ChatRequest, CompletionUsage,
    CreateChatCompletionResponse, FinishReason, ToolChoiceOptions,
};
use talin_provider::openai_chat::{Config, OpenAIConfig};

use crate::llm::error::ContinuationError;
use crate::llm::models::OpenAIModel;
use crate::llm::openai_chat::helper::{
    apply_options, response_to_request_message, select_choice, system_message, text_user_message,
};
use crate::llm::openai_chat::pricing::ChatPricing;
use crate::llm::openai_chat::{ChatOptions, OpenAIChatBuilder};
use crate::llm::{
    Llm, LlmError, LlmRequest, LlmRequestItem, LlmRequestOptions, LlmResponse, Truncation,
    UnsupportedContent, warn_unsupported_mcp,
};
use crate::mcp::McpServerHttp;
use crate::schemas::{BodyMessage, LlmStep, ModelKey, ToolErrorMessage};
use crate::text::prompts::CONTINUE_PARTIAL;
use crate::tools::ToolSpec;
use crate::transcript::{ModelRecord, TruncatedRecord};
use crate::utils::NoneIfEmpty;

/// A client for OpenAI models that uses the Chat Completions API.
///
/// This struct implements tool use by relying on the model's structured tool call capabilities.
/// Consequently, it can also be used for non-OpenAI models with native tool call support as well.
#[derive(Clone)]
pub struct OpenAIChat<C: Config = OpenAIConfig> {
    /// The OpenAI client.
    client: openai_chat::Client<C>,
    /// The model id.
    model: String,
    /// The default call options for the LLM.
    pub(super) default_options: ChatOptions,
    /// The pricing for the configured model.
    pricing: Option<ChatPricing>,
}

impl OpenAIChat {
    /// Creates a [`OpenAIChatBuilder`] to configure an [`OpenAIChat`].
    ///
    /// This is the same as calling [`OpenAIChatBuilder::new`].
    ///
    /// # Example
    /// ```rust
    /// use talin::llm::OpenAIChat;
    ///
    /// let chat: OpenAIChat = OpenAIChat::builder().with_model("gpt-4o").build();
    /// ```
    #[must_use]
    pub fn builder() -> OpenAIChatBuilder {
        OpenAIChatBuilder::new()
    }
}

impl<C: Config> OpenAIChat<C> {
    /// Constructs a new [`OpenAIChat`].
    ///
    /// It is recommended to use [`OpenAIChat::builder()`] instead of calling the constructor
    /// directly.
    #[must_use]
    pub fn new(
        client: openai_chat::Client<C>,
        model: String,
        options: ChatOptions,
        pricing: Option<ChatPricing>,
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
impl<C> Llm for OpenAIChat<C>
where
    C: Config + Clone + Send + Sync + 'static,
{
    type Request = ChatRequest;
    type Response = CreateChatCompletionResponse;

    type System = Vec<String>;
    type Item = ChatCompletionRequestMessage;
    type BuiltinTool = ();
    type Options = ChatOptions;

    type Pricing = ChatPricing;
    type Usage = CompletionUsage;

    fn build_request(
        &self,
        system: Self::System,
        items: Vec<Self::Item>,
        tools: Vec<ToolSpec>,
        _: Vec<Self::BuiltinTool>,
        mcp_servers: Vec<McpServerHttp>,
        options: Self::Options,
    ) -> Self::Request {
        let model = self.model.clone();
        let options = self.default_options.clone().merge(options);

        let messages = system_message(system).into_iter().chain(items).collect();

        let tools = ToolSpec::into_functions(tools)
            .into_iter()
            .map(ChatCompletionTools::from)
            .collect::<Vec<_>>()
            .none_if_empty();
        warn_unsupported_mcp("OpenAI chat", &mcp_servers);

        let mut request = ChatRequest::new(model, messages, tools);
        apply_options(&mut request, options);
        request
    }

    async fn send_request(&self, request: Self::Request) -> Result<Self::Response, LlmError> {
        let response = self.client.chat().generate(&request).await?;
        log::trace!("Received response: {response:?}");
        Ok(response)
    }

    fn parse_response(
        &self,
        response: Self::Response,
    ) -> Result<ModelRecord<Self::Item>, LlmError> {
        let choice = select_choice(&response.choices)
            .ok_or(LlmError::invalid_response("`choices` is empty"))?;
        if let Some(refusal) = choice.message.refusal {
            return Err(LlmError::Refused(refusal));
        }

        let native = response_to_request_message(choice.message.clone());
        let canonical = LlmStep::try_from(choice.message)?;
        Ok(ModelRecord { canonical, native })
    }

    fn build_continuation(
        &self,
        mut request: Self::Request,
        partial: TruncatedRecord<Self::Item>,
    ) -> Result<Self::Request, ContinuationError> {
        // Continuation pointless for structured output.
        if request.inner.response_format.is_some() {
            return Err(ContinuationError::StructuredOutput);
        }

        let prompt = text_user_message(CONTINUE_PARTIAL);
        request.inner.messages.extend([partial.native, prompt]);
        Ok(request)
    }

    fn supports_mcp(&self) -> bool {
        false
    }

    fn model_key(&self) -> ModelKey {
        ModelKey::new(self.model.clone(), "OpenAI chat")
    }

    fn pricing(&self) -> Option<Self::Pricing> {
        self.pricing
    }

    fn with_default_options(mut self, options: impl Into<Self::Options>) -> Self {
        self.default_options = self.default_options.merge(options.into());
        self
    }
}

impl LlmRequest for ChatRequest {
    fn clear_tool_choice(mut self) -> Self {
        self.inner.tool_choice = None;
        self
    }

    fn force_final(mut self) -> Self {
        self.inner.tool_choice = Some(ChatCompletionToolChoiceOption::Mode(
            ToolChoiceOptions::None,
        ));
        self
    }
}

impl LlmResponse for CreateChatCompletionResponse {
    type Usage = CompletionUsage;

    fn get_usage(&self) -> Option<Self::Usage> {
        self.usage.clone()
    }

    fn truncation(&self) -> Option<Truncation> {
        select_choice(&self.choices)
            .is_some_and(|choice| choice.finish_reason == Some(FinishReason::Length))
            .then_some(Truncation::max_tokens())
    }
}

impl LlmRequestItem for ChatCompletionRequestMessage {
    fn concat(self, continuation: Self) -> Result<Self, UnsupportedContent> {
        let (Self::Assistant(prefix), Self::Assistant(mut next)) = (self, continuation) else {
            return Err(UnsupportedContent::new(
                "OpenAI Chat Completions",
                "non-assistant continuation",
            ));
        };
        let mut parts = Vec::new();
        for content in [prefix.content, next.content.take()].into_iter().flatten() {
            match content {
                ChatCompletionRequestAssistantMessageContent::Text(text) => {
                    parts.push(ChatCompletionRequestAssistantMessageContentPart::Text(
                        ChatCompletionRequestMessageContentPartText {
                            text,
                            prompt_cache_breakpoint: None,
                        },
                    ))
                }
                ChatCompletionRequestAssistantMessageContent::Array(items) => parts.extend(items),
            }
        }
        next.content = Some(ChatCompletionRequestAssistantMessageContent::Array(parts));
        Ok(Self::Assistant(next))
    }

    fn from_message(message: BodyMessage) -> Result<Self, UnsupportedContent> {
        Self::try_from(message)
    }

    fn from_message_with(
        message: BodyMessage,
        _: Self,
        _: bool,
    ) -> Result<Self, UnsupportedContent> {
        // `BodyMessage` is modeled around chat completion request message, so it holds all
        // information that the previous message could have had.
        Self::from_message(message)
    }

    fn from_tool_error(message: &ToolErrorMessage) -> Option<Self> {
        let content = ChatCompletionRequestToolMessageContent::Text(message.render());
        let message = ChatCompletionRequestToolMessage {
            tool_call_id: message.call_id.clone(),
            content,
        };
        Some(Self::from(message))
    }
}

impl Default for OpenAIChat<OpenAIConfig> {
    fn default() -> Self {
        Self::new(
            openai_chat::Client::default(),
            OpenAIModel::Gpt4oMini.to_string(),
            ChatOptions::default(),
            None,
        )
    }
}
