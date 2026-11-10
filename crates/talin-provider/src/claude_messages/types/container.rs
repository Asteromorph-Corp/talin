use serde::{Deserialize, Serialize};

use crate::utils::string_enum;

/// Information about the container used in a request.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct Container {
    /// Identifier for the container used in the request.
    ///
    /// See [`id`](https://docs.claude.com/en/api/messages#response-container-id)
    pub id: String,

    /// The time at which the container will expire.
    ///
    /// See [`expires_at`](https://docs.claude.com/en/api/messages#response-container-expires-at)
    pub expires_at: String,

    // Should be serialized as `null` if `None`.
    /// The skills loaded in the container.
    #[cfg_attr(test, schema(required = true))]
    pub skills: Option<Vec<ContainerSkill>>,
}

/// A skill that was loaded in a container (response model).
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ContainerSkill {
    /// Skill ID.
    pub skill_id: String,

    /// Type of skill.
    #[serde(rename = "type")]
    pub type_: SkillType,

    /// The resolved version: a skill version ID for custom skills.
    pub version: String,
}

/// Container identifier for reuse across requests.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum MessageCreateParamsContainer {
    /// Container identifier with additional parameters.
    Params(ContainerParams),
    /// Plain container identifier string.
    Id(String),
}

/// Container parameters with skills to be loaded
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[non_exhaustive]
pub struct ContainerParams {
    /// Container id.
    pub id: Option<String>,
    /// List of skills to load in the container.
    pub skills: Option<Vec<SkillParams>>,
}

/// Specification for a skill to be loaded in a container (request model).
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct SkillParams {
    /// Skill ID
    pub skill_id: String,
    /// Type of skill.
    #[serde(rename = "type")]
    pub type_: SkillType,
    /// Skill version or latest for most recent version.
    #[cfg_attr(test, schema(nullable = false))]
    pub version: Option<String>,
}

string_enum! {
    /// Type of skill.
    #[serde(rename_all = "lowercase")]
    pub enum SkillType {
        /// Built-in skill.
        Anthropic,

        /// User-defined skill.
        Custom,
    }
}

impl Container {
    /// Creates a new [`Container`] from the given id and expiry.
    pub fn new(id: impl Into<String>, expires_at: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            expires_at: expires_at.into(),
            skills: None,
        }
    }

    /// Sets the [`skills`](Self::skills) loaded in the container.
    pub fn with_skills(mut self, skills: Vec<ContainerSkill>) -> Self {
        self.skills = Some(skills);
        self
    }
}

impl ContainerSkill {
    /// Creates a new [`ContainerSkill`] from the given id, type, and version.
    pub fn new(skill_id: impl Into<String>, type_: SkillType, version: impl Into<String>) -> Self {
        Self {
            skill_id: skill_id.into(),
            type_,
            version: version.into(),
        }
    }
}

impl ContainerParams {
    /// Creates a new [`ContainerParams`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the container id.
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Sets the skills to be loaded in the container.
    pub fn with_skills(mut self, skills: Vec<SkillParams>) -> Self {
        self.skills = Some(skills);
        self
    }
}

impl SkillParams {
    /// Creates a new [`SkillParams`].
    pub fn new(skill_id: impl Into<String>, type_: SkillType) -> Self {
        Self {
            skill_id: skill_id.into(),
            type_,
            version: None,
        }
    }

    /// Sets the version of the skill.
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }
}
