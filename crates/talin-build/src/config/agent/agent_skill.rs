use crate::error::AgentSkillError;

/// Checks an entry of an agent's `skills` list.
///
/// An entry is the name of a skill. It is looked up in the skills home at runtime, which depends on
/// the machine the program runs on, so whether the skill exists is only known then.
pub(super) fn validate_skill_name(entry: &str) -> Result<(), AgentSkillError> {
    if entry.trim().is_empty() {
        return Err(AgentSkillError::Empty);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case::name("pdf")]
    #[case::hyphenated("web-search")]
    fn accepts_names(#[case] entry: &str) {
        assert_eq!(validate_skill_name(entry), Ok(()));
    }

    #[rstest]
    #[case::empty("", AgentSkillError::Empty)]
    #[case::blank("  ", AgentSkillError::Empty)]
    fn rejects_everything_else(#[case] entry: &str, #[case] expected: AgentSkillError) {
        assert_eq!(validate_skill_name(entry), Err(expected));
    }
}
