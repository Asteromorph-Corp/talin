use quote::format_ident;
use syn::parse_quote;

use crate::codegen::common::talin::talin_path;

pub(crate) fn crate_attr(target: &str) -> syn::Attribute {
    let talin_path = talin_path();
    let attr_ident = format_ident!("{target}");
    let attr_path = format!("{talin_path}::__private::{target}",);
    parse_quote! { #[#attr_ident(crate = #attr_path)] }
}

pub(crate) fn chain_crate_attr() -> syn::Attribute {
    let talin_path = talin_path();
    parse_quote! { #[chain(crate = #talin_path)] }
}

pub(crate) fn schemars_meta_attr(title: &str, description: Option<&str>) -> syn::Attribute {
    match description {
        Some(desc) => parse_quote! {
            #[schemars(title = #title, description = #desc)]
        },
        None => parse_quote! {
            #[schemars(title = #title)]
        },
    }
}

pub(crate) fn serde_deny_unknown_fields_attr() -> syn::Attribute {
    parse_quote! { #[serde(deny_unknown_fields)] }
}

pub(crate) fn chain_schema_attr() -> syn::Attribute {
    parse_quote! { #[chain(derive_schema)] }
}
