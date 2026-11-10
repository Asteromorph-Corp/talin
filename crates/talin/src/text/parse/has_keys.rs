use std::collections::HashSet;

use serde_json::Value;

/// Returns whether the JSON has at least one of the specified sets of keys.
///
/// Used to distinguish between malformed event outputs and valid JSON-formatted answers
/// to prevent incorrect classification in the event loop.
pub(crate) fn has_keys(json: &Value, keys: &[&[&str]]) -> bool {
    let Some(object) = json.as_object() else {
        return false;
    };
    let object_keys = object.keys().map(|s| s.as_str()).collect::<HashSet<_>>();

    keys.iter()
        .any(|valid| valid.iter().all(|key| object_keys.contains(key)))
}

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use serde_json::json;

    use super::*;

    #[rstest]
    #[case(json!({ "action": 1, "action_input": "lalala" }), &[["action", "action_input"].as_slice()], true)]
    #[case(json!({ "name": "pikachu", "dex_no": 25 }), &[["action", "action_input"].as_slice()], false)]
    fn test_has_keys(#[case] json: Value, #[case] valid_keys: &[&[&str]], #[case] expected: bool) {
        let result = has_keys(&json, valid_keys);
        assert_eq!(result, expected);
    }
}
