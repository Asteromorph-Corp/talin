use indexmap::IndexSet;
use quote::format_ident;
use rstest::fixture;
use syn::parse_quote;

use crate::config::{
    AgentLocalTool, AgentMcpToolGrouped, AgentModel, AgentResponseFormat, Attachments, CallOptions,
    InputVariable, McpServer, Model, ModelPreset, PresetVariable, RuntimeVariable, Tool,
};

#[fixture]
pub fn agent_id() -> &'static str {
    "my_agent"
}

#[fixture]
pub fn agent_symbol() -> syn::Ident {
    format_ident!("MyAgent")
}

#[fixture]
pub fn ext_symbol() -> syn::Ident {
    format_ident!("MyAgentExt")
}

#[fixture]
pub fn session_ext_symbol() -> syn::Ident {
    format_ident!("MyAgentSessionExt")
}

#[fixture]
pub fn profile_symbol() -> syn::Ident {
    format_ident!("MyAgentProfile")
}

#[fixture]
pub fn input_symbol() -> syn::Ident {
    format_ident!("MyAgentInput")
}

#[fixture]
pub fn output_symbol() -> syn::Ident {
    format_ident!("MyAgentOutput")
}

#[fixture]
pub fn input_ctor() -> syn::Type {
    parse_quote!(MyAgentInputCtor)
}

#[fixture]
pub fn output_ctor() -> syn::Type {
    parse_quote!(MyAgentOutputCtor)
}

pub(crate) fn default_model_inline() -> AgentModel {
    AgentModel::Inline(Box::new(model_generic()))
}

pub(crate) fn default_model_ref() -> AgentModel {
    AgentModel::Reference("my_model".to_owned())
}

#[fixture]
pub fn preset_generic() -> ModelPreset {
    ModelPreset::GenericChat {
        flavor: IndexSet::new(),
        drop_thought: false,
    }
}

#[fixture]
pub fn variables() -> Vec<RuntimeVariable> {
    vec![
        RuntimeVariable::Input(Box::new(InputVariable {
            ident: format_ident!("input"),
            type_: parse_quote!(String),
        })),
        RuntimeVariable::Preset(PresetVariable::CurrentDate),
        RuntimeVariable::Env("MY_ENV_VAR".to_owned()),
    ]
}

#[fixture]
pub fn attachments() -> Attachments {
    Attachments {
        images: vec!["image".to_owned()],
        audios: vec!["audio".to_owned()],
        videos: vec!["video".to_owned()],
        documents: vec!["document".to_owned()],
    }
}

#[fixture]
pub fn local_tools() -> Vec<AgentLocalTool> {
    vec![
        AgentLocalTool {
            name: "my_tool".to_owned(),
            config: None,
        },
        AgentLocalTool {
            name: "my_other_tool".to_owned(),
            config: None,
        },
    ]
}

#[fixture]
pub fn mcp_servers() -> Vec<AgentMcpToolGrouped> {
    vec![
        AgentMcpToolGrouped {
            server: "my_server".to_owned(),
            names: Some(vec!["search".to_owned(), "fetch".to_owned()]),
        },
        AgentMcpToolGrouped {
            server: "my_other_server".to_owned(),
            names: None,
        },
    ]
}

#[fixture]
pub fn model_generic() -> Model {
    toml::from_str(
        r#"
            api_base = "https://www.example.com"
            model = "my_model"
        "#,
    )
    .expect("model fixture TOML is valid")
}

#[fixture]
pub fn tool_symbol() -> syn::Ident {
    format_ident!("MyTool")
}

#[fixture]
pub fn tool() -> Tool {
    toml::from_str(
        r#"
            name = "my_tool"
            description = "A tool that does something useful."

            [input]
            query = { type = "String", description = "The search query." }
            limit = "u8"

            [config]
            api_key = { type = "String", description = "The API key." }
        "#,
    )
    .expect("tool fixture TOML is valid")
}

#[fixture]
pub fn mcp_server() -> McpServer {
    toml::from_str(
        r#"
            url = "https://mcp.example.com"
            description = "An example MCP server."
            authorization = "secret-token"
            tools = ["search", "fetch"]
        "#,
    )
    .expect("mcp server fixture TOML is valid")
}

#[fixture]
pub fn mcp_stdio_server() -> McpServer {
    toml::from_str(
        r#"
            command = "npx"
            args = ["-y", "some-mcp-server"]
            env = { API_KEY.env = "SOME_API_KEY", MODE = "fast" }
            tools = ["search"]
        "#,
    )
    .expect("mcp stdio server fixture TOML is valid")
}

#[fixture]
pub fn response_format() -> AgentResponseFormat {
    toml::from_str(
        r#"
            name = "MyResponse"
            description = "A structured agent response."

            [fields]
            answer = { type = "String", description = "The final answer." }
            score = "u8"
        "#,
    )
    .expect("response format fixture TOML is valid")
}

pub(crate) fn call_options() -> CallOptions {
    toml::from_str(
        r#"
            stream = true
            tool_choice = "auto"
            parallel_tool_calls = false
            n = 2
            reasoning_effort = "high"
            temperature = 0.7
            top_p = 0.9
            top_k = 40
            frequency_penalty = 0.1
            presence_penalty = 0.2
            repetition_penalty = 1.1
            max_tokens = 1024
            max_completion_tokens = 512
            stop_words = ["STOP", "END"]

            [extra_params]
            a_string = "hello"
            an_int = 42
            a_float = 3.5
            a_bool = true
            a_datetime = 1979-05-27T07:32:00Z
            an_array = [1, 2, 3]
            a_table = { nested = "value" }
        "#,
    )
    .expect("call options TOML is valid")
}
