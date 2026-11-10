use async_openai::config::Config;

use crate::openai_responses::client::responses::Responses;

mod responses;

pub use crate::openai_chat::Client;

impl<C: Config> Client<C> {
    /// Returns a handle for the responses endpoint.
    pub fn responses(&self) -> Responses<'_, C> {
        Responses::new(self)
    }
}
