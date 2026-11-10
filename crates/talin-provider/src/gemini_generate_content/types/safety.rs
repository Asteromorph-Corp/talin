use serde::{Deserialize, Serialize};

/// The category of a rating.
/// These categories cover various kinds of harms that developers may wish to adjust.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum HarmCategory {
    /// Category is unspecified.
    HarmCategoryUnspecified,
    /// PaLM - Negative or harmful comments targeting identity and/or protected attribute.
    HarmCategoryDerogatory,
    /// PaLM - Content that is rude, disrespectful, or profane.
    HarmCategoryToxicity,
    /// PaLM - Describes scenarios depicting violence against an individual or group, or general
    /// descriptions of gore.
    HarmCategoryViolence,
    /// PaLM - Contains references to sexual acts or other lewd content.
    HarmCategorySexual,
    /// PaLM - Promotes unchecked medical advice.
    HarmCategoryMedical,
    /// PaLM - Dangerous content that promotes, facilitates, or encourages harmful acts.
    HarmCategoryDangerous,
    /// Gemini - Harassment content.
    HarmCategoryHarassment,
    /// Gemini - Hate speech and content.
    HarmCategoryHateSpeech,
    /// Gemini - Sexually explicit content.
    HarmCategorySexuallyExplicit,
    /// Gemini - Dangerous content.
    HarmCategoryDangerousContent,
    /// Gemini - Content that may be used to harm civic integrity. DEPRECATED: use
    /// enableEnhancedCivicAnswers instead.
    HarmCategoryCivicIntegrity,
    /// Gemini - Prompts attempting to bypass or subvert the model's safety guidelines
    /// (jailbreak attempts).
    HarmCategoryJailbreak,
    /// Unknown category not represented by this enum.
    #[serde(other)]
    Unknown,
}

/// The probability that a piece of content is harmful.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum HarmProbability {
    /// Probability is unspecified.
    HarmProbabilityUnspecified,
    /// Content has a negligible chance of being unsafe.
    Negligible,
    /// Content has a low chance of being unsafe.
    Low,
    /// Content has a medium chance of being unsafe.
    Medium,
    /// Content has a high chance of being unsafe.
    High,
    /// Unknown probability not represented by this enum.
    #[serde(other)]
    Unknown,
}

/// Block at and beyond a specified harm probability.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum HarmBlockThreshold {
    /// Threshold is unspecified.
    HarmBlockThresholdUnspecified,
    /// Content with NEGLIGIBLE will be allowed.
    BlockLowAndAbove,
    /// Content with NEGLIGIBLE and LOW will be allowed.
    BlockMediumAndAbove,
    /// Content with NEGLIGIBLE, LOW, and MEDIUM will be allowed.
    BlockOnlyHigh,
    /// All content will be allowed.
    BlockNone,
    /// Turn off the safety filter.
    Off,
}

/// Safety rating for a piece of content.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct SafetyRating {
    /// The category for this rating.
    pub category: HarmCategory,
    /// The probability of harm for this content.
    pub probability: HarmProbability,
    /// Was this content blocked because of this rating?
    #[serde(default)]
    pub blocked: Option<bool>,
}

/// Safety setting, affecting the safety-blocking behavior.
/// Passing a safety setting for a category changes the allowed probability that content is blocked.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct SafetySetting {
    /// The category for this setting.
    pub category: HarmCategory,
    /// Controls the probability threshold at which harm is blocked.
    pub threshold: HarmBlockThreshold,
}

impl SafetySetting {
    /// Creates a new safety setting for the given category and threshold.
    pub fn new(category: HarmCategory, threshold: HarmBlockThreshold) -> Self {
        Self {
            category,
            threshold,
        }
    }
}
