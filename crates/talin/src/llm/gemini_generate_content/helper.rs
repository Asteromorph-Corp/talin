//! A module for Gemini LLM wrappers.

use talin_provider::gemini_generate_content::types::{
    Candidate, FinishReason, FunctionDeclaration, Tool, ToolConfig,
};

use crate::llm::LlmError;
use crate::llm::gemini_generate_content::GenerateContentBuiltinTool;
use crate::schemas::ToolChoice;
use crate::tools::ToolSpec;
use crate::utils::NoneIfEmpty;

pub(super) fn map_tools(
    tools: Vec<ToolSpec>,
    builtin_tools: Vec<GenerateContentBuiltinTool>,
    tool_choice: Option<ToolChoice>,
) -> (Option<Vec<Tool>>, Option<ToolConfig>) {
    let functions = ToolSpec::into_functions(tools);

    let function_tool = (!functions.is_empty()).then(|| {
        let decls = functions
            .into_iter()
            .map(FunctionDeclaration::from)
            .collect();
        Tool::new().with_functions(decls)
    });

    let tool_config = tool_choice.map(ToolConfig::from);
    // set `include_server_side_tool_invocations` if both local and server tools are present
    let tool_config = if function_tool.is_some() && !builtin_tools.is_empty() {
        Some(
            tool_config
                .unwrap_or_default()
                .with_include_server_side_tool_invocations(true),
        )
    } else {
        tool_config
    };

    let tools = function_tool
        .into_iter()
        .chain(builtin_tools.into_iter().map(Tool::from))
        .collect::<Vec<_>>();
    (tools.none_if_empty(), tool_config)
}

pub(super) fn select_candidate(candidates: Vec<Candidate>) -> Option<Candidate> {
    candidates
        .into_iter()
        .min_by_key(|c| match c.finish_reason {
            // Stop is the most preferred finish reason.
            Some(FinishReason::Stop) => 0,
            Some(FinishReason::FinishReasonUnspecified) | None => 1,
            _ => 3,
        })
}

pub(super) fn check_finish_reason(reason: FinishReason) -> Result<(), LlmError> {
    match reason {
        FinishReason::FinishReasonUnspecified | FinishReason::Stop => Ok(()),
        FinishReason::MaxTokens => Ok(()),
        FinishReason::Safety => Err(LlmError::refused("response flagged for safety reasons")),
        FinishReason::Recitation => Err(LlmError::refused("response flagged for recitation")),
        FinishReason::Language => Err(LlmError::refused(
            "response flagged for using unsupported language",
        )),
        FinishReason::Other => Err(LlmError::other("generation stopped for an unknown reason")),
        FinishReason::Blocklist => {
            Err(LlmError::refused("generated text contains forbidden terms"))
        }
        FinishReason::ProhibitedContent => Err(LlmError::refused(
            "generated text contains prohibited content",
        )),
        FinishReason::Spii => Err(LlmError::refused(
            "generated text contains sensitive personally identifiable information",
        )),
        FinishReason::MalformedFunctionCall => {
            Err(LlmError::other("model produced a malformed function call"))
        }
        FinishReason::ImageSafety => Err(LlmError::refused(
            "generated image contains safety violations",
        )),
        FinishReason::ImageProhibitedContent => Err(LlmError::refused(
            "generated image contains prohibited content",
        )),
        FinishReason::ImageOther => Err(LlmError::other(
            "image generation stopped for an unknown reason",
        )),
        FinishReason::NoImage => Err(LlmError::other("expected an image, but none was generated")),
        FinishReason::ImageRecitation => {
            Err(LlmError::refused("generated image flagged for recitation"))
        }
        FinishReason::UnexpectedToolCall => Err(LlmError::other(
            "model attempted a tool call, but no tools were enabled",
        )),
        FinishReason::TooManyToolCalls => Err(LlmError::other(
            "model attempted too many tool calls consecutively",
        )),
        FinishReason::MissingThoughtSignature => Err(LlmError::other(
            "request is missing a required thought signature",
        )),
        // Handle future variants as success, since we don't know what they mean yet.
        _ => Ok(()),
    }
}
