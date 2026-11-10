use syn::parse_quote;

use crate::codegen::common::talin;

pub(super) fn fn_session(
    profile_symbol: &syn::Ident,
    input_ctor: &syn::Type,
    output_ctor: &syn::Type,
) -> syn::ItemFn {
    let talin = talin();

    parse_quote! {
        pub async fn session<P: #profile_symbol>(profile: P) -> #talin::agent::AgentSession<P::Llm, #input_ctor, #output_ctor, P::Strategy> {
            Self::new(profile).await.session()
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::test_utils::{assert_item_eq, input_ctor, output_ctor, profile_symbol};

    #[rstest]
    fn test_fn_session(profile_symbol: syn::Ident, input_ctor: syn::Type, output_ctor: syn::Type) {
        let fn_session = fn_session(&profile_symbol, &input_ctor, &output_ctor);
        assert_item_eq(&fn_session, "agent/fn_session.rs");
    }
}
