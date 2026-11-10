use syn::parse_quote;

use crate::codegen::common::{get_config_symbol, struct_definition, talin};
use crate::config::{InlineOrCustom, ToolConfigDefinition};

/// Generates the config type of the tool.
///
/// - An inline definition generates a struct. It derives `ConfigSchema`, so the required fields are
///   those whose type says so.
/// - A tool without a config gets `type <Tool>Config = ();`, so that `from_config(_: _)` resolves.
/// - A custom definition generates nothing, as the type is provided by the user.
pub(super) fn config_definition(
    tool_symbol: &syn::Ident,
    config_definition: Option<&ToolConfigDefinition>,
) -> Option<syn::Item> {
    let config_symbol = get_config_symbol(tool_symbol);

    let Some(config_definition) = config_definition else {
        return Some(parse_quote! { #[allow(dead_code)] pub type #config_symbol = (); });
    };
    let InlineOrCustom::Inline(config_definition) = &config_definition.0 else {
        return None;
    };

    let talin = talin();
    let struct_def = struct_definition(&config_symbol, config_definition);

    Some(parse_quote! {
        #[derive(Debug, Default, #talin::tools::ConfigSchema)]
        #struct_def
    })
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::config::Tool;
    use crate::test_utils::{assert_item_eq, tool, tool_symbol};

    /// An inline config definition generates a config struct.
    #[rstest]
    fn test_config_definition_inline(tool_symbol: syn::Ident, tool: Tool) {
        let config_definition = config_definition(&tool_symbol, tool.config.as_ref())
            .expect("expected a config struct definition");
        assert_item_eq(&config_definition, "tool/config_definition.rs");
    }

    /// A custom config definition generates no inline config struct.
    #[rstest]
    fn test_config_definition_custom(tool_symbol: syn::Ident, tool: Tool) {
        let tool = Tool {
            config: Some(ToolConfigDefinition(InlineOrCustom::Custom)),
            ..tool
        };
        assert!(config_definition(&tool_symbol, tool.config.as_ref()).is_none());
    }

    /// A tool without a config definition aliases its config type to `()`.
    #[rstest]
    fn test_config_definition_none(tool_symbol: syn::Ident) {
        let config_definition =
            config_definition(&tool_symbol, None).expect("expected a config type alias");
        assert_item_eq(&config_definition, "tool/config_alias.rs");
    }
}
