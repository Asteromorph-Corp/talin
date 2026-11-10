//! Parsing utilities for extracting substrings from text, such as JSON objects, code blocks, and
//! XML-like tags.

use regex::Regex;

/// Returns the text after the closing thought tag `</think>`.
///
/// If no tag is found, returns the input unchanged.
pub(crate) fn remove_thought(text: &str) -> &str {
    if text.contains("</think>") {
        let parts: Vec<&str> = text.split("</think>").collect();
        parts.last().unwrap_or(&"").trim()
    } else {
        text
    }
}

/// Returns the text inside a Markdown code block ```.
///
/// If no code block is found, returns the input unchanged.
pub(crate) fn extract_from_codeblock(text: &str) -> &str {
    let re_single = Regex::new(r"(?m)^\s*```[A-Za-z0-9_+.-]*").unwrap();

    let start = re_single
        .find_iter(text)
        .next()
        .map(|m| m.end())
        .unwrap_or(0);

    let out = text.get(start..).unwrap_or(text);
    let out = if out.is_empty() { text } else { out };
    out.trim_end_matches("```").trim()
}

/// Returns the text inside the specified XML-like tag `<tag></tag>`.
///
/// If no tag is found, returns the input unchanged.
pub(crate) fn extract_from_tag<'a>(text: &'a str, tag: &str) -> &'a str {
    let tag = regex::escape(tag);
    let re_start = Regex::new(&format!(r"<{tag}>")).unwrap();
    let re_end = Regex::new(&format!(r"</{tag}>")).unwrap();

    let start = re_start.find(text).map(|m| m.end()).unwrap_or(0);

    let end = re_end
        .find_iter(&text[start..])
        .last()
        .map(|m| start + m.start())
        .unwrap_or(text.len());

    text[start..end].trim()
}

/// Returns all valid JSON objects or arrays found in the input text.
///
/// If no valid JSON is found, returns the input unchanged.
pub fn extract_jsons(text: &str) -> Vec<&str> {
    if text.is_empty() {
        return Vec::new();
    }

    let text = extract_from_codeblock(text);
    let bytes = text.as_bytes();

    let mut out = Vec::new();
    let mut stack: Vec<u8> = Vec::new();
    let mut start: Option<usize> = None;

    let mut in_string = false;
    let mut escaped = false;

    let mut i = 0;

    while i < bytes.len() {
        let b = bytes[i];

        if in_string {
            match b {
                b'\\' if !escaped => escaped = true,
                b'"' if !escaped => in_string = false,
                _ => escaped = false,
            }
            i += 1;
            continue;
        }

        if b == b'<' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
            if let Some(s) = start.take() {
                out.push(&text[s..i]);
            }
            stack.clear();

            i += 2;
            while i < bytes.len() && bytes[i] != b'>' && !bytes[i].is_ascii_whitespace() {
                i += 1;
            }
            continue;
        }

        match b {
            b'"' => in_string = true,

            b'{' => {
                if stack.is_empty() {
                    start = Some(i);
                }
                stack.push(b'}');
            }
            b'[' => {
                if stack.is_empty() {
                    start = Some(i);
                }
                stack.push(b']');
            }

            b'}' | b']' => match stack.pop() {
                Some(expected) if expected == b => {
                    if stack.is_empty()
                        && let Some(s) = start.take()
                    {
                        out.push(&text[s..=i]);
                    }
                }
                Some(_) => {
                    log::warn!("Mismatched closing brace found.");
                    stack.clear();
                    start = None;
                }
                None => {
                    log::warn!("Unmatched closing brace found.");
                    start = None;
                }
            },
            _ => (),
        }
        i += 1;
    }

    out
}

/// Returns the text preceding the given JSON.
pub(crate) fn extract_preamble<'a>(text: &'a str, json: &str) -> &'a str {
    let re_codeblock = Regex::new(r"\s*```[\w+-]*").unwrap();
    let re_html_tag = Regex::new(r#"<\s*(\/)?[A-Za-z][A-Za-z0-9:_-]*(?:\s+[^<>]*)?\s*>"#).unwrap();

    let Some(thought_end) = text.find(json) else {
        return "";
    };
    let thought = text[..thought_end].trim();

    let codeblock_start = re_codeblock
        .find_iter(thought)
        .last()
        .map(|m| m.start())
        .unwrap_or(thought.len());
    let mut trimmed = thought[..codeblock_start].trim();

    while !trimmed.is_empty()
        && let Some(m) = re_html_tag.find_iter(trimmed).last()
        && m.end() == trimmed.len()
    {
        trimmed = trimmed[..m.start()].trim_end();
    }
    trimmed
}

#[cfg(test)]
mod tests {
    use indoc::indoc;
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case(indoc! {r#"
        ```json
        {
            "key": "value"
        }
        ```"#
    }, indoc! {r#"
        {
            "key": "value"
        }"#
    })]
    #[case(indoc! {r#"
        Pikachu

        ```json
        {
            "key": "value"
        }
        ```"#
    }, indoc! {r#"
        {
            "key": "value"
        }"#
    })]
    #[case(indoc! {r#"
        ```json
        {
            "key": "value"
        }"#
    }, indoc! {r#"
        {
            "key": "value"
        }"#
    })]
    #[case(indoc! {r#"
        {
            "key": "value"
        }
        ```"#
    }, indoc! {r#"
        {
            "key": "value"
        }"#
    })]
    fn test_extract_from_codeblock(#[case] text: &str, #[case] expected: &str) {
        let extracted = extract_from_codeblock(text);
        assert_eq!(extracted, expected);
    }

    #[rstest]
    #[case(
        r#"<tool_call> {"name": "test_tool", "arguments": {"arg1": "value1"}}</tool_call>"#,
        "tool_call",
        r#"{"name": "test_tool", "arguments": {"arg1": "value1"}}"#
    )]
    #[case(
        r#"<tool_call> {"name": "test_tool", "arguments": {"arg1": "value1"}}"#,
        "tool_call",
        r#"{"name": "test_tool", "arguments": {"arg1": "value1"}}"#
    )]
    #[case(
        r#"{"name": "test_tool", "arguments": {"arg1": "value1"}}</tool_call>"#,
        "tool_call",
        r#"{"name": "test_tool", "arguments": {"arg1": "value1"}}"#
    )]
    #[case(
        r#"{"name": "test_tool", "arguments": {"arg1": "value1"}}"#,
        "tool_call",
        r#"{"name": "test_tool", "arguments": {"arg1": "value1"}}"#
    )]
    #[case(
        r#"{"name": "test_tool", "arguments": {"arg1": "value1"}}</tool_call> <FINAL_ANSWER_FORMAT>final answer"#,
        "tool_call",
        r#"{"name": "test_tool", "arguments": {"arg1": "value1"}}"#
    )]
    #[case(
        r#"</tool_call> stray closer then <tool_call>{"name": "test_tool"}"#,
        "tool_call",
        r#"{"name": "test_tool"}"#
    )]
    #[case(
        indoc! {r#"
            <tool_call>
            {
                "name": "test_tool",
                "arguments": {
                    "arg1": "value1"
                }
            }
            </tool_call>"#
        },
        "tool_call",
        indoc! {r#"
        {
            "name": "test_tool",
            "arguments": {
                "arg1": "value1"
            }
        }"#
    },
    )]
    fn test_extract_from_tag(#[case] text: &str, #[case] tag: &str, #[case] expected: &str) {
        let extracted = extract_from_tag(text, tag);
        assert_eq!(extracted, expected);
    }

    #[rstest]
    #[case(
        indoc! {r#"
            So I decided to call this because of that:

            {
                "name": "test_tool",
                "arguments": {
                    "arg1": "value1"
                }
            }"#
        },
        vec![indoc! {r#"
            {
                "name": "test_tool",
                "arguments": {
                    "arg1": "value1"
                }
            }"#
        }]
    )]
    #[case(
        indoc! {r#"
            Here are some of the cities in East Asia:

            [
                {
                    "country": "South Korea",
                    "city": "Seoul",
                },
                {
                    "country": "Japan",
                    "city": "Tokyo",
                }
            ]"#
        },
        vec![indoc! {r#"
            [
                {
                    "country": "South Korea",
                    "city": "Seoul",
                },
                {
                    "country": "Japan",
                    "city": "Tokyo",
                }
            ]"#
        }]
    )]
    #[case(
        indoc!{r#"
            <paper_search>
            {
                "name": "paper_search",
                "argument": {
                    "query": "some search \" \" a keyword"
                }
            }
            </paper_search>
            <paper_search>
            {
                "name": "paper_search",
                "arguments": {
                    "query": "some extra name"
                }</p>
            </paper_search>
            <paper_search>
            {
                "name": "paper_search",
                "arguments": {
                    "query": "some other dna name"
                }
            }
            </paper_search>
            <paper_search>
            {
                "name": "paper_search",
                "arguments": {
                    "query": "some dna name"
                }
            }
            </paper_search>
            <paper_search>
            {
                "name": "paper_search",
                "arguments": {
                    "query": "search {] keyword other"
                }
            }
            </paper_search>"#,
        },
        vec![
            indoc! {r#"
                {
                    "name": "paper_search",
                    "argument": {
                        "query": "some search \" \" a keyword"
                    }
                }"#
            },
            indoc! {r#"
                {
                    "name": "paper_search",
                    "arguments": {
                        "query": "some extra name"
                    }"#
            },
            indoc! {r#"
                {
                    "name": "paper_search",
                    "arguments": {
                        "query": "some other dna name"
                    }
                }"#
            },
            indoc! {r#"
                {
                    "name": "paper_search",
                    "arguments": {
                        "query": "some dna name"
                    }
                }"#
            },
            indoc! {r#"
                {
                    "name": "paper_search",
                    "arguments": {
                        "query": "search {] keyword other"
                    }
                }"#
            },
        ]
    )]
    fn test_extract_jsons(#[case] text: &str, #[case] expected: Vec<&str>) {
        let extracted = extract_jsons(text);
        assert_eq!(extracted, expected);
    }

    #[rstest]
    #[case(
        indoc! {r#"
            I think I should use the tool because of that.

            {
                "name": "test_tool",
                "arguments": {
                    "arg1": "value1"
                }
            }
            "#
        },
        indoc! {r#"
            {
                "name": "test_tool",
                "arguments": {
                    "arg1": "value1"
                }
            }
            "#
        },
        "I think I should use the tool because of that."
    )]
    #[case(
        indoc! {r#"
            I think I should use the tool because of that.

            ```json
            {
                "name": "test_tool",
                "arguments": {
                    "arg1": "value1"
                }
            }
            ```
            "#
        },
        indoc! {r#"
            {
                "name": "test_tool",
                "arguments": {
                    "arg1": "value1"
                }
            }
            "#
        },
        "I think I should use the tool because of that."
    )]
    #[case(
        indoc! {r#"
            ```json
            {
                "name": "test_tool",
                "arguments": {
                    "arg1": "value1"
                }
            }
            ```
            "#
        },
        indoc! {r#"
            {
                "name": "test_tool",
                "arguments": {
                    "arg1": "value1"
                }
            }
            "#
        },
        ""
    )]
    fn test_extract_thought(#[case] text: &str, #[case] json: &str, #[case] expected: &str) {
        let extracted = extract_preamble(text, json);
        assert_eq!(extracted, expected);
    }
}
