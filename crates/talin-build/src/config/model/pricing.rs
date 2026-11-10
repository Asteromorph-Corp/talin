use serde::{Deserialize, Serialize};

/// The pricing specification of a model.
///
/// Used for `model_type`s: `"generic_chat"`, `"openai_chat"`, `"xai"`
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CommonPricing {
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
