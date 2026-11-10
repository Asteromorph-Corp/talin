use std::borrow::Cow;
use std::sync::Arc;

use futures::future::try_join_all;
use indexmap::IndexMap;

use crate::skill::{Skill, SkillError, SkillMetadata};

/// A collection of [`Skill`]s with distinct names, kept in insertion order.
#[derive(Debug, Clone, Default)]
pub struct SkillSet {
    skills: IndexMap<String, Arc<dyn Skill>>,
}

impl SkillSet {
    /// Creates an empty set.
    pub fn new() -> Self {
        Self::default()
    }

    /// The skills in the set together with `additional`, which win over the set's skills of the
    /// same name.
    ///
    /// Without `additional` skills, the set is borrowed as it is.
    pub(crate) fn with_additional(&self, additional: Option<SkillSet>) -> Cow<'_, SkillSet> {
        match additional {
            None => Cow::Borrowed(self),
            Some(additional) => Cow::Owned(self.clone().extended(additional)),
        }
    }

    pub(crate) fn extend(&mut self, other: SkillSet) {
        self.skills.extend(other.skills);
    }

    /// The skills in the set, with `other`'s added and winning over those of the same name.
    pub(crate) fn extended(mut self, other: SkillSet) -> SkillSet {
        self.extend(other);
        self
    }

    /// Adds a skill.
    ///
    /// If the set already has a skill with the same name, the existing skill is kept, like for
    /// tools, and a warning is logged. Returns whether the skill was added.
    pub fn insert(&mut self, skill: impl Skill + 'static) -> Option<Arc<dyn Skill>> {
        self.skills.insert(skill.name().to_owned(), Arc::new(skill))
    }

    /// Removes the skill with the given name, preserving the order of the remaining skills.
    pub fn remove(&mut self, name: &str) -> Option<Arc<dyn Skill>> {
        self.skills.shift_remove(name)
    }

    /// Looks up a skill by name.
    pub fn get(&self, name: &str) -> Option<&dyn Skill> {
        self.skills.get(name).map(Arc::as_ref)
    }

    /// Iterates over the skills in insertion order.
    pub fn iter(&self) -> impl Iterator<Item = &dyn Skill> {
        self.skills.values().map(Arc::as_ref)
    }

    /// The number of skills in the set.
    pub fn len(&self) -> usize {
        self.skills.len()
    }

    /// Whether the set contains no skills.
    pub fn is_empty(&self) -> bool {
        self.skills.is_empty()
    }

    /// The metadata of every skill, in order, fetched concurrently.
    ///
    /// Skills cache their metadata, so this is cheap after the first call. It is also how to check
    /// up front that every skill can be read: it fails with the error of the first skill that
    /// cannot.
    pub async fn metadata(&self) -> Result<Vec<SkillMetadata>, SkillError> {
        try_join_all(self.skills.values().map(|skill| skill.metadata())).await
    }
}

impl FromIterator<Arc<dyn Skill>> for SkillSet {
    fn from_iter<I: IntoIterator<Item = Arc<dyn Skill>>>(iter: I) -> Self {
        let skills = IndexMap::from_iter(
            iter.into_iter()
                .map(|skill| (skill.name().to_owned(), skill)),
        );
        Self { skills }
    }
}

impl PartialEq for SkillSet {
    fn eq(&self, other: &Self) -> bool {
        self.skills.len() == other.skills.len()
            && self
                .skills
                .iter()
                .zip(&other.skills)
                .all(|((a_name, a), (b_name, b))| a_name == b_name && Arc::ptr_eq(a, b))
    }
}

impl Eq for SkillSet {}

#[cfg(test)]
mod tests {
    use indoc::formatdoc;

    use super::*;
    use crate::skill::MemorySkill;

    fn memory(name: &str) -> MemorySkill {
        MemorySkill::from_markdown(&formatdoc! {
            "---
            name: {name}
            description: {name} skill
            ---
            body"
        })
        .unwrap()
    }

    #[test]
    fn sets_equality() {
        let mut set = SkillSet::new();
        set.insert(memory("pdf"));
        assert_eq!(set, set.clone());

        // A skill with the same content, but not the same skill.
        let mut other = SkillSet::new();
        other.insert(memory("pdf"));
        assert_ne!(set, other);
    }

    #[tokio::test]
    async fn metadata_lists_every_skill_in_order() {
        let mut set = SkillSet::new();
        set.insert(memory("b"));
        set.insert(memory("a"));
        let metadata = set.metadata().await.unwrap();
        let names: Vec<_> = metadata.iter().map(|m| m.name()).collect();
        assert_eq!(names, ["b", "a"]);
    }
}
