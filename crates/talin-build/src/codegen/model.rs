use std::collections::BTreeMap;

use heck::ToSnakeCase;
use quote::format_ident;
use syn::parse_quote;

use crate::codegen::common::{model_expr, talin};
use crate::config::Model;

pub(crate) fn generate_models(models: &BTreeMap<String, Model>) -> syn::File {
    let talin = talin();
    let names = models
        .keys()
        .map(|id| id.to_snake_case())
        .collect::<Vec<_>>();
    let fns = models.iter().map(|(id, model)| generate_model(id, model));
    let arms = names.iter().map(|name| -> syn::Arm {
        let fn_name = format_ident!("{}", name);
        parse_quote! { #name => ::std::result::Result::Ok(Self::#fn_name().into()) }
    });

    parse_quote! {
        #[allow(dead_code)]
        pub struct Models;

        impl Models {
            /// The names accepted by [`Models::any`].
            #[allow(dead_code)]
            pub const NAMES: &'static [&'static str] = &[#(#names),*];

            /// Looks up a model preset by name at runtime.
            ///
            /// Fails with `UnknownModel` if no model has the given name; see [`Models::NAMES`] for the accepted names.
            #[allow(dead_code, clippy::match_single_binding)]
            pub fn any(
                name: &str,
            ) -> ::std::result::Result<
                #talin::llm::AnyPreset,
                #talin::llm::UnknownModel,
            > {
                match name {
                    #(#arms,)*
                    other => ::std::result::Result::Err(
                        #talin::llm::UnknownModel::new(other, Self::NAMES),
                    ),
                }
            }

            #(#fns)*
        }
    }
}

fn generate_model(id: &str, model: &Model) -> syn::ItemFn {
    let fn_name = format_ident!("{}", id.to_snake_case());
    let preset_ty = model.model_preset().to_ty();
    let model_expr = model_expr(model);

    parse_quote! {
        #[allow(dead_code)]
        pub fn #fn_name() -> #preset_ty {
            #model_expr
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::test_utils::{assert_item_eq, model_generic};

    /// A single model generates a constructor function using `model_expr`.
    #[rstest]
    fn test_generate_model(model_generic: Model) {
        let generate_model = generate_model("my_model", &model_generic);
        assert_item_eq(&generate_model, "model/generate_model.rs");
    }

    /// `Models` exposes `NAMES` and a runtime `any` lookup alongside the per-model constructors.
    #[rstest]
    fn test_generate_models(model_generic: Model) {
        let models = BTreeMap::from([("my_model".to_owned(), model_generic)]);
        assert_item_eq(&generate_models(&models), "model/generate_models.rs");
    }
}
