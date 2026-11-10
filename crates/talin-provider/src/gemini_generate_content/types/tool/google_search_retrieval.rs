use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// Tool to retrieve public web data for grounding, powered by Google.
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct GoogleSearchRetrieval {
    /// Specifies the dynamic retrieval configuration for the given source.
    pub dynamic_retrieval_config: Option<DynamicRetrievalConfig>,
}

impl GoogleSearchRetrieval {
    /// Creates a new empty GoogleSearchRetrieval.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the dynamic retrieval configuration.
    pub fn with_dynamic_retrieval_config(
        mut self,
        dynamic_retrieval_config: DynamicRetrievalConfig,
    ) -> Self {
        self.dynamic_retrieval_config = Some(dynamic_retrieval_config);
        self
    }
}

/// Describes the options to customize dynamic retrieval.
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct DynamicRetrievalConfig {
    /// The mode of the predictor to be used in dynamic retrieval.
    pub mode: Option<DynamicRetrievalMode>,
    /// The threshold to be used in dynamic retrieval. If not set, a system default value is used.
    pub dynamic_threshold: Option<f64>,
}

impl DynamicRetrievalConfig {
    /// Creates a new empty DynamicRetrievalConfig.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the mode of the predictor to be used in dynamic retrieval.
    pub fn with_mode(mut self, mode: DynamicRetrievalMode) -> Self {
        self.mode = Some(mode);
        self
    }

    /// Sets the threshold to be used in dynamic retrieval.
    pub fn with_dynamic_threshold(mut self, dynamic_threshold: f64) -> Self {
        self.dynamic_threshold = Some(dynamic_threshold);
        self
    }
}

/// The mode of the predictor to be used in dynamic retrieval.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum DynamicRetrievalMode {
    /// Always trigger retrieval.
    ModeUnspecified,
    /// Run retrieval only when system decides it is necessary.
    ModeDynamic,
}
