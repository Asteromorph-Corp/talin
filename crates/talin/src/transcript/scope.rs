//! Scopes and anchors for selecting ranges of entries in a transcript.

use crate::transcript::EntryView;

/// A range of entries in a transcript, specified by two anchors.
///
/// The anchors are clamped to the bounds of the transcript, and if `start` is greater than `end`,
/// `end` is set to `start`.
pub struct Scope {
    /// The anchor for the first entry in the range (inclusive).
    pub start: Box<dyn Anchor>,
    /// The anchor for the last entry in the range (exclusive).
    pub end: Box<dyn Anchor>,
}

impl Scope {
    /// Creates a new scope from `start` to `end`.
    pub fn new(start: impl Anchor + 'static, end: impl Anchor + 'static) -> Self {
        Self {
            start: Box::new(start),
            end: Box::new(end),
        }
    }

    /// Creates a scope that includes all entries in the transcript.
    pub fn full() -> Self {
        Self::new(Start, End)
    }

    /// Creates a scope from the start of the transcript to `end`.
    pub fn to(end: impl Anchor + 'static) -> Self {
        Self::new(Start, end)
    }

    /// Creates a scope from `start` to the end of the transcript.
    pub fn from(start: impl Anchor + 'static) -> Self {
        Self::new(start, End)
    }

    /// Returns the half-open index range specified by this scope.
    ///
    /// The range clamped to the bounds of the transcript, and if `start` is greater than `end`,
    /// `end` is set to `start`.
    pub fn range(&self, views: &[EntryView<'_>]) -> std::ops::Range<usize> {
        let mut start = self.start.index(views).min(views.len());
        let mut end = self.end.index(views).min(views.len());
        if start > views.len() {
            start = views.len();
        }
        if start > end {
            end = start;
        }
        if end > views.len() {
            end = views.len();
        }
        start..end
    }

    /// Returns the entries of `views` in the scope.
    pub fn select<'a, 'b>(&self, views: &'a [EntryView<'b>]) -> &'a [EntryView<'b>] {
        let range = self.range(views);
        &views[range]
    }
}

/// An anchor specifying a position (index) in a transcript.
///
/// # Implementations
///
/// - [`Start`]: the first entry (index `0`).
/// - [`End`]: one past the last entry (index `views.len()`).
/// - [`First`] / [`Last`]: the first / last entry that satisfies a predicate.
/// - `Fn(&[EntryView<'_>]) -> usize`: an ad-hoc anchor computed from the views.
pub trait Anchor: Send + Sync {
    /// Returns the index of the anchored position in `views`.
    fn index(&self, views: &[EntryView<'_>]) -> usize;
}

impl<F> Anchor for F
where
    F: Fn(&[EntryView<'_>]) -> usize + Send + Sync,
{
    fn index(&self, views: &[EntryView<'_>]) -> usize {
        self(views)
    }
}

/// The first in-range entry (index `0`).
pub struct Start;

impl Anchor for Start {
    fn index(&self, _: &[EntryView<'_>]) -> usize {
        0
    }
}

/// One past the last in-range entry (index `views.len()`).
pub struct End;

impl Anchor for End {
    fn index(&self, views: &[EntryView<'_>]) -> usize {
        views.len()
    }
}

/// The first in-range entry satisfying a predicate.
/// If no entry matches (or `skip` exceeds the number of matches), anchors at `views.len()`.
///
/// You can [`skip`](Self::skip) `n` matches after the first, or restrict the forward scan to
/// entries after another anchor with [`from`](Self::from).
pub struct First<M> {
    pred: M,
    skip: usize,
    from: Option<Box<dyn Anchor>>,
}

impl First<fn(&EntryView<'_>) -> bool> {
    /// Anchors at the first step marker.
    pub fn step() -> Self {
        Self::matching(|v| matches!(v, EntryView::StepMarker))
    }

    /// Anchors at the first turn marker.
    pub fn turn() -> Self {
        Self::matching(|v| matches!(v, EntryView::TurnMarker))
    }
}

impl<M> First<M> {
    /// Anchors at the first entry satisfying `pred`.
    pub fn matching(pred: M) -> Self {
        Self {
            pred,
            skip: 0,
            from: None,
        }
    }

    /// Steps forward `n` further matches, anchoring at the `n`-th match.
    pub fn skip(mut self, n: usize) -> Self {
        self.skip = n;
        self
    }

    /// Scans forward from `bound`, considering only entries from it onward.
    pub fn from(mut self, bound: impl Anchor + 'static) -> Self {
        self.from = Some(Box::new(bound));
        self
    }
}

impl<M> Anchor for First<M>
where
    M: Fn(&EntryView<'_>) -> bool + Send + Sync,
{
    fn index(&self, views: &[EntryView<'_>]) -> usize {
        let start = match &self.from {
            Some(bound) => bound.index(views).min(views.len()),
            None => 0,
        };
        views[start..]
            .iter()
            .enumerate()
            .filter(|(_, v)| (self.pred)(v))
            .nth(self.skip)
            .map_or(views.len(), |(i, _)| start + i)
    }
}

/// The last in-range entry satisfying a predicate.
/// If no entry matches (or `skip` exceeds the number of matches), anchors at index `0`.
///
/// You can [`skip`](Self::skip) `n` matches before the last, or restrict the backward scan to
/// entries before another anchor with [`from`](Self::from).
pub struct Last<M> {
    pred: M,
    skip: usize,
    from: Option<Box<dyn Anchor>>,
}

impl Last<fn(&EntryView<'_>) -> bool> {
    /// Anchors at the last step marker.
    pub fn step() -> Self {
        Self::matching(|v| matches!(v, EntryView::StepMarker))
    }

    /// Anchors at the last turn marker.
    pub fn turn() -> Self {
        Self::matching(|v| matches!(v, EntryView::TurnMarker))
    }
}

impl<M> Last<M> {
    /// Anchors at the last entry satisfying `pred`.
    pub fn matching(pred: M) -> Self {
        Self {
            pred,
            skip: 0,
            from: None,
        }
    }

    /// Steps backward `n` further matches, anchoring at the `n`-th match.
    pub fn skip(mut self, n: usize) -> Self {
        self.skip = n;
        self
    }

    /// Scans backward from `bound`, considering only entries up to it.
    pub fn from(mut self, bound: impl Anchor + 'static) -> Self {
        self.from = Some(Box::new(bound));
        self
    }
}

impl<M> Anchor for Last<M>
where
    M: Fn(&EntryView<'_>) -> bool + Send + Sync,
{
    fn index(&self, views: &[EntryView<'_>]) -> usize {
        let end = match &self.from {
            Some(bound) => bound.index(views).min(views.len()),
            None => views.len(),
        };
        views[..end]
            .iter()
            .enumerate()
            .rev()
            .filter(|(_, v)| (self.pred)(v))
            .nth(self.skip)
            .map_or(0, |(i, _)| i)
    }
}

#[cfg(test)]
mod tests {
    use super::{Anchor, First, Last};
    use crate::schemas::{BodyMessage, ContentExt, UserMessage};
    use crate::transcript::EntryView;

    fn user_msg(text: &str) -> BodyMessage {
        BodyMessage::User(UserMessage::new(text))
    }

    #[test]
    fn from_restricts_scan() {
        let (a, b) = (user_msg("x"), user_msg("x"));
        let views = vec![
            EntryView::Message(uuid::Uuid::new_v4(), &a), // 0: match
            EntryView::StepMarker,                        // 1
            EntryView::Message(uuid::Uuid::new_v4(), &b), // 2: match
        ];
        let is_x = |v: &EntryView<'_>| matches!(v, EntryView::Message(_, BodyMessage::User(u)) if u.content.to_text().as_deref() == Some("x"));

        // Unbounded, the last match is index 2; scanning back from the last step marker restricts
        // to entries before it, so the last match is index 0 (the final `x` before the marker).
        assert_eq!(Last::matching(is_x).index(&views), 2);
        assert_eq!(Last::matching(is_x).from(Last::step()).index(&views), 0);
    }

    #[test]
    fn first_mirrors_last() {
        let (a, b) = (user_msg("x"), user_msg("x"));
        let views = vec![
            EntryView::Message(uuid::Uuid::new_v4(), &a), // 0: match
            EntryView::StepMarker,                        // 1
            EntryView::Message(uuid::Uuid::new_v4(), &b), // 2: match
        ];
        let is_x = |v: &EntryView<'_>| matches!(v, EntryView::Message(_, BodyMessage::User(u)) if u.content.to_text().as_deref() == Some("x"));

        // The first match is index 0; scanning forward from the last step marker gives index 2.
        assert_eq!(First::matching(is_x).index(&views), 0);
        assert_eq!(First::matching(is_x).from(Last::step()).index(&views), 2);
    }
}
