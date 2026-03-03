use std::{
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

use futures::{FutureExt, SinkExt, StreamExt, TryStreamExt, io::{AsyncRead, AsyncWrite}};
use hopr_async_runtime::AbortHandle;
use hopr_network_types::prelude::DestinationRouting;
use hopr_protocol_app::prelude::{ApplicationData, ApplicationDataOut};
use hopr_protocol_start::{KeepAliveFlag, KeepAliveMessage};
use tracing::{Instrument, debug, error, instrument};

use crate::{
    AtomicSurbFlowEstimator, SessionId,
    balancer::{BalancerStateValues, RateController, RateLimitStreamExt, SurbFlowEstimator},
    errors::TransportSessionError,
    types::HoprStartProtocol,
};

// ---- Runtime-agnostic bidirectional copy implementation ----

#[derive(Debug)]
enum TransferState {
    Running(CopyBuffer),
    ShuttingDown(u64),
    Done(u64),
}

fn transfer_one_direction<A, B>(
    cx: &mut Context<'_>,
    state: &mut TransferState,
    r: &mut A,
    w: &mut B,
) -> Poll<std::io::Result<u64>>
where
    A: AsyncRead + AsyncWrite + Unpin + ?Sized,
    B: AsyncRead + AsyncWrite + Unpin + ?Sized,
{
    let mut r = Pin::new(r);
    let mut w = Pin::new(w);
    loop {
        match state {
            TransferState::Running(buf) => {
                let count = std::task::ready!(buf.poll_copy(cx, r.as_mut(), w.as_mut()))?;
                tracing::trace!(processed = count, "direction copy complete");
                *state = TransferState::ShuttingDown(count);
            }
            TransferState::ShuttingDown(count) => {
                std::task::ready!(w.as_mut().poll_close(cx))?;
                tracing::trace!(processed = *count, "direction shutdown complete");
                *state = TransferState::Done(*count);
            }
            TransferState::Done(count) => return Poll::Ready(Ok(*count)),
        }
    }
}

/// Runtime-agnostic bidirectional copy between two [`AsyncRead`] + [`AsyncWrite`] objects.
///
/// This is a re-implementation of the tokio bidirectional copy that works with
/// [`futures::io::AsyncRead`] and [`futures::io::AsyncWrite`] traits, allowing it to operate
/// with any async runtime.
///
/// Variant with an option to abort either side early using the given
/// [`AbortRegistrations`](futures::future::AbortRegistration).
///
/// Once a side is aborted, its proper shutdown is initiated, and once done, the other side's
/// shutdown is also initiated.
/// The difference between the two abort handles is only in the order - which side gets shutdown
/// first after the abort is called.
async fn copy_duplex_abortable<A, B>(
    a: &mut A,
    b: &mut B,
    (a_to_b_buffer_size, b_to_a_buffer_size): (usize, usize),
    (a_abort, b_abort): (futures::future::AbortRegistration, futures::future::AbortRegistration),
) -> std::io::Result<(u64, u64)>
where
    A: AsyncRead + AsyncWrite + Unpin + ?Sized,
    B: AsyncRead + AsyncWrite + Unpin + ?Sized,
{
    let mut a_to_b = TransferState::Running(CopyBuffer::new(a_to_b_buffer_size));
    let mut b_to_a = TransferState::Running(CopyBuffer::new(b_to_a_buffer_size));

    // Abort futures are fused: once aborted, each poll returns Err(Aborted)
    let (mut abort_a, mut abort_b) = (
        futures::future::Abortable::new(futures::future::pending::<()>(), a_abort),
        futures::future::Abortable::new(futures::future::pending::<()>(), b_abort),
    );

    std::future::poll_fn(|cx| {
        let mut a_to_b_result = transfer_one_direction(cx, &mut a_to_b, a, b)?;
        let mut b_to_a_result = transfer_one_direction(cx, &mut b_to_a, b, a)?;

        // Initiate A's shutdown if A is aborted while still running
        if let (Poll::Ready(Err(_)), TransferState::Running(buf)) = (abort_a.poll_unpin(cx), &a_to_b) {
            tracing::trace!("A-side has been aborted.");
            a_to_b = TransferState::ShuttingDown(buf.amt);
            // We need an artificial wake-up here, as if an empty read was received
            cx.waker().wake_by_ref();
        }

        // Initiate B's shutdown if B is aborted while still running
        if let (Poll::Ready(Err(_)), TransferState::Running(buf)) = (abort_b.poll_unpin(cx), &b_to_a) {
            tracing::trace!("B-side has been aborted.");
            b_to_a = TransferState::ShuttingDown(buf.amt);
            // We need an artificial wake-up here, as if an empty read was received
            cx.waker().wake_by_ref();
        }

        // Once B-side is done, initiate shutdown of A-side
        if let TransferState::Done(_) = b_to_a
            && let TransferState::Running(buf) = &a_to_b
        {
            tracing::trace!("B-side has completed, terminating A-side.");
            a_to_b = TransferState::ShuttingDown(buf.amt);
            a_to_b_result = transfer_one_direction(cx, &mut a_to_b, a, b)?;
        }

        // Once A-side is done, initiate shutdown of B-side
        if let TransferState::Done(_) = a_to_b
            && let TransferState::Running(buf) = &b_to_a
        {
            tracing::trace!("A-side has completed, terminate B-side.");
            b_to_a = TransferState::ShuttingDown(buf.amt);
            b_to_a_result = transfer_one_direction(cx, &mut b_to_a, b, a)?;
        }

        // Not a problem if ready! returns early
        let a_to_b_bytes_transferred = std::task::ready!(a_to_b_result);
        let b_to_a_bytes_transferred = std::task::ready!(b_to_a_result);

        tracing::trace!(
            a_to_b = a_to_b_bytes_transferred,
            b_to_a = b_to_a_bytes_transferred,
            "copy completed"
        );
        Poll::Ready(Ok((a_to_b_bytes_transferred, b_to_a_bytes_transferred)))
    })
    .await
}

#[derive(Debug)]
struct CopyBuffer {
    read_done: bool,
    need_flush: bool,
    pos: usize,
    cap: usize,
    amt: u64,
    buf: Box<[u8]>,
}

impl CopyBuffer {
    fn new(buf_size: usize) -> Self {
        Self {
            read_done: false,
            need_flush: false,
            pos: 0,
            cap: 0,
            amt: 0,
            buf: vec![0; buf_size].into_boxed_slice(),
        }
    }

    fn poll_fill_buf<R>(&mut self, cx: &mut Context<'_>, reader: Pin<&mut R>) -> Poll<std::io::Result<()>>
    where
        R: AsyncRead + ?Sized,
    {
        let me = &mut *self;
        match reader.poll_read(cx, &mut me.buf[me.cap..]) {
            Poll::Ready(Ok(0)) => {
                me.read_done = true;
                Poll::Ready(Ok(()))
            }
            Poll::Ready(Ok(n)) => {
                me.cap += n;
                Poll::Ready(Ok(()))
            }
            Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
            Poll::Pending => Poll::Pending,
        }
    }

    fn poll_write_buf<R, W>(
        &mut self,
        cx: &mut Context<'_>,
        mut reader: Pin<&mut R>,
        mut writer: Pin<&mut W>,
    ) -> Poll<std::io::Result<usize>>
    where
        R: AsyncRead + ?Sized,
        W: AsyncWrite + ?Sized,
    {
        let this = &mut *self;
        match writer.as_mut().poll_write(cx, &this.buf[this.pos..this.cap]) {
            Poll::Pending => {
                // Top up the buffer towards full if we can read a bit more
                // data - this should improve the chances of a large write
                if !this.read_done && this.cap < this.buf.len() {
                    std::task::ready!(this.poll_fill_buf(cx, reader.as_mut()))?;
                }
                Poll::Pending
            }
            res @ Poll::Ready(_) => res,
        }
    }

    fn poll_copy<R, W>(
        &mut self,
        cx: &mut Context<'_>,
        mut reader: Pin<&mut R>,
        mut writer: Pin<&mut W>,
    ) -> Poll<std::io::Result<u64>>
    where
        R: AsyncRead + ?Sized,
        W: AsyncWrite + ?Sized,
    {
        loop {
            // If our buffer is empty, then we need to read some data to continue.
            if self.pos == self.cap && !self.read_done {
                self.pos = 0;
                self.cap = 0;

                match self.poll_fill_buf(cx, reader.as_mut()) {
                    Poll::Ready(Ok(())) => (),
                    Poll::Ready(Err(err)) => {
                        return Poll::Ready(Err(std::io::Error::new(std::io::ErrorKind::BrokenPipe, err)));
                    }
                    Poll::Pending => {
                        // Try flushing when the reader has no progress to avoid deadlock
                        // when the reader depends on a buffered writer.
                        if self.need_flush {
                            std::task::ready!(writer.as_mut().poll_flush(cx))?;
                            self.need_flush = false;
                        }

                        return Poll::Pending;
                    }
                }
            }

            // If our buffer has some data, let's write it out
            while self.pos < self.cap {
                let i = std::task::ready!(self.poll_write_buf(cx, reader.as_mut(), writer.as_mut()))?;
                if i == 0 {
                    return Poll::Ready(Err(std::io::Error::new(
                        std::io::ErrorKind::WriteZero,
                        "write zero byte",
                    )));
                }
                self.pos += i;
                self.amt += i as u64;
                self.need_flush = true;
            }

            // If pos larger than cap, this loop will never stop.
            debug_assert!(self.pos <= self.cap, "writer returned length larger than input slice");

            // If we've written all the data, and we've seen EOF, flush out the
            // data and finish the transfer.
            if self.pos == self.cap && self.read_done {
                std::task::ready!(writer.as_mut().poll_flush(cx))?;
                return Poll::Ready(Ok(self.amt));
            }
        }
    }
}

// ---- End of bidirectional copy implementation ----

/// Convenience function to copy data in both directions between a [`Session`](crate::HoprSession) and arbitrary
/// async IO stream.
///
/// The function is runtime agnostic and works with any async executor.
///
/// The `abort_stream` will terminate the transfer from the `stream` side, i.e.:
/// 1. Initiates graceful shutdown of `stream`
/// 2. Once done, initiates a graceful shutdown of `session`
/// 3. The function terminates, returning the number of bytes transferred in both directions.
pub async fn transfer_session<S>(
    session: &mut crate::HoprSession,
    stream: &mut S,
    max_buffer: usize,
    abort_stream: Option<futures::future::AbortRegistration>,
) -> std::io::Result<(usize, usize)>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    // We can use equally sized buffer for both directions
    tracing::debug!(
        session_id = ?session.id(),
        egress_buffer = max_buffer,
        ingress_buffer = max_buffer,
        "session buffers"
    );

    // We only allow aborting from the "stream" side, not from the "session side".
    // This is useful for UDP-like streams on the "stream" side, which cannot be terminated
    // by a signal from outside (e.g.: for TCP sockets such signal is socket closure).
    let (_, session_dummy) = futures::future::AbortHandle::new_pair();
    let (_, stream_dummy) = futures::future::AbortHandle::new_pair();
    let stream_abort = abort_stream.unwrap_or(stream_dummy);

    copy_duplex_abortable(
        session,
        stream,
        (max_buffer, max_buffer),
        (session_dummy, stream_abort),
    )
    .await
    .map(|(a, b)| (a as usize, b as usize))
}

/// This function will use the given generator to generate an initial seeding key.
/// It will check whether the given cache already contains a value for that key, and if not,
/// calls the generator (with the previous value) to generate a new seeding key and retry.
/// The function either finds a suitable free slot, inserting value generated by `value_fn` and returns the found key,
/// or terminates with `None` when `gen` returns the initial seed again.
pub(crate) async fn insert_into_next_slot<F, K, U, V>(
    cache: &moka::future::Cache<K, V>,
    mut generator: F,
    value_fn: U,
) -> Option<(K, V)>
where
    F: FnMut(Option<K>) -> K,
    K: Copy + std::hash::Hash + Eq + Send + Sync + 'static,
    U: FnOnce(K) -> V,
    V: Clone + Send + Sync + 'static,
{
    cache.run_pending_tasks().await;

    // Wrap the FnOnce so we can "consume" it exactly once,
    // but only when we actually insert into a free slot.
    let value_fn = std::sync::Arc::new(parking_lot::Mutex::new(Some(value_fn)));

    let initial = generator(None);
    let mut next = initial;
    loop {
        let value_fn = value_fn.clone();
        let insertion_result = cache
            .entry(next)
            .and_try_compute_with(move |e| {
                if e.is_none() {
                    let f = value_fn
                        .lock()
                        .take()
                        .expect("impossible: value_fn was already consumed");

                    futures::future::ok::<_, ()>(moka::ops::compute::Op::Put(f(next)))
                } else {
                    futures::future::ok::<_, ()>(moka::ops::compute::Op::Nop)
                }
            })
            .await;

        // If we inserted successfully, break the loop and return the insertion key
        if let Ok(moka::ops::compute::CompResult::Inserted(val)) = insertion_result {
            return Some((next, val.into_value()));
        }

        // Otherwise, generate the next key
        next = generator(Some(next));

        // If generated keys made it to full loop, return failure
        if next == initial {
            return None;
        }
    }
}

/// Indicates whether the [keep-alive stream](spawn_keep_alive_stream) should notify the Session counterparty
/// about the SURB target (Entry) or SURB level (Exit).
#[derive(Debug, Clone)]
pub(crate) enum SurbNotificationMode {
    /// No keep-alive messages are sent to the Session counterparty.
    DoNotNotify,
    /// Session initiator notifies the Session recipient about the desired SURB target level.
    Target,
    /// Session recipient notifies the Session initiator about the current SURB level.
    Level(AtomicSurbFlowEstimator),
}

/// Spawns a task for a rate-limited stream of Keep-Alive messages to the Session counterparty.
#[instrument(level = "debug", skip(sender, routing, notification_mode, cfg))]
pub(crate) fn spawn_keep_alive_stream<S>(
    session_id: SessionId,
    sender: S,
    routing: DestinationRouting,
    notification_mode: SurbNotificationMode,
    cfg: std::sync::Arc<BalancerStateValues>,
) -> (RateController, AbortHandle)
where
    S: futures::Sink<(DestinationRouting, ApplicationDataOut)> + Clone + Send + Sync + Unpin + 'static,
    S::Error: std::error::Error + Send + Sync + 'static,
{
    // The stream is suspended until the caller sets a rate via the Controller
    let controller = RateController::new(0, Duration::from_secs(1));

    let (ka_stream, abort_handle) = futures::stream::abortable(
        futures::stream::repeat_with(move || match &notification_mode {
            SurbNotificationMode::Target => HoprStartProtocol::KeepAlive(KeepAliveMessage {
                session_id,
                flags: KeepAliveFlag::BalancerTarget.into(),
                additional_data: cfg.target_surb_buffer_size.load(std::sync::atomic::Ordering::Relaxed),
            }),
            SurbNotificationMode::Level(estimator) => HoprStartProtocol::KeepAlive(KeepAliveMessage {
                session_id,
                flags: KeepAliveFlag::BalancerState.into(),
                additional_data: estimator.saturating_diff(),
            }),
            SurbNotificationMode::DoNotNotify => HoprStartProtocol::KeepAlive(KeepAliveMessage {
                session_id,
                flags: None.into(),
                additional_data: 0,
            }),
        })
        .rate_limit_with_controller(&controller),
    );

    let sender_clone = sender.clone();
    let fwd_routing_clone = routing.clone();

    // This task will automatically terminate once the returned abort handle is used.
    debug!(%session_id, "spawning keep-alive stream");
    hopr_async_runtime::prelude::spawn(
        ka_stream
            .map(move |msg| {
                ApplicationData::try_from(msg)
                    .map(|data| (fwd_routing_clone.clone(), ApplicationDataOut::with_no_packet_info(data)))
            })
            .map_err(TransportSessionError::from)
            .try_for_each_concurrent(None, move |msg| {
                let mut sender_clone = sender_clone.clone();
                async move {
                    sender_clone
                        .send(msg)
                        .await
                        .map_err(TransportSessionError::packet_sending)
                }
            })
            .then(move |res| {
                match res {
                    Ok(_) => tracing::debug!(
                        component = "session",
                        %session_id,
                        task = "session keepalive",
                        "background task finished"
                    ),
                    Err(error) => error!(%session_id, %error, "keep-alive stream failed"),
                }
                futures::future::ready(())
            })
            .in_current_span(),
    );

    (controller, abort_handle)
}

#[cfg(test)]
mod tests {
    use std::{
        io::Read,
        pin::Pin,
        sync::Arc,
        task::{Context, Poll},
    };

    use anyhow::{Context as _, anyhow};
    use futures::StreamExt;
    use hopr_crypto_random::Randomizable;
    use hopr_crypto_types::prelude::{ChainKeypair, Keypair};
    use hopr_internal_types::prelude::HoprPseudonym;
    use hopr_network_types::prelude::{DestinationRouting, RoutingOptions};
    use hopr_primitive_types::prelude::Address;
    use hopr_protocol_app::prelude::{ApplicationData, ApplicationDataIn, ApplicationDataOut};

    use super::*;
    use crate::{HoprSession, HoprSessionConfig, SessionId};

    /// A simple mock bidirectional stream for testing:
    /// - Reads from a pre-filled byte buffer (returns EOF once exhausted)
    /// - Captures all bytes written to a shared buffer
    struct MockStream {
        read_data: std::io::Cursor<Vec<u8>>,
        write_data: Arc<parking_lot::Mutex<Vec<u8>>>,
    }

    impl MockStream {
        fn new(read_data: &[u8]) -> (Self, Arc<parking_lot::Mutex<Vec<u8>>>) {
            let write_data = Arc::new(parking_lot::Mutex::new(Vec::new()));
            (
                Self {
                    read_data: std::io::Cursor::new(read_data.to_vec()),
                    write_data: write_data.clone(),
                },
                write_data,
            )
        }
    }

    impl futures::AsyncRead for MockStream {
        fn poll_read(
            mut self: Pin<&mut Self>,
            _cx: &mut Context<'_>,
            buf: &mut [u8],
        ) -> Poll<std::io::Result<usize>> {
            Poll::Ready(self.read_data.read(buf))
        }
    }

    impl futures::AsyncWrite for MockStream {
        fn poll_write(
            self: Pin<&mut Self>,
            _cx: &mut Context<'_>,
            buf: &[u8],
        ) -> Poll<std::io::Result<usize>> {
            self.get_mut().write_data.lock().extend_from_slice(buf);
            Poll::Ready(Ok(buf.len()))
        }

        fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
            Poll::Ready(Ok(()))
        }

        fn poll_close(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
            Poll::Ready(Ok(()))
        }
    }

    type SessionChannels = (
        HoprSession,
        futures::channel::mpsc::UnboundedSender<ApplicationDataIn>,
        futures::channel::mpsc::UnboundedReceiver<(hopr_network_types::prelude::DestinationRouting, ApplicationDataOut)>,
    );

    fn make_session(id: SessionId, dst: Address) -> anyhow::Result<SessionChannels> {
        let (out_tx, out_rx) =
            futures::channel::mpsc::unbounded::<(DestinationRouting, ApplicationDataOut)>();
        let (in_tx, in_rx) = futures::channel::mpsc::unbounded::<ApplicationDataIn>();

        let cfg = HoprSessionConfig::default();

        #[cfg(feature = "telemetry")]
        let metrics = Arc::new(crate::telemetry::SessionTelemetry::new(id, cfg));

        let session = HoprSession::new(
            id,
            DestinationRouting::forward_only(dst, RoutingOptions::Hops(0.try_into()?)),
            cfg,
            (out_tx, in_rx),
            None,
            #[cfg(feature = "telemetry")]
            metrics,
        )?;

        Ok((session, in_tx, out_rx))
    }

    #[tokio::test]
    async fn transfer_session_copies_data_bidirectionally() -> anyhow::Result<()> {
        let id = SessionId::new(42u64, HoprPseudonym::random());
        let dst: Address = (&ChainKeypair::random()).into();

        const SESSION_DATA: &[u8] = b"data_from_session";
        const STREAM_DATA: &[u8] = b"data_from_stream_";

        let (mut session, in_tx, mut out_rx) = make_session(id, dst)?;

        // Feed SESSION_DATA into the session's incoming transport channel.
        // transfer_session will read it from the session side and write it to the stream.
        in_tx.unbounded_send(ApplicationDataIn {
            data: ApplicationData::new(id.tag(), SESSION_DATA).map_err(|e| anyhow!("{e}"))?,
            packet_info: Default::default(),
        })?;
        // Closing the sender causes the session to return EOF once SESSION_DATA is exhausted.
        drop(in_tx);

        // Create a mock stream: provides STREAM_DATA for reading, captures what is written.
        // transfer_session will read STREAM_DATA from the stream and write it to the session.
        let (mut mock_stream, captured_writes) = MockStream::new(STREAM_DATA);

        // Run transfer_session. Both sides are finite, so it terminates naturally.
        transfer_session(&mut session, &mut mock_stream, 256, None).await?;

        // SESSION_DATA must have been read from the session and written to the mock stream.
        assert_eq!(
            captured_writes.lock().as_slice(),
            SESSION_DATA,
            "session data should have been written to stream"
        );

        // STREAM_DATA must have been read from the mock stream and sent through the session
        // transport (alice_out_tx).
        let received = out_rx
            .next()
            .await
            .context("should have received STREAM_DATA via session transport")?;
        assert_eq!(
            received.1.data.plain_text.as_ref(),
            STREAM_DATA,
            "stream data should have been sent through session transport"
        );

        Ok(())
    }

    #[tokio::test]
    async fn transfer_session_with_abort_terminates_early() -> anyhow::Result<()> {
        let id = SessionId::new(99u64, HoprPseudonym::random());
        let dst: Address = (&ChainKeypair::random()).into();

        let (mut session, _in_tx, _out_rx) = make_session(id, dst)?;

        // Create an infinite mock stream (never returns EOF on read side - always returns Pending)
        struct InfiniteStream;

        impl futures::AsyncRead for InfiniteStream {
            fn poll_read(
                self: Pin<&mut Self>,
                _cx: &mut Context<'_>,
                _buf: &mut [u8],
            ) -> Poll<std::io::Result<usize>> {
                Poll::Pending
            }
        }

        impl futures::AsyncWrite for InfiniteStream {
            fn poll_write(
                self: Pin<&mut Self>,
                _cx: &mut Context<'_>,
                buf: &[u8],
            ) -> Poll<std::io::Result<usize>> {
                Poll::Ready(Ok(buf.len()))
            }

            fn poll_flush(
                self: Pin<&mut Self>,
                _cx: &mut Context<'_>,
            ) -> Poll<std::io::Result<()>> {
                Poll::Ready(Ok(()))
            }

            fn poll_close(
                self: Pin<&mut Self>,
                _cx: &mut Context<'_>,
            ) -> Poll<std::io::Result<()>> {
                Poll::Ready(Ok(()))
            }
        }

        let (abort_handle, abort_reg) = futures::future::AbortHandle::new_pair();
        let mut stream = InfiniteStream;

        let transfer = tokio::task::spawn(async move {
            transfer_session(&mut session, &mut stream, 256, Some(abort_reg)).await
        });

        // Give transfer_session time to start, then abort from the stream side.
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        abort_handle.abort();

        // transfer_session must terminate after the abort.
        let result = tokio::time::timeout(std::time::Duration::from_millis(200), transfer)
            .await
            .context("transfer_session did not terminate after abort")??;

        // The function should complete without error once aborted.
        assert!(result.is_ok(), "expected Ok result after clean abort");

        Ok(())
    }

    #[tokio::test]
    async fn test_insert_into_next_slot() -> anyhow::Result<()> {
        let cache = moka::future::Cache::new(10);

        for i in 0..5 {
            let (k, v) = insert_into_next_slot(
                &cache,
                |prev| prev.map(|v| (v + 1) % 5).unwrap_or(0),
                |k| format!("foo_{k}"),
            )
            .await
            .ok_or(anyhow!("should insert"))?;
            assert_eq!(k, i);
            assert_eq!(format!("foo_{i}"), v);
            assert_eq!(Some(v), cache.get(&i).await);
        }

        assert!(
            insert_into_next_slot(
                &cache,
                |prev| prev.map(|v| (v + 1) % 5).unwrap_or(0),
                |_| "foo".to_string()
            )
            .await
            .is_none(),
            "must not find slot when full"
        );

        Ok(())
    }
}
