//! Running one turn of the conversation, asking for approval before some tools run.

use std::io::{self, Write};
use std::sync::{Arc, OnceLock};

use talin::agent::AgentError;
use talin::schemas::{LlmStep, ToolCall, ToolResultMessage, Usage};
use talin::strategy::SharedUsageTrackStrategy;
use talin::tools::ToolError;
use tokio::io::{AsyncBufReadExt, BufReader, Lines, Stdin};
use tokio::sync::Mutex;

use crate::agents::{AssistantAgentInput, AssistantSession};
use crate::tools::NEEDS_APPROVAL;

/// How many times in a row a failing step is tried again before the turn gives up.
const MAX_RETRIES: usize = 2;

/// Reads one line from the terminal, or `None` at the end of the input.
///
/// The read is asynchronous, so a turn that is waiting for an answer can still be cancelled with
/// Ctrl-C. A blocking `read_line` would hold up the task until Enter is pressed.
async fn prompt(label: &str) -> Option<String> {
    static LINES: OnceLock<Mutex<Lines<BufReader<Stdin>>>> = OnceLock::new();
    print!("{label}");
    std::io::stdout().flush().ok()?;
    let lines = LINES.get_or_init(|| Mutex::new(BufReader::new(tokio::io::stdin()).lines()));
    let line = lines.lock().await.next_line().await.ok()??;
    Some(line.trim().to_owned())
}

/// Reads the next line, or `None` when the input ends or Ctrl-C is pressed.
///
/// Once the program listens for Ctrl-C, the signal no longer ends it by default, so the idle
/// prompt has to listen as well.
pub async fn next_line(label: &str) -> Option<String> {
    tokio::select! {
        line = prompt(label) => line.filter(|line| !line.is_empty()),
        _ = tokio::signal::ctrl_c() => {
            println!();
            None
        }
    }
}

/// Asks whether the tool calls may run. Only the calls to tools in [`NEEDS_APPROVAL`] are asked
/// about.
async fn approved(calls: &[ToolCall]) -> bool {
    for call in calls
        .iter()
        .filter(|call| NEEDS_APPROVAL.contains(&call.name.as_str()))
    {
        let label = format!("  Allow `{}` {}? [y/N] ", call.name, call.arguments);
        if !prompt(&label)
            .await
            .is_some_and(|answer| answer.eq_ignore_ascii_case("y"))
        {
            return false;
        }
    }
    true
}

/// Runs a turn and returns the answer.
///
/// The usage of the turn is added to `total`, and returned for display.
pub async fn run(
    session: &mut AssistantSession,
    total: &Arc<Mutex<Usage>>,
    message: String,
) -> Result<(String, Usage), AgentError> {
    let strategy = SharedUsageTrackStrategy::new(total.clone());
    let input = AssistantAgentInput {
        message: message.into(),
    };
    let mut turn = session.turn(strategy, input).await?;

    // Step through the turn, so that tool calls can be inspected before they run.
    let mut fails = 0;
    let partial = loop {
        let result = async {
            let pending = turn.half_step().await?;
            let LlmStep::ToolCall(message) = pending.step.clone() else {
                return pending.complete().await;
            };
            if approved(&message.tool_calls).await {
                return pending.complete().await;
            }
            // Answer the calls ourselves. The model reads this as the tool's result.
            let declined = message
                .tool_calls
                .iter()
                .map(|call| ToolResultMessage {
                    call_id: call.id.clone(),
                    name: call.name.clone(),
                    result: Err(ToolError::execution(io::Error::other(
                        "the user declined this action",
                    ))),
                })
                .collect();
            pending.complete_with(declined);
            Ok(None)
        }
        .await;

        match result {
            Ok(Some(partial)) => break partial,
            Ok(None) => fails = 0,
            Err(e) if e.is_retryable() && fails < MAX_RETRIES => {
                fails += 1;
                eprintln!("  (retrying: {e})");
            }
            Err(e) => return Err(e),
        }
    };

    let output = turn.finalize(partial).await?;
    Ok((output.content, output.extra_content))
}
