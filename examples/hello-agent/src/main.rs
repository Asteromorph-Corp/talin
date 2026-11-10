talin::include_models!();
talin::include_agents!();

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    env_logger::init();

    // `TranslatorAgent` is generated from `[translator]` in `configs/agents.toml`.
    // `default()` builds it with the model that agent names.
    let agent = TranslatorAgent::default().await;

    // The `{{language}}` and `{{text}}` placeholders became the parameters of `run_immed`.
    match agent.run_immed("French", "Good morning, everyone.").await {
        Ok(translation) => println!("{translation}"),
        Err(e) => eprintln!("{e}"),
    }
}
