use std::collections::BTreeMap;

/// A utility trait to merge config values.
pub(crate) trait Merge<T>: Sized {
    /// Merges this config with a parent agent config.
    /// Child values override parent values.
    fn merge(self, parent: T) -> Self;
}

impl<T> Merge<Self> for Vec<T> {
    fn merge(self, mut parent: Self) -> Self {
        parent.extend(self);
        parent
    }
}

impl<K: Ord, V> Merge<Self> for BTreeMap<K, V> {
    fn merge(self, mut parent: Self) -> Self {
        parent.extend(self);
        parent
    }
}

impl<T: Merge<T>> Merge<Self> for Option<T> {
    fn merge(self, parent: Self) -> Self {
        match (self, parent) {
            (Some(child), Some(parent)) => Some(child.merge(parent)),
            (Some(child), None) => Some(child),
            (None, Some(parent)) => Some(parent),
            (None, None) => None,
        }
    }
}
