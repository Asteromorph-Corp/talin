use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;

/// Serde helper: never skip a field, so an always-present response field is
/// still emitted when its value is `None` (serialized as JSON `null`).
fn never<T>(_: &T) -> bool {
    false
}

/// A single `/generate` response object.
///
/// A best-effort recreation of the [response dict](https://github.com/sgl-project/sglang/blob/430418e2182606b5296b26c912df8f12ec9b380f/python/sglang/srt/managers/tokenizer_manager.py#L1862-L2138) constructed in the python source.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateResponse {
    /// Generated text. Absent when `skip_tokenizer_init` is set; may be `null`
    /// for non-incremental intermediate streaming chunks.
    pub text: Option<String>,
    /// Generated token ids (full, or delta when incremental streaming).
    pub output_ids: Option<Vec<i64>>,
    /// Present only when the request set `return_prompt_token_ids = true`.
    pub prompt_token_ids: Option<Vec<i64>>,
    /// Metadata about the generation.
    pub meta_info: MetaInfo,
}

/// Metadata about a generation. Many fields are conditional on request flags
/// and server configuration; unknown/extra keys are captured in `extra`.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaInfo {
    /// Request id.
    pub id: String,
    /// `None` while still generating; otherwise a tagged reason. Always present
    /// in the server response (as `null` while generating).
    #[serde(skip_serializing_if = "never")]
    pub finish_reason: Option<FinishReason>,
    /// Number of prompt tokens.
    pub prompt_tokens: u64,
    /// Always present in the server response (may be `null`).
    #[serde(skip_serializing_if = "never")]
    pub weight_version: Option<String>,
    /// Number of times the request was retracted (preempted) by the scheduler.
    pub num_retractions: u64,

    /// Number of generated tokens.
    pub completion_tokens: Option<u64>,
    /// Number of reasoning tokens in the output.
    pub reasoning_tokens: Option<u64>,
    /// Number of prompt tokens served from the prefix cache.
    pub cached_tokens: Option<u64>,
    /// End-to-end latency in seconds (final response).
    pub e2e_latency: Option<f64>,

    // Scheduler load info (surfaced when available).
    /// Number of requests currently running on the scheduler.
    pub num_running_reqs: Option<u64>,
    /// Number of requests waiting in the scheduler queue.
    pub num_waiting_reqs: Option<u64>,

    /// Cached prompt tokens broken down by cache tier.
    pub cached_tokens_details: Option<HashMap<String, i64>>,
    /// Data-parallel rank that served the request.
    pub dp_rank: Option<i64>,

    // Multimodal prompt token counts (only when consumed).
    /// Number of image tokens in the prompt.
    pub image_tokens: Option<u64>,
    /// Number of audio tokens in the prompt.
    pub audio_tokens: Option<u64>,
    /// Number of video tokens in the prompt.
    pub video_tokens: Option<u64>,

    // Logprobs (present when return_logprob = true).
    /// Logprobs of the prompt tokens.
    pub input_token_logprobs: Option<Vec<LogprobEntry>>,
    /// Logprobs of the generated tokens.
    pub output_token_logprobs: Option<Vec<LogprobEntry>>,
    /// Top logprobs for each prompt token position.
    pub input_top_logprobs: Option<Vec<Option<Vec<LogprobEntry>>>>,
    /// Top logprobs for each generated token position.
    pub output_top_logprobs: Option<Vec<Option<Vec<LogprobEntry>>>>,
    /// Logprobs of the requested token ids for each prompt token position.
    pub input_token_ids_logprobs: Option<Vec<Option<Vec<LogprobEntry>>>>,
    /// Logprobs of the requested token ids for each generated token position.
    pub output_token_ids_logprobs: Option<Vec<Option<Vec<LogprobEntry>>>>,

    /// Present when `return_hidden_states = true`. One entry per generation step: a 2-D chunk for
    /// the prefill pass and a 1-D vector for each decode step.
    pub hidden_states: Option<Vec<HiddenStateChunk>>,
    /// Base64-encoded tensor (`return_routed_experts = true`).
    pub routed_experts: Option<String>,
    /// Base64-encoded tensor (`return_indexer_topk = true`).
    pub indexer_topk: Option<String>,

    // Speculative-decoding stats (when speculative decoding is enabled).
    /// Fraction of drafted tokens accepted.
    pub spec_accept_rate: Option<f64>,
    /// Average number of tokens accepted per verification step.
    pub spec_accept_length: Option<f64>,
    /// Number of correct draft tokens.
    pub spec_num_correct_drafts: Option<u64>,
    /// Number of proposed draft tokens.
    pub spec_num_proposed_drafts: Option<u64>,
    /// Number of verification steps.
    pub spec_verify_ct: Option<u64>,

    /// Any other keys (metrics time-stats, customized info, etc.).
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

/// Why generation stopped. `null` in JSON maps to the absence of this value
/// (see `Option<FinishReason>`).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum FinishReason {
    /// Hit an EOS/stop token, stop string, or stop regex.
    Stop {
        /// The stop string or token that matched, if any.
        #[serde(skip_serializing_if = "Option::is_none")]
        matched: Option<StopMatch>,
    },
    /// Reached `max_new_tokens`.
    Length {
        /// The number of tokens generated when the limit was reached.
        length: u64,
    },
    /// Request aborted.
    Abort {
        /// Human-readable abort message.
        message: String,
        /// HTTP status code of the abort, if any.
        #[serde(skip_serializing_if = "Option::is_none")]
        status_code: Option<i64>,
        /// Error type of the abort, if any.
        #[serde(skip_serializing_if = "Option::is_none")]
        err_type: Option<String>,
    },
}

/// What a `stop` finish matched: token id(s) or a string.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum StopMatch {
    /// A single matched stop token id.
    TokenId(i64),
    /// Multiple matched stop token ids.
    TokenIds(Vec<i64>),
    /// A matched stop string.
    Text(String),
}

/// A single logprob entry: `[logprob, token_id, token_text]`. `token_text` is
/// `null` unless `return_text_in_logprobs = true`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogprobEntry(pub f32, pub i64, pub Option<String>);

/// One entry of `MetaInfo::hidden_states`.
///
/// The prefill pass contributes a single 2-D `[num_tokens, hidden]` chunk;
/// each decode step then contributes a 1-D `[hidden]` vector for the token it
/// produced (see the `.tolist()` calls in
/// `python/sglang/srt/managers/scheduler_components/batch_result_processor.py`).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum HiddenStateChunk {
    /// A single token's hidden vector (1-D), emitted per decode step.
    Token(Vec<f32>),
    /// A chunk of token hidden vectors (2-D), emitted for the prefill pass.
    Tokens(Vec<Vec<f32>>),
}
