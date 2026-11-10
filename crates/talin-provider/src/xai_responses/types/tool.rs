use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::utils::tagged_enum;

mod builtin_tool;
pub use builtin_tool::*;

tagged_enum! {
    /// Definition of one tool that the model can call.
    #[derive(Serialize, Deserialize, Debug, Clone)]
    #[serde(tag = "type", rename_all = "snake_case")]
    pub enum ModelTool {
        /// A function that the model can call.
        Function(FunctionTool),
        /// Search the web.
        WebSearch(WebSearchTool),
        /// Search X.
        XSearch(XSearchTool),
        /// Generate images from text prompts.
        ImageGeneration(ImageGenerationTool),
        /// Search the knowledge bases.
        FileSearch(FileSearchTool),
        /// Execute code.
        CodeInterpreter(CodeInterpreterTool),
        /// A remote MCP server to use.
        Mcp(McpTool),
        /// A local shell execution tool.
        Shell(ShellTool),
        /// Search over deferred tool definitions (server-side execution only).
        ToolSearch(ToolSearchTool),
    }
}

/// A function that the model can call.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct FunctionTool {
    /// The name of the function.
    /// If the model calls the function, this name is used in the response.
    pub name: String,

    /// A description of the function to indicate to the model when to call it.
    pub description: Option<String>,

    /// A JSON schema describing the function parameters.
    /// The model _should_ follow the schema, however, this is not enforced at the moment.
    pub parameters: serde_json::Value,

    /// When true, the definition is hidden from the model's prompt until loaded via a
    /// `tool_search` step.
    pub defer_loading: Option<bool>,

    /// Not supported. Only maintained for compatibility reasons.
    pub strict: Option<bool>,
}

/// Search the web.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[non_exhaustive]
pub struct WebSearchTool {
    /// List of website domains to allow in the search results.
    /// This parameter act as a whitelist where only those websites can be selected.
    /// A maximum of 5 websites can be selected.
    #[cfg_attr(test, schema(max_items = 5))]
    pub allowed_domains: Option<Vec<String>>,

    /// When true, activates the image search server-side tool, enabling the model to search
    /// for images alongside the standard web search.
    pub enable_image_search: Option<bool>,

    /// Enable image understanding during web search.
    pub enable_image_understanding: Option<bool>,

    /// List of website domains to exclude from the search results without protocol specification
    /// or subdomains.
    /// A maximum of 5 websites can be excluded.
    #[cfg_attr(test, schema(max_items = 5))]
    pub excluded_domains: Option<Vec<String>>,

    /// Control whether the web search tool fetches live content or uses only cached content.
    /// For OpenAI API compatibility ONLY. Request will be rejected if this field is set.
    pub external_web_access: Option<bool>,

    /// Filters to apply to the search results. Compatible with OpenAI's API.
    pub filters: Option<WebSearchFilters>,

    /// High level guidance for the amount of context window space to use for the search.
    /// Available values are `"low"`, `"medium"`, or `"high"`.
    /// For OpenAI API compatibility ONLY. Request will be rejected if this field is set.
    pub search_context_size: Option<String>,

    /// The user location to use for the search.
    /// For OpenAI API compatibility ONLY. Request will be rejected if this field is set.
    pub user_location: Option<WebSearchUserLocation>,
}

/// Filters to apply to the web search results.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[non_exhaustive]
pub struct WebSearchFilters {
    /// List of website domains (without protocol specification or subdomains) to restrict search
    /// results to.
    #[cfg_attr(test, schema(max_items = 5))]
    pub allowed_domains: Option<Vec<String>>,

    /// List of website domains (without protocol specification or subdomains) to exclude from
    /// search results.
    #[cfg_attr(test, schema(max_items = 5))]
    pub excluded_domains: Option<Vec<String>>,
}

tagged_enum! {
    /// The user location to use for the search.
    #[skip_serializing_none]
    #[derive(Serialize, Deserialize, Debug, Clone)]
    #[serde(tag = "type", rename_all = "snake_case")]
    pub enum WebSearchUserLocation {
        /// An approximate user location.
        @schema(title = "WebSearchUserLocationApproximate")
        Approximate(WebSearchUserLocationApproximate),
    }
}

/// An approximate user location to use for the search.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[non_exhaustive]
pub struct WebSearchUserLocationApproximate {
    /// City of the user's location.
    pub city: Option<String>,

    /// Two-letter ISO 3166-1 alpha-2 country code, like US, GB, etc.
    pub country: Option<String>,

    /// Region of the user's location.
    pub region: Option<String>,

    /// Timezone of the user's location, IANA timezone like America/Chicago, Europe/London, etc.
    pub timezone: Option<String>,
}

/// Search X.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[non_exhaustive]
pub struct XSearchTool {
    /// List of X Handles of the users from whom to consider the posts.
    #[cfg_attr(test, schema(max_items = 10))]
    pub allowed_x_handles: Option<Vec<String>>,

    /// Enable image understanding during X search.
    pub enable_image_understanding: Option<bool>,

    /// Enable video understanding during X search.
    pub enable_video_understanding: Option<bool>,

    /// List of X Handles of the users from whom to exclude the posts.
    #[cfg_attr(test, schema(max_items = 10))]
    pub excluded_x_handles: Option<Vec<String>>,

    /// Date from which to consider the results in ISO-8601 YYYY-MM-DD.
    #[cfg_attr(test, schema(format = Date))]
    pub from_date: Option<String>,

    /// Date up to which to consider the results in ISO-8601 YYYY-MM-DD.
    #[cfg_attr(test, schema(format = Date))]
    pub to_date: Option<String>,
}

/// Search the knowledge bases.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct FileSearchTool {
    /// List of vector store IDs to search within.
    #[cfg_attr(test, schema(max_items = 10))]
    pub vector_store_ids: Vec<String>,

    /// A filter to apply.
    /// For OpenAI API compatibility ONLY. Request will be rejected if this field is set.
    pub filters: Option<serde_json::Value>,

    /// The maximum number of results to return.
    #[cfg_attr(test, schema(minimum = 1))]
    pub max_num_results: Option<u32>,

    /// Ranking options for search.
    /// For OpenAI API compatibility ONLY. Request will be rejected if this field is set.
    pub ranking_options: Option<serde_json::Value>,
}

/// Execute code.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[non_exhaustive]
pub struct CodeInterpreterTool {
    /// The code interpreter container.
    /// Can be a container ID or an object that specifies uploaded file IDs to make available to
    /// your code.
    /// For OpenAI API compatibility ONLY. Request will be rejected if this field is set.
    pub container: Option<serde_json::Value>,
}

/// A remote MCP server to use.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct McpTool {
    /// The label of the MCP server.
    pub server_label: String,

    /// The URL of the MCP server.
    pub server_url: String,

    /// Allowed tools.
    pub allowed_tools: Option<Vec<String>>,

    /// Authorization.
    pub authorization: Option<String>,

    /// Connector ID.
    pub connector_id: Option<String>,

    /// When true, this server's tool definitions are hidden from the model's prompt but stay
    /// callable, loaded via a `tool_search` step.
    pub defer_loading: Option<bool>,

    /// Headers.
    pub headers: Option<HashMap<String, String>>,

    /// Require approval.
    pub require_approval: Option<String>,

    /// Server description.
    pub server_description: Option<String>,
}

/// Generate images from text prompts.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[non_exhaustive]
pub struct ImageGenerationTool {
    /// Which image capabilities to expose to the model.
    /// One of `"auto"` (the default; both generation and editing), `"generate"`
    /// (text-to-image only), or `"edit"` (image editing only).
    pub action: Option<String>,
}

/// A local shell execution tool.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ShellTool {
    /// The environment configuration for the shell tool.
    pub environment: ShellEnvironment,
}

tagged_enum! {
    /// The environment a shell tool runs in.
    #[derive(Serialize, Deserialize, Debug, Clone)]
    #[serde(tag = "type", rename_all = "snake_case")]
    pub enum ShellEnvironment {
        /// The local environment of the caller.
        @schema(title = "LocalShellEnvironment")
        Local(LocalShellEnvironment),
    }
}

/// The environment configuration for a shell tool.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[non_exhaustive]
pub struct LocalShellEnvironment {
    /// A list of skills available in the local environment.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub skills: Vec<LocalShellSkill>,
}

/// A skill available in the local shell environment.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct LocalShellSkill {
    /// The name of the skill.
    pub name: String,

    /// A description of what the skill does.
    pub description: String,

    /// The path to the directory containing the skill (with a `SKILL.md` file).
    pub path: String,
}

/// Search over deferred tool definitions.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[non_exhaustive]
pub struct ToolSearchTool {
    #[cfg_attr(test, schema(value_type = Option<String>))]
    execution: Option<ToolSearchExecution>,
}

/// Where the search runs. Only `\"server\"` (or unset) is supported.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ToolSearchExecution {
    /// The search runs on the server.
    Server,
    /// A value not represented by this enum, kept as sent for forward compatibility.
    #[serde(untagged)]
    Unknown(String),
}

impl FunctionTool {
    /// Creates a [`FunctionTool`] with the given function name and parameter schema.
    pub fn new(name: impl Into<String>, parameters: serde_json::Value) -> Self {
        Self {
            name: name.into(),
            description: None,
            parameters,
            defer_loading: None,
            strict: None,
        }
    }

    /// Sets the description used to indicate to the model when to call the function.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Sets the JSON schema describing the function parameters.
    pub fn with_parameters(mut self, parameters: serde_json::Value) -> Self {
        self.parameters = parameters;
        self
    }

    /// Sets the [`defer_loading`](Self::defer_loading) field.
    pub fn with_defer_loading(mut self, defer_loading: bool) -> Self {
        self.defer_loading = Some(defer_loading);
        self
    }

    /// Sets the `strict` field (only maintained for compatibility reasons).
    pub fn with_strict(mut self, strict: bool) -> Self {
        self.strict = Some(strict);
        self
    }
}

impl WebSearchTool {
    /// Creates an empty [`WebSearchTool`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the list of website domains to allow in the search results.
    pub fn with_allowed_domains(mut self, allowed_domains: Vec<String>) -> Self {
        self.allowed_domains = Some(allowed_domains);
        self
    }

    /// Enables or disables the image search server-side tool.
    pub fn with_enable_image_search(mut self, enable_image_search: bool) -> Self {
        self.enable_image_search = Some(enable_image_search);
        self
    }

    /// Enables or disables image understanding during web search.
    pub fn with_enable_image_understanding(mut self, enable_image_understanding: bool) -> Self {
        self.enable_image_understanding = Some(enable_image_understanding);
        self
    }

    /// Sets the list of website domains to exclude from the search results.
    pub fn with_excluded_domains(mut self, excluded_domains: Vec<String>) -> Self {
        self.excluded_domains = Some(excluded_domains);
        self
    }

    /// Sets the `external_web_access` field (for OpenAI API compatibility only).
    pub fn with_external_web_access(mut self, external_web_access: bool) -> Self {
        self.external_web_access = Some(external_web_access);
        self
    }

    /// Sets the filters to apply to the search results.
    pub fn with_filters(mut self, filters: WebSearchFilters) -> Self {
        self.filters = Some(filters);
        self
    }

    /// Sets the `search_context_size` field (for OpenAI API compatibility only).
    pub fn with_search_context_size(mut self, search_context_size: impl Into<String>) -> Self {
        self.search_context_size = Some(search_context_size.into());
        self
    }

    /// Sets the user location to use for the search.
    pub fn with_user_location(mut self, user_location: WebSearchUserLocation) -> Self {
        self.user_location = Some(user_location);
        self
    }
}

impl WebSearchFilters {
    /// Creates an empty [`WebSearchFilters`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the list of website domains to restrict search results to.
    pub fn with_allowed_domains(mut self, allowed_domains: Vec<String>) -> Self {
        self.allowed_domains = Some(allowed_domains);
        self
    }

    /// Sets the list of website domains to exclude from search results.
    pub fn with_excluded_domains(mut self, excluded_domains: Vec<String>) -> Self {
        self.excluded_domains = Some(excluded_domains);
        self
    }
}

impl WebSearchUserLocationApproximate {
    /// Creates an empty [`WebSearchUserLocationApproximate`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the city of the user's location.
    pub fn with_city(mut self, city: impl Into<String>) -> Self {
        self.city = Some(city.into());
        self
    }

    /// Sets the two-letter ISO 3166-1 alpha-2 country code.
    pub fn with_country(mut self, country: impl Into<String>) -> Self {
        self.country = Some(country.into());
        self
    }

    /// Sets the region of the user's location.
    pub fn with_region(mut self, region: impl Into<String>) -> Self {
        self.region = Some(region.into());
        self
    }

    /// Sets the timezone of the user's location.
    pub fn with_timezone(mut self, timezone: impl Into<String>) -> Self {
        self.timezone = Some(timezone.into());
        self
    }
}

impl XSearchTool {
    /// Creates an empty [`XSearchTool`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the list of X handles from whom to consider the posts.
    pub fn with_allowed_x_handles(mut self, allowed_x_handles: Vec<String>) -> Self {
        self.allowed_x_handles = Some(allowed_x_handles);
        self
    }

    /// Enables or disables image understanding during X search.
    pub fn with_enable_image_understanding(mut self, enable_image_understanding: bool) -> Self {
        self.enable_image_understanding = Some(enable_image_understanding);
        self
    }

    /// Enables or disables video understanding during X search.
    pub fn with_enable_video_understanding(mut self, enable_video_understanding: bool) -> Self {
        self.enable_video_understanding = Some(enable_video_understanding);
        self
    }

    /// Sets the list of X handles from whom to exclude the posts.
    pub fn with_excluded_x_handles(mut self, excluded_x_handles: Vec<String>) -> Self {
        self.excluded_x_handles = Some(excluded_x_handles);
        self
    }

    /// Sets the date from which to consider the results (ISO-8601 YYYY-MM-DD).
    pub fn with_from_date(mut self, from_date: impl Into<String>) -> Self {
        self.from_date = Some(from_date.into());
        self
    }

    /// Sets the date up to which to consider the results (ISO-8601 YYYY-MM-DD).
    pub fn with_to_date(mut self, to_date: impl Into<String>) -> Self {
        self.to_date = Some(to_date.into());
        self
    }
}

impl FileSearchTool {
    /// Creates a [`FileSearchTool`] that searches within the given vector store IDs.
    pub fn new(vector_store_ids: Vec<String>) -> Self {
        Self {
            vector_store_ids,
            filters: None,
            max_num_results: None,
            ranking_options: None,
        }
    }

    /// Sets the `filters` field (for OpenAI API compatibility only).
    pub fn with_filters(mut self, filters: serde_json::Value) -> Self {
        self.filters = Some(filters);
        self
    }

    /// Sets the maximum number of results to return.
    pub fn with_max_num_results(mut self, max_num_results: u32) -> Self {
        self.max_num_results = Some(max_num_results);
        self
    }

    /// Sets the `ranking_options` field (for OpenAI API compatibility only).
    pub fn with_ranking_options(mut self, ranking_options: serde_json::Value) -> Self {
        self.ranking_options = Some(ranking_options);
        self
    }
}

impl CodeInterpreterTool {
    /// Creates an empty [`CodeInterpreterTool`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the `container` field (for OpenAI API compatibility only).
    pub fn with_container(mut self, container: serde_json::Value) -> Self {
        self.container = Some(container);
        self
    }
}

impl McpTool {
    /// Creates an [`McpTool`] for the MCP server with the given label and URL.
    pub fn new(server_label: impl Into<String>, server_url: impl Into<String>) -> Self {
        Self {
            server_label: server_label.into(),
            server_url: server_url.into(),
            allowed_tools: None,
            authorization: None,
            connector_id: None,
            defer_loading: None,
            headers: None,
            require_approval: None,
            server_description: None,
        }
    }

    /// Sets the list of allowed tools.
    pub fn with_allowed_tools(mut self, allowed_tools: Vec<String>) -> Self {
        self.allowed_tools = Some(allowed_tools);
        self
    }

    /// Sets the authorization value.
    pub fn with_authorization(mut self, authorization: impl Into<String>) -> Self {
        self.authorization = Some(authorization.into());
        self
    }

    /// Sets the connector ID.
    pub fn with_connector_id(mut self, connector_id: impl Into<String>) -> Self {
        self.connector_id = Some(connector_id.into());
        self
    }

    /// Sets the `defer_loading` field.
    pub fn with_defer_loading(mut self, defer_loading: bool) -> Self {
        self.defer_loading = Some(defer_loading);
        self
    }

    /// Sets the headers to send to the MCP server.
    pub fn with_headers(mut self, headers: HashMap<String, String>) -> Self {
        self.headers = Some(headers);
        self
    }

    /// Sets the `require_approval` field.
    pub fn with_require_approval(mut self, require_approval: impl Into<String>) -> Self {
        self.require_approval = Some(require_approval.into());
        self
    }

    /// Sets the server description.
    pub fn with_server_description(mut self, server_description: impl Into<String>) -> Self {
        self.server_description = Some(server_description.into());
        self
    }
}

impl ImageGenerationTool {
    /// Creates an empty [`ImageGenerationTool`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets which image capabilities to expose to the model.
    pub fn with_action(mut self, action: impl Into<String>) -> Self {
        self.action = Some(action.into());
        self
    }
}

impl ShellTool {
    /// Creates a [`ShellTool`] running in the given environment.
    pub fn new(environment: ShellEnvironment) -> Self {
        Self { environment }
    }
}

impl ShellEnvironment {
    /// Creates a local [`ShellEnvironment`] offering the given skills.
    pub fn local(skills: Vec<LocalShellSkill>) -> Self {
        Self::Local(LocalShellEnvironment::new(skills))
    }
}

impl LocalShellEnvironment {
    /// Creates a [`LocalShellEnvironment`] offering the given skills.
    pub fn new(skills: Vec<LocalShellSkill>) -> Self {
        Self { skills }
    }
}

impl LocalShellSkill {
    /// Creates a [`LocalShellSkill`] with the given name, description and path.
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        path: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            path: path.into(),
        }
    }
}

impl ToolSearchTool {
    /// Creates a [`ToolSearchTool`] that runs on the server.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the execution location for the search.
    pub fn with_execution(mut self, execution: ToolSearchExecution) -> Self {
        self.execution = Some(execution);
        self
    }
}
