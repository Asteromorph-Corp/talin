use syn::parse_quote;

use crate::codegen::common::talin;
use crate::config::ModelType;

/// The `AnyPreset`/`AnyAgent` variant for a backend, which is named after the backend's type.
fn variant_ident(model_type: ModelType) -> syn::Ident {
    let syn::Type::Path(path) = model_type.to_ty() else {
        unreachable!("model types are paths");
    };
    path.path
        .segments
        .last()
        .expect("model type path is not empty")
        .ident
        .clone()
}

pub(super) fn fn_any(input_ctor: &syn::Type, output_ctor: &syn::Type) -> syn::ItemFn {
    let talin = talin();
    let arms = ModelType::ALL
        .into_iter()
        .map(variant_ident)
        .map(|variant| -> syn::Arm {
            parse_quote! {
                #talin::llm::AnyPreset::#variant(preset) => {
                    #talin::agent::AnyAgent::#variant(Self::new(*preset).await)
                }
            }
        });

    parse_quote! {
        pub async fn any(
            preset: impl ::std::convert::Into<#talin::llm::AnyPreset>,
        ) -> #talin::agent::AnyAgent<#input_ctor, #output_ctor> {
            match preset.into() {
                #(#arms)*
                _ => unreachable!("`AnyPreset` has a variant that `fn_any` does not handle"),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::test_utils::{assert_item_eq, input_ctor, output_ctor};

    #[rstest]
    fn test_fn_any(input_ctor: syn::Type, output_ctor: syn::Type) {
        let fn_any = fn_any(&input_ctor, &output_ctor);
        assert_item_eq(&fn_any, "agent/fn_any.rs");
    }
}
