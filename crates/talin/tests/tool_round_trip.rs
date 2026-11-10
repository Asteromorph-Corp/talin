//! Drive a two-turn agent loop for each provider:
//! - Turn 1: the model calls the `read_passphrase` tool, the agent runner executes it.
//! - Turn 2: the model returns the final answer. The request for turn 2 must carry turn 1's tool
//!   call and the tool result.
#![allow(unreachable_pub)]

mod common;

/// The initial prompt passed to the model.
pub const PROMPT: &str = "Use the `read_passphrase` tool to read the passphrase from the \
    `main` vault, then report it verbatim.";

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

/// Runs the two-turn tool loop against a mock server.
mod mock {
    use mockito::Matcher;
    use serde_json::{Value, json};
    use talin::agent::Agent;
    use talin::llm::Llm;
    use talin::tools::Tool;

    use crate::PROMPT;
    use crate::common::{self, MockServer, mock};
    use crate::tool::{PASSPHRASE, ReadPassphrase, TOOL_NAME, VAULT};

    macro_rules! partial_json {
        ($request:path) => {
            |id: &str, name: &str| mockito::Matcher::PartialJson($request(id, name))
        };
    }

    /// Tool-call id, echoed back in turn 2's request.
    const TOOL_CALL_ID: &str = "call_test";

    /// Runs the two-turn tool loop:
    /// - Turn 1: the model calls the tool.
    /// - Turn 2: the model returns the final answer. The request body is matched to confirm it
    ///   contains the tool call + result evidence, via `tool_result_request`.
    async fn assert_tool_round_trip<L: Llm>(
        path: &str,
        llm: fn(String, reqwest::Client) -> L,
        tool_call_response: fn(&str, &str, &Value) -> Value,
        tool_result_request: fn(&str, &str) -> Matcher,
        final_response: fn(&str) -> Value,
    ) {
        let mut mock = MockServer::start().await;
        // Turn 1: the model asks to call the tool (request body unconstrained).
        let response1 = tool_call_response(TOOL_CALL_ID, TOOL_NAME, &json!({ "vault": VAULT }));
        let t1 = mock.expect_post(path, Matcher::Any, response1).await;
        // Turn 2: only matches if the request carries the appended tool call + result.
        let request2 = tool_result_request(TOOL_CALL_ID, TOOL_NAME);
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
    async fn generic_chat_completes_tool_round_trip() {
        assert_tool_round_trip(
            "/chat/completions",
            mock::generic_chat,
            common::generic_chat_tool_call_response,
            partial_json!(common::generic_chat_tool_result_request),
            common::openai_chat_response,
        )
        .await;
    }

    #[tokio::test]
    async fn openai_chat_completes_tool_round_trip() {
        assert_tool_round_trip(
            "/chat/completions",
            mock::openai_chat,
            common::openai_chat_tool_call_response,
            partial_json!(common::openai_chat_tool_result_request),
            common::openai_chat_response,
        )
        .await;
    }

    #[tokio::test]
    async fn openai_responses_completes_tool_round_trip() {
        assert_tool_round_trip(
            "/responses",
            mock::openai_responses,
            common::openai_responses_tool_call_response,
            partial_json!(common::openai_responses_tool_result_request),
            common::openai_responses_response,
        )
        .await;
    }

    #[tokio::test]
    async fn xai_responses_completes_tool_round_trip() {
        assert_tool_round_trip(
            "/responses",
            mock::xai_responses,
            common::xai_responses_tool_call_response,
            partial_json!(common::xai_responses_tool_result_request),
            common::xai_responses_response,
        )
        .await;
    }

    #[tokio::test]
    async fn claude_messages_completes_tool_round_trip() {
        assert_tool_round_trip(
            "/messages",
            mock::claude_messages,
            common::claude_messages_tool_call_response,
            partial_json!(common::claude_messages_tool_result_request),
            common::claude_messages_response,
        )
        .await;
    }

    #[tokio::test]
    async fn gemini_generate_content_completes_tool_round_trip() {
        assert_tool_round_trip(
            &mock::gemini_generate_content_path(),
            mock::gemini_generate_content,
            common::gemini_generate_content_tool_call_response,
            partial_json!(common::gemini_generate_content_tool_result_request),
            common::gemini_generate_content_response,
        )
        .await;
    }

    #[tokio::test]
    async fn gemini_interactions_completes_tool_round_trip() {
        assert_tool_round_trip(
            "/v1/interactions",
            mock::gemini_interactions,
            common::gemini_interactions_tool_call_response,
            partial_json!(common::gemini_interactions_tool_result_request),
            common::gemini_interactions_response,
        )
        .await;
    }

    #[tokio::test]
    async fn sglang_completes_tool_round_trip() {
        assert_tool_round_trip(
            "/generate",
            mock::sglang_generate,
            common::sglang_generate_tool_call_response,
            common::sglang_generate_tool_result_request,
            common::sglang_generate_response,
        )
        .await;
    }
}

/// Runs the two-turn tool loop against a live LLM provider.
///
/// The tests hit real providers, cost money, and are non-deterministic, so every test is
/// `#[ignore]`d. Run the tests by:
///
/// ```sh
/// cargo test --test agent live::claude -- --ignored --nocapture
/// ```
mod live {
    use talin::agent::Agent;
    use talin::llm::Llm;
    use talin::tools::Tool;

    use crate::PROMPT;
    use crate::common::live;
    use crate::tool::{PASSPHRASE, ReadPassphrase, VAULT};

    /// Drives one provider through the two-turn loop: the tool ran once, and the passphrase it
    /// returned — absent from the prompt — came back in the answer.
    async fn assert_tool_round_trip<L: Llm>(llm: L) {
        let tool = ReadPassphrase::new();
        let calls = tool.calls.clone();

        let agent: Agent<_> = Agent::builder().tools([Tool::from(tool)]).build(llm);
        let mut session = agent.session();
        let answer = session
            .run_turn(PROMPT)
            .await
            .expect("agent turn failed to reach a final answer");

        let calls = calls.lock().unwrap().clone();
        eprintln!("tool calls: {calls:?}");
        eprintln!("final answer: {answer}");

        assert_eq!(calls.len(), 1, "tool should be executed exactly once");
        assert_eq!(calls[0], VAULT, "tool called with the wrong vault");
        assert!(
            answer.contains(PASSPHRASE),
            "expected final answer to contain {PASSPHRASE:?}, got {answer:?}"
        );
    }

    #[tokio::test]
    #[ignore = "live LLM: requires OPENAI_API_KEY"]
    async fn generic_chat_completes_tool_round_trip() {
        dotenvy::dotenv().ok();
        env_logger::init();
        assert_tool_round_trip(live::generic_chat()).await;
    }

    #[tokio::test]
    #[ignore = "live LLM: requires OPENAI_API_KEY"]
    async fn openai_chat_completes_tool_round_trip() {
        dotenvy::dotenv().ok();
        env_logger::init();
        assert_tool_round_trip(live::openai_chat()).await;
    }

    #[tokio::test]
    #[ignore = "live LLM: requires OPENAI_API_KEY"]
    async fn openai_responses_completes_tool_round_trip() {
        dotenvy::dotenv().ok();
        env_logger::init();
        assert_tool_round_trip(live::openai_responses()).await;
    }

    #[tokio::test]
    #[ignore = "live LLM: requires XAI_API_KEY"]
    async fn xai_responses_completes_tool_round_trip() {
        dotenvy::dotenv().ok();
        env_logger::init();
        assert_tool_round_trip(live::xai_responses()).await;
    }

    #[tokio::test]
    #[ignore = "live LLM: requires CLAUDE_API_KEY"]
    async fn claude_messages_completes_tool_round_trip() {
        dotenvy::dotenv().ok();
        env_logger::init();
        assert_tool_round_trip(live::claude_messages()).await;
    }

    #[tokio::test]
    #[ignore = "live LLM: requires GEMINI_API_KEY"]
    async fn gemini_generate_content_completes_tool_round_trip() {
        dotenvy::dotenv().ok();
        env_logger::init();
        assert_tool_round_trip(live::gemini_generate_content()).await;
    }

    #[tokio::test]
    #[ignore = "live LLM: requires GEMINI_API_KEY"]
    async fn gemini_interactions_completes_tool_round_trip() {
        dotenvy::dotenv().ok();
        env_logger::init();
        assert_tool_round_trip(live::gemini_interactions()).await;
    }
}
