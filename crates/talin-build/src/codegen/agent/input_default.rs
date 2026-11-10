use quote::quote;
use syn::parse_quote;

use crate::codegen::agent::variables::Variables;
use crate::ty::lifetime_a;

pub(super) fn input_default(
    input_symbol: &syn::Ident,
    variables: &Variables<'_>,
) -> Option<syn::ItemImpl> {
    if !variables.inputs().is_empty() {
        return None;
    }

    let lifetime = lifetime_a();
    let maybe_lifetime = variables.has_lifetime().then_some(quote! { <#lifetime> });
    Some(parse_quote! {
        impl #maybe_lifetime Default for #input_symbol #maybe_lifetime {
            fn default() -> Self {
                Self::new()
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::config::{Attachments, RuntimeVariable};
    use crate::options::ArtifactOptions;
    use crate::test_utils::{assert_item_eq, attachments, input_symbol, variables};

    #[rstest]
    fn test_input_default_without_inputs(input_symbol: syn::Ident) {
        let variables =
            Variables::collect(&[], &Attachments::default(), &ArtifactOptions::default());
        let input_default =
            input_default(&input_symbol, &variables).expect("expected a `Default` impl");
        assert_item_eq(&input_default, "agent/input/input_default.rs");
    }

    /// When the agent has input variables, no `Default` impl is generated.
    #[rstest]
    fn test_input_default_with_inputs(
        input_symbol: syn::Ident,
        variables: Vec<RuntimeVariable>,
        attachments: Attachments,
    ) {
        let variables = Variables::collect(&variables, &attachments, &ArtifactOptions::default());
        assert!(input_default(&input_symbol, &variables).is_none());
    }
}
