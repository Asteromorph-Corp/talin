use indexmap::IndexMap;
use serde::Deserialize;

use crate::skill::SkillError;

const MAX_NAME_LEN: usize = 64;
const MAX_DESCRIPTION_LEN: usize = 1024;
const MAX_COMPATIBILITY_LEN: usize = 500;

/// The metadata of a skill, as declared in the frontmatter of its `SKILL.md`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct SkillMetadata {
    name: String,
    description: String,
    license: Option<String>,
    compatibility: Option<String>,
    #[serde(default, deserialize_with = "deserialize_metadata")]
    metadata: IndexMap<String, String>,
    #[serde(rename = "allowed-tools")]
    allowed_tools: Option<String>,
}

/// A `metadata` value.
///
/// The specification says values are strings, but unquoted scalars such as `version: 1` are common
/// in the wild, so numbers and booleans are accepted and stringified. Nested values are rejected.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum MetadataValue {
    String(String),
    Integer(i64),
    Float(f64),
    Bool(bool),
}

impl SkillMetadata {
    /// Creates the metadata of a skill with just the required fields.
    ///
    /// Fails if the name or description violates the specification.
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
    ) -> Result<Self, SkillError> {
        Self {
            name: name.into(),
            description: description.into(),
            license: None,
            compatibility: None,
            metadata: IndexMap::new(),
            allowed_tools: None,
        }
        .validated()
    }

    /// Parses the metadata and the markdown body out of the contents of a `SKILL.md`.
    pub fn from_markdown(markdown: &str) -> Result<(Self, &str), SkillError> {
        let (header, body) = split(markdown)?;
        Ok((Self::parse(header)?, body))
    }

    /// The skill's name, which identifies it.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// What the skill does and when to use it.
    pub fn description(&self) -> &str {
        &self.description
    }

    /// The license name, or a reference to a bundled license file.
    pub fn license(&self) -> Option<&str> {
        self.license.as_deref()
    }

    /// Environment requirements, such as intended product, system packages, or network access.
    pub fn compatibility(&self) -> Option<&str> {
        self.compatibility.as_deref()
    }

    /// Arbitrary additional properties, in the order they were declared.
    pub fn metadata(&self) -> &IndexMap<String, String> {
        &self.metadata
    }

    /// The raw space-separated `allowed-tools` string (experimental in the specification).
    pub fn allowed_tools(&self) -> Option<&str> {
        self.allowed_tools.as_deref()
    }

    fn parse(yaml: &str) -> Result<Self, SkillError> {
        yaml_serde::from_str::<Self>(yaml)?.validated()
    }

    fn validated(self) -> Result<Self, SkillError> {
        validate_name(&self.name)?;
        check_len("description", &self.description, MAX_DESCRIPTION_LEN)?;
        if let Some(compatibility) = &self.compatibility {
            check_len("compatibility", compatibility, MAX_COMPATIBILITY_LEN)?;
        }
        Ok(self)
    }
}

impl MetadataValue {
    fn into_string(self) -> String {
        match self {
            Self::String(s) => s,
            Self::Integer(i) => i.to_string(),
            Self::Float(f) => f.to_string(),
            Self::Bool(b) => b.to_string(),
        }
    }
}

fn deserialize_metadata<'de, D>(deserializer: D) -> Result<IndexMap<String, String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let map: IndexMap<String, MetadataValue> = IndexMap::deserialize(deserializer)?;
    map.into_iter()
        .map(|(k, v)| Ok((k, v.into_string())))
        .collect()
}

/// Splits a `SKILL.md` document into its frontmatter and markdown body.
fn split(source: &str) -> Result<(&str, &str), SkillError> {
    let source = source.strip_prefix('\u{feff}').unwrap_or(source);
    let rest = source
        .strip_prefix("---")
        .and_then(|r| r.strip_prefix("\r\n").or_else(|| r.strip_prefix('\n')))
        .ok_or(SkillError::MissingFrontmatter)?;

    let mut offset = 0;
    for line in rest.split_inclusive('\n') {
        if line.trim_end() == "---" {
            let header = &rest[..offset];
            let body = &rest[offset + line.len()..];
            return Ok((header, body));
        }
        offset += line.len();
    }
    Err(SkillError::MissingFrontmatter)
}

fn validate_name(name: &str) -> Result<(), SkillError> {
    check_len("name", name, MAX_NAME_LEN)?;
    let invalid = |reason| SkillError::invalid_field("name", name, reason);
    if !name
        .chars()
        .all(|c| c == '-' || (c.is_alphanumeric() && !c.is_uppercase()))
    {
        return Err(invalid(
            "may only contain lowercase letters, digits, and hyphens",
        ));
    }
    if name.starts_with('-') || name.ends_with('-') {
        return Err(invalid("must not start or end with a hyphen"));
    }
    if name.contains("--") {
        return Err(invalid("must not contain consecutive hyphens"));
    }
    Ok(())
}

/// Checks that `value` is non-blank and at most `max` characters long.
fn check_len(field: &'static str, value: &str, max: usize) -> Result<(), SkillError> {
    if value.trim().is_empty() {
        return Err(SkillError::invalid_field(field, value, "must not be empty"));
    }
    let len = value.chars().count();
    if len > max {
        return Err(SkillError::invalid_field(
            field,
            value,
            format!("is {len} characters long, must be at most {max}"),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use indoc::{formatdoc, indoc};

    use super::*;

    #[test]
    fn splits_header_and_body() {
        let source = indoc! {
            "---
            name: pdf
            description: Work with PDFs
            ---
            # Body
            "
        };
        let (header, body) = split(source).unwrap();
        assert_eq!(header, "name: pdf\ndescription: Work with PDFs\n");
        assert_eq!(body, "# Body\n");
    }

    #[test]
    fn split_rejects_missing_frontmatter() {
        let err = split("# no header").unwrap_err();
        assert!(matches!(err, SkillError::MissingFrontmatter));
        let err = split("---\nname: x\n").unwrap_err();
        assert!(matches!(err, SkillError::MissingFrontmatter));
    }

    #[test]
    fn from_markdown_returns_the_body() {
        let (metadata, body) =
            SkillMetadata::from_markdown("---\nname: pdf\ndescription: d\n---\n# Body\n").unwrap();
        assert_eq!(metadata.name(), "pdf");
        assert_eq!(metadata.description(), "d");
        assert_eq!(body, "# Body\n");
    }

    #[test]
    fn new_validates_like_the_frontmatter() {
        assert_eq!(SkillMetadata::new("pdf", "d").unwrap().name(), "pdf");
        assert!(SkillMetadata::new("PDF", "d").is_err());
        assert!(SkillMetadata::new("pdf", "").is_err());
    }

    #[test]
    fn parses_all_fields() {
        let source = indoc! {
            "name: pdf
            description: d
            license: MIT
            compatibility: Needs git
            allowed-tools: Bash(git:*) Read
            metadata:
              author: me
              version: 1
              stable: true"
        };
        let fm = SkillMetadata::parse(source).unwrap();
        assert_eq!(fm.description, "d");
        assert_eq!(fm.license.as_deref(), Some("MIT"));
        assert_eq!(fm.compatibility.as_deref(), Some("Needs git"));
        assert_eq!(fm.allowed_tools.as_deref(), Some("Bash(git:*) Read"));
        let metadata: Vec<_> = fm.metadata.into_iter().collect();
        assert_eq!(
            metadata,
            [
                ("author".into(), "me".into()),
                ("version".into(), "1".into()),
                ("stable".into(), "true".into())
            ]
        );
    }

    #[test]
    fn ignores_unknown_fields() {
        let source = indoc! {
            "name: pdf
            description: Work with PDFs
            future-field: [1, 2]"
        };
        let fm = SkillMetadata::parse(source).unwrap();
        assert_eq!(fm.name, "pdf");
    }

    #[test]
    fn rejects_missing_required_fields() {
        let err = SkillMetadata::parse("description: x").unwrap_err();
        assert!(matches!(err, SkillError::InvalidFrontmatter(_)));
        let err = SkillMetadata::parse("name: x").unwrap_err();
        assert!(matches!(err, SkillError::InvalidFrontmatter(_)));
    }

    #[test]
    fn rejects_nested_metadata_values() {
        let source = indoc! {
            "name: pdf
            description: d
            metadata:
              a:
                b: c"
        };
        let err = SkillMetadata::parse(source).unwrap_err();
        assert!(matches!(err, SkillError::InvalidFrontmatter(_)));
    }

    #[test]
    fn validates_names() {
        for bad in [
            "PDF",
            "a_b",
            "-a",
            "a-",
            "a--b",
            "has space",
            &"a".repeat(65),
        ] {
            assert!(validate_name(bad).is_err(), "{bad} should be rejected");
        }
        for good in ["pdf", "a-b", "x1", "a-1-b", "données", "データ"] {
            assert!(validate_name(good).is_ok(), "{good} should be accepted");
        }
    }

    #[test]
    fn enforces_length_limits() {
        let source = formatdoc! {
            "name: pdf
            description: {}",
            "x".repeat(1025)
        };
        let err = SkillMetadata::parse(&source).unwrap_err();
        assert!(matches!(
            err,
            SkillError::InvalidField {
                field: "description",
                ..
            }
        ));

        let source = formatdoc! {
            "name: pdf
            description: d
            compatibility: {}",
            "x".repeat(501)
        };
        let err = SkillMetadata::parse(&source).unwrap_err();
        assert!(matches!(
            err,
            SkillError::InvalidField {
                field: "compatibility",
                ..
            }
        ));
    }

    #[test]
    fn rejects_empty_values() {
        for (field, source) in [
            ("name", "name: \"\"\ndescription: d"),
            ("description", "name: pdf\ndescription: \"\""),
            ("description", "name: pdf\ndescription: \"  \""),
            (
                "compatibility",
                "name: pdf\ndescription: d\ncompatibility: \"\"",
            ),
        ] {
            let err = SkillMetadata::parse(source).unwrap_err();
            assert!(
                matches!(&err, SkillError::InvalidField { field: f, reason, .. }
                    if *f == field && reason == "must not be empty"),
                "{source:?} gave {err}"
            );
        }
    }

    #[test]
    fn truncates_long_values_in_errors() {
        let source = format!("name: pdf\ndescription: {}", "x".repeat(1025));
        let message = SkillMetadata::parse(&source).unwrap_err().to_string();
        assert!(message.len() < 200, "{message}");
        assert!(message.contains("1025 characters"), "{message}");
    }
}
