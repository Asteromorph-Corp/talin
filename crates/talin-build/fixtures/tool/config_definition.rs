#[derive(Debug, Default, ::talin::tools::ConfigSchema)]
pub struct MyToolConfig {
    ///The API key.
    pub api_key: String,
}
