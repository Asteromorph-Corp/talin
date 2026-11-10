use proc_macro2_diagnostics::Diagnostic;
use quote::quote;
use syn::{ImplItem, ItemImpl, parse_quote};

use crate::ast::extract_items;
use crate::extract_tool_id;
use crate::macros::impl_from_config::impl_from_config;
use crate::macros::impl_tool::impl_tool;
use crate::macros::symbols::get_tool_metadata_module;

pub(crate) fn tool(mut impl_block: ItemImpl) -> Result<proc_macro2::TokenStream, Diagnostic> {
    let tool_id = extract_tool_id(impl_block.self_ty.as_ref())?;
    expose_metadata_consts(&mut impl_block, &tool_id);
    let tool_ty = impl_block.self_ty.as_ref();
    let tool_file_path = format!("/tools/{tool_id}.rs");

    let items = extract_items(&impl_block)?;

    let impl_tool = impl_tool(&tool_id, tool_ty, &items);
    let impl_from_config = items
        .fn_from_config
        .map(|fn_from_config| impl_from_config(&tool_id, tool_ty, fn_from_config))
        .transpose()?;

    let expanded = quote! {
        include!(concat!(env!("OUT_DIR"), #tool_file_path));
        #impl_tool
        #impl_from_config
    };

    Ok(expanded)
}

/// Brings `TOOL_NAME` and `TOOL_DESCRIPTION` into scope of every function in the impl block.
///
/// The constants live in a per-tool module to avoid conflicts between tools in the same scope.
fn expose_metadata_consts(impl_block: &mut ItemImpl, tool_id: &str) {
    let module = get_tool_metadata_module(tool_id);

    for item in &mut impl_block.items {
        if let ImplItem::Fn(item_fn) = item {
            item_fn.block.stmts.insert(
                0,
                parse_quote! {
                    #[allow(unused_imports)]
                    use #module::*;
                },
            );
        }
    }
}
