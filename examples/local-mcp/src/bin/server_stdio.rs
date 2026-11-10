//! Dice roller MCP server over stdio.
//!
//! Normally spawned by the client as a child process (see `configs/servers.toml`); run it with
//! `cargo run --features server-stdio --bin server-stdio`. Stdout carries the MCP protocol, so
//! nothing else may be printed to it.

use local_mcp::DiceRoller;
use rmcp::ServiceExt;
use rmcp::transport::stdio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let service = DiceRoller.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}
