mod agents;
mod tools;

use std::io::{self, Write};
use std::sync::OnceLock;

use agents::{AssistantAgent, AssistantAgentInput, Models};
use talin::agent::AgentError;
use talin::schemas::{LlmStep, ToolCall, ToolResultMessage};
use talin::tools::ToolError;
use tokio::io::{AsyncBufReadExt, BufReader, Lines, Stdin};
use tokio::sync::Mutex;

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
async fn next_line(label: &str) -> Option<String> {
    tokio::select! {
        line = prompt(label) => line.filter(|line| !line.is_empty()),
        _ = tokio::signal::ctrl_c() => {
            println!();
            None
        }
    }
}

/// Asks whether the tool calls may run.
async fn approved(calls: &[ToolCall]) -> bool {
    for call in calls {
        println!(
            "  The assistant wants to call `{}` with {}",
            call.name, call.arguments
        );
    }
    prompt("  Allow? [y/N] ")
        .await
        .is_some_and(|answer| answer.eq_ignore_ascii_case("y"))
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    env_logger::init();

    // A session keeps the transcript, so every turn sees the conversation so far.
    let agent = AssistantAgent::new(Models::gpt_4o_mini()).await;
    let mut session = agent.session();
    // Limits apply per turn. They can be changed at any time.
    session.options = session.options.with_max_iterations(6);

    println!(
        "Ask for an email to be drafted. Ctrl-C cancels a reply, and quits at the prompt. An empty line quits too."
    );
    while let Some(message) = next_line("\n> ").await {
        // Dropping the turn when Ctrl-C arrives abandons it. The session stays usable.
        let reply = tokio::select! {
            reply = async {
                let mut turn = session.turn((), AssistantAgentInput { message: message.into() }).await?;

                // Step through the turn so that the tool calls can be inspected before they run.
                // `run_to_finish` would retry failed steps for us. Doing it by hand means we also
                // have to give up eventually.
                let mut fails = 0;
                let partial = loop {
                    let result = async {
                        let pending = turn.half_step().await?;
                        let LlmStep::ToolCall(message) = pending.step.clone() else {
                            // The model answered. `complete` parses the answer.
                            return pending.complete().await;
                        };
                        if approved(&message.tool_calls).await {
                            return pending.complete().await;
                        }
                        // Answer each call ourselves. The model reads this as the tool's result.
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
                        // A step with tool calls, and no answer yet.
                        Ok(None) => fails = 0,
                        // A failed step is rolled back (unless every tool call failed, in which case
                        // the errors are kept for the model to read), so trying again can work.
                        Err(e) if e.is_retryable() && fails < 2 => {
                            fails += 1;
                            eprintln!("  (retrying: {e})");
                        }
                        Err(e) => return Err::<_, AgentError>(e),
                    }
                };

                Ok(turn.finalize(partial).await?.content)
            } => Some(reply),
            _ = tokio::signal::ctrl_c() => None,
        };

        match reply {
            Some(Ok(answer)) => println!("\n{answer}"),
            Some(Err(e)) => eprintln!("\nThe turn failed: {e}"),
            None => println!("\nCancelled."),
        }
    }

    // A cancelled read of the terminal may still be waiting in the background. Exit now instead of
    // waiting for it.
    std::process::exit(0);
}
