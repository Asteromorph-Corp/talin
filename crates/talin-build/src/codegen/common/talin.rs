use once_cell::sync::Lazy;
use proc_macro_crate::{FoundCrate, crate_name};
use quote::format_ident;
use syn::parse_quote;

static TALIN: Lazy<Result<FoundCrate, proc_macro_crate::Error>> = Lazy::new(|| crate_name("talin"));

pub(crate) fn talin() -> syn::Path {
    match TALIN.as_ref() {
        Ok(FoundCrate::Itself) => parse_quote! { ::talin },
        Ok(FoundCrate::Name(name)) => {
            let ident = format_ident!("{name}");
            parse_quote! { ::#ident }
        }
        Err(_) => parse_quote! { ::talin },
    }
}

pub(crate) fn talin_path() -> String {
    match TALIN.as_ref() {
        Ok(FoundCrate::Itself) => "::talin".to_string(),
        Ok(FoundCrate::Name(name)) => format!("::{name}"),
        Err(_) => "::talin".to_string(),
    }
}
