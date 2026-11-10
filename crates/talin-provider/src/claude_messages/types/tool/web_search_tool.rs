use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::claude_messages::types::{AllowedCaller, CacheControlEphemeral, ResponseInclusion};
use crate::utils::{fixed_value, tagged_enum};

/// The web search tool (2025-03-05).
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[non_exhaustive]
pub struct WebSearchTool20250305 {
    /// The name of the tool, must be `"web_search"`.
    #[serde(default)]
    #[cfg_attr(test, schema(required = true))]
    pub name: WebSearchToolName,

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
    pub max_uses: Option<usize>,

    /// List of domains to allow searching from.
    /// Cannot be used alongside [`blocked_domains`](Self::blocked_domains).
    pub allowed_domains: Option<Vec<String>>,

    /// List of domains to block searching from.
    /// Cannot be used alongside [`allowed_domains`](Self::allowed_domains).
    pub blocked_domains: Option<Vec<String>>,

    /// Parameters for the user's location. Used to provide more relevant search results.
    pub user_location: Option<UserLocation>,

    /// Create a cache control breakpoint at this content block.
    pub cache_control: Option<CacheControlEphemeral>,
}

/// The web search tool (2026-02-09).
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[non_exhaustive]
pub struct WebSearchTool20260209 {
    /// The name of the tool, must be `"web_search"`.
    #[serde(default)]
    #[cfg_attr(test, schema(required = true))]
    pub name: WebSearchToolName,

    /// Restricts which callers may invoke this tool.
    #[cfg_attr(test, schema(nullable = false))]
    pub allowed_callers: Option<Vec<AllowedCaller>>,

    /// List of domains to allow searching from.
    /// Cannot be used alongside [`blocked_domains`](Self::blocked_domains).
    pub allowed_domains: Option<Vec<String>>,

    /// List of domains to block searching from.
    /// Cannot be used alongside [`allowed_domains`](Self::allowed_domains).
    pub blocked_domains: Option<Vec<String>>,

    /// Create a cache control breakpoint at this content block.
    pub cache_control: Option<CacheControlEphemeral>,

    /// Whether the tool definition should be deferred until first use.
    #[cfg_attr(test, schema(nullable = false))]
    pub defer_loading: Option<bool>,

    /// Maximum number of times the tool can be used per request.
    #[cfg_attr(test, schema(schema_with = crate::claude_messages::spec::schema::nullable_number_schema))]
    pub max_uses: Option<u32>,

    /// Whether to enforce strict input validation for this tool.
    #[cfg_attr(test, schema(nullable = false))]
    pub strict: Option<bool>,

    /// Parameters for the user's location. Used to provide more relevant search results.
    pub user_location: Option<UserLocation>,
}

/// The web search tool (2026-03-18).
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[non_exhaustive]
pub struct WebSearchTool20260318 {
    /// The name of the tool, must be `"web_search"`.
    #[serde(default)]
    #[cfg_attr(test, schema(required = true))]
    pub name: WebSearchToolName,

    /// Restricts which callers may invoke this tool.
    #[cfg_attr(test, schema(nullable = false))]
    pub allowed_callers: Option<Vec<AllowedCaller>>,

    /// List of domains to allow searching from.
    /// Cannot be used alongside [`blocked_domains`](Self::blocked_domains).
    pub allowed_domains: Option<Vec<String>>,

    /// List of domains to block searching from.
    /// Cannot be used alongside [`allowed_domains`](Self::allowed_domains).
    pub blocked_domains: Option<Vec<String>>,

    /// Create a cache control breakpoint at this content block.
    pub cache_control: Option<CacheControlEphemeral>,

    /// Whether the tool definition should be deferred until first use.
    #[cfg_attr(test, schema(nullable = false))]
    pub defer_loading: Option<bool>,

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

    /// Parameters for the user's location. Used to provide more relevant search results.
    pub user_location: Option<UserLocation>,
}

tagged_enum! {
    /// Parameters for the user's location. Used to provide more relevant search results.
    #[derive(Serialize, Deserialize, Debug, Clone)]
    #[serde(tag = "type", rename_all = "snake_case")]
    pub enum UserLocation {
        /// An approximate location based on IP address.
        Approximate(UserLocationApproximate),
    }
}

/// Approximate user location based on IP address.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
#[non_exhaustive]
pub struct UserLocationApproximate {
    /// The two letter ISO country code of the user. (e.g. "US", "JP", "GB")
    pub country: Option<String>,

    /// The region of the user. (e.g. "California", "Ontario", "Wales")
    pub region: Option<String>,

    /// The city of the user. (e.g. "New York", "Tokyo", "Los Angeles")
    pub city: Option<String>,

    /// The IANA timezone of the user. (e.g. "America/New_York", "Asia/Tokyo", "Europe/London")
    pub timezone: Option<String>,
}

impl WebSearchTool20250305 {
    /// Creates a new [`WebSearchTool20250305`].
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
    pub fn with_max_uses(mut self, max_uses: usize) -> Self {
        self.max_uses = Some(max_uses);
        self
    }

    /// Sets the [`allowed_domains`](Self::allowed_domains) to search from.
    pub fn with_allowed_domains(mut self, allowed_domains: Vec<String>) -> Self {
        self.allowed_domains = Some(allowed_domains);
        self
    }

    /// Sets the [`blocked_domains`](Self::blocked_domains) to block searching from.
    pub fn with_blocked_domains(mut self, blocked_domains: Vec<String>) -> Self {
        self.blocked_domains = Some(blocked_domains);
        self
    }

    /// Sets the [`user_location`](Self::user_location) for more relevant search results.
    pub fn with_user_location(mut self, user_location: UserLocation) -> Self {
        self.user_location = Some(user_location);
        self
    }

    /// Sets the [`cache_control`](Self::cache_control) breakpoint for this content block.
    pub fn with_cache_control(mut self, cache_control: CacheControlEphemeral) -> Self {
        self.cache_control = Some(cache_control);
        self
    }
}

impl WebSearchTool20260209 {
    /// Creates a new [`WebSearchTool20260209`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the [`allowed_callers`](Self::allowed_callers) permitted to invoke this tool.
    pub fn with_allowed_callers(mut self, allowed_callers: Vec<AllowedCaller>) -> Self {
        self.allowed_callers = Some(allowed_callers);
        self
    }

    /// Sets the [`allowed_domains`](Self::allowed_domains) to search from.
    pub fn with_allowed_domains(mut self, allowed_domains: Vec<String>) -> Self {
        self.allowed_domains = Some(allowed_domains);
        self
    }

    /// Sets the [`blocked_domains`](Self::blocked_domains) to block searching from.
    pub fn with_blocked_domains(mut self, blocked_domains: Vec<String>) -> Self {
        self.blocked_domains = Some(blocked_domains);
        self
    }

    /// Sets the [`cache_control`](Self::cache_control) breakpoint for this content block.
    pub fn with_cache_control(mut self, cache_control: CacheControlEphemeral) -> Self {
        self.cache_control = Some(cache_control);
        self
    }

    /// Sets whether the tool definition should be [`defer_loading`](Self::defer_loading) until
    /// first use.
    pub fn with_defer_loading(mut self, defer_loading: bool) -> Self {
        self.defer_loading = Some(defer_loading);
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

    /// Sets the [`user_location`](Self::user_location) for more relevant search results.
    pub fn with_user_location(mut self, user_location: UserLocation) -> Self {
        self.user_location = Some(user_location);
        self
    }
}

impl WebSearchTool20260318 {
    /// Creates a new [`WebSearchTool20260318`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the [`allowed_callers`](Self::allowed_callers) permitted to invoke this tool.
    pub fn with_allowed_callers(mut self, allowed_callers: Vec<AllowedCaller>) -> Self {
        self.allowed_callers = Some(allowed_callers);
        self
    }

    /// Sets the [`allowed_domains`](Self::allowed_domains) to search from.
    pub fn with_allowed_domains(mut self, allowed_domains: Vec<String>) -> Self {
        self.allowed_domains = Some(allowed_domains);
        self
    }

    /// Sets the [`blocked_domains`](Self::blocked_domains) to block searching from.
    pub fn with_blocked_domains(mut self, blocked_domains: Vec<String>) -> Self {
        self.blocked_domains = Some(blocked_domains);
        self
    }

    /// Sets the [`cache_control`](Self::cache_control) breakpoint for this content block.
    pub fn with_cache_control(mut self, cache_control: CacheControlEphemeral) -> Self {
        self.cache_control = Some(cache_control);
        self
    }

    /// Sets whether the tool definition should be [`defer_loading`](Self::defer_loading) until
    /// first use.
    pub fn with_defer_loading(mut self, defer_loading: bool) -> Self {
        self.defer_loading = Some(defer_loading);
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

    /// Sets the [`user_location`](Self::user_location) for more relevant search results.
    pub fn with_user_location(mut self, user_location: UserLocation) -> Self {
        self.user_location = Some(user_location);
        self
    }
}

impl UserLocationApproximate {
    /// Creates a new [`UserLocationApproximate`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the [`country`](Self::country) of the user.
    pub fn with_country(mut self, country: impl Into<String>) -> Self {
        self.country = Some(country.into());
        self
    }

    /// Sets the [`region`](Self::region) of the user.
    pub fn with_region(mut self, region: impl Into<String>) -> Self {
        self.region = Some(region.into());
        self
    }

    /// Sets the [`city`](Self::city) of the user.
    pub fn with_city(mut self, city: impl Into<String>) -> Self {
        self.city = Some(city.into());
        self
    }

    /// Sets the [`timezone`](Self::timezone) of the user.
    pub fn with_timezone(mut self, timezone: impl Into<String>) -> Self {
        self.timezone = Some(timezone.into());
        self
    }
}

fixed_value!(WebSearchToolName::WebSearch);
