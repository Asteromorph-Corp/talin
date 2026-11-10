use heck::{ToPascalCase, ToSnakeCase};
use quote::format_ident;

pub(crate) fn get_input_macro_name(input_symbol: &syn::Ident) -> syn::Ident {
    format_ident!("{}", input_symbol.to_string().to_snake_case())
}

pub(crate) fn get_agent_symbol(id: &str) -> syn::Ident {
    let id = id.to_pascal_case();
    if id.to_lowercase().contains("agent") {
        format_ident!("{id}")
    } else {
        format_ident!("{id}Agent")
    }
}

pub(crate) fn get_agent_ext_symbol(id: &str) -> syn::Ident {
    let id = id.to_pascal_case();
    if id.to_lowercase().contains("agent") {
        format_ident!("{id}Ext")
    } else {
        format_ident!("{id}AgentExt")
    }
}

pub(crate) fn get_agent_session_ext_symbol(id: &str) -> syn::Ident {
    let id = id.to_pascal_case();
    if id.to_lowercase().contains("agent") {
        format_ident!("{id}SessionExt")
    } else {
        format_ident!("{id}AgentSessionExt")
    }
}

pub(crate) fn get_agent_profile_symbol(id: &str) -> syn::Ident {
    let id = id.to_pascal_case();
    if id.to_lowercase().contains("agent") {
        format_ident!("{}Profile", id)
    } else {
        format_ident!("{}AgentProfile", id)
    }
}

pub(crate) fn get_tool_symbol(id: &str) -> syn::Ident {
    // Legacy support for specific tool IDs
    if id == "openalex_search" {
        return format_ident!("OpenAlexSearchTool");
    }

    let id = id.to_pascal_case();
    if id.to_lowercase().contains("tool") {
        format_ident!("{id}")
    } else {
        format_ident!("{id}Tool")
    }
}

pub(crate) fn get_input_symbol(symbol: &syn::Ident) -> syn::Ident {
    format_ident!("{symbol}Input")
}

pub(crate) fn get_output_symbol(symbol: &syn::Ident) -> syn::Ident {
    format_ident!("{symbol}Output")
}

pub(crate) fn get_ctor_symbol(symbol: &syn::Ident) -> syn::Type {
    let ident = format_ident!("{symbol}Ctor");
    syn::parse_quote! { #ident }
}

pub(crate) fn get_config_symbol(symbol: &syn::Ident) -> syn::Ident {
    format_ident!("{symbol}Config")
}

/// Name of the module wrapping a tool's metadata constants (`name` and `description`).
///
/// Must match the name used by the `#[tool]` macro.
pub(crate) fn get_tool_metadata_module(id: &str) -> syn::Ident {
    format_ident!("__{}_tool_metadata", id.to_snake_case())
}

/// Name of the macro expanding to a tool's metadata constant (`name` or `description`).
///
/// Must match the name used by the `#[tool]` macro.
pub(crate) fn get_tool_metadata_macro(id: &str, field: &str) -> syn::Ident {
    format_ident!("__{}_tool_{}", id.to_snake_case(), field)
}
