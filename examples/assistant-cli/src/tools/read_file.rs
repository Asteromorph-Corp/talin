use std::error::Error;
use std::path::PathBuf;

use talin::tool;

pub struct ReadFileTool {
    root: PathBuf,
}

#[tool]
impl ReadFileTool {
    async fn call(&self, input: Self::Input) -> Result<Self::Output, Box<dyn Error + Send + Sync>> {
        // Keep the model inside the project directory.
        let root = self.root.canonicalize()?;
        let path = root
            .join(&input.path)
            .canonicalize()
            .map_err(|e| format!("cannot read {}: {e}", input.path))?;
        if !path.starts_with(&root) {
            return Err(format!("{} is outside the project directory", input.path).into());
        }
        Ok(tokio::fs::read_to_string(path).await?)
    }

    fn summarize_input(&self, input: &Self::Input) -> Option<String> {
        Some(format!("Read {}", input.path))
    }

    fn from_config(config: _) -> Self {
        Self {
            root: config.root.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use talin::tools::{FromConfig, Function};

    use super::*;

    fn tool() -> ReadFileTool {
        ReadFileTool::from_config(ReadFileToolConfig {
            root: "configs".into(),
        })
    }

    #[tokio::test]
    async fn reads_a_file_inside_the_root() {
        let content = tool()
            .call(ReadFileToolInput {
                path: "servers.toml".into(),
            })
            .await
            .unwrap();
        assert!(content.contains("deepwiki"));
    }

    #[tokio::test]
    async fn refuses_a_path_that_escapes_the_root() {
        let error = tool()
            .call(ReadFileToolInput {
                path: "../Cargo.toml".into(),
            })
            .await
            .unwrap_err();
        assert!(error.to_string().contains("outside the project directory"));
    }

    #[tokio::test]
    async fn reports_a_missing_file_to_the_model() {
        let error = tool()
            .call(ReadFileToolInput {
                path: "missing.toml".into(),
            })
            .await
            .unwrap_err();
        assert!(error.to_string().contains("cannot read missing.toml"));
    }
}
