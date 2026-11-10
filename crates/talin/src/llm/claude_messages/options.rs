use talin_provider::claude_messages::types::{
    CacheControlEphemeral, CacheControlEphemeralInner, CacheTtl, JSONOutputFormat, OutputEffort,
    ParallelToolUse, ThinkingConfigAdaptive, ThinkingConfigEnabled, ThinkingConfigParam,
    ToolChoice, ToolChoiceTool,
};

use crate::llm::claude_messages::conversion::map_reasoning_effort;
use crate::llm::merge::Merge;
use crate::llm::options::unsupported_option;
use crate::llm::{LlmOptions, LlmRequestOptions};

/// Call options for the Claude Messages API.
#[derive(Clone, Debug, Default)]
#[non_exhaustive]
pub struct MessagesOptions {
    /// The maximum number of tokens to generate. Defaults to 32000 when unset.
    pub max_tokens: Option<u32>,

    /// Configuration for Claude's extended thinking.
    pub thinking: Option<ThinkingConfigParam>,

    /// Controls which (if any) tool is called by the model, with parallel tool use baked in.
    pub tool_choice: Option<ToolChoice>,

    /// The effort level for the model's output.
    pub output_effort: Option<OutputEffort>,

    /// The structured output format the model must produce.
    pub output_format: Option<JSONOutputFormat>,

    /// Whether to stream the response.
    pub stream: Option<bool>,

    /// The sampling temperature.
    pub temperature: Option<f32>,

    /// The nucleus sampling probability mass.
    pub top_p: Option<f32>,

    /// The top-k sampling cutoff.
    pub top_k: Option<u32>,

    /// Sequences that stop generation.
    pub stop_sequences: Option<Vec<String>>,

    /// The cache control breakpoint applied to the request.
    pub cache_control: Option<CacheControlEphemeral>,
}

impl MessagesOptions {
    /// Sets the [`max_tokens`](Self::max_tokens).
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    /// Sets the [`thinking`](Self::thinking) configuration.
    pub fn with_thinking(mut self, thinking: ThinkingConfigParam) -> Self {
        self.thinking = Some(thinking);
        self
    }

    /// Sets the [`tool_choice`](Self::tool_choice).
    pub fn with_tool_choice(mut self, tool_choice: ToolChoice) -> Self {
        self.tool_choice = Some(tool_choice);
        self
    }

    /// Sets the [`output_effort`](Self::output_effort).
    pub fn with_output_effort(mut self, output_effort: OutputEffort) -> Self {
        self.output_effort = Some(output_effort);
        self
    }

    /// Sets the [`output_format`](Self::output_format).
    pub fn with_output_format(mut self, output_format: JSONOutputFormat) -> Self {
        self.output_format = Some(output_format);
        self
    }

    /// Sets the [`stream`](Self::stream).
    pub fn with_stream(mut self, stream: bool) -> Self {
        self.stream = Some(stream);
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

    /// Sets the [`stop_sequences`](Self::stop_sequences).
    pub fn with_stop_sequences(mut self, stop_sequences: Vec<String>) -> Self {
        self.stop_sequences = Some(stop_sequences);
        self
    }

    /// Sets the [`cache_control`](Self::cache_control).
    pub fn with_cache_control(mut self, cache_control: CacheControlEphemeral) -> Self {
        self.cache_control = Some(cache_control);
        self
    }

    /// Enables ephemeral prompt caching with the given TTL.
    ///
    /// A convenience over [`with_cache_control`](Self::with_cache_control) for the common case.
    pub fn with_cache_ttl(self, ttl: CacheTtl) -> Self {
        self.with_cache_control(CacheControlEphemeral::Ephemeral(
            CacheControlEphemeralInner::new().with_ttl(ttl),
        ))
    }
}

impl From<LlmOptions> for MessagesOptions {
    fn from(options: LlmOptions) -> Self {
        Self::from_options(options)
    }
}

impl LlmRequestOptions for MessagesOptions {
    fn from_options(options: LlmOptions) -> Self {
        unsupported_option!("claude", options.n);
        unsupported_option!("claude", options.frequency_penalty);
        unsupported_option!("claude", options.presence_penalty);
        unsupported_option!("claude", options.repetition_penalty);

        let disable_parallel_tool_use = !options.parallel_tool_calls.unwrap_or(true);
        let tool_choice = options.tool_choice.map(|choice| {
            ToolChoice::from(choice).with_disable_parallel_tool_use(disable_parallel_tool_use)
        });

        let (thinking, output_effort) = options
            .reasoning_effort
            .map_or((None, None), map_reasoning_effort);

        MessagesOptions {
            max_tokens: options.max_completion_tokens.or(options.max_tokens),
            thinking,
            tool_choice,
            output_effort,
            output_format: options
                .response_format
                .and_then(Option::<JSONOutputFormat>::from),
            stream: options.stream,
            temperature: options.temperature,
            top_p: options.top_p,
            top_k: options.top_k,
            stop_sequences: options.stop_words,
            cache_control: Some(CacheControlEphemeral::Ephemeral(
                CacheControlEphemeralInner::new().with_ttl(CacheTtl::FiveMinutes),
            )),
        }
    }

    fn merge(self, other: Self) -> Self {
        Merge::merge(self, other)
    }
}

impl Merge for MessagesOptions {
    fn merge_inplace(&mut self, other: Self) {
        self.max_tokens = other.max_tokens.or(self.max_tokens);
        self.thinking.merge_inplace(other.thinking);
        self.tool_choice.merge_inplace(other.tool_choice);
        self.output_effort = other.output_effort.or(self.output_effort.take());
        self.output_format = other.output_format.or(self.output_format.take());
        self.stream = other.stream.or(self.stream);
        self.temperature = other.temperature.or(self.temperature);
        self.top_p = other.top_p.or(self.top_p);
        self.top_k = other.top_k.or(self.top_k);
        self.stop_sequences.merge_inplace(other.stop_sequences);
        self.cache_control.merge_inplace(other.cache_control);
    }
}

impl Merge for ThinkingConfigParam {
    fn merge_inplace(&mut self, other: Self) {
        match (self, other) {
            (ThinkingConfigParam::Adaptive(config), ThinkingConfigParam::Adaptive(other)) => {
                config.merge_inplace(other);
            }
            (ThinkingConfigParam::Enabled(config), ThinkingConfigParam::Enabled(other)) => {
                config.merge_inplace(other);
            }
            (thinking, other) => *thinking = other,
        }
    }
}

impl Merge for ThinkingConfigAdaptive {
    fn merge_inplace(&mut self, other: Self) {
        self.display = other.display.or(self.display.take());
    }
}

impl Merge for ThinkingConfigEnabled {
    fn merge_inplace(&mut self, other: Self) {
        self.budget_tokens = other.budget_tokens;
        self.display = other.display.or(self.display.take());
    }
}

impl Merge for ToolChoice {
    fn merge_inplace(&mut self, other: Self) {
        match (self, other) {
            (ToolChoice::Auto(config), ToolChoice::Auto(other))
            | (ToolChoice::Any(config), ToolChoice::Any(other)) => config.merge_inplace(other),
            (ToolChoice::Tool(config), ToolChoice::Tool(other)) => config.merge_inplace(other),
            (tool_choice, other) => *tool_choice = other,
        }
    }
}

impl Merge for ParallelToolUse {
    fn merge_inplace(&mut self, other: Self) {
        self.disable_parallel_tool_use = other
            .disable_parallel_tool_use
            .or(self.disable_parallel_tool_use);
    }
}

impl Merge for ToolChoiceTool {
    fn merge_inplace(&mut self, other: Self) {
        self.name = other.name;
        self.disable_parallel_tool_use = other
            .disable_parallel_tool_use
            .or(self.disable_parallel_tool_use);
    }
}

impl Merge for CacheControlEphemeral {
    fn merge_inplace(&mut self, other: Self) {
        match (self, other) {
            (CacheControlEphemeral::Ephemeral(config), CacheControlEphemeral::Ephemeral(other)) => {
                config.merge_inplace(other)
            }
            (cache_control, other) => *cache_control = other,
        }
    }
}

impl Merge for CacheControlEphemeralInner {
    fn merge_inplace(&mut self, other: Self) {
        self.ttl = other.ttl.or(self.ttl.take());
    }
}
