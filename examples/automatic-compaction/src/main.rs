use std::error::Error;
use std::path::PathBuf;

use automatic_compaction::agents::Models;
use automatic_compaction::config::Config;
use automatic_compaction::facts::Recall;
use automatic_compaction::{application, connection};
use clap::{Parser, ValueEnum};
use talin::llm::Llm;
use talin::llm::claude_messages::MessagesOptions;

#[derive(Clone, Copy, ValueEnum)]
enum Provider {
    Openai,
    Claude,
}

#[derive(Parser)]
#[command(about = "Exercise automatic compaction through TOML-generated agents and tools")]
struct Args {
    #[arg(value_enum)]
    provider: Provider,
    #[arg(long)]
    stream: bool,
    #[arg(long, default_value = "target/auto-compaction-report.json")]
    report: PathBuf,
    /// Local recording proxy; model and options still come from models.toml.
    #[arg(long)]
    api_base: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv().ok();
    let args = Args::parse();
    let config = Config::load()?;
    let facts = Recall::fresh();
    let outcome = match args.provider {
        Provider::Openai => {
            if args.stream {
                return Err("select Claude to exercise the streaming scenario".into());
            }
            let llm = match &args.api_base {
                Some(base) => connection::openai(base)?,
                None => Models::openai().model,
            };
            application::openai(llm, config, facts, 1).await
        }
        Provider::Claude => {
            let llm = match &args.api_base {
                Some(base) => connection::claude(base, args.stream)?,
                None => Models::claude()
                    .model
                    .with_default_options(MessagesOptions::default().with_stream(args.stream)),
            };
            application::claude(llm, config, facts, 1).await
        }
    };
    outcome.report.save(&args.report)?;
    outcome.result?;
    println!(
        "Compaction and recall verified. Report: {}",
        args.report.display()
    );
    Ok(())
}
