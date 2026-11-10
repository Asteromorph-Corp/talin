//! Exercise compaction, request-hook failures, and retries through normal sessions.
mod common;

use std::error::Error;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use async_trait::async_trait;
use mockito::{Matcher, Server};
use serde::Serialize;
use talin::agent::{Agent, AgentError, AgentOptions};
use talin::chain::StringCtor;
use talin::llm::claude_messages::types::{
    ContentBlockParam, MessageParam, MessagesRequest, TextOrItems,
};
use talin::llm::{
    ClaudeMessages, ItemOf, LlmError, LlmRequestItem, LlmRequestSystem, OpenAIChat, SystemOf,
};
use talin::schemas::{BodyMessage, Usage, WithUsage};
use talin::strategy::{
    ClaudeMessagesCompactStrategy, ClaudeMessagesCompactor, CompactionCutoff, Compactor,
    CustomCompactStrategy, DropThoughtStrategy, InstructionCompactor, MessageCount, RequestParts,
    Strategy, UsageTrackStrategy,
};
use talin::tools::Function;
use talin::transcript::{AgentTranscript, Entry};
use uuid::Uuid;

use crate::common::mock;

type TestResult = Result<(), Box<dyn Error>>;

struct StoredNative {
    id: Uuid,
    wire: String,
}

fn originals<S, B>(
    transcript: &AgentTranscript<S, B>,
) -> Result<Vec<StoredNative>, serde_json::Error>
where
    S: LlmRequestSystem,
    B: LlmRequestItem + Serialize,
{
    transcript
        .entries()
        .iter()
        .filter_map(|entry| match entry {
            Entry::Message(id, record) => Some(
                serde_json::to_string(record.native()).map(|wire| StoredNative { id: *id, wire }),
            ),
            Entry::ToolError(id, record) => Some(
                serde_json::to_string(record.native()).map(|wire| StoredNative { id: *id, wire }),
            ),
            _ => None,
        })
        .collect()
}

fn assert_originals_unchanged<S, B>(transcript: &AgentTranscript<S, B>, before: &[StoredNative])
where
    S: LlmRequestSystem,
    B: LlmRequestItem + Serialize,
{
    let after = originals(transcript).unwrap();
    for original in before {
        let current = after
            .iter()
            .find(|entry| entry.id == original.id)
            .expect("original UUID disappeared");
        assert_eq!(current.wire, original.wire, "stored native history changed");
    }
}

/// Asserts that every compacting model record directly follows a marker and that every marker
/// precedes a compacting record or a native summary. Returns the number of markers.
fn assert_markers_precede_compactions<S, B>(transcript: &AgentTranscript<S, B>) -> usize
where
    S: LlmRequestSystem,
    B: LlmRequestItem,
{
    let entries = transcript.entries();
    let mut markers = 0;
    for (index, entry) in entries.iter().enumerate() {
        match entry {
            Entry::CompactionMarker { .. } => {
                markers += 1;
                let next = entries.get(index + 1);
                assert!(
                    matches!(next, Some(Entry::Message(_, record)) if record.native().contains_native_compaction())
                        || matches!(next, Some(Entry::Native(..))),
                    "a compaction marker is not followed by a compaction or summary"
                );
            }
            Entry::Message(_, record) if record.native().contains_native_compaction() => {
                assert!(
                    matches!(
                        entries.get(index.wrapping_sub(1)),
                        Some(Entry::CompactionMarker { .. })
                    ),
                    "a compacting model record has no preceding marker"
                );
            }
            _ => {}
        }
    }
    markers
}

/// An application compactor returning a previously obtained native payload verbatim.
struct NativeCompactor {
    payload: MessageParam,
    calls: Arc<AtomicUsize>,
}

#[async_trait]
impl Compactor<ClaudeMessages> for NativeCompactor {
    async fn compact(
        &mut self,
        _llm: &ClaudeMessages,
        _context: (SystemOf<ClaudeMessages>, Vec<MessageParam>),
        _parts: &RequestParts<ClaudeMessages>,
    ) -> Result<WithUsage<Option<MessageParam>>, LlmError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(WithUsage::new(Some(self.payload.clone()), Usage::default()))
    }
}

#[tokio::test]
async fn successful_compaction_survives_generation_failure_without_reapplying_the_cutoff()
-> TestResult {
    let mut server = Server::new_async().await;
    let initial = server
        .mock("POST", "/messages")
        .with_status(200)
        .with_body(common::claude_messages_response("initial reply").to_string())
        .expect(1)
        .create_async()
        .await;
    let agent: Agent<_> = Agent::builder()
        .options(AgentOptions::default().with_max_consecutive_fails(2))
        .build(mock::claude_messages(server.url(), reqwest::Client::new()));
    let mut session = agent.session();
    session.run_turn("initial request").await?;
    initial.assert_async().await;
    initial.remove_async().await;
    let before = originals(&session.transcript)?;
    let (cutoff, payload) = session
        .transcript
        .entries()
        .iter()
        .rev()
        .find_map(|entry| match entry {
            Entry::Message(id, record) => Some((*id, record.native().clone())),
            _ => None,
        })
        .unwrap();
    // A refusal fails the generation step without an HTTP error, so the runner (not the HTTP
    // client) retries it.
    let mut refusal = common::claude_messages_response("");
    refusal["stop_reason"] = serde_json::json!("refusal");
    let failure = server
        .mock("POST", "/messages")
        .with_status(200)
        .with_body(refusal.to_string())
        .expect(1)
        .create_async()
        .await;
    let success = server
        .mock("POST", "/messages")
        .with_status(200)
        .with_body(common::claude_messages_response("retried reply").to_string())
        .expect(1)
        .create_async()
        .await;
    let calls = Arc::new(AtomicUsize::new(0));
    let strategy = CustomCompactStrategy::new(
        NativeCompactor {
            payload,
            calls: calls.clone(),
        },
        CompactionCutoff(cutoff),
    );
    session.compaction = Some(Box::new(strategy));
    session
        .run_turn_with_strategy(DropThoughtStrategy, "next request")
        .await?;
    failure.assert_async().await;
    success.assert_async().await;
    assert_eq!(
        calls.load(Ordering::SeqCst),
        1,
        "generation retry repeated a committed compaction"
    );
    assert_originals_unchanged(&session.transcript, &before);
    assert!(
        session
            .transcript
            .entries()
            .iter()
            .any(|entry| matches!(entry, Entry::CompactionMarker { .. }))
    );
    assert!(
        session
            .transcript
            .clone()
            .migrate::<Vec<String>, BodyMessage>()
            .is_err(),
        "migration silently discarded native-only history"
    );
    Ok(())
}

#[tokio::test]
async fn incomplete_summary_fails_without_changing_history() -> TestResult {
    let mut server = Server::new_async().await;
    let initial = server
        .mock("POST", "/messages")
        .with_status(200)
        .with_body(common::claude_messages_response("initial reply").to_string())
        .expect(1)
        .create_async()
        .await;
    let agent: Agent<_> = Agent::builder()
        .options(AgentOptions::default().with_max_consecutive_fails(1))
        .build(mock::claude_messages(server.url(), reqwest::Client::new()));
    let mut session = agent.session();
    session.run_turn("initial request").await?;
    initial.remove_async().await;
    let before = originals(&session.transcript)?;
    let cutoff = before.last().unwrap().id;
    let compaction = server
        .mock("POST", "/messages")
        .match_body(Matcher::PartialJsonString(
            r#"{"compaction":{"type":"summarize"}}"#.into(),
        ))
        .with_status(200)
        .with_body(include_str!(
            "fixtures/on-demand-compaction/no-summary.json"
        ))
        .expect(1)
        .create_async()
        .await;
    let generation = server
        .mock("POST", "/messages")
        .match_body(Matcher::Regex("next request".into()))
        .with_status(200)
        .with_body(common::claude_messages_response("next reply").to_string())
        .expect(0)
        .create_async()
        .await;
    let strategy =
        CustomCompactStrategy::new(ClaudeMessagesCompactor::default(), CompactionCutoff(cutoff));
    session.compaction = Some(Box::new(strategy));
    let result = session.run_turn("next request").await;
    assert!(matches!(
        result,
        Err(AgentError::Model(LlmError::InvalidResponse(_)))
    ));
    compaction.assert_async().await;
    generation.assert_async().await;
    assert_originals_unchanged(&session.transcript, &before);
    assert!(
        !session
            .transcript
            .entries()
            .iter()
            .any(|entry| matches!(entry, Entry::CompactionMarker { .. }))
    );
    assert!(
        !session
            .transcript
            .entries()
            .iter()
            .any(|entry| matches!(entry, Entry::Native(..)))
    );
    Ok(())
}

#[tokio::test]
async fn compaction_block_is_kept_regardless_of_surrounding_content() -> TestResult {
    let mut server = Server::new_async().await;
    let initial = server
        .mock("POST", "/messages")
        .with_status(200)
        .with_body(common::claude_messages_response("initial reply").to_string())
        .expect(1)
        .create_async()
        .await;
    let agent: Agent<_> =
        Agent::builder().build(mock::claude_messages(server.url(), reqwest::Client::new()));
    let mut session = agent.session();
    session.run_turn("initial request").await?;
    initial.remove_async().await;
    let cutoff = originals(&session.transcript)?.last().unwrap().id;
    let block = serde_json::json!({
        "type": "compaction", "content": "summary", "signature": "sig", "tool_changes": null
    });
    let response = serde_json::json!({
        "id": "msg_compact", "type": "message", "role": "assistant", "model": "claude-test",
        "content": [
            {"type": "thinking", "thinking": "", "signature": "thought"},
            block,
            {"type": "text", "text": "trailing text", "citations": null}
        ],
        "stop_reason": "end_turn", "stop_sequence": null,
        "usage": {"input_tokens": 1, "output_tokens": 1}
    });
    let compaction = server
        .mock("POST", "/messages")
        .match_body(Matcher::PartialJsonString(
            r#"{"compaction":{"type":"summarize"}}"#.into(),
        ))
        .with_status(200)
        .with_body(response.to_string())
        .expect(1)
        .create_async()
        .await;
    let generation = server
        .mock("POST", "/messages")
        .match_body(Matcher::Regex("next request".into()))
        .with_status(200)
        .with_body(common::claude_messages_response("next reply").to_string())
        .expect(1)
        .create_async()
        .await;
    session.compaction = Some(Box::new(CustomCompactStrategy::new(
        ClaudeMessagesCompactor::default(),
        CompactionCutoff(cutoff),
    )));
    session.run_turn("next request").await?;
    compaction.assert_async().await;
    generation.assert_async().await;
    let summary = session
        .transcript
        .entries()
        .iter()
        .find_map(|entry| match entry {
            Entry::Native(_, native) => Some(serde_json::to_value(native)),
            _ => None,
        })
        .expect("the compaction was not stored")?;
    assert_eq!(
        summary,
        serde_json::json!({"role": "assistant", "content": [block]})
    );
    Ok(())
}

const SUMMARIZE: &str = "Summarize the conversation so far.";

#[tokio::test]
async fn instruction_compactor_summarizes_claude_history_without_tools() -> TestResult {
    let mut server = Server::new_async().await;
    let initial = server
        .mock("POST", "/messages")
        .with_status(200)
        .with_body(common::claude_messages_response("initial reply").to_string())
        .expect(1)
        .create_async()
        .await;
    let agent: Agent<_> =
        Agent::builder().build(mock::claude_messages(server.url(), reqwest::Client::new()));
    let mut session = agent.session();
    session.run_turn("initial request").await?;
    initial.remove_async().await;
    let summarize = server
        .mock("POST", "/messages")
        .match_body(Matcher::AllOf(vec![
            Matcher::Regex(SUMMARIZE.into()),
            Matcher::PartialJsonString(r#"{"tool_choice":{"type":"none"}}"#.into()),
        ]))
        .with_status(200)
        .with_body(common::claude_messages_response("SUMMARY").to_string())
        .expect(1)
        .create_async()
        .await;
    let generation = server
        .mock("POST", "/messages")
        .match_body(Matcher::AllOf(vec![
            Matcher::Regex("SUMMARY".into()),
            Matcher::Regex("next request".into()),
        ]))
        .with_status(200)
        .with_body(common::claude_messages_response("next reply").to_string())
        .expect(1)
        .create_async()
        .await;
    session.compaction = Some(Box::new(CustomCompactStrategy::new(
        InstructionCompactor::new(SUMMARIZE),
        MessageCount {
            threshold: 2,
            retain_messages: 1,
        },
    )));
    session.run_turn("next request").await?;
    summarize.assert_async().await;
    generation.assert_async().await;
    let summary = session
        .transcript
        .entries()
        .iter()
        .find_map(|entry| match entry {
            Entry::Native(_, native) => Some(serde_json::to_value(native)),
            _ => None,
        })
        .expect("the summary was not stored")?;
    assert_eq!(summary["content"][0]["text"], "SUMMARY");
    Ok(())
}

#[tokio::test]
async fn instruction_compactor_works_without_native_compaction() -> TestResult {
    let mut server = Server::new_async().await;
    let initial = server
        .mock("POST", "/chat/completions")
        .with_status(200)
        .with_body(common::openai_chat_response("initial reply").to_string())
        .expect(1)
        .create_async()
        .await;
    let agent: Agent<_> =
        Agent::builder().build(mock::openai_chat(server.url(), reqwest::Client::new()));
    let mut session = agent.session();
    session.run_turn("initial request").await?;
    initial.remove_async().await;
    let summarize = server
        .mock("POST", "/chat/completions")
        .match_body(Matcher::Regex(SUMMARIZE.into()))
        .with_status(200)
        .with_body(common::openai_chat_response("SUMMARY").to_string())
        .expect(1)
        .create_async()
        .await;
    let generation = server
        .mock("POST", "/chat/completions")
        .match_body(Matcher::AllOf(vec![
            Matcher::Regex("SUMMARY".into()),
            Matcher::Regex("next request".into()),
        ]))
        .with_status(200)
        .with_body(common::openai_chat_response("next reply").to_string())
        .expect(1)
        .create_async()
        .await;
    session.compaction = Some(Box::new(CustomCompactStrategy::new(
        InstructionCompactor::new(SUMMARIZE),
        MessageCount {
            threshold: 2,
            retain_messages: 1,
        },
    )));
    let answer = session.run_turn("next request").await?;
    assert_eq!(answer, "next reply");
    summarize.assert_async().await;
    generation.assert_async().await;
    assert!(
        session
            .transcript
            .entries()
            .iter()
            .any(|entry| matches!(entry, Entry::CompactionMarker { .. }))
    );
    Ok(())
}

#[tokio::test]
async fn token_count_failure_leaves_original_history_untouched() -> TestResult {
    let mut server = Server::new_async().await;
    let initial = server
        .mock("POST", "/messages")
        .with_status(200)
        .with_body(common::claude_messages_response("initial reply").to_string())
        .expect(1)
        .create_async()
        .await;
    let agent: Agent<_> = Agent::builder()
        .options(AgentOptions::default().with_max_consecutive_fails(1))
        .build(mock::claude_messages(server.url(), reqwest::Client::new()));
    let mut session = agent.session();
    session.run_turn("initial request").await?;
    initial.remove_async().await;
    let before = originals(&session.transcript)?;
    let count = server.mock("POST", "/messages/count_tokens")
        .with_status(400).with_body(r#"{"type":"error","error":{"type":"invalid_request_error","message":"count unavailable"}}"#)
        .expect(1).create_async().await;
    session.compaction = Some(Box::new(ClaudeMessagesCompactStrategy::new(1, 1)));
    let result = session.run_turn("next request").await;
    assert!(result.is_err());
    count.assert_async().await;
    assert_originals_unchanged(&session.transcript, &before);
    assert!(
        !session
            .transcript
            .entries()
            .iter()
            .any(|entry| matches!(entry, Entry::CompactionMarker { .. }))
    );
    Ok(())
}

struct SkipCompactor(Arc<AtomicUsize>);

#[async_trait]
impl Compactor<ClaudeMessages> for SkipCompactor {
    async fn compact(
        &mut self,
        _llm: &ClaudeMessages,
        _context: (SystemOf<ClaudeMessages>, Vec<MessageParam>),
        _parts: &RequestParts<ClaudeMessages>,
    ) -> Result<WithUsage<Option<MessageParam>>, LlmError> {
        self.0.fetch_add(1, Ordering::SeqCst);
        Ok(WithUsage::new(None, Usage::default()))
    }
}

#[tokio::test]
async fn truncated_output_continuations_disable_compaction() -> TestResult {
    let mut server = Server::new_async().await;
    let initial = server
        .mock("POST", "/messages")
        .with_status(200)
        .with_body(common::claude_messages_response("initial reply").to_string())
        .expect(1)
        .create_async()
        .await;
    let agent: Agent<_> = Agent::builder()
        .options(AgentOptions::default().with_max_continuations(1))
        .build(mock::claude_messages(server.url(), reqwest::Client::new()));
    let mut session = agent.session();
    session.run_turn("initial request").await?;
    initial.assert_async().await;
    initial.remove_async().await;
    let cutoff = originals(&session.transcript)?.last().unwrap().id;
    let calls = Arc::new(AtomicUsize::new(0));
    session.compaction = Some(Box::new(CustomCompactStrategy::new(
        SkipCompactor(calls.clone()),
        CompactionCutoff(cutoff),
    )));
    let first = server
        .mock("POST", "/messages")
        .with_status(200)
        .with_body(common::claude_messages_truncated_response("partial").to_string())
        .expect(1)
        .create_async()
        .await;
    let continuation = server
        .mock("POST", "/messages")
        .match_body(Matcher::Regex("partial".into()))
        .with_status(200)
        .with_body(common::claude_messages_response("rest").to_string())
        .expect(1)
        .create_async()
        .await;
    session.run_turn("request").await?;
    first.assert_async().await;
    continuation.assert_async().await;
    assert_eq!(
        calls.load(Ordering::SeqCst),
        1,
        "compaction was enabled during a partial model response"
    );
    Ok(())
}

#[derive(serde::Deserialize, schemars::JsonSchema)]
struct EmptyInput {}

struct CompleteTool;

#[async_trait]
impl Function for CompleteTool {
    type Input = EmptyInput;
    type Output = String;

    fn name(&self) -> String {
        "complete_tool".into()
    }
    fn description(&self) -> String {
        "Completes one local tool call.".into()
    }
    async fn call(&self, _: EmptyInput) -> Result<Self::Output, Box<dyn Error + Send + Sync>> {
        Ok("tool completed".into())
    }
}

#[tokio::test]
async fn custom_selector_cannot_separate_a_client_call_from_its_results() -> TestResult {
    let mut server = Server::new_async().await;
    let call = server
        .mock("POST", "/messages")
        .with_status(200)
        .with_body(
            common::claude_messages_tool_call_response(
                "call_test",
                "complete_tool",
                &serde_json::json!({}),
            )
            .to_string(),
        )
        .expect(1)
        .create_async()
        .await;
    let answer = server
        .mock("POST", "/messages")
        .match_body(Matcher::Regex("tool_result".into()))
        .with_status(200)
        .with_body(common::claude_messages_response("finished").to_string())
        .expect(1)
        .create_async()
        .await;
    let agent: Agent<_> = Agent::builder()
        .tools(vec![CompleteTool.into()])
        .options(AgentOptions::default().with_max_consecutive_fails(1))
        .build(mock::claude_messages(server.url(), reqwest::Client::new()));
    let mut session = agent.session();
    session.run_turn("request").await?;
    call.assert_async().await;
    answer.assert_async().await;
    let before = originals(&session.transcript)?;
    let cutoff = session
        .transcript
        .entries()
        .iter()
        .find_map(|entry| match entry {
            Entry::Message(id, record)
                if matches!(record.canonical(), BodyMessage::ToolCall(_)) =>
            {
                Some(*id)
            }
            _ => None,
        })
        .unwrap();
    let payload = session
        .transcript
        .entries()
        .iter()
        .rev()
        .find_map(|entry| match entry {
            Entry::Message(_, record) => Some(record.native().clone()),
            _ => None,
        })
        .unwrap();
    let calls = Arc::new(AtomicUsize::new(0));
    let strategy = CustomCompactStrategy::new(
        NativeCompactor {
            payload,
            calls: calls.clone(),
        },
        CompactionCutoff(cutoff),
    );
    session.compaction = Some(Box::new(strategy));
    let error = session
        .run_turn("next request")
        .await
        .expect_err("invalid cutoff unexpectedly succeeded");
    assert!(
        error
            .to_string()
            .contains("cannot split a client tool exchange")
    );
    assert_eq!(
        calls.load(Ordering::SeqCst),
        0,
        "invalid selection reached the compactor"
    );
    assert!(
        !session
            .transcript
            .entries()
            .iter()
            .any(|entry| matches!(entry, Entry::CompactionMarker { .. }))
    );
    assert_originals_unchanged(&session.transcript, &before);
    Ok(())
}

struct RejectOnce(bool);

struct AdvisoryMarker {
    annotate: bool,
    requests: Vec<MessagesRequest>,
}

#[async_trait]
impl Strategy<ClaudeMessages, StringCtor> for AdvisoryMarker {
    type Output = Vec<MessagesRequest>;
    type Error = LlmError;

    async fn edit_transcript(
        &mut self,
        transcript: &mut AgentTranscript<SystemOf<ClaudeMessages>, MessageParam>,
    ) -> Result<(), Self::Error> {
        if self.annotate {
            let id = transcript
                .entries()
                .iter()
                .rev()
                .find_map(|entry| match entry {
                    Entry::Message(id, _) => Some(*id),
                    _ => None,
                })
                .unwrap();
            transcript.mark_as_compaction(id, true);
        }
        Ok(())
    }

    async fn process_request(
        &mut self,
        request: MessagesRequest,
    ) -> Result<MessagesRequest, Self::Error> {
        self.requests.push(request.clone());
        Ok(request)
    }

    async fn finalize(self) -> Result<Self::Output, Self::Error> {
        Ok(self.requests)
    }
}

#[tokio::test]
async fn compaction_marker_does_not_require_a_native_only_summary() -> TestResult {
    let mut server = Server::new_async().await;
    let generation = server
        .mock("POST", "/messages")
        .with_status(200)
        .with_body(common::claude_messages_response("reply").to_string())
        .expect(4)
        .create_async()
        .await;
    let agent: Agent<_> =
        Agent::builder().build(mock::claude_messages(server.url(), reqwest::Client::new()));
    let mut requests = Vec::new();
    for annotate in [false, true] {
        let mut session = agent.session();
        session.run_turn("first request").await?;
        let result = session
            .run_turn_with_strategy(
                AdvisoryMarker {
                    annotate,
                    requests: Vec::new(),
                },
                "next request",
            )
            .await?;
        requests.extend(result.extra_content);
    }
    assert_eq!(requests.len(), 2);
    let mut expected = requests.remove(0);
    let marked = requests.remove(0);
    let retained = expected.messages.pop().unwrap();
    expected.messages = vec![retained];
    assert_eq!(
        serde_json::to_value(marked)?,
        serde_json::to_value(expected)?,
        "the backend did not honor the marker before a canonical message"
    );
    generation.assert_async().await;
    Ok(())
}

#[async_trait]
impl Strategy<ClaudeMessages, StringCtor> for RejectOnce {
    type Output = ();
    type Error = LlmError;

    async fn process_request(
        &mut self,
        request: MessagesRequest,
    ) -> Result<MessagesRequest, Self::Error> {
        let has_compaction = request.messages.iter().any(|message| {
            matches!(&message.content(), Some(TextOrItems::Items(blocks))
                if blocks.iter().any(|block| matches!(block, ContentBlockParam::Compaction(_))))
        });
        if has_compaction && std::mem::replace(&mut self.0, false) {
            return Err(LlmError::InvalidRequest("request rejected once".into()));
        }
        Ok(request)
    }
    async fn finalize(self) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[tokio::test]
async fn successful_compaction_usage_survives_a_later_request_hook_error() {
    let mut server = Server::new_async().await;
    let initial = server
        .mock("POST", "/messages")
        .with_status(200)
        .with_body(common::claude_messages_response("reply").to_string())
        .expect(1)
        .create_async()
        .await;
    let agent: Agent<_> = Agent::builder()
        .options(AgentOptions::default().with_max_consecutive_fails(2))
        .build(mock::claude_messages(server.url(), reqwest::Client::new()));
    let mut session = agent.session();
    session.run_turn("first request").await.unwrap();
    initial.remove_async().await;
    let cutoff = session
        .transcript
        .entries()
        .iter()
        .rev()
        .find_map(|entry| match entry {
            Entry::Message(id, _) => Some(*id),
            _ => None,
        })
        .unwrap();
    let mut response = common::claude_messages_response("unused");
    response["content"] =
        serde_json::json!([{"type":"compaction","content":"summary","signature":"signed"}]);
    response["stop_reason"] = "compaction".into();
    response["usage"] = serde_json::json!({"input_tokens":17,"output_tokens":3});
    let compaction = server
        .mock("POST", "/messages")
        .match_body(Matcher::PartialJsonString(
            r#"{"compaction":{"type":"summarize"}}"#.into(),
        ))
        .with_status(200)
        .with_body(response.to_string())
        .expect(1)
        .create_async()
        .await;
    let generation = server
        .mock("POST", "/messages")
        .match_body(Matcher::Regex("next request".into()))
        .with_status(200)
        .with_body(common::claude_messages_response("next answer").to_string())
        .expect(1)
        .create_async()
        .await;
    session.compaction = Some(Box::new(CustomCompactStrategy::new(
        ClaudeMessagesCompactor::default(),
        CompactionCutoff(cutoff),
    )));
    let output = session
        .run_turn_with_strategy(
            (Some(RejectOnce(true)), UsageTrackStrategy::default()),
            "next request",
        )
        .await
        .unwrap();
    compaction.assert_async().await;
    generation.assert_async().await;
    let tokens: u32 = output
        .extra_content
        .1
        .0
        .values()
        .flat_map(|lines| lines.values())
        .map(|line| line.count)
        .sum();
    assert_eq!(
        tokens, 22,
        "paid compaction usage disappeared when a later hook failed"
    );
}

#[tokio::test]
async fn claude_uses_the_latest_native_block_without_a_compaction_strategy() -> TestResult {
    let mut server = Server::new_async().await;
    let agent: Agent<_> = Agent::builder()
        .options(AgentOptions::default().with_max_consecutive_fails(1))
        .build(mock::claude_messages(server.url(), reqwest::Client::new()));
    let mut session = agent.session();
    for turn in 1..=2 {
        let mut response = common::claude_messages_response("answer");
        response["content"] = serde_json::json!([
            {"type":"text", "text":"before compaction"},
            {"type":"compaction", "content":format!("summary {turn}"), "signature":"signed"},
            {"type":"compaction", "content":null},
            {"type":"text", "text":"answer"}
        ]);
        let request = if turn == 1 {
            Matcher::Any
        } else {
            Matcher::PartialJson(serde_json::json!({"messages":[
                {"role":"assistant", "content":[
                    {"type":"compaction", "content":"summary 1", "signature":"signed"},
                    {"type":"compaction", "content":null},
                    {"type":"text", "text":"answer"}
                ]},
                {"role":"user", "content":"request 2"}
            ]}))
        };
        let mock = server
            .mock("POST", "/messages")
            .match_body(request)
            .with_status(200)
            .with_body(response.to_string())
            .expect(1)
            .create_async()
            .await;
        session.run_turn(&format!("request {turn}")).await?;
        mock.assert_async().await;
        mock.remove_async().await;
    }
    let before = originals(&session.transcript)?;
    let mock = server
        .mock("POST", "/messages")
        .match_body(Matcher::PartialJson(serde_json::json!({"messages":[
            {"role":"assistant", "content":[
                {"type":"compaction", "content":"summary 2", "signature":"signed"},
                {"type":"compaction", "content":null},
                {"type":"text", "text":"answer"}
            ]},
            {"role":"user", "content":"request 3"}
        ]})))
        .with_status(200)
        .with_body(common::claude_messages_response("done").to_string())
        .expect(1)
        .create_async()
        .await;
    session.run_turn("request 3").await?;
    mock.assert_async().await;
    assert_originals_unchanged(&session.transcript, &before);
    assert_eq!(assert_markers_precede_compactions(&session.transcript), 2);
    // Migration loses the native compactions, so their markers must not truncate the history.
    let migrated = session
        .transcript
        .clone()
        .migrate::<SystemOf<OpenAIChat>, ItemOf<OpenAIChat>>()?;
    assert!(
        !migrated
            .entries()
            .iter()
            .any(|entry| matches!(entry, Entry::CompactionMarker { .. }))
    );
    assert_eq!(
        migrated.materialize().1.len(),
        session.transcript.materialize().1.len()
    );
    Ok(())
}

#[tokio::test]
async fn responses_uses_the_latest_native_window_without_a_compaction_strategy() -> TestResult {
    let mut server = Server::new_async().await;
    let agent: Agent<_> = Agent::builder()
        .options(AgentOptions::default().with_max_consecutive_fails(1))
        .build(mock::openai_responses(server.url(), reqwest::Client::new()));
    let mut session = agent.session();
    let mut window = Vec::new();
    for turn in 1..=3 {
        let mut response = common::openai_responses_response("answer");
        if turn < 3 {
            response["output"].as_array_mut().unwrap().insert(0, serde_json::json!({
                "type":"compaction", "id":format!("cmp_{turn}"), "encrypted_content":format!("opaque_{turn}")
            }));
        }
        let mut input = Vec::new();
        input.extend(window);
        input.push(serde_json::json!({"role":"user", "content":[{"type":"input_text", "text":format!("request {turn}")}]}));
        let mock = server
            .mock("POST", "/responses")
            .match_body(Matcher::PartialJson(serde_json::json!({"input":input})))
            .with_status(200)
            .with_body(response.to_string())
            .expect(1)
            .create_async()
            .await;
        let before = originals(&session.transcript)?;
        session.run_turn(&format!("request {turn}")).await?;
        mock.assert_async().await;
        mock.remove_async().await;
        assert_originals_unchanged(&session.transcript, &before);
        window = response["output"].as_array().unwrap().clone();
    }
    assert_eq!(assert_markers_precede_compactions(&session.transcript), 2);
    Ok(())
}
