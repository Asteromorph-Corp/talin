use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use super::pair::{paired_block, paired_content};
use crate::claude_messages::types::CacheControlEphemeral;
use crate::utils::string_enum;

/// A text editor code execution tool result block within a response message.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct TextEditorCodeExecutionToolResultBlock {
    /// The id of the server tool use that produced this result.
    pub tool_use_id: String,

    /// The content of the tool result.
    pub content: TextEditorCodeExecutionToolResultBlockContent,
}

/// A text editor code execution tool result block within a request message.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct TextEditorCodeExecutionToolResultBlockParam {
    /// The id of the server tool use that produced this result.
    pub tool_use_id: String,

    /// The content of the tool result.
    pub content: TextEditorCodeExecutionToolResultBlockParamContent,

    /// Create a cache control breakpoint at this content block.
    pub cache_control: Option<CacheControlEphemeral>,
}

paired_content! {
    /// The content of a result from Claude's internal text editor code execution tool.
    TextEditorCodeExecutionToolResultBlockContent as "TextEditorCodeExecutionToolResultBlockContent"
        | TextEditorCodeExecutionToolResultBlockParamContent
            as "TextEditorCodeExecutionToolResultBlockParamContent"
    {
        /// An error that occurred during text editor code execution.
        TextEditorCodeExecutionToolResultError(
            TextEditorCodeExecutionToolResultError | TextEditorCodeExecutionToolResultErrorParam
        ),

        /// A successful file view result.
        TextEditorCodeExecutionViewResult(
            TextEditorCodeExecutionViewResultBlock | TextEditorCodeExecutionViewResultBlockParam
        ),

        /// A successful file creation result.
        TextEditorCodeExecutionCreateResult(
            TextEditorCodeExecutionCreateResultBlock
                | TextEditorCodeExecutionCreateResultBlockParam
        ),

        /// A successful string replacement result.
        TextEditorCodeExecutionStrReplaceResult(
            TextEditorCodeExecutionStrReplaceResultBlock
                | TextEditorCodeExecutionStrReplaceResultBlockParam
        ),
    }
}

paired_block! {
    /// A successful file view result block.
    TextEditorCodeExecutionViewResultBlock | TextEditorCodeExecutionViewResultBlockParam {
        /// The contents viewed from the file.
        content: String,

        /// The type of the file. e.g. "text", "image", "pdf"
        file_type: FileType,

        /// The starting line number of the viewed content.
        #[cfg_attr(test, schema(schema_with = crate::claude_messages::spec::schema::nullable_number_schema))]
        start_line: Option<usize> with with_start_line,

        /// The number of lines included in the view.
        #[cfg_attr(test, schema(schema_with = crate::claude_messages::spec::schema::nullable_number_schema))]
        num_lines: Option<usize> with with_num_lines,

        /// The total number of lines in the file.
        #[cfg_attr(test, schema(schema_with = crate::claude_messages::spec::schema::nullable_number_schema))]
        total_lines: Option<usize> with with_total_lines,
    }
}

paired_block! {
    /// A successful file creation result block.
    TextEditorCodeExecutionCreateResultBlock | TextEditorCodeExecutionCreateResultBlockParam {
        /// Whether the file was updated instead of newly created.
        is_file_update: bool,
    }
}

paired_block! {
    /// A successful string replacement result block.
    TextEditorCodeExecutionStrReplaceResultBlock
        | TextEditorCodeExecutionStrReplaceResultBlockParam {
        /// The lines that were changed.
        lines: Option<Vec<String>> with with_lines,

        /// The starting line number of the old content.
        #[cfg_attr(test, schema(schema_with = crate::claude_messages::spec::schema::nullable_number_schema))]
        old_start: Option<usize> with with_old_start,

        /// The number of old lines replaced.
        #[cfg_attr(test, schema(schema_with = crate::claude_messages::spec::schema::nullable_number_schema))]
        old_lines: Option<usize> with with_old_lines,

        /// The starting line number of the new content.
        #[cfg_attr(test, schema(schema_with = crate::claude_messages::spec::schema::nullable_number_schema))]
        new_start: Option<usize> with with_new_start,

        /// The number of new lines added.
        #[cfg_attr(test, schema(schema_with = crate::claude_messages::spec::schema::nullable_number_schema))]
        new_lines: Option<usize> with with_new_lines,
    }
}

paired_block! {
    /// An error that occurred during text editor code execution.
    TextEditorCodeExecutionToolResultError | TextEditorCodeExecutionToolResultErrorParam {
        /// The error code.
        error_code: TextEditorCodeExecutionToolResultErrorCode,

        /// The error message.
        error_message: Option<String> with with_error_message,
    }
}

string_enum! {
    /// A file type.
    #[serde(rename_all = "snake_case")]
    pub enum FileType {
        /// `"text"`
        Text,

        /// `"image"`
        Image,

        /// `"pdf"`
        Pdf,
    }
}

string_enum! {
    /// An error code from the text editor code execution tool.
    #[serde(rename_all = "snake_case")]
    pub enum TextEditorCodeExecutionToolResultErrorCode {
        /// `"invalid_tool_input"`
        InvalidToolInput,

        /// `"unavailable"`
        Unavailable,

        /// `"too_many_requests"`
        TooManyRequests,

        /// `"execution_time_exceeded"`
        ExecutionTimeExceeded,

        /// `"file_not_found"`
        FileNotFound,
    }
}

impl TextEditorCodeExecutionToolResultBlock {
    /// Creates a new [`TextEditorCodeExecutionToolResultBlock`] from the given tool use id and
    /// content.
    pub fn new(
        tool_use_id: impl Into<String>,
        content: TextEditorCodeExecutionToolResultBlockContent,
    ) -> Self {
        Self {
            tool_use_id: tool_use_id.into(),
            content,
        }
    }
}

impl TextEditorCodeExecutionToolResultBlockParam {
    /// Creates a new [`TextEditorCodeExecutionToolResultBlockParam`] from the given tool use id
    /// and content.
    pub fn new(
        tool_use_id: impl Into<String>,
        content: TextEditorCodeExecutionToolResultBlockParamContent,
    ) -> Self {
        Self {
            tool_use_id: tool_use_id.into(),
            content,
            cache_control: None,
        }
    }

    /// Sets the [`cache_control`](Self::cache_control) breakpoint for this content block.
    pub fn with_cache_control(mut self, cache_control: CacheControlEphemeral) -> Self {
        self.cache_control = Some(cache_control);
        self
    }
}

impl From<TextEditorCodeExecutionToolResultBlock> for TextEditorCodeExecutionToolResultBlockParam {
    fn from(value: TextEditorCodeExecutionToolResultBlock) -> Self {
        Self {
            tool_use_id: value.tool_use_id,
            content: value.content.into(),
            cache_control: None,
        }
    }
}
