//! Parsing utilities for fixing common issues in JSON strings.

use std::collections::VecDeque;

use regex::Regex;

/// Unescapes malformed escape sequences (e.g. `\\n` instead of `\n`) from the input text.
pub(crate) fn unescape_text(text: &str) -> String {
    let re = Regex::new(r"\\(.)").unwrap();
    re.replace_all(text, |caps: &regex::Captures<'_>| match &caps[1] {
        "n" => "\n".to_owned(),
        "t" => "\t".to_owned(),
        "r" => "\r".to_owned(),
        other => other.to_owned(), // Leave unknown sequences unchanged
    })
    .to_string()
}

/// Removes line breaks from JSON strings, replacing them with an escaped newline character.
pub(crate) fn escape_newlines_in_strings(s: &str) -> String {
    let mut cleaned = String::new();
    let mut inside_string = false;
    let mut escaped = false;

    for c in s.chars() {
        match c {
            '"' if !escaped => {
                inside_string = !inside_string;
                cleaned.push(c);
            }
            '\\' if inside_string => {
                escaped = !escaped;
                cleaned.push(c);
            }
            '\n' if inside_string => {
                cleaned.push_str("\\n");
            }
            _ => {
                escaped = false;
                cleaned.push(c);
            }
        }
    }

    cleaned
}

/// Strips trailing commas before closing braces or brackets.
pub(crate) fn strip_trailing_commas(s: &str) -> String {
    let mut cleaned = String::new();
    let mut chars = s.chars();
    let mut inside_string = false;
    let mut escaped = false;

    while let Some(c) = chars.next() {
        match c {
            '"' if !escaped => {
                inside_string = !inside_string;
                cleaned.push(c);
            }
            '\\' if inside_string => {
                escaped = !escaped;
                cleaned.push(c);
                continue;
            }
            ',' if !inside_string => {
                // Peek ahead for } or ]
                if let Some(next_non_ws) = chars.clone().find(|c| !c.is_whitespace())
                    && (next_non_ws == '}' || next_non_ws == ']')
                {
                    // Skip this comma
                    continue;
                }
                cleaned.push(c);
            }
            _ => {
                escaped = false;
                cleaned.push(c);
            }
        }
    }

    cleaned
}

/// Closes unclosed JSON structures by adding the necessary closing parenthesis or brackets.
pub(crate) fn close_unclosed_structures(s: &str) -> String {
    let mut new_s = String::new();
    let mut stack: VecDeque<char> = VecDeque::new();
    let mut is_inside_string = false;
    let mut escaped = false;

    for char in s.chars() {
        match char {
            '"' if !escaped => is_inside_string = !is_inside_string,
            '{' if !is_inside_string => stack.push_back('}'),
            '[' if !is_inside_string => stack.push_back(']'),
            '}' | ']' if !is_inside_string => {
                if let Some(c) = stack.pop_back() {
                    if c != char {
                        return s.into(); // Mismatched closing character, return unmodified
                    }
                } else {
                    return s.into(); // Unbalanced closing character, return unmodified
                }
            }
            '\\' if is_inside_string => escaped = !escaped,
            _ => escaped = false,
        }
        new_s.push(char);
    }

    // Close any open structures.
    while let Some(c) = stack.pop_back() {
        new_s.push(c);
    }

    new_s
}

#[cfg(test)]
mod tests {
    use indoc::indoc;
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case(
        r#"This is a test\nNew line\tTabbed\rCarriage return"#,
        "This is a test\nNew line\tTabbed\rCarriage return"
    )]
    fn test_fix_text(#[case] input: &str, #[case] expected: &str) {
        let fixed = unescape_text(input);
        assert_eq!(fixed, expected);
    }

    #[rstest]
    #[case(
        indoc! {r#"
            {
                "key1": "This is a malformed JSON string.
            The newline within the string is incorrectly parsed as an actual newline.",
                "key2": "value2"
            }"#
        },
        indoc! {r#"
            {
                "key1": "This is a malformed JSON string.\nThe newline within the string is incorrectly parsed as an actual newline.",
                "key2": "value2"
            }"#
        }
    )]
    fn test_remove_multiline(#[case] input: &str, #[case] expected: &str) {
        let cleaned = escape_newlines_in_strings(input);
        assert_eq!(cleaned, expected);
    }

    #[rstest]
    #[case(
        indoc! {r#"
            {
                "key1": "value1",
                "key2": "value2",
            }"#
        },
        indoc! {r#"
            {
                "key1": "value1",
                "key2": "value2"
            }"#
        }
    )]
    #[case(r#"["item1", "item2",]"#, r#"["item1", "item2"]"#)]
    fn test_remove_trailing_commas(#[case] input: &str, #[case] expected: &str) {
        let cleaned = strip_trailing_commas(input);
        assert_eq!(cleaned, expected);
    }

    #[rstest]
    #[case(
        r#"{"key1": "value1", "key2": ["item1", "item2""#,
        r#"{"key1": "value1", "key2": ["item1", "item2"]}"#
    )]
    fn test_balance_parenthesis(#[case] input: &str, #[case] expected: &str) {
        let balanced = close_unclosed_structures(input);
        assert_eq!(balanced, expected);
    }
}
