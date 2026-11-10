use std::collections::HashMap;
use std::mem;

use itertools::Either;
use talin_provider::openai_chat::types::{
    ChatCompletionStreamOptions, ChatCompletionToolChoiceOption, ReasoningEffort, StopConfiguration,
};

use crate::llm::merge::Merge;
use crate::llm::{LlmOptions, LlmRequestOptions};
use crate::schemas::ResponseFormat;

/// Call options for the OpenAI Chat Completions API.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct ChatOptions {
    /// Whether to stream the response.
    pub stream: Option<bool>,

    /// Stream options
    pub stream_options: Option<ChatCompletionStreamOptions>,

    /// Controls which (if any) tool is called by the model.
    pub tool_choice: Option<ChatCompletionToolChoiceOption>,

    /// Whether to enable parallel function calling during tool use.
    pub parallel_tool_calls: Option<bool>,

    /// How many completion choices to generate for each input.
    pub n: Option<u8>,

    /// The reasoning effort for reasoning models.
    pub reasoning_effort: Option<ReasoningEffort>,

    /// The sampling temperature.
    pub temperature: Option<f32>,

    /// The nucleus sampling probability mass.
    pub top_p: Option<f32>,

    /// The top-k sampling cutoff (non-OpenAI extension).
    pub top_k: Option<u32>,

    /// The frequency penalty.
    pub frequency_penalty: Option<f32>,

    /// The presence penalty.
    pub presence_penalty: Option<f32>,

    /// The repetition penalty (non-OpenAI extension).
    pub repetition_penalty: Option<f32>,

    /// The maximum number of tokens to generate. Ignored when
    /// [`max_completion_tokens`](Self::max_completion_tokens) is set.
    pub max_tokens: Option<u32>,

    /// The maximum number of completion tokens to generate.
    pub max_completion_tokens: Option<u32>,

    /// Sequences that stop generation.
    pub stop: Option<StopConfiguration>,

    /// The output format the model must produce.
    ///
    /// Kept in its canonical form because the two chat providers realize it differently:
    /// [`OpenAIChat`](crate::llm::OpenAIChat) sets it as a native request field, while
    /// [`GenericChat`](crate::llm::GenericChat) emulates it via a system prompt instruction.
    pub response_format: Option<ResponseFormat>,

    /// Extra parameters for custom APIs.
    pub extra_params: Option<HashMap<String, serde_json::Value>>,
}

impl ChatOptions {
    /// Sets the [`stream`](Self::stream).
    pub fn with_stream(mut self, stream: bool) -> Self {
        self.stream = Some(stream);
        self
    }

    /// Sets the [`stream_options`](Self::stream_options).
    pub fn with_stream_options(mut self, stream_options: ChatCompletionStreamOptions) -> Self {
        self.stream_options = Some(stream_options);
        self
    }

    /// Sets the [`tool_choice`](Self::tool_choice).
    pub fn with_tool_choice(mut self, tool_choice: ChatCompletionToolChoiceOption) -> Self {
        self.tool_choice = Some(tool_choice);
        self
    }

    /// Sets the [`parallel_tool_calls`](Self::parallel_tool_calls).
    pub fn with_parallel_tool_calls(mut self, parallel_tool_calls: bool) -> Self {
        self.parallel_tool_calls = Some(parallel_tool_calls);
        self
    }

    /// Sets the [`n`](Self::n).
    pub fn with_n(mut self, n: u8) -> Self {
        self.n = Some(n);
        self
    }

    /// Sets the [`reasoning_effort`](Self::reasoning_effort).
    pub fn with_reasoning_effort(mut self, reasoning_effort: ReasoningEffort) -> Self {
        self.reasoning_effort = Some(reasoning_effort);
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
    pub fn with_repetition_penalty(mut self, repetition_penalty: f32) -> Self {
        self.repetition_penalty = Some(repetition_penalty);
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

    /// Sets the [`stop`](Self::stop) configuration.
    pub fn with_stop(mut self, stop: StopConfiguration) -> Self {
        self.stop = Some(stop);
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
}

impl From<LlmOptions> for ChatOptions {
    fn from(options: LlmOptions) -> Self {
        Self::from_options(options)
    }
}

impl LlmRequestOptions for ChatOptions {
    fn from_options(options: LlmOptions) -> Self {
        ChatOptions {
            stream: options.stream,
            stream_options: options.stream.unwrap_or_default().then_some(
                ChatCompletionStreamOptions {
                    include_usage: Some(true),
                    include_obfuscation: None,
                },
            ),
            tool_choice: options
                .tool_choice
                .map(ChatCompletionToolChoiceOption::from),
            parallel_tool_calls: options.parallel_tool_calls,
            n: options.n,
            reasoning_effort: options.reasoning_effort.map(ReasoningEffort::from),
            temperature: options.temperature,
            top_p: options.top_p,
            top_k: options.top_k,
            frequency_penalty: options.frequency_penalty,
            presence_penalty: options.presence_penalty,
            repetition_penalty: options.repetition_penalty,
            // `max_tokens` is ignored when `max_completion_tokens` is set.
            max_tokens: match options.max_completion_tokens {
                Some(_) => None,
                None => options.max_tokens,
            },
            max_completion_tokens: options.max_completion_tokens,
            stop: options.stop_words.map(StopConfiguration::StringArray),
            response_format: options.response_format,
            extra_params: options.extra_params,
        }
    }

    fn merge(self, other: Self) -> Self {
        Merge::merge(self, other)
    }
}

impl Merge for ChatOptions {
    fn merge_inplace(&mut self, other: Self) {
        self.stream = other.stream.or(self.stream);
        self.stream_options.merge_inplace(other.stream_options);
        self.tool_choice = other.tool_choice.or(self.tool_choice.take());
        self.parallel_tool_calls = other.parallel_tool_calls.or(self.parallel_tool_calls);
        self.n = other.n.or(self.n);
        self.reasoning_effort = other.reasoning_effort.or(self.reasoning_effort.take());
        self.temperature = other.temperature.or(self.temperature);
        self.top_p = other.top_p.or(self.top_p);
        self.top_k = other.top_k.or(self.top_k);
        self.frequency_penalty = other.frequency_penalty.or(self.frequency_penalty);
        self.presence_penalty = other.presence_penalty.or(self.presence_penalty);
        self.repetition_penalty = other.repetition_penalty.or(self.repetition_penalty);
        self.max_tokens = other.max_tokens.or(self.max_tokens);
        self.max_completion_tokens = other.max_completion_tokens.or(self.max_completion_tokens);
        self.stop.merge_inplace(other.stop);
        self.response_format = other.response_format.or(self.response_format.take());
        self.extra_params.merge_inplace(other.extra_params);
    }
}

impl Merge for ChatCompletionStreamOptions {
    fn merge_inplace(&mut self, other: Self) {
        self.include_usage = other.include_usage.or(self.include_usage);
        self.include_obfuscation = other.include_obfuscation.or(self.include_obfuscation);
    }
}

impl Merge for StopConfiguration {
    /// Collects the stop sequences of both configurations into a single one.
    fn merge_inplace(&mut self, other: Self) {
        let stop = mem::replace(self, StopConfiguration::StringArray(Vec::new()));
        let stop = stop_sequences(stop)
            .into_iter()
            .chain(stop_sequences(other))
            .collect::<Vec<_>>();
        *self = StopConfiguration::StringArray(stop);
    }
}

/// Returns the stop sequences held by a [`StopConfiguration`].
fn stop_sequences(stop: StopConfiguration) -> impl IntoIterator<Item = String> {
    match stop {
        StopConfiguration::String(stop) => Either::Left(std::iter::once(stop)),
        StopConfiguration::StringArray(stop) => Either::Right(stop.into_iter()),
    }
}
