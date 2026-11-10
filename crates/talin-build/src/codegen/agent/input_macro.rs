use syn::parse_quote;

use crate::codegen::common::symbols::get_input_macro_name;

pub(super) fn input_macro(input_symbol: &syn::Ident) -> syn::ItemMacro {
    let macro_name = get_input_macro_name(input_symbol);

    parse_quote! {
        #[macro_export]
        macro_rules! #macro_name {
            // If value is provided
            (@value $_field:ident, $val:expr) => {
                $val
            };
            // If only identifier is used
            (@value $field:ident) => {
                $field
            };

            (
                $($field:ident $( : $val:expr )?),* $(,)?
            ) => {
                #input_symbol::new(
                    $(
                        $field: #macro_name!(@value $field $(, $val)?),
                    )*
                )
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::test_utils::{assert_item_eq, input_symbol};

    #[rstest]
    fn test_input_macro(input_symbol: syn::Ident) {
        let input_macro = input_macro(&input_symbol);
        assert_item_eq(&input_macro, "agent/input/input_macro.rs");
    }
}
