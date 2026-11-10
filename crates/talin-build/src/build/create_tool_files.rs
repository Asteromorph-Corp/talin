use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use heck::ToSnakeCase;

use crate::build::write_file::write_file;
use crate::codegen::{generate_tool, generate_tool_index};
use crate::config::Tool;
use crate::error::IoError;

pub(super) fn create_tool_files(
    out_dir: &Path,
    tools: &BTreeMap<String, Tool>,
) -> Result<(), IoError> {
    let tools_dir = out_dir.join("tools");
    fs::create_dir_all(&tools_dir).map_err(|e| IoError::create_dir_failure(&tools_dir, e))?;

    tools.iter().try_for_each(|(id, tool)| {
        write_file!(
            tools_dir.join(format!("{}.rs", id.to_snake_case())),
            generate_tool(id, tool),
        )
    })?;
    write_file!(
        out_dir.join("tool_index.rs"),
        generate_tool_index(&tools_dir, tools),
    )
}
