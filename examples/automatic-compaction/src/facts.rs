use talin::chain::{ChainOutput, Ctor};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Project facts originate in the first user turn.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectFacts {
    pub codename: String,
    pub port: u16,
    pub recovery: String,
    pub region: String,
}

/// All facts the follow-up must recover. The revision originates only in the tool result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ChainOutput, Ctor)]
#[chain(derive_schema)]
#[serde(deny_unknown_fields)]
pub struct Recall {
    pub codename: String,
    pub port: u16,
    pub recovery: String,
    pub region: String,
    pub archive_revision: String,
}

impl Recall {
    pub fn fresh() -> Self {
        let bytes = *Uuid::new_v4().as_bytes();
        Self {
            codename: format!("project-{}", Uuid::new_v4()),
            port: 10_000 + u16::from_be_bytes([bytes[0], bytes[1]]) % 50_000,
            recovery: Uuid::new_v4().to_string(),
            region: format!("region-{}", Uuid::new_v4()),
            archive_revision: Uuid::new_v4().to_string(),
        }
    }

    pub fn project(&self) -> ProjectFacts {
        ProjectFacts {
            codename: self.codename.clone(),
            port: self.port,
            recovery: self.recovery.clone(),
            region: self.region.clone(),
        }
    }
}
