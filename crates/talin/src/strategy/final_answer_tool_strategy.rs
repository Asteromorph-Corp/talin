use std::collections::HashSet;
use std::convert::Infallible;

use async_trait::async_trait;
use talin_provider::openai_chat::Config;
use talin_provider::openai_chat::types::{
    ChatCompletionAllowedTools, ChatCompletionTool, ChatCompletionToolChoiceOption,
    ChatCompletionTools, FunctionObject, ResponseFormat, ToolChoiceOptions,
};

use crate::chain::OutputCtor;
use crate::llm::{OpenAIChat, RequestOf};
use crate::schemas::{AssistantMessage, ContentPart, LlmStep};
use crate::strategy::Strategy;

/// A [`Strategy`] that exposes a JSON-schema `response_format` as a `submit_final_answer` tool.
///
/// If the request carries a JSON-schema `response_format`, the strategy removes it and instead
/// injects a `submit_final_answer` tool whose input format is the same as that of the expected
/// final answer.
/// When the model calls the `submit_final_answer` tool, the strategy intercepts the call and
/// converts it back into a final answer output.
///
/// The strategy can be used to acquire structured output from models that supports tool calls but
/// not structured outputs.
/// Moreover, some models are forced to return a final answer if `response_format` is set, even when
/// more tool calls are necessary.
/// This strategy can work around this and allow models to choose between calling a tool or
/// returning a final answer.
///
/// Only compatible with [`OpenAIChat`].
///
/// No return value.
#[derive(Default, Clone)]
pub struct FinalAnswerToolStrategy;

const SUBMIT_FINAL_ANSWER: &str = "submit_final_answer";
const SUBMIT_FINAL_ANSWER_DESCRIPTION: &str = "Submit the final answer to the user. This tool should be used when you are confident that you have the correct answer and want to provide it to the user.";

#[async_trait]
impl<O: OutputCtor, C: Config + Clone + Send + Sync + 'static> Strategy<OpenAIChat<C>, O>
    for FinalAnswerToolStrategy
{
    type Output = ();
    type Error = Infallible;

    async fn process_request(
        &mut self,
        mut request: RequestOf<OpenAIChat<C>>,
    ) -> Result<RequestOf<OpenAIChat<C>>, Self::Error> {
        let Some(ResponseFormat::JsonSchema { json_schema }) = request.inner.response_format else {
            return Ok(request);
        };
        request.inner.response_format = None;

        let tools = request.inner.tools.get_or_insert_default();

        if let Some(original) = &request.inner.tool_choice {
            match original {
                // A tool call is required.
                ChatCompletionToolChoiceOption::Mode(ToolChoiceOptions::Required)
                | ChatCompletionToolChoiceOption::Function(_)
                | ChatCompletionToolChoiceOption::Custom(_) => return Ok(request),
                // Tool calls are prohibited.
                ChatCompletionToolChoiceOption::Mode(ToolChoiceOptions::None) => tools.clear(),
                ChatCompletionToolChoiceOption::AllowedTools(choice) => {
                    let allowed = allowed_tool_names(&choice.allowed_tools);
                    tools.retain(|tool| allowed.contains(tool_name(tool)))
                }
                ChatCompletionToolChoiceOption::Mode(ToolChoiceOptions::Auto) => {}
            }
        }

        tools.push(ChatCompletionTools::Function(ChatCompletionTool {
            function: FunctionObject {
                name: SUBMIT_FINAL_ANSWER.to_owned(),
                description: Some(SUBMIT_FINAL_ANSWER_DESCRIPTION.to_owned()),
                parameters: Some(json_schema.schema),
                strict: json_schema.strict,
            },
        }));
        request.inner.tool_choice = Some(ChatCompletionToolChoiceOption::Mode(
            ToolChoiceOptions::Required,
        ));

        Ok(request)
    }

    async fn process_llm_step(&mut self, output: LlmStep) -> Result<LlmStep, Self::Error> {
        if let LlmStep::ToolCall(msg) = &output
            && let Some(final_answer) = msg
                .tool_calls
                .iter()
                .find(|call| call.name == SUBMIT_FINAL_ANSWER)
        {
            let content = final_answer.arguments.to_string();
            return Ok(LlmStep::Finish(AssistantMessage {
                thought: msg.thought.clone(),
                content: vec![ContentPart::Text(content)],
            }));
        }

        Ok(output)
    }

    async fn finalize(self) -> Result<Self::Output, Self::Error> {
        Ok(())
    }
}

/// The name a tool is identified by in tool calls.
fn tool_name(tool: &ChatCompletionTools) -> &str {
    match tool {
        ChatCompletionTools::Function(tool) => &tool.function.name,
        ChatCompletionTools::Custom(tool) => &tool.custom.name,
    }
}

/// Collect the names of the tools an allow list permits.
///
/// Each allowed tool is an untyped [`serde_json::Value`]; entries that fail to deserialize into a
/// [`ChatCompletionTools`] are skipped rather than aborting the whole request.
fn allowed_tool_names(allowed_tools: &[ChatCompletionAllowedTools]) -> HashSet<String> {
    allowed_tools
        .iter()
        .flat_map(|entry| &entry.tools)
        .filter_map(|tool| serde_json::from_value::<ChatCompletionTools>(tool.clone()).ok())
        .map(|tool| tool_name(&tool).to_owned())
        .collect()
}
