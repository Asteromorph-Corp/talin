use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::sglang_generate::types::OneOrMany;

/// Sampling parameters.
///
/// Mirrors [`SamplingParams`] in the python library.
///
/// [`SamplingParams`]: https://github.com/sgl-project/sglang/blob/430418e2182606b5296b26c912df8f12ec9b380f/python/sglang/srt/sampling/sampling_params.py#L75-L121
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SamplingParams {
    /// The maximum output length measured in tokens.
    pub max_new_tokens: u32,
    /// Forces the model to generate at least min_new_tokens until a stop word or EOS token is
    /// sampled.
    pub min_new_tokens: u32,

    // --- Stopping criteria ---
    /// One or multiple stop words. Generation will stop if one of these words is sampled.
    pub stop: Option<OneOrMany<String>>,
    /// Provide stop words in the form of token IDs. Generation will stop if one of these token IDs
    /// is sampled.
    pub stop_token_ids: Option<Vec<i64>>,
    /// Stop when hitting any of the regex patterns in this list.
    pub stop_regex: Option<OneOrMany<String>>,

    // --- Sampling parameters ---
    /// Temperature when sampling the next token. temperature = 0 corresponds to greedy sampling, a
    /// higher temperature leads to more diversity.
    pub temperature: f32,
    /// Top-p sampling parameter.
    ///
    /// Top-p sampling selects tokens from the smallest sorted set whose cumulative probability
    /// exceeds `top_p`.
    /// When `top_p` = 1, this reduces to unrestricted sampling from all tokens.
    pub top_p: f32,
    /// Top-k sampling parameter.
    ///
    /// Top-k sampling randomly selects from the k highest-probability tokens.
    pub top_k: i32,
    /// Min-p sampling parameter.
    ///
    /// Min-p sampling selects from tokens with probability larger than `min_p` times the
    /// probability of the most likely token.
    pub min_p: f32,

    /// Penalizes tokens based on their frequency in generation so far.
    ///
    /// Must be between -2 and 2, where negative numbers encourage repeatment of tokens and
    /// positive number encourages sampling of new tokens.
    ///
    /// The scaling of penalization grows linearly with each appearance of a token.
    pub frequency_penalty: f32,
    /// Penalizes tokens if they appeared in the generation so far.
    ///
    /// Must be between -2 and 2, where negative numbers encourage repeatment of tokens and
    /// positive number encourages sampling of new tokens.
    ///
    /// The scaling of the penalization is constant if a token occurred.
    pub presence_penalty: f32,
    /// Scales the logits of previously generated tokens to discourage (values > 1) or encourage
    /// (values < 1) repetition.
    ///
    /// Valid range is (0, 2]; 1.0 leaves probabilities unchanged.
    pub repetition_penalty: f32,

    // --- Constrained decoding ---
    /// JSON schema for structured outputs.
    pub json_schema: Option<String>,
    /// Regex for structured outputs.
    pub regex: Option<String>,
    /// EBNF for structured outputs.
    pub ebnf: Option<String>,
    /// The structural tag for structured outputs.
    pub structural_tag: Option<String>,

    // --- Other options
    /// Specifies the number of output sequences to generate per request.
    ///
    /// (Generating multiple outputs in one request (n > 1) is discouraged; repeating the same
    /// prompts several times offers better control and efficiency.)
    pub n: u32,
    /// Don’t stop generation when EOS token is sampled.
    pub ignore_eos: bool,
    /// Remove special tokens during decoding.
    pub skip_special_tokens: bool,
    /// Whether or not to add spaces between special tokens during detokenization.
    pub spaces_between_special_tokens: bool,
    /// Don’t trim stop words or EOS token from the generated text.
    pub no_stop_trim: bool,
    /// Used when employing custom logit processor.
    pub custom_params: Option<HashMap<String, Value>>,

    // --- Undocumented fields (present in python implementation) ---
    /// Interval (in tokens) at which streamed output is emitted.
    pub stream_interval: Option<i64>,
    /// Per-token bias added to logits, keyed by token id.
    pub logit_bias: Option<HashMap<String, f32>>,
    /// Random seed for sampling.
    pub sampling_seed: Option<i64>,
}

impl Default for SamplingParams {
    fn default() -> Self {
        Self {
            max_new_tokens: 128,
            min_new_tokens: 0,
            stop: None,
            stop_token_ids: None,
            stop_regex: None,
            temperature: 1.0,
            top_p: 1.0,
            top_k: -1,
            min_p: 0.0,
            frequency_penalty: 0.0,
            presence_penalty: 0.0,
            repetition_penalty: 1.0,
            n: 1,
            json_schema: None,
            regex: None,
            ebnf: None,
            structural_tag: None,
            ignore_eos: false,
            skip_special_tokens: true,
            spaces_between_special_tokens: true,
            no_stop_trim: false,
            custom_params: None,
            stream_interval: None,
            logit_bias: None,
            sampling_seed: None,
        }
    }
}
