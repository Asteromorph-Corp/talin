//! Ask each provider for an answer constrained by a JSON schema:
//! - The request must carry the schema in the provider's own shape.
//! - The reply must deserialize into the type the schema came from.
//!
//! Schema dialects differ per provider, so a schema that one accepts another rejects — which the
//! mock module cannot see and the live module can.
#![allow(unreachable_pub)]

mod common;

/// The prompt. It names no field, so the field names in a request can only come from the schema.
pub const PROMPT: &str = "Report the record of the traveller Ada Lovelace, who died at 36.";

/// The type the model must answer with.
mod answer {
    use schemars::JsonSchema;
    use talin::chain::{ChainOutput, Ctor};
    use talin::schemas::ResponseFormat;

    /// The name of the response format.
    pub const FORMAT_NAME: &str = "traveller";
    /// A property of [`Traveller`], as the schema spells it.
    pub const PROPERTY: &str = "full_name";
    /// The values the model is expected to report.
    pub const FULL_NAME: &str = "Ada Lovelace";
    pub const AGE_YEARS: u32 = 36;

    #[derive(Debug, JsonSchema, ChainOutput, Ctor)]
    #[serde(deny_unknown_fields)]
    pub struct Traveller {
        /// The traveller's full name.
        pub full_name: String,
        /// The traveller's age in years.
        #[schemars(with = "i32")]
        pub age_years: u32,
    }

    /// A JSON-schema response format for [`Traveller`].
    pub fn response_format() -> ResponseFormat {
        ResponseFormat::json_schema(FORMAT_NAME, None, schemars::schema_for!(Traveller), true)
    }
}

/// Sends the schema to a mock server.
mod mock {
    use mockito::Matcher;
    use serde_json::{Value, json};
    use talin::agent::Agent;
    use talin::chain::StrCtor;
    use talin::llm::{Llm, LlmOptions};

    use crate::PROMPT;
    use crate::answer::{
        AGE_YEARS, FORMAT_NAME, FULL_NAME, PROPERTY, TravellerCtor, response_format,
    };
    use crate::common::{self, MockServer, mock};

    macro_rules! partial_json {
        ($request:path) => {
            |name: &str, property: &str| mockito::Matcher::PartialJson($request(name, property))
        };
    }

    /// Asserts the schema reaches the request, and the answer parses back into [`Traveller`].
    async fn assert_returns_structured_output<L>(
        path: &str,
        llm: fn(String, reqwest::Client) -> L,
        json_schema_request: fn(&str, &str) -> Matcher,
        response: fn(&str) -> Value,
    ) where
        L: Llm,
        L::Options: From<LlmOptions>,
    {
        let mut mock = MockServer::start().await;
        let reported = json!({ "full_name": FULL_NAME, "age_years": AGE_YEARS }).to_string();
        let request = json_schema_request(FORMAT_NAME, PROPERTY);
        let m = mock.expect_post(path, request, response(&reported)).await;

        let options = LlmOptions::default().with_response_format(response_format());
        let llm = llm(mock.base_url(), mock.http_client()).with_default_options(options);
        let agent: Agent<_, StrCtor, TravellerCtor> = Agent::builder().build(llm);
        let mut session = agent.session();
        let result = session.run_turn(PROMPT).await;

        m.assert_async().await;
        let answer = result.expect("agent turn failed");
        assert_eq!(answer.full_name, FULL_NAME);
        assert_eq!(answer.age_years, AGE_YEARS);
    }

    #[tokio::test]
    async fn generic_chat_returns_structured_output() {
        assert_returns_structured_output(
            "/chat/completions",
            mock::generic_chat,
            common::generic_chat_json_schema_request,
            common::openai_chat_response,
        )
        .await;
    }

    #[tokio::test]
    async fn openai_chat_returns_structured_output() {
        assert_returns_structured_output(
            "/chat/completions",
            mock::openai_chat,
            partial_json!(common::openai_chat_json_schema_request),
            common::openai_chat_response,
        )
        .await;
    }

    #[tokio::test]
    async fn openai_responses_returns_structured_output() {
        assert_returns_structured_output(
            "/responses",
            mock::openai_responses,
            partial_json!(common::openai_responses_json_schema_request),
            common::openai_responses_response,
        )
        .await;
    }

    #[tokio::test]
    async fn xai_responses_returns_structured_output() {
        assert_returns_structured_output(
            "/responses",
            mock::xai_responses,
            partial_json!(common::xai_responses_json_schema_request),
            common::xai_responses_response,
        )
        .await;
    }

    #[tokio::test]
    async fn claude_messages_returns_structured_output() {
        assert_returns_structured_output(
            "/messages",
            mock::claude_messages,
            partial_json!(common::claude_messages_json_schema_request),
            common::claude_messages_response,
        )
        .await;
    }

    #[tokio::test]
    async fn gemini_generate_content_returns_structured_output() {
        assert_returns_structured_output(
            &mock::gemini_generate_content_path(),
            mock::gemini_generate_content,
            partial_json!(common::gemini_generate_content_json_schema_request),
            common::gemini_generate_content_response,
        )
        .await;
    }

    #[tokio::test]
    async fn gemini_interactions_returns_structured_output() {
        assert_returns_structured_output(
            "/v1/interactions",
            mock::gemini_interactions,
            partial_json!(common::gemini_interactions_json_schema_request),
            common::gemini_interactions_response,
        )
        .await;
    }

    #[tokio::test]
    async fn sglang_returns_structured_output() {
        assert_returns_structured_output(
            "/generate",
            mock::sglang_generate,
            common::sglang_generate_json_schema_request,
            common::sglang_generate_response,
        )
        .await;
    }
}

/// Sends the schema to a live LLM provider.
///
/// The tests hit real providers, cost money, and are non-deterministic, so every test is
/// `#[ignore]`d. Run the tests by:
///
/// ```sh
/// cargo test --test structured_output live::claude -- --ignored --nocapture
/// ```
mod live {
    use talin::agent::Agent;
    use talin::chain::StrCtor;
    use talin::llm::{Llm, LlmOptions};

    use crate::PROMPT;
    use crate::answer::{AGE_YEARS, FULL_NAME, TravellerCtor, response_format};
    use crate::common::live;

    /// Asks one provider for the constrained answer, and parses it back into [`Traveller`].
    async fn assert_returns_structured_output<L>(llm: L)
    where
        L: Llm,
        L::Options: From<LlmOptions>,
    {
        let options = LlmOptions::default().with_response_format(response_format());
        let agent: Agent<_, StrCtor, TravellerCtor> =
            Agent::builder().build(llm.with_default_options(options));
        let mut session = agent.session();
        let answer = session
            .run_turn(PROMPT)
            .await
            .expect("agent turn failed to reach a final answer");

        eprintln!("answer: {answer:?}");
        assert!(
            answer.full_name.contains(FULL_NAME),
            "expected {FULL_NAME:?}, got {:?}",
            answer.full_name
        );
        assert_eq!(answer.age_years, AGE_YEARS);
    }

    #[tokio::test]
    #[ignore = "live LLM: requires OPENAI_API_KEY"]
    async fn generic_chat_returns_structured_output() {
        dotenvy::dotenv().ok();
        env_logger::init();
        assert_returns_structured_output(live::generic_chat()).await;
    }

    #[tokio::test]
    #[ignore = "live LLM: requires OPENAI_API_KEY"]
    async fn openai_chat_returns_structured_output() {
        dotenvy::dotenv().ok();
        env_logger::init();
        assert_returns_structured_output(live::openai_chat()).await;
    }

    #[tokio::test]
    #[ignore = "live LLM: requires OPENAI_API_KEY"]
    async fn openai_responses_returns_structured_output() {
        dotenvy::dotenv().ok();
        env_logger::init();
        assert_returns_structured_output(live::openai_responses()).await;
    }

    #[tokio::test]
    #[ignore = "live LLM: requires XAI_API_KEY"]
    async fn xai_responses_returns_structured_output() {
        dotenvy::dotenv().ok();
        env_logger::init();
        assert_returns_structured_output(live::xai_responses()).await;
    }

    #[tokio::test]
    #[ignore = "live LLM: requires CLAUDE_API_KEY"]
    async fn claude_messages_returns_structured_output() {
        dotenvy::dotenv().ok();
        env_logger::init();
        assert_returns_structured_output(live::claude_messages()).await;
    }

    #[tokio::test]
    #[ignore = "live LLM: requires GEMINI_API_KEY"]
    async fn gemini_generate_content_returns_structured_output() {
        dotenvy::dotenv().ok();
        env_logger::init();
        assert_returns_structured_output(live::gemini_generate_content()).await;
    }

    #[tokio::test]
    #[ignore = "live LLM: requires GEMINI_API_KEY"]
    async fn gemini_interactions_returns_structured_output() {
        dotenvy::dotenv().ok();
        env_logger::init();
        assert_returns_structured_output(live::gemini_interactions()).await;
    }
}
