use indexmap::IndexSet;
use quote::quote;
use serde::{Deserialize, Serialize};
use serde_with::formats::PreferMany;
use serde_with::{OneOrMany, serde_as};
use syn::parse_quote;

use crate::codegen::common::talin;
use crate::config::model::pricing::CommonPricing;

/// The configuration for `model_type = "generic_chat"`.
#[serde_as]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct GenericChatConfig {
    #[serde(default)]
    #[serde_as(as = "OneOrMany<_, PreferMany>")]
    pub flavor: Vec<GenericChatFlavor>,
    pub drop_thought: Option<bool>,
    pub system_is_assistant: Option<bool>,
    pub pricing: Option<CommonPricing>,
}

/// The flavors available to `generic_chat` models.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum GenericChatFlavor {
    None,
    /// Enables the Qwen3 instructor to coax structured output out of Qwen3 models.
    Qwen3,
    /// Treats the system message as an assistant message. Necessary for models that do not support
    /// system messages.
    SystemIsAssistant,
}

impl GenericChatConfig {
    pub(crate) fn builder_options(&self) -> Vec<proc_macro2::TokenStream> {
        let with_pricing = self.pricing.as_ref().map(|p| {
            let expr = p.to_expr_chat();
            quote! { .with_pricing(#expr) }
        });
        let with_qwen3_instructor = self.flavors().contains(&GenericChatFlavor::Qwen3).then(|| {
            let talin = talin();
            quote! {
                .with_instructor(#talin::llm::instructor::Qwen3Instructor)
            }
        });
        [with_pricing, with_qwen3_instructor]
            .into_iter()
            .flatten()
            .collect()
    }

    pub(crate) fn flavors(&self) -> IndexSet<GenericChatFlavor> {
        let system_is_assistant = self
            .system_is_assistant
            .unwrap_or_default()
            .then_some(GenericChatFlavor::SystemIsAssistant);

        self.flavor
            .iter()
            .copied()
            .chain(system_is_assistant)
            .collect()
    }
}

impl GenericChatFlavor {
    pub(crate) fn to_strategy(self) -> syn::Type {
        let talin = talin();

        match self {
            GenericChatFlavor::SystemIsAssistant => parse_quote! {
                #talin::strategy::SystemToAssistantStrategy
            },
            _ => parse_quote! { () },
        }
    }
}

/// The configuration for `model_type = "openai"`.
#[serde_as]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct OpenAiChatConfig {
    #[serde(default)]
    #[serde_as(as = "OneOrMany<_, PreferMany>")]
    pub flavor: Vec<OpenAiChatFlavor>,
    pub drop_thought: Option<bool>,
    pub pricing: Option<CommonPricing>,
}

/// The flavors available to `openai` chat models.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum OpenAiChatFlavor {
    None,
    /// Removes the `strict` flag from tool definitions, for OpenAI-compatible
    /// providers that reject strict structured outputs.
    Unstrict,
    /// Makes the model submit final answer as a tool call.
    FinalAnswerTool,
}

impl OpenAiChatConfig {
    pub(crate) fn builder_options(&self) -> Vec<proc_macro2::TokenStream> {
        let with_pricing = self.pricing.as_ref().map(|p| {
            let expr = p.to_expr_chat();
            quote! { .with_pricing(#expr) }
        });
        [with_pricing].into_iter().flatten().collect()
    }

    pub(crate) fn flavors(&self) -> IndexSet<OpenAiChatFlavor> {
        self.flavor.iter().copied().collect()
    }
}

impl OpenAiChatFlavor {
    pub(crate) fn to_strategy(self) -> syn::Type {
        let talin = talin();

        match self {
            OpenAiChatFlavor::Unstrict => parse_quote! {
                #talin::strategy::RemoveStrictStrategy
            },
            OpenAiChatFlavor::FinalAnswerTool => parse_quote! {
                #talin::strategy::FinalAnswerToolStrategy
            },
            _ => parse_quote! { () },
        }
    }
}

impl CommonPricing {
    fn to_expr_chat(&self) -> syn::Expr {
        let talin = talin();
        let input_tokens = self.input_tokens;
        let cached_input_tokens: syn::Expr = match self.cached_input_tokens {
            Some(price) => parse_quote!(Some(#price)),
            None => parse_quote!(None),
        };
        let output_tokens = self.output_tokens;

        parse_quote! {
            #talin::llm::openai_chat::ChatPricing::new(
                #input_tokens,
                #cached_input_tokens,
                #output_tokens,
            )
        }
    }
}
