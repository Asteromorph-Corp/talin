use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::utils::{string_enum, tagged_enum};
use crate::xai_responses::types::{ModelInput, ModelTool, ModelToolChoice, ReasoningConfiguration};

/// Request body for the [Create Response](https://docs.x.ai/docs/api-reference#create-new-response) endpoint.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[non_exhaustive]
pub struct ModelRequest {
    /// The input passed to the model. Can be text, image or file.
    pub input: ModelInput,

    /// Model name for the model to use.
    #[cfg_attr(test, schema(nullable = false))]
    pub model: Option<String>,

    /// Whether to process the response asynchronously in the background.
    pub background: Option<bool>,

    /// Optional context-management directives (e.g. compaction).
    /// Parsed but not yet executed.
    pub context_management: Option<Vec<serde_json::Value>>,

    /// What additional output data to include in the response.
    /// Currently the only supported value is `"reasoning.encrypted_content"`.
    pub include: Option<Vec<String>>,

    /// Per-category instructions for model output.
    /// A request whose guidelines cannot be applied, for example on a multi-agent model or with
    /// `stop`, fails with 400.
    pub guidelines: Option<Vec<Guideline>>,

    /// An alternate way to specify the system prompt.
    /// Note that this cannot be used alongside `previous_response_id`, where the system prompt of
    /// the previous message will be used.
    pub instructions: Option<String>,

    /// Whether to return log probabilities of the output tokens or not.
    pub logprobs: Option<bool>,

    /// Max number of tokens that can be generated in a response.
    /// This includes both output and reasoning tokens.
    #[cfg_attr(test, schema(value_type = Option<i32>))]
    pub max_output_tokens: Option<u32>,

    /// Maximum number of agentic tool calling turns allowed for this request.
    /// If not set, defaults to the server's global cap.
    #[cfg_attr(test, schema(value_type = Option<i32>))]
    pub max_turns: Option<u32>,

    /// Not supported. Only maintained for compatibility reasons.
    pub metadata: Option<serde_json::Value>,

    /// Min-p sampling: tokens whose probability is below `min_p` times the probability of the
    /// most likely token are excluded from sampling.
    #[cfg_attr(test, schema(minimum = 0, maximum = 1))]
    pub min_p: Option<f32>,

    /// Whether to allow the model to run parallel tool calls.
    /// Defaults to true.
    pub parallel_tool_calls: Option<bool>,

    /// The ID of the previous response from the model.
    pub previous_response_id: Option<String>,

    /// Plumbed to `x-grok-conv-id` for Open Responses compatibility, used for routing.
    pub prompt_cache_key: Option<String>,

    /// Reasoning configuration. Only for reasoning models.
    pub reasoning: Option<ReasoningConfiguration>,

    /// An alternative to the [`reasoning`](Self::reasoning) configuration.
    /// A non-standard field meant to ease user experience; only read when `reasoning` is unset.
    pub reasoning_effort: Option<String>,

    /// A stable identifier used to help detect users of your application that may be violating
    /// xAI's usage policies.
    pub safety_identifier: Option<String>,

    /// Set the parameters to be used for searched data.
    /// Takes precedence over `"web_search_preview"` tool if specified in the tools.
    pub search_parameters: Option<SearchParameters>,

    /// Processing tier for a request. Determines scheduling priority and billing.
    pub service_tier: Option<ServiceTier>,

    /// Whether to store the input message(s) and model response for later retrieval.
    /// Defaults to true.
    pub store: Option<bool>,

    /// If set, partial message deltas will be sent.
    /// Tokens will be sent as data-only server-sent events as they become available, with the
    /// stream terminated by a `data: [DONE]` message.
    pub stream: Option<bool>,

    /// What sampling temperature to use, between 0 and 2.
    /// Higher values like 0.8 will make the output more random, while lower values like 0.2 will
    /// make it more focused and deterministic.
    #[cfg_attr(test, schema(minimum = 0, maximum = 2))]
    pub temperature: Option<f32>,

    /// Settings for customizing a text response from the model.
    pub text: Option<ModelResponseConfiguration>,

    /// Controls which (if any) tool is called by the model.
    pub tool_choice: Option<ModelToolChoice>,

    /// Top-k sampling: only the `top_k` most probable tokens are considered at each sampling step.
    /// Disabled when unset.
    #[cfg_attr(test, schema(minimum = 1))]
    pub top_k: Option<u32>,

    /// A list of tools the model may call in JSON-schema.
    /// Currently, only functions and web search are supported as tools.
    /// A max of 350 tools are supported.
    #[cfg_attr(test, schema(max_items = 128))]
    pub tools: Option<Vec<ModelTool>>,

    /// An integer between 0 and 8 specifying the number of most likely tokens to return at each
    /// token position, each with an associated log probability.
    /// logprobs must be set to true if this parameter is used.
    #[cfg_attr(test, schema(maximum = 8))]
    pub top_logprobs: Option<u8>,

    /// An alternative to sampling with [`temperature`](Self::temperature), called nucleus
    /// sampling.
    #[cfg_attr(test, schema(exclusive_minimum = 0, maximum = 1))]
    pub top_p: Option<f32>,

    /// The truncation strategy to use for the model response.
    /// Not supported. Only maintained for compatibility reasons.
    #[cfg_attr(test, schema(value_type = Option<String>))]
    pub truncation: Option<Truncation>,

    /// A unique identifier representing your end-user, which can help xAI to monitor and detect
    /// abuse.
    pub user: Option<String>,
}

/// A per-category instruction for model output.
///
/// Each category may appear at most once, and every category accepts `decline`.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Guideline {
    /// Discriminator. Must be `policy`.
    #[serde(rename = "type")]
    pub kind: String,

    /// Content category the guideline applies to.
    pub category: GuidelineCategory,

    /// How the model should handle the category.
    pub behavior: GuidelineBehavior,
}

impl Guideline {
    /// Creates a `policy` guideline for `category` with the given `behavior`.
    pub fn policy(category: GuidelineCategory, behavior: GuidelineBehavior) -> Self {
        Self {
            kind: "policy".to_owned(),
            category,
            behavior,
        }
    }
}

string_enum! {
    /// Content category a guideline applies to.
    #[serde(rename_all = "snake_case")]
    pub enum GuidelineCategory {
        /// Not-safe-for-work content.
        Nsfw,
        /// Violent content.
        Violence,
        /// Profane language.
        Profanity,
        /// Cyber security.
        Cyber,
    }
}

string_enum! {
    /// How the model should handle a [`GuidelineCategory`].
    #[serde(rename_all = "snake_case")]
    pub enum GuidelineBehavior {
        /// Decline to produce content in the category.
        Decline,
        /// Allow content in the category.
        Allow,
    }
}

string_enum! {
    /// Processing tier for a request. Determines scheduling priority and billing.
    #[serde(rename_all = "snake_case")]
    pub enum ServiceTier {
        /// Default processing tier.
        Default,
        /// Priority processing tier.
        Priority,
        /// Fast processing tier.
        Fast,
    }
}

/// The truncation strategy to use for the model response.
///
/// Not supported by xAI; only maintained for OpenAI API compatibility.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum Truncation {
    /// Automatic truncation.
    Auto,
    /// Truncation disabled.
    Disabled,

    /// A value not represented by this enum, kept as sent for forward compatibility.
    #[serde(untagged)]
    Unknown(String),
}

/// Parameters to control realtime data.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[non_exhaustive]
pub struct SearchParameters {
    /// Date from which to consider the results in ISO-8601 YYYY-MM-DD.
    #[cfg_attr(test, schema(format = Date))]
    pub from_date: Option<String>,

    /// Maximum number of search results to use.
    #[cfg_attr(test, schema(minimum = 1, maximum = 30))]
    pub max_search_results: Option<u32>,

    /// Choose the mode to query realtime data.
    #[cfg_attr(test, schema(value_type = Option<String>))]
    pub mode: Option<SearchMode>,

    /// Whether to return citations in the response or not.
    pub return_citations: Option<bool>,

    /// List of sources to search in.
    /// If no sources specified, the model will look over the web and X by default.
    pub sources: Option<Vec<SearchSource>>,

    /// Date up to which to consider the results in ISO-8601 YYYY-MM-DD.
    #[cfg_attr(test, schema(format = Date))]
    pub to_date: Option<String>,
}

/// Mode used to query realtime data.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum SearchMode {
    /// The model never searches and no external data is considered.
    Off,
    /// The model searches every source for relevant data.
    On,
    /// The model chooses whether to search for data and where to search for it.
    Auto,

    /// A value not represented by this enum, kept as sent for forward compatibility.
    #[serde(untagged)]
    Unknown(String),
}

tagged_enum! {
    /// A source to search in for realtime data.
    #[derive(Serialize, Deserialize, Debug, Clone)]
    #[serde(tag = "type", rename_all = "snake_case")]
    pub enum SearchSource {
        /// Search X.
        X(XSearchSource),
        /// Search the web.
        Web(WebSearchConfig),
        /// Search news.
        News(NewsSearchSource),
        /// Search RSS feeds.
        Rss(RssSearchSource),
    }
}

/// Configuration for X search source.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[non_exhaustive]
pub struct XSearchSource {
    /// List of X handles to exclude from the search results.
    /// X posts returned will not include any posts authored by these handles.
    pub excluded_x_handles: Option<Vec<String>>,

    /// X Handles of the users from whom to consider the posts.
    /// Only available if mode is `"auto"`, `"on"` or `"x"`.
    pub included_x_handles: Option<Vec<String>>,

    /// X Handles of the users from whom to consider the posts.
    #[deprecated(note = "use `included_x_handles` instead")]
    pub x_handles: Option<Vec<String>>,

    /// The minimum favorite count of the X posts to consider.
    #[cfg_attr(test, schema(value_type = Option<i32>))]
    pub post_favorite_count: Option<u32>,

    /// The minimum view count of the X posts to consider.
    #[cfg_attr(test, schema(value_type = Option<i32>))]
    pub post_view_count: Option<u32>,
}

/// Configuration for Web search source.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[non_exhaustive]
pub struct WebSearchConfig {
    /// List of website to allow in the search results.
    /// This parameter act as a whitelist where only those websites can be selected.
    /// A maximum of 5 websites can be selected.
    #[cfg_attr(test, schema(max_items = 5))]
    pub allowed_websites: Option<Vec<String>>,

    /// ISO alpha-2 code of the country.
    /// If the country is set, only data coming from this country will be considered.
    #[cfg_attr(test, schema(format = "iso3166-1-alpha-2"))]
    pub country: Option<String>,

    /// List of website to exclude from the search results without protocol specification or
    /// subdomains.
    /// A maximum of 5 websites can be excluded.
    #[cfg_attr(test, schema(max_items = 5))]
    pub excluded_websites: Option<Vec<String>>,

    /// If set to true, mature content won't be considered during the search.
    /// Default to `true`.
    pub safe_search: Option<bool>,
}

/// Configuration for News search source.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[non_exhaustive]
pub struct NewsSearchSource {
    /// ISO alpha-2 code of the country.
    /// If the country is set, only data coming from this country will be considered.
    #[cfg_attr(test, schema(format = "iso3166-1-alpha-2"))]
    pub country: Option<String>,

    /// List of website to exclude from the search results without protocol specification or
    /// subdomains.
    /// A maximum of 5 websites can be excluded.
    #[cfg_attr(test, schema(max_items = 5))]
    pub excluded_websites: Option<Vec<String>>,

    /// If set to true, mature content won't be considered during the search.
    /// Default to `true`.
    pub safe_search: Option<bool>,
}

/// Configuration for RSS search source.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct RssSearchSource {
    /// Links of the RSS feeds. Exactly one link is supported.
    #[cfg_attr(test, schema(min_items = 1, max_items = 1))]
    pub links: Vec<String>,
}

/// Settings for customizing a text response from the model.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[non_exhaustive]
pub struct ModelResponseConfiguration {
    /// An object specifying the format that the model must output.
    pub format: Option<ModelResponseFormat>,
}

/// The format that the model must output.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
#[non_exhaustive]
pub enum ModelResponseFormat {
    /// Text response format.
    Text,
    /// JSON object response format.
    /// Used for backward compatibility. Prefer using [`JsonSchema`](Self::JsonSchema) instead of
    /// this.
    JsonObject,
    /// JSON schema response format.
    JsonSchema {
        /// The JSON schema the model response must follow.
        schema: schemars::Schema,

        /// Only included for compatibility.
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,

        /// Only included for compatibility.
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,

        /// Only included for compatibility.
        #[serde(skip_serializing_if = "Option::is_none")]
        strict: Option<bool>,
    },

    /// A value not represented by this enum, kept as sent for forward compatibility.
    #[serde(untagged)]
    Unknown(serde_json::Value),
}

// The closed form of `ModelResponseFormat`, which is what its description is generated from.
/// The format that the model must output.
#[cfg(test)]
#[allow(dead_code)]
#[derive(utoipa::ToSchema)]
#[schema(as = ModelResponseFormat)]
#[serde(tag = "type", rename_all = "snake_case")]
enum KnownModelResponseFormat {
    /// Text response format.
    Text,
    /// JSON object response format.
    /// Used for backward compatibility. Prefer using [`JsonSchema`](Self::JsonSchema) instead of
    /// this.
    JsonObject,
    /// JSON schema response format.
    JsonSchema {
        /// The JSON schema the model response must follow.
        #[schema(value_type = serde_json::Value)]
        schema: schemars::Schema,

        /// Only included for compatibility.
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,

        /// Only included for compatibility.
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,

        /// Only included for compatibility.
        #[serde(skip_serializing_if = "Option::is_none")]
        strict: Option<bool>,
    },
}

#[cfg(test)]
crate::utils::delegate_schema!(ModelResponseFormat => KnownModelResponseFormat);

impl ModelRequest {
    /// Creates an empty [`ModelRequest`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the input passed to the model.
    pub fn with_input(mut self, input: ModelInput) -> Self {
        self.input = input;
        self
    }

    /// Sets the model name to use.
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    /// Sets whether to process the response asynchronously in the background.
    pub fn with_background(mut self, background: bool) -> Self {
        self.background = Some(background);
        self
    }

    /// Sets the context-management directives.
    pub fn with_context_management(mut self, context_management: Vec<serde_json::Value>) -> Self {
        self.context_management = Some(context_management);
        self
    }

    /// Sets what additional output data to include in the response.
    pub fn with_include(mut self, include: Vec<String>) -> Self {
        self.include = Some(include);
        self
    }

    /// Sets the system prompt instructions.
    pub fn with_instructions(mut self, instructions: impl Into<String>) -> Self {
        self.instructions = Some(instructions.into());
        self
    }

    /// Sets whether to return log probabilities of the output tokens.
    pub fn with_logprobs(mut self, logprobs: bool) -> Self {
        self.logprobs = Some(logprobs);
        self
    }

    /// Sets the maximum number of tokens that can be generated in a response.
    pub fn with_max_output_tokens(mut self, max_output_tokens: u32) -> Self {
        self.max_output_tokens = Some(max_output_tokens);
        self
    }

    /// Sets the maximum number of agentic tool calling turns allowed for this request.
    pub fn with_max_turns(mut self, max_turns: u32) -> Self {
        self.max_turns = Some(max_turns);
        self
    }

    /// Sets the `metadata` field (only maintained for compatibility reasons).
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Sets the min-p sampling threshold.
    pub fn with_min_p(mut self, min_p: f32) -> Self {
        self.min_p = Some(min_p);
        self
    }

    /// Sets whether to allow the model to run parallel tool calls.
    pub fn with_parallel_tool_calls(mut self, parallel_tool_calls: bool) -> Self {
        self.parallel_tool_calls = Some(parallel_tool_calls);
        self
    }

    /// Sets the ID of the previous response from the model.
    pub fn with_previous_response_id(mut self, previous_response_id: impl Into<String>) -> Self {
        self.previous_response_id = Some(previous_response_id.into());
        self
    }

    /// Sets the prompt cache key used for routing.
    pub fn with_prompt_cache_key(mut self, prompt_cache_key: impl Into<String>) -> Self {
        self.prompt_cache_key = Some(prompt_cache_key.into());
        self
    }

    /// Sets the reasoning configuration.
    pub fn with_reasoning(mut self, reasoning: ReasoningConfiguration) -> Self {
        self.reasoning = Some(reasoning);
        self
    }

    /// Sets the reasoning effort, an alternative to the reasoning configuration.
    pub fn with_reasoning_effort(mut self, reasoning_effort: impl Into<String>) -> Self {
        self.reasoning_effort = Some(reasoning_effort.into());
        self
    }

    /// Sets the parameters to be used for searched data.
    pub fn with_search_parameters(mut self, search_parameters: SearchParameters) -> Self {
        self.search_parameters = Some(search_parameters);
        self
    }

    /// Sets the processing tier for the request.
    pub fn with_service_tier(mut self, service_tier: ServiceTier) -> Self {
        self.service_tier = Some(service_tier);
        self
    }

    /// Sets whether to store the input message(s) and model response for later retrieval.
    pub fn with_store(mut self, store: bool) -> Self {
        self.store = Some(store);
        self
    }

    /// Sets whether partial message deltas will be streamed.
    pub fn with_stream(mut self, stream: bool) -> Self {
        self.stream = Some(stream);
        self
    }

    /// Sets the sampling temperature to use.
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    /// Sets the settings for customizing a text response from the model.
    pub fn with_text(mut self, text: ModelResponseConfiguration) -> Self {
        self.text = Some(text);
        self
    }

    /// Sets which (if any) tool is called by the model.
    pub fn with_tool_choice(mut self, tool_choice: ModelToolChoice) -> Self {
        self.tool_choice = Some(tool_choice);
        self
    }

    /// Sets the top-k sampling parameter.
    pub fn with_top_k(mut self, top_k: u32) -> Self {
        self.top_k = Some(top_k);
        self
    }

    /// Sets the list of tools the model may call.
    pub fn with_tools(mut self, tools: Vec<ModelTool>) -> Self {
        self.tools = Some(tools);
        self
    }

    /// Sets the number of most likely tokens to return at each token position.
    pub fn with_top_logprobs(mut self, top_logprobs: u8) -> Self {
        self.top_logprobs = Some(top_logprobs);
        self
    }

    /// Sets the nucleus sampling parameter.
    pub fn with_top_p(mut self, top_p: f32) -> Self {
        self.top_p = Some(top_p);
        self
    }

    /// Sets the truncation strategy to use for the model response.
    pub fn with_truncation(mut self, truncation: Truncation) -> Self {
        self.truncation = Some(truncation);
        self
    }

    /// Sets the end-user identifier.
    pub fn with_user(mut self, user: impl Into<String>) -> Self {
        self.user = Some(user.into());
        self
    }
}

impl SearchParameters {
    /// Creates an empty [`SearchParameters`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the date from which to consider the results (ISO-8601 YYYY-MM-DD).
    pub fn with_from_date(mut self, from_date: impl Into<String>) -> Self {
        self.from_date = Some(from_date.into());
        self
    }

    /// Sets the maximum number of search results to use.
    pub fn with_max_search_results(mut self, max_search_results: u32) -> Self {
        self.max_search_results = Some(max_search_results);
        self
    }

    /// Sets the mode to query realtime data.
    pub fn with_mode(mut self, mode: SearchMode) -> Self {
        self.mode = Some(mode);
        self
    }

    /// Sets whether to return citations in the response.
    pub fn with_return_citations(mut self, return_citations: bool) -> Self {
        self.return_citations = Some(return_citations);
        self
    }

    /// Sets the list of sources to search in.
    pub fn with_sources(mut self, sources: Vec<SearchSource>) -> Self {
        self.sources = Some(sources);
        self
    }

    /// Sets the date up to which to consider the results (ISO-8601 YYYY-MM-DD).
    pub fn with_to_date(mut self, to_date: impl Into<String>) -> Self {
        self.to_date = Some(to_date.into());
        self
    }
}

impl XSearchSource {
    /// Creates an empty [`XSearchSource`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the list of X handles to exclude from the search results.
    pub fn with_excluded_x_handles(mut self, excluded_x_handles: Vec<String>) -> Self {
        self.excluded_x_handles = Some(excluded_x_handles);
        self
    }

    /// Sets the list of X handles from whom to consider the posts.
    pub fn with_included_x_handles(mut self, included_x_handles: Vec<String>) -> Self {
        self.included_x_handles = Some(included_x_handles);
        self
    }

    /// Sets the minimum favorite count of the X posts to consider.
    pub fn with_post_favorite_count(mut self, post_favorite_count: u32) -> Self {
        self.post_favorite_count = Some(post_favorite_count);
        self
    }

    /// Sets the minimum view count of the X posts to consider.
    pub fn with_post_view_count(mut self, post_view_count: u32) -> Self {
        self.post_view_count = Some(post_view_count);
        self
    }
}

impl WebSearchConfig {
    /// Creates an empty [`WebSearchConfig`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the list of websites to allow in the search results.
    pub fn with_allowed_websites(mut self, allowed_websites: Vec<String>) -> Self {
        self.allowed_websites = Some(allowed_websites);
        self
    }

    /// Sets the ISO alpha-2 country code to restrict results to.
    pub fn with_country(mut self, country: impl Into<String>) -> Self {
        self.country = Some(country.into());
        self
    }

    /// Sets the list of websites to exclude from the search results.
    pub fn with_excluded_websites(mut self, excluded_websites: Vec<String>) -> Self {
        self.excluded_websites = Some(excluded_websites);
        self
    }

    /// Sets whether mature content is excluded during the search.
    pub fn with_safe_search(mut self, safe_search: bool) -> Self {
        self.safe_search = Some(safe_search);
        self
    }
}

impl NewsSearchSource {
    /// Creates an empty [`NewsSearchSource`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the ISO alpha-2 country code to restrict results to.
    pub fn with_country(mut self, country: impl Into<String>) -> Self {
        self.country = Some(country.into());
        self
    }

    /// Sets the list of websites to exclude from the search results.
    pub fn with_excluded_websites(mut self, excluded_websites: Vec<String>) -> Self {
        self.excluded_websites = Some(excluded_websites);
        self
    }

    /// Sets whether mature content is excluded during the search.
    pub fn with_safe_search(mut self, safe_search: bool) -> Self {
        self.safe_search = Some(safe_search);
        self
    }
}

impl RssSearchSource {
    /// Creates an [`RssSearchSource`] from the given RSS feed links.
    pub fn new(links: Vec<String>) -> Self {
        Self { links }
    }
}

impl ModelResponseConfiguration {
    /// Creates an empty [`ModelResponseConfiguration`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the format that the model must output.
    pub fn with_format(mut self, format: ModelResponseFormat) -> Self {
        self.format = Some(format);
        self
    }
}

impl ModelResponseFormat {
    /// Creates a JSON schema response format constraining the model to the given schema.
    ///
    /// The fields the API keeps for compatibility — `name`, `description` and `strict` — are
    /// left unset.
    pub fn json_schema(schema: schemars::Schema) -> Self {
        Self::JsonSchema {
            schema,
            description: None,
            name: None,
            strict: None,
        }
    }
}
