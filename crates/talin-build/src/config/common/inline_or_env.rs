use serde::de::IntoDeserializer;
use serde::{Deserialize, Serialize};
use syn::parse_quote;
use url::Url;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum InlineOrEnv<T> {
    Inline(T),
    Env(String),
}

impl InlineOrEnv<String> {
    pub(crate) fn to_expr(&self) -> syn::Expr {
        match self {
            InlineOrEnv::Inline(val) => parse_quote! {
                String::from(#val)
            },
            InlineOrEnv::Env(name) => parse_quote! {
                std::env::var(#name).unwrap_or_else(|e| panic!("Failed to get env var '{}': {}", #name, e))
            },
        }
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for InlineOrEnv<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let out = match InlineOrEnvHelper::<T>::deserialize(deserializer)? {
            InlineOrEnvHelper::Inline(val) => InlineOrEnv::Inline(val),
            InlineOrEnvHelper::ForcedInline(val) => InlineOrEnv::Inline(val),
            InlineOrEnvHelper::Env(name) => InlineOrEnv::Env(name),
        };
        Ok(out)
    }
}

/// Deserialize api key specification for a model.
///
/// Supports syntaxes:
/// - Inline: `api_key.inline = "my_api_key"`
/// - Environment variable: `api_key = "MY_API_KEY"` / `api_key.env = "MY_API_KEY"`
pub(crate) fn deserialize_api_key<'de, D>(
    deserializer: D,
) -> Result<Option<InlineOrEnv<String>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let Some(helper) = Option::<InlineOrEnvHelper<String>>::deserialize(deserializer)? else {
        return Ok(None);
    };
    let out = match helper {
        InlineOrEnvHelper::Inline(val) => InlineOrEnv::Env(val),
        InlineOrEnvHelper::ForcedInline(val) => InlineOrEnv::Inline(val),
        InlineOrEnvHelper::Env(name) => InlineOrEnv::Env(name),
    };
    Ok(Some(out))
}

/// Deserialize api base specification for a model.
///
/// Supports syntaxes:
/// - Inline: `api_base = "https://api.example.com"` / `api_base.inline = "https://api.example.com"`
/// - Environment variable: `api_base = "API_BASE"` / `api_base.env = "API_BASE"`
///
/// Untagged inline values (`api_base = <value>`) are interpreted as env if they are not valid URLs.
pub(crate) fn deserialize_api_base<'de, D>(deserializer: D) -> Result<InlineOrEnv<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let out = match InlineOrEnvHelper::<String>::deserialize(deserializer)? {
        InlineOrEnvHelper::Inline(val) if Url::parse(&val).is_ok() => InlineOrEnv::Inline(val),
        InlineOrEnvHelper::Inline(val) => InlineOrEnv::Env(val),
        InlineOrEnvHelper::ForcedInline(val) => InlineOrEnv::Inline(val),
        InlineOrEnvHelper::Env(name) => InlineOrEnv::Env(name),
    };
    Ok(out)
}

enum InlineOrEnvHelper<T> {
    /// An inline value (key = value), may be reinterpreted as env depending on context
    Inline(T),
    /// A forced inline value (key = { inline = value })
    ForcedInline(T),
    /// A env value (key = { env = "ENV" })
    Env(String),
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for InlineOrEnvHelper<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = toml::Value::deserialize(deserializer)?;
        match value {
            toml::Value::Table(table) if table.len() == 1 => {
                let (key, val) = table.into_iter().next().unwrap();
                match key.as_str() {
                    "inline" => T::deserialize(val.into_deserializer())
                        .map(InlineOrEnvHelper::ForcedInline)
                        .map_err(serde::de::Error::custom),
                    "env" => match val {
                        toml::Value::String(name) => Ok(InlineOrEnvHelper::Env(name)),
                        _ => Err(serde::de::Error::custom(
                            "expected a string for env variable name",
                        )),
                    },
                    other => Err(serde::de::Error::unknown_field(other, &["inline", "env"])),
                }
            }
            _ => T::deserialize(value.into_deserializer())
                .map(InlineOrEnvHelper::Inline)
                .map_err(serde::de::Error::custom),
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use crate::config::{InlineOrEnv, deserialize_api_base, deserialize_api_key};

    #[rstest]
    #[case::inline(serde_json::json!({"inline": "value"}), InlineOrEnv::Inline("value".to_string()))]
    #[case::env(serde_json::json!({"env": "ENV_NAME"}), InlineOrEnv::Env("ENV_NAME".to_string()))]
    #[case::direct(serde_json::json!("value"), InlineOrEnv::Inline("value".to_string()))]
    fn deserializes_inline_or_env(
        #[case] value: serde_json::Value,
        #[case] expected: InlineOrEnv<String>,
    ) {
        let deserialized: InlineOrEnv<String> = serde_json::from_value(value).unwrap();
        assert_eq!(deserialized, expected);
    }

    #[rstest]
    #[case::inline(serde_json::json!({"inline": "value"}), InlineOrEnv::Inline("value".to_string()))]
    #[case::env(serde_json::json!({"env": "ENV_NAME"}), InlineOrEnv::Env("ENV_NAME".to_string()))]
    #[case::direct(serde_json::json!("ENV_NAME"), InlineOrEnv::Env("ENV_NAME".to_string()))]
    fn deserializes_api_key(
        #[case] value: serde_json::Value,
        #[case] expected: InlineOrEnv<String>,
    ) {
        let deserialized: Option<InlineOrEnv<String>> = deserialize_api_key(value).unwrap();
        assert_eq!(deserialized, Some(expected));
    }

    #[rstest]
    #[case::inline(serde_json::json!({"inline": "https://api.example.com"}), InlineOrEnv::Inline("https://api.example.com".to_string()))]
    #[case::env(serde_json::json!({"env": "API_BASE"}), InlineOrEnv::Env("API_BASE".to_string()))]
    #[case::direct_valid_url(serde_json::json!("https://api.example.com"), InlineOrEnv::Inline("https://api.example.com".to_string()))]
    #[case::direct_invalid_url(serde_json::json!("API_BASE"), InlineOrEnv::Env("API_BASE".to_string()))]
    fn deserializes_api_base(
        #[case] value: serde_json::Value,
        #[case] expected: InlineOrEnv<String>,
    ) {
        let deserialized: InlineOrEnv<String> = deserialize_api_base(value).unwrap();
        assert_eq!(deserialized, expected);
    }
}
