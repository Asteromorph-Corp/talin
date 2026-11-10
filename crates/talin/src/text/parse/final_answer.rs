use serde_json::Value;

/// Parses a potentially-nested JSON value into a final answer.
pub(crate) fn parse_final_answer(mut value: Value) -> String {
    while let Value::Object(mut map) = value {
        if let Some(inner) = map.remove("final_answer") {
            value = inner;
        } else {
            value = Value::Object(map);
            break;
        }
    }

    match value {
        Value::String(s) => s,
        other => serde_json::to_string_pretty(&other).unwrap_or_else(|_| other.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use serde_json::json;

    use super::*;

    #[rstest]
    #[case(
        json!({
            "final_answer": "This is the final answer"
        }),
        "This is the final answer"
    )]
    #[case(
        json!({
            "final_answer": {
                "final_answer": "This is the final answer"
            }
        }),
        "This is the final answer"
    )]
    #[case(
        json!({
            "final_answer": {
                "final_answer": {
                    "final_answer": "This is the final answer"
                }
            }
        }),
        "This is the final answer"
    )]
    fn test_flatten_final_answer(#[case] input: Value, #[case] expected: &str) {
        let result = parse_final_answer(input);
        assert_eq!(result, expected);
    }
}
