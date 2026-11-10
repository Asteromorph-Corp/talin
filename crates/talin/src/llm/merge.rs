use std::collections::HashMap;
use std::hash::Hash;

pub(super) trait Merge: Sized {
    /// Merges `other` into `self`.
    ///
    /// `other` takes precedence over `self` in case of conflicts.
    fn merge(mut self, other: Self) -> Self {
        self.merge_inplace(other);
        self
    }

    /// Merges `other` into `self` in-place.
    ///
    /// `other` takes precedence over `self` in case of conflicts.
    fn merge_inplace(&mut self, other: Self);
}

impl<T: Merge> Merge for Option<T> {
    fn merge_inplace(&mut self, other: Self) {
        match (self.as_mut(), other) {
            (Some(value), Some(other)) => value.merge_inplace(other),
            (None, other @ Some(_)) => *self = other,
            (_, None) => {}
        }
    }
}

impl<T> Merge for Vec<T> {
    /// Appends the items of `other` to the items of `self`.
    fn merge_inplace(&mut self, other: Self) {
        self.extend(other);
    }
}

impl<K: Eq + Hash, V> Merge for HashMap<K, V> {
    /// Inserts the entries of `other`, overwriting the entries of `self` with the same key.
    fn merge_inplace(&mut self, other: Self) {
        self.extend(other);
    }
}
