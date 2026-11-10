use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum ReasoningEffort {
    None,
    Minimal,
    Low,
    Medium,
    High,
    Xhigh,
    Max,
}

impl ReasoningEffort {
    pub(crate) fn as_variant(&self) -> syn::Variant {
        match self {
            ReasoningEffort::None => syn::parse_quote! { None },
            ReasoningEffort::Minimal => syn::parse_quote! { Minimal },
            ReasoningEffort::Low => syn::parse_quote! { Low },
            ReasoningEffort::Medium => syn::parse_quote! { Medium },
            ReasoningEffort::High => syn::parse_quote! { High },
            ReasoningEffort::Xhigh => syn::parse_quote! { Xhigh },
            ReasoningEffort::Max => syn::parse_quote! { Max },
        }
    }
}
