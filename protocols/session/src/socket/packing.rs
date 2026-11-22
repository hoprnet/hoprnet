//! Whole-message datagram packer for stateless [`SessionSocket`](super::SessionSocket) writes.
//!
//! Accumulates complete encoded [`SessionMessage`]s into a single transport write of at most `C`
//! bytes (one HOPR packet payload) without ever splitting a message across transport writes.
//! Because every transport write begins and ends on a [`SessionMessage`] boundary, an unmodified
//! receiver using [`SessionCodec`] decodes all packed messages in order and isolates packet loss
//! strictly to the messages contained in the dropped packet.

use std::{
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
    time::{Duration, Instant},
};

use asynchronous_codec::Encoder;
use bytes::{Bytes, BytesMut};
use futures::{
    AsyncWrite, AsyncWriteExt, Sink, StreamExt,
    channel::{mpsc, oneshot},
    future::FutureExt,
};
use parking_lot::Mutex;
use tracing::Instrument;

use crate::{
    errors::SessionError,
    protocol::{SessionCodec, SessionMessage},
};

/// Channel capacity between the `SessionSocket` writer and the background packer task.
///
/// Kept small (~1-2 packed packets' worth of datagrams) so downstream backpressure (e.g., exit
/// SURB rate limiting) propagates promptly to the session writer instead of accumulating seconds
/// of stale datagrams in memory.
const PACKER_CHANNEL_CAPACITY: usize = 32;

/// Maximum grace period given to the background packer task to flush already-queued datagrams
/// when the socket is dropped without calling `poll_close`.
///
/// Healthy downstream writers drain within microseconds (preventing silent loss on drop), while a
/// stalled writer (e.g., exit rate 0 or blocked PIX gate) is aborted after this deadline so the
/// task and transport can never leak.
const DROP_FLUSH_GRACE: Duration = Duration::from_millis(50);

/// Optional dynamic byte cap `<= C` for each packed transport write.
///
/// Used on entry-side (forward) sessions to leave room for piggybacked SURBs whenever the
/// counterparty's SURB buffer is below target, while allowing full `C`-byte packing once full
/// or on return-routed (exit) sessions that never carry SURBs.
pub type PackPayloadBudget = Arc<dyn Fn() -> usize + Send + Sync>;

struct StoredError {
    kind: std::io::ErrorKind,
    message: String,
}

/// A [`Sink`] that packs complete encoded [`SessionMessage`]s into transport writes of at most `C`
/// bytes, flushing whenever:
/// - the next message would exceed the effective byte budget `<= C`,
/// - an isolated message arrives after an idle gap (no prior message within the burst window),
/// - the remaining space cannot fit another message of the same size (or `minimum_message_size`),
/// - `pack_delay` elapses, or
/// - the sink is closed.
pub(crate) struct WholeMessagePackingSink<const C: usize> {
    tx: Option<mpsc::Sender<Bytes>>,
    done: Option<oneshot::Receiver<Result<(), SessionError>>>,
    /// Dropped when the sink is dropped without completing `poll_close`, arming the in-task
    /// `DROP_FLUSH_GRACE` timeout without requiring `tokio::spawn` inside `Drop`.
    drop_guard: Option<oneshot::Sender<()>>,
    error: Arc<Mutex<Option<StoredError>>>,
}

impl<const C: usize> Drop for WholeMessagePackingSink<C> {
    fn drop(&mut self) {
        // Close the sender so the background task sees EOF and flushes queued messages into a
        // healthy writer, then drop `drop_guard` so the in-task watchdog cancels a stalled writer
        // after `DROP_FLUSH_GRACE` without calling `spawn` in `Drop`.
        self.tx.take();
        self.drop_guard.take();
    }
}

impl<const C: usize> WholeMessagePackingSink<C> {
    pub(crate) fn new<W>(
        writer: W,
        pack_delay: Duration,
        budget: Option<PackPayloadBudget>,
        span: tracing::Span,
    ) -> Self
    where
        W: AsyncWrite + Send + Unpin + 'static,
    {
        let (tx, rx) = mpsc::channel(PACKER_CHANNEL_CAPACITY);
        let (done_tx, done_rx) = oneshot::channel();
        let (drop_tx, drop_rx) = oneshot::channel();
        let error = Arc::new(Mutex::new(None));
        let error_for_task = error.clone();

        hopr_utils::runtime::prelude::spawn(
            async move {
                let packer = run_packer::<C, W>(rx, writer, pack_delay, budget, error_for_task, done_tx).fuse();
                let grace = async move {
                    // Only arm the grace timer when `drop_guard` is dropped (not explicitly
                    // notified on `poll_close`).
                    if drop_rx.await.is_err() {
                        futures_time::task::sleep(DROP_FLUSH_GRACE.into()).await;
                    } else {
                        futures::future::pending::<()>().await;
                    }
                }
                .fuse();
                futures::pin_mut!(packer, grace);
                futures::select_biased! {
                    _ = packer => {}
                    _ = grace => {}
                }
            }
            .instrument(span),
        );

        Self {
            tx: Some(tx),
            done: Some(done_rx),
            drop_guard: Some(drop_tx),
            error,
        }
    }

    fn take_error(&self) -> Option<SessionError> {
        self.error
            .lock()
            .take()
            .map(|stored| SessionError::IoError(std::io::Error::new(stored.kind, stored.message)))
    }
}

async fn flush_packet<W: AsyncWrite + Unpin>(writer: &mut W, buf: &mut BytesMut) -> Result<(), SessionError> {
    if buf.is_empty() {
        return Ok(());
    }
    let pkt = buf.split();
    writer.write_all(&pkt).await.map_err(SessionError::IoError)?;
    writer.flush().await.map_err(SessionError::IoError)
}

async fn run_packer<const C: usize, W>(
    mut rx: mpsc::Receiver<Bytes>,
    mut writer: W,
    pack_delay: Duration,
    budget: Option<PackPayloadBudget>,
    error_slot: Arc<Mutex<Option<StoredError>>>,
    done_tx: oneshot::Sender<Result<(), SessionError>>,
) where
    W: AsyncWrite + Unpin,
{
    let min_msg = SessionMessage::<C>::minimum_message_size();
    let burst_window = pack_delay;
    let mut buf = BytesMut::with_capacity(C);
    let mut timer: Option<Pin<Box<futures_time::task::Sleep>>> = None;
    let mut last_msg_at: Option<Instant> = None;

    let result: Result<(), SessionError> = loop {
        let next_msg = if let Some(sleep) = timer.as_mut() {
            futures::select_biased! {
                msg = rx.next() => Some(msg),
                _ = sleep.as_mut().fuse() => None,
            }
        } else {
            Some(rx.next().await)
        };

        match next_msg {
            None => {
                timer = None;
                if let Err(err) = flush_packet(&mut writer, &mut buf).await {
                    break Err(err);
                }
            }
            Some(None) => {
                if let Err(err) = flush_packet(&mut writer, &mut buf).await {
                    break Err(err);
                }
                break writer.close().await.map_err(SessionError::IoError);
            }
            Some(Some(msg)) => {
                let now = Instant::now();
                let in_burst = last_msg_at.is_some_and(|t| now.saturating_duration_since(t) <= burst_window);
                last_msg_at = Some(now);

                let effective_limit = budget.as_ref().map_or(C, |f| f().min(C));
                if !buf.is_empty() && buf.len() + msg.len() > effective_limit {
                    timer = None;
                    if let Err(err) = flush_packet(&mut writer, &mut buf).await {
                        break Err(err);
                    }
                }
                buf.extend_from_slice(&msg);

                let isolated = !in_burst && buf.len() == msg.len();
                let remaining = effective_limit.saturating_sub(buf.len());
                let cannot_fit_another = remaining < msg.len().max(min_msg);

                if pack_delay.is_zero() || isolated || cannot_fit_another {
                    timer = None;
                    if let Err(err) = flush_packet(&mut writer, &mut buf).await {
                        break Err(err);
                    }
                } else if timer.is_none() {
                    timer = Some(Box::pin(futures_time::task::sleep(pack_delay.into())));
                }
            }
        }
    };

    if let Err(ref err) = result {
        let kind = match err {
            SessionError::IoError(io_err) => io_err.kind(),
            _ => std::io::ErrorKind::Other,
        };
        *error_slot.lock() = Some(StoredError {
            kind,
            message: err.to_string(),
        });
    }
    let _ = done_tx.send(result);
}

impl<const C: usize> Sink<SessionMessage<C>> for WholeMessagePackingSink<C> {
    type Error = SessionError;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        if let Some(err) = self.take_error() {
            return Poll::Ready(Err(err));
        }
        let Some(tx) = self.tx.as_mut() else {
            return Poll::Ready(Err(SessionError::IoError(std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                "packer sink closed",
            ))));
        };
        tx.poll_ready(cx)
            .map_err(|e| SessionError::IoError(std::io::Error::new(std::io::ErrorKind::BrokenPipe, e)))
    }

    fn start_send(mut self: Pin<&mut Self>, item: SessionMessage<C>) -> Result<(), Self::Error> {
        let mut encoded = BytesMut::new();
        SessionCodec::<C>.encode(item, &mut encoded)?;
        let tx = self.tx.as_mut().ok_or_else(|| {
            SessionError::IoError(std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                "packer sink closed",
            ))
        })?;
        tx.start_send(encoded.freeze())
            .map_err(|e| SessionError::IoError(std::io::Error::new(std::io::ErrorKind::BrokenPipe, e)))
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        if let Some(err) = self.take_error() {
            return Poll::Ready(Err(err));
        }
        // Messages are already queued in the packer channel and will be emitted within `pack_delay`
        // (or immediately once a packet fills). Flushing synchronously here would defeat packing
        // under `NoDelay` (`flush_immediately = true`).
        Poll::Ready(Ok(()))
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        // Drop the sender to signal EOF so the background task flushes any buffered messages and
        // closes the underlying writer.
        self.tx.take();
        if let Some(done) = self.done.as_mut() {
            match done.poll_unpin(cx) {
                Poll::Ready(Ok(res)) => {
                    self.done = None;
                    if let Some(guard) = self.drop_guard.take() {
                        let _ = guard.send(());
                    }
                    return Poll::Ready(res);
                }
                Poll::Ready(Err(_)) => {
                    self.done = None;
                    if let Some(guard) = self.drop_guard.take() {
                        let _ = guard.send(());
                    }
                    return Poll::Ready(Ok(()));
                }
                Poll::Pending => return Poll::Pending,
            }
        }
        Poll::Ready(Ok(()))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex as StdMutex};

    use futures::{AsyncReadExt, AsyncWriteExt};
    use hopr_protocol_app::prelude::ApplicationData;

    use super::*;
    #[cfg(feature = "telemetry")]
    use crate::socket::telemetry::NoopTracker;
    use crate::{
        MAX_SESSION_MTU,
        socket::{SessionSocket, SessionSocketConfig},
    };

    const WIRE_MTU: usize = ApplicationData::PAYLOAD_SIZE; // 3,238 B

    /// In-memory packet transport that records every individual `poll_write` slice (representing
    /// one HOPR packet payload) and delivers them atomically to a receiver.
    struct RecordingPacketPipe {
        tx: mpsc::UnboundedSender<Bytes>,
        rx: mpsc::UnboundedReceiver<Bytes>,
        read_buf: BytesMut,
        recorded_writes: Arc<StdMutex<Vec<Bytes>>>,
        drop_write_indices: Arc<StdMutex<Vec<usize>>>,
    }

    impl RecordingPacketPipe {
        fn pair(drop_indices: Vec<usize>) -> (Self, Self, Arc<StdMutex<Vec<Bytes>>>) {
            let (a_tx, b_rx) = mpsc::unbounded();
            let (b_tx, a_rx) = mpsc::unbounded();
            let recorded = Arc::new(StdMutex::new(Vec::new()));
            let drop_indices = Arc::new(StdMutex::new(drop_indices));
            let a = Self {
                tx: a_tx,
                rx: a_rx,
                read_buf: BytesMut::new(),
                recorded_writes: recorded.clone(),
                drop_write_indices: drop_indices,
            };
            let b = Self {
                tx: b_tx,
                rx: b_rx,
                read_buf: BytesMut::new(),
                recorded_writes: Arc::new(StdMutex::new(Vec::new())),
                drop_write_indices: Arc::new(StdMutex::new(Vec::new())),
            };
            (a, b, recorded)
        }
    }

    impl futures::AsyncRead for RecordingPacketPipe {
        fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut [u8]) -> Poll<std::io::Result<usize>> {
            if self.read_buf.is_empty() {
                match self.rx.poll_next_unpin(cx) {
                    Poll::Ready(Some(pkt)) => self.read_buf.extend_from_slice(&pkt),
                    Poll::Ready(None) => return Poll::Ready(Ok(0)),
                    Poll::Pending => return Poll::Pending,
                }
            }
            let n = buf.len().min(self.read_buf.len());
            buf[..n].copy_from_slice(&self.read_buf.split_to(n));
            Poll::Ready(Ok(n))
        }
    }

    impl AsyncWrite for RecordingPacketPipe {
        fn poll_write(self: Pin<&mut Self>, _cx: &mut Context<'_>, buf: &[u8]) -> Poll<std::io::Result<usize>> {
            assert!(
                buf.len() <= WIRE_MTU,
                "packed write ({} B) exceeded HOPR payload MTU ({WIRE_MTU} B)",
                buf.len()
            );
            let mut rec = self.recorded_writes.lock().unwrap();
            let idx = rec.len();
            let bytes = Bytes::copy_from_slice(buf);
            rec.push(bytes.clone());
            if !self.drop_write_indices.lock().unwrap().contains(&idx) {
                let _ = self.tx.unbounded_send(bytes);
            }
            Poll::Ready(Ok(buf.len()))
        }

        fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
            Poll::Ready(Ok(()))
        }

        fn poll_close(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
            self.tx.close_channel();
            Poll::Ready(Ok(()))
        }
    }

    #[tokio::test]
    async fn packs_two_full_mtu_datagrams_per_packet_and_unmodified_receiver_decodes_all() -> anyhow::Result<()> {
        let (sender_pipe, receiver_pipe, writes) = RecordingPacketPipe::pair(vec![]);

        let sender_cfg = SessionSocketConfig {
            frame_size: MAX_SESSION_MTU,
            flush_immediately: true,
            datagram: true,
            pack_delay: Some(Duration::from_millis(20)),
            ..Default::default()
        };
        // Unmodified receiver configuration (matches v5.0.4: pack_delay = None).
        let receiver_cfg = SessionSocketConfig {
            frame_size: MAX_SESSION_MTU,
            flush_immediately: true,
            datagram: true,
            pack_delay: None,
            ..Default::default()
        };

        let mut sender = SessionSocket::<WIRE_MTU, _>::new_stateless(
            "sender",
            sender_pipe,
            sender_cfg,
            None,
            #[cfg(feature = "telemetry")]
            NoopTracker,
        )?;
        let mut receiver = SessionSocket::<WIRE_MTU, _>::new_stateless(
            "receiver",
            receiver_pipe,
            receiver_cfg,
            None,
            #[cfg(feature = "telemetry")]
            NoopTracker,
        )?;

        for i in 0..7u8 {
            let msg = vec![0x10 + i; MAX_SESSION_MTU];
            sender.write_all(&msg).await?;
        }
        sender.close().await?;

        let mut received = Vec::new();
        let mut buf = vec![0u8; MAX_SESSION_MTU + 64];
        loop {
            let n = receiver.read(&mut buf).await?;
            if n == 0 {
                break;
            }
            received.push(buf[..n].to_vec());
        }

        assert_eq!(7, received.len(), "receiver must decode all 7 datagrams");
        for (i, msg) in received.iter().enumerate() {
            assert_eq!(MAX_SESSION_MTU, msg.len());
            assert!(msg.iter().all(|&b| b == 0x10 + i as u8));
        }

        // First datagram arrives after an idle gap and flushes immediately (0 ms TTFB);
        // the remaining 6 burst datagrams pack 2 per packet (2,924 B <= 3,238 B) plus the
        // terminating segment into 3 more packets -> 4 writes total (vs 7+1 without packing).
        let recorded = writes.lock().unwrap();
        assert!(
            recorded.len() <= 5 && recorded.iter().any(|b| b.len() >= 2 * MAX_SESSION_MTU),
            "expected burst full-MTU datagrams to pack 2/packet, got {} writes: {:?}",
            recorded.len(),
            recorded.iter().map(|b| b.len()).collect::<Vec<_>>()
        );
        Ok(())
    }

    #[tokio::test]
    async fn packs_small_datagrams_up_to_20_per_packet_without_splitting() -> anyhow::Result<()> {
        let (sender_pipe, receiver_pipe, writes) = RecordingPacketPipe::pair(vec![]);

        let sender_cfg = SessionSocketConfig {
            frame_size: MAX_SESSION_MTU,
            flush_immediately: true,
            datagram: true,
            pack_delay: Some(Duration::from_millis(20)),
            ..Default::default()
        };
        let receiver_cfg = SessionSocketConfig {
            frame_size: MAX_SESSION_MTU,
            flush_immediately: true,
            datagram: true,
            pack_delay: None,
            ..Default::default()
        };

        let mut sender = SessionSocket::<WIRE_MTU, _>::new_stateless(
            "sender",
            sender_pipe,
            sender_cfg,
            None,
            #[cfg(feature = "telemetry")]
            NoopTracker,
        )?;
        let mut receiver = SessionSocket::<WIRE_MTU, _>::new_stateless(
            "receiver",
            receiver_pipe,
            receiver_cfg,
            None,
            #[cfg(feature = "telemetry")]
            NoopTracker,
        )?;

        for i in 0..26u8 {
            let msg = vec![i; 148];
            sender.write_all(&msg).await?;
        }
        sender.close().await?;

        let mut received = Vec::new();
        let mut buf = vec![0u8; 512];
        loop {
            let n = receiver.read(&mut buf).await?;
            if n == 0 {
                break;
            }
            received.push(buf[..n].to_vec());
        }

        assert_eq!(26, received.len());
        for (i, msg) in received.iter().enumerate() {
            assert_eq!(148, msg.len());
            assert!(msg.iter().all(|&b| b == i as u8));
        }

        // Msg 0 flushes immediately (idle gap); msgs 1..=20 pack 20 × 158 B = 3,160 B into one
        // packet; remaining 5 + terminating segment fit in one more packet -> 3 writes total.
        let recorded = writes.lock().unwrap();
        assert!(
            recorded.iter().any(|b| b.len() == 3160),
            "expected a full 20-datagram packed packet (3160 B), got {:?}",
            recorded.iter().map(|b| b.len()).collect::<Vec<_>>()
        );
        assert!(recorded.len() <= 3, "got {:?}", recorded.len());
        Ok(())
    }

    #[tokio::test]
    async fn dropping_one_packed_packet_loses_only_its_datagrams() -> anyhow::Result<()> {
        // Drop packet index 1 (holding burst datagrams 1 and 2, after isolated datagram 0).
        let (sender_pipe, receiver_pipe, _writes) = RecordingPacketPipe::pair(vec![1]);

        let sender_cfg = SessionSocketConfig {
            frame_size: MAX_SESSION_MTU,
            flush_immediately: true,
            datagram: true,
            pack_delay: Some(Duration::from_millis(20)),
            ..Default::default()
        };
        let receiver_cfg = SessionSocketConfig {
            frame_size: MAX_SESSION_MTU,
            frame_timeout: Duration::from_millis(50),
            flush_immediately: true,
            datagram: true,
            pack_delay: None,
            ..Default::default()
        };

        let mut sender = SessionSocket::<WIRE_MTU, _>::new_stateless(
            "sender",
            sender_pipe,
            sender_cfg,
            None,
            #[cfg(feature = "telemetry")]
            NoopTracker,
        )?;
        let mut receiver = SessionSocket::<WIRE_MTU, _>::new_stateless(
            "receiver",
            receiver_pipe,
            receiver_cfg,
            None,
            #[cfg(feature = "telemetry")]
            NoopTracker,
        )?;

        for i in 0..7u8 {
            sender.write_all(&vec![0x20 + i; MAX_SESSION_MTU]).await?;
        }
        sender.close().await?;

        let mut received = Vec::new();
        let mut buf = vec![0u8; MAX_SESSION_MTU + 64];
        loop {
            let n = receiver.read(&mut buf).await?;
            if n == 0 {
                break;
            }
            received.push(buf[0]);
        }

        // Only the 2 datagrams in dropped packet 1 are lost; all others survive intact.
        assert_eq!(5, received.len(), "received: {received:?}");
        assert_eq!(0x20, received[0]);
        assert_eq!(0x26, *received.last().unwrap());
        Ok(())
    }

    #[tokio::test]
    async fn isolated_datagram_flushes_immediately_without_waiting_for_pack_delay() -> anyhow::Result<()> {
        let (sender_pipe, receiver_pipe, writes) = RecordingPacketPipe::pair(vec![]);

        let sender_cfg = SessionSocketConfig {
            frame_size: MAX_SESSION_MTU,
            flush_immediately: true,
            datagram: true,
            // Deliberately huge pack_delay: an isolated datagram must NOT wait for it.
            pack_delay: Some(Duration::from_secs(5)),
            ..Default::default()
        };
        let receiver_cfg = SessionSocketConfig {
            frame_size: MAX_SESSION_MTU,
            flush_immediately: true,
            datagram: true,
            pack_delay: None,
            ..Default::default()
        };

        let mut sender = SessionSocket::<WIRE_MTU, _>::new_stateless(
            "sender",
            sender_pipe,
            sender_cfg,
            None,
            #[cfg(feature = "telemetry")]
            NoopTracker,
        )?;
        let mut receiver = SessionSocket::<WIRE_MTU, _>::new_stateless(
            "receiver",
            receiver_pipe,
            receiver_cfg,
            None,
            #[cfg(feature = "telemetry")]
            NoopTracker,
        )?;

        let payload = b"single interactive ping datagram";
        sender.write_all(payload).await?;

        let mut buf = vec![0u8; 128];
        let n = tokio::time::timeout(Duration::from_millis(100), receiver.read(&mut buf)).await??;
        assert_eq!(payload.as_slice(), &buf[..n]);
        assert_eq!(1, writes.lock().unwrap().len());
        Ok(())
    }

    #[tokio::test]
    async fn nearly_full_packed_packet_flushes_immediately_without_timer_wait() -> anyhow::Result<()> {
        let (sender_pipe, receiver_pipe, writes) = RecordingPacketPipe::pair(vec![]);

        let sender_cfg = SessionSocketConfig {
            frame_size: MAX_SESSION_MTU,
            flush_immediately: true,
            datagram: true,
            // Huge pack_delay: once 2 × 1,462 B = 2,924 B are buffered (314 B left < 1,462 B),
            // the packer must flush immediately instead of waiting 5 seconds.
            pack_delay: Some(Duration::from_secs(5)),
            ..Default::default()
        };
        let receiver_cfg = SessionSocketConfig {
            frame_size: MAX_SESSION_MTU,
            flush_immediately: true,
            datagram: true,
            pack_delay: None,
            ..Default::default()
        };

        let mut sender = SessionSocket::<WIRE_MTU, _>::new_stateless(
            "sender",
            sender_pipe,
            sender_cfg,
            None,
            #[cfg(feature = "telemetry")]
            NoopTracker,
        )?;
        let mut receiver = SessionSocket::<WIRE_MTU, _>::new_stateless(
            "receiver",
            receiver_pipe,
            receiver_cfg,
            None,
            #[cfg(feature = "telemetry")]
            NoopTracker,
        )?;

        // Msg 0 is isolated (flushes immediately); msgs 1 and 2 form a full 2-segment packet
        // (2,924 B) that must flush immediately without waiting for the 5 s timer.
        for i in 0..3u8 {
            sender.write_all(&vec![0x30 + i; MAX_SESSION_MTU]).await?;
        }

        let mut buf = vec![0u8; MAX_SESSION_MTU + 64];
        for expected in 0..3u8 {
            let n = tokio::time::timeout(Duration::from_millis(200), receiver.read(&mut buf)).await??;
            assert_eq!(MAX_SESSION_MTU, n);
            assert_eq!(0x30 + expected, buf[0]);
        }
        assert_eq!(2, writes.lock().unwrap().len());
        Ok(())
    }

    #[tokio::test]
    async fn dropping_socket_aborts_background_task_even_when_writer_is_stalled() -> anyhow::Result<()> {
        struct StalledWriter {
            _drop_signal: oneshot::Sender<()>,
        }

        impl futures::AsyncRead for StalledWriter {
            fn poll_read(self: Pin<&mut Self>, _cx: &mut Context<'_>, _buf: &mut [u8]) -> Poll<std::io::Result<usize>> {
                Poll::Pending
            }
        }

        impl AsyncWrite for StalledWriter {
            fn poll_write(self: Pin<&mut Self>, _cx: &mut Context<'_>, _buf: &[u8]) -> Poll<std::io::Result<usize>> {
                Poll::Pending
            }

            fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
                Poll::Pending
            }

            fn poll_close(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
                Poll::Pending
            }
        }

        let (drop_tx, drop_rx) = oneshot::channel();
        let mut sender = SessionSocket::<WIRE_MTU, _>::new_stateless(
            "stalled",
            StalledWriter { _drop_signal: drop_tx },
            SessionSocketConfig {
                frame_size: MAX_SESSION_MTU,
                flush_immediately: true,
                datagram: true,
                pack_delay: Some(Duration::from_millis(10)),
                ..Default::default()
            },
            None,
            #[cfg(feature = "telemetry")]
            NoopTracker,
        )?;

        // Enqueue a datagram so the packer task is parked inside `writer.write_all`.
        sender.write_all(b"stuck").await?;
        tokio::time::sleep(Duration::from_millis(20)).await;

        // Dropping the socket without `close()` must abort the background task after the drop
        // flush grace window and drop the stalled writer.
        drop(sender);
        let res = tokio::time::timeout(Duration::from_millis(300), drop_rx)
            .await
            .map_err(|_| anyhow::anyhow!("background packer task leaked after socket drop"))?;
        assert!(res.is_err(), "expected oneshot::Canceled when writer is dropped");
        Ok(())
    }

    #[tokio::test]
    async fn dropping_socket_with_healthy_writer_flushes_queued_messages() -> anyhow::Result<()> {
        let (sender_pipe, receiver_pipe, _writes) = RecordingPacketPipe::pair(vec![]);

        let mut sender = SessionSocket::<WIRE_MTU, _>::new_stateless(
            "drop-flush",
            sender_pipe,
            SessionSocketConfig {
                frame_size: MAX_SESSION_MTU,
                flush_immediately: true,
                datagram: true,
                // Long pack_delay so msg 1 is still buffered in the packer when `sender` drops.
                pack_delay: Some(Duration::from_secs(5)),
                ..Default::default()
            },
            None,
            #[cfg(feature = "telemetry")]
            NoopTracker,
        )?;
        let mut receiver = SessionSocket::<WIRE_MTU, _>::new_stateless(
            "receiver",
            receiver_pipe,
            SessionSocketConfig {
                frame_size: MAX_SESSION_MTU,
                flush_immediately: true,
                datagram: true,
                pack_delay: None,
                ..Default::default()
            },
            None,
            #[cfg(feature = "telemetry")]
            NoopTracker,
        )?;

        sender.write_all(b"first").await?;
        sender.write_all(b"second-buffered").await?;
        // Drop without calling `close()`: the grace window must flush `second-buffered` cleanly.
        drop(sender);

        let mut received = Vec::new();
        let mut buf = vec![0u8; 64];
        loop {
            let n = tokio::time::timeout(Duration::from_millis(300), receiver.read(&mut buf)).await??;
            if n == 0 {
                break;
            }
            received.push(buf[..n].to_vec());
        }
        assert_eq!(vec![b"first".to_vec(), b"second-buffered".to_vec()], received);
        Ok(())
    }

    #[tokio::test]
    async fn inter_arrival_above_pack_delay_flushes_each_datagram_immediately() -> anyhow::Result<()> {
        let (sender_pipe, receiver_pipe, writes) = RecordingPacketPipe::pair(vec![]);
        let pack_delay = Duration::from_millis(40);

        let mut sender = SessionSocket::<WIRE_MTU, _>::new_stateless(
            "spaced",
            sender_pipe,
            SessionSocketConfig {
                frame_size: MAX_SESSION_MTU,
                flush_immediately: true,
                datagram: true,
                pack_delay: Some(pack_delay),
                ..Default::default()
            },
            None,
            #[cfg(feature = "telemetry")]
            NoopTracker,
        )?;
        let mut receiver = SessionSocket::<WIRE_MTU, _>::new_stateless(
            "receiver",
            receiver_pipe,
            SessionSocketConfig {
                frame_size: MAX_SESSION_MTU,
                flush_immediately: true,
                datagram: true,
                pack_delay: None,
                ..Default::default()
            },
            None,
            #[cfg(feature = "telemetry")]
            NoopTracker,
        )?;

        let mut buf = vec![0u8; 64];
        // Spacing = 55 ms (> pack_delay = 40 ms, but < 2 × pack_delay = 80 ms): with a 1× burst
        // window each datagram arrives outside the burst window and flushes immediately instead
        // of stalling for 40 ms without packing anything.
        for i in 0..4u8 {
            if i > 0 {
                tokio::time::sleep(Duration::from_millis(55)).await;
            }
            let sent_at = Instant::now();
            sender.write_all(&[i; 32]).await?;
            let n = tokio::time::timeout(Duration::from_millis(200), receiver.read(&mut buf)).await??;
            assert_eq!(32, n);
            assert!(
                sent_at.elapsed() < Duration::from_millis(30),
                "datagram {i} waited {:?} even though inter-arrival exceeded pack_delay",
                sent_at.elapsed()
            );
        }
        assert_eq!(4, writes.lock().unwrap().len());
        Ok(())
    }

    #[tokio::test]
    async fn stalled_writer_propagates_backpressure_within_bounded_channel() -> anyhow::Result<()> {
        struct BlockedWriter;

        impl futures::AsyncRead for BlockedWriter {
            fn poll_read(self: Pin<&mut Self>, _cx: &mut Context<'_>, _buf: &mut [u8]) -> Poll<std::io::Result<usize>> {
                Poll::Pending
            }
        }

        impl AsyncWrite for BlockedWriter {
            fn poll_write(self: Pin<&mut Self>, _cx: &mut Context<'_>, _buf: &[u8]) -> Poll<std::io::Result<usize>> {
                Poll::Pending
            }

            fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
                Poll::Pending
            }

            fn poll_close(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
                Poll::Pending
            }
        }

        let mut sender = SessionSocket::<WIRE_MTU, _>::new_stateless(
            "blocked",
            BlockedWriter,
            SessionSocketConfig {
                frame_size: MAX_SESSION_MTU,
                flush_immediately: true,
                datagram: true,
                pack_delay: Some(Duration::from_millis(10)),
                ..Default::default()
            },
            None,
            #[cfg(feature = "telemetry")]
            NoopTracker,
        )?;

        // With PACKER_CHANNEL_CAPACITY = 32, writing 64 datagrams must hit backpressure and block
        // rather than silently buffering hundreds of messages.
        let blocked = tokio::time::timeout(Duration::from_millis(100), async {
            for _ in 0..(PACKER_CHANNEL_CAPACITY + 16) {
                sender.write_all(b"bounded").await?;
            }
            Ok::<(), std::io::Error>(())
        })
        .await;
        assert!(
            blocked.is_err(),
            "expected write to block once packer channel ({PACKER_CHANNEL_CAPACITY}) filled"
        );
        Ok(())
    }

    #[tokio::test]
    async fn writer_error_preserves_io_error_kind() -> anyhow::Result<()> {
        use futures::SinkExt as _;

        struct FailingWriter;

        impl AsyncWrite for FailingWriter {
            fn poll_write(self: Pin<&mut Self>, _cx: &mut Context<'_>, _buf: &[u8]) -> Poll<std::io::Result<usize>> {
                Poll::Ready(Err(std::io::Error::new(
                    std::io::ErrorKind::ConnectionReset,
                    "peer reset",
                )))
            }

            fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
                Poll::Ready(Ok(()))
            }

            fn poll_close(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
                Poll::Ready(Ok(()))
            }
        }

        let mut sink = WholeMessagePackingSink::<WIRE_MTU>::new(
            FailingWriter,
            Duration::from_millis(5),
            None,
            tracing::Span::none(),
        );
        let seg = crate::Segment {
            frame_id: 1,
            seq_idx: 0,
            seq_flags: crate::protocol::SeqIndicator::new_with_flags(1, false),
            data: b"trigger".to_vec().into_boxed_slice(),
        };
        sink.send(SessionMessage::<WIRE_MTU>::Segment(seg.clone())).await?;
        tokio::time::sleep(Duration::from_millis(20)).await;
        let err = sink
            .send(SessionMessage::<WIRE_MTU>::Segment(seg))
            .await
            .expect_err("expected ConnectionReset error to propagate");
        match err {
            SessionError::IoError(io_err) => assert_eq!(std::io::ErrorKind::ConnectionReset, io_err.kind()),
            other => panic!("expected SessionError::IoError(ConnectionReset), got {other:?}"),
        }
        Ok(())
    }

    #[tokio::test]
    async fn pack_budget_dynamically_limits_packet_size() -> anyhow::Result<()> {
        use std::sync::atomic::{AtomicUsize, Ordering};

        let (sender_pipe, receiver_pipe, writes) = RecordingPacketPipe::pair(vec![]);
        let budget_bytes = Arc::new(AtomicUsize::new(2000));
        let budget_clone = budget_bytes.clone();

        let mut sender = SessionSocket::<WIRE_MTU, _>::new_stateless_with_pack_budget(
            "budgeted",
            sender_pipe,
            SessionSocketConfig {
                frame_size: MAX_SESSION_MTU,
                flush_immediately: true,
                datagram: true,
                pack_delay: Some(Duration::from_millis(20)),
                ..Default::default()
            },
            None,
            Some(Arc::new(move || budget_clone.load(Ordering::Relaxed))),
            #[cfg(feature = "telemetry")]
            NoopTracker,
        )?;
        let mut receiver = SessionSocket::<WIRE_MTU, _>::new_stateless(
            "receiver",
            receiver_pipe,
            SessionSocketConfig {
                frame_size: MAX_SESSION_MTU,
                flush_immediately: true,
                datagram: true,
                pack_delay: None,
                ..Default::default()
            },
            None,
            #[cfg(feature = "telemetry")]
            NoopTracker,
        )?;

        // With budget = 2,000 B, two full-MTU segments (2,924 B) cannot be packed together.
        for i in 0..3u8 {
            sender.write_all(&vec![0x40 + i; MAX_SESSION_MTU]).await?;
        }
        // Wait until all 3 writes have been emitted while budget is still 2,000 B.
        let mut buf = vec![0u8; MAX_SESSION_MTU + 64];
        for _ in 0..3 {
            let _ = tokio::time::timeout(Duration::from_millis(200), receiver.read(&mut buf)).await??;
        }
        assert_eq!(3, writes.lock().unwrap().len(), "budget=2000 must send 1 full-MTU/pkt");

        // Raise budget to full WIRE_MTU (3,238 B): burst datagrams now pack 2/packet.
        budget_bytes.store(WIRE_MTU, Ordering::Relaxed);
        writes.lock().unwrap().clear();
        for i in 0..4u8 {
            sender.write_all(&vec![0x50 + i; MAX_SESSION_MTU]).await?;
        }
        sender.close().await?;
        for _ in 0..4 {
            let _ = tokio::time::timeout(Duration::from_millis(200), receiver.read(&mut buf)).await??;
        }
        assert!(
            writes.lock().unwrap().iter().any(|b| b.len() >= 2 * MAX_SESSION_MTU),
            "budget=WIRE_MTU must pack 2 full-MTU datagrams per packet: {:?}",
            writes.lock().unwrap().iter().map(|b| b.len()).collect::<Vec<_>>()
        );
        Ok(())
    }
}
