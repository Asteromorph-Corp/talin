use syn::parse_quote;

use crate::codegen::agent::variables::Variables;
use crate::ty::{is_cow_str_type, lifetime_a};

pub(super) fn input_from(
    input_symbol: &syn::Ident,
    variables: &Variables<'_>,
) -> Option<syn::ItemImpl> {
    let input_variables = variables.inputs();

    match input_variables.len() {
        0 => Some(parse_quote! {
            #[automatically_derived]
            impl From<()> for #input_symbol {
                fn from(_: ()) -> Self {
                    Self::new()
                }
            }
        }),
        1 if is_cow_str_type(&input_variables[0].type_) => {
            let lifetime = lifetime_a();
            Some(parse_quote! {
                #[automatically_derived]
                impl<#lifetime> From<&#lifetime str> for #input_symbol<#lifetime> {
                    fn from(input: &#lifetime str) -> Self {
                        Self::new(input)
                    }
                }
            })
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use quote::format_ident;
    use rstest::rstest;

    use super::*;
    use crate::config::{Attachments, InputVariable, RuntimeVariable};
    use crate::options::ArtifactOptions;
    use crate::test_utils::{assert_item_eq, attachments, input_symbol, variables};

    /// With a single string-like input variable, a `From<&str>` impl is generated.
    #[rstest]
    #[case::unit(&[], "agent/input/input_from.unit.rs")]
    #[case::single(
        &[RuntimeVariable::Input(Box::new(InputVariable {
            ident: format_ident!("input"),
            type_: parse_quote!(std::borrow::Cow<'a, str>),
        }))],
        "agent/input/input_from.str.rs"
    )]
    fn test_input_from(
        input_symbol: syn::Ident,
        #[case] vars: &[RuntimeVariable],
        #[case] fixture_path: &str,
    ) {
        let variables =
            Variables::collect(vars, &Attachments::default(), &ArtifactOptions::default());
        let input_from = input_from(&input_symbol, &variables).expect("expected a `From` impl");
        assert_item_eq(&input_from, fixture_path);
    }

    /// With multiple input variables, no `From` impl is generated.
    #[rstest]
    fn test_input_from_none(
        input_symbol: syn::Ident,
        variables: Vec<RuntimeVariable>,
        attachments: Attachments,
    ) {
        let variables = Variables::collect(&variables, &attachments, &ArtifactOptions::default());
        assert!(input_from(&input_symbol, &variables).is_none());
    }
}
