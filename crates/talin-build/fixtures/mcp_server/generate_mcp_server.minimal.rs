#[allow(dead_code)]
pub fn my_server() -> ::talin::mcp::McpServer {
    ::talin::mcp::McpServerHttp::new(
        String::from("https://mcp.example.com"),
        "my_server",
    )
    .with_allowed_tools(vec![String::from("search"), String::from("fetch")])
    .into()
}
