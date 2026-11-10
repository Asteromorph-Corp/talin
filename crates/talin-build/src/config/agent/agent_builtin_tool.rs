use std::str::FromStr;

use crate::codegen::common::talin;
use crate::config::agent::agent_builtin_tool::claude_messages::ClaudeMessagesBuiltinTool;
use crate::config::agent::agent_builtin_tool::gemini_generate_content::GeminiGenerateContentBuiltinTool;
use crate::config::agent::agent_builtin_tool::gemini_interactions::GeminiInteractionsBuiltinTool;
use crate::config::agent::agent_builtin_tool::openai_responses::OpenAiResponsesBuiltinTool;
use crate::config::agent::agent_builtin_tool::xai_responses::XaiResponsesBuiltinTool;

mod agent_builtin_tools;
mod claude_messages;
mod gemini_generate_content;
mod gemini_interactions;
mod openai_responses;
mod xai_responses;

pub(super) use agent_builtin_tools::AgentBuiltinTools;
use syn::parse_quote;

/// A provider built-in tool, identified by `<provider>_<api>/<tool_name>`.
///
/// The `provider` tag (and the `<provider>_<api>` half of the string form) is the canonical
/// `provider_apiname` identifier; the pre-`provider_apiname` spellings are kept as aliases for
/// backwards compatibility and are intentionally undocumented.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "provider", rename_all = "snake_case")]
pub(crate) enum AgentBuiltinTool {
    #[serde(
        alias = "openai_responses",
        alias = "responses",
        alias = "Responses",
        alias = "OpenAiResponses"
    )]
    OpenAiResponses(OpenAiResponsesBuiltinTool),
    #[serde(
        alias = "claude",
        alias = "Claude",
        alias = "anthropic",
        alias = "Anthropic"
    )]
    ClaudeMessages(ClaudeMessagesBuiltinTool),
    #[serde(alias = "gemini", alias = "Gemini")]
    GeminiGenerateContent(GeminiGenerateContentBuiltinTool),
    GeminiInteractions(GeminiInteractionsBuiltinTool),
    #[serde(alias = "xai", alias = "Xai", alias = "XaiResponses")]
    XaiResponses(XaiResponsesBuiltinTool),
}

impl FromStr for AgentBuiltinTool {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Some((provider, tool)) = s.split_once("/") else {
            return Err(format!(
                "Invalid format for provider built-in tool: {s}, expected <provider>_<api>/<tool_name>"
            ));
        };
        match provider {
            // Legacy spellings kept for backwards compatibility.
            "openai_responses" | "responses" | "Responses" => Ok(
                AgentBuiltinTool::OpenAiResponses(OpenAiResponsesBuiltinTool::from_str(tool)?),
            ),
            "claude_messages" | "claude" | "Claude" | "anthropic" | "Anthropic" => Ok(
                AgentBuiltinTool::ClaudeMessages(ClaudeMessagesBuiltinTool::from_str(tool)?),
            ),
            "gemini_generate_content" => Ok(AgentBuiltinTool::GeminiGenerateContent(
                GeminiGenerateContentBuiltinTool::from_str(tool)?,
            )),
            "gemini_interactions" | "gemini" | "Gemini" => {
                Ok(AgentBuiltinTool::GeminiInteractions(
                    GeminiInteractionsBuiltinTool::from_str(tool)?,
                ))
            }
            "xai_responses" | "xai" | "Xai" => Ok(AgentBuiltinTool::XaiResponses(
                XaiResponsesBuiltinTool::from_str(tool)?,
            )),
            _ => Err(format!("Unknown provider for built-in tool: {provider}")),
        }
    }
}

impl AgentBuiltinTool {
    pub(crate) fn tool_ty(&self) -> syn::Type {
        let talin = talin();

        match self {
            AgentBuiltinTool::OpenAiResponses(_) => parse_quote! {
                #talin::llm::openai_responses::ResponsesBuiltinTool
            },
            AgentBuiltinTool::ClaudeMessages(_) => parse_quote! {
                #talin::llm::claude_messages::MessagesBuiltinTool
            },
            AgentBuiltinTool::GeminiGenerateContent(_) => parse_quote! {
                #talin::llm::gemini_generate_content::GenerateContentBuiltinTool
            },
            AgentBuiltinTool::GeminiInteractions(_) => parse_quote! {
                #talin::llm::gemini_interactions::InteractionsBuiltinTool
            },
            AgentBuiltinTool::XaiResponses(_) => parse_quote! {
                #talin::llm::xai_responses::ResponsesBuiltinTool
            },
        }
    }

    pub(crate) fn to_json(&self) -> String {
        match self {
            AgentBuiltinTool::OpenAiResponses(tool) => tool.to_json(),
            AgentBuiltinTool::ClaudeMessages(tool) => tool.to_json(),
            AgentBuiltinTool::GeminiGenerateContent(tool) => tool.to_json(),
            AgentBuiltinTool::GeminiInteractions(tool) => tool.to_json(),
            AgentBuiltinTool::XaiResponses(tool) => tool.to_json(),
        }
    }

    /// The model type whose `Llm::BuiltinTool` this tool is expressible as.
    ///
    /// Built-in tools are provider-concrete, so they may only be emitted in the
    /// per-provider `AgentProfile` impl that matches this model type.
    pub(crate) fn model_type(&self) -> crate::config::ModelType {
        use crate::config::ModelType;
        match self {
            AgentBuiltinTool::OpenAiResponses(_) => ModelType::OpenAiResponses,
            AgentBuiltinTool::ClaudeMessages(_) => ModelType::ClaudeMessages,
            AgentBuiltinTool::GeminiGenerateContent(_) => ModelType::GeminiGenerateContent,
            AgentBuiltinTool::GeminiInteractions(_) => ModelType::GeminiInteractions,
            AgentBuiltinTool::XaiResponses(_) => ModelType::XaiResponses,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ModelType;

    /// Built-in tools are addressed as `provider_apiname/tool`; the legacy provider spellings
    /// still resolve to the same variant.
    #[test]
    fn builtin_tool_canonical_and_legacy_providers() {
        let cases = [
            ("openai_responses/web_search", ModelType::OpenAiResponses),
            ("claude_messages/web_search", ModelType::ClaudeMessages),
            (
                "gemini_generate_content/google_search",
                ModelType::GeminiGenerateContent,
            ),
            (
                "gemini_interactions/google_search",
                ModelType::GeminiInteractions,
            ),
            ("xai_responses/web_search", ModelType::XaiResponses),
        ];
        for (spec, expected) in cases {
            let tool = AgentBuiltinTool::from_str(spec)
                .unwrap_or_else(|e| panic!("{spec} should parse: {e}"));
            assert_eq!(tool.model_type(), expected, "{spec}");
        }
    }
}
