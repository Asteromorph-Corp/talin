use heck::{ToPascalCase, ToSnakeCase};
use quote::format_ident;

pub(crate) fn get_input_symbol(tool_id: &str) -> syn::Ident {
    format_ident!("{}ToolInput", tool_id.to_pascal_case())
}

pub(crate) fn get_config_symbol(tool_id: &str) -> syn::Ident {
    format_ident!("{}ToolConfig", tool_id.to_pascal_case())
}

/// Name of the module wrapping a tool's metadata constants (`name` and `description`).
///
/// Must match the name used by the build script.
pub(crate) fn get_tool_metadata_module(tool_id: &str) -> syn::Ident {
    format_ident!("__{}_tool_metadata", tool_id.to_snake_case())
}

/// Name of the macro expanding to a tool's metadata constant (`name` or `description`).
///
/// Must match the name used by the build script.
pub(crate) fn get_tool_metadata_macro(tool_id: &str, field: &str) -> syn::Ident {
    format_ident!("__{}_tool_{}", tool_id.to_snake_case(), field)
}
