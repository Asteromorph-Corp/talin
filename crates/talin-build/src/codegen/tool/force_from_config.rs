use syn::parse_quote;

use crate::codegen::common::talin;

pub(super) fn force_from_config(tool: &syn::Ident) -> syn::ItemFn {
    let talin = talin();

    parse_quote! {
        fn _assert_from_config()
            where #tool: #talin::tools::FromConfig
        {
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::test_utils::{assert_item_eq, tool_symbol};

    #[rstest]
    fn test_force_from_config(tool_symbol: syn::Ident) {
        let force_from_config = force_from_config(&tool_symbol);
        assert_item_eq(&force_from_config, "tool/force_from_config.rs");
    }
}
