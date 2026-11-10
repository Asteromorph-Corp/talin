//! Support for authenticating against Google Vertex AI.
//!
//! Currently, the crate only supports Claude models from Vertex AI.
//! You can use [`claude_messages::VertexConfig`](crate::claude_messages::VertexConfig) to configure
//! the Vertex AI client for Claude models.

use async_trait::async_trait;
use google_cloud_auth::credentials::AccessTokenCredentialsProvider;

mod provider;

pub use google_cloud_auth::credentials::AccessToken;
pub use google_cloud_auth::errors::CredentialsError;
pub use provider::*;

/// A source of Vertex OAuth access tokens, mirrors the [`AccessTokenCredentialsProvider`] trait
/// from [`google-cloud-auth`](google_cloud_auth).
///
/// # Motivations
/// [`AccessTokenCredentials`] does not implement the `AccessTokenCredentialsProvider` trait
/// directly. Therefore, you can't use `AccessTokenCredentialsProvider` as a trait bound and accept
/// `AccessTokenCredentials` as an argument. This trait is a workaround for that limitation.
///
/// # Implementors
/// - Implemented for all types that implement `AccessTokenCredentialsProvider`
/// - `VertexCredentials`: A wrapper around `AccessTokenCredentials` that implements this trait,
///   which addresses the aforementioned limitation. Requires feature `providers`.
/// - `LazyVertexCredentials`: Lazy-initializes the underlying `AccessTokenCredentials` to make
///   initialization infallible. Requires feature `providers`.
///
/// [`AccessTokenCredentials`]: google_cloud_auth::credentials::AccessTokenCredentials
#[async_trait]
pub trait VertexTokenProvider: Send + Sync {
    /// Returns the current access token, refreshing it if necessary.
    async fn access_token(&self) -> Result<AccessToken, CredentialsError>;
}

#[async_trait]
impl<T> VertexTokenProvider for T
where
    T: AccessTokenCredentialsProvider + Send + Sync,
{
    async fn access_token(&self) -> Result<AccessToken, CredentialsError> {
        self.access_token().await
    }
}

#[async_trait]
impl VertexTokenProvider for Box<dyn VertexTokenProvider> {
    async fn access_token(&self) -> Result<AccessToken, CredentialsError> {
        self.as_ref().access_token().await
    }
}
