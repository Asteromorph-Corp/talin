//! Output cut short by the token limit is continued from, instead of being dropped and reissued.
//!
//! The runner echoes the truncated model output back into the next request, followed by a prompt
//! to continue from it, then splices the halves into a single model message. Each provider marks
//! truncation differently, so both the marker and the shape of the echoed partial are per-provider.

mod common;

/// Continues a truncated output from a mock server.
mod mock {
    use mockito::Matcher;
    use serde_json::Value;
    use talin::agent::{Agent, AgentOptions};
    use talin::llm::Llm;
    use talin::schemas::{BodyMessage, ContentPart};
    use talin::transcript::scope::Last;
    use talin::transcript::{EntryView, Scope};

    use crate::common::{self, MockServer, mock};

    const PROMPT: &str = "write about the sea";
    const PARTIAL: &str = "the sea is wide";
    const REST: &str = " and deep";

    /// Counts the model and user messages in a transcript view.
    fn message_counts(entries: &[EntryView<'_>]) -> (usize, usize) {
        let models = entries
            .iter()
            .filter(|entry| {
                matches!(
                    entry,
                    EntryView::Message(_, BodyMessage::Assistant(_) | BodyMessage::ToolCall(_))
                )
            })
            .count();
        let users = entries
            .iter()
            .filter(|entry| matches!(entry, EntryView::Message(_, BodyMessage::User(_))))
            .count();
        (models, users)
    }

    /// Runs the two-call continuation:
    /// - Call 1: the response is cut off after [`PARTIAL`].
    /// - Call 2: the request body is matched on [`PARTIAL`], which is only there if the truncated
    ///   output was echoed back to continue from.
    async fn assert_continues_after_max_tokens<L: Llm>(
        path: &str,
        llm: fn(String, reqwest::Client) -> L,
        truncated_response: fn(&str) -> Value,
        response: fn(&str) -> Value,
        expected_messages: &[&str],
    ) {
        let mut mock = MockServer::start().await;
        let c1 = mock
            .expect_post(path, Matcher::Any, truncated_response(PARTIAL))
            .await;
        let request2 = Matcher::Regex(PARTIAL.to_string());
        let c2 = mock.expect_post(path, request2, response(REST)).await;

        let llm = llm(mock.base_url(), mock.http_client());
        let agent: Agent<_> = Agent::builder()
            .options(AgentOptions::default().with_max_continuations(1))
            .build(llm);
        let mut session = agent.session();
        let result = session.run_turn(PROMPT).await;

        c1.assert_async().await;
        c2.assert_async().await;
        let answer = result.expect("agent turn failed");

        assert_eq!(
            answer,
            format!("{PARTIAL}{REST}"),
            "the halves were not spliced back together"
        );
        // The splice is one message, and the continuation prompt does not outlive it.
        let entries = session.transcript.view(&Scope::from(Last::turn()));
        let (models, users) = message_counts(&entries);
        assert_eq!(models, 1, "the step holds more than one model message");
        assert_eq!(users, 1, "the continuation prompt outlived the merge");
        let text = entries
            .iter()
            .filter_map(|entry| match entry {
                EntryView::Message(_, BodyMessage::Assistant(message)) => {
                    Some(message.content.as_slice())
                }
                _ => None,
            })
            .flatten()
            .collect::<Vec<_>>();
        let expected = expected_messages
            .iter()
            .map(|text| ContentPart::Text((*text).to_owned()))
            .collect::<Vec<_>>();
        assert_eq!(text, expected.iter().collect::<Vec<_>>());
    }

    #[tokio::test]
    async fn generic_chat_continues_after_max_tokens() {
        assert_continues_after_max_tokens(
            "/chat/completions",
            mock::generic_chat,
            common::openai_chat_truncated_response,
            common::openai_chat_response,
            &[&format!("{PARTIAL}{REST}")],
        )
        .await;
    }

    #[tokio::test]
    async fn openai_chat_continues_after_max_tokens() {
        assert_continues_after_max_tokens(
            "/chat/completions",
            mock::openai_chat,
            common::openai_chat_truncated_response,
            common::openai_chat_response,
            &[PARTIAL, REST],
        )
        .await;
    }

    #[tokio::test]
    async fn openai_responses_continues_after_max_tokens() {
        assert_continues_after_max_tokens(
            "/responses",
            mock::openai_responses,
            common::openai_responses_truncated_response,
            common::openai_responses_response,
            &[PARTIAL, REST],
        )
        .await;
    }

    #[tokio::test]
    async fn xai_responses_continues_after_max_tokens() {
        assert_continues_after_max_tokens(
            "/responses",
            mock::xai_responses,
            common::xai_responses_truncated_response,
            common::xai_responses_response,
            &[PARTIAL, REST],
        )
        .await;
    }

    #[tokio::test]
    async fn claude_messages_continues_after_max_tokens() {
        assert_continues_after_max_tokens(
            "/messages",
            mock::claude_messages,
            common::claude_messages_truncated_response,
            common::claude_messages_response,
            &[PARTIAL, REST],
        )
        .await;
    }

    #[tokio::test]
    async fn claude_continuation_preserves_signed_thinking_and_discards_unfinished_calls() {
        use serde_json::json;
        use talin::llm::claude_messages::types::{ContentBlockParam, TextOrItems};
        use talin::schemas::LlmStep;
        use talin::transcript::Entry;

        let mut response = common::claude_messages_thought_tool_call_response(
            "call_unfinished",
            "unfinished_search",
            &json!({}),
            "\nReasoning before the partial text.\n",
        );
        response["content"]
            .as_array_mut()
            .unwrap()
            .insert(1, json!({"type":"text", "text":PARTIAL}));
        response["stop_reason"] = json!("max_tokens");
        let mut server = MockServer::start().await;
        let first = server
            .expect_post("/messages", Matcher::Any, response)
            .await;
        let second = server
            .expect_post(
                "/messages",
                Matcher::AllOf(vec![
                    Matcher::Regex(common::THOUGHT_SIGNATURE.into()),
                    Matcher::Regex(PARTIAL.into()),
                ]),
                common::claude_messages_response(REST),
            )
            .await;
        let llm = mock::claude_messages(server.base_url(), server.http_client());
        let agent = Agent::<_>::builder()
            .options(AgentOptions::default().with_max_continuations(1))
            .build(llm);
        let mut session = agent.session();
        let mut runner = session.turn((), PROMPT).await.unwrap();
        let pending = runner.half_step().await.unwrap();
        assert_eq!(
            pending.step.thought(),
            Some("\nReasoning before the partial text.\n")
        );
        assert!(matches!(&pending.step, LlmStep::Finish(message) if message.content.len() == 2));
        assert_eq!(
            pending.complete().await.unwrap(),
            Some(format!("{PARTIAL}{REST}"))
        );
        drop(runner);
        let native = session
            .transcript
            .entries()
            .iter()
            .find_map(|entry| match entry {
                Entry::Message(_, record)
                    if matches!(record.canonical(), BodyMessage::Assistant(_)) =>
                {
                    Some(record.native())
                }
                _ => None,
            })
            .unwrap();
        let Some(TextOrItems::Items(blocks)) = &native.content() else {
            panic!("expected native blocks")
        };
        assert!(
            matches!(blocks.as_slice(), [ContentBlockParam::Thinking(thought), ContentBlockParam::Text(prefix), ContentBlockParam::Text(rest)]
            if thought.signature == common::THOUGHT_SIGNATURE && prefix.text == PARTIAL && rest.text == REST)
        );
        first.assert_async().await;
        second.assert_async().await;
    }

    #[tokio::test]
    async fn gemini_generate_content_continues_after_max_tokens() {
        assert_continues_after_max_tokens(
            &mock::gemini_generate_content_path(),
            mock::gemini_generate_content,
            common::gemini_generate_content_truncated_response,
            common::gemini_generate_content_response,
            &[PARTIAL, REST],
        )
        .await;
    }

    #[tokio::test]
    async fn gemini_interactions_continues_after_max_tokens() {
        assert_continues_after_max_tokens(
            "/v1/interactions",
            mock::gemini_interactions,
            common::gemini_interactions_truncated_response,
            common::gemini_interactions_response,
            &[PARTIAL, REST],
        )
        .await;
    }

    #[tokio::test]
    async fn sglang_continues_after_max_tokens() {
        assert_continues_after_max_tokens(
            "/generate",
            mock::sglang_generate,
            common::sglang_generate_truncated_response,
            common::sglang_generate_response,
            &[&format!("{PARTIAL}{REST}")],
        )
        .await;
    }
}

/// Continues a truncated output from a live LLM provider.
///
/// The mock fixtures encode what we *believe* a truncated response looks like, and nothing else in
/// the suite ever sends a partial model turn back as input. Only a live provider can confirm both.
///
/// The tests hit real providers, cost money, and are non-deterministic, so every test is
/// `#[ignore]`d. Run the tests by:
///
/// ```sh
/// cargo test --test max_tokens_continuation live::claude -- --ignored --nocapture
/// ```
mod live {
    use talin::agent::{Agent, AgentOptions};
    use talin::llm::{Llm, LlmOptions};
    use talin::schemas::BodyMessage;
    use talin::transcript::EntryView;
    use talin::transcript::scope::{Last, Scope};

    use crate::common::live;

    /// Small enough that the model cannot finish the poem in one call, large enough to get a
    /// meaningful prefix out first.
    const MAX_TOKENS: u32 = 96;

    const PROMPT: &str = "Write a poem of exactly 12 lines about the sea. \
         Number each line from 1 to 12. End with the line `THE END`.";

    /// Asserts the turn survived truncation: the provider accepted the echoed partial, and the
    /// halves came back as a single model message longer than one call could hold.
    async fn assert_continues_after_max_tokens<L>(llm: L)
    where
        L: Llm,
        L::Options: From<LlmOptions>,
    {
        let agent: Agent<_> = Agent::builder()
            .options(AgentOptions::default().with_max_continuations(6))
            .build(live::with_max_tokens(llm, MAX_TOKENS));
        let mut session = agent.session();
        let answer = session
            .run_turn(PROMPT)
            .await
            .expect("agent turn failed to reach a final answer");

        let entries = session.transcript.view(&Scope::from(Last::turn()));
        let models = entries
            .iter()
            .filter(|entry| {
                matches!(
                    entry,
                    EntryView::Message(_, BodyMessage::Assistant(_) | BodyMessage::ToolCall(_))
                )
            })
            .count();
        let users = entries
            .iter()
            .filter(|entry| matches!(entry, EntryView::Message(_, BodyMessage::User(_))))
            .count();
        eprintln!("final answer:\n{answer}");
        eprintln!("model entries: {models}, user entries: {users}");

        // A single call capped at `MAX_TOKENS` could not have produced an answer this long, so the
        // turn was truncated and continued.
        assert!(
            answer.len() > MAX_TOKENS as usize * 4,
            "answer is short enough to have fit in one call, so nothing was continued: {answer}"
        );
        assert_eq!(models, 1, "the step holds more than one model message");
        assert_eq!(users, 1, "the continuation prompt outlived the merge");
    }

    #[tokio::test]
    #[ignore = "live LLM: requires OPENAI_API_KEY"]
    async fn generic_chat_continues_after_max_tokens() {
        dotenvy::dotenv().ok();
        env_logger::init();
        assert_continues_after_max_tokens(live::generic_chat()).await;
    }

    #[tokio::test]
    #[ignore = "live LLM: requires OPENAI_API_KEY"]
    async fn openai_chat_continues_after_max_tokens() {
        dotenvy::dotenv().ok();
        env_logger::init();
        assert_continues_after_max_tokens(live::openai_chat()).await;
    }

    #[tokio::test]
    #[ignore = "live LLM: requires OPENAI_API_KEY"]
    async fn openai_responses_continues_after_max_tokens() {
        dotenvy::dotenv().ok();
        env_logger::init();
        assert_continues_after_max_tokens(live::openai_responses()).await;
    }

    #[tokio::test]
    #[ignore = "live LLM: requires XAI_API_KEY"]
    async fn xai_responses_continues_after_max_tokens() {
        dotenvy::dotenv().ok();
        env_logger::init();
        assert_continues_after_max_tokens(live::xai_responses()).await;
    }

    #[tokio::test]
    #[ignore = "live LLM: requires CLAUDE_API_KEY"]
    async fn claude_messages_continues_after_max_tokens() {
        dotenvy::dotenv().ok();
        env_logger::init();
        assert_continues_after_max_tokens(live::claude_messages()).await;
    }

    #[tokio::test]
    #[ignore = "live LLM: requires GEMINI_API_KEY"]
    async fn gemini_generate_content_continues_after_max_tokens() {
        dotenvy::dotenv().ok();
        env_logger::init();
        assert_continues_after_max_tokens(live::gemini_generate_content()).await;
    }

    #[tokio::test]
    #[ignore = "live LLM: requires GEMINI_API_KEY"]
    async fn gemini_interactions_continues_after_max_tokens() {
        dotenvy::dotenv().ok();
        env_logger::init();
        assert_continues_after_max_tokens(live::gemini_interactions()).await;
    }
}
