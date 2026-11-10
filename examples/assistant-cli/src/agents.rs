use talin::agent::{Agent, AgentSession};
use talin::chain::StringCtor;
use talin::llm::ClaudeMessages;

use crate::tools::*;

talin::include_models!();
talin::include_agents!();

/// The session type of the assistant.
pub type AssistantSession = AgentSession<ClaudeMessages, AssistantAgentInputCtor, StringCtor, ()>;

/// The assistant agent type.
pub type Assistant = Agent<ClaudeMessages, AssistantAgentInputCtor, StringCtor, ()>;
