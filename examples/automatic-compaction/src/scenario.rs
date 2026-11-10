use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use talin::agent::{Agent, AgentError};
use talin::chain::StringCtor;
use talin::strategy::Strategy;
use talin::tools::Tool;
use serde::Serialize;
use thiserror::Error;

use crate::agents::{
    RecallAgentInput, RecallAgentInputCtor, RememberAgentInput, RememberAgentInputCtor,
};
use crate::config::{ArchiveText, Workload};
use crate::evidence::Evidence;
use crate::facts::{Recall, RecallCtor};
use crate::report::{Report, TranscriptState};
use crate::tools::LoadArchiveTool;
use crate::trace::Trace;

#[derive(Debug, Error)]
pub enum RunError {
    #[error(transparent)]
    Agent(#[from] AgentError),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error("recall differs from the supplied data: expected {expected:?}, received {actual:?}")]
    RecallMismatch {
        expected: Box<Recall>,
        actual: Box<Recall>,
    },
    #[error("expected READY, received {0:?}")]
    InitialAnswer(String),
    #[error("expected one archive read, observed {0}")]
    ArchiveCalls(usize),
    #[error("the API returned no successful compaction")]
    NoCompaction,
    #[error("the archive never reached the API")]
    NoArchive,
    #[error("no request reused the native compaction without the old archive")]
    MissingReuse,
}

pub struct Outcome {
    pub report: Report,
    pub result: Result<(), RunError>,
}

pub fn trace<S>(
    model: String,
    workload: &Workload<S>,
    threshold: u32,
    facts: &Recall,
    archive: &ArchiveText,
) -> Trace {
    Trace {
        report: Arc::new(Mutex::new(Report {
            model,
            threshold,
            archive_rows: workload.archive_rows,
            archive_calls: 0,
            input_facts: facts.clone(),
            requests: Vec::new(),
            responses: Vec::new(),
            recalls: Vec::new(),
            turns: Vec::new(),
            passed: false,
            error: None,
        })),
        noise_marker: archive.noise_marker.clone(),
        request_budget: 8,
    }
}

/// Both turns go through generated agents, normal sessions, the tool runner and strategy hooks.
/// This function only observes the transcript. It never inserts records or boundaries.
pub async fn run<L, S>(
    remember: Agent<L, RememberAgentInputCtor, StringCtor, (S, Trace)>,
    recall: Agent<L, RecallAgentInputCtor, RecallCtor, (S, Trace)>,
    workload: Workload<S>,
    facts: Recall,
    archive: ArchiveText,
    trace: Trace,
    followups: usize,
) -> Outcome
where
    L: Evidence,
    L::Request: Serialize,
    L::System: Serialize,
    L::Item: Serialize,
    S: Strategy<L, StringCtor> + Strategy<L, RecallCtor> + Clone,
{
    let calls = Arc::new(AtomicUsize::new(0));
    let archive_tool = LoadArchiveTool {
        rows: workload.archive_rows,
        revision: facts.archive_revision.clone(),
        text: archive,
        calls: calls.clone(),
    };
    let mut session = remember.session();
    let tools = Tool::build_index(vec![archive_tool.into()]);
    session.tools.extend(tools.clone());
    let result = async {
        let project = serde_json::to_string(&facts.project())?;
        let ready = session.run_turn(RememberAgentInput::new(&project)).await;
        trace
            .report
            .lock()
            .unwrap()
            .turns
            .push(TranscriptState::observe(&session.transcript)?);
        let ready = ready?;
        if ready.trim() != "READY" {
            return Err(RunError::InitialAnswer(ready));
        }

        // continue_session preserves the normal conversation and adopts the recall agent's type.
        let mut session = recall.continue_session(session);
        session.tools.extend(tools);
        for _ in 0..followups {
            let actual = session.run_turn(RecallAgentInput::new()).await?;
            let state = TranscriptState::observe(&session.transcript)?;
            let mut report = trace.report.lock().unwrap();
            report.turns.push(state);
            report.recalls.push(actual.clone());
        }
        let count = calls.load(Ordering::SeqCst);
        if count != 1 {
            return Err(RunError::ArchiveCalls(count));
        }
        let report = trace.report.lock().unwrap();
        for actual in &report.recalls {
            if actual != &facts {
                return Err(RunError::RecallMismatch {
                    expected: facts.clone().into(),
                    actual: actual.clone().into(),
                });
            }
        }
        validate(&report)
    }
    .await;
    {
        let mut report = trace.report.lock().unwrap();
        report.archive_calls = calls.load(Ordering::SeqCst);
        report.passed = result.is_ok();
        report.error = result.as_ref().err().map(ToString::to_string);
    }
    drop(remember);
    drop(recall);
    let report = Arc::try_unwrap(trace.report).unwrap().into_inner().unwrap();
    Outcome { report, result }
}

fn validate(report: &Report) -> Result<(), RunError> {
    let summaries = report
        .responses
        .iter()
        .flat_map(|response| &response.compactions)
        .map(|block| block.wire())
        .collect::<Result<Vec<_>, _>>()?;
    if summaries.is_empty() {
        return Err(RunError::NoCompaction);
    }
    if !report
        .requests
        .iter()
        .any(|request| request.contains_archive_noise)
    {
        return Err(RunError::NoArchive);
    }
    for request in &report.requests {
        if request.contains_archive_noise {
            continue;
        }
        for compaction in &request.compactions {
            if summaries.contains(&compaction.wire()?) {
                return Ok(());
            }
        }
    }
    Err(RunError::MissingReuse)
}
