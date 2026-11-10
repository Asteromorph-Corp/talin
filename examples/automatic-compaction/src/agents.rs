use talin::{include_agents, include_models};

use crate::facts::RecallCtor;

pub type RecallAgentOutputCtor = RecallCtor;

include_models!();
include_agents!();
