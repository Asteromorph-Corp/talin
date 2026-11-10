use std::collections::BTreeMap;

use heck::ToSnakeCase;
use quote::{format_ident, quote};
use syn::parse_quote;

use crate::codegen::common::talin;
use crate::config::{McpServer, McpTransport};

pub(crate) fn generate_mcp_servers(models: &BTreeMap<String, McpServer>) -> syn::File {
    let models = models
        .iter()
        .map(|(id, model)| generate_mcp_server(id, model));

    parse_quote! {
        #[allow(dead_code)]
        pub struct McpServers;

        impl McpServers {
            #(#models)*
        }
    }
}

fn generate_mcp_server(id: &str, server: &McpServer) -> syn::ItemFn {
    let talin = talin();
    let fn_name = format_ident!("{}", id.to_snake_case());

    let maybe_with_description = server.description.as_ref().map(|desc| {
        quote! { .with_description(#desc) }
    });
    let allowed_tools = &server.tools;

    let (constructor, setters): (syn::Expr, proc_macro2::TokenStream) = match &server.transport {
        McpTransport::Http {
            url,
            authorization,
            always_local,
        } => {
            let url = url.to_expr();
            let maybe_with_authorization = authorization.as_ref().map(|auth| {
                let auth_expr = auth.to_expr();
                quote! { .with_authorization(#auth_expr) }
            });
            let maybe_always_local = always_local
                .unwrap_or(false)
                .then(|| quote! { .always_local(true) });
            (
                parse_quote!(#talin::mcp::McpServerHttp::new(
                    #url, #id
                )),
                quote! { #maybe_always_local #maybe_with_authorization },
            )
        }
        McpTransport::Stdio { command, args, env } => {
            let maybe_with_args = (!args.is_empty()).then(|| {
                quote! { .with_args(vec![#(String::from(#args)),*]) }
            });
            let maybe_with_env = (!env.is_empty()).then(|| {
                let pairs = env.iter().map(|(key, value)| {
                    let value = value.to_expr();
                    quote! { (String::from(#key), #value.into()) }
                });
                quote! { .with_env(vec![#(#pairs),*]) }
            });
            (
                parse_quote!(#talin::mcp::McpServerStdio::new(
                    #command, #id
                )),
                quote! { #maybe_with_args #maybe_with_env },
            )
        }
    };

    parse_quote! {
        #[allow(dead_code)]
        pub fn #fn_name() -> #talin::mcp::McpServer {
            #constructor
            #maybe_with_description
            .with_allowed_tools(vec![#(String::from(#allowed_tools)),*])
            #setters
            .into()
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::config::InlineOrEnv;
    use crate::test_utils::{assert_item_eq, mcp_server, mcp_stdio_server};

    /// `full` emits both the description and authorization; `minimal` strips them so neither
    /// is emitted; `stdio` emits the child-process constructor with args and env.
    #[rstest]
    #[case::full(mcp_server(), "mcp_server/generate_mcp_server.rs")]
    #[case::minimal(
        McpServer {
            description: None,
            transport: McpTransport::Http {
                url: InlineOrEnv::Inline("https://mcp.example.com".to_owned()),
                authorization: None,
                always_local: None,
            },
            ..mcp_server()
        },
        "mcp_server/generate_mcp_server.minimal.rs"
    )]
    #[case::stdio(mcp_stdio_server(), "mcp_server/generate_mcp_server.stdio.rs")]
    fn test_generate_mcp_server(#[case] server: McpServer, #[case] fixture_path: &str) {
        let generate_mcp_server = generate_mcp_server("my_server", &server);
        assert_item_eq(&generate_mcp_server, fixture_path);
    }
}
