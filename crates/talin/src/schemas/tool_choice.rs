/// A parameter to control which (if any) tool is called by the model.
///
/// Note that these options only apply to models that support function call natively.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum ToolChoice {
    /// The model will not call any tool and instead generates a message.
    None,
    /// The model can pick between generating a message or calling one or more tools.
    Auto,
    /// The model must call one or more tools.
    Required,
    /// Forces the model to call a specific function.
    Function(String),
    /// Forces the model to call a specific tool on a remote MCP server.
    Mcp(String, String),
    /// The model must use a provider specific function.
    ProviderSpecific(&'static str),
}

impl ToolChoice {
    /// The model must use the file search tool (OpenAI Responses).
    pub const FILE_SEARCH: Self = ToolChoice::ProviderSpecific("file_search");

    /// The model must use the web search preview tool (OpenAI Responses).
    pub const WEB_SEARCH_PREVIEW: Self = ToolChoice::ProviderSpecific("web_search_preview");

    /// The model must use the computer use preview tool (OpenAI Responses).
    pub const COMPUTER_USE_PREVIEW: Self = ToolChoice::ProviderSpecific("computer_use_preview");

    /// The model must use the code interpreter tool (OpenAI Responses).
    pub const CODE_INTERPRETER: Self = ToolChoice::ProviderSpecific("code_interpreter");

    /// The model must use the image generation tool (OpenAI Responses).
    pub const IMAGE_GENERATION: Self = ToolChoice::ProviderSpecific("image_generation");

    /// The model must use the apply patch tool (OpenAI Responses).
    pub const APPLY_PATCH: Self = ToolChoice::ProviderSpecific("apply_patch");

    /// The model must use the shell tool (OpenAI Responses).
    pub const SHELL: Self = ToolChoice::ProviderSpecific("shell");

    /// The model must call exactly one specific function.
    #[must_use]
    #[inline]
    pub fn function(name: impl Into<String>) -> Self {
        ToolChoice::Function(name.into())
    }
}
