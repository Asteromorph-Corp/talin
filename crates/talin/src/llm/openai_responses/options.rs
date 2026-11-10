use talin_provider::openai_responses::types::{
    Reasoning, ResponseTextParam, TextResponseFormatConfiguration, ToolChoiceParam,
};

use crate::llm::merge::Merge;
use crate::llm::options::unsupported_option;
use crate::llm::{LlmOptions, LlmRequestOptions};

/// Call options for the OpenAI Responses API.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct ResponsesOptions {
    /// The maximum number of output tokens to generate.
    pub max_output_tokens: Option<u32>,

    /// Whether to enable parallel function calling during tool use.
    pub parallel_tool_calls: Option<bool>,

    /// Configuration for reasoning models.
    pub reasoning: Option<Reasoning>,

    /// The sampling temperature.
    pub temperature: Option<f32>,

    /// The output format configuration.
    pub text: Option<ResponseTextParam>,

    /// Controls which (if any) tool is called by the model.
    pub tool_choice: Option<ToolChoiceParam>,

    /// The nucleus sampling probability mass.
    pub top_p: Option<f32>,
}

impl ResponsesOptions {
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

    /// Sets the [`reasoning`](Self::reasoning) configuration.
    pub fn with_reasoning(mut self, reasoning: Reasoning) -> Self {
        self.reasoning = Some(reasoning);
        self
    }

    /// Sets the [`temperature`](Self::temperature).
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    /// Sets the [`text`](Self::text) output format configuration.
    pub fn with_text(mut self, text: ResponseTextParam) -> Self {
        self.text = Some(text);
        self
    }

    /// Sets the [`tool_choice`](Self::tool_choice).
    pub fn with_tool_choice(mut self, tool_choice: ToolChoiceParam) -> Self {
        self.tool_choice = Some(tool_choice);
        self
    }

    /// Sets the [`top_p`](Self::top_p).
    pub fn with_top_p(mut self, top_p: f32) -> Self {
        self.top_p = Some(top_p);
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
        unsupported_option!("responses", options.stream);
        unsupported_option!("responses", options.n);
        unsupported_option!("responses", options.top_k);
        unsupported_option!("responses", options.frequency_penalty);
        unsupported_option!("responses", options.presence_penalty);
        unsupported_option!("responses", options.repetition_penalty);
        unsupported_option!("responses", options.stop_words);

        ResponsesOptions {
            max_output_tokens: options.max_completion_tokens.or(options.max_tokens),
            parallel_tool_calls: options.parallel_tool_calls,
            reasoning: options.reasoning_effort.map(Reasoning::from),
            temperature: options.temperature,
            text: options.response_format.map(ResponseTextParam::from),
            tool_choice: options.tool_choice.map(ToolChoiceParam::from),
            top_p: options.top_p,
        }
    }

    fn merge(self, other: Self) -> Self {
        Merge::merge(self, other)
    }
}

impl Merge for ResponsesOptions {
    fn merge_inplace(&mut self, other: Self) {
        self.max_output_tokens = other.max_output_tokens.or(self.max_output_tokens);
        self.parallel_tool_calls = other.parallel_tool_calls.or(self.parallel_tool_calls);
        self.reasoning.merge_inplace(other.reasoning);
        self.temperature = other.temperature.or(self.temperature);
        self.text.merge_inplace(other.text);
        self.tool_choice = other.tool_choice.or(self.tool_choice.take());
        self.top_p = other.top_p.or(self.top_p);
    }
}

impl Merge for Reasoning {
    fn merge_inplace(&mut self, other: Self) {
        self.effort = other.effort.or(self.effort.take());
        self.summary = other.summary.or(self.summary.take());
    }
}

impl Merge for ResponseTextParam {
    fn merge_inplace(&mut self, other: Self) {
        // The format has no unset state; the default text format falls back to `self`.
        if !matches!(other.format, TextResponseFormatConfiguration::Text) {
            self.format = other.format;
        }
        self.verbosity = other.verbosity.or(self.verbosity.take());
    }
}
