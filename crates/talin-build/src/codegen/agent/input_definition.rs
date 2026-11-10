use syn::parse_quote;

use crate::codegen::agent::variables::Variables;
use crate::codegen::common::{chain_crate_attr, crate_attr, talin};
use crate::config::RuntimeVariable;
use crate::ty::lifetime_a;

pub(super) fn input_definition(
    input_symbol: &syn::Ident,
    variables: &Variables<'_>,
) -> syn::ItemStruct {
    let talin = talin();

    let lifetime = lifetime_a();
    let maybe_lifetime: Option<syn::AngleBracketedGenericArguments> = variables
        .has_lifetime()
        .then(|| parse_quote! { <#lifetime> });

    let fields = variables.all().into_iter().map(|var| -> syn::Field {
        let ident = var.get_ident();
        let ty = var.get_type();
        let attr: syn::Attribute = match var {
            RuntimeVariable::InputMedia(_) => parse_quote! { #[chain(into = "media")] },
            _ => parse_quote! { #[chain(into = "text")] },
        };
        parse_quote! { #attr pub #ident: #ty }
    });

    let attrs = [
        parse_quote! {
            #[derive(
                #talin::chain::ChainInput,
                #talin::chain::Ctor,
            )]
        },
        crate_attr("serde"), // #[serde(crate = "...")]
        chain_crate_attr(),  // #[chain(crate = "...")]
    ];

    parse_quote! {
        #(#attrs)*
        pub struct #input_symbol #maybe_lifetime {
            #(#fields,)*
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::config::Attachments;
    use crate::options::{ArtifactOptions, InputFieldOrder};
    use crate::test_utils::{assert_item_eq, attachments, input_symbol, variables};

    #[rstest]
    fn test_input_definition(
        input_symbol: syn::Ident,
        variables: Vec<RuntimeVariable>,
        attachments: Attachments,
    ) {
        let variables = Variables::collect(&variables, &attachments, &ArtifactOptions::default());
        let input_definition = input_definition(&input_symbol, &variables);
        assert_item_eq(&input_definition, "agent/input/input_definition.rs");
    }

    #[rstest]
    fn test_input_definition_appearance_order(
        input_symbol: syn::Ident,
        variables: Vec<RuntimeVariable>,
        attachments: Attachments,
    ) {
        let options = ArtifactOptions::default().input_field_order(InputFieldOrder::Appearance);
        let variables = Variables::collect(&variables, &attachments, &options);
        let input_definition = input_definition(&input_symbol, &variables);
        assert_item_eq(
            &input_definition,
            "agent/input/input_definition.appearance.rs",
        );
    }
}
