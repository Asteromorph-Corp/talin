//! Tests each provider returns a proper reply from [`invoke`]
//!
//! [`invoke`]: talin::llm::LlmExt::invoke

mod common;

/// Invokes each provider against a mock server.
mod mock {
    use mockito::Matcher;
    use serde_json::Value;
    use talin::llm::LlmExt;

    use crate::common::{self, MockServer, mock};

    const PING: &str = "ping";
    const PONG: &str = "pong";

    /// Asserts the reply text round-trips out of the provider's response shape.
    async fn assert_returns_model_reply<L: LlmExt>(
        path: &str,
        llm: fn(String, reqwest::Client) -> L,
        response: fn(&str) -> Value,
    ) {
        let mut mock = MockServer::start().await;
        let m = mock.expect_post(path, Matcher::Any, response(PONG)).await;

        let llm = llm(mock.base_url(), mock.http_client());
        let reply = llm.invoke(PING).await.expect("invoke failed");

        m.assert_async().await;
        assert!(
            reply.contains(PONG),
            "expected reply to contain {PONG:?}, got {reply:?}"
        );
    }

    #[tokio::test]
    async fn generic_chat_returns_model_reply() {
        assert_returns_model_reply(
            "/chat/completions",
            mock::generic_chat,
            common::openai_chat_response,
        )
        .await;
    }

    #[tokio::test]
    async fn openai_chat_returns_model_reply() {
        assert_returns_model_reply(
            "/chat/completions",
            mock::openai_chat,
            common::openai_chat_response,
        )
        .await;
    }

    #[tokio::test]
    async fn openai_responses_returns_model_reply() {
        assert_returns_model_reply(
            "/responses",
            mock::openai_responses,
            common::openai_responses_response,
        )
        .await;
    }

    #[tokio::test]
    async fn xai_responses_returns_model_reply() {
        assert_returns_model_reply(
            "/responses",
            mock::xai_responses,
            common::xai_responses_response,
        )
        .await;
    }

    #[tokio::test]
    async fn claude_messages_returns_model_reply() {
        assert_returns_model_reply(
            "/messages",
            mock::claude_messages,
            common::claude_messages_response,
        )
        .await;
    }

    #[tokio::test]
    async fn gemini_generate_content_returns_model_reply() {
        assert_returns_model_reply(
            &mock::gemini_generate_content_path(),
            mock::gemini_generate_content,
            common::gemini_generate_content_response,
        )
        .await;
    }

    #[tokio::test]
    async fn gemini_interactions_returns_model_reply() {
        assert_returns_model_reply(
            "/v1/interactions",
            mock::gemini_interactions,
            common::gemini_interactions_response,
        )
        .await;
    }

    #[tokio::test]
    async fn sglang_returns_model_reply() {
        assert_returns_model_reply(
            "/generate",
            mock::sglang_generate,
            common::sglang_generate_response,
        )
        .await;
    }
}

/// Invokes each provider against a live LLM provider.
///
/// The tests hit real providers, cost money, and are non-deterministic, so every test is
/// `#[ignore]`d. Run the tests by:
///
/// ```sh
/// cargo test --test provider live::claude -- --ignored --nocapture
/// ```
mod live {
    use talin::llm::LlmExt;

    use crate::common::live;

    const PING: &str = "Reply with exactly one word: pong";
    const PONG: &str = "pong";

    async fn assert_returns_model_reply<L: LlmExt>(llm: L) {
        let reply = llm.invoke(PING).await.expect("invoke failed");

        eprintln!("reply: {reply}");
        assert!(
            reply.to_lowercase().contains(PONG),
            "expected reply to contain {PONG:?}, got {reply:?}"
        );
    }

    #[tokio::test]
    #[ignore = "live LLM: requires OPENAI_API_KEY"]
    async fn generic_chat_returns_model_reply() {
        assert_returns_model_reply(live::generic_chat()).await;
    }

    #[tokio::test]
    #[ignore = "live LLM: requires OPENAI_API_KEY"]
    async fn openai_chat_returns_model_reply() {
        assert_returns_model_reply(live::openai_chat()).await;
    }

    #[tokio::test]
    #[ignore = "live LLM: requires OPENAI_API_KEY"]
    async fn openai_responses_returns_model_reply() {
        assert_returns_model_reply(live::openai_responses()).await;
    }

    #[tokio::test]
    #[ignore = "live LLM: requires XAI_API_KEY"]
    async fn xai_responses_returns_model_reply() {
        assert_returns_model_reply(live::xai_responses()).await;
    }

    #[tokio::test]
    #[ignore = "live LLM: requires CLAUDE_API_KEY"]
    async fn claude_messages_returns_model_reply() {
        assert_returns_model_reply(live::claude_messages()).await;
    }

    #[tokio::test]
    #[ignore = "live LLM: requires GEMINI_API_KEY"]
    async fn gemini_generate_content_returns_model_reply() {
        assert_returns_model_reply(live::gemini_generate_content()).await;
    }

    #[tokio::test]
    #[ignore = "live LLM: requires GEMINI_API_KEY"]
    async fn gemini_interactions_returns_model_reply() {
        assert_returns_model_reply(live::gemini_interactions()).await;
    }
}
