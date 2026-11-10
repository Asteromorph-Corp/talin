use std::convert::Infallible;

use async_trait::async_trait;
use futures::future::try_join_all;
use indexmap::IndexSet;

use crate::schemas::{BodyMessage, ToolCall};
use crate::tools::normalize_tool_name;
use crate::transcript::edit::EntryEditScheme;
use crate::transcript::{EntryEdit, EntryView};

/// Removes the tool calls selected by a [`ToolCallSelector`], together with each call's paired
/// result or error.
///
/// A tool-call message is trimmed to its surviving calls. If none survive, the message is dropped
/// entirely.
pub struct RemoveToolCalls<M = ()>(pub M);

#[async_trait]
impl<M: ToolCallSelector> EntryEditScheme for RemoveToolCalls<M> {
    type Error = M::Error;

    async fn edit(&self, views: Vec<EntryView<'_>>) -> Result<Vec<EntryEdit>, Self::Error> {
        let matched = self.0.select(&views).await?;
        let edits = remove_calls(views, &matched);
        Ok(edits)
    }
}

impl RemoveToolCalls {
    /// Removes every tool call that matches a [`filter`](ToolCallFilter).
    pub fn matching<F: ToolCallFilter>(filter: F) -> RemoveToolCalls<Matching<F>> {
        RemoveToolCalls(Matching(filter))
    }

    /// Removes every tool call resolved by a tool error, together with the error.
    pub fn errored() -> RemoveToolCalls<Errored> {
        RemoveToolCalls(Errored)
    }
}

impl<M: ToolCallSelector> RemoveToolCalls<M> {
    /// Returns an edit scheme that drops every matching tool call except the last one, together
    /// with each removed call's paired result or error.
    ///
    /// Ordering is by call id in transcript order; the last matching call (and its result)
    /// survives.
    pub fn except_last(self) -> impl EntryEditScheme<Error = M::Error> {
        RemoveToolCalls(ExceptLast(self.0))
    }
}

/// Selects which tool calls to remove, by id, from a view of the transcript.
///
/// This is the seam every tool-call removal funnels through: a match decides *which* calls die and
/// [`RemoveToolCalls`] excises them consistently — trimming each tool-call message to its survivors
/// and dropping every removed call's paired result or error.
///
/// Because a match sees the whole entry slice, it can key off anything: call names (via
/// [`ToolCallFilter`]), a call's outcome (by pairing calls with their results/errors), chronology
/// (e.g. keep only the most recent match), or position — without a bespoke scheme for each.
///
/// # Implementations
///
/// - [`Matching`]: the ids of calls accepted by a [`ToolCallFilter`].
/// - [`Errored`]: the ids of calls resolved by a tool error.
/// - `()`: the empty match; removes nothing.
/// - `Fn(&[EntryView<'_>]) -> Result<IndexSet<String>, E>`: an ad-hoc match computed from the
///   views.
#[async_trait]
pub trait ToolCallSelector: Send + Sync {
    /// The error returned when selecting tool calls fails.
    type Error: std::error::Error + Send + Sync;

    /// Returns the ids of the tool calls to remove, in the order they occur in the transcript.
    async fn select(&self, views: &[EntryView<'_>]) -> Result<IndexSet<String>, Self::Error>;
}

/// Drops every tool call whose id is in `removed`, together with its output / error.
///
/// A tool-call message is trimmed to its surviving calls.
/// If none survive, the message is dropped entirely.
pub(super) fn remove_calls(
    views: Vec<EntryView<'_>>,
    removed: &IndexSet<String>,
) -> Vec<EntryEdit> {
    views
        .into_iter()
        .filter_map(|view| match view {
            // Drop the result or error resolving a removed call.
            EntryView::Message(_, BodyMessage::Tool(m)) if removed.contains(m.call_id.as_str()) => {
                None
            }
            EntryView::ToolError(_, m) if removed.contains(m.call_id.as_str()) => None,
            // Trim removed calls from a tool-call message; drop it if nothing remains.
            EntryView::Message(id, BodyMessage::ToolCall(m)) => {
                let mut trimmed = m.clone();
                trimmed
                    .tool_calls
                    .retain(|c| !removed.contains(c.id.as_str()));
                if trimmed.tool_calls.is_empty() {
                    None
                } else if trimmed.tool_calls.len() != m.tool_calls.len() {
                    Some(EntryEdit::Replace(id, BodyMessage::ToolCall(trimmed), true))
                } else {
                    Some(EntryEdit::Keep(id))
                }
            }
            other => Some(EntryEdit::from(other)),
        })
        .collect()
}

// --- `ToolCallSelector` impls ---

#[async_trait]
impl ToolCallSelector for () {
    type Error = Infallible;

    async fn select(&self, _views: &[EntryView<'_>]) -> Result<IndexSet<String>, Self::Error> {
        Ok(IndexSet::new())
    }
}

/// The closure may return any collection of ids.
#[async_trait]
impl<F, E, S> ToolCallSelector for F
where
    F: Fn(&[EntryView<'_>]) -> Result<S, E> + Send + Sync,
    S: IntoIterator<Item = String> + Send,
    E: std::error::Error + Send + Sync,
{
    type Error = E;

    async fn select(&self, views: &[EntryView<'_>]) -> Result<IndexSet<String>, Self::Error> {
        Ok(self(views)?.into_iter().collect())
    }
}

// --- `Matching` ---

/// A [`ToolCallSelector`] over the calls a [`ToolCallFilter`] matches. See
/// [`RemoveToolCalls::matching`].
pub struct Matching<F: ToolCallFilter>(pub F);

#[async_trait]
impl<F: ToolCallFilter> ToolCallSelector for Matching<F> {
    type Error = F::Error;

    async fn select(&self, views: &[EntryView<'_>]) -> Result<IndexSet<String>, Self::Error> {
        matching_ids(&self.0, views).await
    }
}

/// Ids of the calls `filter` matches, in transcript order.
async fn matching_ids<F: ToolCallFilter>(
    filter: &F,
    views: &[EntryView<'_>],
) -> Result<IndexSet<String>, F::Error> {
    let calls = views
        .iter()
        .filter_map(|view| match view {
            EntryView::Message(_, BodyMessage::ToolCall(m)) => Some(&m.tool_calls),
            _ => None,
        })
        .flatten();

    let matched = calls.map(|c| async move { Ok(filter.matches(c).await?.then(|| c.id.clone())) });
    Ok(try_join_all(matched).await?.into_iter().flatten().collect())
}

/// A tool call filter.
///
/// A per-call predicate; adapts into a [`ToolCallSelector`] via [`Matching`] for the common
/// "remove by name" case.
///
/// # Implementations
///
/// - `&str`: Matches tool calls with the same name (case-insensitive).
/// - `Fn(&ToolCall) -> Result<bool, E>`: Matches tool calls for which the function returns true.
#[async_trait]
pub trait ToolCallFilter: Send + Sync {
    /// The error returned when testing a tool call fails.
    type Error: std::error::Error + Send + Sync;

    /// Returns true if the given tool call should be removed.
    async fn matches(&self, tool_call: &ToolCall) -> Result<bool, Self::Error>;
}

#[async_trait]
impl ToolCallFilter for &str {
    type Error = Infallible;

    async fn matches(&self, tool_call: &ToolCall) -> Result<bool, Self::Error> {
        Ok(normalize_tool_name(&tool_call.name) == normalize_tool_name(self))
    }
}

#[async_trait]
impl<F, E> ToolCallFilter for F
where
    F: Fn(&ToolCall) -> Result<bool, E> + Send + Sync,
    E: std::error::Error + Send + Sync,
{
    type Error = E;

    async fn matches(&self, tool_call: &ToolCall) -> Result<bool, Self::Error> {
        self(tool_call)
    }
}

// --- `Errored` ---

/// A [`ToolCallSelector`] over the calls resolved by a tool error. Removing them drops the errors
/// too. See [`RemoveToolCalls::errored`].
pub struct Errored;

#[async_trait]
impl ToolCallSelector for Errored {
    type Error = Infallible;

    async fn select(&self, views: &[EntryView<'_>]) -> Result<IndexSet<String>, Self::Error> {
        Ok(errored_call_ids(views))
    }
}

// --- `ExceptLast` ---

/// A [`ToolCallSelector`] that spares the last id of the match it wraps — every matched call is
/// removed except the final one. See [`RemoveToolCalls::except_last`].
struct ExceptLast<M: ToolCallSelector>(M);

#[async_trait]
impl<M: ToolCallSelector> ToolCallSelector for ExceptLast<M> {
    type Error = M::Error;

    async fn select(&self, views: &[EntryView<'_>]) -> Result<IndexSet<String>, Self::Error> {
        let mut ids = self.0.select(views).await?;
        ids.pop(); // spare the last match
        Ok(ids)
    }
}

/// Removes every tool call resolved by a tool error, together with the error.
pub(crate) fn remove_errored_calls(views: Vec<EntryView<'_>>) -> Vec<EntryEdit> {
    // Call ids resolved by the errors being removed; these calls become unresolved.
    let removed = errored_call_ids(&views);
    remove_calls(views, &removed)
}

/// Ids of the calls resolved by a tool error, in transcript order.
fn errored_call_ids(views: &[EntryView<'_>]) -> IndexSet<String> {
    views
        .iter()
        .filter_map(|view| match view {
            EntryView::ToolError(_, m) => Some(m.call_id.clone()),
            _ => None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::convert::Infallible;

    use indexmap::IndexSet;
    use uuid::Uuid;

    use super::RemoveToolCalls;
    use crate::schemas::{
        BodyMessage, ContentExt, ToolCall, ToolCallMessage, ToolErrorMessage, ToolMessage,
        ToolOutput, UserMessage,
    };
    use crate::tools::ToolError;
    use crate::transcript::edit::EntryEditScheme;
    use crate::transcript::{AgentTranscript, EntryEdit, EntryView};

    fn call(id: &str, name: &str) -> ToolCall {
        ToolCall::new(Some(id.into()), name, None)
    }

    fn call_msg(calls: Vec<ToolCall>) -> BodyMessage {
        BodyMessage::ToolCall(ToolCallMessage::new(None, calls))
    }

    fn result_msg(call_id: &str, name: &str) -> BodyMessage {
        BodyMessage::Tool(ToolMessage::new(
            call_id,
            name,
            ToolOutput::from(String::from("result")),
        ))
    }

    fn error_msg(call_id: &str, name: &str) -> ToolErrorMessage {
        ToolErrorMessage::new(call_id, name, ToolError::ToolNotFound(name.into()))
    }

    fn user_msg(text: &str) -> BodyMessage {
        BodyMessage::User(UserMessage::new(text))
    }

    fn keep_id(edit: &EntryEdit) -> Option<Uuid> {
        match edit {
            EntryEdit::Keep(id) => Some(*id),
            _ => None,
        }
    }

    fn replace(edit: &EntryEdit) -> Option<(Uuid, &BodyMessage, bool)> {
        match edit {
            EntryEdit::Replace(id, body, retain) => Some((*id, body, *retain)),
            _ => None,
        }
    }

    #[tokio::test]
    async fn drops_sole_match() {
        let (calls, result, user) = (
            call_msg(vec![call("c1", "wait_for_job")]),
            result_msg("c1", "wait_for_job"),
            user_msg("hi"),
        );
        let user_id = Uuid::new_v4();
        let views = vec![
            EntryView::StepMarker,
            EntryView::Message(Uuid::new_v4(), &calls),
            EntryView::Message(Uuid::new_v4(), &result),
            EntryView::Message(user_id, &user),
        ];

        let edits = RemoveToolCalls::matching("wait_for_job")
            .edit(views)
            .await
            .unwrap();

        // The call message (now empty) and its result are gone; the marker and user survive.
        assert_eq!(edits.len(), 2);
        assert!(matches!(edits[0], EntryEdit::StepMarker));
        assert_eq!(keep_id(&edits[1]), Some(user_id));
    }

    #[tokio::test]
    async fn trims_match() {
        let (calls, r1, r2) = (
            call_msg(vec![call("c1", "wait_for_job"), call("c2", "start_job")]),
            result_msg("c1", "wait_for_job"),
            result_msg("c2", "start_job"),
        );
        let (calls_id, r2_id) = (Uuid::new_v4(), Uuid::new_v4());
        let views = vec![
            EntryView::Message(calls_id, &calls),
            EntryView::Message(Uuid::new_v4(), &r1),
            EntryView::Message(r2_id, &r2),
        ];

        let edits = RemoveToolCalls::matching("wait_for_job")
            .edit(views)
            .await
            .unwrap();

        assert_eq!(edits.len(), 2);
        // The call message is trimmed to `start_job`, reusing the native base.
        let (id, body, retain) = replace(&edits[0]).expect("expected Replace");
        assert_eq!(id, calls_id);
        assert!(retain);
        let BodyMessage::ToolCall(m) = body else {
            panic!("expected a tool-call body");
        };
        assert_eq!(m.tool_calls.len(), 1);
        assert_eq!(m.tool_calls[0].name, "start_job");
        // The `wait_for_job` result is dropped; the `start_job` result is kept.
        assert_eq!(keep_id(&edits[1]), Some(r2_id));
    }

    #[tokio::test]
    async fn keeps_no_match() {
        let (calls, result) = (
            call_msg(vec![call("c2", "start_job")]),
            result_msg("c2", "start_job"),
        );
        let views = vec![
            EntryView::Message(Uuid::new_v4(), &calls),
            EntryView::Message(Uuid::new_v4(), &result),
        ];

        let edits = RemoveToolCalls::matching("wait_for_job")
            .edit(views)
            .await
            .unwrap();

        assert_eq!(edits.iter().filter_map(keep_id).count(), 2);
    }

    #[tokio::test]
    async fn drops_paired_error() {
        // A removed call whose result was an error: the error entry is dropped alongside the call.
        let (calls, err) = (call_msg(vec![call("c1", "run")]), error_msg("c1", "run"));
        let views = vec![
            EntryView::Message(Uuid::new_v4(), &calls),
            EntryView::ToolError(Uuid::new_v4(), &err),
        ];

        let edits = RemoveToolCalls::matching("run").edit(views).await.unwrap();

        // The call message (now empty) and its paired error both disappear.
        assert!(edits.is_empty());
    }

    #[tokio::test]
    async fn str_filter_normalized() {
        let calls = call_msg(vec![call("c1", "Wait For Job")]);
        let id = Uuid::new_v4();

        let by_name = RemoveToolCalls::matching("wait_for_job")
            .edit(vec![EntryView::Message(id, &calls)])
            .await
            .unwrap();
        assert!(by_name.is_empty());
    }

    #[tokio::test]
    async fn closure_filter() {
        let calls = call_msg(vec![call("c1", "Wait For Job")]);
        let id = Uuid::new_v4();

        // `&str` filter normalizes both sides, so spacing/case differences still match.
        // A predicate filter keys off the whole call, e.g. its id.
        let by_pred = RemoveToolCalls::matching(|c: &ToolCall| Ok::<_, Infallible>(c.id == "c1"))
            .edit(vec![EntryView::Message(id, &calls)])
            .await
            .unwrap();
        assert!(by_pred.is_empty());
    }

    #[tokio::test]
    async fn remove_calls_closure_match() {
        // A custom match keys off the whole view slice — here, keep only the last `read` call and
        // drop earlier ones (ordered logic a per-call filter cannot express).
        let (c1, r1, c2, r2) = (
            call_msg(vec![call("c1", "read")]),
            result_msg("c1", "read"),
            call_msg(vec![call("c2", "read")]),
            result_msg("c2", "read"),
        );
        let (c2_id, r2_id) = (Uuid::new_v4(), Uuid::new_v4());
        let views = vec![
            EntryView::Message(Uuid::new_v4(), &c1),
            EntryView::Message(Uuid::new_v4(), &r1),
            EntryView::Message(c2_id, &c2),
            EntryView::Message(r2_id, &r2),
        ];

        let keep_last_read = |views: &[EntryView<'_>]| {
            let mut ids: Vec<String> = views
                .iter()
                .filter_map(|v| match v {
                    EntryView::Message(_, BodyMessage::ToolCall(m)) => Some(&m.tool_calls),
                    _ => None,
                })
                .flatten()
                .filter(|c| c.name == "read")
                .map(|c| c.id.clone())
                .collect();
            ids.pop();
            Ok::<_, Infallible>(ids.into_iter().collect::<IndexSet<_>>())
        };

        let edits = RemoveToolCalls(keep_last_read).edit(views).await.unwrap();

        // The first read and its result are dropped; the last read and its result survive.
        assert_eq!(edits.len(), 2);
        assert_eq!(keep_id(&edits[0]), Some(c2_id));
        assert_eq!(keep_id(&edits[1]), Some(r2_id));
    }

    #[tokio::test]
    async fn except_last_spares_final_match() {
        // The ergonomic form of `remove_calls_closure_match`: drop every `read` but the last.
        let (c1, r1, c2, r2) = (
            call_msg(vec![call("c1", "read")]),
            result_msg("c1", "read"),
            call_msg(vec![call("c2", "read")]),
            result_msg("c2", "read"),
        );
        let (c2_id, r2_id) = (Uuid::new_v4(), Uuid::new_v4());
        let views = vec![
            EntryView::Message(Uuid::new_v4(), &c1),
            EntryView::Message(Uuid::new_v4(), &r1),
            EntryView::Message(c2_id, &c2),
            EntryView::Message(r2_id, &r2),
        ];

        let edits = RemoveToolCalls::matching("read")
            .except_last()
            .edit(views)
            .await
            .unwrap();

        assert_eq!(edits.len(), 2);
        assert_eq!(keep_id(&edits[0]), Some(c2_id));
        assert_eq!(keep_id(&edits[1]), Some(r2_id));
    }

    #[tokio::test]
    async fn except_last_keeps_sole_match() {
        // With a single match there is nothing to remove — the call and its result survive.
        let (calls, result) = (call_msg(vec![call("c1", "read")]), result_msg("c1", "read"));
        let views = vec![
            EntryView::Message(Uuid::new_v4(), &calls),
            EntryView::Message(Uuid::new_v4(), &result),
        ];

        let edits = RemoveToolCalls::matching("read")
            .except_last()
            .edit(views)
            .await
            .unwrap();

        assert_eq!(edits.iter().filter_map(keep_id).count(), 2);
    }

    #[tokio::test]
    async fn applies_edits() {
        let mut transcript = AgentTranscript::<Vec<String>, BodyMessage>::new(
            Vec::new(),
            vec![
                call_msg(vec![call("c1", "wait_for_job")]),
                result_msg("c1", "wait_for_job"),
                user_msg("done"),
            ],
        )
        .unwrap();

        transcript
            .apply_entry_edit(&RemoveToolCalls::matching("wait_for_job"))
            .await
            .unwrap();

        // The call message and its result are gone; only the user message materializes.
        let (_system, items) = transcript.materialize();
        assert_eq!(items.len(), 1);
        assert!(
            matches!(&items[0], BodyMessage::User(u) if u.content.to_text().as_deref() == Some("done"))
        );
    }

    #[tokio::test]
    async fn drops_error_and_its_sole_call() {
        let (calls, err, user) = (
            call_msg(vec![call("c1", "run")]),
            error_msg("c1", "run"),
            user_msg("hi"),
        );
        let user_id = Uuid::new_v4();
        let views = vec![
            EntryView::Message(Uuid::new_v4(), &calls),
            EntryView::ToolError(Uuid::new_v4(), &err),
            EntryView::Message(user_id, &user),
        ];

        let edits = RemoveToolCalls::errored().edit(views).await.unwrap();

        // The error and its now-empty call message are gone; only the user survives.
        assert_eq!(edits.len(), 1);
        assert_eq!(keep_id(&edits[0]), Some(user_id));
    }

    #[tokio::test]
    async fn trims_errored_call_but_keeps_siblings() {
        let (calls, err) = (
            call_msg(vec![call("c1", "run"), call("c2", "ok")]),
            error_msg("c1", "run"),
        );
        let calls_id = Uuid::new_v4();
        let views = vec![
            EntryView::Message(calls_id, &calls),
            EntryView::ToolError(Uuid::new_v4(), &err),
        ];

        let edits = RemoveToolCalls::errored().edit(views).await.unwrap();

        // The errored call is trimmed away; the surviving call keeps the message alive.
        assert_eq!(edits.len(), 1);
        let EntryEdit::Replace(id, BodyMessage::ToolCall(m), true) = &edits[0] else {
            panic!("expected a trimmed tool-call replace");
        };
        assert_eq!(*id, calls_id);
        assert_eq!(m.tool_calls.len(), 1);
        assert_eq!(m.tool_calls[0].name, "ok");
    }

    #[tokio::test]
    async fn keeps_everything_without_errors() {
        let (calls, user) = (call_msg(vec![call("c1", "run")]), user_msg("hi"));
        let views = vec![
            EntryView::Message(Uuid::new_v4(), &calls),
            EntryView::Message(Uuid::new_v4(), &user),
        ];

        let edits = RemoveToolCalls::errored().edit(views).await.unwrap();

        assert_eq!(edits.iter().filter_map(keep_id).count(), 2);
    }
}
