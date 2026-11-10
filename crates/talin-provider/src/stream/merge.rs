/// A utility trait for merging two values.
pub(crate) trait Merge<T> {
    /// Merges `rhs` into `self`.
    fn merge(&mut self, rhs: T);
}

/// Merges two `Option`s and prefer the existing value (`self`) over the new value (`rhs`).
pub(crate) struct PreferOld<T>(pub Option<T>);
/// Merges two `Option`s and prefer the new value (`rhs`) over the existing value (`self`).
pub(crate) struct PreferNew<T>(pub Option<T>);

/// Merges two `u32` by replacing with the rhs value.
impl Merge<u32> for u32 {
    fn merge(&mut self, rhs: u32) {
        *self = rhs;
    }
}

/// Merges two `i32` by replacing with the rhs value.
impl Merge<i32> for i32 {
    fn merge(&mut self, rhs: i32) {
        *self = rhs;
    }
}

/// Merges two `i64` by replacing with the rhs value.
impl Merge<i64> for i64 {
    fn merge(&mut self, rhs: i64) {
        *self = rhs;
    }
}

/// Merges two `f32` by replacing with the rhs value.
impl Merge<f32> for f32 {
    fn merge(&mut self, rhs: f32) {
        *self = rhs;
    }
}

/// Merges two `f64` by replacing with the rhs value.
impl Merge<f64> for f64 {
    fn merge(&mut self, rhs: f64) {
        *self = rhs;
    }
}

/// Merges two `String` values by concatenating them.
impl Merge<String> for String {
    fn merge(&mut self, rhs: String) {
        self.push_str(&rhs);
    }
}

/// Merges two `Vec<T>` values by extending the left vector with the right vector.
impl<T> Merge<Vec<T>> for Vec<T> {
    fn merge(&mut self, rhs: Vec<T>) {
        self.extend(rhs);
    }
}

/// Merges two `Option<T>` values by merging the inner values if both are `Some`,
/// or taking the `Some` value if one is `None`.
impl<T: Merge<T>> Merge<Option<T>> for Option<T> {
    fn merge(&mut self, rhs: Option<T>) {
        let Some(rhs) = rhs else {
            return;
        };

        match self {
            Some(lhs) => lhs.merge(rhs),
            None => *self = Some(rhs),
        }
    }
}

impl<T: Merge<T>> Merge<T> for Option<T> {
    fn merge(&mut self, rhs: T) {
        match self {
            Some(lhs) => lhs.merge(rhs),
            None => *self = Some(rhs),
        }
    }
}

impl<T> Merge<PreferNew<T>> for Option<T> {
    fn merge(&mut self, rhs: PreferNew<T>) {
        if let Some(value) = rhs.0 {
            *self = Some(value);
        }
    }
}

impl<T> Merge<PreferOld<T>> for Option<T> {
    fn merge(&mut self, rhs: PreferOld<T>) {
        if self.is_none() {
            *self = rhs.0;
        }
    }
}
