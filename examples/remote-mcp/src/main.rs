talin::include_models!();
talin::include_agents!();

const QUESTION: &str = "In the rust-lang/rust-clippy repository, how are lints organized?";

/// Usage: `remote-mcp [model]`, where `model` is an entry of `configs/models.toml`:
/// `gpt_4o_mini` (default), `claude_haiku_45` or `gemini_flash_latest`.
///
/// The first two have an MCP client of their own. Gemini does not, so there this program connects
/// to the server instead, with the same configuration.
#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    env_logger::init();

    let model = std::env::args().nth(1).unwrap_or("gpt_4o_mini".into());
    log::info!("Using model {model}");

    let preset = Models::any(&model).unwrap_or_else(|e| panic!("{e}"));
    match RepoGuideAgent::any(preset).await.run_immed(QUESTION).await {
        Ok(output) => println!("{output}"),
        Err(e) => eprintln!("{e}"),
    }
}
