use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use time::OffsetDateTime;

/// The status of the underlying model. This is used to indicate the stage of the underlying model
/// and the retirement time if applicable.
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ModelStatus {
    /// The stage of the underlying model.
    pub model_stage: Option<ModelStage>,

    /// The time at which the model will be retired. Uses RFC 3339 format.
    pub retirement_time: Option<OffsetDateTime>,

    /// A message explaining the model status.
    pub message: Option<String>,
}

/// Defines the stage of the underlying model.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum ModelStage {
    /// Unspecified model stage.
    ModelStageUnspecified,
    /// The underlying model is subject to lots of tunings.
    UnstableExperimental,
    /// Models in this stage are for experimental purposes only.
    Experimental,
    /// Models in this stage are more mature than experimental models.
    Preview,
    /// Models in this stage are considered stable and ready for production use.
    Stable,
    /// If the model is on this stage, it means that this model is on the path to deprecation in
    /// near future. Only existing customers can use this model.
    Legacy,
    /// Models in this stage are deprecated. These models cannot be used.
    Deprecated,
    /// Models in this stage are retired. These models cannot be used.
    Retired,
}
