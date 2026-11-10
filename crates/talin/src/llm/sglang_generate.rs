//! SGLang `/generate` API client.

use async_trait::async_trait;
use talin_provider::sglang_generate;
use talin_provider::sglang_generate::types::{
    Batched, FinishReason, GenerateRequest, GenerateResponse, SamplingParams,
};

use crate::llm::error::ContinuationError;
use crate::llm::instructor::CompletionInstructor;
use crate::llm::{
    Llm, LlmError, LlmRequest, LlmRequestItem, LlmRequestOptions, LlmResponse, Truncation,
    UnsupportedContent, warn_unsupported_mcp,
};
use crate::mcp::McpServerHttp;
use crate::schemas::{
    AssistantMessage, BodyMessage, ContentExt, ModelKey, ToolErrorMessage, UserMessage,
};
use crate::text::prompts::FORCE_FINAL_ANSWER;
use crate::tools::ToolSpec;
use crate::transcript::{ModelRecord, TruncatedRecord};

mod builder;
mod options;

pub use builder::SglangGenerateBuilder;
pub use options::GenerateOptions;
pub use pricing::{GeneratePricing, GenerateUsage};
pub use talin_provider::sglang_generate::ClientConfig as GenerateConfig;

mod pricing;

/// Native [`sglang_generate`] types used in requests, responses, and options.
///
/// Re-exported so that request/response manipulation (e.g. in custom strategies) and
/// [`GenerateOptions`] configuration can name these types without depending on `sglang_generate`
/// directly.
pub mod types {
    pub use talin_provider::sglang_generate::types::*;
}

/// The client for SGLang models using the `/generate` API.
///
/// SGLang's `/generate` endpoint takes a single text prompt rather than a structured list of
/// messages, and has no native tool calling.
///
/// Both concerns are delegated to a [`CompletionInstructor`]: it flattens the conversation into a
/// single prompt and provides the tool use instruction and parser.
pub struct SglangGenerate {
    /// The client used to interact with the SGLang server.
    client: sglang_generate::Client,
    /// The model name to report in usage records.
    model: String,
    /// The instructor used to format the prompt and parse tool calls.
    instructor: Box<dyn CompletionInstructor>,
    /// The default call options for the LLM.
    default_options: GenerateOptions,
    /// The pricing for the configured model.
    pricing: Option<GeneratePricing>,
}

impl SglangGenerate {
    /// Creates a [`SglangGenerateBuilder`] targeting the given server base URL.
    ///
    /// This is the same as calling [`SglangGenerateBuilder::new`].
    #[must_use]
    pub fn builder(base_url: impl Into<String>) -> SglangGenerateBuilder {
        SglangGenerateBuilder::new(base_url)
    }

    /// Constructs a new [`SglangGenerate`].
    ///
    /// It is recommended to use [`SglangGenerate::builder`] instead of calling the constructor
    /// directly.
    #[must_use]
    pub fn new(
        client: sglang_generate::Client,
        model: impl Into<String>,
        instructor: Box<dyn CompletionInstructor>,
        default_options: GenerateOptions,
        pricing: Option<GeneratePricing>,
    ) -> Self {
        Self {
            client,
            model: model.into(),
            instructor,
            default_options,
            pricing,
        }
    }

    /// Sets the [instructor](`CompletionInstructor`) for the model.
    ///
    /// The instructor is used to format prompts, create tool use instruction, and parse tool calls.
    pub fn with_instructor(mut self, instructor: impl CompletionInstructor + 'static) -> Self {
        self.instructor = Box::new(instructor);
        self
    }
}

/// A request to the SGLang `/generate` endpoint.
///
/// The conversation is kept in its structured form and flattened into a single prompt only at send
/// time, so [`force_final`](LlmRequest::force_final) can still drop the tool instruction after the
/// request is built.
#[doc(hidden)]
pub struct SglangGenerateRequest {
    /// The system prompts.
    pub system: Vec<String>,
    /// The conversation body.
    pub messages: Vec<BodyMessage>,
    /// The tool use instruction, folded into the system prompt unless `force_final` is set.
    pub tool_use_instruction: Option<String>,
    /// The sampling parameters.
    pub sampling_params: SamplingParams,
    /// Whether to stream the response.
    pub stream: bool,
    /// Whether the model is forced to emit a final text answer.
    pub force_final: bool,
    /// A model output the prompt is left open on, so the model continues writing it.
    pub partial: Option<String>,
}

impl SglangGenerate {
    /// Flattens the request into the wire-level [`GenerateRequest`].
    fn materialize_request(&self, request: SglangGenerateRequest) -> GenerateRequest {
        let mut system = request.system;
        if !request.force_final {
            system.extend(request.tool_use_instruction);
        }

        let mut messages = request.messages;
        if request.force_final {
            messages.push(BodyMessage::User(UserMessage::new(FORCE_FINAL_ANSWER)));
        }

        let mut prompt = self.instructor.format_prompt(&system, &messages);
        // The prompt is left open on a model turn, so a partial output continues it directly.
        prompt.push_str(request.partial.as_deref().unwrap_or_default());

        GenerateRequest {
            text: Some(Batched::Single(prompt)),
            sampling_params: Some(Batched::Single(request.sampling_params)),
            stream: request.stream,
            ..Default::default()
        }
    }
}

#[async_trait]
impl Llm for SglangGenerate {
    type Request = SglangGenerateRequest;
    type Response = GenerateResponse;

    type System = Vec<String>;
    type Item = GenerateMessage;
    type BuiltinTool = ();
    type Options = GenerateOptions;

    type Pricing = GeneratePricing;
    type Usage = GenerateUsage;

    fn build_request(
        &self,
        system: Self::System,
        items: Vec<Self::Item>,
        tools: Vec<ToolSpec>,
        _: Vec<Self::BuiltinTool>,
        mcp_servers: Vec<McpServerHttp>,
        options: Self::Options,
    ) -> Self::Request {
        let options = self.default_options.clone().merge(options);

        let functions = ToolSpec::into_functions(tools);
        let tool_use_instruction =
            (!functions.is_empty()).then(|| self.instructor.tool_use_instruction(&functions));
        warn_unsupported_mcp("SGLang", &mcp_servers);

        SglangGenerateRequest {
            system,
            messages: items.into_iter().map(|item| item.0).collect(),
            tool_use_instruction,
            sampling_params: options.sampling_params(),
            stream: options.stream.unwrap_or(false),
            force_final: false,
            partial: None,
        }
    }

    async fn send_request(&self, request: Self::Request) -> Result<Self::Response, LlmError> {
        let request = self.materialize_request(request);

        let response = self.client.generate(&request).await?;

        // A non-batched request returns a single object; tolerate an array just in case.
        let response = match response {
            Batched::Single(generation) => generation,
            Batched::Batch(generations) => generations
                .into_iter()
                .next()
                .ok_or_else(|| LlmError::invalid_response("response is empty"))?,
        };

        log::trace!("Received response: {response:?}");
        Ok(response)
    }

    fn parse_response(
        &self,
        response: Self::Response,
    ) -> Result<ModelRecord<Self::Item>, LlmError> {
        if let Some(FinishReason::Abort { message, .. }) = &response.meta_info.finish_reason {
            return Err(LlmError::Refused(message.clone()));
        }

        let text = response.text.unwrap_or_default();
        let canonical = self.instructor.parse_tool_use(text.clone())?;
        let native = GenerateMessage(BodyMessage::Assistant(AssistantMessage::new(None, text)));
        Ok(ModelRecord { canonical, native })
    }

    fn build_continuation(
        &self,
        mut request: Self::Request,
        partial: TruncatedRecord<Self::Item>,
    ) -> Result<Self::Request, ContinuationError> {
        // Continuation pointless for structured output.
        if request.sampling_params.json_schema.is_some() {
            return Err(ContinuationError::StructuredOutput);
        }
        request.partial = partial.native.0.content().to_text();
        Ok(request)
    }

    fn parse_continuation(
        &self,
        response: Self::Response,
        partial: TruncatedRecord<Self::Item>,
    ) -> Result<ModelRecord<Self::Item>, LlmError> {
        if let Some(FinishReason::Abort { message, .. }) = &response.meta_info.finish_reason {
            return Err(LlmError::Refused(message.clone()));
        }
        let partial_raw = partial.native.0.content().to_text().unwrap_or_default();
        let continuation_raw = response.text.unwrap_or_default();
        let text = format!("{partial_raw}{continuation_raw}");
        let canonical = self.instructor.parse_tool_use(text.clone())?;
        let native = GenerateMessage(BodyMessage::Assistant(AssistantMessage::new(None, text)));
        Ok(ModelRecord { canonical, native })
    }

    fn supports_mcp(&self) -> bool {
        false
    }

    fn model_key(&self) -> ModelKey {
        ModelKey::new(self.model.clone(), "SGLang")
    }

    fn pricing(&self) -> Option<Self::Pricing> {
        self.pricing
    }

    fn with_default_options(mut self, options: impl Into<Self::Options>) -> Self {
        self.default_options = self.default_options.merge(options.into());
        self
    }
}

/// A [`SglangGenerate`] request item.
///
/// `/generate` takes a single prompt string, so the conversation is rendered to text by the
/// [`CompletionInstructor`] and a message carrying media has nowhere to go. Rejecting it here keeps
/// the failure at the point the content enters the transcript, rather than silently sending a
/// prompt that omits it.
#[derive(Debug, Clone)]
pub struct GenerateMessage(pub BodyMessage);

impl LlmRequestItem for GenerateMessage {
    fn from_message(message: BodyMessage) -> Result<Self, UnsupportedContent> {
        if !message.content().is_text() {
            return Err(UnsupportedContent::new("SGLang `/generate`", "media input"));
        }
        Ok(Self(message))
    }

    fn from_message_with(
        message: BodyMessage,
        _base: Self,
        _retain_thought: bool,
    ) -> Result<Self, UnsupportedContent> {
        // The item is the canonical message, so there is no provider-side block to carry over.
        Self::from_message(message)
    }

    fn from_tool_error(error: &ToolErrorMessage) -> Option<Self> {
        Some(Self(BodyMessage::Tool(error.as_output())))
    }
}

impl LlmRequest for SglangGenerateRequest {
    fn clear_tool_choice(self) -> Self {
        // SGLang has no native tool choice; tool use is driven entirely by the prompt.
        self
    }

    fn force_final(mut self) -> Self {
        self.force_final = true;
        self
    }
}

impl LlmResponse for GenerateResponse {
    type Usage = GenerateUsage;

    fn truncation(&self) -> Option<Truncation> {
        matches!(
            self.meta_info.finish_reason,
            Some(FinishReason::Length { .. })
        )
        .then_some(Truncation::max_tokens())
    }

    fn get_usage(&self) -> Option<Self::Usage> {
        Some(GenerateUsage::from(&self.meta_info))
    }
}

impl Clone for SglangGenerate {
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

    use super::*;
    use crate::llm::LlmExt;
    use crate::schemas::{ImageMimeType, MediaSource, Message, Prompt};

    #[test]
    fn accepts_text_message() {
        let message = BodyMessage::User(UserMessage::new("hi"));
        assert!(GenerateMessage::from_message(message).is_ok());
    }

    #[test]
    fn rejects_image_over_silent_drop() {
        let image = MediaSource::from_raw(vec![1, 2, 3]).image(ImageMimeType::Png);
        let message = BodyMessage::User(UserMessage::new("what is this?").with_images(vec![image]));

        let error = GenerateMessage::from_message(message).unwrap_err();
        assert!(error.to_string().contains("SGLang"));
    }

    #[tokio::test]
    #[ignore = "Requires a running SGLang server and appropriate environment variables"]
    async fn test_sglang_invoke() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        dotenvy::dotenv().ok();
        let base_url = std::env::var("SGLANG_BASE_URL")?;
        let model = std::env::var("SGLANG_MODEL")?;

        let sglang = SglangGenerate::builder(base_url).with_model(model).build();
        let response = sglang.invoke("Hello, world!").await?;
        println!("{response}");
        Ok(())
    }

    #[tokio::test]
    #[ignore = "Requires a running SGLang server and appropriate environment variables"]
    async fn test_sglang_generate() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        dotenvy::dotenv().ok();
        let base_url = std::env::var("SGLANG_BASE_URL")?;
        let model = std::env::var("SGLANG_MODEL")?;

        let prompt = Prompt::new(vec![
            Message::new_system_message("You are a helpful assistant"),
            Message::new_human_message("Hello, world!"),
            Message::new_ai_message(
                Some("The user must think I am the world, I must clarify that I am not".into()),
                "Hello, but I am not \"world\", I am an AI assistant.",
            ),
        ]);

        let sglang = SglangGenerate::builder(base_url).with_model(model).build();
        let response = sglang
            .generate(prompt, vec![], vec![], vec![], GenerateOptions::default())
            .await?;
        println!("{:?}", response.content);
        Ok(())
    }
}
