/// A utility trait to convert empty collections or strings into None.
pub(crate) trait NoneIfEmpty: Sized {
    /// Returns None if the collection or string is empty, otherwise returns Some(self).
    fn none_if_empty(self) -> Option<Self>;
}

impl<T> NoneIfEmpty for Vec<T> {
    fn none_if_empty(self) -> Option<Self> {
        if self.is_empty() { None } else { Some(self) }
    }
}

impl NoneIfEmpty for String {
    fn none_if_empty(self) -> Option<Self> {
        if self.is_empty() { None } else { Some(self) }
    }
}

impl NoneIfEmpty for &str {
    fn none_if_empty(self) -> Option<Self> {
        if self.is_empty() { None } else { Some(self) }
    }
}
