//! Demonstrates the [`BufferedFinalAnswerStrategy`] strategy.
//!
//! The task is set up so the model is pushed to *assemble* its answer incrementally instead of
//! writing it in one shot: a `next_planet` tool hands out the subject matter one planet at a time
//! and only reveals the next once the current one has been requested. The natural way to cope is to
//! append each section to the shared answer buffer as it arrives, then submit the whole thing at
//! the end — which is exactly what the strategy's `edit_final_answer` / `submit_final_answer` tools
//! are for.
//!
//! With [`FinalAnswerAccess::ReadTool`] the model reads the buffer back on demand; the duplicate
//! Venus entry forces it to revise a section it already wrote rather than only appending.
//!
//! Run with: `ANTHROPIC_API_KEY=... cargo run --example buffered_final_answer`

use std::error::Error;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use talin::agent::{Agent, AgentOptions};
use talin::llm::ClaudeMessages;
use talin::strategy::BufferedFinalAnswerStrategy;
use talin::strategy::buffered_final_answer_strategy::{FinalAnswerAccess, FinalAnswerEdit};
use talin::tools::{EmptyFunctionInput, Function, Tool};

/// The material the study sheet is built from, handed out one entry at a time.
const PLANETS: &[(&str, &str)] = &[
    ("Mercury", "the smallest planet and the closest to the Sun"),
    (
        "Venus",
        "the hottest planet, shrouded in thick clouds of sulfuric acid",
    ),
    ("Earth", "the only planet known to harbor life"),
    (
        "Mars",
        "the red planet, home to the tallest volcano in the solar system",
    ),
    (
        "Venus",
        "an alien has recently been discovered on Venus, which is a shocking revelation for the scientific community",
    ),
];

/// Yields the next planet to document, or `DONE` once every planet has been handed out.
///
/// Because the facts arrive one at a time across separate tool calls, the model can't write the
/// whole study sheet up front — it has to accumulate it.
#[derive(Default)]
struct NextPlanet {
    cursor: Arc<Mutex<usize>>,
}

#[async_trait]
impl Function for NextPlanet {
    type Input = EmptyFunctionInput;
    type Output = String;

    fn name(&self) -> String {
        "next_planet".to_string()
    }

    fn description(&self) -> String {
        "Fetch the next planet to document. Returns the planet name and a fact about it, or DONE \
         once every planet has been handed out. Takes an empty object as input."
            .to_string()
    }

    async fn call(&self, _input: Self::Input) -> Result<String, Box<dyn Error + Send + Sync>> {
        let mut cursor = self.cursor.lock().unwrap();
        match PLANETS.get(*cursor) {
            Some((name, fact)) => {
                *cursor += 1;
                Ok(format!("{name}: {fact}"))
            }
            None => Ok("DONE".to_string()),
        }
    }
}

const SYSTEM: &str = "\
You are compiling a Markdown study sheet about the planets of the solar system.

Work through the planets one at a time:
1. Call `next_planet` to receive the next planet and a fact about it.
2. Append a section for that planet to your answer with `edit_final_answer` \
Each section is a `## ` heading with the planet's name, followed by one sentence describing it.
3. If the fact collides with a previous fact, or an additional information has been given for a \
previously documented planet, update the existing section instead of creating a new one.
4. Repeat until `next_planet` returns DONE.
5. Once every planet has a section, call `submit_final_answer` to deliver the study sheet.

Build the study sheet entirely through the answer buffer — do not write the planet sections \
directly in your replies.";

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    env_logger::init();

    let llm = ClaudeMessages::builder()
        .with_model("claude-opus-4-7")
        .build();
    let agent: Agent<_> = Agent::builder()
        .system_message(SYSTEM)
        .tools([Tool::from(NextPlanet::default())])
        .options(AgentOptions::default().with_max_iterations(30))
        .build(llm);

    // The buffer starts empty; the model fills it via the strategy's tools and submits when done.
    // `Replace` exposes an append/replace edit tool; swap in `FinalAnswerEdit::Patch` or
    // `FinalAnswerEdit::Hashline` for the unified-diff or hashline partial-update formats.
    let strategy =
        BufferedFinalAnswerStrategy::new(FinalAnswerAccess::ReadTool, FinalAnswerEdit::Replace);

    match agent
        .run_with_strategy(strategy, "Compile the planet study sheet.")
        .await
    {
        Ok(output) => {
            println!("Final answer:\n{}", output.content);
        }
        Err(e) => eprintln!("Error: {e:?}"),
    }
}
