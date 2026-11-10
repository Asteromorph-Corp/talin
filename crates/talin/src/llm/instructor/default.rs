use regex::Regex;
use serde::ser::Error;

use crate::llm::instructor::{CompletionInstructor, Instructor};
use crate::schemas::{LlmStep, ToolCall};
use crate::text::parse::{
    ParseError, RawToolCall, extract_jsons, extract_preamble, has_keys, parse_final_answer,
    parse_json, parse_tool_call, remove_thought, unescape_text,
};
use crate::tools::{FunctionSpec, normalize_tool_name};
use crate::utils::NoneIfEmpty;

const DEFAULT_TOOL_PROMPT: &str = r#"<tool_integration>
- You have access to a set of tools to help you answer questions.
- You can either use a tool or provide your final answer.
- You may use the tools iteratively as many times as needed.
</tool_integration>

<tool_instructions>
To use a tool, return a JSON object with the following structure:
  - "name": The name of the tool to use (must be one of: [{{?tool_names}}])
  - "arguments": The arguments to pass to the tool (must conform to the tool's schema)

You must strictly adhere to this format without any additional text, and ensure the JSON is well-formed.

Available tools are described below:
{{?tools}}
</tool_instructions>"#;

const ACTION_KEY: &str = "name";
const ACTION_INPUT_KEY: &str = "arguments";
const VALID_KEYS: &[&[&str]] = &[&[ACTION_KEY, ACTION_INPUT_KEY]];

/// The default instructor implementation.
#[derive(Default, Clone)]
pub struct DefaultInstructor;

impl DefaultInstructor {
    /// Parses the LLM output using regex as a fallback.
    fn parse_with_regex(&self, text: &str) -> Option<LlmStep> {
        let final_answer_re = Regex::new(r#"(?m)"final_answer"\s*:\s*"(.*)"\s*\n"#).unwrap();
        let action_regex = Regex::new(r#"(?m)"action"\s*:\s*"(.*)"\s*\n"#).unwrap();
        let action_input_regex = Regex::new(r#"(?m)"action_input"\s*:\s*"(.*)"\s*\n"#).unwrap();

        if let Some(final_answer) = final_answer_re.captures(text) {
            let final_answer = final_answer.get(1)?.as_str();
            return Some(LlmStep::new_text(None, unescape_text(final_answer)));
        }

        if let (Some(action), Some(action_input)) = (
            action_regex.captures(text),
            action_input_regex.captures(text),
        ) {
            let action = action.get(1)?.as_str();
            let action_input = action_input.get(1)?.as_str();
            let name = unescape_text(action);
            let arguments = Some(serde_json::from_str(action_input).ok()?);
            let tool_call = ToolCall::new(None, name, arguments);
            return Some(LlmStep::new_tool_calls(None, vec![tool_call]));
        }

        None
    }
}

/// Implementation of the [`Instructor`] trait for [`DefaultInstructor`].
///
/// Provides a custom tool use instruction and parses the LLM outputs accordingly.
impl Instructor for DefaultInstructor {
    fn tool_use_instruction(&self, tools: &[FunctionSpec]) -> String {
        let tool_names = tools
            .iter()
            .map(|t| normalize_tool_name(t.name.as_str()))
            .collect::<Vec<_>>()
            .join(", ");
        let tool_descriptions = tools
            .iter()
            .map(FunctionSpec::describe)
            .collect::<Vec<_>>()
            .join("\n");
        DEFAULT_TOOL_PROMPT
            .replace("{{?tool_names}}", &tool_names)
            .replace("{{?tools}}", &tool_descriptions)
    }

    fn parse_tool_use(&self, text: String) -> Result<LlmStep, ParseError> {
        // Remove any </think> tags and all contents prior.
        let thought_removed = remove_thought(&text);

        // Extract all JSON texts from the LLM output.
        let json_texts = extract_jsons(thought_removed);
        let jsons = json_texts
            .iter()
            .cloned()
            .filter_map(|json_text| {
                let json = parse_json(json_text).ok()?;
                Some((json_text, json))
            })
            .collect::<Vec<_>>();

        // Check if the extracted JSONs are events with malformed structure.
        // This means that the output is NOT a final answer.
        let is_malformed_event = jsons.iter().any(|(_, json)| has_keys(json, VALID_KEYS));

        // If there is only one JSON and it has a key "final_answer", return it directly.
        if jsons.len() == 1
            && let Some((json_text, json)) = jsons.first()
            && let Some(final_answer) = json.get("final_answer")
        {
            let final_answer = parse_final_answer(final_answer.clone());
            let thought = extract_preamble(&text, json_text)
                .none_if_empty()
                .map(String::from);
            return Ok(LlmStep::new_text(thought, final_answer));
        }

        // Deserialize all extracted JSONs into `ToolCall`.
        let calls = jsons
            .into_iter()
            .filter_map(|(json_text, json)| {
                let call = parse_tool_call::<RawToolCall>(json).ok()?;
                Some((json_text, call))
            })
            .collect::<Vec<_>>();

        // Construct the final output content based on the parsed results.
        //
        // If there is at least one valid tool call, return them.
        // Else try to parse the output using regex.
        // Else if the output is not a malformed event, return it as plain text.
        let step = if !calls.is_empty()
            && let Some((first_json_text, _)) = calls.first()
        {
            let first_json_text = *first_json_text;
            let calls = calls
                .into_iter()
                .flat_map(|(_, call)| call)
                .collect::<Vec<_>>();
            let thought = extract_preamble(&text, first_json_text)
                .none_if_empty()
                .map(String::from);
            LlmStep::new_tool_calls(thought, calls)
        } else if let Some(step) = self.parse_with_regex(thought_removed) {
            let body_text = match json_texts.first() {
                Some(text) => *text,
                None => thought_removed,
            };
            match extract_preamble(&text, body_text).none_if_empty() {
                Some(thought) => step.with_thought(thought),
                None => step,
            }
        } else if !is_malformed_event {
            LlmStep::new_text(None, text)
        } else {
            return Err(ParseError {
                original: text,
                source: serde_json::Error::custom("failed to parse tool use"),
            });
        };

        Ok(step)
    }
}

impl CompletionInstructor for DefaultInstructor {}

#[cfg(test)]
mod tests {
    use indoc::indoc;
    use rstest::rstest;
    use serde_json::json;

    use super::*;
    use crate::schemas::ContentExt;

    #[rstest]
    #[case::simple(
        indoc! {r#"
            ```json
            {
                "name": "generate",
                "arguments": "Hello, world!"
            }
            ```"#
        },
        None,
        "generate",
        json!("Hello, world!"),
    )]
    #[case::complex_arguments(
        indoc! {r#"
            {
                "name": "get_weather",
                "arguments": {
                    "location": "New York",
                    "date": "2023-10-01"
                }
            }
            ```"#
        },
        None,
        "get_weather",
        json!({
            "location": "New York",
            "date": "2023-10-01"
        }),
    )]
    #[case::with_thought(
        indoc! {r#"
            First, I should gather basic information about the rust programming language.
            ```json
            {
                "action": "search",
                "action_input": {
                    "query": "Rust programming language",
                }
            }"#
        },
        Some("First, I should gather basic information about the rust programming language."),
        "search",
        json!({
            "query": "Rust programming language"
        }),
    )]
    #[case::multiple(
        indoc!{r#"
            <paper_search>
            {
                "name": "paper_search",
                "arguments": {
                    "query": "some search \" \" a keyword"
                }
            }
            </paper_search>
            <paper_search>
            {
                "name": "paper_search",
                "arguments": {
                    "query": "some extra name"
                }</p>
            </paper_search>
            <paper_search>
            {
                "name": "paper_search",
                "arguments": {
                    "query": "some other dna name"
                }
            }
            </paper_search>
            <paper_search>
            {
                "name": "paper_search",
                "arguments": {
                    "query": "some dna name"
                }
            }
            </paper_search>
            <paper_search>
            {
                "name": "paper_search",
                "arguments": {
                    "query": "search {] keyword other"
                }
            }
            </paper_search>"#,
        },
        None,
        "paper_search",
        json!({"query": "some search \" \" a keyword"})
    )]
    fn test_parse_tool_call(
        #[case] output: &str,
        #[case] thought: Option<&str>,
        #[case] name: &str,
        #[case] argument: serde_json::Value,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let step = DefaultInstructor.parse_tool_use(output.into())?;

        let LlmStep::ToolCall(m) = step else {
            panic!("Expected `LlmEvent::ToolCall` got {step:#?}");
        };
        assert_eq!(m.thought.as_deref(), thought);
        let tool_call = &m.tool_calls[0];
        assert_eq!(tool_call.name, name);
        assert_eq!(tool_call.arguments, argument);

        Ok(())
    }

    #[rstest]
    #[case::simple(r#"Goodbye, world!"#, None, "Goodbye, world!")]
    #[case::simple_with_json(
        "The possible options were: [A, B, C, D], and I think A is the best.",
        None,
        "The possible options were: [A, B, C, D], and I think A is the best."
    )]
    #[case::json(
        indoc! {r#"
            ```json
            [
                {
                    "recipe": "Pancakes",
                    "ingredients": ["egg", "milk", "flour"]
                },
                {
                    "recipe": "Omelette",
                    "ingredients": ["egg", "cheese", "ham"]
                }
            ]
            ```"#
        },
        None,
        indoc! {r#"
            ```json
            [
                {
                    "recipe": "Pancakes",
                    "ingredients": ["egg", "milk", "flour"]
                },
                {
                    "recipe": "Omelette",
                    "ingredients": ["egg", "cheese", "ham"]
                }
            ]
            ```"#
        }
    )]
    #[case::final_answer(
        indoc! {r#"
            After considering all options, I conclude that the capital of France is Paris.
            ```json
            {
                "final_answer": "The capital of France is Paris."
            }
            ```
        "#},
        Some("After considering all options, I conclude that the capital of France is Paris."),
        "The capital of France is Paris."
    )]
    #[case::final_answer_messy(
        indoc! {r#"
            Here are some suggestions for the dinner menu :)
            ```json
            {
            "final_answer": "\## Thinking process\n\n### Step 1:\n- Candidate 1: Mac and Cheese\n- Candidate 2: Dumplings\n- Candidate 3: Pancakes"
            }
            ```
        "#},
        Some("Here are some suggestions for the dinner menu :)"),
        indoc! {
            r#"## Thinking process

            ### Step 1:
            - Candidate 1: Mac and Cheese
            - Candidate 2: Dumplings
            - Candidate 3: Pancakes"#
        }
    )]
    #[case::final_answer_multilne(
        indoc! {r#"
            {
                "final_answer": "I don't think pancakes are good for dinner. Here is a detailed explanation:

            - Pancakes are typically considered a breakfast food and may not provide the necessary nutrients for a balanced dinner.
            - They are often high in carbohydrates and sugars, which can lead to energy crashes later in the evening.
            - For dinner, it's generally better to have a meal that includes a good balance of protein, vegetables, and healthy fats."
            }"#},
        None,
        indoc! {r#"
            I don't think pancakes are good for dinner. Here is a detailed explanation:

            - Pancakes are typically considered a breakfast food and may not provide the necessary nutrients for a balanced dinner.
            - They are often high in carbohydrates and sugars, which can lead to energy crashes later in the evening.
            - For dinner, it's generally better to have a meal that includes a good balance of protein, vegetables, and healthy fats."#
        },
    )]
    #[case::final_answer_nested(
        indoc! {r#"
            ```json
            {
                "final_answer": {
                    "final_answer": "The capital of France is Paris.",
                }
            }
            ```
        "#},
        None,
        "The capital of France is Paris.",
    )]
    fn test_parse_final_answer(
        #[case] output: &str,
        #[case] thought: Option<&str>,
        #[case] text: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let step = DefaultInstructor.parse_tool_use(output.into())?;
        let LlmStep::Finish(m) = step else {
            panic!("Expected `LlmEvent::Finish` got {step:#?}");
        };
        assert_eq!(m.thought.as_deref(), thought);
        assert_eq!(m.content.to_text().unwrap_or_default(), text);
        Ok(())
    }
}
