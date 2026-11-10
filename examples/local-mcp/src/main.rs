talin::include_models!();
talin::include_agents!();

/// Usage: `local-mcp [model] [http|stdio]`, where `model` is an entry of `configs/models.toml` such
/// as `gpt_4o_mini` (the default), `claude_haiku_45` or `gemini_flash_latest`.
///
/// - `http` expects the `server-http` binary to be running (`cargo run --features server-http --bin
///   server-http`).
/// - `stdio` spawns the `server-stdio` binary as a child process, so nothing else is needed.
#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    env_logger::init();

    let model = std::env::args().nth(1).unwrap_or("gpt_4o_mini".into());
    let transport = std::env::args().nth(2).unwrap_or("http".into());

    log::info!("Using model: {model}");
    let preset = Models::any(&model).unwrap_or_else(|e| panic!("{e}"));
    let result = match transport.as_str() {
        "stdio" => CombatStdioAgent::any(preset).await.run_immed().await,
        _ => CombatHttpAgent::any(preset).await.run_immed().await,
    };

    println!("------------FINISH------------");

    match result {
        Ok(output) => println!("{output:#?}"),
        Err(e) => eprintln!("Error: {e}"),
    }
}
