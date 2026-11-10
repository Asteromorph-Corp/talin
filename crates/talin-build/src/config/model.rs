use std::convert::Infallible;

use quote::quote;
use serde::{Deserialize, Serialize};
use syn::parse_quote;

use crate::codegen::common::talin;
use crate::config::{
    FromConfig, Headers, InlineOrEnv, deserialize_api_base, deserialize_api_key, is_sensitive,
};

mod chat;
mod claude_messages;
mod gemini_generate_content;
mod gemini_interactions;
mod openai_responses;
mod provider_config;
mod sglang_generate;
mod xai_responses;

mod call_options;
mod model_type;
mod pricing;

pub(crate) use call_options::CallOptions;
pub(crate) use model_type::{ModelPreset, ModelType};
pub(crate) use provider_config::ProviderConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Model {
    #[serde(deserialize_with = "deserialize_api_base")]
    pub api_base: InlineOrEnv<String>,
    pub model: InlineOrEnv<String>,
    #[serde(
        default,
        alias = "api_key_env_var",
        deserialize_with = "deserialize_api_key"
    )]
    pub api_key: Option<InlineOrEnv<String>>,
    #[serde(default)]
    pub headers: Headers,

    #[serde(flatten)]
    pub call_options: CallOptions,

    #[serde(flatten)]
    pub provider_config: ProviderConfig,
}

impl Model {
    pub(crate) fn model_preset(&self) -> ModelPreset {
        self.provider_config.model_preset()
    }

    /// Arguments for the provider's `builder(..)` constructor.
    pub(crate) fn builder_args(&self) -> Option<proc_macro2::TokenStream> {
        match self.provider_config {
            ProviderConfig::SglangGenerate(_) => {
                let api_base = self.api_base.to_expr();
                Some(quote! { #api_base })
            }
            _ => None,
        }
    }

    /// Chained method calls attached to the provider's `builder(..)` constructor, e.g.
    /// `.with_api_base(..)`, `.with_api_key(..)`.
    pub(crate) fn builder_method_calls(&self) -> Vec<proc_macro2::TokenStream> {
        let mut options = vec![];
        match self.provider_config {
            // SGLang takes the API base in `builder(..)`, see `Self::builder_args`.
            ProviderConfig::SglangGenerate(_) => {}
            _ => {
                let api_base = self.api_base.to_expr();
                options.push(quote! { .with_api_base(#api_base) });
            }
        }
        if let Some(api_key) = &self.api_key {
            let api_key = api_key.to_expr();
            options.push(quote! { .with_api_key(#api_key) });
        }

        options
    }

    /// `.default_headers(..)` call for the generated `reqwest::ClientBuilder`, if any headers are
    /// configured.
    pub(crate) fn default_headers(&self) -> Option<proc_macro2::TokenStream> {
        if self.headers.0.is_empty() {
            return None;
        }

        let talin = talin();
        let header = quote! { #talin::__private::reqwest::header };
        let entries = self.headers.0.iter().map(|(name, value)| -> syn::Expr {
            let value_expr = value.to_expr();
            let sensitive = is_sensitive(name, value);
            parse_quote! {
                (
                    #header::HeaderName::from_static(#name),
                    {
                        let mut value = #header::HeaderValue::from_str(&#value_expr)
                            .unwrap_or_else(|e| panic!("Invalid value for header '{}': {}", #name, e));
                        value.set_sensitive(#sensitive);
                        value
                    },
                )
            }
        });

        Some(quote! {
            .default_headers(#header::HeaderMap::from_iter([#(#entries,)*]))
        })
    }

    pub(crate) fn strategy_expr(&self) -> syn::Expr {
        let s = self.provider_config.model_preset().strategy_ty();
        parse_quote! { <#s>::default() }
    }
}

impl FromConfig for Model {
    type Config = Self;
    type Error = Infallible;

    fn from_config(config: Self::Config) -> Result<Self, Self::Error> {
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::model::chat::{GenericChatFlavor, OpenAiChatFlavor};
    use crate::config::model::claude_messages::ClaudeMessagesConfig;

    fn parse(toml_str: &str) -> Model {
        toml::from_str(toml_str).unwrap()
    }

    const BASE: &str = r#"
        api_base = "https://api.openai.com/v1"
        model = "gpt-4o-mini"
    "#;

    #[test]
    fn new_syntax_openai_with_unstrict_flavor() {
        let model = parse(&format!(
            "{BASE}\nmodel_type = \"openai\"\nflavor = \"unstrict\"\n"
        ));
        let ProviderConfig::OpenAiChat(config) = model.provider_config else {
            panic!("Expected OpenAI chat config");
        };
        assert!(config.flavors().contains(&OpenAiChatFlavor::Unstrict));
    }

    #[test]
    fn new_syntax_generic_with_qwen3_flavor() {
        let model = parse(&format!(
            "{BASE}\nmodel_type = \"generic_chat\"\nflavor = \"qwen3\"\n"
        ));
        let ProviderConfig::GenericChat(config) = model.provider_config else {
            panic!("Expected OpenAI chat config");
        };
        assert!(config.flavors().contains(&GenericChatFlavor::Qwen3));
    }

    #[test]
    fn old_syntax_openai_unstrict() {
        let model = parse(&format!("{BASE}\nmodel_type = \"openai_unstrict\"\n"));
        let ProviderConfig::OpenAiChat(config) = model.provider_config else {
            panic!("Expected OpenAI chat config");
        };
        assert!(config.flavors().contains(&OpenAiChatFlavor::Unstrict));
    }

    #[test]
    fn old_syntax_qwen3() {
        let model = parse(&format!("{BASE}\nmodel_type = \"qwen3\"\n"));
        let ProviderConfig::GenericChat(config) = model.provider_config else {
            panic!("Expected OpenAI chat config");
        };
        assert!(config.flavors().contains(&GenericChatFlavor::Qwen3));
    }

    #[test]
    fn default_model_type_is_generic_chat() {
        let model = parse(BASE);
        let ProviderConfig::GenericChat(config) = model.provider_config else {
            panic!("Expected OpenAI chat config");
        };
        assert!(config.flavors().is_empty());
    }

    #[test]
    fn pricing_parses_for_chat_models() {
        let model = parse(&format!(
            "{BASE}\nmodel_type = \"openai_unstrict\"\npricing = {{ input_tokens = 0.15, output_tokens = 0.6 }}\n"
        ));
        let ProviderConfig::OpenAiChat(config) = model.provider_config else {
            panic!("Expected OpenAI chat config");
        };
        assert_eq!(config.pricing.as_ref().unwrap().input_tokens, 0.15);
        assert_eq!(config.pricing.as_ref().unwrap().output_tokens, 0.6);
        assert!(config.flavors().contains(&OpenAiChatFlavor::Unstrict));
    }

    #[test]
    fn anthropic_with_specific_pricing_shape() {
        let model = parse(&format!(
            "{BASE}\nmodel_type = \"anthropic\"\npricing = {{ input_tokens = 1.0, output_tokens = 5.0, cache_writes_5m = 1.25 }}\n"
        ));
        let ProviderConfig::ClaudeMessages(ClaudeMessagesConfig::Anthropic(config)) =
            model.provider_config
        else {
            panic!("Expected Anthropic config");
        };
        assert_eq!(config.pricing.as_ref().unwrap().input_tokens, 1.0);
        assert_eq!(config.pricing.as_ref().unwrap().output_tokens, 5.0);
        assert_eq!(config.pricing.as_ref().unwrap().cache_writes_5m, Some(1.25));
    }

    #[test]
    fn anthropic_options_on_anthropic_model() {
        let model = parse(&format!(
            "{BASE}\nmodel_type = \"anthropic\"\nanthropic_version = \"2023-06-01\"\nanthropic_beta = [\"token-counting\"]\n"
        ));
        let ProviderConfig::ClaudeMessages(ClaudeMessagesConfig::Anthropic(config)) =
            model.provider_config
        else {
            panic!("Expected Anthropic config");
        };
        assert_eq!(config.anthropic_version.unwrap(), "2023-06-01");
        assert_eq!(config.anthropic_beta.unwrap(), vec!["token-counting"]);
    }

    #[test]
    fn anthropic_options_on_non_anthropic_model_is_rejected() {
        let result: Result<Model, _> = toml::from_str(&format!(
            "{BASE}\nmodel_type = \"openai\"\nanthropic_version = \"2023-06-01\"\n"
        ));
        assert!(result.is_err());
    }

    #[test]
    fn flavor_on_non_chat_model_is_rejected() {
        let result: Result<Model, _> = toml::from_str(&format!(
            "{BASE}\nmodel_type = \"anthropic\"\nflavor = \"unstrict\"\n"
        ));
        assert!(result.is_err());
    }

    #[test]
    fn invalid_flavor_for_model_type_is_rejected() {
        // `qwen3` is not a valid flavor for an OpenAI chat model.
        let result: Result<Model, _> = toml::from_str(&format!(
            "{BASE}\nmodel_type = \"openai\"\nflavor = \"qwen3\"\n"
        ));
        assert!(result.is_err());
    }

    #[test]
    fn unknown_field_is_rejected() {
        // Neither a call option nor an option of any provider variant.
        let result: Result<Model, _> = toml::from_str(&format!(
            "{BASE}\nmodel_type = \"openai\"\nnot_a_real_field = 1\n"
        ));
        result.unwrap_err();
    }

    #[test]
    fn call_options_still_flatten_alongside_provider_config() {
        let model = parse(&format!(
            "{BASE}\nmodel_type = \"openai\"\ntemperature = 0.7\nmax_tokens = 256\n"
        ));
        assert_eq!(model.call_options.temperature, Some(0.7));
        assert_eq!(model.call_options.max_tokens, Some(256));
    }

    #[test]
    fn vertex_claude_model_type() {
        let model = parse(&format!(
            "{BASE}\nmodel_type = \"vertex_claude\"\nproject_id = \"my-project\"\n"
        ));
        let ProviderConfig::ClaudeMessages(ClaudeMessagesConfig::Vertex(config)) =
            model.provider_config
        else {
            panic!("Expected Vertex Claude config");
        };
        let InlineOrEnv::Inline(project_id) = config.project_id else {
            panic!("Expected inline project_id");
        };
        assert_eq!(project_id, "my-project");
    }

    #[test]
    fn claude_with_vertex_backend() {
        let model = parse(&format!(
            "{BASE}\nmodel_type = \"claude\"\nbackend = \"vertex\"\nproject_id = \"my-project\"\n"
        ));
        let ProviderConfig::ClaudeMessages(ClaudeMessagesConfig::Vertex(config)) =
            model.provider_config
        else {
            panic!("Expected Vertex Claude config");
        };
        let InlineOrEnv::Inline(project_id) = config.project_id else {
            panic!("Expected inline project_id");
        };
        assert_eq!(project_id, "my-project");
    }

    #[test]
    fn claude_with_anthropic_backend() {
        let model = parse(&format!(
            "{BASE}\nmodel_type = \"claude\"\nbackend = \"anthropic\"\n"
        ));
        let ProviderConfig::ClaudeMessages(ClaudeMessagesConfig::Anthropic(_)) =
            model.provider_config
        else {
            panic!("Expected Anthropic config");
        };
    }

    #[test]
    fn flavor_on_claude_model_is_rejected() {
        // `flavor` is a chat-model concept; Claude uses `backend` instead.
        let result: Result<Model, _> = toml::from_str(&format!(
            "{BASE}\nmodel_type = \"claude\"\nflavor = \"vertex\"\n"
        ));
        assert!(result.is_err());
    }

    #[test]
    fn generic_chat_system_is_assistant_adds_flavor() {
        let model = parse(&format!(
            "{BASE}\nmodel_type = \"generic_chat\"\nsystem_is_assistant = true\n"
        ));
        let ProviderConfig::GenericChat(config) = model.provider_config else {
            panic!("Expected generic chat config");
        };
        assert!(
            config
                .flavors()
                .contains(&GenericChatFlavor::SystemIsAssistant)
        );
    }

    #[test]
    fn gemini_interactions_with_per_modality_pricing() {
        let model = parse(&format!(
            "{BASE}\nmodel_type = \"gemini_interactions\"\n\
             [pricing]\ninput_tokens = 0.3\ncached_input_tokens = 0.075\n\
             audio_input_tokens = 1.0\noutput_tokens = 2.5\n"
        ));
        let ProviderConfig::GeminiInteractions(config) = model.provider_config else {
            panic!("Expected Gemini interactions config");
        };
        let pricing = config.pricing.as_ref().expect("pricing is set");
        assert_eq!(pricing.input_tokens, 0.3);
        assert_eq!(pricing.cached_input_tokens, Some(0.075));
        assert_eq!(pricing.audio_input_tokens, Some(1.0));
        assert_eq!(pricing.image_input_tokens, None);
        assert_eq!(pricing.output_tokens, 2.5);

        let options = config.builder_options();
        assert_eq!(options.len(), 1);
        let rendered = options[0].to_string();
        assert!(rendered.contains("InteractionsPricing"));
        assert!(rendered.contains("with_audio_input"));
        assert!(!rendered.contains("with_image_input"));
    }

    #[test]
    fn sglang_passes_api_base_to_builder() {
        let model = parse(&format!("{BASE}\nmodel_type = \"sglang\"\n"));
        let args = model
            .builder_args()
            .expect("sglang takes api_base in builder()");
        assert!(args.to_string().contains("api.openai.com"));
        assert!(
            model
                .builder_method_calls()
                .iter()
                .all(|o| !o.to_string().contains("with_api_base"))
        );

        let model = parse(&format!("{BASE}\nmodel_type = \"openai\"\n"));
        assert!(model.builder_args().is_none());
        assert!(
            model.builder_method_calls()[0]
                .to_string()
                .contains("with_api_base")
        );
    }

    /// The canonical `model_type` values are `provider_apiname`; the legacy spellings still
    /// deserialize to the same variant.
    #[test]
    fn model_type_canonical_and_legacy_spellings() {
        let cases = [
            ("generic_chat", ModelType::GenericChat),
            ("default", ModelType::GenericChat),
            ("openai_chat", ModelType::OpenAiChat),
            ("openai", ModelType::OpenAiChat),
            ("openai_responses", ModelType::OpenAiResponses),
            ("responses", ModelType::OpenAiResponses),
            ("claude_messages", ModelType::ClaudeMessages),
            ("anthropic", ModelType::ClaudeMessages),
            ("gemini_generate_content", ModelType::GeminiGenerateContent),
            ("gemini", ModelType::GeminiGenerateContent),
            ("gemini_interactions", ModelType::GeminiInteractions),
            ("xai_responses", ModelType::XaiResponses),
            ("xai", ModelType::XaiResponses),
            ("sglang_generate", ModelType::SglangGenerate),
            ("sglang", ModelType::SglangGenerate),
        ];
        for (spelling, expected) in cases {
            let parsed: ModelType = toml::from_str(&format!("v = \"{spelling}\""))
                .map(|w: Wrapper| w.v)
                .unwrap_or_else(|e| panic!("{spelling} should parse: {e}"));
            assert_eq!(parsed, expected, "{spelling}");
        }

        // The canonical spelling round-trips.
        for model_type in ModelType::ALL {
            let serialized = toml::to_string(&Wrapper { v: model_type }).unwrap();
            assert!(
                serialized.contains(model_type.as_str()),
                "{model_type} should serialize as {}",
                model_type.as_str()
            );
        }
    }

    /// `model_type` and `ProviderConfig` must agree on what each spelling means, otherwise
    /// per-model-type overrides would silently never apply.
    #[test]
    fn model_type_agrees_with_provider_config() {
        for spelling in [
            "generic_chat",
            "default",
            "openai_chat",
            "openai",
            "openai_responses",
            "responses",
            "claude_messages",
            "anthropic",
            "gemini_generate_content",
            "gemini",
            "gemini_interactions",
            "xai_responses",
            "xai",
            "sglang_generate",
            "sglang",
        ] {
            let model = parse(&format!("{BASE}\nmodel_type = \"{spelling}\"\n"));
            let from_provider = model.provider_config.model_preset().model_type();
            let direct: ModelType = toml::from_str(&format!("v = \"{spelling}\""))
                .map(|w: Wrapper| w.v)
                .unwrap();
            assert_eq!(from_provider, direct, "{spelling}");
        }
    }

    #[derive(Serialize, Deserialize)]
    struct Wrapper {
        v: ModelType,
    }
}
