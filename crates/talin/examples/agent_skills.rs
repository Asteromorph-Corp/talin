//! An agent with skills.
//!
//! The model sees each skill's name and description, and loads the full instructions on demand.
//! This example writes a throwaway skill to a temporary directory, then loads it with
//! [`SkillSet::discover`].
//!
//! Run with `cargo run --example agent_skills`.

use std::error::Error;

use talin::agent::Agent;
use talin::llm::ClaudeMessages;
use talin::skill::{FsSkill, SkillSet};

const SKILL: &str = "\
---
name: haiku-reviewer
description: Reviews haiku for the 5-7-5 syllable pattern. Use when the user shares a haiku.
---
# Reviewing a haiku

1. Count the syllables of each line.
2. Report whether the poem follows the 5-7-5 pattern.
3. Suggest one concrete fix for each line that does not.
";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv().ok();
    env_logger::init();

    let root = std::env::temp_dir().join("talin-skills-example");
    let skill_dir = root.join("haiku-reviewer");
    tokio::fs::create_dir_all(&skill_dir).await?;
    tokio::fs::write(skill_dir.join("SKILL.md"), SKILL).await?;

    let mut skills = SkillSet::new();
    skills.insert(FsSkill::new(&skill_dir));
    let llm = ClaudeMessages::builder()
        .with_model("claude-haiku-5-5")
        .build();
    let agent: Agent<_> = Agent::builder().skills(skills).build(llm);

    let answer = agent
        .run("Is this a proper haiku?\n\nAn old silent pond / A frog jumps into the pond / Splash! Silence again.")
        .await?;
    println!("{answer}");
    Ok(())
}
