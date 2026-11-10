use quote::format_ident;
use syn::parse_quote;

use crate::codegen::common::symbols::{
    get_tool_metadata_macro, get_tool_metadata_module, get_tool_symbol,
};
use crate::codegen::tool::config_definition::config_definition;
use crate::codegen::tool::force_from_config::force_from_config;
use crate::codegen::tool::input_definition::input_definition;
use crate::config::Tool;

mod config_definition;
mod force_from_config;
mod input_definition;

pub(crate) fn generate_tool(id: &str, tool: &Tool) -> syn::File {
    let tool_symbol = get_tool_symbol(id);

    let metadata = metadata_items(id, tool);

    let input_struct = input_definition(&tool_symbol, tool);
    let config_struct = config_definition(&tool_symbol, tool.config.as_ref());

    let force_from_config = tool
        .config
        .as_ref()
        .map(|_| force_from_config(&tool_symbol));

    parse_quote! {
        #metadata

        #input_struct
        #config_struct

        #force_from_config
    }
}

/// Generates the items for the metadata fields (`name` and `description`) of the tool.
///
/// The defined fields are constants wrapped in a module, so that multiple tools can coexist in the
/// same scope, and each field has a macro expanding to its constant. If a field is not defined,
/// the macro expands to a `compile_error!` instead, which is invoked by the `#[tool]` macro only
/// when the implementor did not define the corresponding function. This lets us report a
/// comprehensible error instead of an unresolved constant.
fn metadata_items(id: &str, tool: &Tool) -> syn::File {
    let module_name = get_tool_metadata_module(id);

    let fields = [("name", &tool.name), ("description", &tool.description)];
    let consts = fields.iter().filter_map(|(field, value)| {
        let const_name = format_ident!("TOOL_{}", field.to_uppercase());
        value.as_deref().map(|value| -> syn::Item {
            parse_quote! { pub const #const_name: &str = #value; }
        })
    });
    let macros = fields.iter().map(|(field, value)| -> syn::Item {
        let const_name = format_ident!("TOOL_{}", field.to_uppercase());
        let macro_name = get_tool_metadata_macro(id, field);
        match value {
            Some(_) => parse_quote! {
                macro_rules! #macro_name {
                    () => { #module_name::#const_name };
                }
            },
            None => {
                let message = format!(
                    "tool `{id}` has no `{field}` in its toml definition. \
                    Either add `{field}` to the toml definition, or define \
                    `fn {field}(&self) -> String` in the `#[tool]` block."
                );
                parse_quote! {
                    macro_rules! #macro_name {
                        () => { compile_error!(#message) };
                    }
                }
            }
        }
    });

    parse_quote! {
        #[doc(hidden)]
        mod #module_name {
            #(#consts)*
        }
        #(#macros)*
    }
}
