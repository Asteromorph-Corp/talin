use std::convert::Infallible;

use async_trait::async_trait;
use tokio::sync::Mutex;

/// Backing storage for a [`BufferedFinalAnswerStrategy`](super::BufferedFinalAnswerStrategy)'s
/// accumulated answer.
#[async_trait]
pub trait FinalAnswerBuffer: Send + Sync {
    /// Error returned when reading from or writing to the buffer fails.
    type Error: std::error::Error + Send + Sync + 'static;

    /// Reads the answer accumulated so far, or the empty string if nothing has been written yet.
    async fn read(&self) -> Result<String, Self::Error>;

    /// Overwrites the accumulated answer.
    async fn write(&self, content: String) -> Result<(), Self::Error>;
}

/// The default [`FinalAnswerBuffer`]: an in-process buffer that lives as long as the strategy.
#[derive(Default)]
pub struct InMemoryBuffer(pub(crate) Mutex<String>);

#[async_trait]
impl FinalAnswerBuffer for InMemoryBuffer {
    type Error = Infallible;

    async fn read(&self) -> Result<String, Self::Error> {
        Ok(self.0.lock().await.clone())
    }

    async fn write(&self, content: String) -> Result<(), Self::Error> {
        *self.0.lock().await = content;
        Ok(())
    }
}
