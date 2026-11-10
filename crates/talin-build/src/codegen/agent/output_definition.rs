use syn::parse_quote;

use crate::codegen::common::{
    chain_crate_attr, chain_schema_attr, schema_struct_definition, schemars_meta_attr,
    serde_deny_unknown_fields_attr, talin,
};
use crate::config::{AgentResponseFormat, InlineOrCustom};

pub(super) fn output_definition(
    output_symbol: &syn::Ident,
    response_format: &Option<InlineOrCustom<AgentResponseFormat>>,
) -> Option<syn::ItemStruct> {
    let Some(InlineOrCustom::Inline(format)) = response_format else {
        return None;
    };

    let struct_def = schema_struct_definition(output_symbol, &format.fields);

    let talin = talin();
    let attrs = [
        parse_quote! {
            #[derive(
                Debug,
                Clone,
                #talin::chain::Ctor,
                #talin::chain::ChainOutput
            )]
        },
        chain_crate_attr(),               // #[chain(crate = "...")]
        serde_deny_unknown_fields_attr(), // #[serde(deny_unknown_fields)]
        // #[schemars(title = "...", description = "...")]
        schemars_meta_attr(&format.name, format.description.as_deref()),
        chain_schema_attr(), // #[chain(derive_schema)]
        parse_quote! { #[allow(dead_code)] },
    ];

    Some(parse_quote! {
        #(#attrs)*
        #struct_def
    })
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::config::AgentResponseFormat;
    use crate::test_utils::{assert_item_eq, output_symbol, response_format};

    /// An inline response format generates an output struct definition.
    #[rstest]
    fn test_output_definition_inline(
        output_symbol: syn::Ident,
        response_format: AgentResponseFormat,
    ) {
        let response_format = Some(InlineOrCustom::Inline(response_format));
        let output_definition = output_definition(&output_symbol, &response_format)
            .expect("expected an output struct definition");
        assert_item_eq(&output_definition, "agent/output_definition.rs");
    }

    /// A custom response format generates no inline output struct.
    #[rstest]
    fn test_output_definition_custom(output_symbol: syn::Ident) {
        let response_format = Some(InlineOrCustom::Custom);
        assert!(output_definition(&output_symbol, &response_format).is_none());
    }

    /// No response format generates no inline output struct.
    #[rstest]
    fn test_output_definition_none(output_symbol: syn::Ident) {
        assert!(output_definition(&output_symbol, &None).is_none());
    }
}
