use talin_provider::openai_chat::types::CompletionUsage;

use crate::llm::{LlmPricing, LlmUsage};
use crate::schemas::{UsageLine, UsageLines};

const PROMPT_TOKENS: &str = "Prompt tokens";
const PROMPT_TOKENS_CACHED: &str = "Prompt tokens (cached)";
const COMPLETION_TOKENS: &str = "Completion tokens";

#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
/// Pricing of the OpenAI Chat Completions API.
pub struct ChatPricing {
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

impl ChatPricing {
    /// Creates a pricing from USD prices per million input, cached input, and output tokens.
    pub fn new(input_tokens: f64, cached_input_tokens: Option<f64>, output_tokens: f64) -> Self {
        ChatPricing {
            input_tokens,
            cached_input_tokens,
            output_tokens,
        }
    }

    /// Builds the usage lines for `usage`, carrying resolved unit prices when `pricing` is `Some`.
    fn usage_lines(usage: CompletionUsage, pricing: Option<&Self>) -> UsageLines {
        let cached_input_tokens = usage
            .prompt_tokens_details
            .and_then(|details| details.cached_tokens)
            .unwrap_or(0);
        let input_tokens = usage.prompt_tokens.saturating_sub(cached_input_tokens);

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
                UsageLine::per_million_tokens(cached_input_tokens, cached_price),
            ),
            (
                COMPLETION_TOKENS.into(),
                UsageLine::per_million_tokens(usage.completion_tokens, output_price),
            ),
        ])
    }
}

impl LlmUsage for CompletionUsage {}

impl LlmPricing for ChatPricing {
    type Usage = CompletionUsage;

    fn default_usage_lines(usage: CompletionUsage) -> UsageLines {
        Self::usage_lines(usage, None)
    }

    fn compute_usage_lines(&self, usage: CompletionUsage) -> UsageLines {
        Self::usage_lines(usage, Some(self))
    }
}
