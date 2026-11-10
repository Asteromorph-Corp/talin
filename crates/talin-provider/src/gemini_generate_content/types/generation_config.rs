use schemars::Schema;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

mod image_config;
mod media_resolution;
mod modality;
mod response_format;
mod response_mime_type;
mod speech_config;
mod thinking_config;
mod translation_config;

pub use image_config::*;
pub use media_resolution::*;
pub use modality::*;
pub use response_format::*;
pub use response_mime_type::*;
pub use speech_config::*;
pub use thinking_config::*;
pub use translation_config::*;

/// Configuration options for model generation and outputs.
/// Not all parameters are configurable for every model.
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct GenerationConfig {
    /// Optional. The set of character sequences (up to 5) that will stop output generation.
    /// If specified, the API will stop at the first appearance of a stop_sequence.
    /// The stop sequence will not be included as part of the response.
    pub stop_sequences: Option<Vec<String>>,

    /// Optional. MIME type of the generated candidate text.
    /// Supported MIME types are:
    /// - `"text/plain"`: (default) Text output.
    /// - `"application/json"`: JSON response in the response candidates.
    /// - `"text/x.enum"`: ENUM as a string response in the response candidates.
    pub response_mime_type: Option<ResponseMimeType>,

    // /// Optional. Output schema of the generated candidate text, openAPI schema format.
    // /// pub response_schema: Option<serde_json::Value>,
    /// Optional. Output schema of the generated candidate text.
    pub response_json_schema: Option<Schema>,

    /// Optional. Configuration for the response output format.
    /// Allows specifying output configuration per modality (text, audio, image).
    pub response_format: Option<ResponseFormatConfig>,

    /// Optional. Config for translation.
    pub translation_config: Option<TranslationConfig>,

    /// Optional. The requested modalities of the response.
    /// Represents the set of modalities that the model can return.
    pub response_modalities: Option<Vec<Modality>>,

    /// Optional. Number of generated responses to return.
    /// If unset, this will default to 1.
    pub candidate_count: Option<u32>,

    /// Optional. The maximum number of tokens to include in a response candidate.
    pub max_output_tokens: Option<u32>,

    /// Optional. Controls the randomness of the output.
    /// Values can range from [0.0, 2.0].
    pub temperature: Option<f32>,

    /// Optional. The maximum cumulative probability of tokens to consider when sampling.
    /// The model uses combined Top-k and Top-p (nucleus) sampling.
    pub top_p: Option<f32>,

    /// Optional. The maximum number of tokens to consider when sampling.
    pub top_k: Option<u32>,

    /// Optional. Seed used in decoding. If not set, the request uses a randomly generated seed.
    pub seed: Option<u32>,

    /// Optional. Presence penalty applied to the next token's logprobs if the token has already
    /// been seen.
    pub presence_penalty: Option<f32>,

    /// Optional. Frequency penalty applied to the next token's logprobs.
    pub frequency_penalty: Option<f32>,

    /// Optional. If true, export the logprobs results in response.
    pub response_logprobs: Option<bool>,

    /// Optional. Only valid if responseLogprobs=True.
    /// This sets the number of top logprobs to return at each decoding step.
    /// The number must be in the range of [0, 20].
    pub logprobs: Option<u32>,

    /// Optional. Enables enhanced civic answers. It may not be available for all models.
    pub enable_enhanced_civic_answers: Option<bool>,

    /// Optional. The speech generation config.
    pub speech_config: Option<SpeechConfig>,

    /// Optional. Config for thinking features.
    pub thinking_config: Option<ThinkingConfig>,

    /// Optional. Config for image generation.
    pub image_config: Option<ImageConfig>,

    /// Optional. If specified, the media resolution specified will be used.
    pub media_resolution: Option<MediaResolution>,
}

impl GenerationConfig {
    /// Creates a new GenerationConfig with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the stop sequences.
    pub fn with_stop_sequences(mut self, stop_sequences: Vec<String>) -> Self {
        self.stop_sequences = Some(stop_sequences);
        self
    }

    /// Sets the response MIME type.
    pub fn with_response_mime_type(mut self, mime_type: ResponseMimeType) -> Self {
        self.response_mime_type = Some(mime_type);
        self
    }

    /// Sets the response JSON schema.
    pub fn with_response_json_schema(mut self, schema: Schema) -> Self {
        self.response_json_schema = Some(schema);
        self
    }

    /// Sets the response modalities.
    pub fn with_response_modalities(mut self, modalities: Vec<Modality>) -> Self {
        self.response_modalities = Some(modalities);
        self
    }

    /// Sets the candidate count.
    pub fn with_candidate_count(mut self, count: u32) -> Self {
        self.candidate_count = Some(count);
        self
    }

    /// Sets the maximum output tokens.
    pub fn with_max_output_tokens(mut self, tokens: u32) -> Self {
        self.max_output_tokens = Some(tokens);
        self
    }

    /// Sets the temperature (0.0 to 2.0).
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    /// Sets the top_p value.
    pub fn with_top_p(mut self, top_p: f32) -> Self {
        self.top_p = Some(top_p);
        self
    }

    /// Sets the top_k value.
    pub fn with_top_k(mut self, top_k: u32) -> Self {
        self.top_k = Some(top_k);
        self
    }

    /// Sets the random seed.
    pub fn with_seed(mut self, seed: u32) -> Self {
        self.seed = Some(seed);
        self
    }

    /// Sets the presence penalty.
    pub fn with_presence_penalty(mut self, penalty: f32) -> Self {
        self.presence_penalty = Some(penalty);
        self
    }

    /// Sets the frequency penalty.
    pub fn with_frequency_penalty(mut self, penalty: f32) -> Self {
        self.frequency_penalty = Some(penalty);
        self
    }

    /// Enables response logprobs.
    pub fn with_response_logprobs(mut self, enabled: bool) -> Self {
        self.response_logprobs = Some(enabled);
        self
    }

    /// Sets the number of logprobs to return (0-20).
    pub fn with_logprobs(mut self, logprobs: u32) -> Self {
        self.logprobs = Some(logprobs);
        self
    }

    /// Enables enhanced civic answers.
    pub fn with_enable_enhanced_civic_answers(mut self, enabled: bool) -> Self {
        self.enable_enhanced_civic_answers = Some(enabled);
        self
    }

    /// Sets the speech generation config.
    pub fn with_speech_config(mut self, config: SpeechConfig) -> Self {
        self.speech_config = Some(config);
        self
    }

    /// Sets the thinking config for thinking models.
    pub fn with_thinking_config(mut self, config: ThinkingConfig) -> Self {
        self.thinking_config = Some(config);
        self
    }

    /// Sets the image generation config.
    pub fn with_image_config(mut self, config: ImageConfig) -> Self {
        self.image_config = Some(config);
        self
    }

    /// Sets the media resolution.
    pub fn with_media_resolution(mut self, resolution: MediaResolution) -> Self {
        self.media_resolution = Some(resolution);
        self
    }

    /// Sets the response output format configuration.
    pub fn with_response_format(mut self, format: ResponseFormatConfig) -> Self {
        self.response_format = Some(format);
        self
    }

    /// Sets the translation configuration.
    pub fn with_translation_config(mut self, config: TranslationConfig) -> Self {
        self.translation_config = Some(config);
        self
    }
}
