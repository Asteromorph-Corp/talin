//! Dice roller MCP server over streamable HTTP.
//!
//! Run with `cargo run --features server-http --bin server-http`.

use local_mcp::DiceRoller;
use rmcp::transport::StreamableHttpService;
use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;

const BIND_ADDRESS: &str = "127.0.0.1:8000";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let service = StreamableHttpService::new(
        || Ok(DiceRoller),
        LocalSessionManager::default().into(),
        Default::default(),
    );

    println!("🎲 Dice roller MCP server running at http://{BIND_ADDRESS}/mcp");

    let router = axum::Router::new().nest_service("/mcp", service);
    let tcp_listener = tokio::net::TcpListener::bind(BIND_ADDRESS).await?;
    axum::serve(tcp_listener, router)
        .with_graceful_shutdown(async { tokio::signal::ctrl_c().await.unwrap() })
        .await?;
    Ok(())
}
