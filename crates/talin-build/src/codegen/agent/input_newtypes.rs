use heck::ToSnakeCase;
use quote::format_ident;
use syn::parse_quote;

use crate::codegen::agent::variables::Variables;

/// The newtypes wrapping string inputs, or `None` unless `use_newtype_inputs` is enabled.
///
/// They live in a public module named after the input struct (`my_agent_input` for
/// `MyAgentInput`), and are only imported privately into the agent module. The agent index
/// glob-re-exports every agent module, so exporting the newtypes themselves would make them
/// collide across agents and with user types.
///
/// The newtypes deliberately have no `From<String>`/`From<&str>`, so that callers must name the
/// newtype (`Language::new("en")`) and cannot silently swap two string arguments with `.into()`.
pub(super) fn input_newtypes(
    input_symbol: &syn::Ident,
    variables: &Variables<'_>,
) -> Option<syn::File> {
    let newtypes = variables.newtypes();
    if newtypes.is_empty() {
        return None;
    }

    let mod_name = format_ident!("{}", input_symbol.to_string().to_snake_case());
    Some(parse_quote! {
        pub mod #mod_name {
            #(
                #[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
                pub struct #newtypes(pub String);

                impl #newtypes {
                    #[allow(dead_code)]
                    pub fn new(value: impl Into<String>) -> Self {
                        Self(value.into())
                    }
                }

                #[automatically_derived]
                impl std::fmt::Display for #newtypes {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        self.0.fmt(f)
                    }
                }
            )*
        }

        #[allow(unused_imports)]
        use #mod_name::{#(#newtypes),*};
    })
}

#[cfg(test)]
mod tests {
    use quote::format_ident;
    use rstest::rstest;

    use super::*;
    use crate::config::{Attachments, InputVariable, RuntimeVariable};
    use crate::options::ArtifactOptions;
    use crate::test_utils::{assert_item_eq, input_symbol};

    #[rstest]
    fn test_input_newtypes(input_symbol: syn::Ident) {
        let vars = [RuntimeVariable::Input(Box::new(InputVariable {
            ident: format_ident!("language"),
            type_: parse_quote!(std::borrow::Cow<'a, str>),
        }))];
        let options = ArtifactOptions::default().use_newtype_inputs(true);
        let variables = Variables::collect(&vars, &Attachments::default(), &options);
        let newtypes = input_newtypes(&input_symbol, &variables).expect("expected newtypes");
        assert_item_eq(&newtypes, "agent/input/input_newtypes.rs");
    }

    #[rstest]
    fn test_input_newtypes_disabled(input_symbol: syn::Ident) {
        let vars = [RuntimeVariable::Input(Box::new(InputVariable {
            ident: format_ident!("language"),
            type_: parse_quote!(std::borrow::Cow<'a, str>),
        }))];
        let variables =
            Variables::collect(&vars, &Attachments::default(), &ArtifactOptions::default());
        assert!(input_newtypes(&input_symbol, &variables).is_none());
    }
}
