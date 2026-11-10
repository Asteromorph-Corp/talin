use talin_provider::gemini_interactions::types::{
    GenerationConfig, ResponseFormat as GeminiResponseFormat, ResponseFormats, ThinkingLevel,
    ThinkingSummaries, ToolChoice as GeminiToolChoice, ToolChoiceConfig, TranscriptionConfig,
};

use crate::llm::merge::Merge;
use crate::llm::options::unsupported_option;
use crate::llm::{LlmOptions, LlmRequestOptions};
use crate::schemas::{ResponseFormat, ToolChoice};

/// Call options for the Gemini Interactions API.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct InteractionsOptions {
    /// The generation configuration applied to the request.
    pub generation_config: GenerationConfig,

    /// Controls which (if any) tool is called by the model.
    pub tool_choice: Option<ToolChoice>,

    /// The response format(s) enforced on the generated response.
    pub response_format: Option<ResponseFormats>,

    /// Whether to stream the response.
    pub stream: Option<bool>,
}

impl InteractionsOptions {
    /// Sets the [`generation_config`](Self::generation_config).
    pub fn with_generation_config(mut self, generation_config: GenerationConfig) -> Self {
        self.generation_config = generation_config;
        self
    }

    /// Sets the [`tool_choice`](Self::tool_choice).
    pub fn with_tool_choice(mut self, tool_choice: ToolChoice) -> Self {
        self.tool_choice = Some(tool_choice);
        self
    }

    /// Sets the [`response_format`](Self::response_format).
    pub fn with_response_format(mut self, response_format: impl Into<ResponseFormats>) -> Self {
        self.response_format = Some(response_format.into());
        self
    }

    /// Sets the [`stream`](Self::stream).
    pub fn with_stream(mut self, stream: bool) -> Self {
        self.stream = Some(stream);
        self
    }
}

impl From<LlmOptions> for InteractionsOptions {
    fn from(options: LlmOptions) -> Self {
        Self::from_options(options)
    }
}

impl LlmRequestOptions for InteractionsOptions {
    fn from_options(options: LlmOptions) -> Self {
        unsupported_option!("Gemini Interactions API", options.n);
        unsupported_option!("Gemini Interactions API", options.top_k);
        unsupported_option!("Gemini Interactions API", options.frequency_penalty);
        unsupported_option!("Gemini Interactions API", options.presence_penalty);
        unsupported_option!("Gemini Interactions API", options.repetition_penalty);
        unsupported_option!("Gemini Interactions API", options.parallel_tool_calls);

        let response_format = options
            .response_format
            .and_then(|format| match format {
                ResponseFormat::Text => None,
                ResponseFormat::JsonObject => Some(GeminiResponseFormat::json_schema(
                    schemars::schema_for!(serde_json::Value),
                )),
                ResponseFormat::JsonSchema(schema) => {
                    Some(GeminiResponseFormat::json_schema(schema.schema))
                }
            })
            .map(ResponseFormats::from);

        let mut generation_config = GenerationConfig::new();
        #[allow(deprecated)]
        {
            generation_config.temperature = options.temperature;
            generation_config.top_p = options.top_p;
        }
        generation_config.max_output_tokens = options.max_completion_tokens.map(|n| n as i32);
        generation_config.stop_sequences = options.stop_words;
        if let Some(effort) = options.reasoning_effort {
            generation_config.thinking_level = Some(ThinkingLevel::from(effort));
            generation_config.thinking_summaries = Some(ThinkingSummaries::Auto);
        }

        InteractionsOptions {
            generation_config,
            tool_choice: options.tool_choice,
            response_format,
            stream: options.stream,
        }
    }

    fn merge(self, other: Self) -> Self {
        Merge::merge(self, other)
    }
}

impl Merge for InteractionsOptions {
    fn merge_inplace(&mut self, other: Self) {
        self.generation_config
            .merge_inplace(other.generation_config);
        self.tool_choice = other.tool_choice.or(self.tool_choice.take());
        self.response_format = other.response_format.or(self.response_format.take());
        self.stream = other.stream.or(self.stream);
    }
}

impl Merge for GenerationConfig {
    #[expect(
        deprecated,
        reason = "all mergeable fields are merged, including the deprecated ones"
    )]
    fn merge_inplace(&mut self, other: Self) {
        self.image_config = other.image_config.or(self.image_config.take());
        self.max_output_tokens = other.max_output_tokens.or(self.max_output_tokens);
        self.seed = other.seed.or(self.seed);
        self.structured_speech_config = other
            .structured_speech_config
            .or(self.structured_speech_config.take());
        self.stop_sequences.merge_inplace(other.stop_sequences);
        self.thinking_level = other.thinking_level.or(self.thinking_level.take());
        self.thinking_summaries = other.thinking_summaries.or(self.thinking_summaries.take());
        self.tool_choice.merge_inplace(other.tool_choice);
        self.transcription_config
            .merge_inplace(other.transcription_config);
        self.video_config = other.video_config.or(self.video_config.take());
    }
}

impl Merge for GeminiToolChoice {
    fn merge_inplace(&mut self, other: Self) {
        match (self, other) {
            (GeminiToolChoice::Config(config), GeminiToolChoice::Config(other)) => {
                config.merge_inplace(other);
            }
            (tool_choice, other) => *tool_choice = other,
        }
    }
}

impl Merge for ToolChoiceConfig {
    fn merge_inplace(&mut self, other: Self) {
        self.allowed_tools = other.allowed_tools.or(self.allowed_tools.take());
    }
}

impl Merge for TranscriptionConfig {
    #[expect(
        deprecated,
        reason = "all mergeable fields are merged, including the deprecated ones"
    )]
    fn merge_inplace(&mut self, other: Self) {
        self.adaptation_phrases
            .merge_inplace(other.adaptation_phrases);
        self.custom_vocabulary
            .merge_inplace(other.custom_vocabulary);
        self.diarization_mode = other.diarization_mode.or(self.diarization_mode.take());
        self.language_codes.merge_inplace(other.language_codes);
        self.timestamp_granularities
            .merge_inplace(other.timestamp_granularities);
    }
}
