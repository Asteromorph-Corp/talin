use serde::de::DeserializeOwned;

use crate::text::parse::{
    close_unclosed_structures, escape_newlines_in_strings, strip_trailing_commas,
};

/// Parses a potentially malformed JSON string into the specified type `T`.
pub fn parse_json<T: DeserializeOwned>(text: &str) -> Result<T, serde_json::Error> {
    if let Ok(val) = serde_json::from_str::<T>(text) {
        return Ok(val);
    }

    let multiline_removed = escape_newlines_in_strings(text);
    if let Ok(val) = serde_json::from_str::<T>(&multiline_removed) {
        return Ok(val);
    }

    // Step 2: Try parsing the cleaned version
    let comma_cleaned = strip_trailing_commas(&multiline_removed);
    if let Ok(val) = serde_json::from_str::<T>(&comma_cleaned) {
        return Ok(val);
    }

    // Step 3: Attempt to balance braces/brackets
    let balanced = close_unclosed_structures(&comma_cleaned);
    serde_json::from_str(&balanced)
}
