use crate::llm::instructor::Instructor;
use crate::schemas::LlmStep;
use crate::text::parse::{
    ParseError, RawToolCall, extract_from_codeblock, extract_from_tag, extract_preamble, has_keys,
    parse_final_answer, parse_json, parse_tool_call, remove_thought,
};
use crate::tools::FunctionSpec;
use crate::utils::NoneIfEmpty;

const QWEN3_TOOL_PROMPT: &str = r#"

# Tools

You may call one or more functions to assist with the user query.

You are provided with function signatures within <tools></tools> XML tags:
<tools>
{{?tools}}
</tools>

For each function call, return a json object with function name and arguments within <tool_call></tool-call> XML tags:
<tool_call>
{"name": <function-name>, "arguments": <args-json-object>}
</tool_call>"#;

const NAME_KEY: &str = "name";
const ARGUMENTS_KEY: &str = "arguments";
const ALTERNATIVE_NAME_KEY: &str = "action";
const ALTERNATIVE_ARGUMENTS_KEY: &str = "action_input";
const VALID_KEYS: &[&[&str]] = &[&[NAME_KEY, ARGUMENTS_KEY]];
const ALTERNATIVE_KEYS: &[&[&str]] = &[&[ALTERNATIVE_NAME_KEY, ALTERNATIVE_ARGUMENTS_KEY]];

/// The instructor implementation for Qwen3 models.
///
/// Uses the [Qwen3 chat template](https://huggingface.co/Qwen/Qwen3-235B-A22B-Thinking-2507?chat_template=default) for tool use instructions.
#[derive(Default, Clone)]
pub struct Qwen3Instructor;

/// Implementation of [`Instructor`] trait for [`Qwen3Instructor`].
///
/// Provides tool use instruction based on the qwen3 prompt format and parses the LLM output
/// accordingly.
impl Instructor for Qwen3Instructor {
    fn tool_use_instruction(&self, tools: &[FunctionSpec]) -> String {
        let tools_str = tools
            .iter()
            .map(FunctionSpec::as_json)
            .collect::<Vec<_>>()
            .join("\n");
        QWEN3_TOOL_PROMPT.replace("{{?tools}}", &tools_str)
    }

    fn parse_tool_use(&self, text: String) -> Result<LlmStep, ParseError> {
        let thought_removed = remove_thought(&text);
        let extracted = extract_from_tag(thought_removed, "tool_call");
        let json_text = extract_from_codeblock(extracted);

        let json = parse_json(json_text);

        let is_malformed_event = json
            .as_ref()
            .is_ok_and(|json| has_keys(json, VALID_KEYS) || has_keys(json, ALTERNATIVE_KEYS));

        // If there is only one JSON and it has a key "final_answer", return it directly.
        if let Ok(json) = &json
            && let Some(final_answer) = json.get("final_answer")
        {
            let final_answer = parse_final_answer(final_answer.clone());
            let thought = extract_preamble(&text, json_text)
                .none_if_empty()
                .map(String::from);
            return Ok(LlmStep::new_text(thought, final_answer));
        }

        match json.and_then(parse_tool_call::<RawToolCall>) {
            Ok(calls) => {
                let thought = extract_preamble(&text, json_text)
                    .none_if_empty()
                    .map(String::from);
                Ok(LlmStep::new_tool_calls(thought, calls))
            }
            Err(_) if !is_malformed_event => Ok(LlmStep::new_text(None, text)),
            Err(e) => Err(ParseError::new(json_text, e)),
        }
    }
}
