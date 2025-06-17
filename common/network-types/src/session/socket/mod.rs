pub mod ack_state;
pub mod state;

use std::{
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

use futures::{FutureExt, SinkExt, StreamExt, TryStreamExt, future, pin_mut};
use futures_concurrency::stream::Merge;
use state::SocketState;
use tracing::{Instrument, instrument};

use crate::session::{
    errors::SessionError,
    frames::{FrameInspector, OrderedFrame},
    processing::{ReassemblerExt, SegmenterExt, SequencerExt},
    protocol::{SessionCodec, SessionMessage},
    socket::state::{SocketComponents, SocketStateEvent, Stateless},
};

/// Configuration object for [`SessionSocket`].
#[derive(Debug, Copy, Clone, Eq, PartialEq, smart_default::SmartDefault)]
pub struct SessionSocketConfig {
    /// The maximum size of a frame on the read/write interface of the [`SessionSocket`].
    ///
    /// Default is 1500 bytes.
    #[default(1500)]
    pub frame_size: usize,
    /// The maximum time to wait for a frame to be fully received.
    ///
    /// Default is 800 ms.
    #[default(Duration::from_millis(800))]
    pub frame_timeout: Duration,
    /// Maximum number of segments to buffer in the downstream transport.
    /// If 0 is given, the transport is unbuffered.
    ///
    /// Default is 0.
    #[default(0)]
    pub max_buffered_segments: usize,
    /// Capacity of the frame reconstructor, the maximum number of incomplete frames, before
    /// they are dropped.
    ///
    /// Default is 8192.
    #[default(8192)]
    pub capacity: usize,
}

/// Socket-like object implementing the Session protocol that can operate on any transport that
/// implements [`futures::io::AsyncRead`] and [`futures::io::AsyncWrite`].
///
/// The [`SocketState`] `S` given during instantiation can facilitate reliable or unreliable
/// behavior.
#[pin_project::pin_project]
pub struct SessionSocket<const C: usize, S> {
    // This is where upstream writes the to-be-segmented frame data to
    upstream_frames_in: Pin<Box<dyn futures::io::AsyncWrite + Send>>,
    // This is where upstream reads the reconstructed frame data from
    downstream_frames_out: Pin<Box<dyn futures::io::AsyncRead + Send>>,
    state: S,
}

impl<const C: usize> SessionSocket<C, Stateless<C>> {
    /// Creates a new stateless socket suitable for fast UDP-like communication.
    ///
    /// Note that this results in a faster socket than if created via [`SessionSocket::new`] with
    /// [`Stateless`]. This is because the frame inspector does not need to be instantiated.
    pub fn new_stateless<T, I>(id: I, transport: T, cfg: SessionSocketConfig) -> Result<Self, SessionError>
    where
        T: futures::io::AsyncRead + futures::io::AsyncWrite + Send + Unpin + 'static,
        I: std::fmt::Display + Clone,
    {
        // Segment data incoming/outgoing using underlying transport
        let mut framed = asynchronous_codec::Framed::new(transport, SessionCodec::<C>);

        // Check if we allow sending multiple segments to downstream in a single write
        // The HWM cannot be 0 bytes
        framed.set_send_high_water_mark(1.max(cfg.max_buffered_segments * C));

        // Downstream transport
        let (packets_out, packets_in) = framed.split();

        // Messages incoming from Upstream
        let upstream_frames_in = packets_out
            .with(|segment| future::ok(SessionMessage::<C>::Segment(segment)))
            .segmenter::<C>(cfg.frame_size);

        // Packets incoming from Downstream
        // - if the State requires it, packets passed by-ref into the State
        // - Packets that represent Segments (filtered out) are passed to the Reconstructor
        let downstream_frames_out = packets_in
            .filter_map(|packet| {
                futures::future::ready(match packet {
                    Ok(packet) => packet.try_as_segment(),
                    Err(error) => {
                        tracing::error!(%error, "unparseable packet");
                        None
                    }
                })
            })
            .reassembler(cfg.frame_timeout, cfg.capacity)
            .filter_map(|maybe_frame| {
                futures::future::ready(match maybe_frame {
                    Ok(frame) => Some(OrderedFrame(frame)),
                    Err(error) => {
                        tracing::error!(%error, "failed to reassemble frame");
                        None
                    }
                })
            })
            .sequencer(cfg.frame_timeout, cfg.capacity)
            .filter_map(move |maybe_frame| {
                future::ready(match maybe_frame {
                    Ok(frame) => Some(Ok(frame.0)),
                    // Downstream skips discarded frames
                    Err(SessionError::FrameDiscarded(frame_id)) | Err(SessionError::IncompleteFrame(frame_id)) => {
                        tracing::error!(frame_id, "frame discarded");
                        None
                    }
                    Err(err) => Some(Err(std::io::Error::other(err))),
                })
            })
            .into_async_read();

        Ok(Self {
            state: Stateless::new(id),
            upstream_frames_in: Box::pin(upstream_frames_in),
            downstream_frames_out: Box::pin(downstream_frames_out),
        })
    }
}

impl<const C: usize, S: SocketState<C> + Clone + 'static> SessionSocket<C, S> {
    /// Creates a stateful socket with frame inspection capabilities - suitable for communication
    /// requiring TCP-like delivery guarantees.
    pub fn new<T>(transport: T, mut state: S, cfg: SessionSocketConfig) -> Result<Self, SessionError>
    where
        T: futures::io::AsyncRead + futures::io::AsyncWrite + Send + Unpin + 'static,
    {
        // Segment data incoming/outgoing using underlying transport
        let mut framed = asynchronous_codec::Framed::new(transport, SessionCodec::<C>);

        // Check if we allow sending multiple segments to downstream in a single write
        // The HWM cannot be 0 bytes
        framed.set_send_high_water_mark(1.max(cfg.max_buffered_segments * C));

        // Downstream transport
        let (packets_out, packets_in) = framed.split();

        let inspector = FrameInspector::new(cfg.capacity);

        let (ctl_tx, ctl_rx) = futures::channel::mpsc::unbounded();
        let sub_events = state.run(SocketComponents {
            inspector: Some(inspector.clone()),
            ctl_tx,
        })?;

        // Messages incoming from Upstream and from the State go downstream as Packets
        // Segmented data coming from Upstream go out right away.
        let (segments_tx, segments_rx) = futures::channel::mpsc::channel(cfg.capacity);
        let mut st_1 = state.clone();
        let upstream_frames_in = segments_tx
            .sink_map_err(|_| SessionError::InvalidSegment)
            .with(move |segment| {
                // The segment_sent event is raised only for segments coming from Upstream,
                // not for the segments from the Control stream (= segment resends).
                if sub_events.contains(SocketStateEvent::SegmentSent) {
                    if let Err(error) = st_1.segment_sent(&segment) {
                        tracing::debug!(%error, "outgoing segment state update failed");
                    }
                }
                future::ok(SessionMessage::<C>::Segment(segment))
            })
            .segmenter::<C>(cfg.frame_size);

        // We have to merge the streams here and spawn a special task for it
        // Since the control messages from the State can come independent of Upstream writes.
        hopr_async_runtime::prelude::spawn(
            (ctl_rx, segments_rx)
                .merge()
                .map(Ok)
                .forward(packets_out)
                .map(move |result| match result {
                    Ok(_) => tracing::debug!("outgoing packet processing done"),
                    Err(error) => {
                        tracing::error!(%error, "error while processing outgoing packets")
                    }
                })
                .instrument(tracing::debug_span!(
                    "SessionSocket::packets_out",
                    session_id = state.session_id()
                )),
        );

        // Packets incoming from Downstream:
        // - if the State requires it, packets passed by-ref into the State
        // - Packets that represent Segments (filtered out) are passed to the Reconstructor
        let mut st_1 = state.clone();
        let mut st_2 = state.clone();
        let mut st_3 = state.clone();
        let downstream_frames_out = packets_in
            .filter_map(move |packet| {
                futures::future::ready(match packet {
                    Ok(packet) => {
                        if let Err(error) = match &packet {
                            SessionMessage::Segment(s) if sub_events.contains(SocketStateEvent::IncomingSegment) => {
                                st_1.incoming_segment(&s.id(), s.seq_len)
                            }
                            SessionMessage::Request(r)
                                if sub_events.contains(SocketStateEvent::IncomingRetransmissionRequest) =>
                            {
                                st_1.incoming_retransmission_request(r.clone())
                            }
                            SessionMessage::Acknowledge(a)
                                if sub_events.contains(SocketStateEvent::IncomingAcknowledgedFrames) =>
                            {
                                st_1.incoming_acknowledged_frames(a.clone())
                            }
                            _ => Ok(()),
                        } {
                            tracing::debug!(%error, "incoming message state update failed");
                        }
                        packet.try_as_segment()
                    }
                    Err(error) => {
                        tracing::error!(%error, "unparseable packet");
                        None
                    }
                })
            })
            .reassembler_with_inspector(cfg.frame_timeout, cfg.capacity, inspector)
            .filter_map(move |maybe_frame| {
                futures::future::ready(match maybe_frame {
                    Ok(frame) => {
                        if let Err(error) = st_2.frame_complete(frame.frame_id) {
                            tracing::error!(%error, "frame complete state update failed");
                        }
                        Some(OrderedFrame(frame))
                    }
                    Err(error) => {
                        tracing::error!(%error, "failed to reassemble frame");
                        None
                    }
                })
            })
            .sequencer(cfg.frame_timeout, cfg.frame_size)
            .filter_map(move |maybe_frame| {
                // Filter out discarded Frames and dispatch events to the State if needed
                future::ready(match maybe_frame {
                    Ok(frame) => {
                        if sub_events.contains(SocketStateEvent::FrameEmitted) {
                            if let Err(error) = st_3.frame_emitted(frame.0.frame_id) {
                                tracing::error!(%error, "frame received state update failed");
                            }
                        }
                        Some(Ok(frame.0))
                    }
                    Err(SessionError::FrameDiscarded(frame_id)) | Err(SessionError::IncompleteFrame(frame_id)) => {
                        if sub_events.contains(SocketStateEvent::FrameDiscarded) {
                            if let Err(error) = st_3.frame_discarded(frame_id) {
                                tracing::error!(%error, "frame discarded state update failed");
                            }
                        }
                        None // Downstream skips discarded frames
                    }
                    Err(err) => Some(Err(std::io::Error::other(err))),
                })
            })
            .into_async_read();

        Ok(Self {
            state,
            upstream_frames_in: Box::pin(upstream_frames_in),
            downstream_frames_out: Box::pin(downstream_frames_out),
        })
    }
}

impl<const C: usize, S: SocketState<C> + Clone + 'static> futures::io::AsyncRead for SessionSocket<C, S> {
    #[instrument(name = "SessionSocket::poll_read", level = "trace", skip(self, cx, buf), fields(session_id = self.state.session_id(), len = buf.len()))]
    fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut [u8]) -> Poll<std::io::Result<usize>> {
        let inner = &mut self.downstream_frames_out;
        pin_mut!(inner);
        inner.poll_read(cx, buf)
    }
}

impl<const C: usize, S: SocketState<C> + Clone + 'static> futures::io::AsyncWrite for SessionSocket<C, S> {
    #[instrument(name = "SessionSocket::poll_write", level = "trace", skip(self, cx, buf), fields(session_id = self.state.session_id(), len = buf.len()))]
    fn poll_write(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<std::io::Result<usize>> {
        let inner = &mut self.upstream_frames_in;
        pin_mut!(inner);
        inner.poll_write(cx, buf)
    }

    #[instrument(name = "SessionSocket::poll_flush", level = "trace", skip(self, cx), fields(session_id = self.state.session_id()))]
    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        let inner = &mut self.upstream_frames_in;
        pin_mut!(inner);
        inner.poll_flush(cx)
    }

    #[instrument(name = "SessionSocket::poll_close", level = "trace", skip(self, cx), fields(session_id = self.state.session_id()))]
    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        let this = self.project();
        let _ = this.state.stop();

        let inner = this.upstream_frames_in;
        pin_mut!(inner);
        inner.poll_close(cx)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use futures::{AsyncReadExt, AsyncWriteExt};

    use super::*;
    use crate::{
        prelude::AcknowledgementState,
        session::{AcknowledgementStateConfig, utils::test::*},
    };

    const MTU: usize = 1000;

    const FRAME_SIZE: usize = 1500;

    const DATA_SIZE: usize = 17 * MTU + 271; // Use some size not directly divisible by the MTU

    #[test_log::test(tokio::test)]
    async fn stateless_socket_unidirectional_should_work() -> anyhow::Result<()> {
        let (alice, bob) = setup_alice_bob::<MTU>(FaultyNetworkConfig::default(), None, None);

        let sock_cfg = SessionSocketConfig {
            frame_size: FRAME_SIZE,
            ..Default::default()
        };

        let mut alice_socket = SessionSocket::<MTU, _>::new_stateless("alice", alice, sock_cfg)?;
        let mut bob_socket = SessionSocket::<MTU, _>::new_stateless("bob", bob, sock_cfg)?;

        let data = hopr_crypto_random::random_bytes::<DATA_SIZE>();

        alice_socket.write_all(&data).await?;
        alice_socket.flush().await?;

        let mut bob_data = [0u8; DATA_SIZE];
        bob_socket.read_exact(&mut bob_data).await?;
        assert_eq!(data, bob_data);

        alice_socket.close().await?;
        bob_socket.close().await?;

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn stateful_socket_unidirectional_should_work() -> anyhow::Result<()> {
        let (alice, bob) = setup_alice_bob::<MTU>(FaultyNetworkConfig::default(), None, None);

        let sock_cfg = SessionSocketConfig {
            frame_size: FRAME_SIZE,
            ..Default::default()
        };

        let ack_cfg = AcknowledgementStateConfig {
            expected_packet_latency: Duration::from_millis(2),
            acknowledgement_delay: Duration::from_millis(5),
            ..Default::default()
        };

        let mut alice_socket =
            SessionSocket::<MTU, _>::new(alice, AcknowledgementState::new("alice", ack_cfg), sock_cfg)?;
        let mut bob_socket = SessionSocket::<MTU, _>::new(bob, AcknowledgementState::new("bob", ack_cfg), sock_cfg)?;

        let data = hopr_crypto_random::random_bytes::<DATA_SIZE>();

        alice_socket.write_all(&data).await?;
        alice_socket.flush().await?;

        let mut bob_data = [0u8; DATA_SIZE];
        bob_socket.read_exact(&mut bob_data).await?;
        assert_eq!(data, bob_data);

        alice_socket.close().await?;
        bob_socket.close().await?;

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn stateless_socket_bidirectional_should_work() -> anyhow::Result<()> {
        let (alice, bob) = setup_alice_bob::<MTU>(FaultyNetworkConfig::default(), None, None);

        let sock_cfg = SessionSocketConfig {
            frame_size: FRAME_SIZE,
            ..Default::default()
        };

        let mut alice_socket = SessionSocket::<MTU, _>::new_stateless("alice", alice, sock_cfg)?;
        let mut bob_socket = SessionSocket::<MTU, _>::new_stateless("bob", bob, sock_cfg)?;

        let alice_sent_data = hopr_crypto_random::random_bytes::<DATA_SIZE>();
        alice_socket.write_all(&alice_sent_data).await?;
        alice_socket.flush().await?;

        let bob_sent_data = hopr_crypto_random::random_bytes::<DATA_SIZE>();
        bob_socket.write_all(&bob_sent_data).await?;
        bob_socket.flush().await?;

        let mut bob_recv_data = [0u8; DATA_SIZE];
        bob_socket.read_exact(&mut bob_recv_data).await?;
        assert_eq!(alice_sent_data, bob_recv_data);

        let mut alice_recv_data = [0u8; DATA_SIZE];
        alice_socket.read_exact(&mut alice_recv_data).await?;
        assert_eq!(bob_sent_data, alice_recv_data);

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn stateful_socket_bidirectional_should_work() -> anyhow::Result<()> {
        let (alice, bob) = setup_alice_bob::<MTU>(FaultyNetworkConfig::default(), None, None);

        let sock_cfg = SessionSocketConfig {
            frame_size: FRAME_SIZE,
            ..Default::default()
        };

        let ack_cfg = AcknowledgementStateConfig {
            expected_packet_latency: Duration::from_millis(2),
            acknowledgement_delay: Duration::from_millis(5),
            ..Default::default()
        };

        let mut alice_socket =
            SessionSocket::<MTU, _>::new(alice, AcknowledgementState::new("alice", ack_cfg), sock_cfg)?;
        let mut bob_socket = SessionSocket::<MTU, _>::new(bob, AcknowledgementState::new("bob", ack_cfg), sock_cfg)?;

        let alice_sent_data = hopr_crypto_random::random_bytes::<DATA_SIZE>();
        alice_socket.write_all(&alice_sent_data).await?;
        alice_socket.flush().await?;

        let bob_sent_data = hopr_crypto_random::random_bytes::<DATA_SIZE>();
        bob_socket.write_all(&bob_sent_data).await?;
        bob_socket.flush().await?;

        let mut bob_recv_data = [0u8; DATA_SIZE];
        bob_socket.read_exact(&mut bob_recv_data).await?;
        assert_eq!(alice_sent_data, bob_recv_data);

        let mut alice_recv_data = [0u8; DATA_SIZE];
        alice_socket.read_exact(&mut alice_recv_data).await?;
        assert_eq!(bob_sent_data, alice_recv_data);

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn stateless_socket_unidirectional_should_work_with_mixing() -> anyhow::Result<()> {
        let network_cfg = FaultyNetworkConfig {
            mixing_factor: 10,
            ..Default::default()
        };

        let (alice, bob) = setup_alice_bob::<MTU>(network_cfg, None, None);

        let sock_cfg = SessionSocketConfig {
            frame_size: FRAME_SIZE,
            ..Default::default()
        };

        let mut alice_socket = SessionSocket::<MTU, _>::new_stateless("alice", alice, sock_cfg)?;
        let mut bob_socket = SessionSocket::<MTU, _>::new_stateless("bob", bob, sock_cfg)?;

        let data = hopr_crypto_random::random_bytes::<DATA_SIZE>();
        alice_socket.write_all(&data).await?;
        alice_socket.flush().await?;

        let mut bob_recv_data = [0u8; DATA_SIZE];
        bob_socket.read_exact(&mut bob_recv_data).await?;
        assert_eq!(data, bob_recv_data);

        alice_socket.close().await?;
        bob_socket.close().await?;

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn stateful_socket_unidirectional_should_work_with_mixing() -> anyhow::Result<()> {
        let network_cfg = FaultyNetworkConfig {
            mixing_factor: 10,
            ..Default::default()
        };

        let (alice, bob) = setup_alice_bob::<MTU>(network_cfg, None, None);

        let sock_cfg = SessionSocketConfig {
            frame_size: FRAME_SIZE,
            ..Default::default()
        };

        let ack_cfg = AcknowledgementStateConfig {
            expected_packet_latency: Duration::from_millis(2),
            acknowledgement_delay: Duration::from_millis(5),
            ..Default::default()
        };

        let mut alice_socket =
            SessionSocket::<MTU, _>::new(alice, AcknowledgementState::new("alice", ack_cfg), sock_cfg)?;
        let mut bob_socket = SessionSocket::<MTU, _>::new(bob, AcknowledgementState::new("bob", ack_cfg), sock_cfg)?;

        let data = hopr_crypto_random::random_bytes::<DATA_SIZE>();
        alice_socket.write_all(&data).await?;
        alice_socket.flush().await?;

        let mut bob_recv_data = [0u8; DATA_SIZE];
        bob_socket.read_exact(&mut bob_recv_data).await?;
        assert_eq!(data, bob_recv_data);

        alice_socket.close().await?;
        bob_socket.close().await?;

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn stateless_socket_bidirectional_should_work_with_mixing() -> anyhow::Result<()> {
        let network_cfg = FaultyNetworkConfig {
            mixing_factor: 10,
            ..Default::default()
        };

        let (alice, bob) = setup_alice_bob::<MTU>(network_cfg, None, None);

        let sock_cfg = SessionSocketConfig {
            frame_size: FRAME_SIZE,
            ..Default::default()
        };

        let mut alice_socket = SessionSocket::<MTU, _>::new_stateless("alice", alice, sock_cfg)?;
        let mut bob_socket = SessionSocket::<MTU, _>::new_stateless("bob", bob, sock_cfg)?;

        let alice_sent_data = hopr_crypto_random::random_bytes::<DATA_SIZE>();
        alice_socket.write_all(&alice_sent_data).await?;
        alice_socket.flush().await?;

        let bob_sent_data = hopr_crypto_random::random_bytes::<DATA_SIZE>();
        bob_socket.write_all(&bob_sent_data).await?;
        bob_socket.flush().await?;

        let mut bob_recv_data = [0u8; DATA_SIZE];
        bob_socket.read_exact(&mut bob_recv_data).await?;
        assert_eq!(alice_sent_data, bob_recv_data);

        let mut alice_recv_data = [0u8; DATA_SIZE];
        alice_socket.read_exact(&mut alice_recv_data).await?;
        assert_eq!(bob_sent_data, alice_recv_data);

        alice_socket.close().await?;
        bob_socket.close().await?;

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn stateful_socket_bidirectional_should_work_with_mixing() -> anyhow::Result<()> {
        let network_cfg = FaultyNetworkConfig {
            mixing_factor: 10,
            ..Default::default()
        };

        let (alice, bob) = setup_alice_bob::<MTU>(network_cfg, None, None);

        let sock_cfg = SessionSocketConfig {
            frame_size: FRAME_SIZE,
            ..Default::default()
        };

        let ack_cfg = AcknowledgementStateConfig {
            expected_packet_latency: Duration::from_millis(2),
            acknowledgement_delay: Duration::from_millis(5),
            ..Default::default()
        };

        let mut alice_socket =
            SessionSocket::<MTU, _>::new(alice, AcknowledgementState::new("alice", ack_cfg), sock_cfg)?;
        let mut bob_socket = SessionSocket::<MTU, _>::new(bob, AcknowledgementState::new("bob", ack_cfg), sock_cfg)?;

        let alice_sent_data = hopr_crypto_random::random_bytes::<DATA_SIZE>();
        alice_socket.write_all(&alice_sent_data).await?;
        alice_socket.flush().await?;

        let bob_sent_data = hopr_crypto_random::random_bytes::<DATA_SIZE>();
        bob_socket.write_all(&bob_sent_data).await?;
        bob_socket.flush().await?;

        let mut bob_recv_data = [0u8; DATA_SIZE];
        bob_socket.read_exact(&mut bob_recv_data).await?;
        assert_eq!(alice_sent_data, bob_recv_data);

        let mut alice_recv_data = [0u8; DATA_SIZE];
        alice_socket.read_exact(&mut alice_recv_data).await?;
        assert_eq!(bob_sent_data, alice_recv_data);

        alice_socket.close().await?;
        bob_socket.close().await?;

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn stateless_socket_unidirectional_should_should_skip_missing_frames() -> anyhow::Result<()> {
        let (alice, bob) = setup_alice_bob::<MTU>(
            FaultyNetworkConfig {
                avg_delay: Duration::from_millis(10),
                ids_to_drop: HashSet::from_iter([0_usize]),
                ..Default::default()
            },
            None,
            None,
        );

        let alice_cfg = SessionSocketConfig {
            frame_size: FRAME_SIZE,
            ..Default::default()
        };

        let bob_cfg = SessionSocketConfig {
            frame_size: FRAME_SIZE,
            frame_timeout: Duration::from_millis(55),
            ..Default::default()
        };

        let mut alice_socket = SessionSocket::<MTU, _>::new_stateless("alice", alice, alice_cfg)?;
        let mut bob_socket = SessionSocket::<MTU, _>::new_stateless("bob", bob, bob_cfg)?;

        let data = hopr_crypto_random::random_bytes::<DATA_SIZE>();
        alice_socket.write_all(&data).await?;
        alice_socket.flush().await?;
        alice_socket.close().await?;

        let mut bob_data = Vec::with_capacity(DATA_SIZE);
        bob_socket.read_to_end(&mut bob_data).await?;

        // The whole first frame is discarded due to the missing first segment
        assert_eq!(data.len() - 1500, bob_data.len());
        assert_eq!(&data[1500..], &bob_data);

        bob_socket.close().await?;

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn stateful_socket_unidirectional_should_should_not_skip_missing_frames() -> anyhow::Result<()> {
        let (alice, bob) = setup_alice_bob::<MTU>(
            FaultyNetworkConfig {
                avg_delay: Duration::from_millis(10),
                ids_to_drop: HashSet::from_iter([0_usize]),
                ..Default::default()
            },
            None,
            None,
        );

        let alice_cfg = SessionSocketConfig {
            frame_size: FRAME_SIZE,
            ..Default::default()
        };

        let bob_cfg = SessionSocketConfig {
            frame_size: FRAME_SIZE,
            frame_timeout: Duration::from_millis(1000),
            ..Default::default()
        };

        let ack_cfg = AcknowledgementStateConfig {
            expected_packet_latency: Duration::from_millis(10),
            acknowledgement_delay: Duration::from_millis(40),
            ..Default::default()
        };

        let mut alice_socket =
            SessionSocket::<MTU, _>::new(alice, AcknowledgementState::new("alice", ack_cfg), alice_cfg)?;
        let mut bob_socket = SessionSocket::<MTU, _>::new(bob, AcknowledgementState::new("bob", ack_cfg), bob_cfg)?;

        let data = hopr_crypto_random::random_bytes::<DATA_SIZE>();
        alice_socket.write_all(&data).await?;
        alice_socket.flush().await?;

        let mut bob_data = [0u8; DATA_SIZE];
        tokio::time::timeout(Duration::from_secs(3), bob_socket.read_exact(&mut bob_data)).await??;
        assert_eq!(data, bob_data);

        bob_socket.close().await?;
        alice_socket.close().await?;

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn stateless_socket_bidirectional_should_should_skip_missing_frames() -> anyhow::Result<()> {
        let (alice, bob) = setup_alice_bob::<MTU>(
            FaultyNetworkConfig {
                avg_delay: Duration::from_millis(10),
                ids_to_drop: HashSet::from_iter([0_usize]),
                ..Default::default()
            },
            None,
            None,
        );

        let alice_cfg = SessionSocketConfig {
            frame_size: FRAME_SIZE,
            frame_timeout: Duration::from_millis(55),
            ..Default::default()
        };

        let bob_cfg = SessionSocketConfig {
            frame_size: FRAME_SIZE,
            frame_timeout: Duration::from_millis(55),
            ..Default::default()
        };

        let mut alice_socket = SessionSocket::<MTU, _>::new_stateless("alice", alice, alice_cfg)?;
        let mut bob_socket = SessionSocket::<MTU, _>::new_stateless("bob", bob, bob_cfg)?;

        let alice_sent_data = hopr_crypto_random::random_bytes::<DATA_SIZE>();
        alice_socket.write_all(&alice_sent_data).await?;
        alice_socket.flush().await?;

        let bob_sent_data = hopr_crypto_random::random_bytes::<DATA_SIZE>();
        bob_socket.write_all(&bob_sent_data).await?;
        bob_socket.flush().await?;

        alice_socket.close().await?;
        bob_socket.close().await?;

        let mut alice_recv_data = Vec::with_capacity(DATA_SIZE);
        alice_socket.read_to_end(&mut alice_recv_data).await?;

        let mut bob_recv_data = Vec::with_capacity(DATA_SIZE);
        bob_socket.read_to_end(&mut bob_recv_data).await?;

        // The whole first frame is discarded due to the missing first segment
        assert_eq!(bob_sent_data.len() - 1500, alice_recv_data.len());
        assert_eq!(&bob_sent_data[1500..], &alice_recv_data);

        assert_eq!(alice_sent_data.len() - 1500, bob_recv_data.len());
        assert_eq!(&alice_sent_data[1500..], &bob_recv_data);

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn stateful_socket_bidirectional_should_should_not_skip_missing_frames() -> anyhow::Result<()> {
        let (alice, bob) = setup_alice_bob::<MTU>(
            FaultyNetworkConfig {
                avg_delay: Duration::from_millis(10),
                ids_to_drop: HashSet::from_iter([0_usize]),
                ..Default::default()
            },
            None,
            None,
        );

        let alice_cfg = SessionSocketConfig {
            frame_size: FRAME_SIZE,
            frame_timeout: Duration::from_millis(1000),
            ..Default::default()
        };

        let bob_cfg = SessionSocketConfig {
            frame_size: FRAME_SIZE,
            frame_timeout: Duration::from_millis(1000),
            ..Default::default()
        };

        let ack_cfg = AcknowledgementStateConfig {
            expected_packet_latency: Duration::from_millis(10),
            acknowledgement_delay: Duration::from_millis(40),
            ..Default::default()
        };

        let mut alice_socket =
            SessionSocket::<MTU, _>::new(alice, AcknowledgementState::new("alice", ack_cfg), alice_cfg)?;
        let mut bob_socket = SessionSocket::<MTU, _>::new(bob, AcknowledgementState::new("bob", ack_cfg), bob_cfg)?;

        let alice_sent_data = hopr_crypto_random::random_bytes::<DATA_SIZE>();
        alice_socket.write_all(&alice_sent_data).await?;
        alice_socket.flush().await?;

        let bob_sent_data = hopr_crypto_random::random_bytes::<DATA_SIZE>();
        bob_socket.write_all(&bob_sent_data).await?;
        bob_socket.flush().await?;

        let mut bob_recv_data = [0u8; DATA_SIZE];
        bob_socket.read_exact(&mut bob_recv_data).await?;
        assert_eq!(alice_sent_data, bob_recv_data);

        let mut alice_recv_data = [0u8; DATA_SIZE];
        alice_socket.read_exact(&mut alice_recv_data).await?;
        assert_eq!(bob_sent_data, alice_recv_data);

        alice_socket.close().await?;
        bob_socket.close().await?;

        Ok(())
    }
}
