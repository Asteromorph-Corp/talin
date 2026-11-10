//! Drive a two-turn agent loop that carries the model's reasoning for each provider:
//! - Turn 1: the model reasons, then calls the `read_passphrase` tool.
//! - Turn 2: the request must carry turn 1's reasoning back, alongside the tool call and result.
//!
//! Reasoning is signed by the provider, so dropping or rewriting it on the way back is rejected —
//! which is what the live module checks, and what the mock module pins on the wire.
#![allow(unreachable_pub)]

mod common;

/// The initial prompt passed to the model.
pub const PROMPT: &str = "Use the `read_passphrase` tool to read the passphrase from the \
    `main` vault, then report it verbatim. Think hard before using the tool.";

/// The tool that returns a passphrase.
mod tool {
    use std::error::Error;
    use std::sync::{Arc, Mutex};

    use async_trait::async_trait;
    use schemars::JsonSchema;
    use serde::Deserialize;
    use talin::tools::Function;

    /// The name of the tool.
    pub const TOOL_NAME: &str = "read_passphrase";
    /// The vault the model is told to read from.
    pub const VAULT: &str = "main";
    /// The passphrase not included in the initial prompt.
    pub const PASSPHRASE: &str = "correct-horse-battery-staple";

    #[derive(Debug, Deserialize, JsonSchema)]
    pub struct ReadPassphraseInput {
        /// The name of the vault to read the passphrase from.
        pub vault: String,
    }

    /// Returns [`PASSPHRASE`], recording the vault of every call.
    pub struct ReadPassphrase {
        pub calls: Arc<Mutex<Vec<String>>>,
    }

    impl ReadPassphrase {
        pub fn new() -> Self {
            Self {
                calls: Arc::new(Mutex::new(Vec::new())),
            }
        }
    }

    #[async_trait]
    impl Function for ReadPassphrase {
        type Input = ReadPassphraseInput;
        type Output = String;

        fn name(&self) -> String {
            TOOL_NAME.to_string()
        }

        fn description(&self) -> String {
            "Reads the passphrase stored in the named vault.".to_string()
        }

        async fn call(
            &self,
            input: Self::Input,
        ) -> Result<Self::Output, Box<dyn Error + Send + Sync>> {
            self.calls.lock().unwrap().push(input.vault);
            Ok(PASSPHRASE.to_string())
        }
    }
}

/// Runs the reasoning round trip against a mock server.
mod mock {
    use mockito::Matcher;
    use serde_json::{Value, json};
    use talin::agent::Agent;
    use talin::llm::Llm;
    use talin::tools::Tool;

    use crate::PROMPT;
    use crate::common::{self, MockServer, mock};
    use crate::tool::{PASSPHRASE, ReadPassphrase, TOOL_NAME, VAULT};

    /// Tool-call id, echoed back in turn 2's request.
    const TOOL_CALL_ID: &str = "call_test";
    /// The model's reasoning. Absent from the prompt and from the tool, so it can only reach turn
    /// 2's request by being carried over from turn 1.
    const THOUGHT: &str = "the passphrase must come from the main vault";

    /// Runs the two-turn loop with reasoning:
    /// - Turn 1: the model reasons and calls the tool.
    /// - Turn 2: the request body must contain every string in `evidence` — the reasoning, plus the
    ///   signature for providers that sign it, since a dropped signature is rejected.
    async fn assert_reasoning_round_trip<L: Llm>(
        path: &str,
        llm: fn(String, reqwest::Client) -> L,
        thought_tool_call_response: fn(&str, &str, &Value, &str) -> Value,
        final_response: fn(&str) -> Value,
        evidence: &[&str],
    ) {
        let mut mock = MockServer::start().await;
        let response1 = thought_tool_call_response(
            TOOL_CALL_ID,
            TOOL_NAME,
            &json!({ "vault": VAULT }),
            THOUGHT,
        );
        let t1 = mock.expect_post(path, Matcher::Any, response1).await;
        let request2 = Matcher::AllOf(
            evidence
                .iter()
                .map(|carried| Matcher::Regex(carried.to_string()))
                .collect(),
        );
        let response2 = final_response(PASSPHRASE);
        let t2 = mock.expect_post(path, request2, response2).await;

        let tool = ReadPassphrase::new();
        let calls = tool.calls.clone();

        let llm = llm(mock.base_url(), mock.http_client());
        let agent: Agent<_> = Agent::builder().tools([Tool::from(tool)]).build(llm);
        let mut session = agent.session();
        let result = session.run_turn(PROMPT).await;

        t1.assert_async().await;
        t2.assert_async().await;
        let answer = result.expect("agent turn failed");
        assert_eq!(
            calls.lock().unwrap().as_slice(),
            [VAULT],
            "the tool should have been executed once, with the vault from the prompt"
        );
        assert!(
            answer.contains(PASSPHRASE),
            "expected final answer to contain {PASSPHRASE:?}, got {answer:?}"
        );
    }

    #[tokio::test]
    async fn generic_chat_completes_reasoning_round_trip() {
        assert_reasoning_round_trip(
            "/chat/completions",
            mock::generic_chat,
            common::generic_chat_thought_tool_call_response,
            common::openai_chat_response,
            &[THOUGHT],
        )
        .await;
    }

    #[tokio::test]
    async fn openai_chat_completes_reasoning_round_trip() {
        assert_reasoning_round_trip(
            "/chat/completions",
            mock::openai_chat,
            common::openai_chat_thought_tool_call_response,
            common::openai_chat_response,
            &[THOUGHT],
        )
        .await;
    }

    #[tokio::test]
    async fn openai_responses_completes_reasoning_round_trip() {
        assert_reasoning_round_trip(
            "/responses",
            mock::openai_responses,
            common::openai_responses_thought_tool_call_response,
            common::openai_responses_response,
            &[THOUGHT, common::THOUGHT_SIGNATURE],
        )
        .await;
    }

    #[tokio::test]
    async fn xai_responses_completes_reasoning_round_trip() {
        assert_reasoning_round_trip(
            "/responses",
            mock::xai_responses,
            common::xai_responses_thought_tool_call_response,
            common::xai_responses_response,
            &[THOUGHT, common::THOUGHT_SIGNATURE],
        )
        .await;
    }

    #[tokio::test]
    async fn claude_messages_completes_reasoning_round_trip() {
        assert_reasoning_round_trip(
            "/messages",
            mock::claude_messages,
            common::claude_messages_thought_tool_call_response,
            common::claude_messages_response,
            &[THOUGHT, common::THOUGHT_SIGNATURE],
        )
        .await;
    }

    #[tokio::test]
    async fn gemini_generate_content_completes_reasoning_round_trip() {
        assert_reasoning_round_trip(
            &mock::gemini_generate_content_path(),
            mock::gemini_generate_content,
            common::gemini_generate_content_thought_tool_call_response,
            common::gemini_generate_content_response,
            &[THOUGHT, common::THOUGHT_SIGNATURE],
        )
        .await;
    }

    #[tokio::test]
    async fn gemini_interactions_completes_reasoning_round_trip() {
        assert_reasoning_round_trip(
            "/v1/interactions",
            mock::gemini_interactions,
            common::gemini_interactions_thought_tool_call_response,
            common::gemini_interactions_response,
            &[THOUGHT, common::THOUGHT_SIGNATURE],
        )
        .await;
    }

    #[tokio::test]
    async fn sglang_completes_reasoning_round_trip() {
        assert_reasoning_round_trip(
            "/generate",
            mock::sglang_generate,
            common::sglang_generate_thought_tool_call_response,
            common::sglang_generate_response,
            &[THOUGHT],
        )
        .await;
    }
}

/// Runs the reasoning round trip against a live LLM provider.
/// The tests hit real providers, cost money, and are non-deterministic, so every test is
/// `#[ignore]`d. Run the tests by:
///
/// ```sh
/// cargo test --test reasoning_round_trip live::claude -- --ignored --nocapture
/// ```
mod live {
    use talin::agent::Agent;
    use talin::llm::{Llm, LlmOptions};
    use talin::schemas::{BodyMessage, ReasoningEffort};
    use talin::tools::Tool;
    use talin::transcript::EntryView;

    use crate::PROMPT;
    use crate::common::live;
    use crate::tool::{PASSPHRASE, ReadPassphrase, VAULT};

    /// Drives one provider through the two-turn loop with reasoning on: the provider accepted the
    /// reasoning it was handed back, the tool ran once, and the passphrase came back.
    async fn assert_reasoning_round_trip<L>(llm: L, thought_required: bool)
    where
        L: Llm,
        L::Options: From<LlmOptions>,
    {
        let tool = ReadPassphrase::new();
        let calls = tool.calls.clone();

        let llm = live::with_reasoning(llm, ReasoningEffort::Low);
        let agent: Agent<_> = Agent::builder().tools([Tool::from(tool)]).build(llm);
        let mut session = agent.session();
        let answer = session
            .run_turn(PROMPT)
            .await
            .expect("agent turn failed to reach a final answer");

        let thoughts = session
            .transcript
            .full_view()
            .iter()
            .filter_map(|entry| match entry {
                EntryView::Message(_, BodyMessage::Assistant(message)) => message.thought.clone(),
                EntryView::Message(_, BodyMessage::ToolCall(message)) => message.thought.clone(),
                _ => None,
            })
            .collect::<Vec<_>>();
        let calls = calls.lock().unwrap().clone();
        eprintln!("thoughts: {thoughts:?}");
        eprintln!("tool calls: {calls:?}");
        eprintln!("final answer: {answer}");

        assert_eq!(calls.len(), 1, "tool should be executed exactly once");
        assert_eq!(calls[0], VAULT, "tool called with the wrong vault");
        assert!(
            answer.contains(PASSPHRASE),
            "expected final answer to contain {PASSPHRASE:?}, got {answer:?}"
        );
        if thought_required {
            assert!(
                !thoughts.is_empty(),
                "the model never reasoned, so nothing was carried back"
            );
        }
    }

    #[tokio::test]
    #[ignore = "live LLM: requires OPENAI_API_KEY"]
    async fn openai_responses_completes_reasoning_round_trip() {
        dotenvy::dotenv().ok();
        env_logger::init();
        assert_reasoning_round_trip(live::openai_responses_reasoning(), false).await;
    }

    #[tokio::test]
    #[ignore = "live LLM: requires XAI_API_KEY"]
    async fn xai_responses_completes_reasoning_round_trip() {
        dotenvy::dotenv().ok();
        env_logger::init();
        assert_reasoning_round_trip(live::xai_responses(), true).await;
    }

    #[tokio::test]
    #[ignore = "live LLM: requires CLAUDE_API_KEY"]
    async fn claude_messages_completes_reasoning_round_trip() {
        dotenvy::dotenv().ok();
        env_logger::init();
        assert_reasoning_round_trip(live::claude_messages_reasoning(), true).await;
    }

    #[tokio::test]
    #[ignore = "live LLM: requires GEMINI_API_KEY"]
    async fn gemini_generate_content_completes_reasoning_round_trip() {
        dotenvy::dotenv().ok();
        env_logger::init();
        assert_reasoning_round_trip(live::gemini_generate_content(), true).await;
    }

    #[tokio::test]
    #[ignore = "live LLM: requires GEMINI_API_KEY"]
    async fn gemini_interactions_completes_reasoning_round_trip() {
        dotenvy::dotenv().ok();
        env_logger::init();
        assert_reasoning_round_trip(live::gemini_interactions(), true).await;
    }
}
