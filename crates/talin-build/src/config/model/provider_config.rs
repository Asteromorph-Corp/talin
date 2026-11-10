use serde::de::Error;
use serde::{Deserialize, Serialize};

use crate::config::ModelPreset;
use crate::config::model::chat::{
    GenericChatConfig, GenericChatFlavor, OpenAiChatConfig, OpenAiChatFlavor,
};
use crate::config::model::claude_messages::{ClaudeMessagesConfig, VertexClaudeMessagesConfig};
use crate::config::model::gemini_generate_content::GeminiGenerateContentConfig;
use crate::config::model::gemini_interactions::GeminiInteractionsConfig;
use crate::config::model::openai_responses::OpenAiResponsesConfig;
use crate::config::model::sglang_generate::SglangGenerateConfig;
use crate::config::model::xai_responses::XaiResponsesConfig;

/// A model's provider configuration, keyed by the `provider_apiname` `model_type`.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "model_type", rename_all = "snake_case")]
pub(crate) enum ProviderConfig {
    GenericChat(GenericChatConfig),
    #[serde(rename = "openai_chat")]
    OpenAiChat(OpenAiChatConfig),
    #[serde(rename = "openai_responses")]
    OpenAiResponses(OpenAiResponsesConfig),
    ClaudeMessages(ClaudeMessagesConfig),
    GeminiGenerateContent(GeminiGenerateContentConfig),
    GeminiInteractions(GeminiInteractionsConfig),
    XaiResponses(XaiResponsesConfig),
    SglangGenerate(SglangGenerateConfig),
}

impl ProviderConfig {
    pub(crate) fn model_preset(&self) -> ModelPreset {
        match self {
            ProviderConfig::GenericChat(config) => ModelPreset::GenericChat {
                flavor: config.flavors(),
                drop_thought: config.drop_thought.unwrap_or(true),
            },
            ProviderConfig::OpenAiChat(config) => ModelPreset::OpenAiChat {
                flavor: config.flavors(),
                drop_thought: config.drop_thought.unwrap_or(true),
            },
            ProviderConfig::OpenAiResponses(_) => ModelPreset::OpenAiResponses,
            ProviderConfig::ClaudeMessages(_) => ModelPreset::ClaudeMessages,
            ProviderConfig::GeminiGenerateContent(_) => ModelPreset::GeminiGenerateContent,
            ProviderConfig::GeminiInteractions(_) => ModelPreset::GeminiInteractions,
            ProviderConfig::XaiResponses(_) => ModelPreset::XaiResponses,
            ProviderConfig::SglangGenerate(config) => ModelPreset::SglangGenerate {
                drop_thought: config.drop_thought.unwrap_or(true),
            },
        }
    }

    /// The provider-specific builder calls for this model: pricing plus any
    /// options unique to the provider. The top-level `api_base` / `api_key` are
    /// applied by the caller, since every builder accepts them.
    pub(crate) fn builder_options(&self) -> Vec<proc_macro2::TokenStream> {
        match self {
            ProviderConfig::GenericChat(config) => config.builder_options(),
            ProviderConfig::OpenAiChat(config) => config.builder_options(),
            ProviderConfig::OpenAiResponses(config) => config.builder_options(),
            ProviderConfig::ClaudeMessages(config) => config.builder_options(),
            ProviderConfig::GeminiGenerateContent(config) => config.builder_options(),
            ProviderConfig::GeminiInteractions(config) => config.builder_options(),
            ProviderConfig::XaiResponses(config) => config.builder_options(),
            ProviderConfig::SglangGenerate(config) => config.builder_options(),
        }
    }
}

impl Default for ProviderConfig {
    fn default() -> Self {
        ProviderConfig::GenericChat(GenericChatConfig::default())
    }
}

/// The wire form of [`ProviderConfig`].
///
/// Variant names are the canonical `provider_apiname` `model_type` values; the `alias`es are the
/// pre-`provider_apiname` spellings, kept for backwards compatibility and intentionally
/// undocumented. `Qwen3` / `OpenAiUnstrict` / `VertexClaude` are likewise legacy shorthands that
/// now desugar into a `provider_apiname` variant plus a flavor or backend.
#[derive(Deserialize)]
#[serde(tag = "model_type", rename_all = "snake_case")]
enum RawProviderConfig {
    #[serde(alias = "default", alias = "Default", alias = "GenericChat")]
    GenericChat(GenericChatConfig),
    #[serde(alias = "Qwen3")]
    Qwen3(GenericChatConfig),
    #[serde(
        rename = "openai_chat",
        alias = "openai",
        alias = "OpenAI",
        alias = "OpenAiChat"
    )]
    OpenAiChat(OpenAiChatConfig),
    #[serde(
        rename = "openai_unstrict",
        alias = "OpenAIUnstrict",
        alias = "OpenAiUnstrict"
    )]
    OpenAiUnstrict(OpenAiChatConfig),
    #[serde(
        rename = "openai_responses",
        alias = "responses",
        alias = "Responses",
        alias = "OpenAiResponses"
    )]
    OpenAiResponses(OpenAiResponsesConfig),
    #[serde(
        alias = "claude",
        alias = "Claude",
        alias = "anthropic",
        alias = "Anthropic"
    )]
    ClaudeMessages(ClaudeMessagesConfig),
    #[serde(alias = "gemini", alias = "Gemini")]
    GeminiGenerateContent(GeminiGenerateContentConfig),
    GeminiInteractions(GeminiInteractionsConfig),
    #[serde(alias = "xai", alias = "Xai")]
    XaiResponses(XaiResponsesConfig),
    #[serde(alias = "sglang", alias = "Sglang", alias = "SGLang")]
    SglangGenerate(SglangGenerateConfig),
    #[serde(alias = "VertexClaude")]
    VertexClaude(VertexClaudeMessagesConfig),
}

impl From<RawProviderConfig> for ProviderConfig {
    fn from(raw: RawProviderConfig) -> Self {
        match raw {
            RawProviderConfig::GenericChat(config) => ProviderConfig::GenericChat(config),
            RawProviderConfig::Qwen3(mut config) => {
                config.flavor.push(GenericChatFlavor::Qwen3);
                ProviderConfig::GenericChat(config)
            }
            RawProviderConfig::OpenAiChat(config) => ProviderConfig::OpenAiChat(config),
            RawProviderConfig::OpenAiUnstrict(mut config) => {
                config.flavor.push(OpenAiChatFlavor::Unstrict);
                ProviderConfig::OpenAiChat(config)
            }
            RawProviderConfig::OpenAiResponses(config) => ProviderConfig::OpenAiResponses(config),
            RawProviderConfig::ClaudeMessages(config) => ProviderConfig::ClaudeMessages(config),
            RawProviderConfig::GeminiGenerateContent(config) => {
                ProviderConfig::GeminiGenerateContent(config)
            }
            RawProviderConfig::GeminiInteractions(config) => {
                ProviderConfig::GeminiInteractions(config)
            }
            RawProviderConfig::XaiResponses(config) => ProviderConfig::XaiResponses(config),
            RawProviderConfig::SglangGenerate(config) => ProviderConfig::SglangGenerate(config),
            RawProviderConfig::VertexClaude(config) => {
                ProviderConfig::ClaudeMessages(ClaudeMessagesConfig::Vertex(config))
            }
        }
    }
}

impl<'de> Deserialize<'de> for ProviderConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let mut value = toml::Value::deserialize(deserializer)?;
        if let Some(table) = value.as_table_mut()
            && !table.contains_key("model_type")
        {
            table.insert("model_type".to_owned(), toml::Value::from("generic_chat"));
        }

        let raw = RawProviderConfig::deserialize(value).map_err(D::Error::custom)?;
        Ok(ProviderConfig::from(raw))
    }
}
