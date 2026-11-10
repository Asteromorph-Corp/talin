use indexmap::IndexSet;
use serde::{Deserialize, Serialize};
use syn::parse_quote;

use crate::codegen::common::talin;
use crate::config::model::chat::{GenericChatFlavor, OpenAiChatFlavor};

#[derive(
    Debug, Default, Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash, Serialize, Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum ModelType {
    #[default]
    #[serde(alias = "default", alias = "Default", alias = "GenericChat")]
    GenericChat,
    #[serde(
        rename = "openai_chat",
        alias = "openai",
        alias = "OpenAI",
        alias = "OpenAiChat"
    )]
    OpenAiChat,
    #[serde(
        rename = "openai_responses",
        alias = "responses",
        alias = "Responses",
        alias = "OpenAiResponses"
    )]
    OpenAiResponses,
    #[serde(
        alias = "claude",
        alias = "Claude",
        alias = "anthropic",
        alias = "Anthropic",
        alias = "ClaudeMessages"
    )]
    ClaudeMessages,
    #[serde(alias = "gemini", alias = "Gemini", alias = "GeminiGenerateContent")]
    GeminiGenerateContent,
    #[serde(alias = "GeminiInteractions")]
    GeminiInteractions,
    #[serde(alias = "xai", alias = "Xai", alias = "XaiResponses")]
    XaiResponses,
    #[serde(
        alias = "sglang",
        alias = "Sglang",
        alias = "SGLang",
        alias = "SglangGenerate"
    )]
    SglangGenerate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ModelPreset {
    GenericChat {
        flavor: IndexSet<GenericChatFlavor>,
        drop_thought: bool,
    },
    OpenAiChat {
        flavor: IndexSet<OpenAiChatFlavor>,
        drop_thought: bool,
    },
    OpenAiResponses,
    ClaudeMessages,
    GeminiGenerateContent,
    GeminiInteractions,
    XaiResponses,
    SglangGenerate {
        drop_thought: bool,
    },
}

impl ModelType {
    pub const ALL: [Self; 8] = [
        ModelType::GenericChat,
        ModelType::OpenAiChat,
        ModelType::OpenAiResponses,
        ModelType::ClaudeMessages,
        ModelType::GeminiGenerateContent,
        ModelType::GeminiInteractions,
        ModelType::XaiResponses,
        ModelType::SglangGenerate,
    ];

    pub fn to_ty(self) -> syn::Type {
        let talin = talin();

        match self {
            ModelType::GenericChat => parse_quote! {
                #talin::llm::GenericChat
            },
            ModelType::OpenAiChat => parse_quote! {
                #talin::llm::OpenAIChat
            },
            ModelType::OpenAiResponses => parse_quote! {
                #talin::llm::OpenAIResponses
            },
            ModelType::ClaudeMessages => parse_quote! {
                #talin::llm::ClaudeMessages
            },
            ModelType::GeminiGenerateContent => parse_quote! {
                #talin::llm::GeminiGenerateContent
            },
            ModelType::GeminiInteractions => parse_quote! {
                #talin::llm::GeminiInteractions
            },
            ModelType::XaiResponses => parse_quote! {
                #talin::llm::XaiResponses
            },
            ModelType::SglangGenerate => parse_quote! {
                #talin::llm::SglangGenerate
            },
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            ModelType::GenericChat => "generic_chat",
            ModelType::OpenAiChat => "openai_chat",
            ModelType::OpenAiResponses => "openai_responses",
            ModelType::ClaudeMessages => "claude_messages",
            ModelType::GeminiGenerateContent => "gemini_generate_content",
            ModelType::GeminiInteractions => "gemini_interactions",
            ModelType::XaiResponses => "xai_responses",
            ModelType::SglangGenerate => "sglang_generate",
        }
    }
}

impl ModelPreset {
    pub(crate) fn model_type(&self) -> ModelType {
        match self {
            ModelPreset::GenericChat { .. } => ModelType::GenericChat,
            ModelPreset::OpenAiChat { .. } => ModelType::OpenAiChat,
            ModelPreset::OpenAiResponses => ModelType::OpenAiResponses,
            ModelPreset::ClaudeMessages => ModelType::ClaudeMessages,
            ModelPreset::GeminiInteractions => ModelType::GeminiInteractions,
            ModelPreset::GeminiGenerateContent => ModelType::GeminiGenerateContent,
            ModelPreset::XaiResponses => ModelType::XaiResponses,
            ModelPreset::SglangGenerate { .. } => ModelType::SglangGenerate,
        }
    }

    pub(crate) fn model_ty(&self) -> syn::Type {
        self.model_type().to_ty()
    }

    pub(crate) fn strategy_ty(&self) -> syn::Type {
        let talin = talin();

        let strategies = match self {
            ModelPreset::GenericChat {
                flavor,
                drop_thought,
            } => {
                let drop_thought = drop_thought.then(|| {
                    parse_quote! {
                        #talin::strategy::DropThoughtStrategy
                    }
                });
                let flavors = flavor.iter().map(|f| f.to_strategy());

                drop_thought.into_iter().chain(flavors).collect::<Vec<_>>()
            }
            ModelPreset::OpenAiChat {
                flavor,
                drop_thought,
            } => {
                let drop_thought = drop_thought.then(|| {
                    parse_quote! {
                        #talin::strategy::DropThoughtStrategy
                    }
                });
                let flavors = flavor.iter().map(|f| f.to_strategy());

                drop_thought.into_iter().chain(flavors).collect::<Vec<_>>()
            }
            ModelPreset::SglangGenerate { drop_thought, .. } => drop_thought
                .then(|| {
                    parse_quote! {
                        #talin::strategy::DropThoughtStrategy
                    }
                })
                .into_iter()
                .collect::<Vec<_>>(),
            _ => Vec::new(),
        };

        if strategies.is_empty() {
            parse_quote! { () }
        } else if strategies.len() == 1 {
            let strategy = strategies.into_iter().next().expect("Element exists");
            parse_quote! { #strategy }
        } else {
            parse_quote! { (#(#strategies),*) }
        }
    }

    pub(crate) fn to_ty(&self) -> syn::Type {
        let talin = talin();

        let model_type = self.model_ty();
        let strategy_type = self.strategy_ty();

        parse_quote! {
            #talin::llm::LlmPreset<#model_type, #strategy_type>
        }
    }
}

impl std::fmt::Display for ModelType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
