use async_trait::async_trait;
use reqwest::Method;
use serde::Serialize;

use crate::client::http1_only_client;
use crate::error::Error;
use crate::xai_responses::ClientConfig;
use crate::xai_responses::client::responses::Responses;
use crate::{ApiClient, Provider};

mod responses;

/// The xAI Responses API.
#[derive(Debug, Clone, Copy)]
pub struct Api;
/// The client for the xAI Responses API.
pub type Client = ApiClient<Api>;

#[async_trait]
impl Provider for Api {
    type Config = ClientConfig;
    type Error = serde_json::Value;

    /// Uses an HTTP/1 only client, as issues were reported when using HTTP/2 for the streaming api.
    fn default_http_client() -> reqwest::Client {
        http1_only_client()
    }

    async fn build_request<I: Serialize + Send + Sync>(
        config: &ClientConfig,
        http: &reqwest::Client,
        method: Method,
        path: &str,
        body: Option<&I>,
        _streaming: bool,
    ) -> Result<reqwest::Request, Error<serde_json::Value>> {
        let mut builder = http
            .request(method, config.url(path))
            .headers(config.headers()?);
        if let Some(body) = body {
            builder = builder.json(body);
        }
        builder.build().map_err(Error::from)
    }
}

impl Client {
    /// Creates an intermediate object for making requests to the `/responses` endpoint.
    pub fn responses(&self) -> Responses<'_> {
        Responses::new(self)
    }
}
