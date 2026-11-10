use std::error::Error;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use talin::tool;
use thiserror::Error;

use crate::config::ArchiveText;

#[derive(Debug, Error)]
#[error("the archive was already read; reuse the existing tool result")]
pub struct ArchiveAlreadyRead;

/// A local generated dataset whose size deliberately exceeds the compaction threshold.
pub struct LoadArchiveTool {
    pub rows: usize,
    pub revision: String,
    pub text: ArchiveText,
    pub calls: Arc<AtomicUsize>,
}

#[tool]
impl LoadArchiveTool {
    type Output = String;

    async fn call(&self, _: LoadArchiveToolInput) -> Result<String, Box<dyn Error + Send + Sync>> {
        if self.calls.fetch_add(1, Ordering::SeqCst) != 0 {
            return Err(ArchiveAlreadyRead.into());
        }
        let mut archive = self.text.header.replace("{revision}", &self.revision);
        for index in 0..self.rows {
            archive.push_str(&self.text.entry.replace("{index}", &index.to_string()));
        }
        archive.push_str(&self.text.footer);
        Ok(archive)
    }
}
