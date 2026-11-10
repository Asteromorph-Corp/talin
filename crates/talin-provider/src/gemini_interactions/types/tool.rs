use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use super::AllowedTools;
use crate::utils::{string_enum, tagged_enum};

mod builtin_tool;
pub use builtin_tool::*;

tagged_enum! {
    /// A tool that the model may use during an interaction.
    ///
    /// Tools are discriminated by their `type` field.
    #[derive(Serialize, Deserialize, Debug, Clone)]
    #[serde(tag = "type", rename_all = "snake_case")]
    pub enum Tool {
        /// A user-defined function the model may call.
        Function(Function),
        /// Enables the model to execute code as part of generation.
        CodeExecution(CodeExecution),
        /// Enables the model to fetch URL context.
        UrlContext(UrlContext),
        /// Enables the model to ground responses with Google Search.
        GoogleSearch(GoogleSearch),
        /// Enables the model to retrieve knowledge from file search stores.
        FileSearch(FileSearch),
        /// Enables the model to ground responses with Google Maps.
        GoogleMaps(GoogleMaps),
        /// Enables the model to interact with a computer.
        ComputerUse(ComputerUse),
        /// Enables the model to call tools hosted on an MCP server.
        McpServer(McpServer),
    }
}

/// A user-defined function the model may call.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[non_exhaustive]
pub struct Function {
    /// A description of the function.
    #[cfg_attr(test, schema(nullable = false))]
    pub description: Option<String>,

    /// The name of the function.
    #[cfg_attr(test, schema(nullable = false))]
    pub name: Option<String>,

    /// The JSON Schema for the function's parameters.
    #[cfg_attr(test, schema(value_type = Value))]
    pub parameters: Option<schemars::Schema>,
}

/// A tool that can be used by the model to execute code.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[non_exhaustive]
pub struct CodeExecution {}

/// A tool that can be used by the model to fetch URL context.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[non_exhaustive]
pub struct UrlContext {}

/// A tool that can be used by the model to search Google.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[non_exhaustive]
pub struct GoogleSearch {
    /// The types of search grounding to enable.
    #[cfg_attr(test, schema(nullable = false))]
    pub search_types: Option<Vec<GoogleSearchType>>,
}

string_enum! {
    /// The type of search grounding to enable.
    #[serde(rename_all = "snake_case")]
    pub enum GoogleSearchType {
        /// Enables web search. Only text results are returned.
        WebSearch,

        /// Enables image search. Image bytes are returned.
        ImageSearch,
    }
}

/// A tool that can be used by the model to search files.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[non_exhaustive]
pub struct FileSearch {
    /// The file search store names to search.
    #[cfg_attr(test, schema(nullable = false))]
    pub file_search_store_names: Option<Vec<String>>,

    /// Metadata filter to apply to the semantic retrieval documents and chunks.
    #[cfg_attr(test, schema(nullable = false))]
    pub metadata_filter: Option<String>,

    /// The number of semantic retrieval chunks to retrieve.
    #[cfg_attr(test, schema(nullable = false))]
    pub top_k: Option<i32>,
}

/// A tool that can be used by the model to call Google Maps.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[non_exhaustive]
pub struct GoogleMaps {
    /// Whether to return a widget context token in the tool call result of the response.
    #[cfg_attr(test, schema(nullable = false))]
    pub enable_widget: Option<bool>,

    /// The latitude of the user's location.
    #[cfg_attr(test, schema(nullable = false))]
    pub latitude: Option<f64>,

    /// The longitude of the user's location.
    #[cfg_attr(test, schema(nullable = false))]
    pub longitude: Option<f64>,
}

/// A tool that can be used by the model to interact with the computer.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[non_exhaustive]
pub struct ComputerUse {
    /// Disabled safety policies for computer use.
    #[cfg_attr(test, schema(nullable = false))]
    pub disabled_safety_policies: Option<Vec<ComputerUseSafetyPolicy>>,

    /// Whether to enable the prompt injection detection check on computer-use requests.
    #[cfg_attr(test, schema(nullable = false))]
    pub enable_prompt_injection_detection: Option<bool>,

    /// The environment being operated.
    #[cfg_attr(test, schema(nullable = false))]
    pub environment: Option<ComputerUseEnvironment>,

    /// The list of predefined functions that are excluded from the model call.
    #[cfg_attr(test, schema(nullable = false))]
    pub excluded_predefined_functions: Option<Vec<String>>,
}

string_enum! {
    /// A safety policy that can be disabled for computer use.
    #[serde(rename_all = "snake_case")]
    pub enum ComputerUseSafetyPolicy {
        /// Safety policy for financial transactions.
        FinancialTransactions,

        /// Safety policy for sensitive data modification.
        SensitiveDataModification,

        /// Safety policy for communication tools (e.g. Gmail, Chat, Meet).
        CommunicationTool,

        /// Safety policy for account creation.
        AccountCreation,

        /// Safety policy for data modification.
        DataModification,

        /// Safety policy for user consent management.
        UserConsentManagement,

        /// Safety policy for legal terms and agreements.
        LegalTermsAndAgreements,
    }
}

string_enum! {
    /// The environment a computer use tool operates in.
    #[serde(rename_all = "snake_case")]
    pub enum ComputerUseEnvironment {
        /// Operates in a web browser.
        Browser,

        /// Operates in a mobile environment.
        Mobile,

        /// Operates in a desktop environment.
        Desktop,
    }
}

/// A server that can be called by the model to perform actions.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[non_exhaustive]
pub struct McpServer {
    /// The allowed tools.
    #[cfg_attr(test, schema(nullable = false))]
    pub allowed_tools: Option<Vec<AllowedTools>>,

    /// Authentication headers, timeouts, and similar fields, if needed.
    #[cfg_attr(test, schema(schema_with = crate::gemini_interactions::spec::schema::map_schema, nullable = false))]
    pub headers: Option<BTreeMap<String, String>>,

    /// The name of the server.
    #[cfg_attr(test, schema(nullable = false))]
    pub name: Option<String>,

    /// The full URL for the server endpoint, for example `https://api.example.com/mcp`.
    #[cfg_attr(test, schema(nullable = false))]
    pub url: Option<String>,
}

/// Configuration for Exa AI Search.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[non_exhaustive]
pub struct ExaAISearchConfig {
    /// The API key for Exa AI Search.
    pub api_key: String,

    /// Any parameter accepted by the Exa AI Search API.
    #[cfg_attr(test, schema(schema_with = crate::gemini_interactions::spec::schema::object_schema, nullable = false))]
    pub custom_config: Option<serde_json::Value>,
}

/// Configuration for Parallel AI Search.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[non_exhaustive]
pub struct ParallelAISearchConfig {
    /// The API key for Parallel AI Search.
    #[cfg_attr(test, schema(nullable = false))]
    pub api_key: Option<String>,

    /// Custom configuration for Parallel AI Search.
    #[cfg_attr(test, schema(schema_with = crate::gemini_interactions::spec::schema::object_schema, nullable = false))]
    pub custom_config: Option<serde_json::Value>,
}

/// Configuration for Vertex AI Search.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[non_exhaustive]
pub struct VertexAISearchConfig {
    /// The Vertex AI Search datastores to use.
    #[cfg_attr(test, schema(nullable = false))]
    pub datastores: Option<Vec<String>>,

    /// The Vertex AI Search engine to use.
    #[cfg_attr(test, schema(nullable = false))]
    pub engine: Option<String>,
}

/// Configuration for the RAG store.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[non_exhaustive]
pub struct RagStoreConfig {
    /// The representation of the RAG source.
    #[cfg_attr(test, schema(nullable = false))]
    pub rag_resources: Option<Vec<RagResource>>,

    /// The retrieval config for the RAG query.
    #[cfg_attr(test, schema(nullable = false))]
    pub rag_retrieval_config: Option<RagRetrievalConfig>,

    /// Number of top k results to return from the selected corpora.
    #[deprecated(note = "use `rag_retrieval_config.top_k` instead")]
    #[cfg_attr(test, schema(nullable = false))]
    pub similarity_top_k: Option<i32>,

    /// Only return results with vector distance smaller than the threshold.
    #[deprecated(note = "use `rag_retrieval_config.filter` instead")]
    #[cfg_attr(test, schema(nullable = false))]
    pub vector_distance_threshold: Option<f64>,
}

/// The definition of a RAG resource.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[non_exhaustive]
pub struct RagResource {
    /// The `RagCorpora` resource name.
    #[cfg_attr(test, schema(nullable = false))]
    pub rag_corpus: Option<String>,

    /// The IDs of the files to retrieve from. The files must be in the corpus named by
    /// `rag_corpus`.
    #[cfg_attr(test, schema(nullable = false))]
    pub rag_file_ids: Option<Vec<String>>,
}

/// The context retrieval config for a RAG query.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[non_exhaustive]
pub struct RagRetrievalConfig {
    /// Config for filters.
    #[cfg_attr(test, schema(nullable = false))]
    pub filter: Option<Filter>,

    /// Config for hybrid search.
    #[cfg_attr(test, schema(nullable = false))]
    pub hybrid_search: Option<HybridSearch>,

    /// Config for ranking and reranking.
    #[cfg_attr(test, schema(nullable = false))]
    pub ranking: Option<Ranking>,

    /// The number of contexts to retrieve.
    #[cfg_attr(test, schema(nullable = false))]
    pub top_k: Option<i32>,
}

/// Config for filters applied to a RAG query.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[non_exhaustive]
pub struct Filter {
    /// String for metadata filtering.
    #[cfg_attr(test, schema(nullable = false))]
    pub metadata_filter: Option<String>,

    /// Only return contexts with vector distance smaller than the threshold.
    #[cfg_attr(test, schema(nullable = false))]
    pub vector_distance_threshold: Option<f64>,

    /// Only return contexts with vector similarity larger than the threshold.
    #[cfg_attr(test, schema(nullable = false))]
    pub vector_similarity_threshold: Option<f64>,
}

/// Config for hybrid search.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[non_exhaustive]
pub struct HybridSearch {
    /// Weight between dense and sparse vector search results.
    #[cfg_attr(test, schema(nullable = false))]
    pub alpha: Option<f32>,
}

tagged_enum! {
    /// Config for ranking and reranking.
    #[derive(Serialize, Deserialize, Debug, Clone)]
    #[serde(tag = "ranking_config", rename_all = "snake_case")]
    pub enum Ranking {
        /// Ranking performed by the rank service.
        RankService(RankServiceRanking),
    }
}

/// Ranking performed by the rank service.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[non_exhaustive]
pub struct RankServiceRanking {
    /// The model name of the rank service.
    #[cfg_attr(test, schema(nullable = false))]
    pub model_name: Option<String>,

    /// Config for the rank service.
    #[cfg_attr(test, schema(nullable = false))]
    pub rank_service: Option<RankService>,
}

/// Config for the rank service.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[non_exhaustive]
pub struct RankService {
    /// The model name of the rank service.
    #[cfg_attr(test, schema(nullable = false))]
    pub model_name: Option<String>,
}

impl Tool {
    /// Creates a function tool with the given name.
    pub fn function(name: impl Into<String>) -> Self {
        Tool::Function(Function {
            description: None,
            name: Some(name.into()),
            parameters: None,
        })
    }

    /// Creates a code execution tool.
    pub fn code_execution() -> Self {
        Tool::CodeExecution(CodeExecution {})
    }

    /// Creates a URL context tool.
    pub fn url_context() -> Self {
        Tool::UrlContext(UrlContext {})
    }

    /// Creates a Google Search tool.
    pub fn google_search() -> Self {
        Tool::GoogleSearch(GoogleSearch::default())
    }

    /// Creates a file search tool for the given store names.
    pub fn file_search(store_names: Vec<String>) -> Self {
        Tool::FileSearch(FileSearch {
            file_search_store_names: Some(store_names),
            metadata_filter: None,
            top_k: None,
        })
    }

    /// Creates a Google Maps tool.
    pub fn google_maps() -> Self {
        Tool::GoogleMaps(GoogleMaps::default())
    }

    /// Creates a computer use tool.
    pub fn computer_use() -> Self {
        Tool::ComputerUse(ComputerUse::default())
    }

    /// Creates an MCP server tool for the given server name and endpoint URL.
    pub fn mcp_server(name: impl Into<String>, url: impl Into<String>) -> Self {
        Tool::McpServer(McpServer::new(name, url))
    }
}

impl Function {
    /// Creates a new function with the given name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            description: None,
            name: Some(name.into()),
            parameters: None,
        }
    }

    /// Sets the name of the function.
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Sets the description of the function.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Sets the JSON Schema for the function's parameters.
    pub fn with_parameters(mut self, parameters: schemars::Schema) -> Self {
        self.parameters = Some(parameters);
        self
    }
}

impl CodeExecution {
    /// Creates a new code execution configuration.
    pub fn new() -> Self {
        Self::default()
    }
}

impl UrlContext {
    /// Creates a new URL context configuration.
    pub fn new() -> Self {
        Self::default()
    }
}

impl GoogleSearch {
    /// Creates a new, empty Google Search configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the types of search grounding to enable.
    pub fn with_search_types(mut self, search_types: Vec<GoogleSearchType>) -> Self {
        self.search_types = Some(search_types);
        self
    }
}

impl FileSearch {
    /// Creates a new file search configuration for the given store names.
    pub fn new(store_names: Vec<String>) -> Self {
        Self {
            file_search_store_names: Some(store_names),
            metadata_filter: None,
            top_k: None,
        }
    }

    /// Sets the metadata filter to apply to the retrieved documents and chunks.
    pub fn with_metadata_filter(mut self, metadata_filter: impl Into<String>) -> Self {
        self.metadata_filter = Some(metadata_filter.into());
        self
    }

    /// Sets the number of semantic retrieval chunks to retrieve.
    pub fn with_top_k(mut self, top_k: i32) -> Self {
        self.top_k = Some(top_k);
        self
    }
}

impl GoogleMaps {
    /// Creates a new, empty Google Maps configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a Google Maps configuration anchored at the given latitude and longitude.
    pub fn at(latitude: f64, longitude: f64) -> Self {
        Self {
            enable_widget: None,
            latitude: Some(latitude),
            longitude: Some(longitude),
        }
    }

    /// Sets the user's location.
    pub fn with_location(mut self, latitude: f64, longitude: f64) -> Self {
        self.latitude = Some(latitude);
        self.longitude = Some(longitude);
        self
    }

    /// Enables returning a widget context token in the tool call result.
    pub fn enable_widget(mut self) -> Self {
        self.enable_widget = Some(true);
        self
    }
}

impl ComputerUse {
    /// Creates a new, empty computer use configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the safety policies to disable.
    pub fn with_disabled_safety_policies(mut self, policies: Vec<ComputerUseSafetyPolicy>) -> Self {
        self.disabled_safety_policies = Some(policies);
        self
    }

    /// Enables the prompt injection detection check on computer-use requests.
    pub fn enable_prompt_injection_detection(mut self) -> Self {
        self.enable_prompt_injection_detection = Some(true);
        self
    }

    /// Sets the environment being operated.
    pub fn with_environment(mut self, environment: ComputerUseEnvironment) -> Self {
        self.environment = Some(environment);
        self
    }

    /// Sets the predefined functions excluded from the model call.
    pub fn with_excluded_predefined_functions(mut self, functions: Vec<String>) -> Self {
        self.excluded_predefined_functions = Some(functions);
        self
    }
}

impl McpServer {
    /// Creates a new MCP server configuration for the given name and endpoint URL.
    pub fn new(name: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            allowed_tools: None,
            headers: None,
            name: Some(name.into()),
            url: Some(url.into()),
        }
    }

    /// Sets the allowed tools.
    pub fn with_allowed_tools(mut self, allowed_tools: Vec<AllowedTools>) -> Self {
        self.allowed_tools = Some(allowed_tools);
        self
    }

    /// Sets the headers sent to the server endpoint.
    pub fn with_headers(mut self, headers: BTreeMap<String, String>) -> Self {
        self.headers = Some(headers);
        self
    }
}
