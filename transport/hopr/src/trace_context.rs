//! Local tracing context carried across the shared outgoing queues.

use std::{
    pin::Pin,
    task::{Context, Poll},
};

use futures::Sink;
use hopr_utils::network_types::crossfire_sink::CrossfireSink;

/// A value and the local span under which its asynchronous work should run.
/// This envelope is never serialized or sent to a remote node.
#[derive(Debug)]
pub struct WithContext<T> {
    pub(crate) value: T,
    pub(crate) span: tracing::Span,
}

impl<T> From<T> for WithContext<T> {
    fn from(value: T) -> Self {
        Self {
            value,
            span: tracing::Span::current(),
        }
    }
}

/// A bounded sender that captures the producer's span when a message is enqueued.
#[derive(Debug)]
pub struct ContextSender<T: Send + 'static> {
    inner: CrossfireSink<WithContext<T>>,
}

impl<T: Send + 'static> Clone for ContextSender<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T: Send + 'static> ContextSender<T> {
    pub(crate) fn new(inner: CrossfireSink<WithContext<T>>) -> Self {
        Self { inner }
    }

    /// Attempts to enqueue without waiting, retaining the original value on failure.
    pub fn try_send(&self, item: T) -> Result<(), crossfire::TrySendError<T>> {
        self.inner.try_send(item.into()).map_err(|error| match error {
            crossfire::TrySendError::Full(message) => crossfire::TrySendError::Full(message.value),
            crossfire::TrySendError::Disconnected(message) => crossfire::TrySendError::Disconnected(message.value),
        })
    }
}

impl<T: Send + 'static> Sink<T> for ContextSender<T>
where
    CrossfireSink<WithContext<T>>: Sink<WithContext<T>> + Unpin,
{
    type Error = <CrossfireSink<WithContext<T>> as Sink<WithContext<T>>>::Error;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.get_mut().inner).poll_ready(cx)
    }

    fn start_send(self: Pin<&mut Self>, item: T) -> Result<(), Self::Error> {
        Pin::new(&mut self.get_mut().inner).start_send(item.into())
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.get_mut().inner).poll_flush(cx)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.get_mut().inner).poll_close(cx)
    }
}

#[cfg(test)]
mod tests {
    use futures::StreamExt;
    use hopr_utils::network_types::crossfire_sink::bounded_sink_channel;

    use super::*;

    #[tokio::test]
    async fn sender_clones_should_share_capacity_and_return_rejected_values() -> anyhow::Result<()> {
        let (tx, rx) = bounded_sink_channel(1);
        let tx = ContextSender::new(tx);
        let clone = tx.clone();
        tx.try_send(1u8)?;
        assert!(matches!(clone.try_send(2), Err(crossfire::TrySendError::Full(2))));
        futures::pin_mut!(rx);
        assert_eq!(rx.next().await.map(|message| message.value), Some(1));
        clone.try_send(3)?;
        Ok(())
    }
}
