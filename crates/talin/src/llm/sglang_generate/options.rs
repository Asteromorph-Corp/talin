use talin_provider::sglang_generate::types::{OneOrMany, SamplingParams};

use crate::llm::merge::Merge;
use crate::llm::options::unsupported_option;
use crate::llm::{LlmOptions, LlmRequestOptions};
use crate::schemas::ResponseFormat;

/// Call options for the SGLang Generate API.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct GenerateOptions {
    /// The maximum number of tokens to generate (`max_new_tokens`).
    pub max_new_tokens: Option<u32>,

    /// The sampling temperature.
    pub temperature: Option<f32>,

    /// The nucleus sampling probability mass.
    pub top_p: Option<f32>,

    /// The top-k sampling cutoff.
    pub top_k: Option<i32>,

    /// The frequency penalty.
    pub frequency_penalty: Option<f32>,

    /// The presence penalty.
    pub presence_penalty: Option<f32>,

    /// The repetition penalty.
    pub repetition_penalty: Option<f32>,

    /// The number of samples to generate per request.
    pub n: Option<u32>,

    /// Sequences that stop generation.
    pub stop: Option<Vec<String>>,

    /// A JSON schema the output is constrained to.
    pub json_schema: Option<String>,

    /// Whether to stream the response.
    pub stream: Option<bool>,
}

impl GenerateOptions {
    /// Sets the [`max_new_tokens`](Self::max_new_tokens).
    pub fn with_max_new_tokens(mut self, max_new_tokens: u32) -> Self {
        self.max_new_tokens = Some(max_new_tokens);
        self
    }

    /// Sets the [`temperature`](Self::temperature).
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    /// Sets the [`top_p`](Self::top_p).
    pub fn with_top_p(mut self, top_p: f32) -> Self {
        self.top_p = Some(top_p);
        self
    }

    /// Sets the [`top_k`](Self::top_k).
    pub fn with_top_k(mut self, top_k: i32) -> Self {
        self.top_k = Some(top_k);
        self
    }

    /// Sets the [`frequency_penalty`](Self::frequency_penalty).
    pub fn with_frequency_penalty(mut self, frequency_penalty: f32) -> Self {
        self.frequency_penalty = Some(frequency_penalty);
        self
    }

    /// Sets the [`presence_penalty`](Self::presence_penalty).
    pub fn with_presence_penalty(mut self, presence_penalty: f32) -> Self {
        self.presence_penalty = Some(presence_penalty);
        self
    }

    /// Sets the [`repetition_penalty`](Self::repetition_penalty).
    pub fn with_repetition_penalty(mut self, repetition_penalty: f32) -> Self {
        self.repetition_penalty = Some(repetition_penalty);
        self
    }

    /// Sets the [`n`](Self::n).
    pub fn with_n(mut self, n: u32) -> Self {
        self.n = Some(n);
        self
    }

    /// Sets the [`stop`](Self::stop) sequences.
    pub fn with_stop(mut self, stop: Vec<String>) -> Self {
        self.stop = Some(stop);
        self
    }

    /// Sets the [`json_schema`](Self::json_schema) constraint.
    pub fn with_json_schema(mut self, json_schema: String) -> Self {
        self.json_schema = Some(json_schema);
        self
    }

    /// Sets the [`stream`](Self::stream).
    pub fn with_stream(mut self, stream: bool) -> Self {
        self.stream = Some(stream);
        self
    }

    /// Assembles the parameters for a request, falling back to the defaults for any unset field.
    pub fn sampling_params(&self) -> SamplingParams {
        let mut params = SamplingParams::default();
        if let Some(max_new_tokens) = self.max_new_tokens {
            params.max_new_tokens = max_new_tokens;
        }
        if let Some(temperature) = self.temperature {
            params.temperature = temperature;
        }
        if let Some(top_p) = self.top_p {
            params.top_p = top_p;
        }
        if let Some(top_k) = self.top_k {
            params.top_k = top_k;
        }
        if let Some(frequency_penalty) = self.frequency_penalty {
            params.frequency_penalty = frequency_penalty;
        }
        if let Some(presence_penalty) = self.presence_penalty {
            params.presence_penalty = presence_penalty;
        }
        if let Some(repetition_penalty) = self.repetition_penalty {
            params.repetition_penalty = repetition_penalty;
        }
        if let Some(n) = self.n {
            params.n = n;
        }
        if let Some(stop) = &self.stop {
            params.stop = Some(OneOrMany::Many(stop.clone()));
        }
        params.json_schema = self.json_schema.clone();
        params
    }
}

impl From<LlmOptions> for GenerateOptions {
    fn from(options: LlmOptions) -> Self {
        Self::from_options(options)
    }
}

impl LlmRequestOptions for GenerateOptions {
    fn from_options(mut options: LlmOptions) -> Self {
        unsupported_option!("sglang", options.tool_choice = None);
        unsupported_option!("sglang", options.parallel_tool_calls = None);
        unsupported_option!("sglang", options.reasoning_effort = None);
        unsupported_option!("sglang", options.extra_params = None);

        // SGLang accepts a JSON schema constraint, but not the legacy free-form JSON object mode.
        let json_schema = options.response_format.and_then(|format| match format {
            ResponseFormat::Text => None,
            ResponseFormat::JsonSchema(schema) => serde_json::to_string(&schema.schema).ok(),
            ResponseFormat::JsonObject => {
                log::warn!(
                    "`LlmOptions::response_format` JSON object mode is not supported by sglang; \
                     use a JSON schema instead"
                );
                None
            }
        });

        GenerateOptions {
            // `max_tokens` is ignored when `max_completion_tokens` is set.
            max_new_tokens: options.max_completion_tokens.or(options.max_tokens),
            temperature: options.temperature,
            top_p: options.top_p,
            top_k: options.top_k.map(|k| k as i32),
            frequency_penalty: options.frequency_penalty,
            presence_penalty: options.presence_penalty,
            repetition_penalty: options.repetition_penalty,
            n: options.n.map(u32::from),
            stop: options.stop_words,
            json_schema,
            stream: options.stream,
        }
    }

    fn merge(self, other: Self) -> Self {
        Merge::merge(self, other)
    }
}

impl Merge for GenerateOptions {
    fn merge_inplace(&mut self, other: Self) {
        self.max_new_tokens = other.max_new_tokens.or(self.max_new_tokens);
        self.temperature = other.temperature.or(self.temperature);
        self.top_p = other.top_p.or(self.top_p);
        self.top_k = other.top_k.or(self.top_k);
        self.frequency_penalty = other.frequency_penalty.or(self.frequency_penalty);
        self.presence_penalty = other.presence_penalty.or(self.presence_penalty);
        self.repetition_penalty = other.repetition_penalty.or(self.repetition_penalty);
        self.n = other.n.or(self.n);
        self.stop.merge_inplace(other.stop);
        self.json_schema = other.json_schema.or(self.json_schema.take());
        self.stream = other.stream.or(self.stream);
    }
}
