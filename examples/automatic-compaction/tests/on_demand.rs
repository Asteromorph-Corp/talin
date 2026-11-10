//! Replay real count, compaction, and generation HTTP exchanges through the live application.
use std::error::Error;
use std::path::Path;

use automatic_compaction::agents::{RecallAgent, RememberAgent};
use automatic_compaction::config::Config;
use automatic_compaction::facts::Recall;
use automatic_compaction::on_demand::{self, OnDemandConfig, OnDemandReport};
use mockito::{Matcher, Mock, Server};
use talin::llm::claude_messages::MessagesOptions;
use talin::llm::openai_responses::ResponsesOptions;
use talin::llm::openai_responses::types::CreateResponse;
use talin::llm::{ClaudeMessages, LlmPreset, OpenAIResponses};
use talin::strategy::CustomCompactStrategy;
use serde::Deserialize;
use serde_json::value::RawValue;

type TestResult = Result<(), Box<dyn Error>>;

#[derive(Deserialize)]
struct Capture {
    report: RecordedReport,
    exchanges: Vec<Exchange>,
}

#[derive(Deserialize)]
struct RecordedReport {
    input_facts: Recall,
    before: RecordedNativeHistory,
    after: RecordedNativeHistory,
}

#[derive(Deserialize)]
struct RecordedNativeHistory {
    native_records: Vec<String>,
}

#[derive(Deserialize)]
struct Exchange {
    path: String,
    request: Box<RawValue>,
    status: u16,
    content_type: String,
    body: String,
}

// Native Claude request parameters implement Serialize only; these are the replay client's options.
#[derive(Deserialize)]
struct ClaudeOptions {
    model: String,
    max_tokens: u32,
    stream: Option<bool>,
}

impl Capture {
    fn load(case: &str) -> Result<Self, Box<dyn Error>> {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../crates/talin/tests/fixtures/on-demand-compaction");
        Ok(serde_json::from_str(&std::fs::read_to_string(
            root.join(format!("{case}.json")),
        )?)?)
    }

    async fn replay(&self, server: &mut mockito::ServerGuard) -> Vec<Mock> {
        let mut mocks = Vec::new();
        for exchange in &self.exchanges {
            mocks.push(
                server
                    .mock("POST", exchange.path.as_str())
                    .match_body(Matcher::JsonString(exchange.request.get().into()))
                    .with_status(exchange.status.into())
                    .with_header("content-type", &exchange.content_type)
                    .with_body(&exchange.body)
                    .expect(1)
                    .create_async()
                    .await,
            );
        }
        mocks
    }

    fn assert_native_history(&self, report: &OnDemandReport) {
        assert_eq!(
            report.before.native_records, self.report.before.native_records,
            "normal tool execution differs from the recorded live history"
        );
        assert_eq!(
            report.after.native_records, self.report.after.native_records,
            "compaction changed a native payload, its position, or the retained suffix"
        );
    }
}

async fn replay_claude(case: &str, custom: bool) -> TestResult {
    let capture = Capture::load(case)?;
    let options: ClaudeOptions = serde_json::from_str(capture.exchanges[0].request.get())?;
    let mut server = Server::new_async().await;
    let mocks = capture.replay(&mut server).await;
    let config = Config::load()?;
    let workload = OnDemandConfig::load()?;
    let llm = ClaudeMessages::builder()
        .with_model(options.model)
        .with_api_key("replay-key")
        .with_api_base(format!("{}/v1", server.url()))
        .with_anthropic_betas(vec!["compact-2026-09-04".into()])
        .with_default_options(
            MessagesOptions::default()
                .with_max_tokens(options.max_tokens)
                .with_stream(options.stream.unwrap_or(false)),
        )
        .build();
    let remember = RememberAgent::new(LlmPreset::new(llm.clone(), ())).await;
    let report = if custom {
        let strategy =
            CustomCompactStrategy::new(workload.custom.clone(), workload.message_count.clone());
        let recall = RecallAgent::new(LlmPreset::new(llm, ())).await;
        on_demand::run(
            remember,
            recall,
            strategy,
            &config,
            &workload,
            capture.report.input_facts.clone(),
        )
        .await?
    } else {
        let recall = RecallAgent::new(LlmPreset::new(llm, ())).await;
        on_demand::run(
            remember,
            recall,
            workload.claude.clone(),
            &config,
            &workload,
            capture.report.input_facts.clone(),
        )
        .await?
    };
    for mock in mocks {
        mock.assert_async().await;
    }
    capture.assert_native_history(&report);
    Ok(())
}

#[tokio::test]
async fn signed_compaction_reuses_the_live_summary_and_keeps_the_trailing_messages() -> TestResult {
    replay_claude("claude", false).await
}

#[tokio::test]
async fn streamed_signed_compaction_preserves_the_same_native_contract() -> TestResult {
    replay_claude("claude-stream", false).await
}

#[tokio::test]
async fn application_defined_compaction_uses_the_fixed_message_selector() -> TestResult {
    replay_claude("custom", true).await
}

#[tokio::test]
async fn responses_compaction_preserves_retained_user_messages_and_the_opaque_item() -> TestResult {
    let capture = Capture::load("openai-sdk-inputs")?;
    let request: CreateResponse = serde_json::from_str(capture.exchanges[0].request.get())?;
    let mut server = Server::new_async().await;
    let mocks = capture.replay(&mut server).await;
    let config = Config::load()?;
    let workload = OnDemandConfig::load()?;
    let mut options = ResponsesOptions::default();
    if let Some(max) = request.max_output_tokens {
        options = options.with_max_output_tokens(max);
    }
    if let Some(reasoning) = request.reasoning {
        options = options.with_reasoning(reasoning);
    }
    let llm = OpenAIResponses::builder()
        .with_model(request.model.unwrap())
        .with_api_key("replay-key")
        .with_api_base(format!("{}/v1", server.url()))
        .with_default_options(options)
        .build();
    let remember = RememberAgent::new(LlmPreset::new(llm.clone(), ())).await;
    let recall = RecallAgent::new(LlmPreset::new(llm, ())).await;
    let report = on_demand::run(
        remember,
        recall,
        workload.openai.clone(),
        &config,
        &workload,
        capture.report.input_facts.clone(),
    )
    .await?;
    for mock in mocks {
        mock.assert_async().await;
    }
    capture.assert_native_history(&report);

    Ok(())
}
