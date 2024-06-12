use crossbeam_queue::ArrayQueue;
use crossbeam_skiplist::SkipMap;
use dashmap::mapref::entry::Entry;
use dashmap::DashMap;
use futures::{AsyncRead, AsyncWrite, AsyncWriteExt, FutureExt, StreamExt, TryStreamExt};
use pin_project::pin_project;
use smart_default::SmartDefault;
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};
use tracing::{debug, error, warn};

use crate::frame::{segment, FrameId, FrameReassembler, SegmentId};
use crate::prelude::Segment;
use crate::session::protocol::{FrameAcknowledgements, SegmentRequest, SessionMessage};
use crate::session::utils::{RetryLog, RetryResult};

/// Configuration of session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, SmartDefault)]
pub struct SessionConfig {
    /// Maximum number of buffered segments.
    #[default = 50_000]
    pub max_buffered_segments: usize,
    /// Size of the buffer for acknowledged frame IDs.
    /// If set to 0, no frame acknowledgement will be sent.
    #[default = 1024]
    pub acknowledged_frames_buffer: usize,
    /// Incomplete frames will be discarded after being in the reassembler
    /// this long.
    #[default(Duration::from_secs(30))]
    pub frame_expiration_age: Duration,
    /// Payload size available for the session sub-protocol.
    #[default = 466]
    pub mtu: usize,
    /// If a frame is incomplete, retries will be made with exponential
    /// backoff starting at this initial retry timeout (RTO).
    /// Retries will be made until `frame_expiration_age` is reached.
    #[default(Duration::from_secs(1))]
    pub rto_base: Duration,
}

/// Contains the cloneable state of the session bound to a [`SessionSocket`].
///
/// It implements the entire [`SessionMessage`] state machine and
/// performs the frame reassembly and sequencing.
/// The underlying transport operations are bound to `T`
///
/// The `SessionState` cannot be created directly, it must always be created via [`SessionSocket`] and
/// then retrieved via [`SessionSocket::state`].
#[derive(Debug)]
pub struct SessionState<T> {
    pub(crate) transport: T,
    lookbehind: Arc<SkipMap<SegmentId, Segment>>,
    acknowledged: Arc<ArrayQueue<FrameId>>,
    incoming_frame_retries: Arc<DashMap<FrameId, RetryLog>>,
    outgoing_frame_id: Arc<AtomicU32>,
    frame_reassembler: Arc<FrameReassembler>,
    cfg: SessionConfig,
}

impl<T: AsyncWrite + Unpin> SessionState<T> {
    async fn consume_segment(&mut self, segment: Segment) {
        let id = segment.id();
        match self.frame_reassembler.push_segment(segment) {
            Ok(_) => {
                debug!("RECEIVED: segment {id:?}");
                // Create a retry record for this segment
                match self.incoming_frame_retries.entry(id.0) {
                    Entry::Vacant(v) => {
                        v.insert(RetryLog::new(Instant::now()));
                    },
                    _ => {}
                }
            }
            Err(e) => warn!("segment {id:?} not pushed: {e}")
        }
    }

    async fn retransmit_segments(&mut self, request: SegmentRequest) -> crate::errors::Result<()> {
        debug!("RECEIVED: request for {} segments", request.len());
        let mut count = 0;
        for segment_id in request {
            if let Some(segment) = self.lookbehind.get(&segment_id) {
                let msg = SessionMessage::Segment(segment.value().clone());
                debug!("SENDING: retransmitted segment: {:?}", SegmentId::from(segment.value()));
                self.transport.write_all(&msg.into_encoded()).await?;
                count += 1;
            } else {
                error!("segment {segment_id:?} not in lookbehind buffer anymore");
            }
        }
        self.transport.flush().await?;
        debug!("retransmitted {count} requested segments");
        Ok(())
    }

    async fn acknowledged_frames(&mut self, acked: FrameAcknowledgements) {
        debug!("RECEIVED: acknowledgement of {} frames", acked.len());
        for frame_id in acked {
            for seg in self.lookbehind.iter().filter(|s| frame_id == s.key().0) {
                seg.remove();
            }
        }
    }

    /// Should be called by the underlying transport when raw packet data are received.
    /// The `data` argument must contain a valid [`SessionMessage`], otherwise the method throws an error.
    pub async fn received_message(&mut self, data: &[u8]) -> crate::errors::Result<()> {
        match SessionMessage::try_from(data)? {
            SessionMessage::Segment(s) => self.consume_segment(s).await,
            SessionMessage::Request(r) => self.retransmit_segments(r).await?,
            SessionMessage::Acknowledge(f) => self.acknowledged_frames(f).await,
        }
        Ok(())
    }

    /// Sends a requests for missing segments in incomplete frames.
    /// One [request](SessionMessage::Request) message is sent per incomplete frame. The message contains
    /// the segment indices missing from that frame. The `max_requests` argument can provide a maximum
    /// number of request messages sent by this call. If `max_requests` is `None`, request messages
    /// are set for all incomplete frames.
    /// Returns the number of sent request messages.
    pub async fn request_missing_segments(&mut self, max_requests: Option<usize>) -> crate::errors::Result<usize> {
        self.frame_reassembler.evict()?;

        if max_requests == Some(0) {
            return Ok(0); // don't even bother
        }

        let tracked_incomplete = self.frame_reassembler.incomplete_frames();
        debug!("tracking {} incomplete frames", tracked_incomplete.len());

        // Filter the frames which we are allowed to retry now
        let mut to_retry = Vec::with_capacity(tracked_incomplete.len());
        let now = Instant::now();
        for info in tracked_incomplete {
            match self.incoming_frame_retries.entry(info.frame_id) {
                Entry::Occupied(e) => {
                    // Check if we can retry this frame now
                    let rto_check = e.get().check(now, self.cfg.rto_base, self.cfg.frame_expiration_age);
                    match rto_check {
                        RetryResult::RetryNow(next_rto) => {
                            // Retry this frame and plan ahead the time of the next retry
                            e.replace_entry(next_rto);
                            to_retry.push(info);
                        }
                        RetryResult::Expired => {
                            warn!("frame {} is already expired and will be evicted", info.frame_id);
                            e.remove();
                        }
                        _ => {}
                    }
                }
                Entry::Vacant(v) => {
                    // Shouldn't happen, but retry the frame anyway
                    v.insert(RetryLog::new(now));
                    to_retry.push(info);
                }
            }
        }

        let mut sent = 0;
        for chunk in to_retry
            .chunks(SegmentRequest::MAX_ENTRIES)
            .take(max_requests.unwrap_or(usize::MAX))
            .map(|chunk| SegmentRequest::from_iter(chunk.iter().cloned()))
        {
            debug!("SENDING: retransmission request for segments {chunk:?}");

            let req = SessionMessage::Request(chunk);
            self.transport.write_all(&req.into_encoded()).await?;
            sent += 1;
        }
        self.transport.flush().await?;

        debug!("RETRANSMISSION BATCH COMPLETE: sent {sent} re-send requests");
        Ok(sent)
    }

    /// Sends [acknowledgement](SessionMessage::Acknowledge) messages containing frames IDs
    /// of all frames that were successfully processed.
    /// If [`acknowledged_frames_buffer`](SessionConfig) was set to `0` during the construction,
    /// this method will do nothing and return `0`.
    /// Otherwise, it returns the number of acknowledged frames.
    /// If `acknowledged_frames_buffer` is non-zero, the buffer behaves like a ring buffer,
    /// which means if this method is not called sufficiently often, the oldest acknowledged
    /// frame IDs will be discarded.
    /// Single [message](SessionMessage::Acknowledge) can accommodate up to [`FrameAcknowledgements::MAX_ACK_FRAMES`] frame IDs, so
    /// this method sends as many messages as needed, or at most `max_message` if was given.
    pub async fn acknowledge_segments(&mut self, max_messages: Option<usize>) -> crate::errors::Result<usize> {
        let mut len = 0;
        let mut msgs = 0;
        while !self.acknowledged.is_empty() {
            let mut ack_frames = FrameAcknowledgements::default();

            if max_messages.map(|max| msgs < max).unwrap_or(true) {
                while !ack_frames.is_full() && !self.acknowledged.is_empty() {
                    if let Some(ack_id) = self.acknowledged.pop() {
                        ack_frames.push(ack_id);
                        len += 1;
                    }
                }

                debug!("SENDING: acknowledgement of {} frames", ack_frames.len());
                self.transport
                    .write_all(&SessionMessage::Acknowledge(ack_frames).into_encoded())
                    .await?;
                msgs += 1;
            } else {
                break; // break out if we sent max allowed
            }
        }
        self.transport.flush().await?;

        debug!("ACKNOWLEDGEMENT BATCH COMPLETE: sent {len} acknowledgements in {msgs} messages");
        Ok(len)
    }

    /// Segments the `data` and sends them as (possibly multiple) [`SessionMessage::Segment`].
    /// Therefore, this method sends as many messages as needed after the data was segmented.
    /// Each segment is inserted into the lookbehind ring buffer for possible retransmissions.
    ///
    /// The size of the lookbehind ring buffer is given by the [`max_buffered_segments`](SessionConfig)
    /// given during the construction. It needs to accommodate as many segments as
    /// is the expected underlying transport bandwidth (segment/sec) to guarantee the retransmission
    /// can still happen within some time window.
    pub async fn send_frame_data(&mut self, data: &[u8]) -> crate::errors::Result<()> {
        let segments = segment(
            data,
            self.cfg.mtu - SessionMessage::HEADER_SIZE,
            self.outgoing_frame_id.fetch_add(1, Ordering::SeqCst),
        );

        for segment in segments {
            self.lookbehind.insert((&segment).into(), segment.clone());

            let msg = SessionMessage::Segment(segment.clone());
            debug!("SENDING: segment {:?}", SegmentId::from(&segment));
            self.transport.write_all(&msg.into_encoded()).await?;

            // TODO: prevent stalling here
            while self.lookbehind.len() > self.cfg.max_buffered_segments {
                self.lookbehind.pop_front();
            }
        }

        self.transport.flush().await?;

        Ok(())
    }
}

impl<T: AsyncWrite + Unpin + Clone> Clone for SessionState<T> {
    fn clone(&self) -> Self {
        Self {
            transport: self.transport.clone(),
            lookbehind: self.lookbehind.clone(),
            acknowledged: self.acknowledged.clone(),
            incoming_frame_retries: self.incoming_frame_retries.clone(),
            outgoing_frame_id: self.outgoing_frame_id.clone(),
            frame_reassembler: self.frame_reassembler.clone(),
            cfg: self.cfg,
        }
    }
}

/// Represents a socket for a session between two nodes bound by the
/// underlying [`NetworkTransport`].
///
/// It also implements [`AsyncRead`] and [`AsyncWrite`] so that it can
/// be used on top of the usual transport stack.
#[pin_project]
pub struct SessionSocket<T> {
    state: SessionState<T>,
    #[pin]
    egress: Box<dyn AsyncRead + Send + Unpin>,
}

impl<T: AsyncWrite + Send + Unpin> SessionSocket<T> {
    /// Create a new socket over the given `transport` that binds the communicating parties.
    pub fn new(transport: T, cfg: SessionConfig) -> Self {
        let (reassembler, egress) = FrameReassembler::new(cfg.frame_expiration_age);

        let acknowledged = Arc::new(ArrayQueue::new(cfg.acknowledged_frames_buffer.max(1)));
        let is_acknowledging = (cfg.acknowledged_frames_buffer > 0).then_some(acknowledged.clone());
        let incoming_frame_retries = Arc::new(DashMap::new());

        let incoming_frame_retries_clone = incoming_frame_retries.clone();
        let egress = Box::new(
            egress
                .inspect(move |frame| {
                    debug!("emit frame {}", frame.frame_id);
                    // The frame has been completed, so remove its retry record
                    incoming_frame_retries_clone.remove(&frame.frame_id);
                    if let Some(ack_buffer) = &is_acknowledging {
                        // Acts as a ring buffer, so if the buffer is full, any unsent acknowledgements
                        // will be discarded.
                        ack_buffer.force_push(frame.frame_id);
                    }
                })
                .map(Ok)
                .into_async_read(),
        );

        let state = SessionState {
            transport,
            acknowledged,
            cfg,
            incoming_frame_retries,
            lookbehind: Arc::new(SkipMap::new()),
            outgoing_frame_id: Arc::new(AtomicU32::new(1)),
            frame_reassembler: Arc::new(reassembler),
        };

        Self { state, egress }
    }

    /// Gets the [state](SessionState) of this socket.
    pub fn state(&self) -> &SessionState<T> {
        &self.state
    }

    /// Gets the mutable [state](SessionState) of this socket.
    pub fn state_mut(&mut self) -> &mut SessionState<T> {
        &mut self.state
    }
}

impl<T: AsyncWrite + Unpin + Send + Sync> AsyncWrite for SessionSocket<T> {
    fn poll_write(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<std::io::Result<usize>> {
        let mut socket_future = self.state.send_frame_data(buf).boxed();
        match Pin::new(&mut socket_future).poll(cx) {
            Poll::Ready(Ok(())) => Poll::Ready(Ok(buf.len())),
            Poll::Ready(Err(e)) => Poll::Ready(Err(std::io::Error::new(std::io::ErrorKind::Other, e))),
            Poll::Pending => Poll::Pending,
        }
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        // Only flush the underlying transport
        let mut flush_future = self.state.transport.flush().boxed();
        match Pin::new(&mut flush_future).poll(cx) {
            Poll::Ready(Ok(())) => Poll::Ready(Ok(())),
            Poll::Ready(Err(e)) => Poll::Ready(Err(std::io::Error::new(std::io::ErrorKind::Other, e))),
            Poll::Pending => Poll::Pending,
        }
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        // Close the reassembler and the underlying transport
        self.state.frame_reassembler.close();
        let mut close_future = self.state.transport.close().boxed();
        match Pin::new(&mut close_future).poll(cx) {
            Poll::Ready(Ok(())) => Poll::Ready(Ok(())),
            Poll::Ready(Err(e)) => Poll::Ready(Err(std::io::Error::new(std::io::ErrorKind::Other, e))),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl<T: AsyncWrite + Unpin + Send + Sync> AsyncRead for SessionSocket<T> {
    fn poll_read(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut [u8]) -> Poll<std::io::Result<usize>> {
        self.project().egress.poll_read(cx, buf)
    }
}

/// Implementations of Tokio's AsyncRead and AsyncWrite for compatibility
#[cfg(feature = "runtime-tokio")]
pub mod tokio_compat {
    use std::pin::Pin;
    use std::task::{Context, Poll};
    use tokio::io::{AsyncRead, AsyncWrite};

    use crate::session::state::SessionSocket;

    impl<T: futures::AsyncWrite + Unpin + Send + Sync> AsyncWrite for SessionSocket<T> {
        fn poll_write(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<std::io::Result<usize>> {
            futures::io::AsyncWrite::poll_write(self, cx, buf)
        }

        fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
            futures::io::AsyncWrite::poll_flush(self, cx)
        }

        fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
            futures::io::AsyncWrite::poll_close(self, cx)
        }
    }

    impl<T: futures::AsyncWrite + Unpin + Send + Sync> AsyncRead for SessionSocket<T> {
        fn poll_read(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut [u8]) -> Poll<std::io::Result<usize>> {
            futures::io::AsyncRead::poll_read(self, cx, buf)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::channel::mpsc::UnboundedSender;
    use futures::future::Either;
    use futures::io::{AsyncReadExt, AsyncWriteExt};
    use futures::pin_mut;
    use lazy_static::lazy_static;
    use rand::{Rng, SeedableRng};
    use std::fmt::{Debug, Formatter};
    use std::iter::Extend;
    use std::sync::OnceLock;
    use test_log::test;
    use tracing::{info, warn};

    lazy_static! {
        static ref RNG_SEED: [u8; 32] = hopr_crypto_random::random_bytes();
        //static ref RNG_SEED: [u8; 32] = hex_literal::hex!("b5da488125eccf36276b6d9c7f04fb0014feebdd050dc882536102bce7af5f4b");
    }

    #[derive(Debug, Clone, Default)]
    pub struct FaultyNetworkConfig {
        pub fault_mod: usize,
        pub mixing_factor: usize,
    }

    pub struct FaultyNetwork {
        sender: UnboundedSender<Box<[u8]>>,
        counterparty: Arc<OnceLock<SessionState<Self>>>,
        cfg: FaultyNetworkConfig,
        msg_counter: u128,
    }

    impl Debug for FaultyNetwork {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "network with {:?}", &self.counterparty)
        }
    }

    impl Clone for FaultyNetwork {
        fn clone(&self) -> Self {
            Self {
                sender: self.sender.clone(),
                counterparty: self.counterparty.clone(),
                cfg: self.cfg.clone(),
                msg_counter: self.msg_counter,
            }
        }
    }

    impl AsyncWrite for FaultyNetwork {
        fn poll_write(mut self: Pin<&mut Self>, _cx: &mut Context<'_>, buf: &[u8]) -> Poll<std::io::Result<usize>> {
            self.send_to_counterparty(buf).unwrap();
            Poll::Ready(Ok(buf.len()))
        }

        fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
            Poll::Ready(Ok(()))
        }

        fn poll_close(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
            Poll::Ready(Ok(()))
        }
    }

    fn setup_alice_bob(
        cfg: SessionConfig,
        network_cfg: FaultyNetworkConfig,
    ) -> (SessionSocket<FaultyNetwork>, SessionSocket<FaultyNetwork>) {
        let alice_to_bob_transport = FaultyNetwork::new(network_cfg.clone());
        let bob_to_alice_transport = FaultyNetwork::new(network_cfg.clone());

        let mut alice_to_bob = SessionSocket::new(alice_to_bob_transport, cfg.clone());
        let mut bob_to_alice = SessionSocket::new(bob_to_alice_transport, cfg.clone());

        alice_to_bob
            .state_mut()
            .transport
            .counterparty
            .set(bob_to_alice.state().clone())
            .unwrap();
        bob_to_alice
            .state_mut()
            .transport
            .counterparty
            .set(alice_to_bob.state().clone())
            .unwrap();

        let mut alice_bob_state = alice_to_bob.state().clone();
        let mut bob_alice_state = bob_to_alice.state().clone();
        async_std::task::spawn(async move {
            loop {
                alice_bob_state.acknowledge_segments(None).await.unwrap();
                alice_bob_state.request_missing_segments(None).await.unwrap();

                bob_alice_state.acknowledge_segments(None).await.unwrap();
                bob_alice_state.request_missing_segments(None).await.unwrap();

                async_std::task::sleep(Duration::from_millis(50)).await;
            }
        });

        (alice_to_bob, bob_to_alice)
    }

    impl FaultyNetwork {
        pub fn new(cfg: FaultyNetworkConfig) -> Self {
            info!("network RNG seed: {}", hex::encode(RNG_SEED.as_slice()));

            let rng = rand_chacha::ChaCha20Rng::from_seed(RNG_SEED.clone());
            let counterparty = Arc::new(OnceLock::<SessionState<FaultyNetwork>>::new());

            let (sender, recv) = futures::channel::mpsc::unbounded::<Box<[u8]>>();
            let mut rng_clone = rng.clone();
            let recv = if cfg.mixing_factor > 0 {
                recv.map(move |x| {
                    async_std::task::sleep(Duration::from_micros(rng_clone.gen_range(10..1000)))
                        .then(|_| futures::future::ready(x))
                })
                .buffer_unordered(cfg.mixing_factor)
                .boxed()
            } else {
                recv.boxed()
            };

            let counterparty_clone = counterparty.clone();
            async_std::task::spawn(async move {
                pin_mut!(recv);
                while let Some(data) = recv.next().await {
                    if let Some(mut counterparty) = counterparty_clone.get().cloned() {
                        counterparty.received_message(&data).await.unwrap();
                    }
                }
            });

            Self {
                sender,
                counterparty,
                cfg,
                msg_counter: 0,
            }
        }

        fn send_to_counterparty(&mut self, data: &[u8]) -> crate::errors::Result<()> {
            if self.cfg.fault_mod > 0 && self.msg_counter > 0 && self.msg_counter % self.cfg.fault_mod as u128 == 0 {
                warn!("msg discarded");
            } else {
                self.sender.unbounded_send(data.into()).unwrap();
            }
            self.msg_counter += 1;
            Ok(())
        }
    }

    #[derive(PartialEq, Eq)]
    enum Direction {
        Send,
        Recv,
        Both,
    }

    async fn send_and_recv<S>(num_frames: usize, frame_size: usize, alice: S, bob: S, timeout: Duration, one_way: bool)
    where
        S: AsyncRead + AsyncWrite + Unpin + Send + 'static,
    {
        let socket_worker = |mut socket: S, d: Direction| async move {
            let mut received = Vec::with_capacity(num_frames * frame_size);
            let mut sent = Vec::with_capacity(num_frames * frame_size);

            if d == Direction::Send || d == Direction::Both {
                for _ in 0..num_frames {
                    let mut write = vec![0u8; frame_size];
                    hopr_crypto_random::random_fill(&mut write);
                    socket.write(&write).await.unwrap();
                    sent.extend(write);
                }
            }

            if d == Direction::Recv || d == Direction::Both {
                for _ in 0..num_frames {
                    let mut read = vec![0u8; frame_size];
                    socket.read_exact(&mut read).await.unwrap();
                    received.extend(read);
                }
            }

            (sent, received)
        };

        let alice_worker = async_std::task::spawn(socket_worker(
            alice,
            if one_way { Direction::Send } else { Direction::Both },
        ));
        let bob_worker = async_std::task::spawn(socket_worker(
            bob,
            if one_way { Direction::Recv } else { Direction::Both },
        ));

        let send_recv = futures::future::join(bob_worker, alice_worker);
        let timeout = async_std::task::sleep(timeout);

        pin_mut!(send_recv);
        pin_mut!(timeout);

        match futures::future::select(send_recv, timeout).await {
            Either::Left((((alice_sent, alice_recv), (bob_sent, bob_recv)), _)) => {
                assert_eq!(
                    alice_sent,
                    bob_recv,
                    "alice sent must be equal to bob received {}",
                    hex::encode(RNG_SEED.clone())
                );
                assert_eq!(
                    bob_sent,
                    alice_recv,
                    "bob sent must be equal to alice received {}",
                    hex::encode(RNG_SEED.clone())
                );
            }
            Either::Right(_) => panic!("timeout {}", hex::encode(RNG_SEED.clone())),
        }
    }

    const NUM_FRAMES: usize = 1000;
    const FRAME_SIZE: usize = 1500;

    #[test(async_std::test)]
    async fn test_reliable_send_recv_no_ack() {
        let (alice_to_bob, bob_to_alice) = setup_alice_bob(Default::default(), Default::default());

        send_and_recv(
            NUM_FRAMES,
            FRAME_SIZE,
            alice_to_bob,
            bob_to_alice,
            Duration::from_secs(10),
            false,
        )
        .await;
    }

    #[test(async_std::test)]
    async fn test_reliable_send_recv() {
        let cfg = SessionConfig {
            acknowledged_frames_buffer: 1024,
            ..Default::default()
        };

        let (alice_to_bob, bob_to_alice) = setup_alice_bob(cfg, Default::default());

        send_and_recv(
            NUM_FRAMES,
            FRAME_SIZE,
            alice_to_bob,
            bob_to_alice,
            Duration::from_secs(10),
            false,
        )
        .await;
    }

    #[test(async_std::test)]
    async fn test_unreliable_send_recv() {
        let cfg = SessionConfig {
            acknowledged_frames_buffer: 1024,
            ..Default::default()
        };

        let net_cfg = FaultyNetworkConfig {
            fault_mod: 3,
            ..Default::default()
        };

        let (alice_to_bob, bob_to_alice) = setup_alice_bob(cfg, net_cfg);

        send_and_recv(
            NUM_FRAMES,
            FRAME_SIZE,
            alice_to_bob,
            bob_to_alice,
            Duration::from_secs(30),
            false,
        )
        .await;

        async_std::task::sleep(Duration::from_millis(100)).await;
    }

    #[test(async_std::test)]
    async fn test_unreliable_mixed_send_recv() {
        let cfg = SessionConfig {
            acknowledged_frames_buffer: 1024,
            ..Default::default()
        };

        let net_cfg = FaultyNetworkConfig {
            fault_mod: 3,
            mixing_factor: 4,
        };

        let (alice_to_bob, bob_to_alice) = setup_alice_bob(cfg, net_cfg);

        send_and_recv(
            NUM_FRAMES,
            FRAME_SIZE,
            alice_to_bob,
            bob_to_alice,
            Duration::from_secs(30),
            false,
        )
        .await;

        async_std::task::sleep(Duration::from_millis(500)).await;
    }

    #[test(async_std::test)]
    async fn test_almost_reliable_mixed_send_recv() {
        let cfg = SessionConfig {
            acknowledged_frames_buffer: 1024,
            ..Default::default()
        };

        let net_cfg = FaultyNetworkConfig {
            fault_mod: 10,
            mixing_factor: 4,
        };

        let (alice_to_bob, bob_to_alice) = setup_alice_bob(cfg, net_cfg);

        send_and_recv(
            NUM_FRAMES,
            FRAME_SIZE,
            alice_to_bob,
            bob_to_alice,
            Duration::from_secs(30),
            false,
        )
        .await;

        async_std::task::sleep(Duration::from_millis(100)).await;
    }

    #[test(async_std::test)]
    async fn test_reliable_mixed_send_recv() {
        let cfg = SessionConfig {
            acknowledged_frames_buffer: 1024,
            ..Default::default()
        };

        let net_cfg = FaultyNetworkConfig {
            fault_mod: 0,
            mixing_factor: 4,
        };

        let (alice_to_bob, bob_to_alice) = setup_alice_bob(cfg, net_cfg);

        send_and_recv(
            NUM_FRAMES,
            FRAME_SIZE,
            alice_to_bob,
            bob_to_alice,
            Duration::from_secs(30),
            false,
        )
        .await;

        async_std::task::sleep(Duration::from_millis(100)).await;
    }
}
