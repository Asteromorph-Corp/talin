use std::error::Error;
use std::path::PathBuf;

use talin::tool;
use tokio::io::AsyncWriteExt;

use super::project_dir;

pub struct TakeNoteTool {
    file: PathBuf,
}

#[tool]
impl TakeNoteTool {
    async fn call(&self, input: Self::Input) -> Result<Self::Output, Box<dyn Error + Send + Sync>> {
        if let Some(dir) = self.file.parent() {
            tokio::fs::create_dir_all(dir).await?;
        }
        let mut file = tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.file)
            .await?;
        file.write_all(format!("- {}\n", input.note).as_bytes())
            .await?;
        file.flush().await?;
        Ok("Noted.".into())
    }

    fn from_config(config: _) -> Self {
        Self {
            file: project_dir().join(config.file),
        }
    }
}

#[cfg(test)]
mod tests {
    use talin::tools::{FromConfig, Function};

    use super::*;

    #[tokio::test]
    async fn appends_notes_to_the_file() {
        // Removes the directory on drop.
        struct Cleanup(PathBuf);
        impl Drop for Cleanup {
            fn drop(&mut self) {
                let _ = std::fs::remove_dir_all(&self.0);
            }
        }

        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir =
            std::env::temp_dir().join(format!("assistant-cli-{}-{nanos}", std::process::id()));
        let file = dir.join("notes.md");

        let _ = std::fs::remove_dir_all(&dir);
        let _cleanup = Cleanup(dir.clone());
        let tool = TakeNoteTool::from_config(TakeNoteToolConfig {
            file: file.to_string_lossy().into_owned(),
        });

        for note in ["First.", "Second."] {
            tool.call(TakeNoteToolInput { note: note.into() })
                .await
                .unwrap();
        }

        assert_eq!(
            std::fs::read_to_string(&file).unwrap(),
            "- First.\n- Second.\n"
        );
    }

    #[test]
    fn relative_paths_are_relative_to_the_package() {
        let tool = TakeNoteTool::from_config(TakeNoteToolConfig {
            file: "notes/notes.md".into(),
        });
        // Not to the working directory, which is wherever `cargo run` was started.
        assert_eq!(
            tool.file,
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("notes/notes.md")
        );
    }
}
