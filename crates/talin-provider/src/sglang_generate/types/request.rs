use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;

use crate::sglang_generate::types::{
    AudioDataItem, Batched, ImageDataItem, MultimodalData, SamplingParams, SessionParams,
    VideoDataItem,
};

/// Request body for `POST /generate`.
///
/// Mirrors [`GenerateReqInput`] in the python source.
///
/// Exactly one of `text`, `input_ids`, or `input_embeds` should be set.
///
/// [`GenerateReqInput`]: https://github.com/sgl-project/sglang/blob/430418e2182606b5296b26c912df8f12ec9b380f/python/sglang/srt/managers/io_struct.py#L151-L304
#[skip_serializing_none]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct GenerateRequest {
    // --- Request id (inherited from BaseReq) ---
    /// Request ID(s). If omitted, generated during normalization.
    ///
    /// For batch requests, a string is expanded to per-item IDs using it as a prefix.
    pub rid: Option<Batched<String>>,
    /// Stable identity shared by requests in the same session.
    ///
    /// Unlike [`session_params`](Self::session_params), this does not alter or reconstruct the
    /// prompt.
    pub session_id: Option<String>,

    // --- Input (choose one) ---
    /// The input prompt. Can be a single prompt or a batch of prompts.
    pub text: Option<Batched<String>>,
    /// The token ids for text.
    ///
    /// Use C-loop validator to replace Pydantic per-element type check for efficiency.
    pub input_ids: Option<Batched<Vec<i64>>>,
    /// The embeddings for input_ids.
    pub input_embeds: Option<Batched<Vec<Vec<f32>>>>,

    // --- Multimodal inputs ---
    /// The image input. Can be an image instance, file name, URL, or base64 encoded string.
    pub image_data: Option<MultimodalData<ImageDataItem>>,
    /// The video input. Can be a file name, a url, or base64 encoded string.
    pub video_data: Option<MultimodalData<VideoDataItem>>,
    /// The audio input. Can be a file name, a url, or base64 encoded string.
    pub audio_data: Option<MultimodalData<AudioDataItem>>,
    /// Optional per-image hashes the caller has already computed (hex strings).
    ///
    /// - Single request: one hash per image.
    /// - Batch request: either one hash per request when each request has one image, or one list
    ///   of hashes per request.
    ///
    /// Intended for external KV routers that compute their own per-image hash for routing
    /// decisions and need sglang's prefix-cache key to align.
    /// When unset, behavior is unchanged (sglang hashes the processor feature tensor).
    pub mm_hashes: Option<Batched<Vec<String>>>,
    /// Whether to extract and process audio from video inputs.
    pub use_audio_in_video: bool,
    /// The modalities of the image data [image, multi-images, video]
    pub modalities: Option<Vec<String>>,

    // --- Sampling ---
    /// The [sampling params](SamplingParams).
    pub sampling_params: Option<Batched<SamplingParams>>,

    // --- Logprobs ---
    /// Whether to return logprobs.
    pub return_logprob: Option<Batched<bool>>,
    /// If return logprobs, the start location in the prompt for returning logprobs.
    ///
    /// By default, this value is "-1", which means it will only return logprobs for output tokens.
    pub logprob_start_len: Option<Batched<i64>>,
    /// If return logprobs, the number of top logprobs to return at each position.
    pub top_logprobs_num: Option<Batched<i64>>,
    /// If return logprobs, the token ids to return logprob for.
    pub token_ids_logprob: Option<Batched<Vec<i64>>>,
    /// Whether to detokenize tokens in text in the returned logprobs.
    pub return_text_in_logprobs: bool,

    // --- Output control ---
    /// Whether to stream output.
    pub stream: bool,
    /// Whether to log metrics for this request (e.g. health_generate calls do not log metrics).
    pub log_metrics: bool,
    /// Whether to return hidden states.
    pub return_hidden_states: Option<Batched<bool>>,
    /// Whether to return captured routed experts.
    pub return_routed_experts: bool,
    /// Absolute start position for returned routings; response covers `[routed_experts_start_len,
    /// seqlen - 1)`.
    ///
    /// Must be in \[0, prompt_tokens\]. 0 = full sequence.
    pub routed_experts_start_len: i64,
    /// Whether to return the top-k indexer outputs.
    pub return_indexer_topk: bool,

    /// (Internal) Whether to return bytes for image generation.
    pub return_bytes: bool,
    /// Whether to return entropy
    pub return_entropy: bool,
    /// Whether to return prompt token IDs without computing logprobs.
    pub return_prompt_token_ids: bool,

    // --- Session / LoRA / custom processors ---
    /// Session info for continual prompting.
    pub session_params: Option<SessionParams>,
    /// The path to the LoRA adaptors.
    pub lora_path: Option<Batched<Option<String>>>,
    /// Custom logit processor for advanced sampling control.
    pub custom_logit_processor: Option<Batched<Option<String>>>,

    // --- Disaggregated inference ---
    /// Host of the prefill server's bootstrap endpoint (for disaggregated prefill/decode).
    pub bootstrap_host: Option<Batched<Option<String>>>,
    /// Port of the prefill server's bootstrap endpoint (for disaggregated prefill/decode).
    pub bootstrap_port: Option<Batched<Option<i64>>>,
    /// Bootstrap room identifying this request's KV transfer (for disaggregated prefill/decode).
    pub bootstrap_room: Option<Batched<Option<i64>>>,
    /// Key pairing a prefill and decode worker (for disaggregated prefill/decode).
    pub bootstrap_pair_key: Option<Batched<Option<String>>>,
    /// Tensor-parallel size of the decode worker (for disaggregated prefill/decode).
    pub decode_tp_size: Option<Batched<Option<i64>>>,

    // --- Routing / control ---
    /// For DP routing — external router assigns a specific DP worker.
    pub routed_dp_rank: Option<i64>,
    /// For PD disagg — hint telling decode which prefill DP worker has the KV cache.
    pub disagg_prefill_dp_rank: Option<i64>,
    /// Routing key for routing-key schedule policy.
    pub routing_key: Option<String>,
    /// Conversation id used for tracking requests
    pub conversation_id: Option<String>,

    /// For background responses (OpenAI responses API).
    pub background: bool,
    /// Require reasoning for the request (hybrid reasoning model only).
    pub require_reasoning: bool,

    /// Priority for the request.
    pub priority: Option<i64>,
    /// Extra cache key for classifying the request (e.g. cache_salt).
    pub extra_key: Option<Batched<String>>,

    /// Whether to disallow logging for this request (e.g. due to ZDR).
    pub no_logs: bool,
    /// For custom metric labels.
    pub custom_labels: Option<HashMap<String, String>>,

    // --- Multimodal tiling ---
    /// Maximum number of dynamic patches (tiles) per image or video.
    pub max_dynamic_patch: Option<i64>,
    /// Minimum number of dynamic patches (tiles) per image or video.
    pub min_dynamic_patch: Option<i64>,
    /// Maximum number of dynamic patches (tiles) per image, overriding
    /// [`max_dynamic_patch`](Self::max_dynamic_patch).
    pub image_max_dynamic_patch: Option<i64>,
    /// Maximum number of dynamic patches (tiles) per video, overriding
    /// [`max_dynamic_patch`](Self::max_dynamic_patch).
    pub video_max_dynamic_patch: Option<i64>,

    /// Free-form image configuration (for Unlimited-OCR).
    pub images_config: Option<HashMap<String, Value>>,

    // --- Multi-item scoring ---
    /// Pre-computed delimiter indices: one list for a single request, a list of
    /// such lists when batching.
    pub multi_item_delimiter_indices: Option<Batched<Vec<i64>>>,
}
