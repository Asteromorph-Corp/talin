use std::collections::BTreeMap;
use std::path::Path;

use heck::ToSnakeCase;
use quote::{format_ident, quote};
use syn::parse_quote;

use crate::config::{Agent, Tool};

pub(crate) fn generate_agent_index(out_dir: &Path, agents: &BTreeMap<String, Agent>) -> syn::File {
    let index = agents.keys().map(|id| {
        let id = id.to_snake_case();
        let mod_path = format!("{}/{}.rs", out_dir.display(), id);
        let mod_ident = format_ident!("{id}");

        quote! {
            #[path = #mod_path]
            mod #mod_ident;
            #[allow(unused_imports)]
            pub use #mod_ident::*;
        }
    });

    parse_quote! { #(#index)* }
}

pub(crate) fn generate_tool_index(out_dir: &Path, tools: &BTreeMap<String, Tool>) -> syn::File {
    let index = tools.keys().map(|id| {
        let tool_id = id.to_snake_case();
        let mod_path = format!("{}/{}.rs", out_dir.display(), tool_id);
        let mod_ident = format_ident!("{tool_id}");

        Some(quote! {
            #[path = #mod_path]
            mod #mod_ident;
            pub use #mod_ident::*;
        })
    });

    parse_quote! { #(#index)* }
}
