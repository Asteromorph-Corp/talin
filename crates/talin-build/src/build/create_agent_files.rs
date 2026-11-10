use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use heck::ToSnakeCase;

use crate::build::write_file::write_file;
use crate::codegen::{generate_agent, generate_agent_index};
use crate::config::Agent;
use crate::error::IoError;
use crate::options::ArtifactOptions;

pub(super) fn create_agent_files(
    out_dir: &Path,
    agents: &BTreeMap<String, Agent>,
    options: &ArtifactOptions,
) -> Result<(), IoError> {
    let agents_dir = out_dir.join("agents");
    fs::create_dir_all(&agents_dir).map_err(|e| IoError::create_dir_failure(&agents_dir, e))?;

    agents.iter().try_for_each(|(id, agent)| {
        write_file!(
            agents_dir.join(format!("{}.rs", id.to_snake_case())),
            generate_agent(id, agent, options)
        )
    })?;

    write_file!(
        out_dir.join("agent_index.rs"),
        generate_agent_index(&agents_dir, agents),
    )
}
