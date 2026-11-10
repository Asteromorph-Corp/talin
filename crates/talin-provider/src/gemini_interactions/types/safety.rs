use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::utils::string_enum;

string_enum! {
    /// The category of harm a [`SafetySetting`] applies to.
    ///
    /// These categories cover various kinds of harms that developers may wish to adjust.
    #[serde(rename_all = "snake_case")]
    pub enum HarmCategory {
        /// Hate speech and content.
        HateSpeech,

        /// Dangerous content.
        DangerousContent,

        /// Harassment content.
        Harassment,

        /// Sexually explicit content.
        SexuallyExplicit,

        /// An attempt to bypass the model's safety behaviour.
        Jailbreak,
    }
}

string_enum! {
    /// Block at and beyond a specified harm probability.
    #[serde(rename_all = "snake_case")]
    pub enum HarmBlockThreshold {
        /// Block content with a low harm probability or higher.
        BlockLowAndAbove,

        /// Block content with a medium harm probability or higher.
        BlockMediumAndAbove,

        /// Block content with a high harm probability.
        BlockOnlyHigh,

        /// Turn off the safety filter entirely.
        Off,
    }
}

string_enum! {
    /// The method by which harm is scored when deciding whether to block content.
    #[serde(rename_all = "snake_case")]
    pub enum HarmBlockMethod {
        /// Use both the probability and severity scores.
        Severity,

        /// Use the probability score.
        Probability,
    }
}

/// Safety setting, affecting the safety-blocking behavior.
///
/// Passing a safety setting for a category changes the allowed probability that content
/// is blocked.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct SafetySetting {
    /// The method for blocking content.
    ///
    /// If unset, the default behaviour is to use the probability score.
    #[cfg_attr(test, schema(nullable = false))]
    pub method: Option<HarmBlockMethod>,

    /// Controls the probability threshold at which harm is blocked.
    pub threshold: HarmBlockThreshold,

    /// The category of harm to be blocked.
    ///
    /// Sent as `type` on the wire.
    #[serde(rename = "type")]
    pub category: HarmCategory,
}

impl SafetySetting {
    /// Creates a new safety setting for the given category and threshold.
    pub fn new(category: HarmCategory, threshold: HarmBlockThreshold) -> Self {
        Self {
            method: None,
            threshold,
            category,
        }
    }

    /// Sets the method by which harm is scored.
    pub fn with_method(mut self, method: HarmBlockMethod) -> Self {
        self.method = Some(method);
        self
    }
}
