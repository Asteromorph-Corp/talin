//! Gemini Interactions API client.

use async_trait::async_trait;
use talin_provider::gemini_interactions;
use talin_provider::gemini_interactions::types::{
    CreateInteractionRequest, FunctionResultStep, FunctionResultValue, Interaction,
    InteractionStatus, InteractionsInput, ModelInteraction, Step, ToolChoice as GeminiToolChoice,
    ToolMode, Usage,
};

use crate::llm::error::ContinuationError;
use crate::llm::gemini_interactions::helper::{check_status, map_tools, model_steps};
use crate::llm::{
    Llm, LlmBuiltinTool, LlmError, LlmRequest, LlmRequestItem, LlmRequestOptions, LlmResponse,
    Truncation, UnsupportedContent, warn_unsupported_mcp,
};
use crate::mcp::McpServerHttp;
use crate::schemas::{BodyMessage, LlmStep, ModelKey, ToolErrorMessage};
use crate::text::prompts::CONTINUE_PARTIAL;
use crate::tools::ToolSpec;
use crate::transcript::{ModelRecord, TruncatedRecord};

mod builder;
mod conversion;
mod helper;
mod options;
mod pricing;

pub use builder::GeminiInteractionsBuilder;
pub use options::InteractionsOptions;
pub use pricing::InteractionsPricing;
pub use talin_provider::gemini_interactions::ClientConfig as InteractionsConfig;
pub use talin_provider::gemini_interactions::types::BuiltinTool as InteractionsBuiltinTool;

/// Native [`gemini_interactions`] types used in requests, responses, and options.
///
/// Re-exported so that request/response manipulation (e.g. in custom strategies) and
/// [`InteractionsOptions`] configuration can name these types without depending on
/// `gemini_interactions` directly.
pub mod types {
    pub use talin_provider::gemini_interactions::types::*;
}

/// The client for Gemini models via the stateful Interactions API.
#[derive(Debug, Clone)]
pub struct GeminiInteractions {
    /// The client used to interact with the Gemini API.
    client: gemini_interactions::Client,
    /// The model name to use.
    model: String,
    /// The default call options for the LLM.
    default_options: InteractionsOptions,
    /// The pricing for the configured model.
    pricing: Option<InteractionsPricing>,
}

/// [`CreateModelInteractionParams`] with a `stream` flag.
#[doc(hidden)]
pub struct GeminiInteractionsRequest {
    pub inner: ModelInteraction,
    pub stream: bool,
}

impl GeminiInteractions {
    /// Creates a [`GeminiInteractionsBuilder`] to configure a [`GeminiInteractions`].
    ///
    /// This is the same as calling [`GeminiInteractionsBuilder::default`].
    #[must_use]
    pub fn builder() -> GeminiInteractionsBuilder {
        GeminiInteractionsBuilder::default()
    }

    /// Constructs a new [`GeminiInteractions`].
    ///
    /// It is recommended to use [`GeminiInteractions::builder`] instead of calling the constructor
    /// directly.
    #[must_use]
    pub fn new(
        client: gemini_interactions::Client,
        model: String,
        default_options: InteractionsOptions,
        pricing: Option<InteractionsPricing>,
    ) -> Self {
        Self {
            client,
            model,
            default_options,
            pricing,
        }
    }
}

impl Default for GeminiInteractions {
    fn default() -> Self {
        Self::builder().build()
    }
}

#[async_trait]
impl Llm for GeminiInteractions {
    type Request = GeminiInteractionsRequest;
    type Response = Interaction;

    type System = Vec<String>;
    type Item = Vec<Step>;
    type BuiltinTool = InteractionsBuiltinTool;
    type Options = InteractionsOptions;

    type Pricing = InteractionsPricing;
    type Usage = Usage;

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

        // A single message can expand to several steps (e.g. one per tool call), so items are
        // flattened into a single ordered step list.
        let steps = items.into_iter().flatten().collect::<Vec<Step>>();

        let mut inner = ModelInteraction::new(self.model.clone(), steps);

        if !system.is_empty() {
            inner.system_instruction = Some(system.join("\n\n"));
        }

        inner.tools = map_tools(tools, builtin_tools);
        warn_unsupported_mcp("Gemini Interactions", &mcp_servers);

        let mut generation_config = options.generation_config;
        if let Some(tool_choice) = options.tool_choice {
            generation_config.tool_choice = Some(GeminiToolChoice::from(tool_choice));
        }
        inner.generation_config = Some(generation_config);
        inner.response_format = options.response_format;

        let stream = options.stream.unwrap_or(false);
        GeminiInteractionsRequest { inner, stream }
    }

    async fn send_request(&self, request: Self::Request) -> Result<Self::Response, LlmError> {
        let body = CreateInteractionRequest::Model(Box::new(request.inner));
        let response = if request.stream {
            self.client.interactions().create_with_stream(&body).await?
        } else {
            self.client.interactions().create(&body).await?
        };

        log::trace!("Received response: {response:?}");
        Ok(response)
    }

    fn parse_response(
        &self,
        response: Self::Response,
    ) -> Result<ModelRecord<Self::Item>, LlmError> {
        check_status(response.status.clone())?;

        let native = model_steps(&response);
        if native.is_empty() {
            return Err(LlmError::invalid_response("no model step in `steps`"));
        }

        let canonical = LlmStep::from(native.clone());
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

        let mut steps = match request.inner.input {
            InteractionsInput::Steps(steps) => steps,
            InteractionsInput::Content(content) => vec![Step::user_input(vec![content])],
            InteractionsInput::Contents(contents) => vec![Step::user_input(contents)],
            InteractionsInput::Text(text) => vec![Step::user_text(text)],
            _ => Vec::new(),
        };
        steps.extend(partial.native);
        steps.push(Step::user_text(CONTINUE_PARTIAL));
        request.inner.input = InteractionsInput::Steps(steps);
        Ok(request)
    }

    fn supports_mcp(&self) -> bool {
        false
    }

    fn model_key(&self) -> ModelKey {
        ModelKey::new(self.model.clone(), "Google")
    }

    fn pricing(&self) -> Option<Self::Pricing> {
        self.pricing
    }

    fn with_default_options(mut self, options: impl Into<Self::Options>) -> Self {
        self.default_options = self.default_options.merge(options.into());
        self
    }
}

impl LlmRequest for GeminiInteractionsRequest {
    fn clear_tool_choice(mut self) -> Self {
        if let Some(generation_config) = &mut self.inner.generation_config {
            generation_config.tool_choice = None;
        }
        self
    }

    fn force_final(mut self) -> Self {
        self.inner.tools = None;
        if let Some(gen_config) = &mut self.inner.generation_config {
            gen_config.tool_choice = Some(GeminiToolChoice::Mode(ToolMode::None));
        }
        self
    }
}

impl LlmResponse for Interaction {
    type Usage = Usage;

    fn get_usage(&self) -> Option<Usage> {
        self.usage.clone()
    }

    fn truncation(&self) -> Option<Truncation> {
        // The interaction completed with a truncated result (e.g. hitting `max_output_tokens`).
        matches!(self.status, InteractionStatus::Incomplete).then_some(Truncation::max_tokens())
    }
}

impl LlmRequestItem for Vec<Step> {
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
        let is_model = matches!(
            &message,
            BodyMessage::Assistant(_) | BodyMessage::ToolCall(_)
        );

        let mut steps = Vec::new();
        // Thought steps carry backend signatures and can only be reused for a model turn.
        if retain_thought && is_model {
            steps.extend(base.into_iter().filter(|s| matches!(s, Step::Thought(_))));
        }
        steps.extend(Self::from_message(message)?);
        Ok(steps)
    }

    fn from_tool_error(message: &ToolErrorMessage) -> Option<Self> {
        let result = FunctionResultValue::Text(message.render());
        let step = FunctionResultStep::new(message.call_id.clone(), result)
            .with_name(message.name.clone())
            .mark_error();
        Some(vec![Step::FunctionResult(step)])
    }
}

impl LlmBuiltinTool for InteractionsBuiltinTool {}
