use async_openai::error::{
    ApiError as OpenAIApiError, OpenAIError, StreamError as OpenAIStreamError,
};

use crate::error::{ApiError, Error, StreamError};

impl From<OpenAIError> for Error<OpenAIApiError> {
    fn from(err: OpenAIError) -> Self {
        match err {
            OpenAIError::Reqwest(e) => Error::Reqwest(e),
            OpenAIError::ApiError(e) => {
                Error::Api(ApiError::with_status(e.status_code, e.api_error))
            }
            OpenAIError::JSONDeserialize(e, original) => Error::JSONDeserialize(e, original),
            OpenAIError::StreamError(e) => {
                let stream_error = match *e {
                    OpenAIStreamError::UnknownEvent(evt) => {
                        StreamError::InvalidEvent(format!("unknown event: {evt:?}"))
                    }
                    OpenAIStreamError::EventStream(msg) => StreamError::EventSource(msg),
                };
                Error::StreamError(stream_error)
            }
            OpenAIError::InvalidArgument(msg) => Error::InvalidArgument(msg),
            other => Error::Other(other.to_string()),
        }
    }
}
