talin::include_agents!();

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    env_logger::init();

    let mit_cs = MitCsAgent::default().await;
    let mit_cs_result = mit_cs
        .run_immed("What is the future of machine learning?")
        .await;

    let stanford_physics = StanfordPhysicsAgent::default().await;
    let stanford_physics_result = stanford_physics.run_immed("What is a dark matter?").await;

    println!("------------FINISH------------");

    for result in [mit_cs_result, stanford_physics_result] {
        match result {
            Ok(output) => println!("{output}"),
            Err(e) => eprintln!("{e}"),
        }
    }
}
