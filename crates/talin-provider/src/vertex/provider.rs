use async_trait::async_trait;
use google_cloud_auth::credentials::{AccessToken, AccessTokenCredentials, Builder};
use google_cloud_auth::errors::CredentialsError;
use tokio::sync::OnceCell;

use crate::vertex::VertexTokenProvider;

const CLOUD_PLATFORM_SCOPE: &str = "https://www.googleapis.com/auth/cloud-platform";

/// A newtype wrapper around [`AccessTokenCredentials`] that implements [`VertexTokenProvider`].
///
/// `AccessTokenCredentials` does not implement [`AccessTokenCredentialsProvider`], so a blanket
/// impl doesn't cover it.
///
/// [`AccessTokenCredentialsProvider`]: google_cloud_auth::credentials::AccessTokenCredentialsProvider
#[repr(transparent)]
pub struct VertexCredentials(pub AccessTokenCredentials);

/// A lazy-initialized wrapper around [`AccessTokenCredentials`] that implements
/// [`VertexTokenProvider`].
///
/// This struct initializes the `AccessTokenCredentials` lazily on the first call to
/// [`access_token`], so the initialization of this struct itself is infallible.
///
/// [`access_token`]: VertexTokenProvider::access_token
pub struct LazyVertexCredentials {
    scopes: Vec<String>,
    inner: OnceCell<AccessTokenCredentials>,
}

impl VertexCredentials {
    /// Creates credentials from Application Default Credentials with the given OAuth scopes.
    pub fn new(scopes: Vec<String>) -> Result<Self, google_cloud_auth::build_errors::Error> {
        let credentials = Builder::default()
            .with_scopes(scopes)
            .build_access_token_credentials()?;
        Ok(Self(credentials))
    }

    /// Creates credentials with the default `cloud-platform` scope.
    pub fn with_default_scopes() -> Result<Self, google_cloud_auth::build_errors::Error> {
        Self::new(vec![CLOUD_PLATFORM_SCOPE.to_string()])
    }
}

impl LazyVertexCredentials {
    /// Creates lazy credentials from Application default credentials with the given OAuth scopes.
    ///
    /// The credentials are only initialized when fetching the `access_token` for the first time.
    pub fn new(scopes: Vec<String>) -> Self {
        Self {
            scopes,
            inner: OnceCell::new(),
        }
    }

    /// Creates lazy credentials with the default `cloud-platform` scope.
    pub fn with_default_scopes() -> Self {
        Self::new(vec![CLOUD_PLATFORM_SCOPE.to_string()])
    }
}

impl Default for LazyVertexCredentials {
    fn default() -> Self {
        Self::with_default_scopes()
    }
}

#[async_trait]
impl VertexTokenProvider for VertexCredentials {
    async fn access_token(&self) -> Result<AccessToken, CredentialsError> {
        self.0.access_token().await
    }
}

#[async_trait]
impl VertexTokenProvider for LazyVertexCredentials {
    async fn access_token(&self) -> Result<AccessToken, CredentialsError> {
        let auth = self
            .inner
            .get_or_try_init(|| async {
                Builder::default()
                    .with_scopes(&self.scopes)
                    .build_access_token_credentials()
                    .map_err(|e| CredentialsError::from_source(false, e))
            })
            .await?;
        auth.access_token().await
    }
}
