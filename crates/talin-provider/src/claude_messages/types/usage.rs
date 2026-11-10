use serde::{Deserialize, Serialize};

use crate::claude_messages::types::{Model, ServiceTier, Speed};
use crate::utils::string_enum;

/// Billing and rate-limit usage information.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct Usage {
    // Should be serialized as `null` if `None`.
    /// Breakdown of cached tokens by TTL.
    ///
    /// See [`cache_creation`](https://docs.claude.com/en/api/messages#response-usage-cache-creation)
    #[cfg_attr(test, schema(required = true))]
    pub cache_creation: Option<CacheCreation>,

    // Should be serialized as `null` if `None`.
    /// The number of input tokens used to create the cache entry.
    ///
    /// See [`cache_creation_input_tokens`](https://docs.claude.com/en/api/messages#response-usage-cache-creation-input-tokens)
    #[cfg_attr(test, schema(required = true, schema_with = crate::claude_messages::spec::schema::nullable_number_schema))]
    pub cache_creation_input_tokens: Option<u32>,

    // Should be serialized as `null` if `None`.
    /// The number of input tokens read from the cache.
    ///
    /// See [`cache_read_input_tokens`](https://docs.claude.com/en/api/messages#response-usage-cache-read-input-tokens)
    #[cfg_attr(test, schema(required = true, schema_with = crate::claude_messages::spec::schema::nullable_number_schema))]
    pub cache_read_input_tokens: Option<u32>,

    /// The number of input tokens which were used.
    ///
    /// See [`input_tokens`](https://docs.claude.com/en/api/messages#response-usage-input-tokens)
    #[cfg_attr(test, schema(schema_with = crate::claude_messages::spec::schema::number_schema))]
    pub input_tokens: u32,

    /// The number of output tokens which were generated.
    ///
    /// See [`output_tokens`](https://docs.claude.com/en/api/messages#response-usage-output-tokens)
    #[cfg_attr(test, schema(schema_with = crate::claude_messages::spec::schema::number_schema))]
    pub output_tokens: u32,

    /// Per-iteration usage, including compaction work excluded from the top-level totals.
    ///
    /// Anthropic's beta stream accumulators currently replace this array when a `message_delta`
    /// includes it:
    /// - [TypeScript](https://github.com/anthropics/anthropic-sdk-typescript/blob/main/src/lib/BetaMessageStream.ts#L2885-L2889)
    /// - [Go](https://github.com/anthropics/anthropic-sdk-go/blob/main/betamessageutil.go#L1705-L1709)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iterations: Option<Vec<IterationUsage>>,

    // Should be serialized as `null` if `None`.
    /// Detailed breakdown of the output tokens which were generated.
    ///
    /// See [`output_tokens_details`](https://docs.claude.com/en/api/messages#response-usage-output-tokens-details)
    #[cfg_attr(test, schema(required = true))]
    pub output_tokens_details: Option<OutputTokensDetails>,

    // Should be serialized as `null` if `None`.
    /// The geographic region used for inference.
    #[cfg_attr(test, schema(required = true))]
    pub inference_geo: Option<String>,

    // Should be serialized as `null` if `None`.
    /// The number of server tool requests.
    ///
    /// See [`server_tool_use`](https://docs.claude.com/en/api/messages#response-usage-server-tool-use)
    #[cfg_attr(test, schema(required = true))]
    pub server_tool_use: Option<ServerToolUsage>,

    // Should be serialized as `null` if `None`.
    /// If the request used the priority, standard, or batch tier.
    ///
    /// See [`service_tier`](https://docs.claude.com/en/api/messages#response-usage-service-tier)
    #[cfg_attr(test, schema(required = true))]
    pub service_tier: Option<ServiceTier>,

    // Should be serialized as `null` if `None`.
    /// The inference speed mode used for this request.
    #[cfg_attr(test, schema(required = true))]
    pub speed: Option<Speed>,
}

/// Breakdown of cached tokens by TTL.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct CacheCreation {
    /// The number of input tokens used to create the 1 hour cache entry.
    ///
    /// See [`ephemeral_1h_input_tokens`](https://docs.claude.com/en/api/messages#response-usage-cache-creation-ephemeral-1h-input-tokens)
    #[cfg_attr(test, schema(schema_with = crate::claude_messages::spec::schema::number_schema))]
    pub ephemeral_1h_input_tokens: u32,

    /// The number of input tokens used to create the 5 minute cache entry.
    ///
    /// See [`ephemeral_5m_input_tokens`](https://docs.claude.com/en/api/messages#response-usage-cache-creation-ephemeral-5m-input-tokens)
    #[cfg_attr(test, schema(schema_with = crate::claude_messages::spec::schema::number_schema))]
    pub ephemeral_5m_input_tokens: u32,
}

/// Token usage for one sampling, compaction, advisor, or fallback iteration.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct IterationUsage {
    /// The kind of configuration or iteration represented by this object.
    #[serde(rename = "type")]
    pub type_: IterationUsageType,
    /// Input tokens consumed by this iteration.
    pub input_tokens: u32,
    /// Output tokens produced by this iteration.
    pub output_tokens: u32,
    /// Cache creation usage for this iteration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_creation: Option<CacheCreation>,
    /// Input tokens used to create a cache entry during this iteration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_creation_input_tokens: Option<u32>,
    /// Input tokens read from cache during this iteration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_read_input_tokens: Option<u32>,
    /// The model used for this iteration, when reported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<Model>,
}

string_enum! {
    /// The kind of work represented by an iteration usage entry.
    #[serde(rename_all = "snake_case")]
    pub enum IterationUsageType {
        /// Usage for sampling a normal message.
        Message,

        /// Usage for generating a compaction summary.
        Compaction,

        /// Usage for sampling an advisor message.
        AdvisorMessage,

        /// Usage for sampling a fallback message.
        FallbackMessage,
    }
}

/// Detailed breakdown of the output tokens which were generated.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct OutputTokensDetails {
    /// The number of output tokens the model generated as internal reasoning, including the
    /// thinking-block delimiter tokens.
    ///
    /// See [`thinking_tokens`](https://docs.claude.com/en/api/messages#response-usage-output-tokens-details-thinking-tokens)
    #[cfg_attr(test, schema(schema_with = crate::claude_messages::spec::schema::number_schema))]
    pub thinking_tokens: u32,
}

/// The number of server tool requests.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ServerToolUsage {
    /// The number of web search tool requests.
    ///
    /// See [`web_search_requests`](https://docs.claude.com/en/api/messages#response-usage-server-tool-use-web-search-requests)
    #[cfg_attr(test, schema(schema_with = crate::claude_messages::spec::schema::number_schema))]
    pub web_search_requests: u32,

    /// The number of web fetch tool requests.
    #[cfg_attr(test, schema(required = true, schema_with = crate::claude_messages::spec::schema::number_schema))]
    pub web_fetch_requests: Option<u32>,
}

impl Usage {
    /// Creates a new [`Usage`] from the given input and output token counts.
    pub fn new(input_tokens: u32, output_tokens: u32) -> Self {
        Self {
            cache_creation: None,
            cache_creation_input_tokens: None,
            cache_read_input_tokens: None,
            input_tokens,
            output_tokens,
            iterations: None,
            output_tokens_details: None,
            inference_geo: None,
            server_tool_use: None,
            service_tier: None,
            speed: None,
        }
    }

    /// Sets the [`cache_creation`](Self::cache_creation) breakdown.
    pub fn with_cache_creation(mut self, cache_creation: CacheCreation) -> Self {
        self.cache_creation = Some(cache_creation);
        self
    }

    /// Sets the per-iteration token usage breakdown.
    pub fn with_iterations(mut self, iterations: Vec<IterationUsage>) -> Self {
        self.iterations = Some(iterations);
        self
    }

    /// Sets the [`cache_creation_input_tokens`](Self::cache_creation_input_tokens) count.
    pub fn with_cache_creation_input_tokens(mut self, cache_creation_input_tokens: u32) -> Self {
        self.cache_creation_input_tokens = Some(cache_creation_input_tokens);
        self
    }

    /// Sets the [`cache_read_input_tokens`](Self::cache_read_input_tokens) count.
    pub fn with_cache_read_input_tokens(mut self, cache_read_input_tokens: u32) -> Self {
        self.cache_read_input_tokens = Some(cache_read_input_tokens);
        self
    }

    /// Sets the [`output_tokens_details`](Self::output_tokens_details) breakdown.
    pub fn with_output_tokens_details(
        mut self,
        output_tokens_details: OutputTokensDetails,
    ) -> Self {
        self.output_tokens_details = Some(output_tokens_details);
        self
    }

    /// Sets the [`inference_geo`](Self::inference_geo) used for inference.
    pub fn with_inference_geo(mut self, inference_geo: impl Into<String>) -> Self {
        self.inference_geo = Some(inference_geo.into());
        self
    }

    /// Sets the [`server_tool_use`](Self::server_tool_use) counts.
    pub fn with_server_tool_use(mut self, server_tool_use: ServerToolUsage) -> Self {
        self.server_tool_use = Some(server_tool_use);
        self
    }

    /// Sets the [`service_tier`](Self::service_tier) that processed the request.
    pub fn with_service_tier(mut self, service_tier: ServiceTier) -> Self {
        self.service_tier = Some(service_tier);
        self
    }
}

impl Usage {
    /// Sets the [`speed`](Self::speed) mode that processed the request.
    pub fn with_speed(mut self, speed: Speed) -> Self {
        self.speed = Some(speed);
        self
    }
}

impl CacheCreation {
    /// Creates a new [`CacheCreation`] from the given per-TTL token counts.
    pub fn new(ephemeral_1h_input_tokens: u32, ephemeral_5m_input_tokens: u32) -> Self {
        Self {
            ephemeral_1h_input_tokens,
            ephemeral_5m_input_tokens,
        }
    }
}

impl OutputTokensDetails {
    /// Creates a new [`OutputTokensDetails`] from the given thinking token count.
    pub fn new(thinking_tokens: u32) -> Self {
        Self { thinking_tokens }
    }
}

impl ServerToolUsage {
    /// Creates a new [`ServerToolUsage`] from the given web search request count.
    pub fn new(web_search_requests: u32) -> Self {
        Self {
            web_search_requests,
            web_fetch_requests: None,
        }
    }

    /// Sets the [`web_fetch_requests`](Self::web_fetch_requests) count.
    pub fn with_web_fetch_requests(mut self, web_fetch_requests: u32) -> Self {
        self.web_fetch_requests = Some(web_fetch_requests);
        self
    }
}
