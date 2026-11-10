use std::collections::BTreeMap;

use heck::ToSnakeCase;
use quote::{format_ident, quote};
use syn::parse_quote;

use crate::codegen::agent::tool_params::tool_init_params;
use crate::codegen::common::symbols::get_tool_symbol;
use crate::codegen::common::talin;
use crate::config::{
    AgentBuiltinTool, AgentLocalTool, AgentLocalToolConfig, AgentLocalToolConfigNested,
    AgentMcpToolGrouped, AgentToolbox,
};

pub(super) fn local_tool_expr(tool: &AgentLocalTool) -> syn::Expr {
    let talin = talin();
    let tool_symbol = get_tool_symbol(&tool.name);
    let init_symbol: syn::Type =
        parse_quote! { <#tool_symbol as #talin::tools::FromConfig>::Config };
    let assert_complete = assert_config_complete(&tool.name, &init_symbol, tool.config.as_ref());

    let init: syn::Expr = match &tool.config {
        Some(config) => {
            let config = tool_init_params(&init_symbol, config);
            parse_quote! {
                <#tool_symbol as #talin::tools::FromConfig>::from_config(#config)
            }
        }
        None => parse_quote! {
            #[allow(clippy::unit_arg)]
            <#tool_symbol as #talin::tools::FromConfig>::from_config(#init_symbol::default())
        },
    };

    parse_quote! {
        {
            #assert_complete
            #init
        }
    }
}

/// Generates a `const` assertion that the config supplied for the tool provides every field its
/// config type requires.
///
/// Config types that do not implement `ConfigSchema` are not checked. The message of a const
/// `panic!` cannot be formatted, so it names the tool, and the field is found by comparing the
/// supplied config with the required fields of the config type.
fn assert_config_complete(
    tool_name: &str,
    config_ty: &syn::Type,
    config: Option<&AgentLocalToolConfig>,
) -> syn::Stmt {
    let talin = talin();
    let supplied = match config {
        Some(AgentLocalToolConfig::Nested(AgentLocalToolConfigNested::Struct(fields))) => {
            supplied_fields(fields)
        }
        _ => quote! { &[] },
    };
    let message =
        format!("the config supplied for tool `{tool_name}` is missing a required field.");

    parse_quote! {
        const _: () = {
            #[allow(unused_imports)]
            use #talin::__private::ConfigProbeFallback as _;
            const SUPPLIED: &[#talin::__private::Supplied] = #supplied;
            const {
                assert!(
                    #talin::__private::ConfigProbe::<#config_ty>::SCHEMA.is_complete(SUPPLIED),
                    #message
                )
            }
        };
    }
}

fn supplied_fields(fields: &BTreeMap<String, AgentLocalToolConfig>) -> proc_macro2::TokenStream {
    let talin = talin();
    let entries = fields.iter().map(|(name, value)| match value {
        AgentLocalToolConfig::Nested(AgentLocalToolConfigNested::Struct(nested)) => {
            let children = supplied_fields(nested);
            quote! { #talin::__private::Supplied::node(#name, #children) }
        }
        _ => quote! { #talin::__private::Supplied::leaf(#name) },
    });
    quote! { &[#(#entries),*] }
}

pub(super) fn mcp_server_expr(tool: &AgentMcpToolGrouped) -> syn::Expr {
    let server_fn = format_ident!("{}", tool.server.to_snake_case());

    let tools = tool.names.as_ref().map(|names| {
        quote! { .with_allowed_tools(vec![#(String::from(#names)),*]) }
    });

    parse_quote! { __agent_deps::McpServers::#server_fn() #tools }
}

pub(super) fn builtin_tool_expr(tool: &AgentBuiltinTool) -> syn::Expr {
    let talin = talin();
    let tool_ty = tool.tool_ty();
    let json_str = tool.to_json();

    parse_quote! {
        #talin::__private::serde_json::from_str::<#tool_ty>(#json_str)
            .expect("Tool deserialization shouldn't fail")
    }
}

#[allow(dead_code)]
pub(super) fn agent_toolboxes(toolboxes: &[AgentToolbox]) -> Option<syn::Expr> {
    if toolboxes.is_empty() {
        return None;
    }

    let talin = talin();

    let toolbox_exprs = toolboxes.iter().map(|toolbox| -> syn::Expr {
        let toolbox_name = &toolbox.name;
        let toolbox_tools = toolbox.tools.iter().map(local_tool_expr);
        parse_quote! {
            #talin::tools::SimpleToolbox::from_tools(
                #toolbox_name, vec![ #(#toolbox_tools),* ]
            )
        }
    });
    Some(parse_quote! { vec![#(#toolbox_exprs),*] })
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::str::FromStr;

    use rstest::rstest;

    use super::*;
    use crate::config::{AgentLocalToolConfig, AgentLocalToolConfigNested};
    use crate::test_utils::assert_expr_eq;

    fn nested_config() -> AgentLocalToolConfig {
        AgentLocalToolConfig::Nested(AgentLocalToolConfigNested::Struct(BTreeMap::from([(
            "query".to_owned(),
            AgentLocalToolConfig::String("rust".to_owned()),
        )])))
    }

    #[rstest]
    #[case::default(
        AgentLocalTool {
            name: "my_tool".to_owned(),
            config: None,
        },
        "agent/tools/local_tool.default.rs"
    )]
    #[case::config(
        AgentLocalTool {
            name: "my_tool".to_owned(),
            config: Some(nested_config()),
        },
        "agent/tools/local_tool.config.rs"
    )]
    fn test_local_tool_expr(#[case] tool: AgentLocalTool, #[case] fixture_path: &str) {
        let expr = local_tool_expr(&tool);
        assert_expr_eq(&expr, fixture_path);
    }

    #[rstest]
    #[case::all(
        AgentMcpToolGrouped {
            server: "my_server".to_owned(),
            names: None,
        },
        "agent/tools/mcp_server.all.rs",
    )]
    #[case::allowed(
        AgentMcpToolGrouped {
            server: "my_server".to_owned(),
            names: Some(vec!["search".to_owned(), "fetch".to_owned()]),
        },
        "agent/tools/mcp_server.allowed.rs",
    )]
    fn test_mcp_server_expr(#[case] tool: AgentMcpToolGrouped, #[case] fixture_path: &str) {
        let expr = mcp_server_expr(&tool);
        assert_expr_eq(&expr, fixture_path);
    }

    #[rstest]
    fn test_builtin_tool_expr() {
        let tool = AgentBuiltinTool::from_str("claude_messages/web_search").unwrap();
        let expr = builtin_tool_expr(&tool);
        assert_expr_eq(&expr, "agent/tools/builtin_tool.rs");
    }
}
