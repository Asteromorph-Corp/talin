//! Server-sent event streaming shared by every provider client.

use futures::StreamExt;
use reqwest::Response;
use serde::de::DeserializeOwned;

use crate::error::Error;

mod merge;
pub(crate) use merge::{Merge, PreferNew, PreferOld};

mod partial;
pub(crate) use partial::{PartialResponse, or_partial};

/// A parsed server-sent event stream, yielding values until the `[DONE]` sentinel is received
/// from the server.
#[cfg(not(target_family = "wasm"))]
pub type ResponseStream<T, E> =
    std::pin::Pin<Box<dyn futures::Stream<Item = Result<T, Error<E>>> + Send>>;

/// A parsed server-sent event stream, yielding values until the `[DONE]` sentinel is received
/// from the server.
#[cfg(target_family = "wasm")]
pub type ResponseStream<T, E> = std::pin::Pin<Box<dyn futures::Stream<Item = Result<T, Error<E>>>>>;

/// Trait to support WASM and native targets.
#[cfg(not(target_family = "wasm"))]
pub(crate) trait MaybeSend: Send {}
#[cfg(not(target_family = "wasm"))]
impl<T> MaybeSend for T where T: Send {}

#[cfg(target_family = "wasm")]
pub(crate) trait MaybeSend {}
#[cfg(target_family = "wasm")]
impl<T> MaybeSend for T {}

/// Parses an HTTP response body as a stream of
/// [server-sent events](https://developer.mozilla.org/en-US/docs/Web/API/Server-sent_events/Using_server-sent_events#event_stream_format),
/// deserializing each event's `data` payload into `O`.
pub(crate) async fn stream<O, E>(response: Response) -> ResponseStream<O, E>
where
    O: DeserializeOwned + MaybeSend + 'static,
    E: MaybeSend + 'static,
{
    stream_mapped_raw_events(response, |event| {
        serde_json::from_str::<O>(&event.data).map_err(|error| {
            let data = String::from_utf8_lossy(event.data.as_bytes()).into_owned();
            Error::JSONDeserialize(error, data)
        })
    })
    .await
}

#[cfg(target_family = "wasm")]
async fn stream_mapped_raw_events<O, E>(
    response: Response,
    event_mapper: impl Fn(eventsource_stream::Event) -> Result<O, Error<E>> + 'static,
) -> ResponseStream<O, E>
where
    O: DeserializeOwned + 'static,
    E: 'static,
{
    let byte_stream = response.bytes_stream();
    let event_stream = Box::pin(eventsource_stream::EventStream::new(byte_stream));

    Box::pin(futures::stream::unfold(
        (event_stream, event_mapper),
        |(mut event_stream, event_mapper)| async move {
            let event = match event_stream.next().await {
                Some(Ok(event)) => event,
                Some(Err(error)) => {
                    return Some((Err(Error::from(error)), (event_stream, event_mapper)));
                }
                None => return None,
            };

            if event.data == "[DONE]" {
                return None;
            }

            let response = event_mapper(event);
            Some((response, (event_stream, event_mapper)))
        },
    ))
}

#[cfg(not(target_family = "wasm"))]
async fn stream_mapped_raw_events<O, E>(
    response: Response,
    event_mapper: impl Fn(eventsource_stream::Event) -> Result<O, Error<E>> + Send + 'static,
) -> ResponseStream<O, E>
where
    O: DeserializeOwned + Send + 'static,
    E: Send + 'static,
{
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

    tokio::spawn(async move {
        let byte_stream = response.bytes_stream();
        let mut event_stream = std::pin::pin!(eventsource_stream::EventStream::new(byte_stream));

        while let Some(ev) = event_stream.next().await {
            let event = match ev {
                Ok(e) => e,
                Err(e) => {
                    drop(tx.send(Err(Error::from(e))));
                    break;
                }
            };
            if event.data == "[DONE]" {
                break;
            }

            let response = event_mapper(event);

            if tx.send(response).is_err() {
                break;
            }
        }
    });

    Box::pin(tokio_stream::wrappers::UnboundedReceiverStream::new(rx))
}
