use talin::strategy::{AutoCompactStrategy, ClaudeMessagesAutoCompactStrategy};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Workload<S> {
    #[serde(flatten)]
    pub compaction: S,
    pub archive_rows: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ArchiveText {
    pub header: String,
    pub entry: String,
    pub footer: String,
    pub noise_marker: String,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub openai: Workload<AutoCompactStrategy>,
    pub claude: Workload<ClaudeMessagesAutoCompactStrategy>,
    pub archive: ArchiveText,
}

impl Config {
    pub fn load() -> Result<Self, toml::de::Error> {
        toml::from_str(include_str!("../configs/workload.toml"))
    }
}
