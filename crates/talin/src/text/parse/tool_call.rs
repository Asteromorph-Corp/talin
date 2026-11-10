use serde::Deserialize;
use serde::de::DeserializeOwned;

use crate::schemas::ToolCall;

/// Parses a JSON value into a vector of [`ToolCall`]s using the provided type `T` for
/// deserialization.
pub(crate) fn parse_tool_call<T>(
    value: serde_json::Value,
) -> Result<Vec<ToolCall>, serde_json::Error>
where
    T: DeserializeOwned + Into<ToolCall> + 'static,
{
    let tool_calls = serde_json::from_value::<OneOrMany<T>>(value)?;
    let tool_calls = match tool_calls {
        OneOrMany::One(call) => vec![call.into()],
        OneOrMany::Many(calls) => calls.into_iter().map(|call| call.into()).collect(),
    };
    Ok(tool_calls)
}

/// A helper enum to handle both single and multiple tool calls in the LLM output.
#[derive(Deserialize)]
#[serde(untagged)]
enum OneOrMany<T> {
    One(T),
    Many(Vec<T>),
}

/// A helper struct to deserialize the LLM output before converting it into a [`ToolCall`].
#[derive(Debug, Deserialize)]
pub(crate) struct RawToolCall {
    #[serde(default)]
    id: Option<String>,
    #[serde(alias = "action")]
    name: String,
    #[serde(default, alias = "action_input")]
    arguments: Option<serde_json::Value>,
}

impl From<RawToolCall> for ToolCall {
    fn from(helper: RawToolCall) -> Self {
        ToolCall::new(helper.id, helper.name, helper.arguments)
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use serde_json::json;

    use super::*;

    #[rstest]
    #[case::name_and_arguments(
        json!({"name": "search", "arguments": {"query": "rust"}}),
        &[("search", json!({"query": "rust"}))],
    )]
    #[case::with_id(
        json!({"id": "call_1", "name": "search", "arguments": {"query": "rust"}}),
        &[("search", json!({"query": "rust"}))],
    )]
    #[case::action_alias(
        json!({"action": "search", "action_input": {"query": "rust"}}),
        &[("search", json!({"query": "rust"}))],
    )]
    #[case::missing_arguments(
        json!({"name": "get_time"}),
        &[("get_time", serde_json::Value::Object(serde_json::Map::default()))],
    )]
    #[case::many(
        json!([
            {"name": "search", "arguments": {"query": "rust"}},
            {"name": "get_weather", "arguments": {"city": "Paris"}},
        ]),
        &[("search", json!({"query": "rust"})), ("get_weather", json!({"city": "Paris"}))],
    )]
    fn test_deserialize_tool_call(
        #[case] value: serde_json::Value,
        #[case] expected: &[(&str, serde_json::Value)],
    ) {
        let calls = parse_tool_call::<RawToolCall>(value).unwrap();
        assert_eq!(calls.len(), expected.len());
        for (call, (name, arguments)) in calls.iter().zip(expected) {
            assert_eq!(call.name, *name);
            assert_eq!(call.arguments, *arguments);
        }
    }

    #[rstest]
    #[case::missing_name(json!({"arguments": {"query": "rust"}}))]
    #[case::empty_object(json!({}))]
    #[case::null(json!(null))]
    #[case::string(json!("not a tool call"))]
    #[case::number(json!(42))]
    #[case::name_is_not_string(json!({"name": 123}))]
    fn test_deserialize_tool_call_error(#[case] value: serde_json::Value) {
        assert!(parse_tool_call::<RawToolCall>(value).is_err());
    }
}
