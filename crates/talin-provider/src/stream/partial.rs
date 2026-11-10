use crate::error::Error;

/// A partial response that may contain some data before an error occurred.
pub(crate) struct PartialResponse<R, E> {
    /// The accumulated response data before the error occurred.
    pub response: Option<Box<R>>,
    /// The error that occurred while processing the stream.
    pub error: Error<E>,
}

impl<R, E> PartialResponse<R, E> {
    /// Constructs a new [`PartialResponse`] with the given response and error.
    pub(crate) fn new(response: Option<R>, error: impl Into<Error<E>>) -> Self {
        Self {
            response: response.map(Box::new),
            error: error.into(),
        }
    }
}

macro_rules! or_partial {
    ($body:expr) => {
        match $body {
            Ok(val) => val,
            Err(e) => return Err(PartialResponse::new(None, e)),
        }
    };
    ($body:expr, $partial:expr) => {
        match $body {
            Ok(val) => val,
            Err(e) => return Err(PartialResponse::new(Some($partial), e)),
        }
    };
    ($body:expr, ? $partial:expr) => {
        match $body {
            Ok(val) => val,
            Err(e) => return Err(PartialResponse::new($partial, e)),
        }
    };
}
pub(crate) use or_partial;
