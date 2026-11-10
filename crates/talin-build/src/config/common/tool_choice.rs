use serde::{Deserialize, Serialize};
use syn::parse_quote;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ToolChoice {
    None,
    Auto,
    Required,
    Function(String),
    #[serde(untagged)]
    ProviderSpecific(String),
}

impl ToolChoice {
    pub(crate) fn to_expr(&self) -> syn::Expr {
        match self {
            ToolChoice::None => parse_quote! { None },
            ToolChoice::Auto => parse_quote! { Auto },
            ToolChoice::Required => parse_quote! { Required },
            ToolChoice::ProviderSpecific(id) => parse_quote! { ProviderSpecific(#id) },
            ToolChoice::Function(func_name) => parse_quote! { function(#func_name) },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_single<T>(s: &str) -> T
    where
        for<'de> T: Deserialize<'de>,
    {
        #[derive(Deserialize)]
        struct Helper<T2> {
            x: T2,
        }

        let wrapped = format!("x = {}", s);
        let helper: Helper<T> = toml::from_str(&wrapped).unwrap();
        helper.x
    }

    #[test]
    fn test_deserialize_tool_choice() {
        let tool_choice = "\"web_search\"";
        let deserialized: ToolChoice = parse_single(tool_choice);
        match deserialized {
            ToolChoice::ProviderSpecific(id) => assert_eq!(id, "web_search"),
            _ => panic!("Expected ToolChoice::WebSearch"),
        }

        let tool_choice = "function = \"none\"";

        let deserialized: ToolChoice = toml::from_str(tool_choice).unwrap();
        match deserialized {
            ToolChoice::Function(func) => assert_eq!(func, "none"),
            _ => panic!("Expected ToolChoice::Function"),
        }
    }
}
