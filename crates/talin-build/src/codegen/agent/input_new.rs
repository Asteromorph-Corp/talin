use quote::quote;
use syn::parse_quote;

use crate::codegen::agent::variables::Variables;
use crate::config::RuntimeVariable;
use crate::ty::lifetime_a;

pub(super) fn input_new(input_symbol: &syn::Ident, variables: &Variables<'_>) -> syn::ItemImpl {
    let input_variables = variables.inputs();

    let lifetime = lifetime_a();
    let maybe_lifetime = variables.has_lifetime().then_some(quote! { <#lifetime> });

    let args = input_variables.iter().map(|var| -> syn::FnArg {
        let field = &var.ident;
        let type_ = &var.type_;
        if variables.is_newtyped(field) {
            parse_quote! { #field: #type_ }
        } else {
            parse_quote! { #field: impl Into<#type_> }
        }
    });

    let field_initializers = variables.all().into_iter().map(|var| -> syn::FieldValue {
        let field: syn::Ident = var.get_ident();
        match var {
            RuntimeVariable::Input(_) if variables.is_newtyped(&field) => parse_quote! { #field },
            RuntimeVariable::Input(_) | RuntimeVariable::InputMedia(_) => {
                parse_quote! { #field: #field.into() }
            }
            RuntimeVariable::Preset(var) => {
                let value = var.get_expr();
                parse_quote! { #field: #value }
            }
            RuntimeVariable::Env(var) => parse_quote! {
                #field: std::env::var(#var)
                    .unwrap_or_else(|e| panic!("Failed to get env var '{}': {}", #var, e))
            },
        }
    });

    parse_quote! {
        impl #maybe_lifetime #input_symbol #maybe_lifetime {
            /// Create a new instance of the input struct.
            pub fn new(#(#args),*) -> Self {
                Self {
                    #(#field_initializers),*
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::config::Attachments;
    use crate::options::ArtifactOptions;
    use crate::test_utils::{assert_item_eq, attachments, input_symbol, variables};

    #[rstest]
    fn test_input_new(
        input_symbol: syn::Ident,
        variables: Vec<RuntimeVariable>,
        attachments: Attachments,
    ) {
        let variables = Variables::collect(&variables, &attachments, &ArtifactOptions::default());
        let input_new = input_new(&input_symbol, &variables);
        assert_item_eq(&input_new, "agent/input/input_new.rs");
    }
}
