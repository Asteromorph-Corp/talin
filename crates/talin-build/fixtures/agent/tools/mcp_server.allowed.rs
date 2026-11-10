fn __dummy() {
    let _ = __agent_deps::McpServers::my_server()
        .with_allowed_tools(vec![String::from("search"), String::from("fetch")]);
}
