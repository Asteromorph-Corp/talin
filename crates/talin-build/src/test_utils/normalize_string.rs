/// Normalizes token-level noise that `prettyplease` reproduces verbatim but that doesn't
/// affect the meaning of the generated code, so two equivalent inputs compare equal.
///
/// `prettyplease` is not a canonical form. Most of its non-canonical output is fixed by
/// the caller's re-parse + re-print pass, but two kinds of noise live *inside* macro
/// invocations (e.g. `vec![..]`), whose token streams `prettyplease` prints using each
/// token's original spacing rather than re-formatting — so re-parsing can't reach them:
///
/// * Punctuation spacing: `quote!`-generated code spaces `>` differently from the same tokens
///   parsed out of a fixture (`> ::from_config` vs `>::from_config`).
/// * Trailing commas: a hand-written fixture keeps them (`from_config(arg,)`) while generated code
///   has none.
///
/// Both are reconciled here, textually.
pub(super) fn normalize_string(input: &str) -> String {
    let normalized = remove_double_colon_whitespace(input);
    remove_trailing_commas(&normalized)
}

/// Collapses whitespace around the path separator `::` (`Foo :: Bar` → `Foo::Bar`).
///
/// The leading character is captured and required to be neither whitespace nor `:`, so a
/// trait bound's `L: ::prompt` is left untouched instead of being mangled into the
/// unparseable `L:::prompt` — collapsing there would also wrongly fuse the bound colon
/// with the path's leading `::`.
pub(super) fn remove_double_colon_whitespace(s: &str) -> String {
    let re = regex::Regex::new(r"([^\s:])\s*::\s*").unwrap();
    re.replace_all(s, "${1}::").to_string()
}

pub(super) fn remove_trailing_commas(s: &str) -> String {
    let mut chars: Vec<char> = s.chars().collect();
    let mut i = 0;

    while i + 1 < chars.len() {
        if chars[i] == ',' {
            let mut j = i + 1;
            // Skip whitespace after the comma
            while j < chars.len() && chars[j].is_whitespace() {
                j += 1;
            }
            // If next significant char is a closing delimiter → remove comma (+ whitespace).
            // `>` closes a generic/type-argument list, e.g. `Foo::<Bar,>` vs `Foo::<Bar>`,
            // which `prettyplease` emits inconsistently depending on where it wraps.
            if j < chars.len() && matches!(chars[j], ')' | ']' | '}' | '>') {
                chars.drain(i..j);
                continue; // re-process index i since characters shifted
            }
        }
        i += 1;
    }

    chars.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_double_colon_whitespace() {
        let input = "Hello :: World";
        let expected = "Hello::World";
        assert_eq!(remove_double_colon_whitespace(input), expected);
    }

    #[test]
    fn test_remove_double_colon_whitespace_collapses_qualified_path() {
        // The `> ::` spacing emitted for generated code collapses to the fixture's `>::`;
        // the unrelated space before `>` is left alone.
        let input = "FromConfig > ::from_config";
        let expected = "FromConfig >::from_config";
        assert_eq!(remove_double_colon_whitespace(input), expected);
    }

    #[test]
    fn test_remove_double_colon_whitespace_preserves_trait_bound() {
        // `L: ::prompt` must not collapse to the unparseable `L:::prompt`.
        let input = "L: ::talin::llm::Llm";
        let expected = "L: ::talin::llm::Llm";
        assert_eq!(remove_double_colon_whitespace(input), expected);
    }

    #[test]
    fn test_remove_trailing_commas() {
        let input = "Hello, World, )";
        let expected = "Hello, World)";
        assert_eq!(remove_trailing_commas(input), expected);
    }

    #[test]
    fn test_remove_trailing_commas_before_generic_close() {
        let input = "from_str::<ClaudeBuiltinTool, >(...)";
        let expected = "from_str::<ClaudeBuiltinTool>(...)";
        assert_eq!(remove_trailing_commas(input), expected);
    }
}
