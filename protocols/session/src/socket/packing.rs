//! Whole-message datagram packer for stateless [`SessionSocket`](super::SessionSocket) writes.
//!
//! Accumulates complete encoded [`SessionMessage`]s into a single transport write of at most `C`
//! bytes (one HOPR packet payload) without ever splitting a message across transport writes.
//! The message-boundary and packet-loss isolation guarantees hold when the underlying `AsyncWrite`
//! accepts each packed slice in a single `poll_write` call (as a packet-atomic transport does); a
//! writer that returns short writes allows `write_all` to split a message across transport writes
//! and invalidates strict single-packet loss isolation. Under a packet-atomic writer, an
//! unmodified receiver using [`SessionCodec`] decodes all packed messages in order and isolates
//! packet loss strictly to the messages contained in the dropped packet.

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
    future::{Fuse, FutureExt},
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
#[derive(Clone)]
pub struct PackPayloadBudget(Arc<dyn Fn() -> usize + Send + Sync>);

impl PackPayloadBudget {
    /// Wraps a dynamic byte-limit callback.
    #[must_use]
    pub fn new(f: impl Fn() -> usize + Send + Sync + 'static) -> Self {
        Self(Arc::new(f))
    }

    /// Evaluates the current byte cap.
    pub(crate) fn current_limit(&self) -> usize {
        (self.0)()
    }
}

impl std::fmt::Debug for PackPayloadBudget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("PackPayloadBudget").field(&"..").finish()
    }
}

#[derive(Clone)]
struct StoredError {
    kind: std::io::ErrorKind,
    message: String,
}

impl StoredError {
    fn from_io(err: &std::io::Error) -> Self {
        Self {
            kind: err.kind(),
            message: err.to_string(),
        }
    }

    fn to_session_error(&self) -> SessionError {
        SessionError::IoError(std::io::Error::new(self.kind, self.message.clone()))
    }
}

fn broken_pipe(msg: impl Into<Box<dyn std::error::Error + Send + Sync>>) -> SessionError {
    SessionError::IoError(std::io::Error::new(std::io::ErrorKind::BrokenPipe, msg))
}

enum PackerCommand {
    Message(Bytes),
    Batch(Vec<Bytes>),
    Flush(oneshot::Sender<std::io::Result<()>>),
}

enum PackerEvent {
    TimerExpired,
    Command(PackerCommand),
    Closed,
}

struct FlushTimer {
    deadline: Instant,
    sleep: Pin<Box<Fuse<futures_time::task::Sleep>>>,
}

impl FlushTimer {
    fn arm_until(deadline: Instant, remaining: Duration) -> Self {
        Self {
            deadline,
            sleep: Box::pin(futures_time::task::sleep(remaining.into()).fuse()),
        }
    }

    fn is_expired(&self, now: Instant) -> bool {
        now >= self.deadline
    }
}

/// Outcome of pushing one encoded [`SessionMessage`] into [`PackBuffer`].
#[derive(Debug, PartialEq, Eq)]
struct PushOutcome {
    /// Previously buffered packet(s) evicted before appending the incoming message because adding
    /// it would exceed the effective byte budget (regrouped along message boundaries if the budget
    /// shrank while messages were buffered).
    overflow: Vec<Bytes>,
    /// Packet flushed immediately after appending the incoming message (isolated datagram, zero
    /// `pack_delay`, or remaining budget too small for another message).
    immediate: Option<Bytes>,
}

/// Pure whole-message accumulator deciding when buffered [`SessionMessage`]s overflow, pack, or
/// flush immediately.
struct PackBuffer<const C: usize> {
    buf: BytesMut,
    /// Encoded lengths of the whole messages currently stored in `buf`, preserving message
    /// boundaries so buffered batches can be regrouped if the dynamic payload budget shrinks
    /// before emission.
    msg_lens: Vec<usize>,
    pack_delay: Duration,
    min_msg: usize,
    last_msg_at: Option<Instant>,
}

impl<const C: usize> PackBuffer<C> {
    fn new(pack_delay: Duration) -> Self {
        Self {
            buf: BytesMut::with_capacity(C),
            msg_lens: Vec::new(),
            pack_delay,
            min_msg: SessionMessage::<C>::minimum_message_size(),
            last_msg_at: None,
        }
    }

    /// Appends one complete encoded message at `now` under `budget_limit` (clamped to `C`).
    fn push(&mut self, msg: &[u8], now: Instant, budget_limit: usize) -> PushOutcome {
        let in_burst = self
            .last_msg_at
            .is_some_and(|t| now.saturating_duration_since(t) <= self.pack_delay);
        self.last_msg_at = Some(now);

        let effective_limit = budget_limit.min(C);
        let overflow = if !self.buf.is_empty() && self.buf.len().saturating_add(msg.len()) > effective_limit {
            self.take_chunks(effective_limit)
        } else {
            Vec::new()
        };

        self.buf.extend_from_slice(msg);
        self.msg_lens.push(msg.len());

        let isolated = !in_burst && self.buf.len() == msg.len();
        let remaining = effective_limit.saturating_sub(self.buf.len());
        let cannot_fit_another = remaining < msg.len().max(self.min_msg);

        let immediate = if self.pack_delay.is_zero() || isolated || cannot_fit_another {
            self.msg_lens.clear();
            Some(self.buf.split().freeze())
        } else {
            None
        };

        PushOutcome { overflow, immediate }
    }

    /// Takes any currently buffered packet bytes, regrouping along whole-message boundaries if
    /// `budget_limit` (clamped to `C`) shrank below the currently buffered total.
    ///
    /// Preserves the combined batch without copying when the current budget still fits (or only a
    /// single message is buffered, preserving single-message fallback).
    fn take_chunks(&mut self, budget_limit: usize) -> Vec<Bytes> {
        if self.buf.is_empty() {
            return Vec::new();
        }

        let effective_limit = budget_limit.min(C);
        if self.buf.len() <= effective_limit || self.msg_lens.len() <= 1 {
            self.msg_lens.clear();
            return vec![self.buf.split().freeze()];
        }

        let mut chunks = Vec::new();
        let mut chunk_len = 0usize;
        for msg_len in self.msg_lens.drain(..) {
            if chunk_len > 0 && chunk_len.saturating_add(msg_len) > effective_limit {
                chunks.push(self.buf.split_to(chunk_len).freeze());
                chunk_len = 0;
            }
            chunk_len = chunk_len.saturating_add(msg_len);
        }
        if chunk_len > 0 {
            chunks.push(self.buf.split_to(chunk_len).freeze());
        }
        debug_assert!(self.buf.is_empty());
        chunks
    }

    #[cfg(test)]
    fn buffered_len(&self) -> usize {
        self.buf.len()
    }
}

/// A [`Sink`] that packs complete encoded [`SessionMessage`]s into transport writes of at most `C`
/// bytes, flushing whenever:
/// - the next message would exceed the effective byte budget `<= C`,
/// - an isolated message arrives after an idle gap (no prior message within the burst window),
/// - the remaining space cannot fit another message of the same size (or `minimum_message_size`),
/// - `pack_delay` elapses,
/// - `poll_flush` is called, or
/// - the sink is closed.
#[must_use = "sinks do nothing unless polled"]
pub(crate) struct WholeMessagePackingSink<const C: usize> {
    tx: Option<mpsc::Sender<PackerCommand>>,
    /// Encoded segments of a multi-segment frame being collected so the entire frame submits into
    /// a single reserved channel slot without ever parking mid-frame in `WriteState::Draining`.
    pending_frame: Vec<Bytes>,
    pending_flush: Option<oneshot::Receiver<std::io::Result<()>>>,
    done: Option<oneshot::Receiver<std::io::Result<()>>>,
    /// Dropped when the sink is dropped without completing `poll_close`, arming the in-task
    /// `DROP_FLUSH_GRACE` timeout without requiring `tokio::spawn` inside `Drop`.
    drop_guard: Option<oneshot::Sender<()>>,
    error: Arc<Mutex<Option<StoredError>>>,
    encode_buf: BytesMut,
}

const _: () = {
    const fn assert_send<T: Send>() {}
    const fn assert_send_sync<T: Send + Sync>() {}
    assert_send::<WholeMessagePackingSink<1500>>();
    assert_send_sync::<PackPayloadBudget>();
};

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
                    _ = grace => {
                        tracing::debug!(
                            grace_ms = DROP_FLUSH_GRACE.as_millis() as u64,
                            "aborting stalled datagram packer task after drop grace period"
                        );
                    }
                }
            }
            .instrument(span),
        );

        Self {
            tx: Some(tx),
            pending_frame: Vec::new(),
            pending_flush: None,
            done: Some(done_rx),
            drop_guard: Some(drop_tx),
            error,
            encode_buf: BytesMut::with_capacity(C),
        }
    }

    fn stored_error(&self) -> Option<SessionError> {
        self.error.lock().as_ref().map(StoredError::to_session_error)
    }
}

async fn flush_packet<W: AsyncWrite + Unpin>(writer: &mut W, pkt: &[u8]) -> std::io::Result<()> {
    if pkt.is_empty() {
        return Ok(());
    }
    writer.write_all(pkt).await?;
    writer.flush().await
}

async fn flush_chunks<W: AsyncWrite + Unpin>(writer: &mut W, chunks: Vec<Bytes>) -> std::io::Result<()> {
    for pkt in chunks {
        flush_packet(writer, &pkt).await?;
    }
    Ok(())
}

async fn flush_packet_and_writer<const C: usize, W: AsyncWrite + Unpin>(
    writer: &mut W,
    pack_buf: &mut PackBuffer<C>,
    limit: usize,
) -> std::io::Result<()> {
    let chunks = pack_buf.take_chunks(limit);
    if chunks.is_empty() {
        writer.flush().await
    } else {
        flush_chunks(writer, chunks).await
    }
}

async fn process_packed_message<const C: usize, W: AsyncWrite + Unpin>(
    msg: Bytes,
    writer: &mut W,
    pack_buf: &mut PackBuffer<C>,
    timer: &mut Option<FlushTimer>,
    pack_delay: Duration,
    budget: Option<&PackPayloadBudget>,
) -> std::io::Result<()> {
    let now = Instant::now();
    if timer.as_ref().is_some_and(|t| t.is_expired(now)) {
        *timer = None;
        let limit = budget.map_or(C, PackPayloadBudget::current_limit);
        flush_chunks(writer, pack_buf.take_chunks(limit)).await?;
    }

    let buffered_at = Instant::now();
    let limit = budget.map_or(C, PackPayloadBudget::current_limit);
    let outcome = pack_buf.push(&msg, buffered_at, limit);

    if !outcome.overflow.is_empty() {
        *timer = None;
        flush_chunks(writer, outcome.overflow).await?;
    }

    if let Some(immediate) = outcome.immediate {
        *timer = None;
        flush_packet(writer, &immediate).await?;
    } else if timer.is_none() {
        // `pack_buf` holds `msg`, which was buffered at `buffered_at`. If writing an overflow packet
        // already spent `>= pack_delay` (or `pack_delay` overflows `Instant`), emit the buffered
        // packet right away rather than panicking or starting a fresh full window; otherwise arm
        // for the remaining time until `buffered_at + pack_delay`.
        let after_write = Instant::now();
        match buffered_at.checked_add(pack_delay) {
            Some(deadline) if after_write < deadline => {
                *timer = Some(FlushTimer::arm_until(deadline, deadline - after_write));
            }
            _ => {
                let current_limit = budget.map_or(C, PackPayloadBudget::current_limit);
                flush_chunks(writer, pack_buf.take_chunks(current_limit)).await?;
            }
        }
    }

    Ok(())
}

async fn run_packer<const C: usize, W>(
    mut rx: mpsc::Receiver<PackerCommand>,
    mut writer: W,
    pack_delay: Duration,
    budget: Option<PackPayloadBudget>,
    error_slot: Arc<Mutex<Option<StoredError>>>,
    done_tx: oneshot::Sender<std::io::Result<()>>,
) where
    W: AsyncWrite + Unpin,
{
    let mut pack_buf = PackBuffer::<C>::new(pack_delay);
    let mut timer: Option<FlushTimer> = None;

    let result: std::io::Result<()> = loop {
        let event = if let Some(t) = timer.as_mut() {
            futures::select_biased! {
                _ = t.sleep.as_mut() => PackerEvent::TimerExpired,
                cmd = rx.next() => cmd.map_or(PackerEvent::Closed, PackerEvent::Command),
            }
        } else {
            rx.next().await.map_or(PackerEvent::Closed, PackerEvent::Command)
        };

        match event {
            PackerEvent::TimerExpired => {
                timer = None;
                let limit = budget.as_ref().map_or(C, PackPayloadBudget::current_limit);
                if let Err(err) = flush_chunks(&mut writer, pack_buf.take_chunks(limit)).await {
                    break Err(err);
                }
            }
            PackerEvent::Closed => {
                let limit = budget.as_ref().map_or(C, PackPayloadBudget::current_limit);
                if let Err(err) = flush_chunks(&mut writer, pack_buf.take_chunks(limit)).await {
                    break Err(err);
                }
                break writer.close().await;
            }
            PackerEvent::Command(PackerCommand::Flush(ack_tx)) => {
                timer = None;
                let limit = budget.as_ref().map_or(C, PackPayloadBudget::current_limit);
                match flush_packet_and_writer(&mut writer, &mut pack_buf, limit).await {
                    Ok(()) => {
                        let _ = ack_tx.send(Ok(()));
                    }
                    Err(err) => {
                        *error_slot.lock() = Some(StoredError::from_io(&err));
                        let _ = ack_tx.send(Err(std::io::Error::new(err.kind(), err.to_string())));
                        break Err(err);
                    }
                }
            }
            PackerEvent::Command(PackerCommand::Message(msg)) => {
                if let Err(err) =
                    process_packed_message(msg, &mut writer, &mut pack_buf, &mut timer, pack_delay, budget.as_ref())
                        .await
                {
                    break Err(err);
                }
            }
            PackerEvent::Command(PackerCommand::Batch(msgs)) => {
                let mut batch_err = None;
                for msg in msgs {
                    if let Err(err) =
                        process_packed_message(msg, &mut writer, &mut pack_buf, &mut timer, pack_delay, budget.as_ref())
                            .await
                    {
                        batch_err = Some(err);
                        break;
                    }
                }
                if let Some(err) = batch_err {
                    break Err(err);
                }
            }
        }
    };

    if let Err(ref err) = result {
        *error_slot.lock() = Some(StoredError::from_io(err));
    }
    let _ = done_tx.send(result);
}

impl<const C: usize> Sink<SessionMessage<C>> for WholeMessagePackingSink<C> {
    type Error = SessionError;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        if let Some(err) = self.stored_error() {
            return Poll::Ready(Err(err));
        }
        let this = self.as_mut().get_mut();
        let Some(tx) = this.tx.as_mut() else {
            return Poll::Ready(Err(broken_pipe("packer sink closed")));
        };
        // Remaining segments of a multi-segment frame share the single channel slot reserved
        // before segment 0, so `poll_drain_segments` never parks mid-frame.
        if !this.pending_frame.is_empty() {
            return Poll::Ready(Ok(()));
        }
        tx.poll_ready(cx)
            .map_err(|e| this.stored_error().unwrap_or_else(|| broken_pipe(e)))
    }

    fn start_send(mut self: Pin<&mut Self>, item: SessionMessage<C>) -> Result<(), Self::Error> {
        if let Some(err) = self.stored_error() {
            return Err(err);
        }
        let multi_seg_state = match &item {
            SessionMessage::Segment(seg) if seg.seq_flags.seq_len() > 1 => Some((seg.seq_idx, seg.seq_flags.seq_len())),
            _ => None,
        };
        let this = self.as_mut().get_mut();
        this.encode_buf.clear();
        SessionCodec::<C>.encode(item, &mut this.encode_buf)?;
        let encoded = this.encode_buf.split().freeze();
        // Any in-flight `PackerCommand::Flush` was queued before this message in the FIFO channel
        // and therefore cannot acknowledge it. Drop the stale receiver so the next `poll_flush`
        // enqueues a fresh flush after this message.
        this.pending_flush = None;

        if let Some((seq_idx, seq_len)) = multi_seg_state {
            this.pending_frame.push(encoded);
            if seq_idx.saturating_add(1) < seq_len {
                return Ok(());
            }
            let batch = std::mem::take(&mut this.pending_frame);
            let Some(tx) = this.tx.as_mut() else {
                return Err(broken_pipe("packer sink closed"));
            };
            tx.start_send(PackerCommand::Batch(batch))
                .map_err(|e| this.stored_error().unwrap_or_else(|| broken_pipe(e)))?;
            return Ok(());
        }

        let Some(tx) = this.tx.as_mut() else {
            return Err(broken_pipe("packer sink closed"));
        };
        let cmd = if this.pending_frame.is_empty() {
            PackerCommand::Message(encoded)
        } else {
            let mut batch = std::mem::take(&mut this.pending_frame);
            batch.push(encoded);
            PackerCommand::Batch(batch)
        };
        tx.start_send(cmd)
            .map_err(|e| this.stored_error().unwrap_or_else(|| broken_pipe(e)))?;
        Ok(())
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let this = self.as_mut().get_mut();
        if !this.pending_frame.is_empty() {
            let Some(tx) = this.tx.as_mut() else {
                this.pending_frame.clear();
                return Poll::Ready(Err(this
                    .stored_error()
                    .unwrap_or_else(|| broken_pipe("packer sink closed"))));
            };
            let batch = std::mem::take(&mut this.pending_frame);
            if let Err(e) = tx.start_send(PackerCommand::Batch(batch)) {
                return Poll::Ready(Err(this.stored_error().unwrap_or_else(|| broken_pipe(e))));
            }
            this.pending_flush = None;
        }
        let ack_rx = match this.pending_flush.as_mut() {
            Some(rx) => rx,
            None => {
                if let Some(err) = this.stored_error() {
                    return Poll::Ready(Err(err));
                }
                let Some(tx) = this.tx.as_mut() else {
                    return Poll::Ready(Err(broken_pipe("packer sink closed")));
                };
                if let Err(e) = futures::ready!(tx.poll_ready(cx)) {
                    return Poll::Ready(Err(this.stored_error().unwrap_or_else(|| broken_pipe(e))));
                }
                let (ack_tx, ack_rx) = oneshot::channel();
                if let Err(e) = tx.start_send(PackerCommand::Flush(ack_tx)) {
                    return Poll::Ready(Err(this.stored_error().unwrap_or_else(|| broken_pipe(e))));
                }
                this.pending_flush.insert(ack_rx)
            }
        };

        match ack_rx.poll_unpin(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Ok(Ok(()))) => {
                this.pending_flush = None;
                Poll::Ready(Ok(()))
            }
            Poll::Ready(Ok(Err(io_err))) => {
                this.pending_flush = None;
                Poll::Ready(Err(SessionError::IoError(io_err)))
            }
            Poll::Ready(Err(_)) => {
                this.pending_flush = None;
                Poll::Ready(Err(this.stored_error().unwrap_or_else(|| {
                    broken_pipe("packer task terminated before flush completed")
                })))
            }
        }
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let this = self.as_mut().get_mut();
        if !this.pending_frame.is_empty()
            && let Some(tx) = this.tx.as_mut()
        {
            let batch = std::mem::take(&mut this.pending_frame);
            let _ = tx.start_send(PackerCommand::Batch(batch));
        }
        // Drop the sender to signal EOF so the background task flushes any buffered messages and
        // closes the underlying writer.
        this.tx.take();
        this.pending_flush = None;
        if let Some(done) = this.done.as_mut() {
            match done.poll_unpin(cx) {
                Poll::Ready(Ok(res)) => {
                    this.done = None;
                    if let Some(guard) = this.drop_guard.take() {
                        let _ = guard.send(());
                    }
                    return Poll::Ready(res.map_err(SessionError::IoError));
                }
                Poll::Ready(Err(_)) => {
                    this.done = None;
                    if let Some(guard) = this.drop_guard.take() {
                        let _ = guard.send(());
                    }
                    let err = this.stored_error().unwrap_or_else(|| {
                        let io_err =
                            std::io::Error::new(std::io::ErrorKind::BrokenPipe, "packer task terminated unexpectedly");
                        *this.error.lock() = Some(StoredError::from_io(&io_err));
                        SessionError::IoError(io_err)
                    });
                    return Poll::Ready(Err(err));
                }
                Poll::Pending => return Poll::Pending,
            }
        }
        match this.stored_error() {
            Some(err) => Poll::Ready(Err(err)),
            None => Poll::Ready(Ok(())),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use anyhow::Context as _;
    use futures::{AsyncReadExt, AsyncWriteExt, SinkExt as _};
    use hopr_protocol_app::prelude::ApplicationData;
    use parking_lot::Mutex;

    use super::*;
    use crate::{
        MAX_SESSION_MTU,
        socket::{SessionSocket, SessionSocketConfig, state::Stateless},
    };

    const WIRE_MTU: usize = ApplicationData::PAYLOAD_SIZE; // 3,238 B

    /// In-memory packet transport that records every individual `poll_write` slice (representing
    /// one HOPR packet payload) and delivers them atomically to a receiver.
    struct RecordingPacketPipe {
        tx: mpsc::UnboundedSender<Bytes>,
        rx: mpsc::UnboundedReceiver<Bytes>,
        read_buf: BytesMut,
        recorded_writes: Arc<Mutex<Vec<Bytes>>>,
        drop_write_indices: Arc<Mutex<Vec<usize>>>,
    }

    impl RecordingPacketPipe {
        fn pair(drop_indices: Vec<usize>) -> (Self, Self, Arc<Mutex<Vec<Bytes>>>) {
            let (a_tx, b_rx) = mpsc::unbounded();
            let (b_tx, a_rx) = mpsc::unbounded();
            let recorded = Arc::new(Mutex::new(Vec::new()));
            let drop_indices = Arc::new(Mutex::new(drop_indices));
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
                recorded_writes: Arc::new(Mutex::new(Vec::new())),
                drop_write_indices: Arc::new(Mutex::new(Vec::new())),
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
            let mut rec = self.recorded_writes.lock();
            let idx = rec.len();
            let bytes = Bytes::copy_from_slice(buf);
            rec.push(bytes.clone());
            if !self.drop_write_indices.lock().contains(&idx) {
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

    type PackedSocketPair = (
        SessionSocket<WIRE_MTU, Stateless<WIRE_MTU>>,
        SessionSocket<WIRE_MTU, Stateless<WIRE_MTU>>,
        Arc<Mutex<Vec<Bytes>>>,
    );

    fn datagram_socket_cfg(pack_delay: Option<Duration>) -> SessionSocketConfig {
        SessionSocketConfig {
            frame_size: MAX_SESSION_MTU,
            flush_immediately: true,
            datagram: true,
            pack_delay,
            ..Default::default()
        }
    }

    /// Builds a packed sender and an unmodified (v5.0.4: `pack_delay: None`) receiver wired
    /// through a [`RecordingPacketPipe`].
    fn make_packed_socket_pair(
        pack_delay: Duration,
        drop_indices: Vec<usize>,
        budget: Option<PackPayloadBudget>,
        receiver_frame_timeout: Option<Duration>,
    ) -> anyhow::Result<PackedSocketPair> {
        let (sender_pipe, receiver_pipe, writes) = RecordingPacketPipe::pair(drop_indices);
        let sender = SessionSocket::<WIRE_MTU, _>::stateless_builder(
            "sender",
            sender_pipe,
            datagram_socket_cfg(Some(pack_delay)),
        )
        .pack_budget(budget)
        .build()?;

        let mut receiver_cfg = datagram_socket_cfg(None);
        if let Some(timeout) = receiver_frame_timeout {
            receiver_cfg.frame_timeout = timeout;
        }
        let receiver =
            SessionSocket::<WIRE_MTU, _>::stateless_builder("receiver", receiver_pipe, receiver_cfg).build()?;

        Ok((sender, receiver, writes))
    }

    #[test]
    fn pack_payload_budget_and_stateless_builder_debug_are_opaque() {
        let budget = PackPayloadBudget::new(|| 1024);
        assert_eq!(1024, budget.current_limit());
        assert_eq!("PackPayloadBudget(\"..\")", format!("{budget:?}"));

        let (pipe, _peer, _writes) = RecordingPacketPipe::pair(vec![]);
        let builder = SessionSocket::<WIRE_MTU, _>::stateless_builder(
            "debug-check",
            pipe,
            datagram_socket_cfg(Some(Duration::from_millis(10))),
        )
        .pack_budget(budget);
        let dbg = format!("{builder:?}");
        assert!(dbg.contains("StatelessSocketBuilder"), "unexpected Debug output: {dbg}");
        assert!(dbg.contains("PackPayloadBudget"), "unexpected Debug output: {dbg}");
    }

    #[test]
    fn pack_buffer_handles_isolated_burst_overflow_and_budget_limits() -> anyhow::Result<()> {
        let t0 = Instant::now();
        let delay = Duration::from_millis(20);
        let mut buf = PackBuffer::<200>::new(delay);

        // First message after idle is isolated and flushes immediately.
        let out = buf.push(&[1u8; 50], t0, 200);
        assert!(out.overflow.is_empty());
        assert_eq!(Some(Bytes::from(vec![1u8; 50])), out.immediate);
        assert_eq!(0, buf.buffered_len());

        // Second message within burst window stays buffered when another of its size fits.
        let t1 = t0 + Duration::from_millis(5);
        let out = buf.push(&[2u8; 50], t1, 200);
        assert_eq!(
            PushOutcome {
                overflow: Vec::new(),
                immediate: None
            },
            out
        );
        assert_eq!(50, buf.buffered_len());

        // Third message fits alongside second; 100 B left >= 50 B, so still buffered.
        let t2 = t1 + Duration::from_millis(5);
        let out = buf.push(&[3u8; 50], t2, 200);
        assert_eq!(
            PushOutcome {
                overflow: Vec::new(),
                immediate: None
            },
            out
        );
        assert_eq!(100, buf.buffered_len());

        // Fourth message (80 B) brings total to 180 B; only 20 B left (< 80 B), so flushes immediately.
        let t3 = t2 + Duration::from_millis(5);
        let out = buf.push(&[4u8; 80], t3, 200);
        assert!(out.overflow.is_empty());
        let pkt = out.immediate.context("near-full buffer must flush immediately")?;
        assert_eq!(180, pkt.len());
        assert_eq!(0, buf.buffered_len());

        // Overflow case: buffer 60 B, then push 80 B with a dynamic budget of 100 B.
        // The 60 B packet overflows and flushes first; the 80 B message has 20 B remaining
        // (< 80 B) so it flushes immediately too.
        let t4 = t3 + Duration::from_millis(5);
        assert_eq!(
            PushOutcome {
                overflow: Vec::new(),
                immediate: None
            },
            buf.push(&[5u8; 60], t4, 200)
        );
        let t5 = t4 + Duration::from_millis(5);
        let out = buf.push(&[6u8; 80], t5, 100);
        assert_eq!(vec![Bytes::from(vec![5u8; 60])], out.overflow);
        assert_eq!(Some(Bytes::from(vec![6u8; 80])), out.immediate);
        assert_eq!(0, buf.buffered_len());

        // Dynamic budget > C is clamped to C: two 120 B messages overflow 200 B even if budget=500.
        let t6 = t5 + Duration::from_millis(5);
        let out = buf.push(&[7u8; 120], t6, 500);
        // 200 - 120 = 80 < 120 -> flushes immediately because budget is clamped to C=200.
        assert!(out.overflow.is_empty());
        assert_eq!(Some(Bytes::from(vec![7u8; 120])), out.immediate);

        // After an idle gap > pack_delay, the next message is isolated again.
        let t7 = t6 + Duration::from_millis(25);
        let out = buf.push(&[8u8; 40], t7, 200);
        assert!(out.overflow.is_empty());
        assert_eq!(Some(Bytes::from(vec![8u8; 40])), out.immediate);
        Ok(())
    }

    #[test]
    fn pack_buffer_with_zero_delay_always_flushes_immediately() {
        let t0 = Instant::now();
        let mut buf = PackBuffer::<200>::new(Duration::ZERO);
        for i in 0..3u8 {
            let out = buf.push(&[i; 30], t0, 200);
            assert!(out.overflow.is_empty());
            assert_eq!(Some(Bytes::from(vec![i; 30])), out.immediate);
        }
        assert!(buf.take_chunks(200).is_empty());
    }

    #[test]
    fn pack_buffer_regroups_buffered_messages_when_budget_shrinks() {
        let t0 = Instant::now();
        let mut buf = PackBuffer::<1_000>::new(Duration::from_millis(20));
        // Seed burst state with an isolated write.
        let _ = buf.push(&[0x00; 100], t0, 1_000);
        // Buffer two 300 B messages under a full 1000 B budget (total 600 B).
        assert_eq!(
            PushOutcome {
                overflow: Vec::new(),
                immediate: None
            },
            buf.push(&[0x11; 300], t0 + Duration::from_millis(1), 1_000)
        );
        assert_eq!(
            PushOutcome {
                overflow: Vec::new(),
                immediate: None
            },
            buf.push(&[0x22; 300], t0 + Duration::from_millis(2), 1_000)
        );
        // Unchanged budget emits a single combined 600 B batch.
        let combined = buf.take_chunks(1_000);
        assert_eq!(1, combined.len());
        assert_eq!(600, combined[0].len());

        // Buffer two 300 B messages again, then shrink budget to 473 B before emission:
        // take_chunks must regroup along whole-message boundaries into [300, 300].
        assert!(
            buf.push(&[0x11; 300], t0 + Duration::from_millis(3), 1_000)
                .immediate
                .is_none()
        );
        assert!(
            buf.push(&[0x22; 300], t0 + Duration::from_millis(4), 1_000)
                .immediate
                .is_none()
        );
        let split = buf.take_chunks(473);
        assert_eq!(vec![Bytes::from(vec![0x11; 300]), Bytes::from(vec![0x22; 300])], split);

        // Overflow under a shrunken budget also regroups the evicted batch.
        assert!(
            buf.push(&[0x33; 300], t0 + Duration::from_millis(5), 1_000)
                .immediate
                .is_none()
        );
        assert!(
            buf.push(&[0x44; 300], t0 + Duration::from_millis(6), 1_000)
                .immediate
                .is_none()
        );
        let out = buf.push(&[0x55; 100], t0 + Duration::from_millis(7), 473);
        assert_eq!(
            vec![Bytes::from(vec![0x33; 300]), Bytes::from(vec![0x44; 300])],
            out.overflow
        );
        assert_eq!(None, out.immediate);
        assert_eq!(vec![Bytes::from(vec![0x55; 100])], buf.take_chunks(473));

        // Single-message fallback: a lone message larger than the shrunk budget still emits intact.
        assert!(
            buf.push(&[0x66; 500], t0 + Duration::from_millis(8), 1_000)
                .immediate
                .is_none()
        );
        assert_eq!(vec![Bytes::from(vec![0x66; 500])], buf.take_chunks(400));
    }

    #[tokio::test]
    async fn packs_two_full_mtu_datagrams_per_packet_and_unmodified_receiver_decodes_all() -> anyhow::Result<()> {
        let (mut sender, mut receiver, writes) =
            make_packed_socket_pair(Duration::from_millis(20), vec![], None, None)?;

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
        let recorded = writes.lock();
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
        let (mut sender, mut receiver, writes) =
            make_packed_socket_pair(Duration::from_millis(20), vec![], None, None)?;

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
        let recorded = writes.lock();
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
        let (mut sender, mut receiver, writes) = make_packed_socket_pair(
            Duration::from_millis(20),
            vec![1],
            None,
            Some(Duration::from_millis(50)),
        )?;

        for i in 0..7u8 {
            sender.write_all(&vec![0x20 + i; MAX_SESSION_MTU]).await?;
        }
        sender.close().await?;

        // Verify the dropped write (write #1) held exactly the two expected datagrams (0x21 and 0x22).
        let dropped_write = writes
            .lock()
            .get(1)
            .cloned()
            .context("expected dropped packet write at index 1")?;
        let mut decode_buf = BytesMut::from(dropped_write.as_ref());
        let mut dropped_payloads = Vec::new();
        while let Some(msg) = asynchronous_codec::Decoder::decode(&mut SessionCodec::<WIRE_MTU>, &mut decode_buf)? {
            if let SessionMessage::Segment(seg) = msg {
                dropped_payloads.push(seg.data.to_vec());
            }
        }
        assert!(decode_buf.is_empty(), "dropped write must end on a message boundary");
        assert_eq!(
            vec![vec![0x21; MAX_SESSION_MTU], vec![0x22; MAX_SESSION_MTU]],
            dropped_payloads,
            "dropped packet 1 must contain exactly datagrams 0x21 and 0x22"
        );

        let mut received = Vec::new();
        let mut buf = vec![0u8; MAX_SESSION_MTU + 64];
        loop {
            let n = receiver.read(&mut buf).await?;
            if n == 0 {
                break;
            }
            received.push(buf[..n].to_vec());
        }

        // Only the 2 datagrams in dropped packet 1 (0x21, 0x22) are lost; all 5 others survive
        // with their full payloads intact and in order.
        let expected: Vec<Vec<u8>> = [0x20u8, 0x23, 0x24, 0x25, 0x26]
            .into_iter()
            .map(|b| vec![b; MAX_SESSION_MTU])
            .collect();
        assert_eq!(expected, received);
        Ok(())
    }

    #[tokio::test]
    async fn isolated_datagram_flushes_immediately_without_waiting_for_pack_delay() -> anyhow::Result<()> {
        // Deliberately huge pack_delay: an isolated datagram must NOT wait for it.
        let (mut sender, mut receiver, writes) = make_packed_socket_pair(Duration::from_secs(5), vec![], None, None)?;

        let payload = b"single interactive ping datagram";
        sender.write_all(payload).await?;

        let mut buf = vec![0u8; 128];
        let n = tokio::time::timeout(Duration::from_millis(100), receiver.read(&mut buf)).await??;
        assert_eq!(payload.as_slice(), &buf[..n]);
        assert_eq!(1, writes.lock().len());
        Ok(())
    }

    #[tokio::test]
    async fn nearly_full_packed_packet_flushes_immediately_without_timer_wait() -> anyhow::Result<()> {
        // Huge pack_delay: once 2 × 1,462 B = 2,924 B are buffered (314 B left < 1,462 B),
        // the packer must flush immediately instead of waiting 5 seconds.
        let (mut sender, mut receiver, writes) = make_packed_socket_pair(Duration::from_secs(5), vec![], None, None)?;

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
        assert_eq!(2, writes.lock().len());
        Ok(())
    }

    #[tokio::test]
    async fn explicit_flush_blocks_on_stalled_transport_and_completes_once_drained() -> anyhow::Result<()> {
        use tokio::io::AsyncReadExt as _;
        use tokio_util::compat::TokioAsyncReadCompatExt;

        let (transport, mut remote) = tokio::io::duplex(1);
        let mut sender = SessionSocket::<WIRE_MTU, _>::stateless_builder(
            "duplex-flush",
            transport.compat(),
            SessionSocketConfig {
                frame_size: MAX_SESSION_MTU,
                flush_immediately: true,
                datagram: true,
                pack_delay: Some(Duration::from_secs(5)),
                ..Default::default()
            },
        )
        .build()?;

        // Enqueues the segment into the packer task, which stalls inside `writer.write_all` on
        // the 1-byte duplex because `remote` is open and not draining yet.
        sender.write_all(b"must-wait-for-transport-flush").await?;

        let mut flush_fut = Box::pin(sender.flush());
        assert!(
            tokio::time::timeout(Duration::from_millis(50), &mut flush_fut)
                .await
                .is_err(),
            "explicit flush() must remain Pending while the underlying transport cannot drain"
        );

        // Drain the remote end concurrently so `flush()` can finish cleanly.
        let drain = tokio::spawn(async move {
            let mut drained = Vec::new();
            let mut tmp = [0u8; 64];
            let mut total = 0usize;
            while total < b"must-wait-for-transport-flush".len() + 10 {
                let n = remote.read(&mut tmp).await?;
                if n == 0 {
                    break;
                }
                drained.extend_from_slice(&tmp[..n]);
                total += n;
            }
            Ok::<_, std::io::Error>(drained)
        });

        tokio::time::timeout(Duration::from_millis(200), flush_fut)
            .await
            .context("flush() must complete once the transport drains")??;
        let drained = drain.await??;
        assert!(!drained.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn expired_pack_timer_takes_priority_over_queued_datagrams() -> anyhow::Result<()> {
        struct RecordingWriter {
            writes: Arc<Mutex<Vec<Bytes>>>,
        }

        impl AsyncWrite for RecordingWriter {
            fn poll_write(self: Pin<&mut Self>, _cx: &mut Context<'_>, buf: &[u8]) -> Poll<std::io::Result<usize>> {
                self.writes.lock().push(Bytes::copy_from_slice(buf));
                Poll::Ready(Ok(buf.len()))
            }

            fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
                Poll::Ready(Ok(()))
            }

            fn poll_close(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
                Poll::Ready(Ok(()))
            }
        }

        let writes = Arc::new(Mutex::new(Vec::new()));
        let (mut tx, rx) = mpsc::channel(PACKER_CHANNEL_CAPACITY);
        let (done_tx, _done_rx) = oneshot::channel();
        let error_slot = Arc::new(Mutex::new(None));
        let pack_delay = Duration::from_millis(25);

        let mut task = Box::pin(run_packer::<WIRE_MTU, _>(
            rx,
            RecordingWriter { writes: writes.clone() },
            pack_delay,
            None,
            error_slot,
            done_tx,
        ));
        let mut cx = Context::from_waker(futures::task::noop_waker_ref());

        // Msg 0 is isolated and flushes immediately on the first poll.
        tx.try_send(PackerCommand::Message(Bytes::from(vec![0xA0; 64])))
            .context("enqueue msg 0")?;
        assert!(task.as_mut().poll(&mut cx).is_pending());
        assert_eq!(1, writes.lock().len(), "msg 0 must flush immediately");

        // Msg 1 arrives within the burst window -> buffered and arms the 25 ms FlushTimer.
        tx.try_send(PackerCommand::Message(Bytes::from(vec![0xA1; 64])))
            .context("enqueue msg 1")?;
        assert!(task.as_mut().poll(&mut cx).is_pending());
        assert_eq!(1, writes.lock().len(), "msg 1 must wait in the pack buffer");

        // Let the timer expire BEFORE polling again, then queue msg 2 so both the expired timer
        // and the newly queued message are simultaneously ready on the next poll.
        tokio::time::sleep(Duration::from_millis(45)).await;
        tx.try_send(PackerCommand::Message(Bytes::from(vec![0xA2; 64])))
            .context("enqueue msg 2")?;

        assert!(task.as_mut().poll(&mut cx).is_pending());

        drop(tx);
        task.await;

        let recorded = writes.lock();
        assert_eq!(
            vec![64, 64, 64],
            recorded.iter().map(Bytes::len).collect::<Vec<_>>(),
            "expired timer must flush msg 1 before msg 2 is appended"
        );
        assert!(recorded[1].iter().all(|&b| b == 0xA1));
        assert!(recorded[2].iter().all(|&b| b == 0xA2));
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
        let mut sender = SessionSocket::<WIRE_MTU, _>::stateless_builder(
            "stalled",
            StalledWriter { _drop_signal: drop_tx },
            datagram_socket_cfg(Some(Duration::from_millis(10))),
        )
        .build()?;

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
        // Long pack_delay so msg 1 is still buffered in the packer when `sender` drops.
        let (mut sender, mut receiver, _writes) = make_packed_socket_pair(Duration::from_secs(5), vec![], None, None)?;

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
        let (mut sender, mut receiver, writes) =
            make_packed_socket_pair(Duration::from_millis(40), vec![], None, None)?;

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
        assert_eq!(4, writes.lock().len());
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

        let mut sender = SessionSocket::<WIRE_MTU, _>::stateless_builder(
            "blocked",
            BlockedWriter,
            datagram_socket_cfg(Some(Duration::from_millis(10))),
        )
        .build()?;

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
    async fn writer_error_is_sticky_and_preserves_io_error_kind() -> anyhow::Result<()> {
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

        // First send triggers the failing write and flushes; wait for the error to propagate.
        match sink.send(SessionMessage::<WIRE_MTU>::Segment(seg.clone())).await {
            Err(SessionError::IoError(io_err)) => assert_eq!(std::io::ErrorKind::ConnectionReset, io_err.kind()),
            other => anyhow::bail!("expected SessionError::IoError(ConnectionReset), got {other:?}"),
        }

        // Subsequent send, flush, and close calls must all repeat the same ConnectionReset error.
        for _ in 0..2 {
            match sink.send(SessionMessage::<WIRE_MTU>::Segment(seg.clone())).await {
                Err(SessionError::IoError(io_err)) => assert_eq!(std::io::ErrorKind::ConnectionReset, io_err.kind()),
                other => anyhow::bail!("expected sticky ConnectionReset on send, got {other:?}"),
            }
            match sink.flush().await {
                Err(SessionError::IoError(io_err)) => assert_eq!(std::io::ErrorKind::ConnectionReset, io_err.kind()),
                other => anyhow::bail!("expected sticky ConnectionReset on flush, got {other:?}"),
            }
        }
        match sink.close().await {
            Err(SessionError::IoError(io_err)) => assert_eq!(std::io::ErrorKind::ConnectionReset, io_err.kind()),
            other => anyhow::bail!("expected sticky ConnectionReset on close, got {other:?}"),
        }
        Ok(())
    }

    #[tokio::test]
    async fn poll_close_returns_error_when_packer_task_cancels_unexpectedly() -> anyhow::Result<()> {
        let (tx, _rx) = mpsc::channel(1);
        let (done_tx, done_rx) = oneshot::channel();
        let (drop_tx, _drop_rx) = oneshot::channel();

        // Simulate a background packer task that terminated/panicked without reporting a result.
        drop(done_tx);

        let mut sink = WholeMessagePackingSink::<WIRE_MTU> {
            tx: Some(tx),
            pending_frame: Vec::new(),
            pending_flush: None,
            done: Some(done_rx),
            drop_guard: Some(drop_tx),
            error: Arc::new(Mutex::new(None)),
            encode_buf: BytesMut::with_capacity(WIRE_MTU),
        };

        // Both the first and subsequent close() calls must report BrokenPipe (sticky error).
        for attempt in 0..2 {
            match sink.close().await {
                Err(SessionError::IoError(io_err)) => assert_eq!(
                    std::io::ErrorKind::BrokenPipe,
                    io_err.kind(),
                    "close attempt {attempt} must return BrokenPipe"
                ),
                other => anyhow::bail!(
                    "expected BrokenPipe on close attempt {attempt} when packer task cancels unexpectedly, got \
                     {other:?}"
                ),
            }
        }

        // By contrast, a cleanly completed sink must succeed on repeated close() calls.
        let (clean_pipe, _rx_pipe, _writes) = RecordingPacketPipe::pair(vec![]);
        let mut clean_sink =
            WholeMessagePackingSink::<WIRE_MTU>::new(clean_pipe, Duration::from_millis(5), None, tracing::Span::none());
        clean_sink.close().await?;
        clean_sink.close().await?;
        Ok(())
    }

    #[tokio::test]
    async fn pack_budget_dynamically_limits_packet_size() -> anyhow::Result<()> {
        use std::sync::atomic::{AtomicUsize, Ordering};

        let budget_bytes = Arc::new(AtomicUsize::new(2000));
        let budget_clone = budget_bytes.clone();

        let (mut sender, mut receiver, writes) = make_packed_socket_pair(
            Duration::from_millis(20),
            vec![],
            Some(PackPayloadBudget::new(move || budget_clone.load(Ordering::Relaxed))),
            None,
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
        assert_eq!(3, writes.lock().len(), "budget=2000 must send 1 full-MTU/pkt");

        // Raise budget to full WIRE_MTU (3,238 B): burst datagrams now pack 2/packet.
        budget_bytes.store(WIRE_MTU, Ordering::Relaxed);
        writes.lock().clear();
        for i in 0..4u8 {
            sender.write_all(&vec![0x50 + i; MAX_SESSION_MTU]).await?;
        }
        sender.close().await?;
        for _ in 0..4 {
            let _ = tokio::time::timeout(Duration::from_millis(200), receiver.read(&mut buf)).await??;
        }
        assert!(
            writes.lock().iter().any(|b| b.len() >= 2 * MAX_SESSION_MTU),
            "budget=WIRE_MTU must pack 2 full-MTU datagrams per packet: {:?}",
            writes.lock().iter().map(|b| b.len()).collect::<Vec<_>>()
        );
        Ok(())
    }

    fn test_segment_msg(frame_id: u32, payload: &[u8]) -> SessionMessage<WIRE_MTU> {
        SessionMessage::<WIRE_MTU>::Segment(crate::Segment {
            frame_id,
            seq_idx: 0,
            seq_flags: crate::protocol::SeqIndicator::new_with_flags(1, false),
            data: payload.to_vec().into_boxed_slice(),
        })
    }

    /// Regression test for cancelled `poll_flush` followed by `start_send`: dropping a pending
    /// flush future, accepting a new message, and calling `flush()` again must wait for the new
    /// message to reach the underlying writer instead of resolving on the stale flush ack.
    #[tokio::test]
    async fn cancelled_flush_followed_by_write_does_not_complete_before_new_write_reaches_writer() -> anyhow::Result<()>
    {
        struct GatedWriter {
            waker: Arc<Mutex<Option<std::task::Waker>>>,
            gate_open: Arc<std::sync::atomic::AtomicBool>,
            gate_after_writes: usize,
            writes: Arc<Mutex<Vec<Bytes>>>,
        }

        impl AsyncWrite for GatedWriter {
            fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<std::io::Result<usize>> {
                let this = self.get_mut();
                if this.writes.lock().len() >= this.gate_after_writes
                    && !this.gate_open.load(std::sync::atomic::Ordering::Acquire)
                {
                    *this.waker.lock() = Some(cx.waker().clone());
                    return Poll::Pending;
                }
                this.writes.lock().push(Bytes::copy_from_slice(buf));
                Poll::Ready(Ok(buf.len()))
            }

            fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
                Poll::Ready(Ok(()))
            }

            fn poll_close(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
                Poll::Ready(Ok(()))
            }
        }

        let waker_slot: Arc<Mutex<Option<std::task::Waker>>> = Arc::new(Mutex::new(None));
        let gate_open = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let writes = Arc::new(Mutex::new(Vec::new()));
        let writer = GatedWriter {
            waker: waker_slot.clone(),
            gate_open: gate_open.clone(),
            gate_after_writes: 1,
            writes: writes.clone(),
        };

        let mut sink =
            WholeMessagePackingSink::<WIRE_MTU>::new(writer, Duration::from_millis(20), None, tracing::Span::none());

        // Send a first small message and poll `flush()` once so `PackerCommand::Flush(ack1)` is
        // enqueued and `pending_flush` is populated, then cancel the flush future.
        sink.feed(test_segment_msg(1, b"first")).await?;
        {
            let mut flush1 = sink.flush();
            let waker = futures::task::noop_waker();
            let mut cx = Context::from_waker(&waker);
            let _ = Pin::new(&mut flush1).poll(&mut cx);
        }

        // Wait for the first write to land while the second write is gated.
        let deadline = Instant::now() + Duration::from_secs(1);
        while writes.lock().is_empty() && Instant::now() < deadline {
            tokio::time::sleep(Duration::from_millis(2)).await;
        }
        assert_eq!(1, writes.lock().len());

        // Accept a second message while the writer is blocked on write #2.
        sink.feed(test_segment_msg(2, &[0xAB; 128])).await?;

        // A new `flush()` must remain pending until the writer unblocks and accepts msg2.
        let mut flush2 = Box::pin(sink.flush());
        assert!(
            tokio::time::timeout(Duration::from_millis(40), &mut flush2)
                .await
                .is_err(),
            "flush must not complete on a stale ack before msg2 reaches the writer"
        );
        assert_eq!(1, writes.lock().len());

        // Unblock the writer; now flush2 must complete and msg2 must have been written.
        gate_open.store(true, std::sync::atomic::Ordering::Release);
        if let Some(waker) = waker_slot.lock().take() {
            waker.wake();
        }
        tokio::time::timeout(Duration::from_secs(1), flush2).await??;
        assert_eq!(2, writes.lock().len());
        Ok(())
    }

    /// Regression test for overflow-write stalling past `pack_delay`: if flushing the overflow
    /// packet stalls longer than `pack_delay`, the follow-up message already buffered in
    /// `pack_buf` must be flushed immediately once the writer recovers instead of waiting another
    /// full `pack_delay`.
    #[tokio::test]
    async fn stalled_overflow_write_past_pack_delay_flushes_buffered_message_immediately() -> anyhow::Result<()> {
        struct DelayedNthWrite {
            delay_on_write_index: usize,
            delay: Duration,
            writes: Arc<Mutex<Vec<(Instant, usize)>>>,
            sleep: Option<Pin<Box<tokio::time::Sleep>>>,
        }

        impl AsyncWrite for DelayedNthWrite {
            fn poll_write(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<std::io::Result<usize>> {
                let this = self.as_mut().get_mut();
                if this.writes.lock().len() == this.delay_on_write_index {
                    let sleep = this
                        .sleep
                        .get_or_insert_with(|| Box::pin(tokio::time::sleep(this.delay)));
                    futures::ready!(sleep.as_mut().poll(cx));
                    this.sleep = None;
                }
                this.writes.lock().push((Instant::now(), buf.len()));
                Poll::Ready(Ok(buf.len()))
            }

            fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
                Poll::Ready(Ok(()))
            }

            fn poll_close(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
                Poll::Ready(Ok(()))
            }
        }

        let pack_delay = Duration::from_millis(80);
        let budget = PackPayloadBudget::new(|| 1_100);
        assert!(format!("{budget:?}").contains("PackPayloadBudget"));

        let overhead = {
            let mut buf = BytesMut::new();
            SessionCodec::<WIRE_MTU>.encode(test_segment_msg(1, &[]), &mut buf)?;
            buf.len()
        };
        let payload_300 = vec![0x33; 300 - overhead];
        let payload_510 = vec![0x55; 510 - overhead];

        let writes = Arc::new(Mutex::new(Vec::new()));
        let writer = DelayedNthWrite {
            // Write #0 = isolated 300 B; Write #1 = overflow packet (300 + 300 = 600 B) stalled
            // 120 ms (> 80 ms pack_delay); Write #2 = buffered 510 B message (remaining 590 B >=
            // 510 B so `outcome.immediate` is `None`), flushed immediately once Write #1 finishes
            // because its `buffered_at + pack_delay` deadline has already passed.
            delay_on_write_index: 1,
            delay: Duration::from_millis(120),
            writes: writes.clone(),
            sleep: None,
        };
        let mut sink =
            WholeMessagePackingSink::<WIRE_MTU>::new(writer, pack_delay, Some(budget), tracing::Span::none());

        sink.feed(test_segment_msg(1, &payload_300)).await?;
        sink.feed(test_segment_msg(2, &payload_300)).await?;
        sink.feed(test_segment_msg(3, &payload_300)).await?;
        sink.feed(test_segment_msg(4, &payload_510)).await?;

        let deadline = Instant::now() + Duration::from_secs(2);
        while writes.lock().len() < 3 && Instant::now() < deadline {
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
        let recorded = writes.lock().clone();
        let lens: Vec<usize> = recorded.iter().map(|(_, len)| *len).collect();
        assert_eq!(vec![300, 600, 510], lens);
        let gap_after_overflow = recorded[2].0.saturating_duration_since(recorded[1].0);
        assert!(
            gap_after_overflow < pack_delay / 2,
            "buffered 510 B message should flush immediately after stalled overflow write, not wait another \
             {pack_delay:?} (gap={gap_after_overflow:?})"
        );
        sink.close().await?;
        Ok(())
    }

    #[tokio::test]
    async fn sink_reports_closed_and_writer_errors_across_all_methods() -> anyhow::Result<()> {
        struct FailingWriter {
            fail_on_write: bool,
            fail_on_flush: bool,
            fail_on_close: bool,
        }

        impl AsyncWrite for FailingWriter {
            fn poll_write(self: Pin<&mut Self>, _cx: &mut Context<'_>, buf: &[u8]) -> Poll<std::io::Result<usize>> {
                if self.fail_on_write {
                    Poll::Ready(Err(std::io::Error::new(
                        std::io::ErrorKind::ConnectionReset,
                        "write boom",
                    )))
                } else {
                    Poll::Ready(Ok(buf.len()))
                }
            }

            fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
                if self.fail_on_flush {
                    Poll::Ready(Err(std::io::Error::new(std::io::ErrorKind::WriteZero, "flush boom")))
                } else {
                    Poll::Ready(Ok(()))
                }
            }

            fn poll_close(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
                if self.fail_on_close {
                    Poll::Ready(Err(std::io::Error::other("close boom")))
                } else {
                    Poll::Ready(Ok(()))
                }
            }
        }

        // 1. After clean `close()`, subsequent `poll_ready`, `start_send`, `poll_flush` return BrokenPipe and a second
        //    `close()` succeeds idempotently.
        let (a, _b, _writes) = RecordingPacketPipe::pair(vec![]);
        let mut closed_sink =
            WholeMessagePackingSink::<WIRE_MTU>::new(a, Duration::from_millis(10), None, tracing::Span::none());
        closed_sink.close().await?;
        closed_sink.close().await?;
        assert!(matches!(
            closed_sink.send(test_segment_msg(1, b"after-close")).await,
            Err(SessionError::IoError(e)) if e.kind() == std::io::ErrorKind::BrokenPipe
        ));
        assert!(matches!(
            closed_sink.flush().await,
            Err(SessionError::IoError(e)) if e.kind() == std::io::ErrorKind::BrokenPipe
        ));
        assert!(matches!(
            Pin::new(&mut closed_sink).start_send(test_segment_msg(2, b"start-send-after-close")),
            Err(SessionError::IoError(e)) if e.kind() == std::io::ErrorKind::BrokenPipe
        ));

        // 2. Writer flush error surfaces on `flush()`, then persists on `poll_ready`, `start_send`, and `close()`.
        let mut flush_err_sink = WholeMessagePackingSink::<WIRE_MTU>::new(
            FailingWriter {
                fail_on_write: false,
                fail_on_flush: true,
                fail_on_close: false,
            },
            Duration::from_millis(50),
            None,
            tracing::Span::none(),
        );
        assert!(matches!(
            flush_err_sink.flush().await,
            Err(SessionError::IoError(e)) if e.kind() == std::io::ErrorKind::WriteZero
        ));
        assert!(matches!(
            Pin::new(&mut flush_err_sink).start_send(test_segment_msg(3, b"after-err")),
            Err(SessionError::IoError(e)) if e.kind() == std::io::ErrorKind::WriteZero
        ));
        assert!(matches!(
            flush_err_sink.flush().await,
            Err(SessionError::IoError(e)) if e.kind() == std::io::ErrorKind::WriteZero
        ));
        assert!(matches!(
            flush_err_sink.close().await,
            Err(SessionError::IoError(e)) if e.kind() == std::io::ErrorKind::WriteZero
        ));
        // Second close after error still surfaces stored error.
        assert!(matches!(
            flush_err_sink.close().await,
            Err(SessionError::IoError(e)) if e.kind() == std::io::ErrorKind::WriteZero
        ));

        // 3. Writer write error during timer-expired flush: write #0 (isolated) succeeds, then msg #2 stays buffered
        //    until `pack_delay` expires and write #1 fails with ConnectionReset. Verify the worker records the error
        //    before explicit `flush()`.
        struct FailAfterFirstWrite {
            attempts: Arc<std::sync::atomic::AtomicUsize>,
        }

        impl AsyncWrite for FailAfterFirstWrite {
            fn poll_write(self: Pin<&mut Self>, _cx: &mut Context<'_>, buf: &[u8]) -> Poll<std::io::Result<usize>> {
                let n = self.attempts.fetch_add(1, std::sync::atomic::Ordering::AcqRel);
                if n == 0 {
                    Poll::Ready(Ok(buf.len()))
                } else {
                    Poll::Ready(Err(std::io::Error::new(
                        std::io::ErrorKind::ConnectionReset,
                        "timer write boom",
                    )))
                }
            }

            fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
                Poll::Ready(Ok(()))
            }

            fn poll_close(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
                Poll::Ready(Ok(()))
            }
        }

        let attempts = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let mut timer_err_sink = WholeMessagePackingSink::<WIRE_MTU>::new(
            FailAfterFirstWrite {
                attempts: attempts.clone(),
            },
            Duration::from_millis(20),
            None,
            tracing::Span::none(),
        );
        timer_err_sink.feed(test_segment_msg(4, b"first-ok")).await?;
        timer_err_sink.feed(test_segment_msg(5, b"timer-boom")).await?;

        let deadline = Instant::now() + Duration::from_secs(1);
        while timer_err_sink.stored_error().is_none() && Instant::now() < deadline {
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
        assert_eq!(2, attempts.load(std::sync::atomic::Ordering::Acquire));
        assert!(matches!(
            timer_err_sink.stored_error(),
            Some(SessionError::IoError(e)) if e.kind() == std::io::ErrorKind::ConnectionReset
        ));
        assert!(matches!(
            timer_err_sink.flush().await,
            Err(SessionError::IoError(e)) if e.kind() == std::io::ErrorKind::ConnectionReset
        ));
        Ok(())
    }

    #[tokio::test]
    async fn sink_regroups_buffered_messages_when_budget_shrinks_before_timer_flush_or_close() -> anyhow::Result<()> {
        use std::sync::atomic::{AtomicUsize, Ordering};

        // Case 1: explicit flush after budget shrinks while two 300 B messages are buffered.
        for emit_kind in ["flush", "timer", "close"] {
            let (sender_pipe, _receiver_pipe, writes) = RecordingPacketPipe::pair(vec![]);
            let budget_bytes = Arc::new(AtomicUsize::new(2_000));
            let budget_clone = budget_bytes.clone();

            let pack_delay = if emit_kind == "timer" {
                Duration::from_millis(25)
            } else {
                Duration::from_secs(5)
            };
            let mut sink = WholeMessagePackingSink::<WIRE_MTU>::new(
                sender_pipe,
                pack_delay,
                Some(PackPayloadBudget::new(move || budget_clone.load(Ordering::Acquire))),
                tracing::Span::none(),
            );

            let msg_300 = |frame_id: u32, byte: u8| {
                // Encoded segment = 300 B total (`SEGMENT_OVERHEAD` bytes of framing + payload).
                let target_payload = 300usize - SessionMessage::<WIRE_MTU>::SEGMENT_OVERHEAD;
                SessionMessage::<WIRE_MTU>::Segment(crate::Segment {
                    frame_id,
                    seq_idx: 0,
                    seq_flags: crate::SeqIndicator::new(1),
                    data: vec![byte; target_payload].into_boxed_slice(),
                })
            };

            // Seed burst state with an isolated 300 B message (flushes immediately as write #0).
            sink.feed(msg_300(1, 0x01)).await?;
            // Buffer two 300 B messages under the 2000 B budget (total 600 B buffered).
            sink.feed(msg_300(2, 0x02)).await?;
            sink.feed(msg_300(3, 0x03)).await?;

            // Wait until the isolated write #0 lands so messages #2 and #3 are buffered in the worker.
            let deadline = Instant::now() + Duration::from_secs(1);
            while writes.lock().is_empty() && Instant::now() < deadline {
                tokio::time::sleep(Duration::from_millis(2)).await;
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
            assert_eq!(
                1,
                writes.lock().len(),
                "{emit_kind}: only isolated write #0 should be emitted"
            );

            // Shrink budget to 473 B before emission.
            budget_bytes.store(473, Ordering::Release);

            match emit_kind {
                "flush" => {
                    sink.flush().await?;
                }
                "timer" => {
                    let deadline = Instant::now() + Duration::from_secs(1);
                    while writes.lock().len() < 3 && Instant::now() < deadline {
                        tokio::time::sleep(Duration::from_millis(5)).await;
                    }
                }
                "close" => {
                    sink.close().await?;
                }
                _ => unreachable!(),
            }

            let lens: Vec<usize> = writes.lock().iter().map(Bytes::len).collect();
            assert_eq!(
                vec![300, 300, 300],
                lens,
                "{emit_kind}: shrinking budget to 473 B must split buffered 600 B batch into two 300 B writes"
            );
        }
        Ok(())
    }
}
