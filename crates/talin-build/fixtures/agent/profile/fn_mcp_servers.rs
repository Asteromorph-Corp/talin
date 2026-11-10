fn mcp_servers() -> Vec<::talin::mcp::McpServer> {
    vec![
        __agent_deps::McpServers::my_server()
            .with_allowed_tools(vec![String::from("search"), String::from("fetch")]),
        __agent_deps::McpServers::my_other_server(),
    ]
}
