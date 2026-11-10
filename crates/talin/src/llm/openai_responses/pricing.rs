use talin_provider::openai_responses::types::ResponseUsage;

use crate::llm::{LlmPricing, LlmUsage};
use crate::schemas::{UsageLine, UsageLines};

const INPUT_TOKENS: &str = "Input tokens";
const INPUT_TOKENS_CACHED: &str = "Input tokens (cached)";
const OUTPUT_TOKENS: &str = "Output tokens";

#[derive(Debug, Clone, Copy, Default)]
#[non_exhaustive]
/// Pricing specification for the OpenAI Responses API.
pub struct ResponsesPricing {
    /// USD price per million input tokens.
    pub input_tokens: f64,
    /// USD price per million cached input tokens.
    ///
    /// If unspecified, defaults to the same price as `input_tokens` for the sake of price
    /// calculation.
    pub cached_input_tokens: Option<f64>,
    /// USD price per million output tokens.
    pub output_tokens: f64,
}

impl ResponsesPricing {
    /// Creates a new [`ResponsesPricing`] with the given prices.
    pub fn new(input_tokens: f64, cached_input_tokens: Option<f64>, output_tokens: f64) -> Self {
        ResponsesPricing {
            input_tokens,
            cached_input_tokens,
            output_tokens,
        }
    }

    /// Builds the usage lines for `usage`, carrying resolved unit prices when `pricing` is `Some`.
    fn usage_lines(usage: ResponseUsage, pricing: Option<&Self>) -> UsageLines {
        let cached_input_tokens = usage.input_tokens_details.cached_tokens;
        let input_tokens = usage.input_tokens.saturating_sub(cached_input_tokens);

        let input_price = pricing.map(|p| p.input_tokens);
        let cached_price = pricing.map(|p| p.cached_input_tokens.unwrap_or(p.input_tokens));
        let output_price = pricing.map(|p| p.output_tokens);

        UsageLines::from_iter([
            (
                INPUT_TOKENS.into(),
                UsageLine::per_million_tokens(input_tokens, input_price),
            ),
            (
                INPUT_TOKENS_CACHED.into(),
                UsageLine::per_million_tokens(cached_input_tokens, cached_price),
            ),
            (
                OUTPUT_TOKENS.into(),
                UsageLine::per_million_tokens(usage.output_tokens, output_price),
            ),
        ])
    }
}

impl LlmUsage for ResponseUsage {}

impl LlmPricing for ResponsesPricing {
    type Usage = ResponseUsage;

    fn default_usage_lines(usage: ResponseUsage) -> UsageLines {
        Self::usage_lines(usage, None)
    }

    fn compute_usage_lines(&self, usage: ResponseUsage) -> UsageLines {
        Self::usage_lines(usage, Some(self))
    }
}
