//! Local tracing context carried across the shared outgoing queues.

use std::{
    pin::Pin,
    sync::atomic::{AtomicU64, Ordering},
    task::{Context, Poll},
};

use futures::Sink;
use hopr_utils::network_types::crossfire_sink::CrossfireSink;

pub(crate) fn next_packet_id() -> u64 {
    static NEXT_PACKET_ID: AtomicU64 = AtomicU64::new(1);
    NEXT_PACKET_ID.fetch_add(1, Ordering::Relaxed)
}

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
            #[cfg(feature = "tracing-context")]
            span: tracing::Span::current(),
            #[cfg(not(feature = "tracing-context"))]
            span: tracing::Span::none(),
        }
    }
}

/// Restores each message's span while polling a downstream sink.
pub(crate) struct ContextSink<S> {
    inner: S,
    span: tracing::Span,
}

impl<S> ContextSink<S> {
    pub(crate) fn new(inner: S) -> Self {
        Self {
            inner,
            span: tracing::Span::none(),
        }
    }
}

impl<S, T> Sink<WithContext<T>> for ContextSink<S>
where
    S: Sink<T> + Unpin,
{
    type Error = S::Error;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let this = self.get_mut();
        let _entered = this.span.enter();
        Pin::new(&mut this.inner).poll_ready(cx)
    }

    fn start_send(self: Pin<&mut Self>, item: WithContext<T>) -> Result<(), Self::Error> {
        let this = self.get_mut();
        this.span = item.span;
        let _entered = this.span.enter();
        Pin::new(&mut this.inner).start_send(item.value)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let this = self.get_mut();
        let _entered = this.span.enter();
        Pin::new(&mut this.inner).poll_flush(cx)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let this = self.get_mut();
        let _entered = this.span.enter();
        Pin::new(&mut this.inner).poll_close(cx)
    }
}

/// A bounded sender that captures the producer's span when a message is enqueued.
/// Capture requires `tracing-context`; without it, messages carry a disabled span.
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

    #[test]
    fn disconnected_sender_should_return_the_owned_message() {
        let (tx, rx) = bounded_sink_channel(1);
        let tx = ContextSender::new(tx);
        drop(rx);
        let message = vec![1u8, 2, 3];
        match tx.try_send(message.clone()) {
            Err(crossfire::TrySendError::Disconnected(returned)) => assert_eq!(returned, message),
            result => panic!("expected disconnected sender to return the message, got {result:?}"),
        }
    }

    #[tokio::test]
    #[serial_test::serial(routing_logs)]
    async fn sink_should_capture_the_producer_span_and_close_after_delivery() -> anyhow::Result<()> {
        use futures::SinkExt;
        use tracing::{Instrument, instrument::WithSubscriber};

        let logs = crate::tests::support::Logs::default();
        async {
            let (tx, rx) = bounded_sink_channel(1);
            let mut tx = ContextSender::new(tx);
            let producer = tracing::debug_span!("producer");
            let expected = if cfg!(feature = "tracing-context") {
                producer.id()
            } else {
                None
            };
            tx.send(vec![42u8]).instrument(producer).await?;
            futures::pin_mut!(rx);
            let message = rx
                .next()
                .await
                .ok_or_else(|| anyhow::anyhow!("missing queued message"))?;
            assert_eq!(message.value, [42]);
            assert_eq!(message.span.id(), expected);
            tx.close().await?;
            drop(tx);
            assert!(
                rx.next().await.is_none(),
                "closing must preserve delivery and then terminate"
            );
            anyhow::Ok(())
        }
        .with_subscriber(logs.subscriber(tracing::Level::DEBUG))
        .await
    }
}
