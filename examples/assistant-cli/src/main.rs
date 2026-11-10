//! A terminal assistant, built on `talin`.
//!
//! See the README for a tour of the features it uses.

mod agents;
mod tools;
mod turn;

use std::sync::Arc;

use agents::{
    Assistant, AssistantAgent, AssistantSession, Models, ReporterAgent, ReporterAgentInput,
    ReporterAgentOutput,
};
use talin::agent::AgentError;
use talin::schemas::Usage;
use talin::strategy::{CustomCompactStrategy, InstructionCompactor, MessageCount};
use tokio::sync::Mutex;

/// Where `take_note` writes, as configured in `agents.toml`.
const NOTES_FILE: &str = "notes/notes.md";

const HELP: &str = "\
Ask anything about the project. Commands:
  /report [focus]  write a report of the conversation
  /notes           show the notes the assistant took
  /usage           show the tokens used so far, and their cost
  /help            show this message
Ctrl-C cancels a reply, and quits at the prompt. An empty line quits too.";

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    env_logger::init();

    // The usage of every turn is added to this total.
    let total = Arc::new(Mutex::new(Usage::default()));

    let assistant = AssistantAgent::new(Models::claude_haiku_45()).await;
    let mut session = assistant.session().with_compaction(
        // Once the conversation holds 30 messages, everything but the last 10 is summarized.
        // The messages stay in the transcript, but requests carry the summary instead.
        CustomCompactStrategy::new(
            InstructionCompactor::new(
                "Summarize the conversation so far. Keep the names of files, decisions, and open questions.",
            ),
            MessageCount {
                threshold: 30,
                retain_messages: 10,
            },
        ),
    );

    println!("{HELP}");
    while let Some(line) = turn::next_line("\n> ").await {
        match line.split_once(' ').unwrap_or((line.as_str(), "")) {
            ("/help", _) => println!("{HELP}"),
            ("/notes", _) => match std::fs::read_to_string(tools::project_dir().join(NOTES_FILE)) {
                Ok(notes) => print!("{notes}"),
                Err(_) => println!("No notes yet."),
            },
            ("/usage", _) => println!("{}", total.lock().await),
            ("/report", focus) => {
                let focus = if focus.is_empty() {
                    "what was decided"
                } else {
                    focus
                };
                let (back, report) = report(&assistant, session, focus).await;
                session = back;
                match report {
                    Ok(report) => print_report(&report),
                    Err(e) => eprintln!("The report failed: {e}"),
                }
            }
            _ => {
                // Dropping the turn when Ctrl-C arrives abandons it. The session stays usable.
                let reply = tokio::select! {
                    reply = turn::run(&mut session, &total, line) => Some(reply),
                    _ = tokio::signal::ctrl_c() => None,
                };
                match reply {
                    Some(Ok((answer, usage))) => println!("\n{answer}\n\n[this turn]\n{usage}"),
                    Some(Err(e)) => eprintln!("\nThe turn failed: {e}"),
                    None => println!("\nCancelled."),
                }
            }
        }
    }

    println!("\n[session total]\n{}", total.lock().await);

    // A cancelled read of the terminal may still be waiting in the background. Exit now instead of
    // waiting for it.
    std::process::exit(0);
}

/// Has the reporter agent write a report on the conversation.
///
/// The session is handed to the reporter and back, so the report is part of the history afterwards.
async fn report(
    assistant: &Assistant,
    session: AssistantSession,
    focus: &str,
) -> (AssistantSession, Result<ReporterAgentOutput, AgentError>) {
    let reporter = ReporterAgent::new(Models::claude_haiku_45()).await;
    let mut session = reporter.continue_session(session);
    let report = session
        .run_turn(ReporterAgentInput {
            focus: focus.into(),
        })
        .await;
    (assistant.continue_session(session), report)
}

fn print_report(report: &ReporterAgentOutput) {
    println!("\n{}\n{}\n", report.title, report.summary);
    println!("Decisions:");
    report.decisions.iter().for_each(|d| println!("  - {d}"));
    println!("Follow-ups:");
    report.follow_ups.iter().for_each(|f| println!("  - {f}"));
}
