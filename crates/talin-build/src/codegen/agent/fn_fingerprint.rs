use syn::parse_quote;

pub(super) fn fn_fingerprint(fingerprint: &str) -> syn::ImplItemFn {
    parse_quote! {
        pub fn fingerprint(&self) -> &'static str {
            #fingerprint
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::test_utils::assert_item_eq;

    #[rstest]
    fn test_fn_fingerprint() {
        let fingerprint = "test_fingerprint";
        let fn_fingerprint = fn_fingerprint(fingerprint);
        assert_item_eq(&fn_fingerprint, "agent/fn_fingerprint.rs");
    }
}
