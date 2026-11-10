mod agents;
mod tools;

use agents::{GuesserAgent, GuesserAgentInput, Models, SummarizerAgent, SummarizerAgentInput};
use talin::transcript::{EntryEdit, EntryView, Scope};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    dotenvy::dotenv().ok();
    env_logger::init();

    let mut session = GuesserAgent::session(Models::gpt_4o_mini()).await;

    let result1 = session.run_turn(GuesserAgentInput {}).await?;
    println!("{:#?}", result1);

    let agent2 = GuesserAgent::new(Models::claude_haiku_45()).await;

    let mut session = agent2.continue_session_migrated(session)?;
    let mut turn2 = session.turn((), GuesserAgentInput {}).await?;
    let result2 = loop {
        if let Some(result) = turn2.step().await? {
            break result;
        }

        let scope = Scope::full();
        let view = turn2.session.transcript.view(&scope);
        let edits = view
            .into_iter()
            .filter_map(|entry| match entry {
                EntryView::ToolError(_, _) => None,
                other => Some(EntryEdit::from(other)),
            })
            .collect::<Vec<_>>();
        turn2.session.transcript.modify(edits, &scope)?;
    };
    let result2 = turn2.finalize(result2).await?;
    println!("{:#?}", result2.content);

    let agent = SummarizerAgent::new(Models::claude_haiku_45()).await;
    let mut session = agent.continue_session(session);
    let turn3 = session.turn((), SummarizerAgentInput {}).await?;
    let result3 = turn3.run_to_finish().await?;
    println!("{:#?}", result3.content);

    Ok(())
}
