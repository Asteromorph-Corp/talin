//! Mock HTTP server for testing provider requests.

use mockito::{Matcher, Mock, Server, ServerGuard};
use serde_json::Value;

/// A local HTTP server that answers a provider request with a canned body.
pub struct MockServer {
    server: ServerGuard,
}

impl MockServer {
    pub async fn start() -> Self {
        Self {
            server: Server::new_async().await,
        }
    }

    /// Base URL to feed each builder's `with_api_base`.
    pub fn base_url(&self) -> String {
        self.server.url()
    }

    /// A plain client for `with_http_client`; the mock owns the transport behavior.
    pub fn http_client(&self) -> reqwest::Client {
        reqwest::Client::new()
    }

    /// Registers a canned `200` JSON response for `POST {path}` whose request body must match
    /// `body`.
    pub async fn expect_post(&mut self, path: &str, request: Matcher, response: Value) -> Mock {
        self.server
            .mock("POST", path)
            .match_query(Matcher::Any) // path is asserted exactly; query (e.g. Gemini's `?key=`) is not
            .match_body(request)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(response.to_string())
            .create_async()
            .await
    }
}
