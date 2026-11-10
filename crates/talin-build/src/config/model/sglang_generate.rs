use quote::quote;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use syn::parse_quote;

use crate::codegen::common::talin;
use crate::config::model::pricing::CommonPricing;

/// The configuration for `model_type = "sglang"`.
#[serde_as]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SglangGenerateConfig {
    pub drop_thought: Option<bool>,
    pub pricing: Option<CommonPricing>,
}

impl SglangGenerateConfig {
    pub(crate) fn builder_options(&self) -> Vec<proc_macro2::TokenStream> {
        let with_pricing = self.pricing.as_ref().map(|p| {
            let expr = p.to_expr_sglang();
            quote! { .with_pricing(#expr) }
        });
        [with_pricing].into_iter().flatten().collect()
    }
}

impl CommonPricing {
    fn to_expr_sglang(&self) -> syn::Expr {
        let input_tokens = self.input_tokens;
        let cached_input_tokens: syn::Expr = match self.cached_input_tokens {
            Some(price) => parse_quote!(Some(#price)),
            None => parse_quote!(None),
        };
        let output_tokens = self.output_tokens;

        let talin = talin();
        parse_quote! {
            #talin::llm::sglang_generate::GeneratePricing::new(
                #input_tokens,
                #cached_input_tokens,
                #output_tokens
            )
        }
    }
}
