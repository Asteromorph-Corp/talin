use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// Parameters for a session.
///
/// Mirrors [`SessionParams`] in the python source.
///
/// [`SessionParams`]: https://github.com/sgl-project/sglang/blob/430418e2182606b5296b26c912df8f12ec9b380f/python/sglang/srt/managers/io_struct.py#L109-L132
#[skip_serializing_none]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SessionParams {
    /// The session identifier.
    pub id: Option<String>,
    /// A request identifier *within* the session.
    ///
    /// In non-streaming sessions the session maintains a tree of request nodes keyed by rid; this
    /// field selects which node to continue from (append) or replace.
    ///
    /// If unset, the default branch point is used (latest node for streaming, all nodes cleared
    /// on replace).
    pub rid: Option<String>,
    /// Token-level insertion point.
    ///
    /// When set, the new request's tokens are spliced into the accumulated context at this
    /// position instead of being appended at the end (i.e. `context[:offset] + new_tokens`).
    pub offset: Option<i64>,
    /// When True, the request node identified by ``rid`` (or all nodes if `rid` is None) is
    /// aborted and its children are cleared before the new request is inserted.
    ///
    /// Not supported in streaming sessions.
    pub replace: Option<bool>,
    /// When True, the previous request's generated output tokens are excluded from the accumulated
    /// context so the new turn sees only the original input.
    ///
    /// Not supported in streaming sessions.
    pub drop_previous_output: Option<bool>,
}
