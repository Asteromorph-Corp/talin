use quote::format_ident;
use syn::parse_quote;

use crate::codegen::common::{
    crate_attr, schema_struct_definition, serde_deny_unknown_fields_attr, talin,
};
use crate::config::{InlineOrCustom, Tool};

pub(super) fn input_definition(tool_symbol: &syn::Ident, tool: &Tool) -> Option<syn::ItemStruct> {
    let talin = talin();

    let input_symbol = format_ident!("{}Input", tool_symbol);
    let InlineOrCustom::Inline(struct_) = &tool.input else {
        return None;
    };
    let struct_def = schema_struct_definition(&input_symbol, struct_);
    // Empty tool inputs must have an explicit object schema for strict tool validation.
    let empty_object_shape: Option<syn::Attribute> = struct_.0.is_empty().then(|| {
        parse_quote! {
            #[schemars(extend("properties" = {}, "required" = []))]
        }
    });

    let attrs = [
        parse_quote! {
            #[derive(
                Debug,
                Default,
                #talin::__private::serde::Serialize,
                #talin::__private::serde::Deserialize,
                #talin::__private::schemars::JsonSchema,
            )]
        },
        crate_attr("serde"),              // #[serde(crate = "...")]
        crate_attr("schemars"),           // #[schemars(crate = "...")]
        serde_deny_unknown_fields_attr(), // #[serde(deny_unknown_fields)]
    ];

    Some(parse_quote! {
        #(#attrs)*
        #empty_object_shape
        #struct_def
    })
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::test_utils::{assert_item_eq, tool, tool_symbol};

    /// An inline input definition generates an input struct.
    #[rstest]
    fn test_input_definition_inline(tool_symbol: syn::Ident, tool: Tool) {
        let input_definition =
            input_definition(&tool_symbol, &tool).expect("expected an input struct definition");
        assert_item_eq(&input_definition, "tool/input_definition.rs");
    }

    /// A custom input definition generates no inline input struct.
    #[rstest]
    fn test_input_definition_custom(tool_symbol: syn::Ident, tool: Tool) {
        let tool = Tool {
            input: InlineOrCustom::Custom,
            ..tool
        };
        assert!(input_definition(&tool_symbol, &tool).is_none());
    }
}
