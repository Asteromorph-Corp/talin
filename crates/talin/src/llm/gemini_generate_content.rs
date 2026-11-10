//! Gemini generateContent API client.

use async_trait::async_trait;
use talin_provider::gemini_generate_content;
use talin_provider::gemini_generate_content::types::{
    Content, Data, FinishReason, GenerateContentRequest, GenerateContentResponse, Part, Role,
    ToolConfig, UsageMetadata,
};

use crate::llm::error::ContinuationError;
use crate::llm::gemini_generate_content::helper::{
    check_finish_reason, map_tools, select_candidate,
};
use crate::llm::{
    Llm, LlmBuiltinTool, LlmError, LlmRequest, LlmRequestItem, LlmRequestOptions, LlmRequestSystem,
    LlmResponse, Truncation, UnsupportedContent, warn_unsupported_mcp,
};
use crate::mcp::McpServerHttp;
use crate::schemas::{BodyMessage, LlmStep, ModelKey, ToolChoice, ToolErrorMessage};
use crate::text::prompts::CONTINUE_PARTIAL;
use crate::tools::ToolSpec;
use crate::transcript::{ModelRecord, TruncatedRecord};

mod builder;
mod conversion;
mod helper;
mod options;
mod pricing;

pub use builder::GeminiGenerateContentBuilder;
pub use options::GenerateContentOptions;
pub use pricing::GenerateContentPricing;
pub use talin_provider::gemini_generate_content::ClientConfig as GenerateContentConfig;
pub use talin_provider::gemini_generate_content::types::BuiltinTool as GenerateContentBuiltinTool;

/// Native [`gemini_generate_content`] types used in requests, responses, and options.
///
/// Re-exported so that request/response manipulation (e.g. in custom strategies) and
/// [`GenerateContentOptions`] configuration can name these types without depending on
/// `gemini_generate_content` directly.
pub mod types {
    pub use talin_provider::gemini_generate_content::types::*;
}

/// The client for Gemini models.
#[derive(Debug, Clone)]
pub struct GeminiGenerateContent {
    /// The client used to interact with the Gemini API.
    client: gemini_generate_content::Client,
    /// The model name to use.
    model: String,
    /// The default call options for the LLM.
    default_options: GenerateContentOptions,
    /// The pricing for the configured model.
    pricing: Option<GenerateContentPricing>,
}

/// [`GenerateContentRequest`] with a `stream` flag.
#[doc(hidden)]
pub struct GeminiGenerateContentRequest {
    pub inner: GenerateContentRequest,
    pub stream: bool,
}

impl GeminiGenerateContent {
    /// Creates a [`GeminiGenerateContentBuilder`] to configure a [`GeminiGenerateContent`].
    ///
    /// This is the same as calling [`GeminiGenerateContentBuilder::default`].
    #[must_use]
    pub fn builder() -> GeminiGenerateContentBuilder {
        GeminiGenerateContentBuilder::default()
    }

    /// Constructs a new [`GeminiGenerateContent`].
    ///
    /// It is recommended to use [`GeminiGenerateContent::builder`] instead of calling the
    /// constructor directly.
    #[must_use]
    pub fn new(
        client: gemini_generate_content::Client,
        model: String,
        default_options: GenerateContentOptions,
        pricing: Option<GenerateContentPricing>,
    ) -> Self {
        Self {
            client,
            model,
            default_options,
            pricing,
        }
    }
}

impl Default for GeminiGenerateContent {
    fn default() -> Self {
        Self::builder().build()
    }
}

#[async_trait]
impl Llm for GeminiGenerateContent {
    type Request = GeminiGenerateContentRequest;
    type Response = GenerateContentResponse;

    type System = Option<Content>;
    type Item = Content;
    type BuiltinTool = GenerateContentBuiltinTool;
    type Options = GenerateContentOptions;

    type Pricing = GenerateContentPricing;
    type Usage = UsageMetadata;

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

        let (tools, tool_config) = map_tools(tools, builtin_tools, options.tool_choice);
        warn_unsupported_mcp("Gemini", &mcp_servers);

        let mut inner = GenerateContentRequest::new(items);
        inner.tools = tools;
        inner.tool_config = tool_config;
        inner.system_instruction = system;
        inner.generation_config = Some(options.generation_config);

        let stream = options.stream.unwrap_or(false);
        GeminiGenerateContentRequest { inner, stream }
    }

    async fn send_request(&self, request: Self::Request) -> Result<Self::Response, LlmError> {
        let response = if request.stream {
            self.client
                .generate_content(&self.model)
                .generate_with_stream(&request.inner)
                .await?
        } else {
            self.client
                .generate_content(&self.model)
                .generate(&request.inner)
                .await?
        };

        log::trace!("Received response: {response:?}");
        Ok(response)
    }

    fn parse_response(
        &self,
        response: Self::Response,
    ) -> Result<ModelRecord<Self::Item>, LlmError> {
        let Some(candidate) = response.candidates.and_then(select_candidate) else {
            return Err(LlmError::invalid_response("`candidates` is empty"));
        };

        if let Some(reason) = candidate.finish_reason {
            check_finish_reason(reason)?;
        }

        let Some(native) = candidate.content else {
            return Err(LlmError::other("missing usage metadata in response"));
        };
        let canonical = LlmStep::from(native.clone());

        Ok(ModelRecord { canonical, native })
    }

    fn build_continuation(
        &self,
        mut request: Self::Request,
        partial: TruncatedRecord<Self::Item>,
    ) -> Result<Self::Request, ContinuationError> {
        // Continuation pointless for structured output.
        if request
            .inner
            .generation_config
            .as_ref()
            .is_some_and(|c| c.response_json_schema.is_some())
        {
            return Err(ContinuationError::StructuredOutput);
        }

        let prompt = Content::user_text(CONTINUE_PARTIAL);
        request.inner.contents.extend([partial.native, prompt]);
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

impl LlmRequest for GeminiGenerateContentRequest {
    fn clear_tool_choice(mut self) -> Self {
        if let Some(tool_config) = &mut self.inner.tool_config {
            tool_config.function_calling_config = None;
        }
        self
    }

    fn force_final(mut self) -> Self {
        self.inner.tools = None;
        self.inner.tool_config = Some(ToolConfig::from(ToolChoice::None));
        self
    }
}

impl LlmResponse for GenerateContentResponse {
    type Usage = UsageMetadata;

    fn get_usage(&self) -> Option<UsageMetadata> {
        self.usage_metadata.clone()
    }

    fn truncation(&self) -> Option<Truncation> {
        self.candidates
            .clone()
            .and_then(select_candidate)
            .is_some_and(|candidate| candidate.finish_reason == Some(FinishReason::MaxTokens))
            .then_some(Truncation::max_tokens())
    }
}

impl LlmRequestSystem for Option<Content> {
    fn from_contents(contents: Vec<String>) -> Self {
        if contents.is_empty() {
            return None;
        }

        let mut system = Content::system();
        for content in contents {
            system = system.with_text(content);
        }
        Some(system)
    }

    fn from_contents_with(contents: Vec<String>, base: Self) -> Self {
        let Some(base) = base else {
            return Self::from_contents(contents);
        };

        // System content carries no role; if the baseline disagrees, its parts can't be
        // meaningfully carried over — rebuild fresh.
        if base.role.is_some() {
            return Self::from_contents(contents);
        }

        // Replace text parts with the new contents; preserve everything else (e.g. media).
        let base_parts = base
            .parts
            .into_iter()
            .flatten()
            .filter(|p| !matches!(&p.data, Data::Text(_)));
        let new_parts = contents.into_iter().map(Part::text);
        let parts = base_parts.chain(new_parts).collect::<Vec<_>>();

        if parts.is_empty() {
            return None;
        }
        // Role-less content (system instruction); preserve the assembled parts.
        let mut content = Content::system();
        content.parts = Some(parts);
        Some(content)
    }
}

impl LlmRequestItem for Content {
    fn concat(mut self, continuation: Self) -> Result<Self, UnsupportedContent> {
        self.parts
            .get_or_insert_default()
            .extend(continuation.parts.into_iter().flatten());
        Ok(self)
    }

    fn from_message(message: BodyMessage) -> Result<Self, UnsupportedContent> {
        Content::try_from(message)
    }

    fn from_message_with(
        message: BodyMessage,
        base: Self,
        retain_thought: bool,
    ) -> Result<Self, UnsupportedContent> {
        let expected_role = match &message {
            BodyMessage::User(_) | BodyMessage::Tool(_) => Role::User,
            BodyMessage::Assistant(_) | BodyMessage::ToolCall(_) => Role::Model,
        };

        if base.role.is_some_and(|r| r != expected_role) {
            return Self::from_message(message);
        }

        let filter_fn = |p: &Part| {
            if p.thought == Some(true) {
                return retain_thought;
            }
            !matches!(
                &p.data,
                Data::Text(_) | Data::FunctionCall(_) | Data::FunctionResponse(_)
            )
        };

        let base_parts = base.parts.into_iter().flatten().filter(filter_fn);
        let new_parts = Self::from_message(message)?.parts.into_iter().flatten();
        let parts = base_parts.chain(new_parts).collect::<Vec<_>>();

        Ok(Content::new_with_parts(expected_role, parts))
    }

    fn from_tool_error(message: &ToolErrorMessage) -> Option<Self> {
        Some(Content::from(message.as_output()))
    }
}

impl LlmBuiltinTool for GenerateContentBuiltinTool {}
