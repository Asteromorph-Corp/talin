//! A failed tool call and its error are kept in the transcript, instead of being stripped.
#![allow(unreachable_pub)]

mod common;

const PROMPT: &str = "Use the `read_record` tool to fetch the record and report its exact contents.
You do not know which record id is valid.
If a call fails, read the error message carefully and try again with a valid id.";

/// A tool that fails unless it is given a valid id the caller cannot know up front.
///
/// The error message informs the model of the valid id.
mod tool {
    use std::error::Error;
    use std::sync::{Arc, Mutex};

    use async_trait::async_trait;
    use schemars::JsonSchema;
    use serde::Deserialize;
    use talin::tools::Function;

    /// Name of the failing lookup tool.
    pub const RECORD_TOOL_NAME: &str = "read_record";
    /// Tool-call id.
    pub const TOOL_CALL_ID: &str = "call_test";
    /// The only valid id for the tool. Not included in the initial prompt.
    pub const VALID_ID: u64 = 42;
    /// An id that does not exist, used to trigger the error.
    pub const INVALID_ID: u64 = 1;
    /// The record contents, returned only for [`VALID_ID`].
    pub const RECORD: &str = "Test secret.";

    /// The error text a failed [`ReadRecord`] call returns; the only place [`VALID_ID`] is
    /// revealed.
    pub fn record_error(id: u64) -> String {
        format!("no record with id={id}; the only valid record id is {VALID_ID}")
    }

    #[derive(Debug, Deserialize, JsonSchema)]
    pub struct ReadRecordInput {
        /// The numeric id of the record to read.
        pub id: u64,
    }

    /// A lookup tool whose valid input is unknowable a priori: a wrong id fails, and the error is
    /// the only place the caller can discover [`VALID_ID`]. Records every attempted id.
    pub struct ReadRecord {
        pub attempts: Arc<Mutex<Vec<u64>>>,
    }

    impl ReadRecord {
        pub fn new() -> Self {
            Self {
                attempts: Arc::new(Mutex::new(Vec::new())),
            }
        }
    }

    #[async_trait]
    impl Function for ReadRecord {
        type Input = ReadRecordInput;
        type Output = String;

        fn name(&self) -> String {
            RECORD_TOOL_NAME.to_string()
        }

        fn description(&self) -> String {
            "Reads a record from the test datastore by its numeric id and returns the record's contents."
                .to_string()
        }

        async fn call(
            &self,
            input: Self::Input,
        ) -> Result<Self::Output, Box<dyn Error + Send + Sync>> {
            self.attempts.lock().unwrap().push(input.id);
            if input.id == VALID_ID {
                Ok(RECORD.to_string())
            } else {
                Err(record_error(input.id).into())
            }
        }
    }
}

/// Sends the tool error to a mock server.
mod mock {
    use mockito::Matcher;
    use serde_json::{Value, json};
    use talin::agent::Agent;
    use talin::llm::Llm;
    use talin::tools::Tool;
    use talin::transcript::EntryView;

    use crate::PROMPT;
    use crate::common::{self, MockServer, mock};
    use crate::tool::{
        INVALID_ID, RECORD, RECORD_TOOL_NAME, ReadRecord, TOOL_CALL_ID, record_error,
    };

    /// Runs the two-turn loop over a failing tool:
    /// - Turn 1: the model calls the tool with an id that does not exist.
    /// - Turn 2: the request body is matched on the error text, which is only there if the failed
    ///   call and its error survived into the next request.
    async fn assert_sends_tool_error<L: Llm>(
        path: &str,
        llm: fn(String, reqwest::Client) -> L,
        tool_call_response: fn(&str, &str, &Value) -> Value,
        final_response: fn(&str) -> Value,
    ) {
        let mut mock = MockServer::start().await;
        let response1 =
            tool_call_response(TOOL_CALL_ID, RECORD_TOOL_NAME, &json!({ "id": INVALID_ID }));
        let t1 = mock.expect_post(path, Matcher::Any, response1).await;
        let request2 = Matcher::Regex(record_error(INVALID_ID));
        let t2 = mock
            .expect_post(path, request2, final_response(RECORD))
            .await;

        let tool = ReadRecord::new();
        let attempts = tool.attempts.clone();

        let llm = llm(mock.base_url(), mock.http_client());
        let agent: Agent<_> = Agent::builder().tools([Tool::from(tool)]).build(llm);
        let mut session = agent.session();
        let result = session.run_turn(PROMPT).await;

        t1.assert_async().await;
        t2.assert_async().await;
        result.expect("agent turn failed");

        assert_eq!(
            attempts.lock().unwrap().as_slice(),
            [INVALID_ID],
            "the tool should have been called once, with the invalid id"
        );
        let tool_errors = session
            .transcript
            .full_view()
            .iter()
            .filter(|entry| matches!(entry, EntryView::ToolError(..)))
            .count();
        assert_eq!(
            tool_errors, 1,
            "the tool error is missing from the transcript"
        );
    }

    #[tokio::test]
    async fn generic_chat_sends_tool_error() {
        assert_sends_tool_error(
            "/chat/completions",
            mock::generic_chat,
            common::generic_chat_tool_call_response,
            common::openai_chat_response,
        )
        .await;
    }

    #[tokio::test]
    async fn openai_chat_sends_tool_error() {
        assert_sends_tool_error(
            "/chat/completions",
            mock::openai_chat,
            common::openai_chat_tool_call_response,
            common::openai_chat_response,
        )
        .await;
    }

    #[tokio::test]
    async fn openai_responses_sends_tool_error() {
        assert_sends_tool_error(
            "/responses",
            mock::openai_responses,
            common::openai_responses_tool_call_response,
            common::openai_responses_response,
        )
        .await;
    }

    #[tokio::test]
    async fn xai_responses_sends_tool_error() {
        assert_sends_tool_error(
            "/responses",
            mock::xai_responses,
            common::xai_responses_tool_call_response,
            common::xai_responses_response,
        )
        .await;
    }

    #[tokio::test]
    async fn claude_messages_sends_tool_error() {
        assert_sends_tool_error(
            "/messages",
            mock::claude_messages,
            common::claude_messages_tool_call_response,
            common::claude_messages_response,
        )
        .await;
    }

    #[tokio::test]
    async fn gemini_generate_content_sends_tool_error() {
        assert_sends_tool_error(
            &mock::gemini_generate_content_path(),
            mock::gemini_generate_content,
            common::gemini_generate_content_tool_call_response,
            common::gemini_generate_content_response,
        )
        .await;
    }

    #[tokio::test]
    async fn gemini_interactions_sends_tool_error() {
        assert_sends_tool_error(
            "/v1/interactions",
            mock::gemini_interactions,
            common::gemini_interactions_tool_call_response,
            common::gemini_interactions_response,
        )
        .await;
    }

    #[tokio::test]
    async fn sglang_sends_tool_error() {
        assert_sends_tool_error(
            "/generate",
            mock::sglang_generate,
            common::sglang_generate_tool_call_response,
            common::sglang_generate_response,
        )
        .await;
    }
}

/// Recovers from the tool error against a live LLM provider.
///
/// The tests hit real providers, cost money, and are non-deterministic, so every test is
/// `#[ignore]`d. Run the tests by:
///
/// ```sh
/// cargo test --test tool_error_recovery live::claude -- --ignored --nocapture
/// ```
mod live {
    use talin::agent::Agent;
    use talin::llm::Llm;
    use talin::tools::Tool;
    use talin::transcript::EntryView;

    use crate::PROMPT;
    use crate::common::live;
    use crate::tool::{RECORD, ReadRecord, VALID_ID};

    /// Asserts the model read the error, retried with the id the error revealed, and reported the
    /// record it then reached.
    async fn assert_recovers_from_tool_error<L: Llm>(llm: L) {
        let tool = ReadRecord::new();
        let attempts = tool.attempts.clone();

        let agent: Agent<_> = Agent::builder().tools([Tool::from(tool)]).build(llm);
        let mut session = agent.session();
        let answer = session
            .run_turn(PROMPT)
            .await
            .expect("agent turn failed to reach a final answer");

        let tool_errors = session
            .transcript
            .full_view()
            .iter()
            .filter(|entry| matches!(entry, EntryView::ToolError(..)))
            .count();
        let attempts = attempts.lock().unwrap().clone();
        eprintln!("attempts: {attempts:?}");
        eprintln!("tool errors in transcript: {tool_errors}");
        eprintln!("final answer: {answer}");

        // The scenario only proves something if a call actually failed first.
        assert!(
            attempts.iter().any(|&id| id != VALID_ID),
            "expected at least one invalid attempt, got {attempts:?}"
        );
        assert!(
            tool_errors >= 1,
            "expected a ToolError entry in the transcript, found none"
        );
        assert!(
            attempts.contains(&VALID_ID),
            "the model never recovered to the valid id; attempts={attempts:?}"
        );
        assert!(
            answer.contains(RECORD),
            "expected final answer to contain {RECORD:?}, got {answer:?}"
        );
    }

    #[tokio::test]
    #[ignore = "live LLM: requires OPENAI_API_KEY"]
    async fn generic_chat_recovers_from_tool_error() {
        dotenvy::dotenv().ok();
        env_logger::init();
        assert_recovers_from_tool_error(live::generic_chat()).await;
    }

    #[tokio::test]
    #[ignore = "live LLM: requires OPENAI_API_KEY"]
    async fn openai_chat_recovers_from_tool_error() {
        dotenvy::dotenv().ok();
        env_logger::init();
        assert_recovers_from_tool_error(live::openai_chat()).await;
    }

    #[tokio::test]
    #[ignore = "live LLM: requires OPENAI_API_KEY"]
    async fn openai_responses_recovers_from_tool_error() {
        dotenvy::dotenv().ok();
        env_logger::init();
        assert_recovers_from_tool_error(live::openai_responses()).await;
    }

    #[tokio::test]
    #[ignore = "live LLM: requires XAI_API_KEY"]
    async fn xai_responses_recovers_from_tool_error() {
        dotenvy::dotenv().ok();
        env_logger::init();
        assert_recovers_from_tool_error(live::xai_responses()).await;
    }

    #[tokio::test]
    #[ignore = "live LLM: requires CLAUDE_API_KEY"]
    async fn claude_messages_recovers_from_tool_error() {
        dotenvy::dotenv().ok();
        env_logger::init();
        assert_recovers_from_tool_error(live::claude_messages()).await;
    }

    #[tokio::test]
    #[ignore = "live LLM: requires GEMINI_API_KEY"]
    async fn gemini_generate_content_recovers_from_tool_error() {
        dotenvy::dotenv().ok();
        env_logger::init();
        assert_recovers_from_tool_error(live::gemini_generate_content()).await;
    }

    #[tokio::test]
    #[ignore = "live LLM: requires GEMINI_API_KEY"]
    async fn gemini_interactions_recovers_from_tool_error() {
        dotenvy::dotenv().ok();
        env_logger::init();
        assert_recovers_from_tool_error(live::gemini_interactions()).await;
    }
}
