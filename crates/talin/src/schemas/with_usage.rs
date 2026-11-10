use crate::schemas::Usage;

/// An output with the token usage information.
///
/// # Fields
/// - [`content`](Self::content): The actual output content.
/// - [`usage`](Self::usage): The token usage information.
#[derive(Debug, Clone)]
pub struct WithUsage<O> {
    /// The actual output content.
    pub content: O,
    /// The token usage information.
    pub usage: Usage,
}

impl<O> WithUsage<O> {
    /// Pairs the content with the given token usage.
    pub fn new(content: O, usage: Usage) -> Self {
        Self { content, usage }
    }
}

/// A trait to convert a value into a `WithUsage`.
///
/// The trait is implemented for all types.
pub trait IntoWithUsage<T> {
    /// Wraps the value with the given token usage information.
    fn with_usage(self, usage: Usage) -> WithUsage<T>;
}

impl<T> IntoWithUsage<T> for T {
    fn with_usage(self, usage: Usage) -> WithUsage<T> {
        WithUsage {
            content: self,
            usage,
        }
    }
}
