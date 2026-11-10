use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use super::pair::{paired_block, paired_content};
use crate::claude_messages::types::CacheControlEphemeral;
use crate::utils::string_enum;

/// A code execution tool result block within a response message.
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct CodeExecutionToolResultBlock {
    /// The id of the server tool use that produced this result.
    pub tool_use_id: String,

    /// The content of the tool result.
    pub content: CodeExecutionToolResultBlockContent,
}

/// A code execution tool result block within a request message.
#[skip_serializing_none]
#[cfg_attr(test, derive(utoipa::ToSchema))]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct CodeExecutionToolResultBlockParam {
    /// The id of the server tool use that produced this result.
    pub tool_use_id: String,

    /// The content of the tool result.
    pub content: CodeExecutionToolResultBlockParamContent,

    /// Create a cache control breakpoint at this content block.
    pub cache_control: Option<CacheControlEphemeral>,
}

paired_content! {
    /// The content of a result from Claude's internal code execution tool.
    CodeExecutionToolResultBlockContent as "CodeExecutionToolResultBlockContent"
        | CodeExecutionToolResultBlockParamContent
            as "CodeExecutionToolResultBlockParamContent"
    {
        /// An error that occurred during code execution.
        CodeExecutionToolResultError(
            CodeExecutionToolResultError | CodeExecutionToolResultErrorParam
        ),

        /// A successful code execution result.
        CodeExecutionResult(CodeExecutionResultBlock | CodeExecutionResultBlockParam),

        /// A successful code execution result whose standard output is encrypted.
        EncryptedCodeExecutionResult(
            EncryptedCodeExecutionResultBlock | EncryptedCodeExecutionResultBlockParam
        ),
    }
}

paired_block! {
    /// A successful code execution result block.
    CodeExecutionResultBlock | CodeExecutionResultBlockParam {
        /// The files the executed code generated.
        content: Vec<CodeExecutionResultBlockContentItem>
               | Vec<CodeExecutionResultBlockContentItemParam>,

        /// The exit code of the executed code.
        #[cfg_attr(test, schema(schema_with = crate::claude_messages::spec::schema::number_schema))]
        return_code: isize,

        /// The standard output of the executed code.
        stdout: String,

        /// The standard error of the executed code.
        stderr: String,
    }
}

paired_block! {
    /// A successful code execution result block whose standard output is encrypted.
    EncryptedCodeExecutionResultBlock | EncryptedCodeExecutionResultBlockParam {
        /// The files the executed code generated.
        content: Vec<CodeExecutionResultBlockContentItem>
               | Vec<CodeExecutionResultBlockContentItemParam>,

        /// The encrypted standard output of the executed code.
        encrypted_stdout: String,

        /// The exit code of the executed code.
        #[cfg_attr(test, schema(schema_with = crate::claude_messages::spec::schema::number_schema))]
        return_code: isize,

        /// The standard error of the executed code.
        stderr: String,
    }
}

paired_content! {
    /// The content block within a successful code execution result. Can only be a code execution
    /// output.
    CodeExecutionResultBlockContentItem as "CodeExecutionResultBlockContentItem"
        | CodeExecutionResultBlockContentItemParam
            as "CodeExecutionResultBlockContentItemParam"
    {
        /// A code execution output.
        CodeExecutionOutput(CodeExecutionOutputBlock | CodeExecutionOutputBlockParam),
    }
}

paired_block! {
    /// A code execution output block.
    CodeExecutionOutputBlock | CodeExecutionOutputBlockParam {
        /// The id of the generated file.
        file_id: String,
    }
}

paired_block! {
    /// An error that occurred during code execution.
    CodeExecutionToolResultError | CodeExecutionToolResultErrorParam {
        /// The error code.
        error_code: CodeExecutionToolResultErrorCode,
    }
}

string_enum! {
    /// An error code from the code execution tool.
    #[serde(rename_all = "snake_case")]
    pub enum CodeExecutionToolResultErrorCode {
        /// `"invalid_tool_input"`
        InvalidToolInput,

        /// `"unavailable"`
        Unavailable,

        /// `"too_many_requests"`
        TooManyRequests,

        /// `"execution_time_exceeded"`
        ExecutionTimeExceeded,
    }
}

impl CodeExecutionToolResultBlock {
    /// Creates a new [`CodeExecutionToolResultBlock`] from the given tool use id and content.
    pub fn new(
        tool_use_id: impl Into<String>,
        content: CodeExecutionToolResultBlockContent,
    ) -> Self {
        Self {
            tool_use_id: tool_use_id.into(),
            content,
        }
    }
}

impl CodeExecutionToolResultBlockParam {
    /// Creates a new [`CodeExecutionToolResultBlockParam`] from the given tool use id and content.
    pub fn new(
        tool_use_id: impl Into<String>,
        content: CodeExecutionToolResultBlockParamContent,
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

impl From<CodeExecutionToolResultBlock> for CodeExecutionToolResultBlockParam {
    fn from(value: CodeExecutionToolResultBlock) -> Self {
        Self {
            tool_use_id: value.tool_use_id,
            content: value.content.into(),
            cache_control: None,
        }
    }
}
