use syn::parse_quote;

use crate::codegen::agent::variables::Variables;
use crate::config::RuntimeVariable;
use crate::ty::is_array_like_type;

pub(super) fn input_display(input_symbol: &syn::Ident, variables: &Variables<'_>) -> syn::ItemImpl {
    let maybe_lifetime: Option<syn::AngleBracketedGenericArguments> =
        variables.has_lifetime().then(|| parse_quote! { <'_> });

    let write_fields = variables.all().into_iter().map(|var| -> Option<syn::Stmt> {
        let ident = var.get_ident();

        match var {
            RuntimeVariable::Input(var) if is_array_like_type(&var.type_) => {
                Some(parse_quote! { writeln!(f, "{}: {:?}", stringify!(#ident), self.#ident)?; })
            }
            RuntimeVariable::Input(_) | RuntimeVariable::Preset(_) => {
                Some(parse_quote! { writeln!(f, "{}: {}", stringify!(#ident), self.#ident)?; })
            }
            _ => None,
        }
    });

    parse_quote! {
        #[automatically_derived]
        impl std::fmt::Display for #input_symbol #maybe_lifetime {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                #(#write_fields)*
                Ok(())
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
    fn test_input_display(
        input_symbol: syn::Ident,
        variables: Vec<RuntimeVariable>,
        attachments: Attachments,
    ) {
        let variables = Variables::collect(&variables, &attachments, &ArtifactOptions::default());
        let input_display = input_display(&input_symbol, &variables);
        assert_item_eq(&input_display, "agent/input/input_display.rs");
    }
}
