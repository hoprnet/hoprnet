use async_trait::async_trait;
use crossbeam_queue::ArrayQueue;
use crossbeam_skiplist::SkipMap;
use futures::future::BoxFuture;
use futures::{AsyncRead, AsyncWrite, FutureExt, StreamExt, TryStreamExt};
use pin_project::pin_project;
use smart_default::SmartDefault;
use std::future::Future;
use std::ops::Deref;
use std::pin::Pin;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Duration;
use tracing::{debug, error};

use crate::errors::NetworkTypeError;
use crate::frame::{segment, FrameId, FrameReassembler, SegmentId};
use crate::prelude::Segment;
use crate::session::protocol::{FrameAcknowledgements, SessionMessage};

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait NetworkTransport {
    async fn send_to_counterparty(&self, data: &[u8]) -> crate::errors::Result<()>;
}

/// Configuration of session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, SmartDefault)]
pub struct SessionConfig {
    /// Maximum number of buffered segments.
    #[default = 5120]
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
    pub mtu: u16,
}

/// Contains the cloneable state of the session bound to a [`SessionSocket`].
///
/// It implements the entire [`SessionMessage`] state machine and
/// performs the frame reassembly and sequencing.
/// The underlying transport operations are bound to [`NetworkTransport`]
///
/// The `SessionState` cannot be created directly, it must always be created via [`SessionSocket`] and
/// then retrieved via [`SessionSocket::state`].
#[derive(Debug)]
pub struct SessionState<T: NetworkTransport> {
    transport: Arc<T>,
    lookbehind: Arc<SkipMap<SegmentId, Segment>>,
    acknowledged: Arc<Option<ArrayQueue<FrameId>>>,
    outgoing_frame_id: Arc<AtomicU32>,
    frame_reassembler: Arc<FrameReassembler>,
    cfg: SessionConfig,
}

impl<T: NetworkTransport> SessionState<T> {
    /// Should be called by the underlying transport when raw packet data are received.
    /// The `data` argument must contain a valid [`SessionMessage`], otherwise the method throws an error.
    pub async fn received_message(&self, data: &[u8]) -> crate::errors::Result<()> {
        match SessionMessage::try_from(data)? {
            SessionMessage::Segment(s) => {
                let id = SegmentId::from(&s);
                if let Err(e) = self.frame_reassembler.push_segment(s) {
                    error!("segment {id:?} not pushed: {e}")
                } else {
                    debug!("received segment {id:?}");
                }
            }
            SessionMessage::Request(r) => {
                debug!("received request for {} segments in {}", r.len(), r.frame_id);
                let mut count = 0;
                for segment_id in r {
                    if let Some(segment) = self.lookbehind.get(&segment_id) {
                        let msg = SessionMessage::Segment(segment.value().clone());
                        self.transport.send_to_counterparty(&msg.into_encoded()).await?;
                        count += 1;
                    } else {
                        error!("segment {segment_id:?} not in lookbehind buffer anymore");
                    }
                }
                debug!("re-sent {count} requested segments");
            }
            SessionMessage::Acknowledge(f) => {
                debug!("received acknowledgement of {} frames", f.len());
                for frame_id in f {
                    for seg in self.lookbehind.iter().filter(|s| frame_id == s.key().0) {
                        seg.remove();
                    }
                }
            }
        }

        Ok(())
    }

    /// Sends a requests for missing segments in incomplete frames.
    /// One [request](SessionMessage::Request) message is sent per incomplete frame. The message contains
    /// the segment indices missing from that frame. The `max_requests` argument can provide a maximum
    /// number of request messages sent by this call. If `max_requests` is `None`, request messages
    /// are set for all incomplete frames.
    /// Returns the number of sent request messages.
    pub async fn request_missing_segments(&self, max_requests: Option<usize>) -> crate::errors::Result<usize> {
        self.frame_reassembler.evict()?;

        let mut incomplete = self
            .frame_reassembler
            .incomplete_frames()
            .values()
            .cloned()
            .collect::<Vec<_>>();

        if incomplete.is_empty() {
            debug!("tracking {} incomplete frames", incomplete.len());
            return Ok(0);
        }

        debug!("tracking {} incomplete frames", incomplete.len());
        incomplete.sort_unstable_by(|a, b| a.last_update.cmp(&b.last_update));
        let max = max_requests.unwrap_or(incomplete.len());

        let mut sent = 0;
        for req in incomplete
            .into_iter()
            .take(max)
            .map(|info| SessionMessage::Request(info.into()))
        {
            let r = req.try_as_request_ref().unwrap();
            debug!(
                "sending retransmission request for segments {:?} in frame {}",
                r.clone().into_iter().collect::<Vec<_>>(),
                r.frame_id
            );
            self.transport.send_to_counterparty(&req.into_encoded()).await?;
            sent += 1;
        }

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
    pub async fn acknowledge_segments(&self, max_messages: Option<usize>) -> crate::errors::Result<usize> {
        if let Some(acked_buffer) = self.acknowledged.deref() {
            let mut len = 0;
            let mut msgs = 0;
            while !acked_buffer.is_empty() {
                let mut ack_frames = FrameAcknowledgements::default();

                if max_messages.map(|max| msgs < max).unwrap_or(true) {
                    while !ack_frames.is_full() && !acked_buffer.is_empty() {
                        if let Some(ack_id) = acked_buffer.pop() {
                            ack_frames.push(ack_id);
                            len += 1;
                        }
                    }

                    self.transport
                        .send_to_counterparty(&SessionMessage::Acknowledge(ack_frames).into_encoded())
                        .await?;
                    msgs += 1;
                } else {
                    // Break out if we sent max allowed
                    return Ok(len);
                }
            }

            debug!("sent out {len} ack in {msgs} messages");
            Ok(len)
        } else {
            Ok(0)
        }
    }

    /// Segments the `data` and sends them as (possibly multiple) [`SessionMessage::Segment`].
    /// Therefore, this method sends as many messages as needed after the data was segmented.
    /// Each segment is inserted into the lookbehind ring buffer for possible retransmissions.
    ///
    /// The size of the lookbehind ring buffer is given by the [`max_buffered_segments`](SessionConfig)
    /// given during the construction. It needs to accommodate as many segments as
    /// is the expected underlying transport bandwidth (segment/sec) to guarantee the retransmission
    /// can still happen within some time window.
    pub async fn send_frame_data(&self, data: &[u8]) -> crate::errors::Result<()> {
        let segments = segment(
            data,
            self.cfg.mtu - SessionMessage::HEADER_SIZE as u16,
            self.outgoing_frame_id.fetch_add(1, Ordering::SeqCst),
        );

        for segment in segments {
            self.lookbehind.insert((&segment).into(), segment.clone());

            let msg = SessionMessage::Segment(segment.clone());
            debug!("sending segment {:?}", SegmentId::from(&segment));
            self.transport.send_to_counterparty(&msg.into_encoded()).await?;

            // TODO: prevent stalling here
            while self.lookbehind.len() > self.cfg.max_buffered_segments {
                self.lookbehind.pop_front();
            }
        }

        Ok(())
    }
}

impl<T: NetworkTransport> Clone for SessionState<T> {
    fn clone(&self) -> Self {
        Self {
            transport: self.transport.clone(),
            lookbehind: self.lookbehind.clone(),
            acknowledged: self.acknowledged.clone(),
            outgoing_frame_id: self.outgoing_frame_id.clone(),
            frame_reassembler: self.frame_reassembler.clone(),
            cfg: self.cfg.clone(),
        }
    }
}

/// Represents a socket for a session between two nodes bound by the
/// underlying [`NetworkTransport`].
///
/// It also implements [`AsyncRead`] and [`AsyncWrite`] so that it can
/// be used on top of the usual transport stack.
#[pin_project]
pub struct SessionSocket<T: NetworkTransport> {
    state: SessionState<T>,
    #[pin]
    egress: Box<dyn AsyncRead + Send + Unpin>,
}

impl<T: NetworkTransport> SessionSocket<T> {
    /// Create a new socket over the given `transport` that binds the communicating parties.
    pub fn new(transport: Arc<T>, cfg: SessionConfig) -> Self {
        let (reassembler, egress) = FrameReassembler::new(cfg.frame_expiration_age.into());

        let acknowledged =
            Arc::new((cfg.acknowledged_frames_buffer > 0).then(|| ArrayQueue::new(cfg.acknowledged_frames_buffer)));

        let state = SessionState {
            transport,
            lookbehind: Arc::new(SkipMap::new()),
            acknowledged: acknowledged.clone(),
            outgoing_frame_id: Arc::new(AtomicU32::new(1)),
            frame_reassembler: Arc::new(reassembler),
            cfg,
        };

        let egress = Box::new(
            egress
                .inspect(move |frame| {
                    debug!("emit frame {}", frame.frame_id);
                    if let Some(ack_buf) = acknowledged.deref() {
                        // Acts as a ring buffer, so if the buffer is full, any unsent acknowledgements
                        // will be discarded.
                        ack_buf.force_push(frame.frame_id);
                    }
                })
                .map(Ok)
                .into_async_read(),
        );

        Self { state, egress }
    }

    /// The [state](SessionState) of this socket.
    pub fn state(&self) -> &SessionState<T> {
        &self.state
    }
}

#[pin_project]
struct SocketWriter<'a, T: NetworkTransport + Send + Sync> {
    state: &'a SessionState<T>,
    #[pin]
    future: BoxFuture<'a, Result<(), NetworkTypeError>>,
}

impl<'a, T: NetworkTransport + Send + Sync> SocketWriter<'a, T> {
    fn new(state: &'a SessionState<T>, data: &'a [u8]) -> Self {
        Self {
            state,
            future: state.send_frame_data(data).boxed(),
        }
    }
}

impl<'a, T: NetworkTransport + Send + Sync> Future for SocketWriter<'a, T> {
    type Output = Result<(), NetworkTypeError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        this.future.poll(cx)
    }
}

impl<T: NetworkTransport + Send + Sync> AsyncWrite for SessionSocket<T> {
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<std::io::Result<usize>> {
        let mut socket_future = SocketWriter::new(&self.state, buf);
        match Pin::new(&mut socket_future).poll(cx) {
            Poll::Ready(Ok(())) => Poll::Ready(Ok(buf.len())),
            Poll::Ready(Err(e)) => Poll::Ready(Err(std::io::Error::new(std::io::ErrorKind::Other, e))),
            Poll::Pending => Poll::Pending,
        }
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Poll::Ready(Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Poll::Ready(Ok(()))
    }
}

impl<T: NetworkTransport + Send + Sync> AsyncRead for SessionSocket<T> {
    fn poll_read(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut [u8]) -> Poll<std::io::Result<usize>> {
        self.project().egress.poll_read(cx, buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::tests::sample_index;
    use async_stream::stream;
    use futures::channel::mpsc::UnboundedSender;
    use futures::future::Either;
    use futures::io::{AsyncReadExt, AsyncWriteExt};
    use futures::lock::Mutex;
    use futures::{pin_mut, Stream};
    use hex_literal::hex;
    use lazy_static::lazy_static;
    use rand::{thread_rng, Rng, SeedableRng};
    use rand_distr::{Distribution, Normal};
    use std::collections::VecDeque;
    use std::fmt::{Debug, Formatter};
    use std::ops::DerefMut;
    use std::sync::OnceLock;
    use test_log::test;
    use tracing::{info, warn};

    lazy_static! {
        //static ref RNG_SEED: [u8; 32] = hopr_crypto_random::random_bytes();
        static ref RNG_SEED: [u8; 32] = hex!("4042a10ca2b3f9f9e34ae3c8d5fa6298dfbacf6009a16c04754a7d7c626ec1dc");
    }

    fn randomized_stream<S, R>(mut input_stream: S, mut rng: R, mixing_factor: f64) -> impl Stream<Item = S::Item>
    where
        S: Stream + Unpin + 'static,
        S::Item: Clone + Unpin + 'static,
        R: Rng + Unpin + 'static,
    {
        const BUFFER_SIZE: usize = 10;

        stream! {
            let mut buffer = VecDeque::with_capacity(BUFFER_SIZE);
            let mut dist = Normal::new(0.0, mixing_factor).unwrap();

            while let Some(item) = input_stream.next().await {
                buffer.push_back(item);
                if buffer.len() >= BUFFER_SIZE {
                    while !buffer.is_empty() {
                        if mixing_factor == 0.0 {
                            yield buffer.pop_front().unwrap();
                        } else {
                            yield buffer.remove(sample_index(&mut dist, &mut rng, buffer.len())).unwrap();
                        }
                    }
                }
            }

            // Output remaining elements
            while !buffer.is_empty() {
                if mixing_factor == 0.0 {
                    yield buffer.pop_front().unwrap();
                } else {
                    yield buffer.remove(sample_index(&mut dist, &mut rng, buffer.len())).unwrap();
                }
            }
        }
    }

    #[derive(Debug, Clone, Default)]
    pub struct FaultyNetworkConfig {
        pub fault_probability: f64,
        pub mixing_factor: f64,
    }

    pub struct FaultyNetwork {
        rng: Mutex<(rand_distr::Bernoulli, rand_chacha::ChaCha20Rng)>,
        sender: UnboundedSender<Box<[u8]>>,
        counterparty: Arc<OnceLock<SessionState<FaultyNetwork>>>,
    }

    impl Debug for FaultyNetwork {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "network with {:?}", &self.counterparty)
        }
    }

    fn setup_alice_bob(
        cfg: SessionConfig,
        network_cfg: FaultyNetworkConfig,
    ) -> (
        Arc<Mutex<SessionSocket<FaultyNetwork>>>,
        Arc<Mutex<SessionSocket<FaultyNetwork>>>,
    ) {
        let alice_to_bob_transport = Arc::new(FaultyNetwork::new(network_cfg.clone()));
        let bob_to_alice_transport = Arc::new(FaultyNetwork::new(network_cfg.clone()));

        let alice_to_bob = SessionSocket::new(alice_to_bob_transport.clone(), cfg.clone());
        let bob_to_alice = SessionSocket::new(bob_to_alice_transport.clone(), cfg.clone());

        alice_to_bob_transport
            .counterparty
            .set(bob_to_alice.state().clone())
            .unwrap();
        bob_to_alice_transport
            .counterparty
            .set(alice_to_bob.state().clone())
            .unwrap();

        let alice_bob_state = alice_to_bob.state().clone();
        let bob_alice_state = bob_to_alice.state().clone();
        async_std::task::spawn(async move {
            loop {
                async_std::task::sleep(Duration::from_millis(300)).await;

                alice_bob_state.acknowledge_segments(None).await.unwrap();
                alice_bob_state.request_missing_segments(None).await.unwrap();

                bob_alice_state.acknowledge_segments(None).await.unwrap();
                bob_alice_state.request_missing_segments(None).await.unwrap();
            }
        });

        (Arc::new(Mutex::new(alice_to_bob)), Arc::new(Mutex::new(bob_to_alice)))
    }

    impl FaultyNetwork {
        pub fn new(cfg: FaultyNetworkConfig) -> Self {
            info!("network RNG seed: {}", hex::encode(RNG_SEED.as_slice()));

            let rng = rand_chacha::ChaCha20Rng::from_seed(RNG_SEED.clone());
            let counterparty = Arc::new(OnceLock::<SessionState<FaultyNetwork>>::new());

            let (sender, recv) = futures::channel::mpsc::unbounded::<Box<[u8]>>();
            let recv = randomized_stream(recv, rng.clone(), cfg.mixing_factor);
            let counterparty_clone = counterparty.clone();

            async_std::task::spawn(async move {
                pin_mut!(recv);
                while let Some(data) = recv.next().await {
                    if let Some(counterparty) = counterparty_clone.get() {
                        counterparty.received_message(&data).await.unwrap();
                    }
                }
            });

            Self {
                rng: Mutex::new((rand_distr::Bernoulli::new(1.0 - cfg.fault_probability).unwrap(), rng)),
                sender,
                counterparty,
            }
        }
    }

    #[async_trait]
    impl NetworkTransport for FaultyNetwork {
        async fn send_to_counterparty(&self, data: &[u8]) -> crate::errors::Result<()> {
            let mut prob_rng = self.rng.lock().await;
            let (distr, rng) = prob_rng.deref_mut();
            if distr.sample(rng) {
                self.sender.unbounded_send(data.into()).unwrap();
            } else {
                warn!("msg discarded");
            }
            Ok(())
        }
    }

    async fn send_and_recv<R: AsyncRead + Unpin + Send + 'static, W: AsyncWrite + Unpin + Send + 'static>(
        num_frames: usize,
        frame_size: usize,
        sender: Arc<Mutex<W>>,
        recv: Arc<Mutex<R>>,
        timeout: Duration,
    ) {
        let sender = async_std::task::spawn(async move {
            let mut sent = Vec::new();
            for _ in 0..num_frames {
                let mut data = vec![0u8; frame_size];
                hopr_crypto_random::random_fill(&mut data);
                sender.lock().await.write(&data).await.unwrap();
                sent.push(data);
            }
            sent
        });

        let receiver = async_std::task::spawn(async move {
            let mut received = Vec::new();
            for _ in 0..num_frames {
                let mut read = vec![0u8; frame_size];
                recv.lock().await.read_exact(&mut read).await.unwrap();
                received.push(read);
            }
            received
        });

        let send_recv = futures::future::join(sender, receiver);
        let timeout = async_std::task::sleep(timeout);

        pin_mut!(send_recv);
        pin_mut!(timeout);

        match futures::future::select(send_recv, timeout).await {
            Either::Left(((sent, received), _)) => assert_eq!(sent, received),
            Either::Right(_) => panic!("timeout"),
        }
    }

    #[async_std::test]
    async fn test_randomized_stream_without_mixing_outputs_in_order() {
        const SIZE: usize = 1000;
        let stream = futures::stream::iter(0..SIZE);
        let randomized = randomized_stream(stream, thread_rng(), 0.0);

        let out = randomized.collect::<Vec<_>>().await;
        assert_eq!(out, (0..SIZE).into_iter().collect::<Vec<_>>());
    }

    #[async_std::test]
    async fn test_randomized_stream_with_mixing_outputs_out_of_order() {
        const SIZE: usize = 1000;
        let stream = futures::stream::iter(0..SIZE);
        let randomized = randomized_stream(stream, thread_rng(), 10.0);

        let mut out = randomized.collect::<Vec<_>>().await;
        assert_ne!(out, (0..SIZE).into_iter().collect::<Vec<_>>());

        out.sort();
        assert_eq!(out, (0..SIZE).into_iter().collect::<Vec<_>>());
    }

    #[test(async_std::test)]
    async fn test_reliable_send_recv_no_ack() {
        let (alice_to_bob, bob_to_alice) = setup_alice_bob(Default::default(), Default::default());

        send_and_recv(
            1000,
            1500,
            alice_to_bob.clone(),
            bob_to_alice.clone(),
            Duration::from_secs(10),
        )
        .await;

        send_and_recv(
            1000,
            1500,
            bob_to_alice.clone(),
            alice_to_bob.clone(),
            Duration::from_secs(10),
        )
        .await
    }

    #[test(async_std::test)]
    async fn test_reliable_send_recv() {
        let cfg = SessionConfig {
            acknowledged_frames_buffer: 1024,
            ..Default::default()
        };

        let (alice_to_bob, bob_to_alice) = setup_alice_bob(cfg, Default::default());

        send_and_recv(
            1000,
            1500,
            alice_to_bob.clone(),
            bob_to_alice.clone(),
            Duration::from_secs(10),
        )
        .await;

        send_and_recv(
            1000,
            1500,
            bob_to_alice.clone(),
            alice_to_bob.clone(),
            Duration::from_secs(10),
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
            fault_probability: 0.1,
            ..Default::default()
        };

        let (alice_to_bob, bob_to_alice) = setup_alice_bob(cfg, net_cfg);

        send_and_recv(
            1000,
            1500,
            alice_to_bob.clone(),
            bob_to_alice.clone(),
            Duration::from_secs(30),
        )
        .await;

        send_and_recv(
            1000,
            1500,
            bob_to_alice.clone(),
            alice_to_bob.clone(),
            Duration::from_secs(30),
        )
        .await;

        async_std::task::sleep(Duration::from_millis(500)).await;
    }

    #[test(async_std::test)]
    async fn test_unreliable_mixed_send_recv() {
        let cfg = SessionConfig {
            acknowledged_frames_buffer: 1024,
            ..Default::default()
        };

        let net_cfg = FaultyNetworkConfig {
            fault_probability: 0.1,
            mixing_factor: 4.0,
        };

        let (alice_to_bob, bob_to_alice) = setup_alice_bob(cfg, net_cfg);

        send_and_recv(
            1000,
            1500,
            alice_to_bob.clone(),
            bob_to_alice.clone(),
            Duration::from_secs(30),
        )
        .await;

        send_and_recv(
            1000,
            1500,
            bob_to_alice.clone(),
            alice_to_bob.clone(),
            Duration::from_secs(30),
        )
        .await;

        async_std::task::sleep(Duration::from_millis(500)).await;
    }

    #[test(async_std::test)]
    async fn test_reliable_mixed_send_recv() {
        let cfg = SessionConfig {
            acknowledged_frames_buffer: 1024,
            ..Default::default()
        };

        let net_cfg = FaultyNetworkConfig {
            fault_probability: 0.0,
            mixing_factor: 4.0,
        };

        let (alice_to_bob, bob_to_alice) = setup_alice_bob(cfg, net_cfg);

        send_and_recv(
            1000,
            1500,
            alice_to_bob.clone(),
            bob_to_alice.clone(),
            Duration::from_secs(30),
        )
        .await;

        send_and_recv(
            1000,
            1500,
            bob_to_alice.clone(),
            alice_to_bob.clone(),
            Duration::from_secs(30),
        )
        .await;

        async_std::task::sleep(Duration::from_millis(500)).await;
    }
}
