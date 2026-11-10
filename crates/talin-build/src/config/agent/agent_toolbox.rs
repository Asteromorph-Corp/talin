use serde::{Deserialize, Serialize};

use crate::config::AgentLocalTool;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AgentToolbox {
    pub name: String,
    pub tools: Vec<AgentLocalTool>,
}
