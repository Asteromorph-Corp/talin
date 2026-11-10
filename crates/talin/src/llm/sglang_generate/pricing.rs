use talin_provider::sglang_generate::types::MetaInfo;

use crate::llm::{LlmPricing, LlmUsage};
use crate::schemas::{UsageLine, UsageLines};

const PROMPT_TOKENS: &str = "Prompt tokens";
const PROMPT_TOKENS_CACHED: &str = "Prompt tokens (cached)";
const COMPLETION_TOKENS: &str = "Completion tokens";

/// Token usage extracted from a `/generate` response's `meta_info`.
#[derive(Debug, Clone, Default)]
pub struct GenerateUsage {
    /// The number of prompt tokens, including any cached ones.
    pub prompt_tokens: u32,
    /// The number of prompt tokens served from cache.
    pub cached_tokens: u32,
    /// The number of generated tokens.
    pub completion_tokens: u32,
}

impl From<&MetaInfo> for GenerateUsage {
    fn from(meta: &MetaInfo) -> Self {
        Self {
            prompt_tokens: meta.prompt_tokens as u32,
            cached_tokens: meta.cached_tokens.unwrap_or(0) as u32,
            completion_tokens: meta.completion_tokens.unwrap_or(0) as u32,
        }
    }
}

impl LlmUsage for GenerateUsage {}

/// Pricing specification for SGLang Generate API.
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub struct GeneratePricing {
    /// USD price per million input tokens.
    pub input_tokens: f64,
    /// USD price per million cached input tokens.
    ///
    /// If unspecified, defaults to the same price as `input_tokens`.
    pub cached_input_tokens: Option<f64>,
    /// USD price per million output tokens.
    pub output_tokens: f64,
}

impl GeneratePricing {
    /// Creates a new [`GeneratePricing`] with the given prices.
    pub fn new(input_tokens: f64, cached_input_tokens: Option<f64>, output_tokens: f64) -> Self {
        GeneratePricing {
            input_tokens,
            cached_input_tokens,
            output_tokens,
        }
    }

    /// Builds the usage lines for `usage`, carrying resolved unit prices when `pricing` is `Some`.
    fn usage_lines(usage: GenerateUsage, pricing: Option<&Self>) -> UsageLines {
        let input_tokens = usage.prompt_tokens.saturating_sub(usage.cached_tokens);

        let input_price = pricing.map(|p| p.input_tokens);
        let cached_price = pricing.map(|p| p.cached_input_tokens.unwrap_or(p.input_tokens));
        let output_price = pricing.map(|p| p.output_tokens);

        UsageLines::from_iter([
            (
                PROMPT_TOKENS.into(),
                UsageLine::per_million_tokens(input_tokens, input_price),
            ),
            (
                PROMPT_TOKENS_CACHED.into(),
                UsageLine::per_million_tokens(usage.cached_tokens, cached_price),
            ),
            (
                COMPLETION_TOKENS.into(),
                UsageLine::per_million_tokens(usage.completion_tokens, output_price),
            ),
        ])
    }
}

impl LlmPricing for GeneratePricing {
    type Usage = GenerateUsage;

    fn default_usage_lines(usage: GenerateUsage) -> UsageLines {
        Self::usage_lines(usage, None)
    }

    fn compute_usage_lines(&self, usage: GenerateUsage) -> UsageLines {
        Self::usage_lines(usage, Some(self))
    }
}
