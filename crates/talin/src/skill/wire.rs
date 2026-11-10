//! The wires that connect the model to skills. Used for providers without native skill support.
//!
//! Consists of two tools and a system edit scheme:
//! - `load_skill` loads a skill's instructions into the context when the model decides to use it.
//! - `read_skill_file` reads a file bundled with a skill, such as a reference
//! - `InjectCatalog` appends the skill catalog to the system prompt so the model can see what
//!   skills are available.

use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;

use async_trait::async_trait;
use indoc::formatdoc;
use schemars::JsonSchema;
use serde::Deserialize;

use crate::skill::{SkillMetadata, SkillSet};
use crate::tools::{Function, Tool};
use crate::transcript::edit::SystemEditScheme;

/// The name of the tool that loads a skill's instructions.
pub(crate) const LOAD_SKILL: &str = "load_skill";
/// The name of the tool that reads a file bundled with a skill.
pub(crate) const READ_SKILL_FILE: &str = "read_skill_file";

pub(crate) fn skill_tools(skills: &Arc<SkillSet>) -> HashMap<String, Tool> {
    if skills.is_empty() {
        return HashMap::new();
    }
    Tool::build_index(vec![
        Tool::from(LoadSkill::new(Arc::clone(skills))),
        Tool::from(ReadSkillFile::new(Arc::clone(skills))),
    ])
}

struct LoadSkill {
    skills: Arc<SkillSet>,
}

#[derive(Deserialize, JsonSchema)]
struct LoadSkillInput {
    /// The name of the skill to load, as listed in the available skills.
    name: String,
}

impl LoadSkill {
    /// Creates a new [`LoadSkill`] tool that loads skills from the given set.
    pub(crate) fn new(skills: Arc<SkillSet>) -> Self {
        Self { skills }
    }
}

#[async_trait]
impl Function for LoadSkill {
    type Input = LoadSkillInput;
    type Output = String;

    fn name(&self) -> String {
        LOAD_SKILL.to_owned()
    }

    fn description(&self) -> String {
        "Load the full instructions of an available skill. Call this when a task matches a \
         skill's description before attempting the task."
            .to_owned()
    }

    async fn call(
        &self,
        input: Self::Input,
    ) -> Result<Self::Output, Box<dyn std::error::Error + Send + Sync>> {
        let skill = self
            .skills
            .get(&input.name)
            .ok_or_else(|| not_found(&self.skills, &input.name))?;

        let instructions = skill.instructions().await?;
        let files = skill.files().await?;
        let files_info = (!files.is_empty()).then(|| {
            let file_names = files.join("\n");
            formatdoc! {
                "Read bundled files with the `{READ_SKILL_FILE}` tool:
                <skill_files>
                {file_names}
                </skill_files>",
            }
        });

        let extra_info = [skill.notes(), files_info]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .join("\n\n");
        let extra_info = if extra_info.is_empty() {
            extra_info
        } else {
            format!("\n\n{extra_info}")
        };

        Ok(formatdoc! {
            "<skill_content name=\"{}\">
            {}{}
            </skill_content>",
            skill.name(),
            instructions,
            extra_info,
        })
    }
}

struct ReadSkillFile {
    skills: Arc<SkillSet>,
}

#[derive(Deserialize, JsonSchema)]
struct ReadSkillFileInput {
    /// The name of the skill the file belongs to.
    skill: String,
    /// The path of the file, relative to the skill directory.
    path: String,
}

impl ReadSkillFile {
    /// Creates a new [`ReadSkillFile`] tool that reads files from the given set of skills.
    pub(crate) fn new(skills: Arc<SkillSet>) -> Self {
        Self { skills }
    }
}

#[async_trait]
impl Function for ReadSkillFile {
    type Input = ReadSkillFileInput;
    type Output = String;

    fn name(&self) -> String {
        READ_SKILL_FILE.to_owned()
    }

    fn description(&self) -> String {
        "Read a file bundled with a skill, such as a reference document or template.".to_owned()
    }

    async fn call(
        &self,
        input: Self::Input,
    ) -> Result<Self::Output, Box<dyn std::error::Error + Send + Sync>> {
        let skill = self
            .skills
            .get(&input.skill)
            .ok_or_else(|| not_found(&self.skills, &input.skill))?;
        Ok(skill.read_file(&input.path).await?)
    }
}

fn not_found(skills: &SkillSet, name: &str) -> String {
    let available = skills.iter().map(|s| s.name()).collect::<Vec<_>>();
    format!(
        "skill `{name}` not found; available skills: {}",
        available.join(", ")
    )
}

/// A [`SystemEditScheme`] that appends the skill catalog to the system prompt.
pub(crate) struct InjectCatalog<'a>(pub(crate) &'a [SkillMetadata]);

#[async_trait]
impl SystemEditScheme for InjectCatalog<'_> {
    type Error = Infallible;

    async fn edit_system(&self, contents: &[String]) -> Result<Option<Vec<String>>, Self::Error> {
        if self.0.is_empty() {
            return Ok(None);
        }
        let mut contents = contents.to_vec();
        contents.push(catalog(self.0));
        Ok(Some(contents))
    }
}

fn catalog(skills: &[SkillMetadata]) -> String {
    formatdoc! {
        "# Skills
        Skills provide specialized instructions for particular tasks. \
        When a task matches a skill's description, call the `{LOAD_SKILL}` tool \
        with its name to load the full instructions, then follow them.

        <available_skills>
        {}
        </available_skills>",
        skills.iter().map(entry).collect::<Vec<_>>().join("\n")
    }
}

fn entry(skill: &SkillMetadata) -> String {
    formatdoc! {
        "<skill>
        <name>{}</name>
        <description>{}</description>
        </skill>",
        escape(skill.name()),
        escape(skill.description()),
    }
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;
    use crate::skill::{FsSkill, Skill};

    async fn skill_set(dir: &Path) -> Arc<SkillSet> {
        let root = dir.join("pdf");
        tokio::fs::create_dir_all(root.join("references"))
            .await
            .unwrap();
        tokio::fs::write(
            root.join("SKILL.md"),
            "---\nname: pdf\ndescription: Handle <PDF> files\n---\nDo the thing.\n",
        )
        .await
        .unwrap();
        tokio::fs::write(root.join("references/guide.md"), "guide text")
            .await
            .unwrap();
        let skill: Arc<dyn Skill> = Arc::new(FsSkill::new(&root).load().await.unwrap());
        Arc::new(SkillSet::from_iter([skill]))
    }

    fn tempdir() -> std::path::PathBuf {
        std::env::temp_dir().join(format!("pe-wire-{}", uuid::Uuid::new_v4()))
    }

    #[tokio::test]
    async fn system_updated_by_catalog() {
        let tmp = tempdir();
        let skills = skill_set(&tmp).await;
        let metadata = skills.metadata().await.unwrap();
        let out = InjectCatalog(&metadata)
            .edit_system(&["base".to_owned()])
            .await
            .unwrap()
            .unwrap();
        assert_eq!(out[0], "base");
        assert!(out[1].contains("<name>pdf</name>"));
        assert!(out[1].contains("Handle &lt;PDF&gt; files"));
        tokio::fs::remove_dir_all(tmp).await.unwrap();
    }

    #[tokio::test]
    async fn system_unchanged_for_empty_catalog() {
        let out = InjectCatalog(&[])
            .edit_system(&["base".to_owned()])
            .await
            .unwrap();
        assert!(out.is_none());
    }

    #[tokio::test]
    async fn load_skill_returns_skill() {
        let tmp = tempdir();
        let skills = skill_set(&tmp).await;
        let tool = LoadSkill { skills };
        let out = Function::call(&tool, LoadSkillInput { name: "pdf".into() })
            .await
            .unwrap();
        assert!(out.contains("Do the thing."));
        assert!(out.contains("Relative paths in this skill are relative to"));
        assert!(out.contains("references/guide.md"));

        let err = Function::call(
            &tool,
            LoadSkillInput {
                name: "nope".into(),
            },
        )
        .await
        .unwrap_err();
        assert!(err.to_string().contains("available skills: pdf"));
        tokio::fs::remove_dir_all(tmp).await.unwrap();
    }

    #[tokio::test]
    async fn load_skill_reports_read_failure() {
        let skill: Arc<dyn Skill> = Arc::new(FsSkill::new("/dir/broken"));
        let skills = Arc::new(SkillSet::from_iter([skill]));
        let tool = LoadSkill { skills };
        let err = Function::call(
            &tool,
            LoadSkillInput {
                name: "broken".into(),
            },
        )
        .await
        .unwrap_err();
        assert!(err.to_string().contains("/dir/broken"), "{err}");
    }
}
