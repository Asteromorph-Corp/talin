use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::claude_messages::types::{
    AllowedCaller, CacheControlEphemeral, CitationsConfigParam, ResponseInclusion,
};
use crate::utils::fixed_value;

/// The web fetch tool (2025-09-10).
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[non_exhaustive]
pub struct WebFetchTool20250910 {
    /// The name of the tool, must be `"web_fetch"`.
    #[serde(default)]
    #[cfg_attr(test, schema(required = true))]
    pub name: WebFetchToolName,

    /// Restricts which callers may invoke this tool.
    #[cfg_attr(test, schema(nullable = false))]
    pub allowed_callers: Option<Vec<AllowedCaller>>,

    /// Whether the tool definition should be deferred until first use.
    #[cfg_attr(test, schema(nullable = false))]
    pub defer_loading: Option<bool>,

    /// Whether to enforce strict input validation for this tool.
    #[cfg_attr(test, schema(nullable = false))]
    pub strict: Option<bool>,

    /// Maximum number of times the tool can be used per request.
    #[cfg_attr(test, schema(schema_with = crate::claude_messages::spec::schema::nullable_number_schema))]
    pub max_uses: Option<u32>,

    /// List of domains to allow fetching from.
    /// Cannot be used alongside [`blocked_domains`](Self::blocked_domains).
    pub allowed_domains: Option<Vec<String>>,

    /// List of domains to block fetching from.
    /// Cannot be used alongside [`allowed_domains`](Self::allowed_domains).
    pub blocked_domains: Option<Vec<String>>,

    /// Citations configuration for fetched documents. Citations are disabled by default.
    pub citations: Option<CitationsConfigParam>,

    /// Maximum number of tokens used by including web page text content in the context. The limit
    /// is approximate and does not apply to binary content such as PDFs.
    #[cfg_attr(test, schema(schema_with = crate::claude_messages::spec::schema::nullable_number_schema))]
    pub max_content_tokens: Option<u32>,

    /// Create a cache control breakpoint at this content block.
    pub cache_control: Option<CacheControlEphemeral>,

    /// Which sources contribute to the set of URLs web fetch may fetch.
    pub url_sources: Option<WebFetchURLSources>,
}

/// The web fetch tool (2026-02-09).
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[non_exhaustive]
pub struct WebFetchTool20260209 {
    /// The name of the tool, must be `"web_fetch"`.
    #[serde(default)]
    #[cfg_attr(test, schema(required = true))]
    pub name: WebFetchToolName,

    /// Restricts which callers may invoke this tool.
    #[cfg_attr(test, schema(nullable = false))]
    pub allowed_callers: Option<Vec<AllowedCaller>>,

    /// List of domains to allow fetching from.
    /// Cannot be used alongside [`blocked_domains`](Self::blocked_domains).
    pub allowed_domains: Option<Vec<String>>,

    /// List of domains to block fetching from.
    /// Cannot be used alongside [`allowed_domains`](Self::allowed_domains).
    pub blocked_domains: Option<Vec<String>>,

    /// Create a cache control breakpoint at this content block.
    pub cache_control: Option<CacheControlEphemeral>,

    /// Citations configuration for fetched documents. Citations are disabled by default.
    pub citations: Option<CitationsConfigParam>,

    /// Whether the tool definition should be deferred until first use.
    #[cfg_attr(test, schema(nullable = false))]
    pub defer_loading: Option<bool>,

    /// Maximum number of tokens used by including web page text content in the context. The limit
    /// is approximate and does not apply to binary content such as PDFs.
    #[cfg_attr(test, schema(schema_with = crate::claude_messages::spec::schema::nullable_number_schema))]
    pub max_content_tokens: Option<u32>,

    /// Maximum number of times the tool can be used per request.
    #[cfg_attr(test, schema(schema_with = crate::claude_messages::spec::schema::nullable_number_schema))]
    pub max_uses: Option<u32>,

    /// Whether to enforce strict input validation for this tool.
    #[cfg_attr(test, schema(nullable = false))]
    pub strict: Option<bool>,

    /// Which sources contribute to the set of URLs web fetch may fetch.
    pub url_sources: Option<WebFetchURLSources>,
}

/// The web fetch tool (2026-03-09).
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[non_exhaustive]
pub struct WebFetchTool20260309 {
    /// The name of the tool, must be `"web_fetch"`.
    #[serde(default)]
    #[cfg_attr(test, schema(required = true))]
    pub name: WebFetchToolName,

    /// Restricts which callers may invoke this tool.
    #[cfg_attr(test, schema(nullable = false))]
    pub allowed_callers: Option<Vec<AllowedCaller>>,

    /// List of domains to allow fetching from.
    /// Cannot be used alongside [`blocked_domains`](Self::blocked_domains).
    pub allowed_domains: Option<Vec<String>>,

    /// List of domains to block fetching from.
    /// Cannot be used alongside [`allowed_domains`](Self::allowed_domains).
    pub blocked_domains: Option<Vec<String>>,

    /// Create a cache control breakpoint at this content block.
    pub cache_control: Option<CacheControlEphemeral>,

    /// Citations configuration for fetched documents. Citations are disabled by default.
    pub citations: Option<CitationsConfigParam>,

    /// Whether the tool definition should be deferred until first use.
    #[cfg_attr(test, schema(nullable = false))]
    pub defer_loading: Option<bool>,

    /// Maximum number of tokens used by including web page text content in the context. The limit
    /// is approximate and does not apply to binary content such as PDFs.
    #[cfg_attr(test, schema(schema_with = crate::claude_messages::spec::schema::nullable_number_schema))]
    pub max_content_tokens: Option<u32>,

    /// Maximum number of times the tool can be used per request.
    #[cfg_attr(test, schema(schema_with = crate::claude_messages::spec::schema::nullable_number_schema))]
    pub max_uses: Option<u32>,

    /// Whether to enforce strict input validation for this tool.
    #[cfg_attr(test, schema(nullable = false))]
    pub strict: Option<bool>,

    /// Whether to use cached content. Set to `false` to bypass the cache and fetch fresh content.
    #[cfg_attr(test, schema(nullable = false))]
    pub use_cache: Option<bool>,

    /// Which sources contribute to the set of URLs web fetch may fetch.
    pub url_sources: Option<WebFetchURLSources>,
}

/// The web fetch tool (2026-03-18).
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[non_exhaustive]
pub struct WebFetchTool20260318 {
    /// The name of the tool, must be `"web_fetch"`.
    #[serde(default)]
    #[cfg_attr(test, schema(required = true))]
    pub name: WebFetchToolName,

    /// Restricts which callers may invoke this tool.
    #[cfg_attr(test, schema(nullable = false))]
    pub allowed_callers: Option<Vec<AllowedCaller>>,

    /// List of domains to allow fetching from.
    /// Cannot be used alongside [`blocked_domains`](Self::blocked_domains).
    pub allowed_domains: Option<Vec<String>>,

    /// List of domains to block fetching from.
    /// Cannot be used alongside [`allowed_domains`](Self::allowed_domains).
    pub blocked_domains: Option<Vec<String>>,

    /// Create a cache control breakpoint at this content block.
    pub cache_control: Option<CacheControlEphemeral>,

    /// Citations configuration for fetched documents. Citations are disabled by default.
    pub citations: Option<CitationsConfigParam>,

    /// Whether the tool definition should be deferred until first use.
    #[cfg_attr(test, schema(nullable = false))]
    pub defer_loading: Option<bool>,

    /// Maximum number of tokens used by including web page text content in the context. The limit
    /// is approximate and does not apply to binary content such as PDFs.
    #[cfg_attr(test, schema(schema_with = crate::claude_messages::spec::schema::nullable_number_schema))]
    pub max_content_tokens: Option<u32>,

    /// Maximum number of times the tool can be used per request.
    #[cfg_attr(test, schema(schema_with = crate::claude_messages::spec::schema::nullable_number_schema))]
    pub max_uses: Option<u32>,

    /// How this tool's result blocks appear in the response when the result was consumed by a
    /// completed code execution call in the same turn.
    #[cfg_attr(test, schema(nullable = false))]
    pub response_inclusion: Option<ResponseInclusion>,

    /// Whether to enforce strict input validation for this tool.
    #[cfg_attr(test, schema(nullable = false))]
    pub strict: Option<bool>,

    /// Whether to use cached content. Set to `false` to bypass the cache and fetch fresh content.
    #[cfg_attr(test, schema(nullable = false))]
    pub use_cache: Option<bool>,

    /// Which sources contribute to the set of URLs web fetch may fetch.
    pub url_sources: Option<WebFetchURLSources>,
}

/// Which sources contribute to the set of URLs web fetch may fetch.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[non_exhaustive]
pub struct WebFetchURLSources {
    /// Whether URLs in user messages are fetchable.
    #[cfg_attr(test, schema(nullable = false))]
    pub user_input: Option<WebFetchURLSourcesUserInput>,

    /// Which client tools' results contribute fetchable URLs.
    #[cfg_attr(test, schema(nullable = false))]
    pub client_tool_results: Option<WebFetchURLSourcesClientToolResults>,

    /// Which server tools' results contribute fetchable URLs.
    #[cfg_attr(test, schema(nullable = false))]
    pub server_tool_results: Option<WebFetchURLSourcesServerToolResults>,
}

/// Whether URLs in user messages are fetchable.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
#[non_exhaustive]
pub enum WebFetchURLSourcesUserInput {
    /// Fetch URLs from user messages.
    All(WebFetchUrlSourceAll),
    /// Do not fetch URLs from user messages.
    None(WebFetchURLSourceNone),
}

/// Which client tools' results contribute fetchable URLs.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
#[non_exhaustive]
pub enum WebFetchURLSourcesClientToolResults {
    /// Fetch URLs from all client tool results.
    All(WebFetchUrlSourceAll),
    /// Do not fetch URLs from any client tool results.
    None(WebFetchURLSourceNone),
    /// Fetch URLs only from the specified client tools' results.
    Only(WebFetchURLSourceOnly),
    /// Fetch URLs from all client tools' results except for the specified tools.
    Except(WebFetchURLSourceExcept),
}

/// Which server tools' results contribute fetchable URLs.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
#[non_exhaustive]
pub enum WebFetchURLSourcesServerToolResults {
    /// Fetch URLS from all server tool results.
    All(WebFetchUrlSourceAll),
    /// Do not fetch URLs from any server tool results.
    None(WebFetchURLSourceNone),
    /// Fetch URLs only from the specified server tools' results.
    Only(WebFetchURLSourceOnly),
    /// Fetch URLs from all server tools' results except for the specified tools.
    Except(WebFetchURLSourceExcept),
}

/// The `url_sources` variant under which a source contributes in full.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[non_exhaustive]
pub struct WebFetchUrlSourceAll {}

/// The `url_sources` variant under which a source contributes nothing.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[non_exhaustive]
pub struct WebFetchURLSourceNone {}

/// The tool filter variant under which only the named tools' results contribute.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct WebFetchURLSourceOnly {
    /// The tools whose results may contribute to the set of fetchable URLs.
    pub tools: Vec<WebFetchURLSourceToolReference>,
}

/// The tool filter variant under which every result but the named tools' contributes.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct WebFetchURLSourceExcept {
    /// The tools whose results should not contribute to the set of fetchable URLs.
    pub tools: Vec<WebFetchURLSourceToolReference>,
}

/// One entry of a tool filter's `tools`.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
#[non_exhaustive]
pub enum WebFetchURLSourceToolReference {
    /// A reference to a tool by name.
    ToolReference(WebFetchURLSourceToolReferenceTool),
}

/// One entry of a tool filter's `tools`.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct WebFetchURLSourceToolReferenceTool {
    /// The name of the tool
    pub name: String,
}

impl WebFetchTool20250910 {
    /// Creates a new [`WebFetchTool20250910`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the [`allowed_callers`](Self::allowed_callers) permitted to invoke this tool.
    pub fn with_allowed_callers(mut self, allowed_callers: Vec<AllowedCaller>) -> Self {
        self.allowed_callers = Some(allowed_callers);
        self
    }

    /// Sets whether the tool definition should be [`defer_loading`](Self::defer_loading) until
    /// first use.
    pub fn with_defer_loading(mut self, defer_loading: bool) -> Self {
        self.defer_loading = Some(defer_loading);
        self
    }

    /// Sets whether to enforce [`strict`](Self::strict) input validation for this tool.
    pub fn with_strict(mut self, strict: bool) -> Self {
        self.strict = Some(strict);
        self
    }

    /// Sets the [`max_uses`](Self::max_uses) per request.
    pub fn with_max_uses(mut self, max_uses: u32) -> Self {
        self.max_uses = Some(max_uses);
        self
    }

    /// Sets the [`allowed_domains`](Self::allowed_domains) to fetch from.
    pub fn with_allowed_domains(mut self, allowed_domains: Vec<String>) -> Self {
        self.allowed_domains = Some(allowed_domains);
        self
    }

    /// Sets the [`blocked_domains`](Self::blocked_domains) to block fetching from.
    pub fn with_blocked_domains(mut self, blocked_domains: Vec<String>) -> Self {
        self.blocked_domains = Some(blocked_domains);
        self
    }

    /// Sets the [`citations`](Self::citations) configuration for fetched documents.
    pub fn with_citations(mut self, citations: CitationsConfigParam) -> Self {
        self.citations = Some(citations);
        self
    }

    /// Sets the [`max_content_tokens`](Self::max_content_tokens) used by fetched content.
    pub fn with_max_content_tokens(mut self, max_content_tokens: u32) -> Self {
        self.max_content_tokens = Some(max_content_tokens);
        self
    }

    /// Sets the [`cache_control`](Self::cache_control) breakpoint for this content block.
    pub fn with_cache_control(mut self, cache_control: CacheControlEphemeral) -> Self {
        self.cache_control = Some(cache_control);
        self
    }

    /// Sets which sources contribute to the set of URLs web fetch may fetch.
    pub fn with_url_sources(mut self, url_sources: WebFetchURLSources) -> Self {
        self.url_sources = Some(url_sources);
        self
    }
}

impl WebFetchTool20260209 {
    /// Creates a new [`WebFetchTool20260209`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the [`allowed_callers`](Self::allowed_callers) permitted to invoke this tool.
    pub fn with_allowed_callers(mut self, allowed_callers: Vec<AllowedCaller>) -> Self {
        self.allowed_callers = Some(allowed_callers);
        self
    }

    /// Sets the [`allowed_domains`](Self::allowed_domains) to fetch from.
    pub fn with_allowed_domains(mut self, allowed_domains: Vec<String>) -> Self {
        self.allowed_domains = Some(allowed_domains);
        self
    }

    /// Sets the [`blocked_domains`](Self::blocked_domains) to block fetching from.
    pub fn with_blocked_domains(mut self, blocked_domains: Vec<String>) -> Self {
        self.blocked_domains = Some(blocked_domains);
        self
    }

    /// Sets the [`cache_control`](Self::cache_control) breakpoint for this content block.
    pub fn with_cache_control(mut self, cache_control: CacheControlEphemeral) -> Self {
        self.cache_control = Some(cache_control);
        self
    }

    /// Sets the [`citations`](Self::citations) configuration for fetched documents.
    pub fn with_citations(mut self, citations: CitationsConfigParam) -> Self {
        self.citations = Some(citations);
        self
    }

    /// Sets whether the tool definition should be [`defer_loading`](Self::defer_loading) until
    /// first use.
    pub fn with_defer_loading(mut self, defer_loading: bool) -> Self {
        self.defer_loading = Some(defer_loading);
        self
    }

    /// Sets the [`max_content_tokens`](Self::max_content_tokens) used by fetched content.
    pub fn with_max_content_tokens(mut self, max_content_tokens: u32) -> Self {
        self.max_content_tokens = Some(max_content_tokens);
        self
    }

    /// Sets the [`max_uses`](Self::max_uses) per request.
    pub fn with_max_uses(mut self, max_uses: u32) -> Self {
        self.max_uses = Some(max_uses);
        self
    }

    /// Sets whether to enforce [`strict`](Self::strict) input validation for this tool.
    pub fn with_strict(mut self, strict: bool) -> Self {
        self.strict = Some(strict);
        self
    }

    /// Sets which sources contribute to the set of URLs web fetch may fetch.
    pub fn with_url_sources(mut self, url_sources: WebFetchURLSources) -> Self {
        self.url_sources = Some(url_sources);
        self
    }
}

impl WebFetchTool20260309 {
    /// Creates a new [`WebFetchTool20260309`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the [`allowed_callers`](Self::allowed_callers) permitted to invoke this tool.
    pub fn with_allowed_callers(mut self, allowed_callers: Vec<AllowedCaller>) -> Self {
        self.allowed_callers = Some(allowed_callers);
        self
    }

    /// Sets the [`allowed_domains`](Self::allowed_domains) to fetch from.
    pub fn with_allowed_domains(mut self, allowed_domains: Vec<String>) -> Self {
        self.allowed_domains = Some(allowed_domains);
        self
    }

    /// Sets the [`blocked_domains`](Self::blocked_domains) to block fetching from.
    pub fn with_blocked_domains(mut self, blocked_domains: Vec<String>) -> Self {
        self.blocked_domains = Some(blocked_domains);
        self
    }

    /// Sets the [`cache_control`](Self::cache_control) breakpoint for this content block.
    pub fn with_cache_control(mut self, cache_control: CacheControlEphemeral) -> Self {
        self.cache_control = Some(cache_control);
        self
    }

    /// Sets the [`citations`](Self::citations) configuration for fetched documents.
    pub fn with_citations(mut self, citations: CitationsConfigParam) -> Self {
        self.citations = Some(citations);
        self
    }

    /// Sets whether the tool definition should be [`defer_loading`](Self::defer_loading) until
    /// first use.
    pub fn with_defer_loading(mut self, defer_loading: bool) -> Self {
        self.defer_loading = Some(defer_loading);
        self
    }

    /// Sets the [`max_content_tokens`](Self::max_content_tokens) used by fetched content.
    pub fn with_max_content_tokens(mut self, max_content_tokens: u32) -> Self {
        self.max_content_tokens = Some(max_content_tokens);
        self
    }

    /// Sets the [`max_uses`](Self::max_uses) per request.
    pub fn with_max_uses(mut self, max_uses: u32) -> Self {
        self.max_uses = Some(max_uses);
        self
    }

    /// Sets whether to enforce [`strict`](Self::strict) input validation for this tool.
    pub fn with_strict(mut self, strict: bool) -> Self {
        self.strict = Some(strict);
        self
    }

    /// Sets whether to [`use_cache`](Self::use_cache) for fetched content.
    pub fn with_use_cache(mut self, use_cache: bool) -> Self {
        self.use_cache = Some(use_cache);
        self
    }

    /// Sets which sources contribute to the set of URLs web fetch may fetch.
    pub fn with_url_sources(mut self, url_sources: WebFetchURLSources) -> Self {
        self.url_sources = Some(url_sources);
        self
    }
}

impl WebFetchTool20260318 {
    /// Creates a new [`WebFetchTool20260318`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the [`allowed_callers`](Self::allowed_callers) permitted to invoke this tool.
    pub fn with_allowed_callers(mut self, allowed_callers: Vec<AllowedCaller>) -> Self {
        self.allowed_callers = Some(allowed_callers);
        self
    }

    /// Sets the [`allowed_domains`](Self::allowed_domains) to fetch from.
    pub fn with_allowed_domains(mut self, allowed_domains: Vec<String>) -> Self {
        self.allowed_domains = Some(allowed_domains);
        self
    }

    /// Sets the [`blocked_domains`](Self::blocked_domains) to block fetching from.
    pub fn with_blocked_domains(mut self, blocked_domains: Vec<String>) -> Self {
        self.blocked_domains = Some(blocked_domains);
        self
    }

    /// Sets the [`cache_control`](Self::cache_control) breakpoint for this content block.
    pub fn with_cache_control(mut self, cache_control: CacheControlEphemeral) -> Self {
        self.cache_control = Some(cache_control);
        self
    }

    /// Sets the [`citations`](Self::citations) configuration for fetched documents.
    pub fn with_citations(mut self, citations: CitationsConfigParam) -> Self {
        self.citations = Some(citations);
        self
    }

    /// Sets whether the tool definition should be [`defer_loading`](Self::defer_loading) until
    /// first use.
    pub fn with_defer_loading(mut self, defer_loading: bool) -> Self {
        self.defer_loading = Some(defer_loading);
        self
    }

    /// Sets the [`max_content_tokens`](Self::max_content_tokens) used by fetched content.
    pub fn with_max_content_tokens(mut self, max_content_tokens: u32) -> Self {
        self.max_content_tokens = Some(max_content_tokens);
        self
    }

    /// Sets the [`max_uses`](Self::max_uses) per request.
    pub fn with_max_uses(mut self, max_uses: u32) -> Self {
        self.max_uses = Some(max_uses);
        self
    }

    /// Sets how this tool's results are included in the response
    /// ([`response_inclusion`](Self::response_inclusion)).
    pub fn with_response_inclusion(mut self, response_inclusion: ResponseInclusion) -> Self {
        self.response_inclusion = Some(response_inclusion);
        self
    }

    /// Sets whether to enforce [`strict`](Self::strict) input validation for this tool.
    pub fn with_strict(mut self, strict: bool) -> Self {
        self.strict = Some(strict);
        self
    }

    /// Sets whether to [`use_cache`](Self::use_cache) for fetched content.
    pub fn with_use_cache(mut self, use_cache: bool) -> Self {
        self.use_cache = Some(use_cache);
        self
    }

    /// Sets which sources contribute to the set of URLs web fetch may fetch.
    pub fn with_url_sources(mut self, url_sources: WebFetchURLSources) -> Self {
        self.url_sources = Some(url_sources);
        self
    }
}

impl WebFetchURLSourceOnly {
    /// Creates a new [`WebFetchURLSourceOnly`] with the specified tools.
    pub fn new(tools: Vec<WebFetchURLSourceToolReference>) -> Self {
        Self { tools }
    }
}

impl WebFetchURLSourceExcept {
    /// Creates a new [`WebFetchURLSourceExcept`] with the specified tools.
    pub fn new(tools: Vec<WebFetchURLSourceToolReference>) -> Self {
        Self { tools }
    }
}

impl WebFetchURLSourceToolReferenceTool {
    /// Creates a new [`WebFetchURLSourceToolReferenceTool`] with the specified name.
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

fixed_value!(WebFetchToolName::WebFetch);
