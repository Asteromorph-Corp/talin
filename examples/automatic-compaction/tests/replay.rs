//! Actual HTTP captures run through the same generated agents and session code as the live app.
use std::collections::BTreeMap;
use std::error::Error;
use std::path::Path;

use automatic_compaction::application;
use automatic_compaction::config::Config;
use automatic_compaction::facts::Recall;
use automatic_compaction::report::{Report, StoredEntry, TranscriptState};
use automatic_compaction::scenario::{Outcome, RunError};
use mockito::{Mock, Server};
use talin::agent::AgentError;
use talin::llm::claude_messages::MessagesOptions;
use talin::llm::claude_messages::types::TextBlockParam;
use talin::llm::openai_responses::ResponsesOptions;
use talin::llm::openai_responses::types::{
    CreateResponse, InputItem, InputParam, Item, OutputItem, Response, Tool,
};
use talin::llm::{ClaudeMessages, Llm, OpenAIResponses};
use talin::schemas::BodyMessage;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::value::{RawValue, to_raw_value};

type TestResult = Result<(), Box<dyn Error>>;

/// The SDK response only deserializes; keep captured model content and metadata opaque.
#[derive(Serialize, Deserialize)]
struct ClaudeWireResponse {
    content: Vec<Box<RawValue>>,
    #[serde(flatten)]
    fields: BTreeMap<String, serde_json::Value>,
}

#[derive(Deserialize)]
struct RecordedWorkload {
    input_facts: Recall,
    archive_rows: usize,
    threshold: u32,
}

#[derive(Deserialize)]
struct Recording<R> {
    workload: RecordedWorkload,
    exchanges: Vec<Exchange<R>>,
}

#[derive(Deserialize)]
struct Exchange<R> {
    request: R,
    status: u16,
    content_type: String,
    body: String,
}

/// Claude request parameter types only implement Serialize. Keep their wire content opaque
/// here rather than inventing duplicate SDK types; the real driver constructs these requests.
#[derive(Deserialize)]
struct ClaudeRequest {
    model: String,
    max_tokens: u32,
    stream: Option<bool>,
    messages: Vec<Box<RawValue>>,
    #[serde(default)]
    tools: Vec<Box<RawValue>>,
    context_management: Option<Box<RawValue>>,
}

#[derive(Serialize)]
struct OpenAIHistory<'a> {
    input: &'a InputParam,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<&'a [Tool]>,
}

#[derive(Serialize)]
struct ClaudeHistory<'a> {
    messages: Vec<ClaudeMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<&'a [Box<RawValue>]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    context_management: Option<&'a RawValue>,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
enum Role {
    User,
    Assistant,
}

#[derive(Serialize, Deserialize)]
struct ClaudeMessage {
    role: Role,
    content: ClaudeContent,
}

#[derive(Serialize, Deserialize)]
#[serde(untagged, try_from = "Box<RawValue>")]
enum ClaudeContent {
    Text(String),
    Blocks(Vec<Box<RawValue>>),
}

impl TryFrom<Box<RawValue>> for ClaudeContent {
    type Error = serde_json::Error;

    fn try_from(raw: Box<RawValue>) -> Result<Self, Self::Error> {
        if raw.get().starts_with('"') {
            serde_json::from_str(raw.get()).map(Self::Text)
        } else {
            serde_json::from_str(raw.get()).map(Self::Blocks)
        }
    }
}

trait CapturedRequest: DeserializeOwned {
    const PATH: &'static str;
    fn history(&self, include_configuration: bool) -> Result<String, serde_json::Error>;
    fn record_count(&self) -> usize;
}

impl CapturedRequest for CreateResponse {
    const PATH: &'static str = "/v1/responses";
    fn history(&self, include_configuration: bool) -> Result<String, serde_json::Error> {
        serde_json::to_string(&OpenAIHistory {
            input: &self.input,
            tools: if include_configuration {
                self.tools.as_deref()
            } else {
                None
            },
        })
    }
    fn record_count(&self) -> usize {
        match &self.input {
            InputParam::Items(items) => items.len(),
            InputParam::Text(_) => 1,
        }
    }
}

impl CapturedRequest for ClaudeRequest {
    const PATH: &'static str = "/v1/messages";
    fn history(&self, include_configuration: bool) -> Result<String, serde_json::Error> {
        // The API combines consecutive messages of the same role. Compare their block sequences.
        let mut messages = Vec::<ClaudeMessage>::new();
        for raw in &self.messages {
            let mut message: ClaudeMessage = serde_json::from_str(raw.get())?;
            let blocks = match message.content {
                ClaudeContent::Text(text) => {
                    vec![to_raw_value(&TextBlockParam::new(text))?]
                }
                ClaudeContent::Blocks(blocks) => blocks,
            };
            if let Some(last) = messages.last_mut()
                && matches!(
                    (last.role, message.role),
                    (Role::User, Role::User) | (Role::Assistant, Role::Assistant)
                )
            {
                let ClaudeContent::Blocks(content) = &mut last.content else {
                    unreachable!("comparison history contains block arrays");
                };
                content.extend(blocks);
            } else {
                message.content = ClaudeContent::Blocks(blocks);
                messages.push(message);
            }
        }
        serde_json::to_string(&ClaudeHistory {
            messages,
            tools: include_configuration.then_some(self.tools.as_slice()),
            context_management: self
                .context_management
                .as_deref()
                .filter(|_| include_configuration),
        })
    }
    fn record_count(&self) -> usize {
        self.messages
            .iter()
            .map(|raw| {
                let message: ClaudeMessage =
                    serde_json::from_str(raw.get()).expect("captured request message");
                match message.content {
                    ClaudeContent::Text(_) => 1,
                    ClaudeContent::Blocks(blocks) => blocks.len(),
                }
            })
            .sum()
    }
}

fn load<R: DeserializeOwned>(case: &str) -> Result<Recording<R>, Box<dyn Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../crates/talin/tests/fixtures/auto-compaction")
        .join(format!("{case}.json"));
    Ok(serde_json::from_slice(&std::fs::read(path)?)?)
}

async fn serve<R: CapturedRequest + 'static>(
    server: &mut Server,
    recording: &Recording<R>,
    extra_turns: usize,
    include_configuration: bool,
) -> Result<Vec<Mock>, Box<dyn Error>> {
    let mut mocks = Vec::new();
    for exchange in &recording.exchanges {
        // Compare the independently recorded conversation. Claude may split assistant envelopes,
        // but each block and its position must match. New captures also compare tool schemas
        // and compaction settings.
        let expected: serde_json::Value =
            serde_json::from_str(&exchange.request.history(include_configuration)?)?;
        let mock = server
            .mock("POST", R::PATH)
            .match_request(move |request| {
                serde_json::from_slice::<R>(request.body().unwrap())
                    .and_then(|request| request.history(include_configuration))
                    .and_then(|history| serde_json::from_str::<serde_json::Value>(&history))
                    .is_ok_and(|actual| actual == expected)
            })
            .with_status(exchange.status.into())
            .with_header("content-type", &exchange.content_type)
            .with_body(&exchange.body)
            .expect(1)
            .create_async()
            .await;
        mocks.push(mock);
    }
    if extra_turns > 0 {
        let last = recording.exchanges.last().unwrap();
        let previous_count = last.request.record_count();
        // Reuse the real final response for additional turns. The transcript must grow through
        // normal recording, while its existing entries and native records remain unchanged.
        let mock = server
            .mock("POST", R::PATH)
            .match_request(move |request| {
                serde_json::from_slice::<R>(request.body().unwrap())
                    .is_ok_and(|request| request.record_count() > previous_count)
            })
            .with_status(last.status.into())
            .with_header("content-type", &last.content_type)
            .with_body(&last.body)
            .expect(extra_turns)
            .create_async()
            .await;
        mocks.push(mock);
    }
    Ok(mocks)
}

async fn replay_openai(case: &str, followups: usize) -> Result<Outcome, Box<dyn Error>> {
    let recording = load::<CreateResponse>(case)?;
    let mut server = Server::new_async().await;
    let mocks = serve(
        &mut server,
        &recording,
        followups - 1,
        case.starts_with("typed-"),
    )
    .await?;
    let first = &recording.exchanges[0].request;
    let mut options =
        ResponsesOptions::default().with_max_output_tokens(first.max_output_tokens.unwrap());
    if let Some(reasoning) = &first.reasoning {
        options = options.with_reasoning(reasoning.clone());
    }
    let llm = OpenAIResponses::builder()
        .with_api_base(format!("{}/v1", server.url()))
        .with_api_key("replay-only")
        .with_model(first.model.as_ref().unwrap())
        .with_default_options(options)
        .build();
    let mut config = Config::load()?;
    config.openai.compaction.threshold = recording.workload.threshold;
    config.openai.archive_rows = recording.workload.archive_rows;
    let outcome = application::openai(llm, config, recording.workload.input_facts, followups).await;
    for mock in mocks {
        mock.assert_async().await;
    }
    Ok(outcome)
}

async fn replay_claude(case: &str, followups: usize) -> Result<Outcome, Box<dyn Error>> {
    let recording = load::<ClaudeRequest>(case)?;
    let mut server = Server::new_async().await;
    let mocks = serve(
        &mut server,
        &recording,
        followups - 1,
        case.starts_with("typed-"),
    )
    .await?;
    let outcome = run_claude(&server, recording, followups).await?;
    for mock in mocks {
        mock.assert_async().await;
    }
    Ok(outcome)
}

async fn run_claude(
    server: &Server,
    recording: Recording<ClaudeRequest>,
    followups: usize,
) -> Result<Outcome, Box<dyn Error>> {
    let first = &recording.exchanges[0].request;
    let llm = ClaudeMessages::builder()
        .with_api_base(format!("{}/v1", server.url()))
        .with_api_key("replay-only")
        .with_model(&first.model)
        .with_anthropic_beta("compact-2026-01-12")
        .with_default_options(
            MessagesOptions::default()
                .with_max_tokens(first.max_tokens)
                .with_stream(first.stream.unwrap_or(false)),
        )
        .build();
    let mut config = Config::load()?;
    config.claude.compaction.threshold = recording.workload.threshold;
    config.claude.archive_rows = recording.workload.archive_rows;
    let outcome = application::claude(llm, config, recording.workload.input_facts, followups).await;
    Ok(outcome)
}

#[tokio::test]
async fn claude_compaction_and_recall_match_the_live_conversation() -> TestResult {
    let outcome = replay_claude("typed-claude-no-pause", 1).await?;
    outcome.result?;
    Ok(())
}

#[tokio::test]
async fn claude_streamed_tool_compaction_and_recall_match_the_live_conversation() -> TestResult {
    let outcome = replay_claude("typed-claude-stream-no-pause", 1).await?;
    outcome.result?;
    Ok(())
}

#[tokio::test]
async fn pending_step_keeps_compaction_native() -> TestResult {
    use talin::agent::Agent;
    use talin::llm::claude_messages::types::{ContentBlock, MessagesResponse};
    use talin::schemas::{ContentPart, LlmStep};

    let recording = load::<ClaudeRequest>("typed-claude-no-pause")?;
    let exchange = &recording.exchanges[1];
    let response: MessagesResponse = serde_json::from_str(&exchange.body)?;
    let mut server = Server::new_async().await;
    let mock = server
        .mock("POST", "/v1/messages")
        .with_status(exchange.status.into())
        .with_header("content-type", &exchange.content_type)
        .with_body(&exchange.body)
        .expect(1)
        .create_async()
        .await;
    let llm = ClaudeMessages::builder()
        .with_api_base(format!("{}/v1", server.url()))
        .with_api_key("replay-only")
        .with_model(&exchange.request.model)
        .build();
    let agent = Agent::<_>::builder().build(llm);
    let mut session = agent.session();
    let mut runner = session.turn((), "Replay the captured response.").await?;
    let pending = runner.half_step().await?;
    let mut text = Vec::new();
    for block in response.content {
        match block {
            ContentBlock::Compaction(_) => {}
            ContentBlock::Thinking(_) => {}
            ContentBlock::Text(block) => text.push(ContentPart::Text(block.text)),
            block => panic!("unexpected block in the captured response: {block:?}"),
        }
    }
    let LlmStep::Finish(message) = &pending.step else {
        panic!("the captured response ends with an answer")
    };
    assert_eq!(message.content, text);
    assert_eq!(
        pending.runner.session.transcript.get_llm_step(),
        Some(pending.step.clone())
    );
    let before = serde_json::to_value(pending.runner.session.transcript.clone().materialize().1)?;
    pending
        .runner
        .session
        .transcript
        .modify_step(pending.step.clone())?;
    let after = serde_json::to_value(pending.runner.session.transcript.clone().materialize().1)?;
    assert_eq!(
        before, after,
        "an unchanged final message must retain the entire native response"
    );
    assert!(pending.complete().await?.is_some());
    mock.assert_async().await;
    Ok(())
}

#[tokio::test]
async fn parallel_client_calls_share_the_complete_native_response() -> TestResult {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use automatic_compaction::tools::LoadArchiveTool;
    use talin::agent::Agent;
    use talin::llm::claude_messages::types::{
        ContentBlock, ContentBlockParam, TextOrItems,
    };
    use talin::schemas::LlmStep;
    use talin::tools::Tool;
    use talin::transcript::Entry;

    let recording = load::<ClaudeRequest>("typed-claude-no-pause")?;
    let mut response: ClaudeWireResponse = serde_json::from_str(&recording.exchanges[0].body)?;
    let context: ClaudeWireResponse = serde_json::from_str(&recording.exchanges[1].body)?;
    let mut call: ContentBlock = serde_json::from_str(response.content[0].get())?;
    let ContentBlock::ToolUse(second_call) = &mut call else {
        panic!("expected captured client call")
    };
    second_call.id = uuid::Uuid::new_v4().to_string();
    response.content = vec![
        context.content[0].clone(),
        context.content[1].clone(),
        response.content[0].clone(),
        to_raw_value(&ContentBlockParam::from(call))?,
    ];
    let mut server = Server::new_async().await;
    let mock = server
        .mock("POST", "/v1/messages")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(serde_json::to_string(&response)?)
        .expect(1)
        .create_async()
        .await;
    let llm = ClaudeMessages::builder()
        .with_api_base(format!("{}/v1", server.url()))
        .with_api_key("replay-only")
        .with_model(&recording.exchanges[0].request.model)
        .build();
    let config = Config::load()?;
    let calls = Arc::new(AtomicUsize::new(0));
    let tool = LoadArchiveTool {
        rows: 1,
        revision: recording.workload.input_facts.archive_revision,
        text: config.archive,
        calls: calls.clone(),
    };
    let agent = Agent::<_>::builder().build(llm);
    let mut session = agent.session();
    session.tools.extend(Tool::build_index(vec![tool.into()]));
    let mut runner = session.turn((), "Replay the captured response.").await?;
    let pending = runner.half_step().await?;
    let LlmStep::ToolCall(batch) = &pending.step else {
        panic!("expected client calls")
    };
    assert_eq!(batch.tool_calls.len(), 2);
    assert!(batch.content.is_empty());
    assert!(pending.complete().await?.is_none());
    assert_eq!(calls.load(Ordering::SeqCst), 2);
    drop(runner);
    let record = session
        .transcript
        .entries()
        .iter()
        .find_map(|entry| match entry {
            Entry::Message(_, record) if matches!(record.canonical(), BodyMessage::ToolCall(_)) => {
                Some(record)
            }
            _ => None,
        })
        .expect("client-call record must remain inspectable");
    let Some(TextOrItems::Items(blocks)) = record.native().content() else {
        panic!("expected native blocks");
    };
    assert!(matches!(
        blocks.as_slice(),
        [
            ContentBlockParam::Compaction(_),
            ContentBlockParam::Thinking(_),
            ContentBlockParam::ToolUse(_),
            ContentBlockParam::ToolUse(_)
        ]
    ));
    mock.assert_async().await;
    Ok(())
}

#[tokio::test]
async fn fresh_turns_preserve_native_history_and_compaction_markers() -> TestResult {
    let outcome = replay_claude("typed-claude-no-pause", 3).await?;
    outcome.result?;
    assert_history_preserved(&outcome.report);
    Ok(())
}

#[tokio::test]
async fn openai_native_history_stays_unchanged_even_when_the_summary_loses_a_fact() -> TestResult {
    let outcome = replay_openai("openai-nano-loss", 3).await?;
    assert!(matches!(
        outcome.result,
        Err(RunError::RecallMismatch { .. })
    ));
    assert_history_preserved(&outcome.report);
    Ok(())
}

/// Checks, with the backend's own `contains_native_compaction`, that a marker directly precedes
/// every compacting record and that every marker announces a compacting record or a native summary.
fn assert_markers_precede_compactions(state: &TranscriptState) -> usize {
    let mut compacting = state.compacting.iter();
    let mut announced = false;
    let mut markers = 0;
    for entry in &state.entries {
        match entry {
            StoredEntry::CompactionMarker => {
                assert!(!announced, "two compaction markers in a row");
                announced = true;
                markers += 1;
            }
            StoredEntry::StepMarker | StoredEntry::TurnMarker => {
                assert!(!announced, "a marker is not directly before its compaction");
            }
            StoredEntry::Message { .. }
            | StoredEntry::ToolError { .. }
            | StoredEntry::Native { .. } => {
                let compacting = *compacting.next().expect("one flag per native record");
                if announced {
                    assert!(
                        compacting || matches!(entry, StoredEntry::Native { .. }),
                        "a compaction marker announces no compaction"
                    );
                } else {
                    assert!(!compacting, "a compacting record has no preceding marker");
                }
                announced = false;
            }
        }
    }
    markers
}

fn assert_history_preserved(report: &Report) {
    // A fresh strategy clone is used by run_turn each time. No test invokes a strategy hook.
    assert_eq!(
        report.recalls.len(),
        3,
        "all three follow-ups must go through the runner"
    );
    for pair in report.turns[1..].windows(2) {
        let before = &pair[0];
        let after = &pair[1];
        assert!(
            assert_markers_precede_compactions(before) > 0,
            "the captured compaction has no transcript marker"
        );
        assert_eq!(
            after.entries[..before.entries.len()],
            before.entries,
            "a fresh turn changed existing entries, UUIDs or markers"
        );
        assert_eq!(
            after.native_records[..before.native_records.len()],
            before.native_records,
            "a fresh turn rebuilt existing provider records"
        );
        assert_eq!(
            after.system, before.system,
            "a fresh turn changed the system prompt"
        );
        assert!(
            !after.entries[before.entries.len()..].contains(&StoredEntry::CompactionMarker),
            "an already covered compaction inserted a duplicate marker"
        );
    }
}

#[tokio::test]
async fn openai_revision_loss_is_reported_as_a_recall_mismatch() -> TestResult {
    let outcome = replay_openai("openai-nano-loss", 1).await?;
    assert_openai_compaction_record(&outcome.report, "openai-nano-loss")?;
    let RunError::RecallMismatch { expected, actual } = outcome
        .result
        .expect_err("the recorded API lost the revision")
    else {
        panic!("the replay failed for an unrelated reason");
    };
    assert_ne!(actual.archive_revision, expected.archive_revision);
    assert_eq!(
        actual.project(),
        expected.project(),
        "the other user facts survived this capture"
    );
    Ok(())
}

#[tokio::test]
async fn openai_repeated_archive_call_reaches_the_real_tool_error_path() -> TestResult {
    let outcome = replay_openai("openai", 1).await?;
    assert_openai_compaction_record(&outcome.report, "openai")?;
    assert_eq!(
        outcome.report.archive_calls, 2,
        "both calls must reach the actual tool"
    );
    assert!(
        matches!(
            outcome.result,
            Err(RunError::Agent(AgentError::AllToolsFailed))
        ),
        "the captured second read must fail in the actual tool runner"
    );
    Ok(())
}

#[tokio::test]
async fn generated_empty_tool_matches_the_api_accepted_strict_schema() -> TestResult {
    // This live capture uses the generated tool, unlike the older non-strict captures.
    // Replay matches its tool definitions too: omitting properties/required breaks request 1.
    let outcome = replay_openai("typed-openai", 1).await?;
    assert_eq!(outcome.report.archive_calls, 2);
    assert!(matches!(
        outcome.result,
        Err(RunError::Agent(AgentError::AllToolsFailed))
    ));
    Ok(())
}

fn assert_openai_compaction_record(report: &Report, case: &str) -> TestResult {
    let recording = load::<CreateResponse>(case)?;
    let response = serde_json::from_str::<Response>(&recording.exchanges[1].body)?;
    let expected = response
        .output
        .iter()
        .find_map(|item| match item {
            OutputItem::Compaction(item) => Some(item),
            _ => None,
        })
        .expect("the captured response contains compaction");
    let transcript = &report.turns[0];
    let native = transcript
        .native_records
        .iter()
        .map(|record| serde_json::from_str::<Vec<InputItem>>(record))
        .collect::<Result<Vec<_>, _>>()?;
    let record = native
        .iter()
        .find(|record| {
            record.iter().any(|item| {
                matches!(item,
        InputItem::Item(Item::Compaction(item)) if item.id.as_deref() == Some(expected.id.as_str()))
            })
        })
        .expect("the native compaction must remain stored");
    let expected_native = response
        .output
        .into_iter()
        .map(Into::<InputItem>::into)
        .collect::<Vec<_>>();
    assert_eq!(
        serde_json::to_value(record)?,
        serde_json::to_value(expected_native)?
    );
    Ok(())
}

#[test]
fn claude_server_tools_remain_native_without_changing_the_canonical_step() -> TestResult {
    use talin::llm::claude_messages::types::{
        ContentBlock, ContentBlockParam, MessagesResponse, ServerToolName, ServerToolUseBlock,
        TextOrItems, WebSearchToolResultBlock, WebSearchToolResultBlockContent,
    };

    let recording = load::<ClaudeRequest>("typed-claude-no-pause")?;
    let client = ClaudeMessages::builder()
        .with_api_key("replay-only")
        .with_model(&recording.exchanges[0].request.model)
        .build();
    let call_id = "srvtoolu_search";
    for index in [0, 2] {
        let mut response: MessagesResponse =
            serde_json::from_str(&recording.exchanges[index].body)?;
        let expected = client.parse_response(response.clone())?;
        let mut content = vec![
            ContentBlock::ServerToolUse(ServerToolUseBlock::new(
                call_id,
                ServerToolName::WebSearch,
                serde_json::json!({"query": "archive"}),
            )),
            ContentBlock::WebSearchToolResult(WebSearchToolResultBlock::new(
                call_id,
                WebSearchToolResultBlockContent::Results(Vec::new()),
            )),
        ];
        content.extend(response.content);
        response.content = content;
        let actual = client.parse_response(response)?;
        assert_eq!(actual.canonical(), expected.canonical());
        let Some(TextOrItems::Items(items)) = actual.native().content() else {
            panic!("expected native content blocks")
        };
        assert!(matches!(&items[0], ContentBlockParam::ServerToolUse(call) if call.id == call_id));
        assert!(
            matches!(&items[1], ContentBlockParam::WebSearchToolResult(result)
            if result.tool_use_id == call_id)
        );
    }
    Ok(())
}

/// Exercise the generated application's tool round-trip with multiple signed thoughts and a
/// server-tool loop. Model answers and client tool calls come from the captured HTTP exchange.
#[tokio::test]
async fn claude_native_response_preserves_thought_strings_signatures_and_order() -> TestResult {
    use talin::llm::claude_messages::types::{
        ContentBlock, ContentBlockParam, ServerToolName, ServerToolUseBlockParam,
        ThinkingBlockParam, WebSearchToolResultBlockParam, WebSearchToolResultBlockParamContent,
    };

    // The SDK response types only deserialize. Keep unrelated captured fields opaque.
    #[derive(Serialize, Deserialize)]
    struct WireResponse {
        content: Vec<Box<RawValue>>,
        #[serde(flatten)]
        fields: BTreeMap<String, serde_json::Value>,
    }
    #[derive(Serialize)]
    struct AssistantTurn<'a> {
        role: &'static str,
        content: &'a [Box<RawValue>],
    }
    #[derive(Deserialize)]
    struct NativeTurn {
        content: Vec<Box<RawValue>>,
    }

    let mut recording = load::<ClaudeRequest>("typed-claude-no-pause")?;
    let mut response: WireResponse = serde_json::from_str(&recording.exchanges[0].body)?;
    let thoughts = [
        ThinkingBlockParam::new(" first\n생각 ", "signature-first"),
        ThinkingBlockParam::new("", "signature-empty"),
        ThinkingBlockParam::new("second\nthought", "signature-second"),
    ];
    let compacted: WireResponse = serde_json::from_str(&recording.exchanges[1].body)?;
    let compaction: ContentBlock = serde_json::from_str(compacted.content[0].get())?;
    assert!(matches!(compaction, ContentBlock::Compaction(_)));
    let prefix = [
        ContentBlockParam::Thinking(thoughts[0].clone()),
        ContentBlockParam::ServerToolUse(ServerToolUseBlockParam::new(
            "srvtoolu_search",
            ServerToolName::WebSearch,
            serde_json::json!({"query": "archive"}),
        )),
        ContentBlockParam::WebSearchToolResult(WebSearchToolResultBlockParam::new(
            "srvtoolu_search",
            WebSearchToolResultBlockParamContent::Results(Vec::new()),
        )),
        compaction.into(),
        ContentBlockParam::Thinking(thoughts[1].clone()),
        ContentBlockParam::Thinking(thoughts[2].clone()),
    ];
    let mut content = prefix
        .iter()
        .map(to_raw_value)
        .collect::<Result<Vec<_>, _>>()?;
    content.extend(response.content);
    response.content = content;
    recording.exchanges[0].body = serde_json::to_string(&response)?;
    let expected_native = serde_json::to_value(&response.content)?;

    let assistant = recording.exchanges[1].request.messages.len() - 2;
    recording.exchanges[1].request.messages.drain(..assistant);
    let mut messages = Vec::new();
    recording.exchanges[1].request.messages.remove(0);
    messages.push(to_raw_value(&AssistantTurn {
        role: "assistant",
        content: &response.content[3..],
    })?);
    messages.append(&mut recording.exchanges[1].request.messages);
    recording.exchanges[1].request.messages = messages;

    let mut server = Server::new_async().await;
    let mocks = serve(&mut server, &recording, 0, false).await?;
    let outcome = run_claude(&server, recording, 1).await?;
    for mock in mocks {
        mock.assert_async().await;
    }
    outcome.result?;
    let transcript = &outcome.report.turns[0];
    let turn: NativeTurn = serde_json::from_str(&transcript.native_records[1])?;
    assert_eq!(serde_json::to_value(turn.content)?, expected_native);
    Ok(())
}
