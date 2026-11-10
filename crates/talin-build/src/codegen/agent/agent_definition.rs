use syn::parse_quote;

pub(super) fn agent_definition(agent_symbol: &syn::Ident) -> syn::ItemStruct {
    parse_quote! {
        pub struct #agent_symbol;
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::test_utils::{agent_symbol, assert_item_eq};

    #[rstest]
    fn test_agent_definition(agent_symbol: syn::Ident) {
        let agent_definition = agent_definition(&agent_symbol);
        assert_item_eq(&agent_definition, "agent/agent_definition.rs");
    }
}
