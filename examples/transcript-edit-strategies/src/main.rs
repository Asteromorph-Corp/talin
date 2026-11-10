mod agents;
mod tools;

use agents::{Models, MonitorAgent, MonitorAgentInput};
use talin::schemas::BodyMessage;
use talin::strategy::ApplyEntryEditStrategy;
use talin::transcript::EntryView;
use talin::transcript::edit::RemoveToolCalls;

const REQUEST: &str = "Tell me when the deployment is finished.";

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    env_logger::init();

    // Every poll adds a tool call and its result to the history, and all of them are sent with the
    // next request. Only the latest status matters, so drop the rest.
    //
    // The scheme is the same in both runs. What differs is when it applies.
    let scheme = || RemoveToolCalls::matching("check_deploy").except_last();

    // Permanent: the polls are removed from the stored transcript, once per step.
    let mut session = MonitorAgent::session(Models::gpt_4o_mini()).await;

    let input = MonitorAgentInput {
        request: REQUEST.into(),
    };
    let answer = match session
        .run_turn_with_strategy(ApplyEntryEditStrategy::new(scheme()), input)
        .await
    {
        Ok(output) => output.content,
        Err(e) => return eprintln!("{e}"),
    };
    let kept = session
        .transcript
        .full_view()
        .iter()
        .filter_map(|entry| match entry {
            EntryView::Message(_, BodyMessage::ToolCall(message)) => Some(message.tool_calls.len()),
            _ => None,
        })
        .sum::<usize>();
    println!("{kept} tool calls in the transcript. Answer: {answer}")
}
