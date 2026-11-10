use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use super::pair::{paired_block, paired_content};
use crate::claude_messages::types::CacheControlEphemeral;
use crate::utils::string_enum;

/// A bash code execution tool result block within a response message.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct BashCodeExecutionToolResultBlock {
    /// The id of the server tool use that produced this result.
    pub tool_use_id: String,

    /// The content of the tool result.
    pub content: BashCodeExecutionToolResultBlockContent,
}

/// A bash code execution tool result block within a request message.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct BashCodeExecutionToolResultBlockParam {
    /// The id of the server tool use that produced this result.
    pub tool_use_id: String,

    /// The content of the tool result.
    pub content: BashCodeExecutionToolResultBlockParamContent,

    /// Create a cache control breakpoint at this content block.
    pub cache_control: Option<CacheControlEphemeral>,
}

paired_content! {
    /// The content of a result from Claude's internal bash code execution tool.
    BashCodeExecutionToolResultBlockContent as "BashCodeExecutionToolResultBlockContent"
        | BashCodeExecutionToolResultBlockParamContent
            as "BashCodeExecutionToolResultBlockParamContent"
    {
        /// An error that occurred during bash code execution.
        BashCodeExecutionToolResultError(
            BashCodeExecutionToolResultError | BashCodeExecutionToolResultErrorParam
        ),

        /// A successful bash code execution result.
        BashCodeExecutionResult(
            BashCodeExecutionResultBlock | BashCodeExecutionResultBlockParam
        ),
    }
}

paired_block! {
    /// A successful bash code execution result block.
    BashCodeExecutionResultBlock | BashCodeExecutionResultBlockParam {
        /// The files the executed bash code generated.
        content: Vec<BashCodeExecutionResultBlockContentItem>
               | Vec<BashCodeExecutionResultBlockContentItemParam>,

        /// The exit code of the executed bash code.
        #[cfg_attr(test, schema(schema_with = crate::claude_messages::spec::schema::number_schema))]
        return_code: isize,

        /// The standard output of the executed bash code.
        stdout: String,

        /// The standard error of the executed bash code.
        stderr: String,
    }
}

paired_content! {
    /// The content block within a successful bash code execution result. Can only be a bash code
    /// execution output.
    BashCodeExecutionResultBlockContentItem as "BashCodeExecutionResultBlockContentItem"
        | BashCodeExecutionResultBlockContentItemParam
            as "BashCodeExecutionResultBlockContentItemParam"
    {
        /// A bash code execution output.
        BashCodeExecutionOutput(
            BashCodeExecutionOutputBlock | BashCodeExecutionOutputBlockParam
        ),
    }
}

paired_block! {
    /// A bash code execution output block.
    BashCodeExecutionOutputBlock | BashCodeExecutionOutputBlockParam {
        /// The id of the generated file.
        file_id: String,
    }
}

paired_block! {
    /// An error that occurred during bash code execution.
    BashCodeExecutionToolResultError | BashCodeExecutionToolResultErrorParam {
        /// The error code.
        error_code: BashCodeExecutionToolResultErrorCode,
    }
}

string_enum! {
    /// An error code from the bash code execution tool.
    #[serde(rename_all = "snake_case")]
    pub enum BashCodeExecutionToolResultErrorCode {
        /// `"invalid_tool_input"`
        InvalidToolInput,

        /// `"unavailable"`
        Unavailable,

        /// `"too_many_requests"`
        TooManyRequests,

        /// `"execution_time_exceeded"`
        ExecutionTimeExceeded,

        /// `"output_file_too_large"`
        OutputFileTooLarge,
    }
}

impl BashCodeExecutionToolResultBlock {
    /// Creates a new [`BashCodeExecutionToolResultBlock`] from the given tool use id and content.
    pub fn new(
        tool_use_id: impl Into<String>,
        content: BashCodeExecutionToolResultBlockContent,
    ) -> Self {
        Self {
            tool_use_id: tool_use_id.into(),
            content,
        }
    }
}

impl BashCodeExecutionToolResultBlockParam {
    /// Creates a new [`BashCodeExecutionToolResultBlockParam`] from the given tool use id and
    /// content.
    pub fn new(
        tool_use_id: impl Into<String>,
        content: BashCodeExecutionToolResultBlockParamContent,
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

impl From<BashCodeExecutionToolResultBlock> for BashCodeExecutionToolResultBlockParam {
    fn from(value: BashCodeExecutionToolResultBlock) -> Self {
        Self {
            tool_use_id: value.tool_use_id,
            content: value.content.into(),
            cache_control: None,
        }
    }
}
