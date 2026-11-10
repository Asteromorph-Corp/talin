//! The result blocks of Claude's internal tools.
//!
//! Each submodule holds one tool's result block, the request-side twin of that block, and the
//! payload the block's `content` carries.

mod pair;

mod bash_code_execution;
mod code_execution;
mod text_editor_code_execution;
mod tool_search;
mod web_fetch;
mod web_search;

pub use bash_code_execution::*;
pub use code_execution::*;
pub use text_editor_code_execution::*;
pub use tool_search::*;
pub use web_fetch::*;
pub use web_search::*;
