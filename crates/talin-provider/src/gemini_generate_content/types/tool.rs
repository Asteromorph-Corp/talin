use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

mod builtin_tool;
mod code_execution;
mod computer_use;
mod file_search;
mod function_decalaration;
mod google_maps;
mod google_search;
mod google_search_retrieval;
mod mcp_server;
mod url_context;

pub use builtin_tool::*;
pub use code_execution::*;
pub use computer_use::*;
pub use file_search::*;
pub use function_decalaration::*;
pub use google_maps::*;
pub use google_search::*;
pub use google_search_retrieval::*;
pub use mcp_server::*;
pub use url_context::*;

/// A Tool is a piece of code that enables the system to interact with external systems.
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Tool {
    /// Optional. A list of FunctionDeclarations available to the model.
    pub function_declarations: Option<Vec<FunctionDeclaration>>,

    /// Optional. Retrieval tool that is powered by Google search.
    pub google_search_retrieval: Option<GoogleSearchRetrieval>,

    /// Optional. Enables the model to execute code as part of generation.
    pub code_execution: Option<CodeExecution>,

    /// Optional. GoogleSearch tool type. Tool to support Google Search in Model. Powered by
    /// Google.
    pub google_search: Option<GoogleSearch>,

    /// Optional. Tool to support the model interacting directly with the computer. If enabled, it
    /// automatically populates computer-use specific Function Declarations.
    pub computer_use: Option<ComputerUse>,

    /// Optional. Tool to support URL context retrieval.
    pub url_context: Option<UrlContext>,

    /// Optional. FileSearch tool type. Tool to retrieve knowledge from Semantic Retrieval corpora.
    pub file_search: Option<FileSearch>,

    /// Optional. Tool that allows grounding the model's response with geospatial context related
    /// to the user's query.
    pub google_maps: Option<GoogleMaps>,

    /// Optional. Configuration for Model Context Protocol (MCP) servers that
    /// expose tools to the model.
    pub mcp_servers: Option<Vec<McpServer>>,
}

impl Tool {
    /// Creates a new empty Tool.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a Tool with code execution enabled.
    pub fn code_execution() -> Self {
        Self {
            code_execution: Some(CodeExecution {}),
            ..Default::default()
        }
    }

    /// Creates a Tool with the given function declarations.
    pub fn functions(declarations: Vec<FunctionDeclaration>) -> Self {
        Self {
            function_declarations: Some(declarations),
            ..Default::default()
        }
    }

    /// Creates a Tool with Google Search enabled.
    pub fn google_search() -> Self {
        Self {
            google_search: Some(GoogleSearch::default()),
            ..Default::default()
        }
    }

    /// Adds a function declaration to this tool.
    pub fn with_function(mut self, declaration: FunctionDeclaration) -> Self {
        self.function_declarations
            .get_or_insert_with(Vec::new)
            .push(declaration);
        self
    }

    /// Adds multiple function declarations to this tool.
    pub fn with_functions(mut self, declarations: Vec<FunctionDeclaration>) -> Self {
        self.function_declarations
            .get_or_insert_with(Vec::new)
            .extend(declarations);
        self
    }

    /// Enables code execution for this tool.
    pub fn with_code_execution(mut self) -> Self {
        self.code_execution = Some(CodeExecution {});
        self
    }

    /// Sets Google Search retrieval for this tool.
    pub fn with_google_search_retrieval(mut self, retrieval: GoogleSearchRetrieval) -> Self {
        self.google_search_retrieval = Some(retrieval);
        self
    }

    /// Enables Google Search for this tool.
    pub fn with_google_search(mut self, search: GoogleSearch) -> Self {
        self.google_search = Some(search);
        self
    }

    /// Enables computer use for this tool.
    pub fn with_computer_use(mut self, computer_use: ComputerUse) -> Self {
        self.computer_use = Some(computer_use);
        self
    }

    /// Enables URL context for this tool.
    pub fn with_url_context(mut self, url_context: UrlContext) -> Self {
        self.url_context = Some(url_context);
        self
    }

    /// Enables file search for this tool.
    pub fn with_file_search(mut self, file_search: FileSearch) -> Self {
        self.file_search = Some(file_search);
        self
    }

    /// Enables Google Maps for this tool.
    pub fn with_google_maps(mut self, google_maps: GoogleMaps) -> Self {
        self.google_maps = Some(google_maps);
        self
    }

    /// Sets the MCP servers for this tool.
    pub fn with_mcp_servers(mut self, servers: Vec<McpServer>) -> Self {
        self.mcp_servers = Some(servers);
        self
    }
}
