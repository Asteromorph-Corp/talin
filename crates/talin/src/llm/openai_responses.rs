//! OpenAI responses API client.

use async_trait::async_trait;
use talin_provider::openai_responses;
use talin_provider::openai_responses::types::{
    CreateResponse, FunctionCallOutput, FunctionCallOutputItemParam, InputItem, InputParam, Item,
    MessageItem, Response, ResponseUsage, Status, TextResponseFormatConfiguration, Tool,
    ToolChoiceOptions, ToolChoiceParam,
};
use talin_provider::openai_responses::{Config, OpenAIConfig};

use crate::llm::error::ContinuationError;
use crate::llm::models::OpenAIModel;
use crate::llm::openai_responses::helper::{
    discard_compacted, has_compaction, system_message, text_user_item,
};
use crate::llm::{
    Llm, LlmError, LlmRequest, LlmRequestItem, LlmRequestOptions, LlmResponse, Truncation,
    UnsupportedContent,
};
use crate::mcp::McpServerHttp;
use crate::schemas::{BodyMessage, LlmStep, ModelKey, ToolErrorMessage};
use crate::strategy::CompactionModel;
use crate::text::prompts::CONTINUE_PARTIAL;
use crate::tools::ToolSpec;
use crate::transcript::{ModelRecord, TruncatedRecord};
use crate::utils::NoneIfEmpty;

mod builder;
mod conversion;
pub(crate) mod helper;
mod options;
mod pricing;
mod tool;

pub use builder::OpenAIResponsesBuilder;
pub use options::ResponsesOptions;
pub use pricing::ResponsesPricing;
pub use tool::ResponsesBuiltinTool;

/// Native [`async_openai`] Responses-API types used in requests, responses, and options.
///
/// Re-exported so that request/response manipulation (e.g. in custom strategies) and
/// [`ResponsesOptions`] configuration can name these types without depending on `async_openai`
/// directly.
///
/// [`async_openai`]: https://crates.io/crates/async_openai
pub mod types {
    pub use talin_provider::openai_responses::types::*;
}

/// The [`Status::Incomplete`] reason reported when the output token limit is reached.
const MAX_OUTPUT_TOKENS: &str = "max_output_tokens";

/// A client for OpenAI models that uses the Responses API.
#[derive(Clone)]
pub struct OpenAIResponses<C: Config = OpenAIConfig> {
    /// The HTTP client to use for requests.
    client: openai_responses::Client<C>,
    /// The model id.
    model: String,
    /// The default call options for the LLM.
    pub(super) default_options: ResponsesOptions,
    /// The pricing information for the configured model.
    pricing: Option<ResponsesPricing>,
}

impl<C: Config + Default> OpenAIResponses<C> {
    /// Creates a [`OpenAIResponsesBuilder`] to configure an [`OpenAIResponses`].
    ///
    /// This is the same as calling [`OpenAIResponsesBuilder::new`].
    ///
    /// # Example
    /// ```rust
    /// use talin::llm::OpenAIResponses;
    ///
    /// let chat: OpenAIResponses = OpenAIResponses::builder().with_model("gpt-4o").build();
    /// ```
    #[must_use]
    pub fn builder() -> OpenAIResponsesBuilder<C> {
        OpenAIResponsesBuilder::new()
    }
}

impl<C: Config> OpenAIResponses<C> {
    /// Creates a new [`OpenAIResponses`] instance.
    pub fn new(
        client: openai_responses::Client<C>,
        model: String,
        default_options: ResponsesOptions,
        pricing: Option<ResponsesPricing>,
    ) -> Self {
        Self {
            client,
            model,
            default_options,
            pricing,
        }
    }
    /// Runs the native on-demand compaction endpoint without parsing an agent action.
    pub async fn compact(
        &self,
        request: types::CompactResponseRequest,
    ) -> Result<types::CompactResource, LlmError> {
        Ok(self.client.responses().compact(request).await?)
    }
}

impl Default for OpenAIResponses<OpenAIConfig> {
    fn default() -> Self {
        Self::new(
            openai_responses::Client::default(),
            OpenAIModel::Gpt4oMini.to_string(),
            ResponsesOptions::default(),
            None,
        )
    }
}

#[async_trait]
impl<C> Llm for OpenAIResponses<C>
where
    C: Config + Clone + Send + Sync + 'static,
{
    type Request = CreateResponse;
    type Response = Response;

    type System = Vec<String>;
    type Item = Vec<InputItem>;
    type BuiltinTool = ResponsesBuiltinTool;
    type Options = ResponsesOptions;

    type Pricing = ResponsesPricing;
    type Usage = ResponseUsage;

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

        let system_item = system_message(system)
            .map(|msg| InputItem::Item(Item::Message(MessageItem::Input(msg))));
        discard_compacted(&mut items);
        let body_items = items.into_iter().flatten();
        let input = InputParam::Items(
            system_item
                .into_iter()
                .chain(body_items)
                .collect::<Vec<_>>(),
        );

        let functions = tools.into_iter().map(Tool::from);
        let tools = functions
            .chain(builtin_tools.into_iter().map(Tool::from))
            .chain(mcp_servers.into_iter().map(Tool::from))
            .collect::<Vec<_>>()
            .none_if_empty();

        CreateResponse {
            input,
            max_output_tokens: options.max_output_tokens,
            model: Some(model),
            parallel_tool_calls: options.parallel_tool_calls,
            reasoning: options.reasoning,
            temperature: options.temperature,
            text: options.text,
            tool_choice: options.tool_choice,
            tools,
            top_p: options.top_p,
            ..Default::default()
        }
    }

    async fn send_request(&self, request: Self::Request) -> Result<Self::Response, LlmError> {
        let response = self.client.responses().generate(request).await?;
        log::trace!("Received response: {response:?}");
        Ok(response)
    }

    fn parse_response(
        &self,
        response: Self::Response,
    ) -> Result<ModelRecord<Self::Item>, LlmError> {
        let canonical = LlmStep::try_from(response.output.clone())?;
        let native = response
            .output
            .into_iter()
            .map(InputItem::from)
            .collect::<Vec<_>>();
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
            .is_some_and(|t| !matches!(t.format, TextResponseFormatConfiguration::Text))
        {
            return Err(ContinuationError::StructuredOutput);
        }

        let prompt = text_user_item(CONTINUE_PARTIAL);
        if let InputParam::Items(items) = &mut request.input {
            items.extend(partial.native.into_iter().chain([prompt]));
        }
        Ok(request)
    }

    fn model_key(&self) -> ModelKey {
        ModelKey::new(self.model.clone(), "OpenAI")
    }

    fn pricing(&self) -> Option<Self::Pricing> {
        self.pricing
    }

    fn with_default_options(mut self, options: impl Into<Self::Options>) -> Self {
        self.default_options = self.default_options.merge(options.into());
        self
    }
}

impl LlmRequest for CreateResponse {
    fn clear_tool_choice(mut self) -> Self {
        self.tool_choice = None;
        self
    }

    fn force_final(mut self) -> Self {
        self.tool_choice = Some(ToolChoiceParam::Option(ToolChoiceOptions::None));
        self
    }
}

impl LlmResponse for Response {
    type Usage = ResponseUsage;

    fn get_usage(&self) -> Option<Self::Usage> {
        self.usage.clone()
    }

    fn truncation(&self) -> Option<Truncation> {
        if self.status != Status::Incomplete {
            return None;
        }
        let truncation = match self.incomplete_details.as_ref().map(|d| d.reason.as_str()) {
            Some(MAX_OUTPUT_TOKENS) => Truncation::max_tokens(),
            Some(reason) => Truncation::new(reason),
            None => Truncation::unspecified(),
        };
        Some(truncation)
    }
}

impl LlmRequestItem for Vec<InputItem> {
    fn contains_native_compaction(&self) -> bool {
        has_compaction(self)
    }

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
        // Rebuild from scratch if base is empty
        if base.is_empty() {
            return Self::from_message(message);
        };

        let filter_fn = |item: &InputItem| match item {
            InputItem::EasyMessage(_)
            | InputItem::Item(Item::Message(_))
            | InputItem::Item(Item::FunctionCall(_))
            | InputItem::Item(Item::FunctionCallOutput(_)) => false,
            InputItem::Item(Item::Reasoning(_)) => retain_thought,
            _ => true,
        };

        let mut base_items = base.into_iter().filter(filter_fn).peekable();

        if base_items.peek().is_none() {
            return Self::from_message(message);
        }

        let new_items = Self::from_message(message)?;
        Ok(base_items.chain(new_items).collect::<Vec<_>>())
    }

    fn from_tool_error(message: &ToolErrorMessage) -> Option<Self> {
        let output = FunctionCallOutput::Text(message.render());
        let output = FunctionCallOutputItemParam {
            call_id: Some(message.call_id.clone()),
            output,
            id: None,
            status: None,
            name: None,
            namespace: None,
            caller: None,
        };
        Some(vec![InputItem::Item(Item::FunctionCallOutput(output))])
    }
}

#[async_trait]
impl<C: Config + Clone + Send + Sync + 'static> CompactionModel for OpenAIResponses<C> {
    async fn count_tokens(&self, request: &CreateResponse) -> Result<u32, LlmError> {
        let body = types::TokenCountsBody {
            input: Some(request.input.clone()),
            model: request.model.clone(),
            instructions: request.instructions.clone(),
            conversation: request.conversation.clone(),
            previous_response_id: request.previous_response_id.clone(),
            parallel_tool_calls: request.parallel_tool_calls,
            reasoning: request.reasoning.clone(),
            text: request.text.clone(),
            tools: request.tools.clone(),
            tool_choice: request.tool_choice.clone(),
            ..Default::default()
        };
        Ok(self
            .client
            .responses()
            .count_tokens(body)
            .await?
            .input_tokens)
    }
}
