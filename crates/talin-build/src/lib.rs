//! Build-time code generation for `talin` agents from TOML configuration files

use std::path::PathBuf;

use crate::build::create_files;
use crate::config::{load_configs, validate_configs};
use crate::env::{CARGO_MANIFEST_DIR, OUT_DIR, get_env_var};
use crate::error::TalinError;
pub use crate::options::{ArtifactOptions, InputFieldOrder};

#[cfg(test)]
mod test_utils;

mod build;
mod codegen;
mod config;
mod env;
mod error;
mod options;
mod resolution;
mod ty;

/// Builds the artifacts required for the `talin` application.
///
/// This generates necessary files based on configuration files located in the `configs/` directory,
/// using the default [`ArtifactOptions`].
pub fn build_artifacts() -> Result<(), TalinError> {
    build_artifacts_with(ArtifactOptions::default())
}

/// Builds the artifacts like [`build_artifacts`], with the given [`ArtifactOptions`].
pub fn build_artifacts_with(options: ArtifactOptions) -> Result<(), TalinError> {
    let cargo_manifest_dir = get_env_var(CARGO_MANIFEST_DIR)?;
    let config_dir = PathBuf::from(cargo_manifest_dir).join("configs/");
    let out_dir = PathBuf::from(get_env_var(OUT_DIR)?);

    let configs = load_configs(&config_dir)?;
    validate_configs(&configs)?;
    create_files(&out_dir, &configs, &options)?;
    println!("cargo::rerun-if-changed={}", config_dir.display());

    Ok(())
}
