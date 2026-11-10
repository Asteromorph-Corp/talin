use talin_provider::xai_responses::types::{
    ModelResponseConfiguration, ModelResponseFormat, ModelToolChoice, ReasoningConfiguration,
};

use crate::llm::merge::Merge;
use crate::llm::options::unsupported_option;
use crate::llm::{LlmOptions, LlmRequestOptions};

/// Call options for the xAI Responses API.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct ResponsesOptions {
    /// Controls which (if any) tool is called by the model.
    pub tool_choice: Option<ModelToolChoice>,

    /// The output format configuration.
    pub text: Option<ModelResponseConfiguration>,

    /// Configuration for reasoning models.
    pub reasoning: Option<ReasoningConfiguration>,

    /// The sampling temperature.
    pub temperature: Option<f32>,

    /// The nucleus sampling probability mass.
    pub top_p: Option<f32>,

    /// The maximum number of output tokens to generate.
    pub max_output_tokens: Option<u32>,

    /// Whether to enable parallel function calling during tool use.
    pub parallel_tool_calls: Option<bool>,
}

impl ResponsesOptions {
    /// Sets the [`tool_choice`](Self::tool_choice).
    pub fn with_tool_choice(mut self, tool_choice: ModelToolChoice) -> Self {
        self.tool_choice = Some(tool_choice);
        self
    }

    /// Sets the [`text`](Self::text) output format configuration.
    pub fn with_text(mut self, text: ModelResponseConfiguration) -> Self {
        self.text = Some(text);
        self
    }

    /// Sets the [`reasoning`](Self::reasoning) configuration.
    pub fn with_reasoning(mut self, reasoning: ReasoningConfiguration) -> Self {
        self.reasoning = Some(reasoning);
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

    /// Sets the [`max_output_tokens`](Self::max_output_tokens).
    pub fn with_max_output_tokens(mut self, max_output_tokens: u32) -> Self {
        self.max_output_tokens = Some(max_output_tokens);
        self
    }

    /// Sets the [`parallel_tool_calls`](Self::parallel_tool_calls).
    pub fn with_parallel_tool_calls(mut self, parallel_tool_calls: bool) -> Self {
        self.parallel_tool_calls = Some(parallel_tool_calls);
        self
    }
}

impl From<LlmOptions> for ResponsesOptions {
    fn from(options: LlmOptions) -> Self {
        Self::from_options(options)
    }
}

impl LlmRequestOptions for ResponsesOptions {
    fn from_options(options: LlmOptions) -> Self {
        unsupported_option!("xai", options.frequency_penalty);
        unsupported_option!("xai", options.presence_penalty);
        unsupported_option!("xai", options.repetition_penalty);
        unsupported_option!("xai", options.n);
        unsupported_option!("xai", options.stop_words);
        unsupported_option!("xai", options.top_k);
        unsupported_option!("xai", options.stream);

        let text = options
            .response_format
            .map(ModelResponseFormat::from)
            .map(|format| ModelResponseConfiguration::new().with_format(format));
        let reasoning = options
            .reasoning_effort
            .and_then(Option::<ReasoningConfiguration>::from);

        ResponsesOptions {
            tool_choice: options.tool_choice.map(ModelToolChoice::from),
            text,
            reasoning,
            temperature: options.temperature,
            top_p: options.top_p,
            max_output_tokens: options.max_completion_tokens.or(options.max_tokens),
            parallel_tool_calls: options.parallel_tool_calls,
        }
    }

    fn merge(self, other: Self) -> Self {
        Merge::merge(self, other)
    }
}

impl Merge for ResponsesOptions {
    fn merge_inplace(&mut self, other: Self) {
        self.tool_choice = other.tool_choice.or(self.tool_choice.take());
        self.text.merge_inplace(other.text);
        self.reasoning.merge_inplace(other.reasoning);
        self.temperature = other.temperature.or(self.temperature);
        self.top_p = other.top_p.or(self.top_p);
        self.max_output_tokens = other.max_output_tokens.or(self.max_output_tokens);
        self.parallel_tool_calls = other.parallel_tool_calls.or(self.parallel_tool_calls);
    }
}

impl Merge for ModelResponseConfiguration {
    fn merge_inplace(&mut self, other: Self) {
        self.format = other.format.or(self.format.take());
    }
}

impl Merge for ReasoningConfiguration {
    fn merge_inplace(&mut self, other: Self) {
        self.effort = other.effort.or(self.effort.take());
        self.generate_summary = other.generate_summary.or(self.generate_summary.take());
        self.summary = other.summary.or(self.summary.take());
    }
}
