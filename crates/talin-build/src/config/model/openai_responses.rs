use quote::quote;
use serde::{Deserialize, Serialize};
use syn::parse_quote;

use crate::codegen::common::talin;
use crate::config::model::pricing::CommonPricing;

/// The configuration for the `model_type = "openai_responses"`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct OpenAiResponsesConfig {
    pricing: Option<CommonPricing>,
}

impl OpenAiResponsesConfig {
    pub(crate) fn builder_options(&self) -> Vec<proc_macro2::TokenStream> {
        let with_pricing = self.pricing.as_ref().map(|p| {
            let expr = p.to_expr_responses();
            quote! { .with_pricing(#expr) }
        });
        [with_pricing].into_iter().flatten().collect()
    }
}

impl CommonPricing {
    fn to_expr_responses(&self) -> syn::Expr {
        let input_tokens = self.input_tokens;
        let cached_input_tokens: syn::Expr = match self.cached_input_tokens {
            Some(cached_input_tokens) => parse_quote!(Some(#cached_input_tokens)),
            None => parse_quote!(None),
        };
        let output_tokens = self.output_tokens;

        let talin = talin();
        parse_quote! {
            #talin::llm::openai_responses::ResponsesPricing::new(
                #input_tokens,
                #cached_input_tokens,
                #output_tokens,
            )
        }
    }
}
