use syn::parse_quote;

use crate::ast::ImplItems;
use crate::macros::crate_path::talin;
use crate::macros::symbols::{get_input_symbol, get_tool_metadata_macro};

pub(super) fn impl_tool(
    tool_id: &str,
    tool_ty: &syn::Type,
    items: &ImplItems<'_>,
) -> syn::ItemImpl {
    let talin = talin();
    let input_symbol = get_input_symbol(tool_id);
    let name_macro = get_tool_metadata_macro(tool_id, "name");
    let description_macro = get_tool_metadata_macro(tool_id, "description");
    let ImplItems {
        output_ty,
        fn_call,
        fn_name,
        fn_description,
        fn_available,
        fn_summarize_input,
        fn_summarize_output,
        fn_from_config: _,
    } = items;

    let default_output_ty = parse_quote! { type Output = String; };
    let default_name_fn = parse_quote! {
        fn name(&self) -> String {
            #name_macro!().to_owned()
        }
    };
    let default_description_fn = parse_quote! {
        fn description(&self) -> String {
            #description_macro!().to_owned()
        }
    };

    let output_ty = output_ty.unwrap_or(&default_output_ty);
    let fn_name = fn_name.unwrap_or(&default_name_fn);
    let fn_description = fn_description.unwrap_or(&default_description_fn);

    parse_quote! {
        #[#talin::__private::async_trait::async_trait]
        impl #talin::tools::Function for #tool_ty {
            type Input = #input_symbol;
            #output_ty

            #fn_name
            #fn_description

            fn strict(&self) -> bool {
                true
            }

            #fn_call
            #fn_available
            #fn_summarize_input
            #fn_summarize_output
        }
    }
}
