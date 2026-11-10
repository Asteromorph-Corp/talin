fn agent_options() -> ::talin::agent::AgentOptions {
    ::talin::agent::AgentOptions::default()
        .with_max_consecutive_fails(5usize)
        .with_max_iterations(10usize)
        .with_max_continuations(2usize)
}
