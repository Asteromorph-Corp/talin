//! Strategies and helpers for tracking token usage.

use std::convert::Infallible;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::{Mutex, RwLock};

use crate::chain::OutputCtor;
use crate::llm::Llm;
use crate::schemas::Usage;
use crate::strategy::Strategy;

/// A [`Strategy`] that tracks usage for a turn.
///
/// Accumulates token usage across every LLM call in the turn.
///
/// Returns the accumulated usage upon finalization.
#[derive(Default)]
pub struct UsageTrackStrategy {
    usage: Usage,
}

#[async_trait]
impl<L: Llm, O: OutputCtor> Strategy<L, O> for UsageTrackStrategy {
    type Output = Usage;
    type Error = Infallible;

    async fn inspect_usage(&mut self, usage: &Usage) -> Result<(), Self::Error> {
        self.usage.merge_inplace(usage);
        Ok(())
    }

    async fn finalize(self) -> Result<Self::Output, Self::Error> {
        Ok(self.usage)
    }
}

/// A [`Strategy`] that tracks usage for a turn and merges it into a shared usage object.
///
/// Accumulates token usage across every LLM call in the turn.
/// The usage is merged into the shared usage object upon finalization.
///
/// Returns the (individual) accumulated usage upon finalization.
#[derive(Debug, Clone, Default)]
pub struct SharedUsageTrackStrategy<S: SharedUsage> {
    shared_usage: S,
    individual_usage: Usage,
}

impl<S: SharedUsage> SharedUsageTrackStrategy<S> {
    /// Create a new [`SharedUsageTrackStrategy`] with the given shared usage object.
    pub fn new(shared: S) -> Self {
        Self {
            shared_usage: shared,
            individual_usage: Usage::default(),
        }
    }

    /// Returns a reference to the shared usage object.
    pub fn get_shared_usage(&self) -> &S {
        &self.shared_usage
    }
}

#[async_trait]
impl<L: Llm, O: OutputCtor, S: SharedUsage> Strategy<L, O> for SharedUsageTrackStrategy<S> {
    type Output = Usage;
    type Error = S::Error;

    async fn inspect_usage(&mut self, usage: &Usage) -> Result<(), Self::Error> {
        self.individual_usage.merge_inplace(usage);
        Ok(())
    }

    async fn finalize(self) -> Result<Self::Output, Self::Error> {
        self.shared_usage.merge(&self.individual_usage).await?;
        Ok(self.individual_usage)
    }
}

/// A trait for shared usage objects that can be used with [`SharedUsageTrackStrategy`].
#[async_trait]
pub trait SharedUsage: Send + Sync {
    /// Error returned when reading or merging the shared usage fails.
    type Error: std::error::Error + Send + Sync + 'static;

    /// Get the current usage from the shared usage object.
    async fn get(&self) -> Result<Usage, Self::Error>;
    /// Merge the given usage into the shared usage object.
    async fn merge(&self, usage: &Usage) -> Result<(), Self::Error>;
}

#[async_trait]
impl<T: SharedUsage + ?Sized> SharedUsage for Arc<T> {
    type Error = T::Error;

    async fn get(&self) -> Result<Usage, Self::Error> {
        (**self).get().await
    }
    async fn merge(&self, usage: &Usage) -> Result<(), Self::Error> {
        (**self).merge(usage).await
    }
}

#[async_trait]
impl SharedUsage for Mutex<Usage> {
    type Error = Infallible;

    async fn get(&self) -> Result<Usage, Self::Error> {
        let usage = self.lock().await;
        Ok(usage.clone())
    }

    async fn merge(&self, usage: &Usage) -> Result<(), Self::Error> {
        let mut total = self.lock().await;
        total.merge_inplace(usage);
        Ok(())
    }
}

#[async_trait]
impl SharedUsage for RwLock<Usage> {
    type Error = Infallible;

    async fn get(&self) -> Result<Usage, Self::Error> {
        let usage = self.read().await;
        Ok(usage.clone())
    }

    async fn merge(&self, usage: &Usage) -> Result<(), Self::Error> {
        let mut total = self.write().await;
        total.merge_inplace(usage);
        Ok(())
    }
}
