use serde::{Deserialize, Serialize};

/// The model that will complete your prompt.
///
/// See [models](https://ai.google.dev/gemini-api/docs/models) for additional details.
///
/// This is an open enum: known models are represented by [`ModelOption::Known`], while any
/// other value (for example a newly released model) round-trips through
/// [`ModelOption::Custom`].
#[cfg_attr(test, derive(utoipa::ToSchema), schema(as = KnownModel))]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(untagged)]
pub enum ModelOption {
    /// A model known to this crate at build time.
    Known(KnownModel),
    /// Any other model identifier.
    Custom(String),
}

/// Models known to this crate at build time.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
#[serde(rename = "ModelOption")]
pub enum KnownModel {
    /// The latest release of Gemini Pro.
    #[serde(rename = "gemini-pro-latest")]
    GeminiProLatest,
    /// The latest release of Gemini Flash.
    #[serde(rename = "gemini-flash-latest")]
    GeminiFlashLatest,
    /// The latest release of Gemini Flash-Lite.
    #[serde(rename = "gemini-flash-lite-latest")]
    GeminiFlashLiteLatest,

    /// A state-of-the-art multipurpose model, which excels at coding and complex reasoning tasks.
    #[serde(rename = "gemini-2.5-pro")]
    Gemini25Pro,
    /// A hybrid reasoning model that supports a 1M token context window and has thinking budgets.
    #[serde(rename = "gemini-2.5-flash")]
    Gemini25Flash,
    /// The smallest and most cost-effective model of the Gemini 2.5 generation, built for
    /// at-scale usage.
    #[serde(rename = "gemini-2.5-flash-lite")]
    Gemini25FlashLite,
    /// A cost-efficient model, for high-volume agentic tasks, translation, and simple data
    /// processing.
    #[serde(rename = "gemini-3.1-flash-lite")]
    Gemini31FlashLite,
    /// A cost-efficient model of the Gemini 3.5 generation, for high-volume tasks.
    #[serde(rename = "gemini-3.5-flash-lite")]
    Gemini35FlashLite,
    /// A model built for sustained performance in agentic and coding tasks.
    #[serde(rename = "gemini-3.5-flash")]
    Gemini35Flash,
    /// A model built for sustained performance in agentic and coding tasks, superseding
    /// [`KnownModel::Gemini35Flash`].
    #[serde(rename = "gemini-3.6-flash")]
    Gemini36Flash,
    /// Gemini 3.7 Flash
    #[serde(rename = "gemini-3.7-flash")]
    Gemini37Flash,
    /// Gemini 3.8 Flash
    #[serde(rename = "gemini-3.8-flash")]
    Gemini38Flash,

    /// A text-to-speech model of the Gemini 3.8 generation.
    #[serde(rename = "gemini-3.8-flash-tts")]
    Gemini38FlashTts,
    /// A cost-efficient text-to-speech model of the Gemini 3.8 generation.
    #[serde(rename = "gemini-3.8-flash-lite-tts")]
    Gemini38FlashLiteTts,

    /// A native image generation model, optimized for speed, flexibility, and contextual
    /// understanding.
    #[serde(rename = "gemini-2.5-flash-image")]
    Gemini25FlashImage,
    /// The Flash-tier image generation model of the Gemini 3.1 generation.
    #[serde(rename = "gemini-3.1-flash-image")]
    Gemini31FlashImage,
    /// The Flash-Lite-tier image generation model of the Gemini 3.1 generation.
    #[serde(rename = "gemini-3.1-flash-lite-image")]
    Gemini31FlashLiteImage,
    /// The Pro-tier image generation model of the Gemini 3 generation.
    #[serde(rename = "gemini-3-pro-image")]
    Gemini3ProImage,

    /// Gemini Nano Banana 2.1, an image generation model.
    #[serde(rename = "gemini-nano-banana-2.1")]
    GeminiNanoBanana21,
    /// The flagship music generation model, optimized for full-length songs with complex
    /// structural coherence.
    #[serde(rename = "lyria-3.5")]
    Lyria35,
    /// A high-performance multimodal model designed for fast, conversational video generation,
    /// editing, and cinematic control.
    #[serde(rename = "gemini-omni-1.1-flash")]
    GeminiOmni11Flash,
}

/// The agent to interact with.
///
/// This is an open enum: known agents are represented by [`AgentOption::Known`], while any
/// other value round-trips through [`AgentOption::Custom`].
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(untagged)]
#[cfg_attr(test, schema(as = KnownAgent))]
pub enum AgentOption {
    /// An agent known to this crate at build time.
    Known(KnownAgent),
    /// Any other agent identifier.
    Custom(String),
}

/// Agents known to this crate at build time.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
#[serde(rename = "AgentOption")]
pub enum KnownAgent {
    /// Gemini Deep Research Agent.
    #[serde(rename = "deep-research-preview-04-2026")]
    DeepResearchPreview042026,
    /// Gemini Deep Research Max Agent.
    #[serde(rename = "deep-research-max-preview-04-2026")]
    DeepResearchMaxPreview042026,
    /// Use the Antigravity managed agent to perform multi-step tasks that require reasoning,
    /// file operations, and tool use.
    #[serde(rename = "antigravity-preview-05-2026")]
    AntigravityPreview052026,
}

/// Configuration parameters for an agent interaction.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct AgentConfig {
    /// The agent this configures. (e.g. `deep-research`, `antigravity`).
    #[serde(rename = "type")]
    pub agent_type: String,

    /// The agent-specific configuration fields.
    #[serde(flatten)]
    pub extra: std::collections::BTreeMap<String, serde_json::Value>,
}

impl AgentConfig {
    /// Creates an empty configuration for the given agent type.
    pub fn new(agent_type: impl Into<String>) -> Self {
        Self {
            agent_type: agent_type.into(),
            extra: std::collections::BTreeMap::new(),
        }
    }

    /// Sets one agent-specific configuration field.
    pub fn with_field(
        mut self,
        name: impl Into<String>,
        value: impl Into<serde_json::Value>,
    ) -> Self {
        self.extra.insert(name.into(), value.into());
        self
    }
}

impl From<String> for ModelOption {
    fn from(value: String) -> Self {
        // Route to a known variant when the string matches, otherwise preserve it verbatim.
        match serde_json::from_value::<KnownModel>(serde_json::Value::String(value.clone())) {
            Ok(known) => ModelOption::Known(known),
            Err(_) => ModelOption::Custom(value),
        }
    }
}

impl From<&str> for ModelOption {
    fn from(value: &str) -> Self {
        ModelOption::from(value.to_string())
    }
}

impl From<String> for AgentOption {
    fn from(value: String) -> Self {
        match serde_json::from_value::<KnownAgent>(serde_json::Value::String(value.clone())) {
            Ok(known) => AgentOption::Known(known),
            Err(_) => AgentOption::Custom(value),
        }
    }
}

impl From<&str> for AgentOption {
    fn from(value: &str) -> Self {
        AgentOption::from(value.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_option_known_round_trip() {
        let model = ModelOption::from("gemini-3.5-flash");
        assert_eq!(model, ModelOption::Known(KnownModel::Gemini35Flash));
        assert_eq!(
            serde_json::to_string(&model).unwrap(),
            "\"gemini-3.5-flash\""
        );
    }

    #[test]
    fn model_option_custom_round_trip() {
        let model = ModelOption::from("gemini-9.0-ultra");
        assert_eq!(model, ModelOption::Custom("gemini-9.0-ultra".into()));
        assert_eq!(
            serde_json::to_string(&model).unwrap(),
            "\"gemini-9.0-ultra\""
        );
    }
}
