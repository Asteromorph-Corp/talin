use std::collections::BTreeMap;

use serde::{Deserialize, Deserializer, Serialize};

use super::inline_or_env::InlineOrEnv;

/// Headers the generated client or the provider builders set themselves.
///
/// Request-level headers take precedence over a client's default headers, so
/// configuring any of these would be silently ignored. Reject them instead.
const RESERVED_HEADERS: &[&str] = &[
    "anthropic-version",
    "authorization",
    "content-length",
    "content-type",
    "host",
    "x-api-key",
    "x-goog-api-key",
];

/// Header values that should be hidden from `Debug` output and logs.
const SENSITIVE_HEADERS: &[&str] = &["cookie", "proxy-authorization", "api-key"];

/// Normalized (lowercase) header name -> value specification.
#[derive(Debug, Clone, Default, Serialize)]
pub(crate) struct Headers(pub(crate) BTreeMap<String, InlineOrEnv<String>>);

/// Whether a header value should be marked sensitive in generated code.
pub(crate) fn is_sensitive(name: &str, value: &InlineOrEnv<String>) -> bool {
    matches!(value, InlineOrEnv::Env(_)) || SENSITIVE_HEADERS.contains(&name)
}

impl<'de> Deserialize<'de> for Headers {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = BTreeMap::<String, InlineOrEnv<String>>::deserialize(deserializer)?;
        let mut headers = BTreeMap::default();
        for (name, value) in raw {
            let normalized = validate_name(&name).map_err(serde::de::Error::custom)?;
            validate_value(&normalized, &value).map_err(serde::de::Error::custom)?;
            if headers.insert(normalized.clone(), value).is_some() {
                return Err(serde::de::Error::custom(format!(
                    "header `{normalized}` is defined more than once (names are case-insensitive)"
                )));
            }
        }
        Ok(Headers(headers))
    }
}

fn validate_name(name: &str) -> Result<String, String> {
    // RFC 9110 `token`
    let is_token_char = |c: char| c.is_ascii_alphanumeric() || "!#$%&'*+-.^_`|~".contains(c);
    if name.is_empty() || !name.chars().all(is_token_char) {
        return Err(format!("`{name}` is not a valid header name"));
    }
    let normalized = name.to_ascii_lowercase();
    if RESERVED_HEADERS.contains(&normalized.as_str()) {
        return Err(format!(
            "header `{normalized}` is managed by the library and cannot be overridden \
             (use `api_key` for authentication)"
        ));
    }
    Ok(normalized)
}

fn validate_value(name: &str, value: &InlineOrEnv<String>) -> Result<(), String> {
    match value {
        InlineOrEnv::Inline(v) if v.chars().any(|c| c.is_control() && c != '\t') => Err(format!(
            "value of header `{name}` contains control characters"
        )),
        InlineOrEnv::Env(var) if var.is_empty() => {
            Err(format!("header `{name}` has an empty env variable name"))
        }
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[test]
    fn parses_inline_and_env_and_lowercases_names() {
        let headers: Headers = serde_json::from_value(serde_json::json!({
            "X-Tenant": "acme",
            "x-token": { "env": "TOKEN" },
        }))
        .unwrap();
        assert_eq!(
            headers.0["x-tenant"],
            InlineOrEnv::Inline("acme".to_owned())
        );
        assert_eq!(headers.0["x-token"], InlineOrEnv::Env("TOKEN".to_owned()));
    }

    #[rstest]
    #[case::empty_name(serde_json::json!({ "": "v" }), "not a valid header name")]
    #[case::space(serde_json::json!({ "x bad": "v" }), "not a valid header name")]
    #[case::colon(serde_json::json!({ "x:bad": "v" }), "not a valid header name")]
    #[case::reserved(serde_json::json!({ "Content-Type": "text/plain" }), "managed by the library")]
    #[case::auth(serde_json::json!({ "authorization": "Bearer x" }), "use `api_key`")]
    #[case::duplicate(serde_json::json!({ "x-a": "1", "X-A": "2" }), "more than once")]
    #[case::newline(serde_json::json!({ "x-a": "1\r\nx-b: 2" }), "control characters")]
    #[case::empty_env(serde_json::json!({ "x-a": { "env": "" } }), "empty env variable")]
    fn rejects_invalid(#[case] json: serde_json::Value, #[case] message: &str) {
        let err = serde_json::from_value::<Headers>(json)
            .unwrap_err()
            .to_string();
        assert!(err.contains(message), "{err:?} should contain {message:?}");
    }

    #[test]
    fn sensitivity() {
        let inline = InlineOrEnv::Inline("v".to_owned());
        let env = InlineOrEnv::Env("E".to_owned());
        assert!(!is_sensitive("x-tenant", &inline));
        assert!(is_sensitive("x-tenant", &env));
        assert!(is_sensitive("cookie", &inline));
    }
}
