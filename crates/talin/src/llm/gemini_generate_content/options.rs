use talin_provider::gemini_generate_content::types::{
    GenerationConfig, ResponseFormatConfig, ResponseMimeType, ThinkingConfig,
};

use crate::llm::merge::Merge;
use crate::llm::{LlmOptions, LlmRequestOptions};
use crate::schemas::{ResponseFormat, ToolChoice};

/// Call options for the Gemini Generate Content API.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct GenerateContentOptions {
    /// The generation configuration applied to the request.
    pub generation_config: GenerationConfig,

    /// Controls which (if any) tool is called by the model.
    pub tool_choice: Option<ToolChoice>,

    /// Whether to stream the response.
    pub stream: Option<bool>,
}

impl GenerateContentOptions {
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

    /// Sets the [`stream`](Self::stream).
    pub fn with_stream(mut self, stream: bool) -> Self {
        self.stream = Some(stream);
        self
    }
}

impl From<LlmOptions> for GenerateContentOptions {
    fn from(options: LlmOptions) -> Self {
        Self::from_options(options)
    }
}

impl LlmRequestOptions for GenerateContentOptions {
    fn from_options(options: LlmOptions) -> Self {
        let response_json_schema = options.response_format.and_then(|format| match format {
            ResponseFormat::Text => None,
            ResponseFormat::JsonObject => Some(schemars::schema_for!(serde_json::Value)),
            ResponseFormat::JsonSchema(schema) => Some(schema.schema),
        });
        let response_mime_type = response_json_schema
            .is_some()
            .then_some(ResponseMimeType::ApplicationJson);

        let mut gen_config = GenerationConfig::new();
        gen_config.candidate_count = options.n.map(|n| n as u32);
        gen_config.thinking_config = options.reasoning_effort.map(ThinkingConfig::from);
        gen_config.temperature = options.temperature;
        gen_config.top_p = options.top_p;
        gen_config.top_k = options.top_k;
        gen_config.frequency_penalty = options.frequency_penalty;
        gen_config.presence_penalty = options.presence_penalty;
        gen_config.max_output_tokens = options.max_completion_tokens;
        gen_config.stop_sequences = options.stop_words;
        gen_config.response_json_schema = response_json_schema;
        gen_config.response_mime_type = response_mime_type;

        GenerateContentOptions {
            generation_config: gen_config,
            tool_choice: options.tool_choice,
            stream: options.stream,
        }
    }

    fn merge(self, other: Self) -> Self {
        Merge::merge(self, other)
    }
}

impl Merge for GenerateContentOptions {
    fn merge_inplace(&mut self, other: Self) {
        self.generation_config
            .merge_inplace(other.generation_config);
        self.tool_choice = other.tool_choice.or(self.tool_choice.take());
        self.stream = other.stream.or(self.stream);
    }
}

impl Merge for GenerationConfig {
    fn merge_inplace(&mut self, other: Self) {
        self.stop_sequences.merge_inplace(other.stop_sequences);
        self.response_mime_type = other.response_mime_type.or(self.response_mime_type.take());
        self.response_json_schema = other
            .response_json_schema
            .or(self.response_json_schema.take());
        self.response_format.merge_inplace(other.response_format);
        self.translation_config = other.translation_config.or(self.translation_config.take());
        self.response_modalities = other
            .response_modalities
            .or(self.response_modalities.take());
        self.candidate_count = other.candidate_count.or(self.candidate_count);
        self.max_output_tokens = other.max_output_tokens.or(self.max_output_tokens);
        self.temperature = other.temperature.or(self.temperature);
        self.top_p = other.top_p.or(self.top_p);
        self.top_k = other.top_k.or(self.top_k);
        self.seed = other.seed.or(self.seed);
        self.presence_penalty = other.presence_penalty.or(self.presence_penalty);
        self.frequency_penalty = other.frequency_penalty.or(self.frequency_penalty);
        self.response_logprobs = other.response_logprobs.or(self.response_logprobs);
        self.logprobs = other.logprobs.or(self.logprobs);
        self.enable_enhanced_civic_answers = other
            .enable_enhanced_civic_answers
            .or(self.enable_enhanced_civic_answers);
        self.speech_config = other.speech_config.or(self.speech_config.take());
        self.thinking_config.merge_inplace(other.thinking_config);
        self.image_config = other.image_config.or(self.image_config.take());
        self.media_resolution = other.media_resolution.or(self.media_resolution.take());
    }
}

impl Merge for ResponseFormatConfig {
    fn merge_inplace(&mut self, other: Self) {
        self.text = other.text.or(self.text.take());
        self.image = other.image.or(self.image.take());
        self.audio = other.audio.or(self.audio.take());
    }
}

impl Merge for ThinkingConfig {
    fn merge_inplace(&mut self, other: Self) {
        self.include_thoughts = other.include_thoughts.or(self.include_thoughts);
        self.thinking_budget = other.thinking_budget.or(self.thinking_budget);
        self.thinking_level = other.thinking_level.or(self.thinking_level.take());
    }
}
