//! Tracing context carried across bounded queues shared by asynchronous producers.

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
    /// The queued value.
    pub value: T,
    span: Option<Box<tracing::Span>>,
}

impl<T> WithContext<T> {
    /// Attaches an explicit local span, independently of automatic capture.
    ///
    /// ```
    /// let message = hopr_utils_tracing::WithContext::new("payload", tracing::Span::none());
    /// let (value, span) = message.into_parts();
    /// assert_eq!(value, "payload");
    /// assert!(span.is_disabled());
    /// ```
    pub fn new(value: T, span: tracing::Span) -> Self {
        Self {
            value,
            // Keep disabled messages compact and allocation-free. Active spans retain their
            // subscriber and lifetime exactly as an inline Span would.
            span: (!span.is_disabled()).then(|| Box::new(span)),
        }
    }

    /// Borrows the retained span, or a disabled span when no context was captured.
    pub fn span(&self) -> &tracing::Span {
        static DISABLED: tracing::Span = tracing::Span::none();
        self.span.as_deref().unwrap_or(&DISABLED)
    }

    /// Recovers the value and its owned span without cloning either.
    pub fn into_parts(self) -> (T, tracing::Span) {
        (self.value, self.span.map_or_else(tracing::Span::none, |span| *span))
    }
}

impl<T> From<T> for WithContext<T> {
    fn from(value: T) -> Self {
        #[cfg(feature = "tracing-context")]
        let span = tracing::Span::current();
        #[cfg(not(feature = "tracing-context"))]
        let span = tracing::Span::none();
        Self::new(value, span)
    }
}

/// Restores message context while sending values and completing downstream I/O.
/// Readiness uses the caller's context. The last message's span is retained across pending
/// flush/close polls and released when either operation completes successfully.
pub struct ContextSink<S> {
    inner: S,
    span: tracing::Span,
}

impl<S> ContextSink<S> {
    /// Wraps a downstream sink that accepts the unwrapped message values.
    pub fn new(inner: S) -> Self {
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
        Pin::new(&mut this.inner).poll_ready(cx)
    }

    fn start_send(self: Pin<&mut Self>, item: WithContext<T>) -> Result<(), Self::Error> {
        let this = self.get_mut();
        let (value, span) = item.into_parts();
        this.span = span;
        let _entered = this.span.enter();
        Pin::new(&mut this.inner).start_send(value)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let this = self.get_mut();
        let poll = {
            let _entered = this.span.enter();
            Pin::new(&mut this.inner).poll_flush(cx)
        };
        if matches!(poll, Poll::Ready(Ok(()))) {
            this.span = tracing::Span::none();
        }
        poll
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let this = self.get_mut();
        let poll = {
            let _entered = this.span.enter();
            Pin::new(&mut this.inner).poll_close(cx)
        };
        if matches!(poll, Poll::Ready(Ok(()))) {
            this.span = tracing::Span::none();
        }
        poll
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
    /// Wraps a bounded queue sender whose receiver yields context envelopes.
    pub fn new(inner: CrossfireSink<WithContext<T>>) -> Self {
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

    #[derive(Default)]
    struct ObservedSink {
        events: Vec<(&'static str, Option<tracing::span::Id>)>,
        received: Vec<u8>,
        pending: bool,
    }

    impl ObservedSink {
        fn observe(&mut self, operation: &'static str) {
            self.events.push((operation, tracing::Span::current().id()));
        }

        fn poll_io(&mut self, operation: &'static str) -> Poll<Result<(), std::convert::Infallible>> {
            self.observe(operation);
            if std::mem::take(&mut self.pending) {
                Poll::Pending
            } else {
                Poll::Ready(Ok(()))
            }
        }
    }

    impl Sink<u8> for ObservedSink {
        type Error = std::convert::Infallible;

        fn poll_ready(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            self.get_mut().observe("ready");
            Poll::Ready(Ok(()))
        }

        fn start_send(self: Pin<&mut Self>, item: u8) -> Result<(), Self::Error> {
            let this = self.get_mut();
            this.observe("send");
            this.received.push(item);
            Ok(())
        }

        fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            self.get_mut().poll_io("flush")
        }

        fn poll_close(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            self.get_mut().poll_io("close")
        }
    }

    #[test]
    fn sink_readiness_should_use_the_caller_context_instead_of_the_previous_message() -> anyhow::Result<()> {
        tracing::subscriber::with_default(test_log::tracing_subscriber::registry(), || {
            let previous = tracing::debug_span!("previous_packet");
            let previous_id = previous.id();
            let caller = tracing::debug_span!("next_caller");
            let mut sink = ContextSink::new(ObservedSink::default());
            let mut cx = Context::from_waker(futures::task::noop_waker_ref());
            assert_eq!(Pin::new(&mut sink).poll_ready(&mut cx), Poll::Ready(Ok(())));
            Pin::new(&mut sink).start_send(WithContext::new(42, previous))?;
            caller.in_scope(|| {
                assert_eq!(Pin::new(&mut sink).poll_ready(&mut cx), Poll::Ready(Ok(())));
            });
            assert!(previous_id.is_some(), "fixture needs an active message span");
            assert!(caller.id().is_some(), "fixture needs an active caller span");
            assert_eq!(sink.inner.received, [42]);
            assert_eq!(
                sink.inner.events,
                [("ready", None), ("send", previous_id), ("ready", caller.id())]
            );
            anyhow::Ok(())
        })
    }

    #[test]
    fn completed_io_should_release_the_last_message_span_while_the_sink_stays_alive() -> anyhow::Result<()> {
        use std::sync::{
            Arc,
            atomic::{AtomicUsize, Ordering},
        };

        use test_log::tracing_subscriber::{self as subscriber, layer::SubscriberExt};

        struct Closures(Arc<AtomicUsize>);
        impl<S: tracing::Subscriber> subscriber::Layer<S> for Closures {
            fn on_close(&self, _: tracing::span::Id, _: subscriber::layer::Context<'_, S>) {
                self.0.fetch_add(1, Ordering::Relaxed);
            }
        }

        for closing in [false, true] {
            let closed = Arc::new(AtomicUsize::new(0));
            let subscriber = subscriber::registry().with(Closures(closed.clone()));
            tracing::subscriber::with_default(subscriber, || {
                use anyhow::Context as _;

                let message_span = tracing::debug_span!("in_flight_packet");
                let message_id = message_span.id().context("fixture needs an active span")?;
                let mut sink = ContextSink::new(ObservedSink {
                    pending: true,
                    ..Default::default()
                });
                let mut cx = Context::from_waker(futures::task::noop_waker_ref());
                let operation = if closing { "close" } else { "flush" };
                let poll_io = |sink: &mut ContextSink<ObservedSink>, cx: &mut Context<'_>| {
                    if closing {
                        Pin::new(sink).poll_close(cx)
                    } else {
                        Pin::new(sink).poll_flush(cx)
                    }
                };
                assert_eq!(Pin::new(&mut sink).poll_ready(&mut cx), Poll::Ready(Ok(())));
                Pin::new(&mut sink).start_send(WithContext::new(42, message_span))?;
                assert_eq!(closed.load(Ordering::Relaxed), 0);
                assert_eq!(poll_io(&mut sink, &mut cx), Poll::Pending);
                assert_eq!(closed.load(Ordering::Relaxed), 0, "pending I/O must retain context");
                assert_eq!(poll_io(&mut sink, &mut cx), Poll::Ready(Ok(())));
                assert_eq!(
                    closed.load(Ordering::Relaxed),
                    1,
                    "completed {operation} must close the final span without dropping the sink"
                );
                let mut expected = vec![
                    ("ready", None),
                    ("send", Some(message_id.clone())),
                    (operation, Some(message_id.clone())),
                    (operation, Some(message_id)),
                ];
                if !closing {
                    // Once flushed, empty I/O must not reuse the completed message's context.
                    assert_eq!(Pin::new(&mut sink).poll_flush(&mut cx), Poll::Ready(Ok(())));
                    assert_eq!(Pin::new(&mut sink).poll_close(&mut cx), Poll::Ready(Ok(())));
                    expected.extend([("flush", None), ("close", None)]);
                }
                assert_eq!(sink.inner.received, [42]);
                assert_eq!(sink.inner.events, expected);
                anyhow::Ok(())
            })?;
        }
        Ok(())
    }

    #[test]
    fn disabled_context_should_keep_the_envelope_compact() {
        assert_eq!(
            std::mem::size_of::<WithContext<u64>>(),
            std::mem::size_of::<(u64, usize)>(),
            "disabled spans must not occupy an inline Span in every queue slot"
        );
        let message = WithContext::new(42, tracing::Span::none());
        assert!(message.span.is_none(), "disabled context must not allocate");
        let (value, span) = message.into_parts();
        assert_eq!(value, 42);
        assert!(span.is_disabled());
    }

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
    async fn sink_should_capture_the_producer_span_and_close_after_delivery() -> anyhow::Result<()> {
        use futures::SinkExt;
        use tracing::{Instrument, instrument::WithSubscriber};

        async {
            let (tx, rx) = bounded_sink_channel(1);
            let mut tx = ContextSender::new(tx);
            let producer = tracing::debug_span!("producer");
            assert!(!producer.is_disabled(), "fixture needs an active producer span");
            // Explicit pipeline context is preserved even when automatic capture is disabled.
            let explicit = WithContext::new(7u8, producer.clone());
            assert!(explicit.span.is_some());
            let (value, restored) = explicit.into_parts();
            assert_eq!(value, 7);
            assert_eq!(restored.id(), producer.id());
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
            assert_eq!(message.span().id(), expected);
            tx.close().await?;
            drop(tx);
            assert!(
                rx.next().await.is_none(),
                "closing must preserve delivery and then terminate"
            );
            anyhow::Ok(())
        }
        .with_subscriber(test_log::tracing_subscriber::registry())
        .await
    }
}
