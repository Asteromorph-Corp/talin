use async_trait::async_trait;
use reqwest::Method;
use serde::Serialize;

use crate::error::Error;
use crate::sglang_generate::ClientConfig;
use crate::sglang_generate::types::{Batched, GenerateRequest, GenerateResponse};
use crate::{ApiClient, Provider};

/// The SGLang generate API.
#[derive(Debug, Clone, Copy)]
pub struct Api;

/// The client for the SGLang generate API.
pub type Client = ApiClient<Api>;

#[async_trait]
impl Provider for Api {
    type Config = ClientConfig;
    type Error = serde_json::Value;

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
    /// Generates a response from the API using the given request.
    pub async fn generate(
        &self,
        request: &GenerateRequest,
    ) -> Result<Batched<GenerateResponse>, Error<serde_json::Value>> {
        self.post("/generate", request).await
    }
}
