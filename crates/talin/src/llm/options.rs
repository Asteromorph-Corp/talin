use std::collections::HashMap;

use crate::llm::merge::Merge;
use crate::schemas::{ReasoningEffort, ResponseFormat, ToolChoice};

/// Options for LLM calls.
///
/// This option attempts to encapsulate a maximum common set of parameters supported by various LLM.
/// As such, not all options are supported by all providers. Unsupported options will be ignored
/// with a warning.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct LlmOptions {
    /// If set to true, the model response data will be streamed to the client as it is generated
    /// using server-sent events.
    pub stream: Option<bool>,

    /// Controls which (if any) tool is called by the model.
    pub tool_choice: Option<ToolChoice>,

    /// Whether to enable parallel function calling during tool use.
    pub parallel_tool_calls: Option<bool>,

    /// How many chat completion choices to generate for each input message.
    ///
    /// Not compatible with the responses api.
    pub n: Option<u8>,

    /// Constrains effort on reasoning for reasoning models.
    ///
    /// Only supported for certain OpenAI models.
    pub reasoning_effort: Option<ReasoningEffort>,

    /// The sampling temperature. Higher values like 0.8 will make the output more random, while
    /// lower values like 0.2 will make it more focused and deterministic.
    pub temperature: Option<f32>,

    /// The parameter for the top-p sampling method. The model considers the results of the tokens
    /// with `top_p` probability mass. Lower values will make the output more deterministic.
    pub top_p: Option<f32>,

    /// The parameter for the top-k sampling method. The model considers the top `top_k` tokens
    /// with the highest probability. Lower values will make the output more deterministic.
    ///
    /// Not part of the OpenAI chat completions API or the responses API, but used by some other
    /// providers.
    pub top_k: Option<u32>,

    /// Number between -2.0 and 2.0. Positive values penalize new tokens based on their existing
    /// frequency in the text so far, decreasing the model's likelihood to repeat the same line
    /// verbatim.
    pub frequency_penalty: Option<f32>,

    /// Number between -2.0 and 2.0. Positive values penalize new tokens based on whether they
    /// appear in the text so far, increasing the model's likelihood to talk about new topics.
    pub presence_penalty: Option<f32>,

    /// Number between 0.0 and 2.0. Positive values discourage the model from repeating the same
    /// line verbatim.
    ///
    /// Not part of the OpenAI chat completions API or the responses API, but used by some other
    /// providers.
    pub repetition_penalty: Option<f32>,

    /// The maximum number of tokens to generate in the completion.
    ///
    /// OpenAI API deprecated `max_tokens` in favor of `max_completion_tokens`, but other providers
    /// still use `max_tokens`.
    ///
    /// This value is ignored if [`max_completion_tokens`](Self::max_completion_tokens) is set.
    pub max_tokens: Option<u32>,

    /// The maximum number of tokens to generate in the completion.
    ///
    /// This field takes precedence over [`max_tokens`](Self::max_tokens).
    pub max_completion_tokens: Option<u32>,

    /// Up to 4 sequences where the API will stop generating further tokens. The returned text will
    /// not contain the stop sequence.
    ///
    /// Not compatible with the responses api.
    pub stop_words: Option<Vec<String>>,

    /// An object specifying the format that the model must output.
    pub response_format: Option<ResponseFormat>,

    /// Extra parameters for custom api.
    pub extra_params: Option<HashMap<String, serde_json::Value>>,
}

impl Default for LlmOptions {
    fn default() -> Self {
        LlmOptions::new()
    }
}

impl LlmOptions {
    /// Constructs a new [`LlmOptions`].
    pub fn new() -> Self {
        LlmOptions {
            stream: None,
            tool_choice: None,
            parallel_tool_calls: None,
            n: None,
            reasoning_effort: None,
            temperature: None,
            top_p: None,
            top_k: None,
            frequency_penalty: None,
            presence_penalty: None,
            repetition_penalty: None,
            max_tokens: None,
            max_completion_tokens: None,
            stop_words: None,
            response_format: None,
            extra_params: None,
        }
    }

    /// Sets the [`stream`](Self::stream).
    pub fn with_stream(mut self, stream: bool) -> Self {
        self.stream = Some(stream);
        self
    }

    /// Sets the [`tool_choice`](Self::tool_choice).
    pub fn with_tool_choice(mut self, tool_choice: ToolChoice) -> Self {
        self.tool_choice = Some(tool_choice);
        self
    }

    /// Sets the [`parallel_tool_calls`](Self::parallel_tool_calls).
    pub fn with_parallel_tool_calls(mut self, parallel_tool_calls: bool) -> Self {
        self.parallel_tool_calls = Some(parallel_tool_calls);
        self
    }

    /// Sets the [`n`](Self::n).
    ///
    /// Not compatible with the responses api.
    pub fn with_n(mut self, n: u8) -> Self {
        self.n = Some(n);
        self
    }

    /// Sets the [`reasoning_effort`](Self::reasoning_effort).
    ///
    /// Only supported for certain OpenAI models.
    pub fn with_reasoning_effort(mut self, reasoning_effort: ReasoningEffort) -> Self {
        self.reasoning_effort = Some(reasoning_effort);
        self
    }

    /// Sets the [`max_tokens`](Self::max_tokens).
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    /// Sets the [`max_completion_tokens`](Self::max_completion_tokens).
    pub fn with_max_completion_tokens(mut self, max_completion_tokens: u32) -> Self {
        self.max_completion_tokens = Some(max_completion_tokens);
        self
    }

    /// Sets the [`top_p`](Self::top_p).
    pub fn with_top_p(mut self, top_p: f32) -> Self {
        self.top_p = Some(top_p);
        self
    }

    /// Sets the [`top_k`](Self::top_k).
    ///
    /// Not part of the OpenAI chat completions API or the responses API, but used by some other
    /// providers.
    pub fn with_top_k(mut self, top_k: u32) -> Self {
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
    ///
    /// Not part of the OpenAI chat completions API or the responses API, but used by some other
    /// providers.
    pub fn with_repetition_penalty(mut self, repetition_penalty: f32) -> Self {
        self.repetition_penalty = Some(repetition_penalty);
        self
    }

    /// Sets the [`temperature`](Self::temperature).
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    /// Sets the [`stop_words`](Self::stop_words).
    ///
    /// Not compatible with the responses api.
    pub fn with_stop_words(mut self, stop_words: Vec<String>) -> Self {
        self.stop_words = Some(stop_words);
        self
    }

    /// Sets the [`response_format`](Self::response_format).
    pub fn with_response_format(mut self, response_format: ResponseFormat) -> Self {
        self.response_format = Some(response_format);
        self
    }

    /// Sets the [`extra_params`](Self::extra_params).
    pub fn with_extra_params(mut self, extra_params: HashMap<String, serde_json::Value>) -> Self {
        self.extra_params = Some(extra_params);
        self
    }

    /// Merges two [`LlmOptions`] into one.
    pub fn merge(self, other: Self) -> Self {
        Merge::merge(self, other)
    }

    /// Merges another [`LlmOptions`] into this one.
    ///
    /// For each field, if the incoming option is `Some`, it will replace the existing value.
    /// Otherwise, the existing value is retained.
    pub fn merge_inplace(&mut self, other: LlmOptions) {
        Merge::merge_inplace(self, other);
    }
}

impl Merge for LlmOptions {
    fn merge_inplace(&mut self, other: Self) {
        self.stream = other.stream.or(self.stream);

        self.tool_choice = other.tool_choice.or(self.tool_choice.take());
        self.parallel_tool_calls = other.parallel_tool_calls.or(self.parallel_tool_calls);

        self.n = other.n.or(self.n);

        self.reasoning_effort = other.reasoning_effort.or(self.reasoning_effort);

        self.temperature = other.temperature.or(self.temperature);

        self.top_p = other.top_p.or(self.top_p);
        self.top_k = other.top_k.or(self.top_k);

        self.frequency_penalty = other.frequency_penalty.or(self.frequency_penalty);
        self.presence_penalty = other.presence_penalty.or(self.presence_penalty);
        self.repetition_penalty = other.repetition_penalty.or(self.repetition_penalty);

        self.max_tokens = other.max_tokens.or(self.max_tokens);
        self.max_completion_tokens = other.max_completion_tokens.or(self.max_completion_tokens);

        self.stop_words.merge_inplace(other.stop_words);
        self.response_format = other.response_format.or(self.response_format.take());
        self.extra_params.merge_inplace(other.extra_params);
    }
}

/// Logs a warning if an unsupported option is set.
macro_rules! unsupported_option {
    ($provider:literal, $options:ident. $option:ident) => {
        if $options.$option.is_some() {
            ::log::warn!(
                "`LlmOptions::{}` does not work with {}",
                stringify!($option),
                $provider
            )
        }
    };
    ($provider:literal, $options:ident. $option:ident = None) => {
        if $options.$option.is_some() {
            ::log::warn!(
                "`LlmOptions::{}` is not supported by {}, and will be ignored",
                stringify!($option),
                $provider
            );
            $options.$option = None;
        }
    };
}
pub(super) use unsupported_option;
