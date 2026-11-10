use crate::config::AgentBuiltinTool;
use crate::config::agent::agent_builtin_tool::claude_messages::ClaudeMessagesBuiltinTool;
use crate::config::agent::agent_builtin_tool::gemini_generate_content::GeminiGenerateContentBuiltinTool;
use crate::config::agent::agent_builtin_tool::gemini_interactions::GeminiInteractionsBuiltinTool;
use crate::config::agent::agent_builtin_tool::openai_responses::OpenAiResponsesBuiltinTool;
use crate::config::agent::agent_builtin_tool::xai_responses::XaiResponsesBuiltinTool;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct BuiltinTools<T> {
    pub tools: Vec<T>,
}

/// Legacy struct for old `builtin_tools` field.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "provider", rename_all = "snake_case")]
pub(crate) enum AgentBuiltinTools {
    #[serde(alias = "responses", alias = "Responses", alias = "OpenAiResponses")]
    OpenAiResponses(BuiltinTools<OpenAiResponsesBuiltinTool>),
    #[serde(
        alias = "claude",
        alias = "Claude",
        alias = "anthropic",
        alias = "Anthropic"
    )]
    ClaudeMessages(BuiltinTools<ClaudeMessagesBuiltinTool>),
    GeminiGenerateContent(BuiltinTools<GeminiGenerateContentBuiltinTool>),
    #[serde(alias = "gemini", alias = "Gemini")]
    GeminiInteractions(BuiltinTools<GeminiInteractionsBuiltinTool>),
    #[serde(alias = "xai", alias = "Xai", alias = "XaiResponses")]
    XaiResponses(BuiltinTools<XaiResponsesBuiltinTool>),
}

impl IntoIterator for AgentBuiltinTools {
    type Item = AgentBuiltinTool;
    type IntoIter = std::vec::IntoIter<AgentBuiltinTool>;

    fn into_iter(self) -> Self::IntoIter {
        let tools = match self {
            AgentBuiltinTools::OpenAiResponses(responses_tools) => responses_tools
                .tools
                .into_iter()
                .map(AgentBuiltinTool::OpenAiResponses)
                .collect::<Vec<_>>(),
            AgentBuiltinTools::ClaudeMessages(claude_tools) => claude_tools
                .tools
                .into_iter()
                .map(AgentBuiltinTool::ClaudeMessages)
                .collect::<Vec<_>>(),
            AgentBuiltinTools::GeminiGenerateContent(gemini_tools) => gemini_tools
                .tools
                .into_iter()
                .map(AgentBuiltinTool::GeminiGenerateContent)
                .collect::<Vec<_>>(),
            AgentBuiltinTools::GeminiInteractions(gemini_tools) => gemini_tools
                .tools
                .into_iter()
                .map(AgentBuiltinTool::GeminiInteractions)
                .collect::<Vec<_>>(),
            AgentBuiltinTools::XaiResponses(xai_tools) => xai_tools
                .tools
                .into_iter()
                .map(AgentBuiltinTool::XaiResponses)
                .collect::<Vec<_>>(),
        };
        tools.into_iter()
    }
}
