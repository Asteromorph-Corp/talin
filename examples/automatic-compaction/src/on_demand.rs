//! On-demand compaction policies used through generated agents and normal session continuation.
use std::error::Error;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use talin::agent::Agent;
use talin::chain::StringCtor;
use talin::llm::Llm;
use talin::schemas::Usage;
use talin::strategy::{
    ClaudeMessagesCompactStrategy, CompactionStrategy, InstructionCompactor, MessageCount,
    OpenAIResponsesCompactStrategy, Strategy, UsageTrackStrategy,
};
use talin::tools::Tool;
use talin::transcript::Entry;
use serde::{Deserialize, Serialize};

use crate::agents::{
    RecallAgentInput, RecallAgentInputCtor, RememberAgentInput, RememberAgentInputCtor,
};
use crate::config::Config;
use crate::facts::{Recall, RecallCtor};
use crate::report::TranscriptState;
use crate::tools::LoadArchiveTool;

#[derive(Debug, Clone, Deserialize)]
pub struct OnDemandConfig {
    pub archive_rows: usize,
    pub claude: ClaudeMessagesCompactStrategy,
    pub openai: OpenAIResponsesCompactStrategy,
    pub custom: InstructionCompactor,
    pub message_count: MessageCount,
}

impl OnDemandConfig {
    pub fn load() -> Result<Self, toml::de::Error> {
        toml::from_str(include_str!("../configs/on-demand.toml"))
    }
}

#[derive(Debug, Serialize)]
pub struct OnDemandReport {
    pub input_facts: Recall,
    pub recalls: Vec<Recall>,
    pub archive_calls: usize,
    pub before: TranscriptState,
    pub after: TranscriptState,
    pub usage: Usage,
}

pub async fn run<L, S, C>(
    remember: Agent<L, RememberAgentInputCtor, StringCtor>,
    recall: Agent<L, RecallAgentInputCtor, RecallCtor, S>,
    compaction: C,
    config: &Config,
    workload: &OnDemandConfig,
    facts: Recall,
) -> Result<OnDemandReport, Box<dyn Error>>
where
    L: Llm,
    L::System: Serialize,
    L::Item: Serialize,
    S: Strategy<L, RecallCtor> + Clone,
    C: CompactionStrategy<L> + 'static,
{
    let calls = Arc::new(AtomicUsize::new(0));
    let tools = Tool::build_index(vec![
        LoadArchiveTool {
            rows: workload.archive_rows,
            revision: facts.archive_revision.clone(),
            text: config.archive.clone(),
            calls: calls.clone(),
        }
        .into(),
    ]);
    let mut session = remember.session();
    session.tools.extend(tools.clone());
    let project = serde_json::to_string(&facts.project())?;
    let ready = session.run_turn(RememberAgentInput::new(&project)).await?;
    if ready.trim() != "READY" {
        return Err(format!("expected READY, received {ready:?}").into());
    }
    let before = TranscriptState::observe(&session.transcript)?;
    let originals = session
        .transcript
        .entries()
        .iter()
        .filter_map(|entry| match entry {
            Entry::Message(id, record) => {
                Some(serde_json::to_string(record.native()).map(|native| (*id, native)))
            }
            Entry::ToolError(id, record) => {
                Some(serde_json::to_string(record.native()).map(|native| (*id, native)))
            }
            _ => None,
        })
        .collect::<Result<Vec<_>, serde_json::Error>>()?;
    let mut session = recall.continue_session(session).with_compaction(compaction);
    session.tools.extend(tools);
    let mut recalls = Vec::new();
    let mut usage = Usage::default();
    for _ in 0..2 {
        let result = session
            .run_turn_with_strategy(UsageTrackStrategy::default(), RecallAgentInput::new())
            .await?
            .into_with_usage();
        usage.merge_inplace(&result.usage);
        if result.content != facts {
            return Err(format!(
                "recall mismatch: expected {facts:?}, received {:?}",
                result.content
            )
            .into());
        }
        recalls.push(result.content);
    }
    for (id, native) in originals {
        let current = session
            .transcript
            .entries()
            .iter()
            .find_map(|entry| match entry {
                Entry::Message(current, record) if *current == id => {
                    Some(serde_json::to_string(record.native()))
                }
                Entry::ToolError(current, record) if *current == id => {
                    Some(serde_json::to_string(record.native()))
                }
                _ => None,
            })
            .transpose()?;
        if current.as_ref() != Some(&native) {
            return Err(format!("original transcript entry {id} changed").into());
        }
    }
    let after = TranscriptState::observe(&session.transcript)?;
    let entries = session.transcript.entries();
    if !entries
        .iter()
        .any(|entry| matches!(entry, Entry::CompactionMarker { .. }))
        || !entries
            .iter()
            .any(|entry| matches!(entry, Entry::Native(..)))
    {
        return Err("on-demand policy did not record a native compaction".into());
    }
    let archive_calls = calls.load(Ordering::SeqCst);
    if archive_calls != 1 {
        return Err(format!("expected one archive call, observed {archive_calls}").into());
    }
    Ok(OnDemandReport {
        input_facts: facts,
        recalls,
        archive_calls,
        before,
        after,
        usage,
    })
}
