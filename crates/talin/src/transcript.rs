//! The agent transcript: the conversation state exchanged with the LLM.

use std::collections::{HashMap, HashSet};
use std::ops::Range;

use uuid::Uuid;

use crate::llm::{LlmRequestItem, LlmRequestSystem, UnsupportedContent};
use crate::schemas::{BodyMessage, LlmStep, ToolErrorMessage};
use crate::transcript::edit::{EntryEditScheme, SystemEditScheme, remove_errored_calls};
use crate::transcript::scope::Last;

pub mod edit;
mod entry;
mod error;
pub mod query;
pub mod scope;

pub use entry::{
    Entry, EntryEdit, EntryView, MessageRecord, ModelRecord, Record, SystemRecord, ToolErrorRecord,
    TruncatedRecord,
};
pub use error::{ApplyEditError, ModifyError};
pub use scope::{Anchor, Scope};

/// The conversation state exchanged with the LLM.
#[derive(Debug, Clone)]
pub struct AgentTranscript<NS, NM> {
    system: SystemRecord<NS>,
    entries: Vec<Entry<NM>>,
}

impl<S: LlmRequestSystem, B: LlmRequestItem> AgentTranscript<S, B> {
    /// Creates a transcript from the system prompt contents and the messages.
    ///
    /// Returns error if a message holds content the provider does not support.
    pub fn new(
        system: Vec<String>,
        messages: Vec<BodyMessage>,
    ) -> Result<Self, UnsupportedContent> {
        let system = SystemRecord::new(system);
        let entries = messages
            .into_iter()
            .map(Entry::try_from)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self { system, entries })
    }

    /// Consumes the transcript and renders all recorded history, omitting markers.
    /// Requests use [`Llm::materialize`](crate::llm::Llm::materialize), which also honors boundary
    /// compaction markers.
    pub fn materialize(self) -> (S, Vec<B>) {
        let items = self
            .entries
            .into_iter()
            .filter_map(Entry::into_native)
            .collect();
        (self.system.native, items)
    }

    /// Consumes the transcript, returning its system record and every stored entry.
    pub fn into_parts(self) -> (SystemRecord<S>, Vec<Entry<B>>) {
        (self.system, self.entries)
    }

    /// Appends an entry to the transcript.
    pub fn push(&mut self, entry: Entry<B>) {
        self.entries.push(entry);
    }

    /// Appends multiple entries to the transcript.
    pub fn extend(&mut self, entries: impl IntoIterator<Item = Entry<B>>) {
        self.entries.extend(entries);
    }

    /// Appends a message to the transcript.
    ///
    /// Fails if the provider cannot represent the message's content; the transcript is left
    /// unchanged.
    pub fn push_message(&mut self, message: BodyMessage) -> Result<(), UnsupportedContent> {
        self.push(Entry::try_from(message)?);
        Ok(())
    }

    /// Appends a tool error to the transcript as an [`Entry::ToolError`].
    pub fn push_tool_error(&mut self, error: ToolErrorMessage) {
        if let Some(native) = B::from_tool_error(&error) {
            self.push(Entry::from(ToolErrorRecord {
                canonical: error,
                native,
            }));
        }
    }

    /// Marks the start of a turn in the transcript.
    ///
    /// A turn refers to a single user query, the subsequent tool calls, and the model answer.
    pub fn mark_turn_start(&mut self) {
        self.push(Entry::TurnMarker);
    }

    /// Marks the start of a step in the transcript.
    ///
    /// A step refers to a single tool call and its results.
    pub fn mark_step_start(&mut self) {
        self.push(Entry::StepMarker);
    }

    /// Appends a compaction marker. With `boundary`, requests start after it; otherwise it only
    /// records that a compaction happened.
    pub fn mark_compaction(&mut self, boundary: bool) {
        self.push(Entry::CompactionMarker { boundary });
    }

    /// Marks the message as a compaction by inserting a compaction marker before it. With
    /// `boundary`, requests start at the message; otherwise the marker only records it.
    /// Idempotent with respect to a message id.
    /// No-op if id is not found.
    pub fn mark_as_compaction(&mut self, message_id: Uuid, boundary: bool) {
        let Some(index) = self
            .entries
            .iter()
            .rposition(|entry| matches!(entry, Entry::Message(id, _) if *id == message_id))
        else {
            return;
        };
        if index > 0 && matches!(self.entries[index - 1], Entry::CompactionMarker { .. }) {
            return;
        }
        self.entries
            .insert(index, Entry::CompactionMarker { boundary });
    }

    /// Inserts a compaction after the selected message, retaining all stored history and UUIDs.
    /// Original entries remain available to the LLM backend. The marker is a boundary unless the
    /// summary itself is a native compaction, which the backend then uses to omit earlier context.
    /// Idempotent with respect to `after`; no-op if it is missing or already compacted.
    /// Returns whether a compaction was inserted.
    pub fn insert_compaction(&mut self, after: Uuid, native: B) -> bool {
        let Some(index) = self.entries.iter().position(|entry| match entry {
            Entry::Message(id, _) | Entry::ToolError(id, _) | Entry::Native(id, _) => *id == after,
            _ => false,
        }) else {
            return false;
        };
        if self.entries[index..]
            .iter()
            .any(|entry| matches!(entry, Entry::CompactionMarker { .. }))
        {
            return false;
        }
        let boundary = !native.contains_native_compaction();
        self.entries
            .insert(index + 1, Entry::CompactionMarker { boundary });
        self.entries
            .insert(index + 2, Entry::Native(Uuid::new_v4(), native));
        true
    }

    /// Retrieves the most recent assistant message in the current step, if any.
    pub fn get_llm_step(&self) -> Option<LlmStep> {
        self.entries
            .iter()
            .rev()
            .take_while(|entry| !matches!(entry, Entry::StepMarker))
            .find_map(|entry| match entry {
                Entry::Message(_, record) => match &record.canonical {
                    BodyMessage::Assistant(m) => Some(LlmStep::Finish(m.clone())),
                    BodyMessage::ToolCall(m) => Some(LlmStep::ToolCall(m.clone())),
                    _ => None,
                },
                _ => None,
            })
    }

    /// Remove all entries from the position anchored by `from` to the end, including whatever
    /// `from` anchors on (e.g. a marker).
    pub fn rollback(&mut self, anchor: &dyn Anchor) {
        let marker = anchor.index(&self.full_view());
        self.entries.truncate(marker);
    }

    /// Rebuilds the transcript against another provider's native representation.
    ///
    /// Fails with [`UnsupportedContent`] if the target provider cannot represent content the
    /// source provider could.
    pub fn migrate<S2, B2>(self) -> Result<AgentTranscript<S2, B2>, UnsupportedContent>
    where
        S2: LlmRequestSystem,
        B2: LlmRequestItem,
    {
        let system = SystemRecord {
            canonical: self.system.canonical.clone(),
            native: S2::from_contents(self.system.canonical),
        };
        let mut entries = Vec::with_capacity(self.entries.len());
        let mut source = self.entries.into_iter().peekable();
        while let Some(entry) = source.next() {
            match entry {
                Entry::Message(id, record) => {
                    let native = B2::from_message(record.canonical.clone())?;
                    let canonical = record.canonical;
                    entries.push(Entry::Message(id, MessageRecord { canonical, native }));
                }
                // A target that cannot represent tool errors drops them, as when they were first
                // recorded. See [`LlmRequestItem::from_tool_error`].
                Entry::ToolError(id, record) => {
                    let Some(native) = B2::from_tool_error(&record.canonical) else {
                        continue;
                    };
                    let canonical = record.canonical;
                    entries.push(Entry::ToolError(id, ToolErrorRecord { canonical, native }));
                }
                Entry::Native(_, _) => {
                    return Err(UnsupportedContent::new(
                        std::any::type_name::<B2>(),
                        "migration of native-only transcript entries",
                    ));
                }
                Entry::StepMarker => entries.push(Entry::StepMarker),
                Entry::TurnMarker => entries.push(Entry::TurnMarker),
                // The native compaction a marker announces is lost when its record is rebuilt from
                // the canonical message, so the marker would no longer be true.
                Entry::CompactionMarker { .. }
                    if matches!(source.peek(), Some(Entry::Message(_, record))
                        if record.native.contains_native_compaction()) => {}
                Entry::CompactionMarker { boundary } => {
                    entries.push(Entry::CompactionMarker { boundary })
                }
            }
        }
        Ok(AgentTranscript { system, entries })
    }

    /// Applies a [`SystemEditScheme`] to the system prompt.
    pub async fn apply_system_edit<E: SystemEditScheme>(
        &mut self,
        scheme: &E,
    ) -> Result<(), E::Error> {
        if let Some(contents) = scheme.edit_system(&self.system.canonical).await? {
            self.modify_system(contents);
        }
        Ok(())
    }

    /// Applies an [`EntryEditScheme`] to the transcript entries.
    ///
    /// The scheme's [`scope`](EntryEditScheme::scope) picks the window of entries it edits;
    /// entries outside that window pass through unchanged.
    pub async fn apply_entry_edit<E: EntryEditScheme>(
        &mut self,
        scheme: &E,
    ) -> Result<(), ApplyEditError<E::Error>> {
        let scope = scheme.scope();
        let views = self.view(&scope);
        let edits = scheme.edit(views).await.map_err(ApplyEditError::Scheme)?;
        self.modify(edits, &scope)?;
        Ok(())
    }

    /// Applies a scheme that edits both the system prompt and the entries (e.g. [`Interpolate`]),
    /// running the system pass then the entry pass.
    ///
    /// [`Interpolate`]: edit::Interpolate
    pub async fn apply_edits<E, Err>(&mut self, scheme: &E) -> Result<(), ApplyEditError<Err>>
    where
        E: EntryEditScheme<Error = Err> + SystemEditScheme<Error = Err>,
        Err: std::error::Error + Send + Sync,
    {
        // The system prompt is plain text, so the system pass cannot fail on unsupported content.
        self.apply_system_edit(scheme)
            .await
            .map_err(ApplyEditError::Scheme)?;
        self.apply_entry_edit(scheme).await?;
        Ok(())
    }

    /// Replaces the system prompt with new contents.
    pub fn set_system(&mut self, contents: Vec<String>) {
        self.system = SystemRecord::new(contents);
    }

    /// Modifies the system prompt with new contents, preserving the provider-specific template.
    pub fn modify_system(&mut self, new_contents: Vec<String>) {
        self.system = self.system.clone().modified(new_contents);
    }

    /// Returns every stored entry, including provider-native content and markers.
    pub fn entries(&self) -> &[Entry<B>] {
        &self.entries
    }

    /// Returns an ordered view of every entry in the transcript.
    pub fn full_view(&self) -> Vec<EntryView<'_>> {
        self.entries
            .iter()
            .map(|entry| match entry {
                Entry::Message(id, record) => EntryView::Message(*id, &record.canonical),
                Entry::ToolError(id, record) => EntryView::ToolError(*id, &record.canonical),
                Entry::Native(id, _) => EntryView::Native(*id),
                Entry::StepMarker => EntryView::StepMarker,
                Entry::TurnMarker => EntryView::TurnMarker,
                Entry::CompactionMarker { boundary } => EntryView::CompactionMarker {
                    boundary: *boundary,
                },
            })
            .collect()
    }

    /// Returns the entries selected by `scope`, in order.
    pub fn view(&self, scope: &Scope) -> Vec<EntryView<'_>> {
        scope.select(&self.full_view()).to_vec()
    }

    /// Removes all unresolved tool calls from the transcript.
    pub fn remove_unresolved_tool_calls(
        &mut self,
        scope: &Scope,
    ) -> Result<(), UnsupportedContent> {
        let completed_call_ids = self
            .view(scope)
            .iter()
            .filter_map(|entry| match entry {
                EntryView::Message(_, message) if let BodyMessage::Tool(tool) = &message => {
                    Some(tool.call_id.clone())
                }
                EntryView::ToolError(_, message) => Some(message.call_id.clone()),
                _ => None,
            })
            .collect::<HashSet<_>>();

        let edits = self
            .view(scope)
            .into_iter()
            .filter_map(|entry| {
                if let EntryView::Message(id, BodyMessage::ToolCall(m)) = entry {
                    let mut new = m.clone();
                    new.tool_calls
                        .retain(|call| completed_call_ids.contains(call.id.as_str()));
                    if new.tool_calls.is_empty() {
                        return None;
                    }
                    if new.tool_calls.len() != m.tool_calls.len() {
                        return Some(EntryEdit::Replace(id, BodyMessage::ToolCall(new), true));
                    }
                }
                Some(EntryEdit::from(entry))
            })
            .collect();
        self.modify_trusted(edits, scope)
    }

    /// Removes all errored tool calls from the transcript.
    pub fn remove_errored_tool_calls(&mut self, scope: &Scope) -> Result<(), UnsupportedContent> {
        let view = self.view(scope);
        let edits = remove_errored_calls(view);
        self.modify_trusted(edits, scope)
    }

    /// Replaces the current LLM step with a new step.
    pub fn modify_step(&mut self, step: LlmStep) -> Result<(), UnsupportedContent> {
        let current = self.get_llm_step();
        if current.is_some_and(|curr| curr == step) {
            return Ok(());
        }

        let mut message = Some(BodyMessage::from(step));
        let mut edits = self
            .view(&Scope::from(Last::step()))
            .into_iter()
            .rev()
            .map(|entry| {
                if let EntryView::Message(id, m) = entry
                    && matches!(m, BodyMessage::Assistant(_) | BodyMessage::ToolCall(_))
                    && let Some(message) = message.take()
                {
                    return EntryEdit::Replace(id, message, true);
                }
                EntryEdit::from(entry)
            })
            .collect::<Vec<_>>();
        edits.reverse();
        if let Some(message) = message {
            edits.push(EntryEdit::Insert(message));
        }
        self.modify_trusted(edits, &Scope::from(Last::step()))
    }

    /// Modifies the entries using `edits` as the specification of the new sequence.
    ///
    /// `edits` is the full ordered specification of the new sequence:
    /// - [`EntryEdit::Keep(uuid)`](EntryEdit::Keep) re-emits the entry (message or tool error) with
    ///   the specified `uuid` unchanged.
    /// - [`EntryEdit::Replace(uuid, new_message, retain)`](EntryEdit::Replace) replaces the entry
    ///   with the specified `uuid` with a message. For a message target, if the final boolean is
    ///   true, the original provider-specific item is reused as a base. For a tool error target,
    ///   this value is ignored.
    /// - [`EntryEdit::Insert(new_message)`](EntryEdit::Insert) inserts `new_message` as a new
    ///   message entry with a fresh `uuid`.
    /// - [`EntryEdit::StepMarker`] places a step marker.
    ///
    /// Any entry not referenced by a `Keep`/`Replace` is dropped.
    /// It is encouraged to build `edits` from [`view`](Self::view).
    /// Use `EntryEdit::from(entry)` to keep the entry unchanged.
    ///
    /// Fails without touching the transcript if the `edits` sequence is invalid.
    pub fn modify(&mut self, edits: Vec<EntryEdit>, scope: &Scope) -> Result<(), ModifyError> {
        self.verify_edits(&edits, scope)?;
        Ok(self.modify_trusted(edits, scope)?)
    }

    /// Checks that all entries referenced by `edits` are present in range.
    fn verify_edits(&self, edits: &[EntryEdit], scope: &Scope) -> Result<(), ModifyError> {
        let mut ids: HashSet<Uuid> = self
            .view(scope)
            .iter()
            .filter_map(|entry| match entry {
                EntryView::Message(id, _) | EntryView::ToolError(id, _) | EntryView::Native(id) => {
                    Some(*id)
                }
                EntryView::StepMarker
                | EntryView::TurnMarker
                | EntryView::CompactionMarker { .. } => None,
            })
            .collect();
        for edit in edits {
            let (EntryEdit::Keep(uuid) | EntryEdit::Replace(uuid, ..)) = edit else {
                continue;
            };
            if !ids.remove(uuid) {
                return Err(ModifyError::UnknownEntry(*uuid));
            }
        }
        Ok(())
    }

    /// Modifies the entries using `edits` as the specification of the new sequence, trusting that
    /// all referenced entries are present in range.
    ///
    /// Unknown entries are silently ignored.
    fn modify_trusted(
        &mut self,
        edits: Vec<EntryEdit>,
        scope: &Scope,
    ) -> Result<(), UnsupportedContent> {
        /// An [`EntryEdit`] whose fallible conversion has already been performed, leaving only
        /// the reordering to apply.
        enum ResolvedEdit<N> {
            Keep(Uuid),
            Entry(Entry<N>),
        }

        let Range { start, end } = scope.range(&self.full_view());
        // Resolve the fallible edits against a borrow of the current entries first, so an edit the
        // provider cannot represent leaves the transcript untouched rather than truncated.
        let mut originals: HashMap<Uuid, &Entry<B>> = self.entries[start..end]
            .iter()
            .filter_map(|entry| match entry {
                Entry::Message(id, _) | Entry::ToolError(id, _) | Entry::Native(id, _) => {
                    Some((*id, entry))
                }
                Entry::StepMarker => None,
                Entry::TurnMarker => None,
                Entry::CompactionMarker { .. } => None,
            })
            .collect();

        let mut resolved = Vec::with_capacity(edits.len());
        for edit in edits {
            resolved.push(match edit {
                EntryEdit::Keep(uuid) => {
                    if originals.remove(&uuid).is_none() {
                        continue;
                    }
                    ResolvedEdit::Keep(uuid)
                }
                EntryEdit::Replace(uuid, new_message, retain) => {
                    let Some(entry) = originals.remove(&uuid) else {
                        continue;
                    };
                    let record = match entry {
                        Entry::Message(_, record) => {
                            record.clone().modified(new_message, retain)?
                        }
                        Entry::Native(_, native) => MessageRecord {
                            native: B::from_message_with(
                                new_message.clone(),
                                native.clone(),
                                retain,
                            )?,
                            canonical: new_message,
                        },
                        // A tool error has no message item to use as a base, so `retain` is moot.
                        _ => MessageRecord::new(new_message)?,
                    };
                    ResolvedEdit::Entry(Entry::Message(uuid, record))
                }
                EntryEdit::Insert(new_message) => {
                    ResolvedEdit::Entry(Entry::from(MessageRecord::new(new_message)?))
                }
                EntryEdit::StepMarker => ResolvedEdit::Entry(Entry::StepMarker),
                EntryEdit::TurnMarker => ResolvedEdit::Entry(Entry::TurnMarker),
                EntryEdit::CompactionMarker { boundary } => {
                    ResolvedEdit::Entry(Entry::CompactionMarker { boundary })
                }
            });
        }

        let after = self.entries.split_off(end);
        let mut originals: HashMap<Uuid, Entry<B>> = self
            .entries
            .drain(start..)
            .filter_map(|entry| match entry {
                Entry::Message(id, _) | Entry::ToolError(id, _) | Entry::Native(id, _) => {
                    Some((id, entry))
                }
                Entry::StepMarker => None,
                Entry::TurnMarker => None,
                Entry::CompactionMarker { .. } => None,
            })
            .collect();

        for edit in resolved {
            let entry = match edit {
                // Resolution walked the same edits over an identical key set and removed each id it
                // kept, so every id reaching here is present exactly once.
                ResolvedEdit::Keep(uuid) => originals
                    .remove(&uuid)
                    .expect("resolution proved this entry present"),
                ResolvedEdit::Entry(entry) => entry,
            };
            self.entries.push(entry);
        }
        self.entries.extend(after);
        Ok(())
    }
}

impl<S: LlmRequestSystem, B: LlmRequestItem> Default for AgentTranscript<S, B> {
    fn default() -> Self {
        Self {
            system: SystemRecord::new(Vec::new()),
            entries: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::UnsupportedContent;
    use crate::schemas::{AssistantMessage, ContentExt, ImageMimeType, MediaSource, UserMessage};
    use crate::tools::ToolError;

    /// A request item that accepts text but rejects any attached image, standing in for a provider
    /// that cannot represent a given media source.
    #[derive(Debug, Clone, PartialEq)]
    struct TextOnly(String);

    impl LlmRequestItem for TextOnly {
        fn from_message(message: BodyMessage) -> Result<Self, UnsupportedContent> {
            if !message.content().is_text() {
                return Err(UnsupportedContent::new("test", "images"));
            }
            Ok(TextOnly(message.content().to_text().unwrap_or_default()))
        }

        fn from_message_with(
            message: BodyMessage,
            _: Self,
            _: bool,
        ) -> Result<Self, UnsupportedContent> {
            Self::from_message(message)
        }

        fn from_tool_error(_: &ToolErrorMessage) -> Option<Self> {
            None
        }
    }

    type Transcript = AgentTranscript<Vec<String>, TextOnly>;

    fn text(content: &str) -> BodyMessage {
        BodyMessage::User(UserMessage::new(content))
    }

    fn with_image(content: &str) -> BodyMessage {
        let image = MediaSource::from_url("https://e.com/a.png").image(ImageMimeType::Png);
        BodyMessage::User(UserMessage::new(content).with_images(vec![image]))
    }

    fn contents(transcript: &Transcript) -> Vec<String> {
        transcript
            .clone()
            .materialize()
            .1
            .into_iter()
            .map(|item| item.0)
            .collect()
    }

    fn compaction_markers(transcript: &Transcript) -> Vec<usize> {
        transcript
            .entries()
            .iter()
            .enumerate()
            .filter_map(|(index, entry)| {
                matches!(entry, Entry::CompactionMarker { .. }).then_some(index)
            })
            .collect()
    }

    fn summary(content: &str) -> Entry<TextOnly> {
        Entry::try_from(BodyMessage::Assistant(AssistantMessage::new(None, content))).unwrap()
    }

    #[test]
    fn compaction_markers_can_be_added_out_of_order_without_changing_history() {
        let mut transcript = Transcript::new(
            vec!["system".to_owned()],
            vec![text("old"), text("summary"), text("tail")],
        )
        .unwrap();
        let ids = transcript
            .full_view()
            .into_iter()
            .filter_map(|entry| match entry {
                EntryView::Message(id, _) => Some(id),
                _ => None,
            })
            .collect::<Vec<_>>();
        let full_history = transcript.clone().materialize();

        for id in [ids[2], ids[0], ids[1]] {
            transcript.mark_as_compaction(id, true);
            assert!(transcript.entries().windows(2).any(|pair| {
                matches!(pair, [Entry::CompactionMarker { .. }, Entry::Message(marked, _)] if *marked == id)
            }));
            assert_eq!(transcript.clone().materialize(), full_history);
        }

        let markers = compaction_markers(&transcript);
        let entry_count = transcript.entries.len();
        // Repeated and absent targets leave the compaction markers unchanged.
        for id in [ids[0], ids[1], ids[2], Uuid::nil()] {
            transcript.mark_as_compaction(id, true);
            assert_eq!(compaction_markers(&transcript), markers);
            assert_eq!(transcript.entries.len(), entry_count);
        }
        assert_eq!(transcript.clone().materialize(), full_history);
        assert_eq!(
            transcript
                .full_view()
                .into_iter()
                .filter_map(|entry| match entry {
                    EntryView::Message(id, _) => Some(id),
                    _ => None,
                })
                .collect::<Vec<_>>(),
            ids
        );
    }

    #[test]
    fn compaction_marker_is_idempotent_across_generated_layouts() {
        let first_id = Uuid::nil();
        let second_id = Uuid::from_u128(u128::MAX);
        let missing_id = Uuid::from_u128(1);
        let alphabet = [
            Entry::Message(first_id, MessageRecord::new(text("first")).unwrap()),
            Entry::Message(second_id, MessageRecord::new(text("second")).unwrap()),
            Entry::ToolError(
                second_id,
                ToolErrorRecord {
                    canonical: ToolErrorMessage::new(
                        "call",
                        "missing",
                        ToolError::ToolNotFound("missing".to_owned()),
                    ),
                    native: TextOnly("tool error".to_owned()),
                },
            ),
            Entry::StepMarker,
            Entry::TurnMarker,
            Entry::CompactionMarker { boundary: true },
        ];

        // Enumerate every layout up to five entries, including duplicate UUIDs, adjacent
        // boundaries, trailing markers and a tool error sharing a message UUID.
        // Compare one application with two: expected states are not written by hand.
        for length in 0..=5 {
            for layout in 0..alphabet.len().pow(length) {
                let mut digits = layout;
                let mut original = Transcript::default();
                for _ in 0..length {
                    original.push(alphabet[digits % alphabet.len()].clone());
                    digits /= alphabet.len();
                }
                for id in [first_id, second_id, missing_id] {
                    let mut transcript = original.clone();
                    transcript.mark_as_compaction(id, true);
                    let once = transcript.clone();
                    transcript.mark_as_compaction(id, true);

                    assert_eq!(transcript.entries.len(), once.entries.len());
                    assert_eq!(compaction_markers(&transcript), compaction_markers(&once));
                    assert!(
                        transcript
                            .full_view()
                            .into_iter()
                            .filter_map(|entry| match entry {
                                EntryView::Message(id, _) => Some(id),
                                _ => None,
                            })
                            .eq(once
                                .full_view()
                                .into_iter()
                                .filter_map(|entry| match entry {
                                    EntryView::Message(id, _) => Some(id),
                                    _ => None,
                                }))
                    );
                    assert_eq!(transcript.materialize(), once.materialize());
                }
            }
        }
    }

    #[test]
    fn materialization_preserves_history_with_compaction_markers() {
        let mut transcript = Transcript::new(vec!["system".to_owned()], vec![text("old")]).unwrap();
        transcript.mark_compaction(true);
        transcript.push(summary("first summary"));
        transcript.push_message(text("older tail")).unwrap();
        transcript.mark_turn_start();
        transcript.mark_compaction(true);
        transcript.push(summary("latest summary"));
        transcript.mark_step_start();
        transcript.push_message(text("new turn")).unwrap();
        let full = transcript.clone().materialize();
        assert_eq!(
            full.1
                .iter()
                .map(|item| item.0.as_str())
                .collect::<Vec<_>>(),
            [
                "old",
                "first summary",
                "older tail",
                "latest summary",
                "new turn"
            ]
        );
        transcript.rollback(&Last::turn());
        assert_eq!(
            contents(&transcript),
            ["old", "first summary", "older tail"]
        );
    }

    #[test]
    fn renders_no_compaction_and_tool_errors() {
        let mut transcript = Transcript::new(vec![], vec![text("old")]).unwrap();
        transcript.push(Entry::from(ToolErrorRecord {
            canonical: ToolErrorMessage::new(
                "call_1",
                "missing",
                ToolError::ToolNotFound("missing".to_owned()),
            ),
            native: TextOnly("tool error".to_owned()),
        }));
        assert_eq!(contents(&transcript), ["old", "tool error"]);
        transcript.mark_compaction(true);
        transcript.push(summary("summary"));
        assert_eq!(contents(&transcript), ["old", "tool error", "summary"]);
    }

    #[test]
    fn keeps_compaction_marker_through_edits_and_migration() {
        let mut transcript = Transcript::new(vec![], vec![text("old")]).unwrap();
        transcript.mark_compaction(true);
        transcript.push_message(text("custom summary")).unwrap();
        let edits = transcript
            .view(&Scope::full())
            .into_iter()
            .map(EntryEdit::from)
            .collect();
        transcript.modify(edits, &Scope::full()).unwrap();
        assert_eq!(contents(&transcript), ["old", "custom summary"]);

        let migrated = transcript.migrate::<Vec<String>, BodyMessage>().unwrap();
        assert!(matches!(
            migrated.view(&Scope::full())[1],
            EntryView::CompactionMarker { .. }
        ));
        assert_eq!(migrated.materialize().1.len(), 2);
    }

    #[test]
    fn rejects_unsupported_content_on_push() {
        let mut transcript = Transcript::default();
        transcript.push_message(text("a")).unwrap();

        transcript.push_message(with_image("b")).unwrap_err();

        // The rejected message is not recorded, so it can never reach a request.
        assert_eq!(contents(&transcript), ["a"]);
    }

    #[test]
    fn rejects_unsupported_content_on_init() {
        Transcript::new(Vec::new(), vec![text("a"), with_image("b")]).unwrap_err();
    }

    #[test]
    fn rejects_edit_referencing_out_of_range_entry() {
        let mut transcript = Transcript::new(Vec::new(), vec![text("a")]).unwrap();
        let stale = Uuid::new_v4();

        let error = transcript
            .modify(vec![EntryEdit::Keep(stale)], &Scope::full())
            .unwrap_err();

        // Skipping the edit would silently drop an entry the caller asked to keep.
        assert!(matches!(error, ModifyError::UnknownEntry(id) if id == stale));
        assert_eq!(contents(&transcript), ["a"]);
    }

    #[test]
    fn rejects_edit_referencing_same_entry_twice() {
        let mut transcript = Transcript::new(Vec::new(), vec![text("a")]).unwrap();
        let EntryView::Message(id, _) = transcript.full_view()[0] else {
            unreachable!("the entry is a message");
        };

        let edits = vec![EntryEdit::Keep(id), EntryEdit::Keep(id)];
        let error = transcript.modify(edits, &Scope::full()).unwrap_err();

        assert!(matches!(error, ModifyError::UnknownEntry(_)));
    }

    #[test]
    fn transcript_untouched_on_rejected_edit() {
        let mut transcript = Transcript::new(Vec::new(), vec![text("a"), text("b")]).unwrap();
        let view = transcript.full_view();
        let target = match view[1] {
            EntryView::Message(id, _) => id,
            _ => unreachable!("the second entry is a message"),
        };

        // `Keep` the first entry, then fail on the second: the drained tail must be restored.
        let edits = vec![
            EntryEdit::from(view[0]),
            EntryEdit::Replace(target, with_image("c"), false),
        ];
        transcript.modify(edits, &Scope::full()).unwrap_err();

        assert_eq!(contents(&transcript), ["a", "b"]);
    }
}
