#[allow(dead_code)]
pub fn my_server() -> ::talin::mcp::McpServer {
    ::talin::mcp::McpServerStdio::new("npx", "my_server")
        .with_allowed_tools(vec![String::from("search")])
        .with_args(vec![String::from("-y"), String::from("some-mcp-server")])
        .with_env(vec![
            (
                String::from("API_KEY"),
                std::env::var("SOME_API_KEY")
                    .unwrap_or_else(|e| panic!("Failed to get env var '{}': {}", "SOME_API_KEY", e))
                    .into(),
            ),
            (String::from("MODE"), String::from("fast").into()),
        ])
        .into()
}
