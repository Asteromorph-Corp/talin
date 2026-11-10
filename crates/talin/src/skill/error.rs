use thiserror::Error;

/// Errors that can occur while loading or reading a [`Skill`](crate::skill::Skill).
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum SkillError {
    /// An error specific to the [`Skill`](crate::skill::Skill) implementation.
    #[error("skill source error: {0}")]
    Source(#[source] Box<dyn std::error::Error + Send + Sync>),

    /// The `SKILL.md` file has no valid frontmatter block.
    #[error("missing frontmatter")]
    MissingFrontmatter,

    /// The frontmatter is not valid YAML or has a field of the wrong type.
    #[error("invalid frontmatter: {0}")]
    InvalidFrontmatter(#[from] yaml_serde::Error),

    /// A frontmatter field has an invalid value.
    #[error("`{field}` {reason} (got {value:?})")]
    InvalidField {
        /// The name of the field.
        field: &'static str,
        /// The value of the field, truncated if long.
        value: String,
        /// Why the field was rejected, phrased as a predicate on the field.
        reason: String,
    },

    /// The requested file does not exist in the skill.
    #[error("file not found in skill: {0}")]
    FileNotFound(String),

    /// The requested skill does not exist.
    #[error("skill not found: {0}")]
    NotFound(String),
}

impl SkillError {
    pub(crate) fn is_retryable(&self) -> bool {
        matches!(self, Self::Source(_))
    }

    pub(crate) fn invalid_field(
        field: &'static str,
        value: &str,
        reason: impl Into<String>,
    ) -> Self {
        const MAX_VALUE_CHARS: usize = 64;
        let mut chars = value.chars();
        let mut value: String = chars.by_ref().take(MAX_VALUE_CHARS).collect();
        if chars.next().is_some() {
            value.push('…');
        }
        Self::InvalidField {
            field,
            value,
            reason: reason.into(),
        }
    }
}
