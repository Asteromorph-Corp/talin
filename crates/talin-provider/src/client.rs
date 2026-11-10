//! The HTTP client shared by non-OpenAI providers.

use std::future::Future;

use backon::Retryable;
use reqwest::Method;
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::error::{ApiError, Error};
use crate::stream::{ResponseStream, stream};
use crate::{Provider, ProviderError};

/// Creates an HTTP client that only speaks HTTP/1.
///
/// Some issues were reported when using HTTP/2 for streaming APIs.
pub(crate) fn http1_only_client() -> reqwest::Client {
    #[cfg(not(target_family = "wasm"))]
    {
        reqwest::Client::builder()
            .http1_only()
            .build()
            .expect("build shouldn't fail")
    }
    #[cfg(target_family = "wasm")]
    {
        reqwest::Client::new()
    }
}

/// The client for an API.
#[derive(Debug, Clone)]
pub struct ApiClient<P: Provider> {
    /// The HTTP client to use for requests.
    pub(crate) http: reqwest::Client,

    /// The configuration for the client.
    pub(crate) config: P::Config,

    /// The backoff strategy to use for retries.
    ///
    /// Defaults to [`backon::ExponentialBuilder::default`], which retries up to 3 times with
    /// delays starting at 1s and capped at 60s.
    backoff: backon::ExponentialBuilder,
}

impl<P> Default for ApiClient<P>
where
    P: Provider,
    P::Config: Default,
{
    fn default() -> Self {
        Self::with_config(P::Config::default())
    }
}

impl<P: Provider> ApiClient<P> {
    /// Creates a new [`ApiClient`] with the default configuration.
    pub fn new() -> Self
    where
        P::Config: Default,
    {
        Self::default()
    }

    /// Creates a new [`ApiClient`] with the given configuration.
    pub fn with_config(config: impl Into<P::Config>) -> Self {
        Self {
            http: P::default_http_client(),
            config: config.into(),
            backoff: Default::default(),
        }
    }

    /// Sets the HTTP client to use for requests.
    pub fn with_http_client(mut self, http: reqwest::Client) -> Self {
        self.http = http;
        self
    }

    /// Sets the exponential backoff configuration for retrying requests.
    pub fn with_exponential_backoff(mut self, backoff: backon::ExponentialBuilder) -> Self {
        self.backoff = backoff;
        self
    }

    /// Makes an HTTP request with exponential backoff and returns the validated response body.
    pub(crate) async fn execute<O, M, Fut>(&self, request_maker: M) -> Result<O, Error<P::Error>>
    where
        O: DeserializeOwned,
        M: Fn() -> Fut,
        Fut: Future<Output = Result<reqwest::Request, Error<P::Error>>>,
    {
        let response = self.send(request_maker).await?.json::<O>().await?;
        Ok(response)
    }

    /// Makes an HTTP request and returns a streaming response.
    pub(crate) async fn execute_stream<O>(
        &self,
        request: reqwest::Request,
    ) -> Result<ResponseStream<O, P::Error>, Error<P::Error>>
    where
        O: DeserializeOwned + Send + 'static,
        P::Error: Send + 'static,
    {
        let response = self.http.execute(request).await?;
        let response = check_status::<P>(response).await?;
        Ok(stream(response).await)
    }

    /// Makes an HTTP request with exponential backoff and returns the successful response.
    async fn send<M, Fut>(&self, request_maker: M) -> Result<reqwest::Response, Error<P::Error>>
    where
        M: Fn() -> Fut,
        Fut: Future<Output = Result<reqwest::Request, Error<P::Error>>>,
    {
        let http = self.http.clone();
        (|| async {
            let request = request_maker().await?;
            let response = http.execute(request).await?;
            check_status::<P>(response).await
        })
        .retry(self.backoff)
        .when(|err| matches!(err, Error::Api(api) if api.is_retryable()))
        .await
    }

    /// Makes a request with exponential backoff and returns the response body.
    pub(crate) async fn send_json<I, O>(
        &self,
        method: Method,
        path: &str,
        body: Option<&I>,
    ) -> Result<O, Error<P::Error>>
    where
        I: Serialize + Send + Sync,
        O: DeserializeOwned,
    {
        let request_maker = || async {
            P::build_request(&self.config, &self.http, method.clone(), path, body, false).await
        };
        self.execute(request_maker).await
    }

    /// Makes a request and returns a streaming response.
    pub(crate) async fn send_stream<I, O>(
        &self,
        method: Method,
        path: &str,
        body: Option<&I>,
    ) -> Result<ResponseStream<O, P::Error>, Error<P::Error>>
    where
        I: Serialize + Send + Sync,
        O: DeserializeOwned + Send + 'static,
        P::Error: Send + 'static,
    {
        let request = P::build_request(&self.config, &self.http, method, path, body, true).await?;
        self.execute_stream(request).await
    }

    /// Makes a `POST` request with exponential backoff and returns the response body.
    pub(crate) async fn post<I, O>(&self, path: &str, body: &I) -> Result<O, Error<P::Error>>
    where
        I: Serialize + Send + Sync,
        O: DeserializeOwned,
    {
        self.send_json(Method::POST, path, Some(body)).await
    }

    /// Makes a `POST` request and returns a streaming response.
    pub(crate) async fn post_stream<I, O>(
        &self,
        path: &str,
        body: &I,
    ) -> Result<ResponseStream<O, P::Error>, Error<P::Error>>
    where
        I: Serialize + Send + Sync,
        O: DeserializeOwned + Send + 'static,
    {
        self.send_stream(Method::POST, path, Some(body)).await
    }
}

/// Returns the response if its status is a success, and the provider's [`ApiError`] otherwise.
async fn check_status<P: Provider>(
    response: reqwest::Response,
) -> Result<reqwest::Response, Error<P::Error>> {
    let status = response.status();
    if status.is_success() {
        return Ok(response);
    }

    let body = response.text().await.unwrap_or_default();
    Err(ApiError::with_status(status, P::Error::parse(status, &body)).into())
}
