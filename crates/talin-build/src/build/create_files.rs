use std::fs;
use std::path::Path;

use crate::build::create_agent_files::create_agent_files;
use crate::build::create_tool_files::create_tool_files;
use crate::build::write_file::write_file;
use crate::codegen::{generate_mcp_servers, generate_models};
use crate::config::Configs;
use crate::error::IoError;
use crate::options::ArtifactOptions;

pub(crate) fn create_files(
    out_dir: &Path,
    configs: &Configs,
    options: &ArtifactOptions,
) -> Result<(), IoError> {
    fs::create_dir_all(out_dir).map_err(|e| IoError::create_dir_failure(out_dir, e))?;

    write_file!(out_dir.join("models.rs"), generate_models(&configs.models))?;
    write_file!(
        out_dir.join("servers.rs"),
        generate_mcp_servers(&configs.servers)
    )?;
    create_agent_files(out_dir, &configs.agents, options)?;
    create_tool_files(out_dir, &configs.tools)?;

    Ok(())
}
