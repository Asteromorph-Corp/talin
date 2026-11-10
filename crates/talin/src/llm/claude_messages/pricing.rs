use talin_provider::claude_messages::types::{CacheCreation, Usage};

use crate::llm::{LlmPricing, LlmUsage};
use crate::schemas::{UsageLine, UsageLines};

const INPUT_TOKENS: &str = "Input tokens";
const CACHE_WRITES_5M: &str = "5-minute cache writes";
const CACHE_WRITES_1H: &str = "1-hour cache writes";
const CACHE_HITS_REFRESHES: &str = "Cache hits and refreshes";
const OUTPUT_TOKENS: &str = "Output tokens";
const WEB_SEARCH_REQUESTS: &str = "Web search requests";

/// Pricing specification for the Claude Messages API.
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub struct MessagesPricing {
    /// USD price per million input tokens.
    pub base_input_tokens: f64,
    /// USD price for cache writes with 5m ttl (per million tokens).
    ///
    /// If unspecified, defaults to 1.25x `base_input_tokens`.
    pub cache_writes_5m: Option<f64>,
    /// USD price for cache writes with 1h ttl (per million tokens).
    ///
    /// If unspecified, defaults to 2x `base_input_tokens`
    pub cache_writes_1h: Option<f64>,
    /// USD price for cache hits and refreshes (per million tokens).
    ///
    /// If unspecified, defaults to 0.1x `base_input_tokens`
    pub cache_hits_refreshes: Option<f64>,
    /// USD price per million output tokens.
    pub output_tokens: f64,
    /// USD price per web search request.
    pub web_search: Option<f64>,
}

impl MessagesPricing {
    /// Creates a new [`MessagesPricing`] with the given input and output token prices.
    pub fn new(input_tokens: f64, output_tokens: f64) -> Self {
        MessagesPricing {
            base_input_tokens: input_tokens,
            cache_writes_5m: None,
            cache_writes_1h: None,
            cache_hits_refreshes: None,
            output_tokens,
            web_search: None,
        }
    }

    /// Sets the [`cache_writes_5m`](Self::cache_writes_5m) price.
    pub fn with_cache_writes_5m(mut self, price: f64) -> Self {
        self.cache_writes_5m = Some(price);
        self
    }

    /// Sets the [`cache_writes_1h`](Self::cache_writes_1h) price.
    pub fn with_cache_writes_1h(mut self, price: f64) -> Self {
        self.cache_writes_1h = Some(price);
        self
    }

    /// Sets the [`cache_hits_refreshes`](Self::cache_hits_refreshes) price.
    pub fn with_cache_hits_refreshes(mut self, price: f64) -> Self {
        self.cache_hits_refreshes = Some(price);
        self
    }

    /// Sets the [`web_search`](Self::web_search) price.
    pub fn with_web_search(mut self, price: f64) -> Self {
        self.web_search = Some(price);
        self
    }

    /// Builds the usage lines for `usage`.
    ///
    /// When `pricing` is `Some`, each line carries its resolved unit price; when `None`, prices are
    /// omitted.
    fn usage_lines(mut usage: Usage, pricing: Option<&Self>) -> UsageLines {
        if let Some(iterations) = usage.iterations.take() {
            usage.input_tokens = iterations
                .iter()
                .map(|iteration| iteration.input_tokens)
                .sum();
            usage.output_tokens = iterations
                .iter()
                .map(|iteration| iteration.output_tokens)
                .sum();
            usage.cache_read_input_tokens = Some(
                iterations
                    .iter()
                    .map(|iteration| iteration.cache_read_input_tokens.unwrap_or_default())
                    .sum(),
            );
            usage.cache_creation = Some(CacheCreation::new(
                iterations
                    .iter()
                    .filter_map(|iteration| iteration.cache_creation.as_ref())
                    .map(|cache| cache.ephemeral_1h_input_tokens)
                    .sum(),
                iterations
                    .iter()
                    .filter_map(|iteration| iteration.cache_creation.as_ref())
                    .map(|cache| cache.ephemeral_5m_input_tokens)
                    .sum(),
            ));
        }
        let input_tokens = usage.input_tokens;
        let cache_writes_5m = usage
            .cache_creation
            .as_ref()
            .map(|c| c.ephemeral_5m_input_tokens)
            .unwrap_or_default();
        let cache_writes_1h = usage
            .cache_creation
            .as_ref()
            .map(|c| c.ephemeral_1h_input_tokens)
            .unwrap_or_default();
        let cache_hits_refreshes = usage.cache_read_input_tokens.unwrap_or_default();
        let output_tokens = usage.output_tokens;
        let web_search_requests = usage
            .server_tool_use
            .map(|s| s.web_search_requests)
            .unwrap_or_default();

        let input_price = pricing.map(|p| p.base_input_tokens);
        let cache_5m_price =
            pricing.map(|p| p.cache_writes_5m.unwrap_or(p.base_input_tokens * 1.25));
        let cache_1h_price =
            pricing.map(|p| p.cache_writes_1h.unwrap_or(p.base_input_tokens * 2.0));
        let cache_read_price =
            pricing.map(|p| p.cache_hits_refreshes.unwrap_or(p.base_input_tokens * 0.1));
        let output_price = pricing.map(|p| p.output_tokens);
        let web_search_price = pricing.and_then(|p| p.web_search);

        UsageLines::from_iter([
            (
                INPUT_TOKENS.into(),
                UsageLine::per_million_tokens(input_tokens, input_price),
            ),
            (
                CACHE_WRITES_5M.into(),
                UsageLine::per_million_tokens(cache_writes_5m, cache_5m_price),
            ),
            (
                CACHE_WRITES_1H.into(),
                UsageLine::per_million_tokens(cache_writes_1h, cache_1h_price),
            ),
            (
                CACHE_HITS_REFRESHES.into(),
                UsageLine::per_million_tokens(cache_hits_refreshes, cache_read_price),
            ),
            (
                OUTPUT_TOKENS.into(),
                UsageLine::per_million_tokens(output_tokens, output_price),
            ),
            (
                WEB_SEARCH_REQUESTS.into(),
                UsageLine::new(web_search_requests, web_search_price, "requests", 1),
            ),
        ])
    }
}

impl LlmUsage for Usage {}

impl LlmPricing for MessagesPricing {
    type Usage = Usage;

    fn default_usage_lines(usage: Self::Usage) -> UsageLines {
        Self::usage_lines(usage, None)
    }

    fn compute_usage_lines(&self, usage: Self::Usage) -> UsageLines {
        Self::usage_lines(usage, Some(self))
    }
}
