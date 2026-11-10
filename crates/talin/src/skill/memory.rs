use std::collections::BTreeMap;

use async_trait::async_trait;

use crate::skill::{Skill, SkillError, SkillMetadata};

/// A [`Skill`] held in memory.
#[derive(Debug, Clone)]
pub struct MemorySkill {
    metadata: SkillMetadata,
    instructions: String,
    files: BTreeMap<String, String>,
}

impl MemorySkill {
    /// Creates a skill from its metadata and instructions.
    pub fn new(metadata: SkillMetadata, instructions: impl Into<String>) -> Self {
        Self {
            metadata,
            instructions: instructions.into(),
            files: BTreeMap::new(),
        }
    }

    /// Creates a skill from the contents of a `SKILL.md`.
    pub fn from_markdown(markdown: &str) -> Result<Self, SkillError> {
        let (metadata, body) = SkillMetadata::from_markdown(markdown)?;
        Ok(Self::new(metadata, body.trim()))
    }

    /// Adds a bundled file.
    pub fn with_file(mut self, path: impl Into<String>, content: impl Into<String>) -> Self {
        self.files.insert(path.into(), content.into());
        self
    }
}

#[async_trait]
impl Skill for MemorySkill {
    fn name(&self) -> &str {
        self.metadata.name()
    }

    async fn metadata(&self) -> Result<SkillMetadata, SkillError> {
        Ok(self.metadata.clone())
    }

    async fn instructions(&self) -> Result<String, SkillError> {
        Ok(self.instructions.clone())
    }

    async fn files(&self) -> Result<Vec<String>, SkillError> {
        Ok(self.files.keys().cloned().collect())
    }

    async fn read_file(&self, path: &str) -> Result<String, SkillError> {
        self.files
            .get(path.strip_prefix("./").unwrap_or(path))
            .cloned()
            .ok_or_else(|| SkillError::FileNotFound(path.to_owned()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn serves_contents() {
        let skill = MemorySkill::from_markdown("---\nname: pdf\ndescription: d\n---\n# Body\n")
            .unwrap()
            .with_file("references/b.md", "b")
            .with_file("references/a.md", "a");

        assert_eq!(skill.name(), "pdf");
        assert_eq!(skill.metadata().await.unwrap().description(), "d");
        assert_eq!(skill.instructions().await.unwrap(), "# Body");
        assert_eq!(
            skill.files().await.unwrap(),
            ["references/a.md", "references/b.md"]
        );
        assert_eq!(skill.read_file("references/a.md").await.unwrap(), "a");
        assert_eq!(skill.read_file("./references/a.md").await.unwrap(), "a");
        assert!(matches!(
            skill.read_file("../a.md").await,
            Err(SkillError::FileNotFound(_))
        ));
        assert!(skill.notes().is_none());
    }

    #[test]
    fn rejects_invalid_markdown() {
        assert!(MemorySkill::from_markdown("no frontmatter").is_err());
    }
}
