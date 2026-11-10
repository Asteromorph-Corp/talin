use std::error::Error;
use std::path::PathBuf;

use automatic_compaction::agents::{Models, RecallAgent, RememberAgent};
use automatic_compaction::config::Config;
use automatic_compaction::connection;
use automatic_compaction::facts::Recall;
use automatic_compaction::on_demand::{self, OnDemandConfig};
use clap::{Parser, ValueEnum};
use talin::llm::claude_messages::MessagesOptions;
use talin::llm::{Llm, LlmPreset};
use talin::strategy::CustomCompactStrategy;

#[derive(Clone, Copy, ValueEnum)]
enum Mode {
    Claude,
    Openai,
    Custom,
}

#[derive(Parser)]
#[command(about = "Exercise on-demand and custom compaction through normal agent sessions")]
struct Args {
    #[arg(value_enum)]
    mode: Mode,
    #[arg(long)]
    stream: bool,
    #[arg(long)]
    api_base: Option<String>,
    #[arg(long, default_value = "target/on-demand-report.json")]
    report: PathBuf,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv().ok();
    let args = Args::parse();
    if args.stream && matches!(args.mode, Mode::Openai) {
        return Err("select Claude or custom to exercise streaming".into());
    }
    let config = Config::load()?;
    let workload = OnDemandConfig::load()?;
    let facts = Recall::fresh();
    let report = match args.mode {
        Mode::Openai => {
            let llm = match &args.api_base {
                Some(base) => connection::openai_compact(base)?,
                None => Models::openai_compact().model,
            };
            let remember = RememberAgent::new(LlmPreset::new(llm.clone(), ())).await;
            let recall = RecallAgent::new(LlmPreset::new(llm, ())).await;
            on_demand::run(
                remember,
                recall,
                workload.openai.clone(),
                &config,
                &workload,
                facts,
            )
            .await?
        }
        Mode::Claude | Mode::Custom => {
            let llm = match (&args.api_base, args.mode) {
                (Some(base), Mode::Custom) => connection::claude_custom(base, args.stream)?,
                (Some(base), _) => connection::claude_compact(base, args.stream)?,
                (None, Mode::Custom) => Models::claude_custom().model,
                (None, _) => Models::claude_compact().model,
            }
            .with_default_options(MessagesOptions::default().with_stream(args.stream));
            let remember = RememberAgent::new(LlmPreset::new(llm.clone(), ())).await;
            match args.mode {
                Mode::Claude => {
                    let recall = RecallAgent::new(LlmPreset::new(llm, ())).await;
                    on_demand::run(
                        remember,
                        recall,
                        workload.claude.clone(),
                        &config,
                        &workload,
                        facts,
                    )
                    .await?
                }
                Mode::Custom => {
                    let strategy = CustomCompactStrategy::new(
                        workload.custom.clone(),
                        workload.message_count.clone(),
                    );
                    let recall = RecallAgent::new(LlmPreset::new(llm, ())).await;
                    on_demand::run(remember, recall, strategy, &config, &workload, facts).await?
                }
                Mode::Openai => unreachable!(),
            }
        }
    };
    if let Some(parent) = args.report.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&args.report, serde_json::to_vec_pretty(&report)?)?;
    println!(
        "Compaction, recall, and stored history verified: {}",
        args.report.display()
    );
    Ok(())
}
