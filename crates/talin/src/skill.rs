//! [Skills](https://agentskills.io/specification) extend AI agent capabilities with specialized
//! knowledge and workflows.
//!
//! A skill is a folder containing a `SKILL.md` file.
//! `SKILL.md` starts with a YAML frontmatter block giving the skill's `name` and `description`,
//! followed by markdown instructions:
//!
//! ```text
//! ---
//! name: pdf
//! description: Extract text and tables from PDF files. Use when the user mentions PDFs.
//! license: Apache-2.0
//! ---
//! # PDF processing
//! ...
//! ```
//!
//! The model sees the skill's metadata up front, and may choose to load the instructions and files
//! if a task calls for them.
//!
//! [`Skill`] trait defines how to load a skill's metadata, instructions, and bundled files from
//! various sources. Skills can be fetched from a filesystem ([`FsSkill`]), from memory
//! ([`MemorySkill`]), or from an external skill registry.

use std::fmt::Debug;

use async_trait::async_trait;

mod error;
mod fs;
mod memory;
mod metadata;
mod set;
pub(crate) mod wire;

pub use error::SkillError;
pub use fs::{FsSkill, FsSkillError};
pub use memory::MemorySkill;
pub use metadata::SkillMetadata;
pub use set::SkillSet;

/// The file that defines a skill.
pub const SKILL_FILE: &str = "SKILL.md";

/// A skill that can be used extend agent AI capabilities with specialized knowledge and workflows.
///
/// The trait defines how to load a skill's metadata, instructions, and bundled files from it
/// sources:
/// - [`name`](Skill::name) returns the skill's name.
///
///
///
/// The model sees the skill's metadata up front, and loads the instructions
/// and files only when a task calls for them.
///
/// The model
/// sees the  metadata the model sees up front, and instructions and files it loads on
/// demand.
#[async_trait]
pub trait Skill: Debug + Send + Sync {
    /// The name that identifies the skill, used as a key in [`SkillSet`]
    ///
    /// Should be the same as [`SkillMetadata::name`].
    fn name(&self) -> &str;

    /// The skill's metadata obtained by parsing the YAML frontmatter of `SKILL.md`.
    async fn metadata(&self) -> Result<SkillMetadata, SkillError>;

    /// The markdown instructions of the skill, without the frontmatter.
    async fn instructions(&self) -> Result<String, SkillError>;

    /// Lists the files bundled with the skill, excluding `SKILL.md` itself.
    async fn files(&self) -> Result<Vec<String>, SkillError>;

    /// Reads a bundled file as UTF-8 text.
    async fn read_file(&self, path: &str) -> Result<String, SkillError>;

    /// Notes appended to the skill when the model loads it, e.g.  where relative paths resolve.
    fn notes(&self) -> Option<String> {
        None
    }
}
