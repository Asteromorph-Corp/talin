//! The tools of the assistant. Each `#[tool]` impl lives in its own module.

use std::path::PathBuf;

mod read_file;
mod take_note;

pub use read_file::ReadFileTool;
pub use take_note::TakeNoteTool;

/// The tools that ask the user for approval before they run.
pub const NEEDS_APPROVAL: &[&str] = &["read_file"];

/// The directory that relative paths in `agents.toml` are relative to.
///
/// This is the package directory under `cargo run`, wherever `cargo` is run from, and the working
/// directory otherwise. Skills are found the same way.
pub fn project_dir() -> PathBuf {
    std::env::var_os("CARGO_MANIFEST_DIR").map_or_else(|| PathBuf::from("."), PathBuf::from)
}
