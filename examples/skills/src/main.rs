//! An agent that loads skills configured in `agents.toml`.
//!
//! Run with `cargo run` to run the agent (needs `ANTHROPIC_API_KEY`), or `cargo run -- --list` to
//! only list the skills the agent loaded.

mod agents;

use agents::{Models, ReviewerAgent, ReviewerAgentInput};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    env_logger::init();

    let mut session = ReviewerAgent::session(Models::claude_haiku_45()).await;
    // Skills are only read when first used. Asking for their metadata checks them up front.
    match session.skills.metadata().await {
        Ok(skills) => {
            for skill in skills {
                println!("{}: {}", skill.name(), skill.description());
            }
        }
        Err(e) => return eprintln!("{e}"),
    }
    if std::env::args().any(|arg| arg == "--list") {
        return;
    }

    let poem = "An old silent pond / A frog jumps into the pond / Splash! Silence again.";
    match session
        .run_turn(ReviewerAgentInput { poem: poem.into() })
        .await
    {
        Ok(review) => println!("{review}"),
        Err(e) => eprintln!("{e}"),
    }
}
