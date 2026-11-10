use std::path::Path;

use talin::llm::{LlmRequestItem, LlmRequestSystem};
use talin::transcript::{AgentTranscript, EntryView};
use serde::Serialize;
use uuid::Uuid;

use crate::evidence::{Compaction, TokenUsage};
use crate::facts::Recall;

#[derive(Debug, Serialize)]
pub struct RequestEvidence {
    pub bytes: usize,
    pub contains_archive_noise: bool,
    pub compactions: Vec<Compaction>,
}

#[derive(Debug, Serialize)]
pub struct ResponseEvidence {
    pub compactions: Vec<Compaction>,
    pub usage: TokenUsage,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub enum StoredEntry {
    Message {
        id: Uuid,
        canonical: String,
    },
    ToolError {
        id: Uuid,
        call_id: String,
        name: String,
        feedback: String,
    },
    Native {
        id: Uuid,
    },
    TurnMarker,
    StepMarker,
    CompactionMarker,
}

/// An observation of the normal session, never used to construct or edit it.
#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct TranscriptState {
    pub entries: Vec<StoredEntry>,
    pub system: String,
    pub native_records: Vec<String>,
    /// Whether each native record holds a compaction, as decided by the LLM backend.
    pub compacting: Vec<bool>,
}

impl TranscriptState {
    pub fn observe<S, B>(transcript: &AgentTranscript<S, B>) -> Result<Self, serde_json::Error>
    where
        S: LlmRequestSystem + Serialize,
        B: LlmRequestItem + Serialize,
    {
        let entries = transcript
            .full_view()
            .into_iter()
            .map(|entry| {
                Ok(match entry {
                    EntryView::Message(id, message) => StoredEntry::Message {
                        id,
                        canonical: serde_json::to_string(message)?,
                    },
                    EntryView::ToolError(id, error) => StoredEntry::ToolError {
                        id,
                        call_id: error.call_id.clone(),
                        name: error.name.clone(),
                        feedback: format!("{:?}", error.feedback),
                    },
                    EntryView::Native(id) => StoredEntry::Native { id },
                    EntryView::TurnMarker => StoredEntry::TurnMarker,
                    EntryView::StepMarker => StoredEntry::StepMarker,
                    EntryView::CompactionMarker { .. } => StoredEntry::CompactionMarker,
                })
            })
            .collect::<Result<_, serde_json::Error>>()?;
        let (system, records) = transcript.clone().materialize();
        Ok(Self {
            entries,
            system: serde_json::to_string(&system)?,
            native_records: records
                .iter()
                .map(serde_json::to_string)
                .collect::<Result<_, _>>()?,
            compacting: records.iter().map(B::contains_native_compaction).collect(),
        })
    }
}

#[derive(Debug, Serialize)]
pub struct Report {
    pub model: String,
    pub threshold: u32,
    pub archive_rows: usize,
    pub archive_calls: usize,
    pub input_facts: Recall,
    pub requests: Vec<RequestEvidence>,
    pub responses: Vec<ResponseEvidence>,
    pub recalls: Vec<Recall>,
    pub turns: Vec<TranscriptState>,
    pub passed: bool,
    pub error: Option<String>,
}

impl Report {
    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, serde_json::to_vec_pretty(self)?)
    }
}
