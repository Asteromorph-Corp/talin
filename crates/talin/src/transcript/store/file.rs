use std::ffi::OsString;
use std::path::{Path, PathBuf};

use async_trait::async_trait;
use thiserror::Error;
use tokio::fs::{self, OpenOptions};
use tokio::io::AsyncWriteExt;

use crate::transcript::store::{StoreError, TranscriptStore};

/// A [`TranscriptStore`] that keeps a transcript as a JSON file.
#[derive(Debug)]
pub struct FileStore {
    path: PathBuf,
}

/// An error from [`FileStore`].
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum FileStoreError {
    /// A filesystem operation failed.
    #[error("filesystem error at `{}`: {source}", path.display())]
    Io {
        /// The path of the file.
        path: PathBuf,
        /// The I/O error.
        #[source]
        source: std::io::Error,
    },
}

impl FileStore {
    /// Creates a store that writes to `path`.
    ///
    /// Use [`restore`](AgentSession::restore) to continue an existing transcript.
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    /// The path of the file.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Writes the file by first writing to a temporary file and then renaming it into place.
    async fn write_with_tmp(&self, json: &str) -> Result<(), std::io::Error> {
        let mut temp: OsString = OsString::from(self.path.as_os_str());
        temp.push(".tmp");
        let temp = PathBuf::from(temp);

        let mut file: fs::File = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&temp)
            .await?;
        file.write_all(json.as_bytes()).await?;
        file.sync_data().await?;
        drop(file);
        fs::rename(&temp, &self.path).await
    }

    /// Replaces the file's contents with `json`.
    async fn write(&self, json: &str) -> Result<(), FileStoreError> {
        if let Err(source) = self.write_with_tmp(json).await {
            let path = self.path.clone();
            return Err(FileStoreError::Io { path, source });
        }
        Ok(())
    }

    /// Reads the file's contents, or `None` if the file does not exist.
    async fn read(&mut self) -> Result<Option<String>, FileStoreError> {
        let json = match fs::read_to_string(&self.path).await {
            Ok(json) => json,
            Err(source) if source.kind() == std::io::ErrorKind::NotFound => {
                return Ok(None);
            }
            Err(source) => {
                let path = self.path.clone();
                return Err(FileStoreError::Io { path, source });
            }
        };
        Ok(Some(json))
    }
}

#[async_trait]
impl TranscriptStore for FileStore {
    async fn checkpoint_raw(&mut self, json: &str) -> Result<(), StoreError> {
        self.write(json).await?;
        Ok(())
    }

    async fn restore_raw(&mut self) -> Result<Option<String>, StoreError> {
        Ok(self.read().await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schemas::{BodyMessage, UserMessage};
    use crate::transcript::AgentTranscript;
    use crate::transcript::store::TranscriptStoreExt;

    type Transcript = AgentTranscript<Vec<String>, BodyMessage>;

    fn temp_path(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("pe-store-{}-{name}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        dir.join("transcript.json")
    }

    fn user(text: &str) -> BodyMessage {
        BodyMessage::User(UserMessage::new(text))
    }

    fn json_of(transcript: &Transcript) -> String {
        serde_json::to_string(transcript).unwrap()
    }

    #[tokio::test]
    async fn keeps_the_file_in_step_with_the_transcript() {
        let path = temp_path("sync");
        let mut store = FileStore::new(&path);
        let mut transcript = Transcript::new(vec!["sys".into()], vec![user("one")]).unwrap();

        store.checkpoint(&transcript).await.unwrap();
        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            json_of(&transcript)
        );

        transcript.mark_turn_start();
        transcript.push_message(user("two")).unwrap();
        store.checkpoint(&transcript).await.unwrap();
        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            json_of(&transcript)
        );

        // A rollback shrinks the file.
        transcript.rollback(&crate::transcript::scope::Last::turn());
        store.checkpoint(&transcript).await.unwrap();
        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            json_of(&transcript)
        );

        // The temporary file does not linger.
        assert!(!path.with_extension("json.tmp").exists());
        std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[tokio::test]
    async fn restores_where_the_file_left_off() {
        let path = temp_path("resume");
        let mut store = FileStore::new(&path);
        let transcript = Transcript::new(vec!["sys".into()], vec![user("one")]).unwrap();
        store.checkpoint(&transcript).await.unwrap();
        drop(store);

        let mut store = FileStore::new(&path);
        let mut restored = store
            .restore::<Vec<String>, BodyMessage>()
            .await
            .unwrap()
            .unwrap();
        assert_eq!(json_of(&restored), json_of(&transcript));

        restored.push_message(user("two")).unwrap();
        store.checkpoint(&restored).await.unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), json_of(&restored));

        std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[tokio::test]
    async fn a_missing_file_restores_nothing() {
        let path = temp_path("missing");
        let mut store = FileStore::new(&path);
        assert!(
            store
                .restore::<Vec<String>, BodyMessage>()
                .await
                .unwrap()
                .is_none()
        );
        let transcript = Transcript::default();
        store.checkpoint(&transcript).await.unwrap();
        assert!(path.exists());
        std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[tokio::test]
    async fn overwrites_a_file_changed_behind_its_back() {
        let path = temp_path("overwrite");
        let mut store = FileStore::new(&path);
        let transcript = Transcript::new(vec!["sys".into()], vec![user("one")]).unwrap();
        store.checkpoint(&transcript).await.unwrap();

        std::fs::write(&path, "not a transcript").unwrap();
        store.checkpoint(&transcript).await.unwrap();
        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            json_of(&transcript)
        );

        std::fs::remove_file(&path).unwrap();
        store.checkpoint(&transcript).await.unwrap();
        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            json_of(&transcript)
        );
        std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }
}
