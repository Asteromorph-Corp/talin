//! xAI responses API client.

use async_trait::async_trait;
use talin_provider::xai_responses;
use talin_provider::xai_responses::types::{
    IncompleteDetails, InputMessage, InputTextItem, ModelInput, ModelInputContent,
    ModelInputContentItem, ModelInputPart, ModelOutput, ModelRequest, ModelResponse,
    ModelResponseFormat, ModelTool, ModelToolChoice, ModelToolChoiceMode, ModelUsage,
    OutputMessageContent, ResponseStatus, Role,
};

use crate::llm::error::ContinuationError;
use crate::llm::xai_responses::helper::text_user_parts;
use crate::llm::{
    Llm, LlmBuiltinTool, LlmError, LlmRequest, LlmRequestItem, LlmRequestOptions, LlmRequestSystem,
    LlmResponse, Truncation, UnsupportedContent,
};
use crate::mcp::McpServerHttp;
use crate::schemas::{BodyMessage, LlmStep, ModelKey, ToolErrorMessage};
use crate::text::prompts::CONTINUE_PARTIAL;
use crate::tools::ToolSpec;
use crate::transcript::{ModelRecord, TruncatedRecord};
use crate::utils::NoneIfEmpty;

mod builder;
mod conversion;
mod helper;
mod options;
mod pricing;

pub use builder::XaiResponsesBuilder;
pub use options::ResponsesOptions;
pub use pricing::ResponsesPricing;
pub use talin_provider::xai_responses::ClientConfig as ResponsesConfig;
pub use talin_provider::xai_responses::types::BuiltinTool as ResponsesBuiltinTool;

/// Native [`xai_responses`] types used in provider's requests, responses, and options.
///
/// Re-exported so that request/response manipulation (e.g. in custom strategies) and
/// [`ResponsesOptions`] configuration can name these types without depending on `xai_responses`
/// directly.
pub mod types {
    pub use talin_provider::xai_responses::types::*;
}

/// The client for xAI models.
#[derive(Debug, Clone)]
pub struct XaiResponses {
    /// The HTTP client to use for requests.
    pub(crate) client: xai_responses::Client,
    /// The xAI model to use.
    model: String,
    /// The default call options for the LLM.
    default_options: ResponsesOptions,
    /// The pricing for the configured model.
    pricing: Option<ResponsesPricing>,
}

impl XaiResponses {
    /// Creates a [`XaiResponsesBuilder`] to configure a [`XaiResponses`].
    ///
    /// This is the same as calling [`XaiResponsesBuilder::default`].
    #[must_use]
    pub fn builder() -> XaiResponsesBuilder {
        XaiResponsesBuilder::default()
    }

    /// Constructs a new [`XaiResponses`].
    #[must_use]
    pub fn new(
        client: xai_responses::Client,
        model: String,
        options: ResponsesOptions,
        pricing: Option<ResponsesPricing>,
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
impl Llm for XaiResponses {
    type Request = ModelRequest;
    type Response = ModelResponse;

    type System = Option<InputMessage>;
    type Item = Vec<ModelInputPart>;
    type BuiltinTool = ResponsesBuiltinTool;
    type Options = ResponsesOptions;

    type Pricing = ResponsesPricing;
    type Usage = ModelUsage;

    fn build_request(
        &self,
        system: Self::System,
        items: Vec<Self::Item>,
        tools: Vec<ToolSpec>,
        builtin_tools: Vec<Self::BuiltinTool>,
        mcp_servers: Vec<McpServerHttp>,
        options: Self::Options,
    ) -> Self::Request {
        let options = self.default_options.clone().merge(options);
        let model = self.model.clone();

        let messages = system
            .into_iter()
            .map(ModelInputPart::Message)
            .chain(items.into_iter().flatten())
            .collect::<Vec<_>>();

        let function_tools = tools.into_iter().map(ModelTool::from);
        let tools = function_tools
            .chain(builtin_tools.into_iter().map(ModelTool::from))
            .chain(mcp_servers.into_iter().map(ModelTool::from))
            .collect::<Vec<_>>()
            .none_if_empty();

        let ResponsesOptions {
            tool_choice,
            text,
            reasoning,
            temperature,
            top_p,
            max_output_tokens,
            parallel_tool_calls,
        } = options;

        let mut request = ModelRequest::new();
        request.model = Some(model);
        request.input = ModelInput::Parts(messages);
        request.tools = tools;
        request.tool_choice = tool_choice;
        request.stream = Some(false);
        request.temperature = temperature;
        request.top_p = top_p;
        request.max_output_tokens = max_output_tokens;
        request.parallel_tool_calls = parallel_tool_calls;
        request.text = text;
        request.reasoning = reasoning;
        request
    }

    async fn send_request(&self, request: Self::Request) -> Result<Self::Response, LlmError> {
        let response = self.client.responses().create(&request).await?;
        log::trace!("Received response: {response:?}");
        Ok(response)
    }

    fn parse_response(
        &self,
        response: Self::Response,
    ) -> Result<ModelRecord<Self::Item>, LlmError> {
        let canonical = LlmStep::from(response.output.clone());
        let native: Vec<_> = response
            .output
            .into_iter()
            .map(ModelInputPart::ModelOutput)
            .collect();
        Ok(ModelRecord { canonical, native })
    }

    fn build_continuation(
        &self,
        mut request: Self::Request,
        partial: TruncatedRecord<Self::Item>,
    ) -> Result<Self::Request, ContinuationError> {
        // Continuation pointless for structured output.
        if request
            .text
            .as_ref()
            .and_then(|t| t.format.as_ref())
            .is_some_and(|format| !matches!(format, ModelResponseFormat::Text))
        {
            return Err(ContinuationError::StructuredOutput);
        }

        let prompt = text_user_parts(CONTINUE_PARTIAL);
        if let ModelInput::Parts(parts) = &mut request.input {
            parts.extend(partial.native.into_iter().chain(prompt));
        }
        Ok(request)
    }

    fn model_key(&self) -> ModelKey {
        ModelKey::new(self.model.clone(), "Xai")
    }

    fn pricing(&self) -> Option<Self::Pricing> {
        self.pricing
    }

    fn with_default_options(mut self, options: impl Into<Self::Options>) -> Self {
        self.default_options = self.default_options.merge(options.into());
        self
    }
}

impl LlmRequest for ModelRequest {
    fn clear_tool_choice(mut self) -> Self {
        self.tool_choice = None;
        self
    }

    fn force_final(mut self) -> Self {
        self.tool_choice = Some(ModelToolChoice::Mode(ModelToolChoiceMode::None));
        self
    }
}

impl LlmResponse for ModelResponse {
    type Usage = ModelUsage;

    fn get_usage(&self) -> Option<Self::Usage> {
        self.usage.clone()
    }

    fn truncation(&self) -> Option<Truncation> {
        if self.status != ResponseStatus::Incomplete {
            return None;
        }
        let truncation = match &self.incomplete_details {
            Some(IncompleteDetails::MaxOutputTokens) => Truncation::max_tokens(),
            Some(IncompleteDetails::MaxPromptTokens) => {
                Truncation::new("prompt token limit reached")
            }
            Some(IncompleteDetails::MaxTimeLimit) => Truncation::new("time limit reached"),
            Some(IncompleteDetails::Unknown(reason)) => Truncation::new(reason.to_string()),
            _ => Truncation::unspecified(),
        };
        Some(truncation)
    }
}

impl LlmRequestSystem for Option<InputMessage> {
    fn from_contents(contents: Vec<String>) -> Self {
        if contents.is_empty() {
            return None;
        }

        let content = if contents.len() == 1 {
            ModelInputContent::Text(contents.into_iter().next().unwrap())
        } else {
            let parts = contents
                .into_iter()
                .map(|text| ModelInputContentItem::InputText(InputTextItem::new(text)))
                .collect();
            ModelInputContent::Parts(parts)
        };

        Some(InputMessage::new(content, Role::System))
    }

    fn from_contents_with(contents: Vec<String>, base: Self) -> Self {
        // A plain-text baseline has nothing rich to preserve; rebuild from scratch.
        let Some(InputMessage {
            content: ModelInputContent::Parts(base_parts),
            ..
        }) = base
        else {
            return Self::from_contents(contents);
        };

        // Replace text parts with the new contents; preserve everything else (e.g. images, files).
        let base_parts = base_parts
            .into_iter()
            .filter(|p| !matches!(p, ModelInputContentItem::InputText(_)));
        let new_parts = contents
            .into_iter()
            .map(|text| ModelInputContentItem::InputText(InputTextItem::new(text)));
        let new_parts = base_parts.chain(new_parts).collect::<Vec<_>>();

        Some(InputMessage::new(
            ModelInputContent::Parts(new_parts),
            Role::System,
        ))
    }
}

impl LlmRequestItem for Vec<ModelInputPart> {
    fn concat(mut self, continuation: Self) -> Result<Self, UnsupportedContent> {
        self.extend(continuation);
        Ok(self)
    }

    fn from_message(message: BodyMessage) -> Result<Self, UnsupportedContent> {
        Self::try_from(message)
    }

    fn from_message_with(
        message: BodyMessage,
        base: Self,
        retain_thought: bool,
    ) -> Result<Self, UnsupportedContent> {
        if base.is_empty() {
            return Self::from_message(message);
        }

        let mut base_roles = base.iter().filter_map(|p| match p {
            ModelInputPart::Message(m) => Some(m.role.clone()),
            ModelInputPart::FunctionCallOutput(_) => Some(Role::User),
            ModelInputPart::ModelOutput(_) => Some(Role::Assistant),
            _ => None,
        });
        let expected_role = match &message {
            BodyMessage::User(_) | BodyMessage::Tool(_) => Role::User,
            BodyMessage::Assistant(_) | BodyMessage::ToolCall(_) => Role::Assistant,
        };

        if base_roles.any(|r| r != expected_role) {
            return Self::from_message(message);
        }

        let filter_map_fn = |mut p: ModelInputPart| match &mut p {
            ModelInputPart::Message(msg) => {
                let ModelInputContent::Parts(items) = &mut msg.content else {
                    return None;
                };
                items.retain(|part| !matches!(part, ModelInputContentItem::InputText(_)));
                if items.is_empty() {
                    return None;
                }
                Some(p)
            }
            ModelInputPart::ModelOutput(ModelOutput::Message(msg)) => {
                msg.content
                    .retain(|c| !matches!(c, OutputMessageContent::OutputText(_)));
                if msg.content.is_empty() {
                    return None;
                }
                Some(p)
            }
            ModelInputPart::ModelOutput(ModelOutput::FunctionCall(_))
            | ModelInputPart::FunctionCallOutput(_) => None,
            ModelInputPart::ModelOutput(ModelOutput::Reasoning(_)) => retain_thought.then_some(p),
            _ => Some(p),
        };

        let base_parts = base.into_iter().filter_map(filter_map_fn);
        let new_parts = Self::from_message(message)?;
        Ok(base_parts.chain(new_parts).collect())
    }

    fn from_tool_error(message: &ToolErrorMessage) -> Option<Self> {
        // A tool result carries only text, so the conversion cannot reject it.
        Self::try_from(BodyMessage::Tool(message.as_output())).ok()
    }
}

impl LlmBuiltinTool for ResponsesBuiltinTool {}

impl Default for XaiResponses {
    fn default() -> Self {
        XaiResponses::builder().build()
    }
}
