use std::{
    marker::PhantomData,
    pin::Pin,
    sync::{Arc, atomic::AtomicU64},
    task::{Context, Poll},
};

/// Convenience function to copy data in both directions between a [`Session`] and arbitrary
/// async IO stream.
/// This function is only available with Tokio and will panic with other runtimes.
///
/// The `abort_stream` will terminate the transfer from the `stream` side, i.e.:
/// 1. Initiates graceful shutdown of `stream`
/// 2. Once done, initiates a graceful shutdown of `session`
/// 3. The function terminates, returning the number of bytes transfered in both directions.
#[cfg(feature = "runtime-tokio")]
pub async fn transfer_session<S>(
    session: &mut crate::Session,
    stream: &mut S,
    max_buffer: usize,
    abort_stream: Option<futures::future::AbortRegistration>,
) -> std::io::Result<(usize, usize)>
where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
{
    // We can use equally sized buffer for both directions
    tracing::debug!(
        session_id = ?session.id(),
        egress_buffer = max_buffer,
        ingress_buffer = max_buffer,
        "session buffers"
    );

    if let Some(abort_stream) = abort_stream {
        // We only allow aborting from the "stream" side, not from the "session side"
        // This is useful for UDP-like streams on the "stream" side, which cannot be terminated
        // by a signal from outside (e.g.: for TCP sockets such signal is socket closure).
        let (_, dummy) = futures::future::AbortHandle::new_pair();
        hopr_network_types::utils::copy_duplex_abortable(
            session,
            stream,
            (max_buffer, max_buffer),
            (dummy, abort_stream),
        )
        .await
        .map(|(a, b)| (a as usize, b as usize))
    } else {
        hopr_network_types::utils::copy_duplex(session, stream, (max_buffer, max_buffer))
            .await
            .map(|(a, b)| (a as usize, b as usize))
    }
}

/// Decorates a `Sink` with a scoring function that is called for each successfully sent item,
/// accumulating the score into an `AtomicU64`.
///
/// This adapter is also `Clone` if the underlying `Sink` is.
#[pin_project::pin_project]
pub(crate) struct ScoringSink<I, S, F> {
    #[pin]
    inner: S,
    score: Arc<AtomicU64>,
    scoring_fn: F,
    _d: PhantomData<I>,
}

impl<I, S, F> ScoringSink<I, S, F>
where
    S: futures::Sink<I>,
    F: Fn(&I) -> u64,
{
    pub fn new(inner: S, score: Arc<AtomicU64>, scoring_fn: F) -> Self {
        Self {
            inner,
            score,
            scoring_fn,
            _d: PhantomData,
        }
    }
}

impl<I, S, F> futures::Sink<I> for ScoringSink<I, S, F>
where
    S: futures::Sink<I>,
    F: Fn(&I) -> u64,
{
    type Error = S::Error;

    fn poll_ready(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().inner.poll_ready(cx)
    }

    fn start_send(self: Pin<&mut Self>, item: I) -> Result<(), Self::Error> {
        let this = self.project();
        let score = (this.scoring_fn)(&item);
        this.inner.start_send(item).inspect(|_| {
            this.score.fetch_add(score, std::sync::atomic::Ordering::Relaxed);
        })
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().inner.poll_flush(cx)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.project().inner.poll_close(cx)
    }
}

impl<I, S: Clone, F: Clone> Clone for ScoringSink<I, S, F> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            score: self.score.clone(),
            scoring_fn: self.scoring_fn.clone(),
            _d: PhantomData,
        }
    }
}
