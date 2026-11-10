use async_trait::async_trait;
use itertools::Itertools;
use talin_provider::openai_chat;
use talin_provider::openai_chat::types::{
    ChatCompletionRequestAssistantMessage, ChatCompletionRequestAssistantMessageContent,
    ChatCompletionRequestMessage, ChatCompletionRequestUserMessage,
    ChatCompletionRequestUserMessageContent, ChatRequest, CompletionUsage,
    CreateChatCompletionResponse,
};
use talin_provider::openai_chat::{Config, OpenAIConfig};

use crate::llm::error::ContinuationError;
use crate::llm::instructor::{DefaultInstructor, Instructor};
use crate::llm::models::OpenAIModel;
use crate::llm::openai_chat::helper::{
    apply_options, response_to_request_message, select_choice, system_message, text_user_message,
};
use crate::llm::openai_chat::pricing::ChatPricing;
use crate::llm::openai_chat::{ChatOptions, GenericChatBuilder};
use crate::llm::{
    Llm, LlmError, LlmRequest, LlmRequestItem, LlmRequestOptions, UnsupportedContent,
    warn_unsupported_mcp,
};
use crate::mcp::McpServerHttp;
use crate::schemas::{AssistantMessage, BodyMessage, ModelKey, ToolErrorMessage, UserMessage};
use crate::text::prompts::{CONTINUE_PARTIAL, FORCE_FINAL_ANSWER};
use crate::tools::ToolSpec;
use crate::transcript::{ModelRecord, TruncatedRecord};

/// The client for generic chat models that support OpenAI Chat Completions API.
///
/// This struct implements tool use by prompting the model to emit tool calls as plain text and
/// manually parsing them. The specific prompt and parsing logic is injected via the [`Instructor`]
/// object.
///
/// While this works with any OpenAI-comptaible model. However, for models that support structured
/// tool call, it is recommended to use [`OpenAIChat`](crate::llm::OpenAIChat) for better
/// reliability.
pub struct GenericChat<C: Config = OpenAIConfig> {
    /// The OpenAI client.
    client: openai_chat::Client<C>,
    /// The model id.
    model: String,
    /// The instructor used to create tool use instruction and parse tool calls.
    pub(super) instructor: Box<dyn Instructor>,
    /// The default call options for the LLM.
    pub(super) default_options: ChatOptions,
    /// The pricing for the configured model.
    pricing: Option<ChatPricing>,
}

#[derive(Debug, Clone)]
pub struct GenericChatRequest {
    pub system: Vec<String>,
    pub tool_use_instruction: Option<String>,
    pub request: ChatRequest,
}

impl GenericChat {
    /// Creates a [`GenericChatBuilder`] to configure a [`GenericChat`].
    ///
    /// This is the same as calling [`GenericChatBuilder::new`].
    ///
    /// # Example
    /// ```rust
    /// use talin::llm::GenericChat;
    /// use talin::llm::openai_chat::GenericChatBuilder;
    ///
    /// let chat: GenericChat = GenericChat::builder().with_model("gpt-4o-mini").build();
    /// ```
    #[must_use]
    pub fn builder() -> GenericChatBuilder {
        GenericChatBuilder::new()
    }
}

impl<C: Config> GenericChat<C> {
    /// Constructs a new [`GenericChat`].
    ///
    /// It is recommended to use [`GenericChat::builder()`] instead of calling the constructor
    /// directly.
    #[must_use]
    pub fn new(
        client: openai_chat::Client<C>,
        model: String,
        instructor: Box<dyn Instructor>,
        default_options: ChatOptions,
        pricing: Option<ChatPricing>,
    ) -> Self {
        Self {
            client,
            model,
            instructor,
            default_options,
            pricing,
        }
    }

    /// Sets the [instructor](`Instructor`) for the model.
    ///
    /// The instructor is used to create tool use instruction and parse tool calls.
    pub fn with_instructor(mut self, instructor: impl Instructor + 'static) -> Self {
        self.instructor = Box::new(instructor);
        self
    }
}

impl GenericChatRequest {
    fn materialize(self) -> ChatRequest {
        let mut request = self.request;
        let system = self
            .system
            .into_iter()
            .chain(self.tool_use_instruction)
            .collect::<Vec<_>>();
        let system_msg = system_message(system);
        request.inner.messages = system_msg
            .into_iter()
            .chain(request.inner.messages)
            .collect();
        request
    }
}

#[async_trait]
impl<C> Llm for GenericChat<C>
where
    C: Config + Clone + Send + Sync + 'static,
{
    type Request = GenericChatRequest;
    type Response = CreateChatCompletionResponse;

    type System = Vec<String>;
    type Item = GenericChatMessage;
    type BuiltinTool = ();
    type Options = ChatOptions;

    type Pricing = ChatPricing;
    type Usage = CompletionUsage;

    fn build_request(
        &self,
        mut system: Self::System,
        items: Vec<Self::Item>,
        tools: Vec<ToolSpec>,
        _: Vec<Self::BuiltinTool>,
        mcp_servers: Vec<McpServerHttp>,
        options: Self::Options,
    ) -> Self::Request {
        let model = self.model.clone();
        let options = self.default_options.clone().merge(options);

        let functions = ToolSpec::into_functions(tools);
        let tool_use_instruction =
            (!functions.is_empty()).then(|| self.instructor.tool_use_instruction(&functions));
        warn_unsupported_mcp("Generic chat", &mcp_servers);

        if let Some(rf) = options.response_format.clone()
            && let Some(rf_instruction) = rf.describe()
        {
            log::debug!("Response format instruction (Appended to system):\n{rf_instruction}");
            system.push(rf_instruction);
        };

        let messages = items.into_iter().map(|item| item.0).collect();
        let mut request = ChatRequest::new(model, messages, None);
        apply_options(&mut request, options);

        GenericChatRequest {
            system,
            tool_use_instruction,
            request,
        }
    }

    async fn send_request(&self, request: Self::Request) -> Result<Self::Response, LlmError> {
        let request = request.materialize();
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

        let native = GenericChatMessage(response_to_request_message(choice.message.clone()));
        let canonical = self
            .instructor
            .parse_tool_use(choice.message.content.unwrap_or_default())?;
        Ok(ModelRecord { canonical, native })
    }

    fn build_continuation(
        &self,
        mut request: Self::Request,
        partial: TruncatedRecord<Self::Item>,
    ) -> Result<Self::Request, ContinuationError> {
        request
            .request
            .inner
            .messages
            .extend([partial.native.0, text_user_message(CONTINUE_PARTIAL)]);
        Ok(request)
    }

    fn parse_continuation(
        &self,
        response: Self::Response,
        partial: TruncatedRecord<Self::Item>,
    ) -> Result<ModelRecord<Self::Item>, LlmError> {
        let choice = select_choice(&response.choices)
            .ok_or(LlmError::invalid_response("`choices` is empty"))?;
        if let Some(refusal) = choice.message.refusal {
            return Err(LlmError::Refused(refusal));
        }

        let ChatCompletionRequestMessage::Assistant(prefix) = partial.native.0 else {
            return Err(LlmError::invalid_response(
                "expected an assistant continuation",
            ));
        };
        let Some(ChatCompletionRequestAssistantMessageContent::Text(partial_raw)) = prefix.content
        else {
            return Err(LlmError::invalid_response(
                "expected an unmodified text continuation",
            ));
        };
        let continuation_raw = choice.message.content.unwrap_or_default();
        let text = format!("{partial_raw}{continuation_raw}");
        let canonical = self.instructor.parse_tool_use(text.clone())?;
        let native = GenericChatMessage(
            ChatCompletionRequestAssistantMessage {
                content: Some(ChatCompletionRequestAssistantMessageContent::Text(text)),
                ..Default::default()
            }
            .into(),
        );
        Ok(ModelRecord { canonical, native })
    }

    fn supports_mcp(&self) -> bool {
        false
    }

    fn model_key(&self) -> ModelKey {
        ModelKey::new(self.model.clone(), "Generic chat")
    }

    fn pricing(&self) -> Option<Self::Pricing> {
        self.pricing
    }

    fn with_default_options(mut self, options: impl Into<Self::Options>) -> Self {
        self.default_options = self.default_options.merge(options.into());
        self
    }
}

/// A [`GenericChat`] request item.
#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct GenericChatMessage(pub ChatCompletionRequestMessage);

impl LlmRequestItem for GenericChatMessage {
    fn from_message(message: BodyMessage) -> Result<Self, UnsupportedContent> {
        let message = match message {
            BodyMessage::ToolCall(message) => {
                let content = format!(
                    "```json\n{}\n```",
                    message
                        .tool_calls
                        .iter()
                        .map(|call| call.to_string())
                        .join("\n")
                );
                BodyMessage::Assistant(AssistantMessage::new(message.thought, content))
            }
            BodyMessage::Tool(message) => {
                BodyMessage::User(UserMessage::from_parts(message.output.content))
            }
            other => other,
        };
        Ok(Self(ChatCompletionRequestMessage::try_from(message)?))
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
        let content = ChatCompletionRequestUserMessageContent::Text(message.render());
        let message = ChatCompletionRequestUserMessage {
            content,
            name: None,
        };
        Some(Self(ChatCompletionRequestMessage::from(message)))
    }
}

impl LlmRequest for GenericChatRequest {
    fn clear_tool_choice(mut self) -> Self {
        self.request.inner.tool_choice = None;
        self
    }

    fn force_final(mut self) -> Self {
        self.tool_use_instruction = None;

        if let Some(ChatCompletionRequestMessage::Assistant(msg)) =
            self.request.inner.messages.last_mut()
        {
            if msg.tool_calls.is_some() {
                msg.tool_calls = None;
                msg.content = Some(ChatCompletionRequestAssistantMessageContent::Text(
                    "_".into(),
                ))
            }
        } else {
            #[allow(deprecated)]
            let message = ChatCompletionRequestAssistantMessage {
                content: Some(ChatCompletionRequestAssistantMessageContent::Text(
                    "_".to_owned(),
                )),
                refusal: None,
                name: None,
                audio: None,
                tool_calls: None,
                function_call: None,
            };
            self.request
                .inner
                .messages
                .push(ChatCompletionRequestMessage::from(message));
        }
        self.request
            .inner
            .messages
            .push(text_user_message(FORCE_FINAL_ANSWER));

        self
    }
}

impl Default for GenericChat<OpenAIConfig> {
    fn default() -> Self {
        Self::new(
            openai_chat::Client::default(),
            OpenAIModel::Gpt4oMini.to_string(),
            Box::new(DefaultInstructor),
            ChatOptions::default(),
            None,
        )
    }
}

impl<C: Config + Clone> Clone for GenericChat<C> {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            model: self.model.clone(),
            instructor: dyn_clone::clone_box(&*self.instructor),
            default_options: self.default_options.clone(),
            pricing: self.pricing,
        }
    }
}

#[cfg(test)]
mod tests {
    use talin_provider::openai_chat::types::ChatCompletionRequestAssistantMessageContent;

    use super::*;
    use crate::schemas::{ContentPart, ToolCall, ToolCallMessage, ToolMessage, ToolOutput};

    fn assistant_text(message: &GenericChatMessage) -> &str {
        match &message.0 {
            ChatCompletionRequestMessage::Assistant(m) => match m.content.as_ref() {
                Some(ChatCompletionRequestAssistantMessageContent::Text(text)) => text,
                other => panic!("expected assistant text, got {other:?}"),
            },
            other => panic!("expected an assistant message, got {other:?}"),
        }
    }

    #[test]
    fn maps_tool_call_to_assistant_text() {
        let call = ToolCall::new(
            Some("call_1".to_owned()),
            "search",
            Some(serde_json::json!({"q": "rust"})),
        );
        let message = BodyMessage::ToolCall(ToolCallMessage::new(None, vec![call]));

        // The model has no native tool calling, so the call is rendered as a fenced JSON block.
        let item = GenericChatMessage::from_message(message).unwrap();
        assert!(assistant_text(&item).starts_with("```json"));
    }

    #[test]
    fn maps_tool_result_to_user_text() {
        let output = ToolOutput {
            content: vec![ContentPart::Text("42".to_owned())],
            summary: None,
        };
        let message = BodyMessage::Tool(ToolMessage::new("call_1", "search", output));

        let item = GenericChatMessage::from_message(message).unwrap();
        assert!(matches!(item.0, ChatCompletionRequestMessage::User(_)));
    }
}
