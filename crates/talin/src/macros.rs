/// Includes generated agent definitions from the build output directory.
///
/// This macro includes three generated files:
/// - `models.rs`: Model definitions and configurations
/// - `servers.rs`: Server endpoint definitions
/// - `agent_index.rs`: Index of all available agents
#[macro_export]
macro_rules! include_agents {
    () => {
        #[doc(hidden)]
        mod __agent_deps {
            include!(concat!(env!("OUT_DIR"), "/models.rs"));
            include!(concat!(env!("OUT_DIR"), "/servers.rs"));
        }
        include!(concat!(env!("OUT_DIR"), "/agent_index.rs"));
    };
}

/// Includes generated model definitions from the build output directory.
///
/// This macro includes the `models.rs` file which contains LLM model configurations and type
/// definitions.
#[macro_export]
macro_rules! include_models {
    () => {
        include!(concat!(env!("OUT_DIR"), "/models.rs"));
    };
}
