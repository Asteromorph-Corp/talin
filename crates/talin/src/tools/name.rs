/// Normalize the tool names to avoid incorrect matches.
pub(crate) fn normalize_tool_name(name: &str) -> String {
    name.to_lowercase().replace(" ", "_")
}
