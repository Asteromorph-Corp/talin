use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use crate::utils::tagged_enum;
use crate::xai_responses::types::{Reasoning, Role, ShellEnvironment, ToolSearchExecution};

tagged_enum! {
    /// The output from the model.
    #[derive(Serialize, Deserialize, Debug, Clone)]
    #[serde(tag = "type", rename_all = "snake_case")]
    pub enum ModelOutput {
        /// An output message from the model.
        @schema(title = "OutputMessage")
        Message(OutputMessage),
        /// A tool call to run a function.
        @schema(title = "FunctionToolCall")
        FunctionCall(FunctionToolCall),
        /// The reasoning done by the model.
        @schema(title = "Reasoning")
        Reasoning(Reasoning),
        /// The output of a web search tool call.
        @schema(title = "WebSearchCall")
        WebSearchCall(WebSearchCall),
        /// The output of a file search tool call.
        @schema(title = "FileSearchCall")
        FileSearchCall(FileSearchCall),
        /// The output of a code interpreter tool call.
        @schema(title = "CodeInterpreterCall")
        CodeInterpreterCall(CodeInterpreterCall),
        /// The output of a MCP tool call.
        @schema(title = "McpCall")
        McpCall(McpCall),
        /// The output of a custom tool call.
        @schema(title = "CustomToolCall")
        CustomToolCall(CustomToolCall),
        /// A shell command call emitted by the model for local execution.
        @schema(title = "ShellCall")
        ShellCall(ShellCall),
        /// The output of an image generation tool call.
        @schema(title = "ImageGenerationCall")
        ImageGenerationCall(ImageGenerationCall),
        /// A server-side tool search call made by the model.
        ToolSearchCall(ToolSearchCall),
        /// The tool definitions loaded by a completed tool search call.
        ToolSearchOutput(ToolSearchOutput),
    }
}

/// An output message from the model.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct OutputMessage {
    /// Content of the output message.
    pub content: Vec<OutputMessageContent>,

    /// The unique ID of the output message.
    #[cfg_attr(test, schema(nullable = false))]
    pub id: Option<String>,

    /// The role of the output message.
    #[cfg_attr(test, schema(value_type = String))]
    pub role: Role,

    /// Status of the item.
    #[cfg_attr(test, schema(value_type = Option<String>, nullable = false))]
    pub status: Option<ItemStatus>,
}

/// Status of an item.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ItemStatus {
    /// Completed.
    Completed,
    /// In progress.
    InProgress,
    /// Incomplete.
    Incomplete,

    /// A value not represented by this enum, kept as sent for forward compatibility.
    #[serde(untagged)]
    Unknown(String),
}

tagged_enum! {
    /// Content of the output message.
    #[derive(Serialize, Deserialize, Debug, Clone)]
    #[serde(tag = "type", rename_all = "snake_case")]
    pub enum OutputMessageContent {
        /// Text output.
        OutputText(OutputTextContent),
        /// Refusal.
        Refusal(RefusalContent),
    }
}

/// Text output from the model.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct OutputTextContent {
    /// The text output from the model.
    pub text: String,

    /// Citations.
    #[cfg_attr(test, schema(nullable = false))]
    pub annotations: Option<Vec<Annotation>>,

    /// The log probabilities.
    #[cfg_attr(test, schema(nullable = false))]
    pub logprobs: Option<Vec<TokenLogProb>>,
}

/// A refusal returned by the model instead of an output text.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct RefusalContent {
    /// The reason for the refusal.
    pub refusal: String,
}

/// A citation annotating the text output.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct Annotation {
    /// The start index of the annotation.
    pub start_index: Option<i32>,

    /// The end index of the annotation.
    pub end_index: Option<i32>,

    /// The title of the annotation.
    pub title: Option<String>,

    /// The type of the annotation. Only supported type currently is `"url_citation"`.
    #[serde(rename = "type")]
    pub type_: String,

    /// The URL of the web resource.
    pub url: String,
}

/// The log probability of an output token.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct TokenLogProb {
    /// The token.
    pub token: String,

    /// The log probability.
    pub logprob: f32,

    /// The ASCII encoding of the output character.
    pub bytes: Option<Vec<u8>>,

    /// An array of the most likely tokens to return at this token position.
    pub top_logprobs: Vec<TopLogProb>,
}

/// The log probability of one of the most likely tokens at a token position.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct TopLogProb {
    /// The token.
    pub token: String,

    /// The log probability.
    pub logprob: f32,

    /// The ASCII encoding of the output character.
    pub bytes: Option<Vec<u8>>,
}

/// A tool call to run a function.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct FunctionToolCall {
    /// The unique ID of the function tool call generated by the model.
    pub call_id: String,

    /// The arguments to pass to the function, as a JSON string.
    pub arguments: String,

    /// The unique ID of the function tool call.
    #[cfg_attr(test, schema(nullable = false))]
    pub id: Option<String>,

    /// The name of the function.
    pub name: String,

    /// Status of the item.
    #[cfg_attr(test, schema(value_type = Option<String>, nullable = false))]
    pub status: Option<ItemStatus>,
}

/// The output of a web search tool call.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct WebSearchCall {
    /// An object describing the specific action taken in this web search call.
    pub action: WebSearchAction,

    /// The unique ID of the web search tool call.
    #[cfg_attr(test, schema(nullable = false))]
    pub id: Option<String>,

    /// The status of the web search tool call.
    #[cfg_attr(test, schema(value_type = Option<String>, nullable = false))]
    pub status: Option<ItemStatus>,
}

tagged_enum! {
    /// Web search action.
    #[derive(Serialize, Deserialize, Debug, Clone)]
    #[serde(tag = "type", rename_all = "snake_case")]
    pub enum WebSearchAction {
        /// Performs a web search query.
        Search(WebSearchActionSearch),
        /// Opens a specific URL from search results.
        OpenPage(WebSearchActionOpenPage),
        /// Searches for a pattern within a loaded page.
        #[serde(rename = "find_in_page")]
        Find(WebSearchActionFind),
    }
}

/// A web search action that performs a search query.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct WebSearchActionSearch {
    /// The search query.
    pub query: String,
    /// The sources used in the search.
    #[cfg_attr(test, schema(nullable = false))]
    pub sources: Option<Vec<WebSearchActionSource>>,
}

/// A web search action that opens a specific URL from search results.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct WebSearchActionOpenPage {
    /// The URL of the page to open.
    pub url: String,
}

/// A web search action that searches for a pattern within a loaded page.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct WebSearchActionFind {
    /// The URL of the page being searched within.
    pub url: String,
    /// The pattern or text to search for within the page.
    pub pattern: String,
}

/// A source used in a web search action.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct WebSearchActionSource {
    /// The type of source.
    #[serde(rename = "type")]
    pub type_: String,
    /// The URL of the source.
    #[cfg_attr(test, schema(nullable = false))]
    pub url: Option<String>,
}

/// The output of a file search tool call.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct FileSearchCall {
    /// The unique ID of the file search tool call.
    #[cfg_attr(test, schema(nullable = false))]
    pub id: Option<String>,

    /// The queries used to search for files.
    pub queries: Vec<String>,

    /// The results of the file search tool call.
    pub results: Vec<FileSearchResult>,

    /// The status of the file search tool call.
    #[cfg_attr(test, schema(value_type = Option<String>, nullable = false))]
    pub status: Option<ItemStatus>,
}

/// A result of a file search tool call.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct FileSearchResult {
    /// The file ID of the file search result.
    pub file_id: String,
    /// The filename of the file search result.
    pub filename: String,
    /// The score of the file search result.
    #[cfg_attr(test, schema(nullable = false))]
    pub score: Option<f64>,
    /// The text of the file search result.
    pub text: String,
}

/// The output of a code interpreter tool call.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct CodeInterpreterCall {
    /// The code of the code interpreter tool call.
    #[cfg_attr(test, schema(nullable = false))]
    pub code: Option<String>,

    /// The unique ID of the code interpreter tool call.
    #[cfg_attr(test, schema(nullable = false))]
    pub id: Option<String>,

    /// The outputs of the code interpreter tool call.
    pub outputs: Vec<CodeInterpreterOutput>,

    /// The status of the code interpreter tool call.
    #[cfg_attr(test, schema(value_type = Option<String>, nullable = false))]
    pub status: Option<ItemStatus>,
}

tagged_enum! {
    /// The output of a code interpreter tool call.
    #[derive(Serialize, Deserialize, Debug, Clone)]
    #[serde(tag = "type", rename_all = "snake_case")]
    pub enum CodeInterpreterOutput {
        /// Logs output.
        Logs(CodeInterpreterOutputLogs),
        /// Image output.
        Image(CodeInterpreterOutputImage),
    }
}

/// The logs output of a code interpreter tool call.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct CodeInterpreterOutputLogs {
    /// The output of the code interpreter tool call.
    pub logs: String,
}

/// The image output of a code interpreter tool call.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct CodeInterpreterOutputImage {
    /// The URL of the image.
    pub url: String,
}

/// The output of a MCP tool call.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct McpCall {
    /// A JSON string of the arguments passed to the tool.
    pub arguments: String,

    /// The error message of the MCP tool call.
    #[cfg_attr(test, schema(nullable = false))]
    pub error: Option<String>,

    /// The unique ID of the MCP tool call.
    #[cfg_attr(test, schema(nullable = false))]
    pub id: Option<String>,

    /// The name of the tool that was run.
    pub name: String,

    /// The output of the MCP tool call.
    pub output: String,

    /// The label of the MCP server running the tool.
    pub server_label: String,

    /// The status of the MCP tool call.
    #[cfg_attr(test, schema(value_type = Option<String>, nullable = false))]
    pub status: Option<ItemStatus>,
}

/// The output of a custom tool call.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct CustomToolCall {
    /// The unique ID of the custom tool call generated by the model.
    pub call_id: String,

    /// The unique ID of the custom tool call item.
    pub id: String,

    /// The input for the custom tool call generated by the model.
    #[cfg_attr(test, schema(nullable = false))]
    pub input: Option<String>,

    /// An identifier used to map this custom tool call to a tool call output.
    pub name: String,

    /// Status of the item.
    #[cfg_attr(test, schema(value_type = Option<String>, nullable = false))]
    pub status: Option<ItemStatus>,
}

/// A shell command call emitted by the model for local execution.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ShellCall {
    /// The shell commands and limits that describe how to run the tool call.
    pub action: ShellCallAction,

    /// The unique ID of the shell tool call generated by the model.
    pub call_id: String,

    /// The environment for the shell tool call.
    pub environment: Option<ShellEnvironment>,

    /// The unique ID of the shell call item.
    #[cfg_attr(test, schema(nullable = false))]
    pub id: Option<String>,

    /// Status of the item.
    #[cfg_attr(test, schema(value_type = Option<String>, nullable = false))]
    pub status: Option<ItemStatus>,
}

tagged_enum! {
    /// The action a shell tool call performs.
    #[skip_serializing_none]
    #[derive(Serialize, Deserialize, Debug, Clone)]
    #[serde(tag = "type", rename_all = "snake_case")]
    pub enum ShellCallAction {
        /// Executes shell commands.
        @schema(title = "ShellCallActionExec")
        Exec(ShellCallActionExec),
    }
}

/// The shell commands and limits that describe how to run a shell tool call.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ShellCallActionExec {
    /// The commands to run.
    pub commands: Vec<String>,

    /// Maximum number of characters to return from each command.
    #[cfg_attr(test, schema(value_type = Option<i32>))]
    pub max_output_length: Option<u32>,

    /// Timeout in milliseconds for the commands.
    pub timeout_ms: Option<i64>,
}

/// The output of an image generation tool call.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[non_exhaustive]
pub struct ImageGenerationCall {
    /// The unique ID of the image generation tool call.
    #[cfg_attr(test, schema(nullable = false))]
    pub id: Option<String>,

    /// The prompt used to generate the image.
    pub prompt: Option<String>,

    /// The generated image encoded in base64 (bare base64, no data-URL prefix), or `None`
    /// while the call is still in progress or has failed.
    /// The image format can be determined from the decoded magic bytes.
    pub result: Option<String>,

    /// The status of the image generation tool call.
    #[cfg_attr(test, schema(value_type = Option<String>, nullable = false))]
    pub status: Option<ImageGenerationStatus>,
}

/// The status of an image generation tool call.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ImageGenerationStatus {
    /// In progress.
    InProgress,
    /// Generating.
    Generating,
    /// Completed.
    Completed,
    /// Failed.
    Failed,

    /// A value not represented by this enum, kept as sent for forward compatibility.
    #[serde(untagged)]
    Unknown(String),
}

/// A server-side tool search call made by the model.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ToolSearchCall {
    /// The unique ID of the tool search call item.
    pub id: String,
    /// Always `null`; present for OpenAI compatibility.
    pub call_id: Option<String>,
    /// Where the search runs. Always `server`.
    #[cfg_attr(test, schema(value_type = Option<String>, nullable = false))]
    pub execution: Option<ToolSearchExecution>,
    /// Status of the item. One of `in_progress`, `completed` or `incomplete` (a failed search).
    #[cfg_attr(test, schema(value_type = Option<String>, nullable = false))]
    pub status: Option<ToolSearchStatus>,
    /// The arguments of the tool search call: `{\"query\": ..., \"limit\": ...}`.
    #[cfg_attr(test, schema(nullable = false))]
    pub arguments: Option<serde_json::Value>,
}

/// The output of a server-side tool search.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct ToolSearchOutput {
    /// The unique ID of the tool search call item.
    pub id: String,
    /// Always `null`; present for OpenAI compatibility.
    pub call_id: Option<String>,
    /// Where the search runs. Always `server`.
    #[cfg_attr(test, schema(value_type = Option<String>, nullable = false))]
    pub execution: Option<ToolSearchExecution>,
    /// Status of the item. Always `completed`.
    #[cfg_attr(test, schema(value_type = Option<String>, nullable = false))]
    pub status: Option<ToolSearchStatus>,
    /// The loaded tool definitions, echoed in the shape they were supplied in.
    #[cfg_attr(test, schema(nullable = false))]
    pub tools: Option<Vec<serde_json::Value>>,
}

/// The status of a tool search call.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ToolSearchStatus {
    /// `in_progress`
    InProgress,
    /// `completed`
    Completed,
    /// `incomplete`
    Incomplete,
    /// A value not represented by this enum, kept as sent for forward compatibility.
    #[serde(untagged)]
    Unknown(String),
}

impl OutputMessage {
    /// Creates an [`OutputMessage`] with the given content and role.
    pub fn new(content: Vec<OutputMessageContent>, role: Role) -> Self {
        Self {
            content,
            id: None,
            role,
            status: None,
        }
    }

    /// Sets the unique ID of the output message.
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Sets the status of the item.
    pub fn with_status(mut self, status: ItemStatus) -> Self {
        self.status = Some(status);
        self
    }
}

impl OutputTextContent {
    /// Creates an [`OutputTextContent`] with the given text.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            annotations: None,
            logprobs: None,
        }
    }

    /// Sets the citations of the text output.
    pub fn with_annotations(mut self, annotations: Vec<Annotation>) -> Self {
        self.annotations = Some(annotations);
        self
    }

    /// Sets the log probabilities of the text output.
    pub fn with_logprobs(mut self, logprobs: Vec<TokenLogProb>) -> Self {
        self.logprobs = Some(logprobs);
        self
    }
}

impl RefusalContent {
    /// Creates a [`RefusalContent`] with the given refusal reason.
    pub fn new(refusal: impl Into<String>) -> Self {
        Self {
            refusal: refusal.into(),
        }
    }
}

impl Annotation {
    /// Creates an [`Annotation`] of the given type pointing at the given URL.
    pub fn new(type_: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            start_index: None,
            end_index: None,
            title: None,
            type_: type_.into(),
            url: url.into(),
        }
    }

    /// Sets the start index of the annotation.
    pub fn with_start_index(mut self, start_index: i32) -> Self {
        self.start_index = Some(start_index);
        self
    }

    /// Sets the end index of the annotation.
    pub fn with_end_index(mut self, end_index: i32) -> Self {
        self.end_index = Some(end_index);
        self
    }

    /// Sets the title of the annotation.
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }
}

impl TokenLogProb {
    /// Creates a [`TokenLogProb`] for the given token, log probability and alternatives.
    pub fn new(token: impl Into<String>, logprob: f32, top_logprobs: Vec<TopLogProb>) -> Self {
        Self {
            token: token.into(),
            logprob,
            bytes: None,
            top_logprobs,
        }
    }

    /// Sets the ASCII encoding of the output character.
    pub fn with_bytes(mut self, bytes: Vec<u8>) -> Self {
        self.bytes = Some(bytes);
        self
    }
}

impl TopLogProb {
    /// Creates a [`TopLogProb`] for the given token and log probability.
    pub fn new(token: impl Into<String>, logprob: f32) -> Self {
        Self {
            token: token.into(),
            logprob,
            bytes: None,
        }
    }

    /// Sets the ASCII encoding of the output character.
    pub fn with_bytes(mut self, bytes: Vec<u8>) -> Self {
        self.bytes = Some(bytes);
        self
    }
}

impl FunctionToolCall {
    /// Creates a [`FunctionToolCall`] with the given call ID, arguments and function name.
    pub fn new(
        call_id: impl Into<String>,
        arguments: impl Into<String>,
        name: impl Into<String>,
    ) -> Self {
        Self {
            call_id: call_id.into(),
            arguments: arguments.into(),
            id: None,
            name: name.into(),
            status: None,
        }
    }

    /// Sets the id of the item.
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Sets the status of the item.
    pub fn with_status(mut self, status: ItemStatus) -> Self {
        self.status = Some(status);
        self
    }
}

impl WebSearchCall {
    /// Creates a [`WebSearchCall`] describing the given action.
    pub fn new(action: WebSearchAction) -> Self {
        Self {
            action,
            id: None,
            status: None,
        }
    }

    /// Sets the unique ID of the web search tool call.
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Sets the status of the web search tool call.
    pub fn with_status(mut self, status: ItemStatus) -> Self {
        self.status = Some(status);
        self
    }
}

impl WebSearchActionSearch {
    /// Creates a [`WebSearchActionSearch`] for the given query.
    pub fn new(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            sources: None,
        }
    }

    /// Sets the sources used in the search.
    pub fn with_sources(mut self, sources: Vec<WebSearchActionSource>) -> Self {
        self.sources = Some(sources);
        self
    }
}

impl WebSearchActionOpenPage {
    /// Creates a [`WebSearchActionOpenPage`] for the given URL.
    pub fn new(url: impl Into<String>) -> Self {
        Self { url: url.into() }
    }
}

impl WebSearchActionFind {
    /// Creates a [`WebSearchActionFind`] searching the given page for the given pattern.
    pub fn new(url: impl Into<String>, pattern: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            pattern: pattern.into(),
        }
    }
}

impl WebSearchActionSource {
    /// Creates a [`WebSearchActionSource`] of the given type.
    pub fn new(type_: impl Into<String>) -> Self {
        Self {
            type_: type_.into(),
            url: None,
        }
    }

    /// Sets the URL of the source.
    pub fn with_url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }
}

impl FileSearchCall {
    /// Creates a [`FileSearchCall`] with the given queries and results.
    pub fn new(queries: Vec<String>, results: Vec<FileSearchResult>) -> Self {
        Self {
            id: None,
            queries,
            results,
            status: None,
        }
    }

    /// Sets the unique ID of the file search tool call.
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Sets the status of the file search tool call.
    pub fn with_status(mut self, status: ItemStatus) -> Self {
        self.status = Some(status);
        self
    }
}

impl FileSearchResult {
    /// Creates a [`FileSearchResult`] for the given file and matched text.
    pub fn new(
        file_id: impl Into<String>,
        filename: impl Into<String>,
        text: impl Into<String>,
    ) -> Self {
        Self {
            file_id: file_id.into(),
            filename: filename.into(),
            score: None,
            text: text.into(),
        }
    }

    /// Sets the score of the file search result.
    pub fn with_score(mut self, score: f64) -> Self {
        self.score = Some(score);
        self
    }
}

impl CodeInterpreterCall {
    /// Creates a [`CodeInterpreterCall`] with the given outputs.
    pub fn new(outputs: Vec<CodeInterpreterOutput>) -> Self {
        Self {
            code: None,
            id: None,
            outputs,
            status: None,
        }
    }

    /// Sets the code of the code interpreter tool call.
    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }

    /// Sets the unique ID of the code interpreter tool call.
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Sets the status of the code interpreter tool call.
    pub fn with_status(mut self, status: ItemStatus) -> Self {
        self.status = Some(status);
        self
    }
}

impl CodeInterpreterOutputLogs {
    /// Creates a [`CodeInterpreterOutputLogs`] carrying the given logs.
    pub fn new(logs: impl Into<String>) -> Self {
        Self { logs: logs.into() }
    }
}

impl CodeInterpreterOutputImage {
    /// Creates a [`CodeInterpreterOutputImage`] pointing at the given image URL.
    pub fn new(url: impl Into<String>) -> Self {
        Self { url: url.into() }
    }
}

impl McpCall {
    /// Creates an [`McpCall`] for the given tool, arguments and output.
    pub fn new(
        arguments: impl Into<String>,
        name: impl Into<String>,
        output: impl Into<String>,
        server_label: impl Into<String>,
    ) -> Self {
        Self {
            arguments: arguments.into(),
            error: None,
            id: None,
            name: name.into(),
            output: output.into(),
            server_label: server_label.into(),
            status: None,
        }
    }

    /// Sets the error message of the MCP tool call.
    pub fn with_error(mut self, error: impl Into<String>) -> Self {
        self.error = Some(error.into());
        self
    }

    /// Sets the unique ID of the MCP tool call.
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Sets the status of the MCP tool call.
    pub fn with_status(mut self, status: ItemStatus) -> Self {
        self.status = Some(status);
        self
    }
}

impl CustomToolCall {
    /// Creates a [`CustomToolCall`] with the given call ID, item ID and tool name.
    pub fn new(call_id: impl Into<String>, id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            call_id: call_id.into(),
            id: id.into(),
            input: None,
            name: name.into(),
            status: None,
        }
    }

    /// Sets the input for the custom tool call.
    pub fn with_input(mut self, input: impl Into<String>) -> Self {
        self.input = Some(input.into());
        self
    }

    /// Sets the status of the item.
    pub fn with_status(mut self, status: ItemStatus) -> Self {
        self.status = Some(status);
        self
    }
}

impl ShellCall {
    /// Creates a [`ShellCall`] running the given action for the given call ID.
    pub fn new(action: ShellCallAction, call_id: impl Into<String>) -> Self {
        Self {
            action,
            call_id: call_id.into(),
            environment: None,
            id: None,
            status: None,
        }
    }

    /// Sets the environment for the shell tool call.
    pub fn with_environment(mut self, environment: ShellEnvironment) -> Self {
        self.environment = Some(environment);
        self
    }

    /// Sets the unique ID of the shell call item.
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Sets the status of the item.
    pub fn with_status(mut self, status: ItemStatus) -> Self {
        self.status = Some(status);
        self
    }
}

impl ShellCallAction {
    /// Creates a [`ShellCallAction`] executing the given commands.
    pub fn exec(commands: Vec<String>) -> Self {
        Self::Exec(ShellCallActionExec::new(commands))
    }
}

impl ShellCallActionExec {
    /// Creates a [`ShellCallActionExec`] running the given commands.
    pub fn new(commands: Vec<String>) -> Self {
        Self {
            commands,
            max_output_length: None,
            timeout_ms: None,
        }
    }

    /// Sets the maximum number of characters to return from each command.
    pub fn with_max_output_length(mut self, max_output_length: u32) -> Self {
        self.max_output_length = Some(max_output_length);
        self
    }

    /// Sets the timeout in milliseconds for the commands.
    pub fn with_timeout_ms(mut self, timeout_ms: i64) -> Self {
        self.timeout_ms = Some(timeout_ms);
        self
    }
}

impl ImageGenerationCall {
    /// Creates an empty [`ImageGenerationCall`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the unique ID of the image generation tool call.
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Sets the prompt used to generate the image.
    pub fn with_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.prompt = Some(prompt.into());
        self
    }

    /// Sets the generated image, encoded as bare base64.
    pub fn with_result(mut self, result: impl Into<String>) -> Self {
        self.result = Some(result.into());
        self
    }

    /// Sets the status of the image generation tool call.
    pub fn with_status(mut self, status: ImageGenerationStatus) -> Self {
        self.status = Some(status);
        self
    }
}

impl ToolSearchCall {
    /// Creates a [`ToolSearchCall`] with the given id.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            call_id: None,
            status: None,
            execution: Some(ToolSearchExecution::Server),
            arguments: None,
        }
    }

    /// Sets the `call_id` of the tool search call.
    ///
    /// `call_id` is documented as always being `null` in the API spec, update `id` instead.
    pub fn with_call_id(mut self, call_id: impl Into<String>) -> Self {
        self.call_id = Some(call_id.into());
        self
    }

    /// Sets the status of the tool search call.
    pub fn with_status(mut self, status: ToolSearchStatus) -> Self {
        self.status = Some(status);
        self
    }

    /// Sets the execution of the tool search call.
    pub fn with_execution(mut self, execution: ToolSearchExecution) -> Self {
        self.execution = Some(execution);
        self
    }

    /// Sets the arguments of the tool search call.
    pub fn with_arguments(mut self, arguments: serde_json::Value) -> Self {
        self.arguments = Some(arguments);
        self
    }
}

impl ToolSearchOutput {
    /// Creates a [`ToolSearchOutput`] with the given id.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            call_id: None,
            status: Some(ToolSearchStatus::Completed),
            execution: Some(ToolSearchExecution::Server),
            tools: None,
        }
    }

    /// Sets the `call_id` of the tool search output.
    ///
    /// `call_id` is documented as always being `null` in the API spec, update `id` instead.
    pub fn with_call_id(mut self, call_id: impl Into<String>) -> Self {
        self.call_id = Some(call_id.into());
        self
    }

    /// Sets the status of the tool search output.
    pub fn with_status(mut self, status: ToolSearchStatus) -> Self {
        self.status = Some(status);
        self
    }

    /// Sets the execution of the tool search output.
    pub fn with_execution(mut self, execution: ToolSearchExecution) -> Self {
        self.execution = Some(execution);
        self
    }

    /// Sets the tools of the tool search output.
    pub fn with_tools(mut self, tools: Vec<serde_json::Value>) -> Self {
        self.tools = Some(tools);
        self
    }
}
