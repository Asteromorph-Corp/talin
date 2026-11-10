mod error;
mod extract;
mod final_answer;
mod has_keys;
mod json;
mod repair;
mod tool_call;

pub(crate) use error::ParseError;
pub(crate) use extract::{
    extract_from_codeblock, extract_from_tag, extract_jsons, extract_preamble, remove_thought,
};
pub(crate) use final_answer::parse_final_answer;
pub(crate) use has_keys::has_keys;
pub(crate) use json::parse_json;
pub(crate) use repair::{
    close_unclosed_structures, escape_newlines_in_strings, strip_trailing_commas, unescape_text,
};
pub(crate) use tool_call::{RawToolCall, parse_tool_call};

/// Items needed for macro expansion.
pub(crate) mod macro_support {
    pub use super::error::ParseError;
    pub use super::extract::extract_jsons;
    pub use super::json::parse_json;
}
