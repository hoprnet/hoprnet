//! This module defines the socket-like interface for Session protocol.

pub mod ack_state;
pub mod state;

use std::{
    pin::Pin,
    sync::{Arc, atomic::AtomicU32},
    task::{Context, Poll},
    time::Duration,
};

use futures::{FutureExt, SinkExt, StreamExt, TryStreamExt, future, future::AbortHandle};
use futures_concurrency::stream::Merge;
use state::SocketState;
use tracing::{Instrument, instrument};

use crate::{
    errors::SessionError,
    frames::{OrderedFrame, SeqIndicator},
    processing::{ReassemblerExt, SegmenterExt, SequencerExt, types::FrameInspector},
    protocol::{SegmentRequest, SessionCodec, SessionMessage},
    socket::state::{SocketComponents, Stateless},
};

/// Configuration object for [`SessionSocket`].
#[derive(Debug, Copy, Clone, Eq, PartialEq, smart_default::SmartDefault)]
pub struct SessionSocketConfig {
    /// The maximum size of a frame on the read/write interface of the [`SessionSocket`].
    ///
    /// The size is always greater or equal to the MTU `C` of the underlying transport, and
    /// less or equal to:
    /// - (`C` -  `SessionMessage::SEGMENT_OVERHEAD`) * (`SeqIndicator::MAX` + 1) for stateless sockets, or
    /// - (`C` - `SessionMessage::SEGMENT_OVERHEAD`) * min(`SeqIndicator::MAX` + 1,
    ///   `SegmentRequest::MAX_MISSING_SEGMENTS_PER_FRAME`) for stateful sockets
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

    /// Flushes data written to the socket immediately to the underlying transport.
    ///
    /// Default is false.
    #[default(false)]
    pub flush_immediately: bool,
}

enum WriteState {
    WriteOnly,
    Writing,
    Flushing(usize),
}

/// Socket-like object implementing the Session protocol that can operate on any transport that
/// implements [`futures::io::AsyncRead`] and [`futures::io::AsyncWrite`].
///
/// The [`SocketState`] `S` given during instantiation can facilitate reliable or unreliable
/// behavior (see [`AcknowledgementState`](ack_state::AcknowledgementState))
///
/// The constant argument `C` specifies the MTU in bytes of the underlying transport.
#[pin_project::pin_project]
pub struct SessionSocket<const C: usize, S> {
    // This is where upstream writes the to-be-segmented frame data to
    upstream_frames_in: Pin<Box<dyn futures::io::AsyncWrite + Send>>,
    // This is where upstream reads the reconstructed frame data from
    downstream_frames_out: Pin<Box<dyn futures::io::AsyncRead + Send>>,
    state: S,
    write_state: WriteState,
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
        // The maximum frame size in a stateless socket is only bounded by the size of the SeqIndicator
        let frame_size = cfg.frame_size.clamp(
            C,
            (C - SessionMessage::<C>::SEGMENT_OVERHEAD) * (SeqIndicator::MAX + 1) as usize,
        );

        // Segment data incoming/outgoing using underlying transport
        let mut framed = asynchronous_codec::Framed::new(transport, SessionCodec::<C>);

        // Check if we allow sending multiple segments to downstream in a single write
        // The HWM cannot be 0 bytes
        framed.set_send_high_water_mark(1.max(cfg.max_buffered_segments * C));

        // Downstream transport
        let (packets_out, packets_in) = framed.split();

        // Pipeline IN: Data incoming from Upstream
        let upstream_frames_in = packets_out
            .with(|segment| future::ok::<_, SessionError>(SessionMessage::<C>::Segment(segment)))
            .segmenter_with_terminating_segment::<C>(frame_size);

        let last_emitted_frame = Arc::new(AtomicU32::new(0));
        let last_emitted_frame_clone = last_emitted_frame.clone();

        let session_id_1 = id.to_string();
        let session_id_2 = id.to_string();
        let session_id_3 = id.to_string();

        let (packets_in_abort_handle, packets_in_abort_reg) = AbortHandle::new_pair();

        // Pipeline OUT: Packets incoming from Downstream
        // Continue receiving packets from downstream, unless we received a terminating frame.
        // Once the terminating frame is received, the `packets_in_abort_handle` is triggered, terminating the pipeline.
        let downstream_frames_out = futures::stream::Abortable::new(packets_in, packets_in_abort_reg)
            // Filter-out segments that we've seen already
            .filter_map(move |packet| {
                futures::future::ready(match packet {
                    Ok(packet) => packet
                        .try_as_segment()
                        .filter(|s| s.frame_id > last_emitted_frame.load(std::sync::atomic::Ordering::Relaxed)),
                    Err(error) => {
                        tracing::error!(%error, "unparseable packet");
                        None
                    }
                })
                .instrument(tracing::debug_span!(
                    "SessionSocket::packets_in::pre_reassembly",
                    session_id = session_id_1
                ))
            })
            // Reassemble the segments into frames
            .reassembler(cfg.frame_timeout, cfg.capacity)
            // Discard frames that we could not reassemble
            .filter_map(move |maybe_frame| {
                futures::future::ready(match maybe_frame {
                    Ok(frame) => Some(OrderedFrame(frame)),
                    Err(error) => {
                        tracing::error!(%error, "failed to reassemble frame");
                        None
                    }
                })
                .instrument(tracing::debug_span!(
                    "SessionSocket::packets_in::pre_sequencing",
                    session_id = session_id_2
                ))
            })
            // Put the frames into the correct sequence by Frame Ids
            .sequencer(cfg.frame_timeout, cfg.capacity)
            // Discard frames missing from the sequence
            .filter_map(move |maybe_frame| {
                future::ready(match maybe_frame {
                    Ok(frame) => {
                        last_emitted_frame_clone.store(frame.0.frame_id, std::sync::atomic::Ordering::Relaxed);
                        if frame.0.is_terminating {
                            tracing::warn!("terminating frame received");
                            packets_in_abort_handle.abort();
                        }
                        Some(Ok(frame.0))
                    }
                    // Downstream skips discarded frames
                    Err(SessionError::FrameDiscarded(frame_id)) | Err(SessionError::IncompleteFrame(frame_id)) => {
                        tracing::error!(frame_id, "frame discarded");
                        None
                    }
                    Err(err) => Some(Err(std::io::Error::other(err))),
                })
                .instrument(tracing::debug_span!(
                    "SessionSocket::packets_in::post_sequencing",
                    session_id = session_id_3
                ))
            })
            .into_async_read();

        Ok(Self {
            state: Stateless::new(id),
            upstream_frames_in: Box::pin(upstream_frames_in),
            downstream_frames_out: Box::pin(downstream_frames_out),
            write_state: if cfg.flush_immediately {
                WriteState::Writing
            } else {
                WriteState::WriteOnly
            },
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
        // The maximum frame size is reduced due to the size of the missing segment bitmap in SegmentRequests
        let frame_size = cfg.frame_size.clamp(
            C,
            (C - SessionMessage::<C>::SEGMENT_OVERHEAD)
                * SegmentRequest::<C>::MAX_MISSING_SEGMENTS_PER_FRAME.min((SeqIndicator::MAX + 1) as usize),
        );

        // Segment data incoming/outgoing using underlying transport
        let mut framed = asynchronous_codec::Framed::new(transport, SessionCodec::<C>);

        // Check if we allow sending multiple segments to downstream in a single write
        // The HWM cannot be 0 bytes
        framed.set_send_high_water_mark(1.max(cfg.max_buffered_segments * C));

        // Downstream transport
        let (packets_out, packets_in) = framed.split();

        let inspector = FrameInspector::new(cfg.capacity);

        let (ctl_tx, ctl_rx) = futures::channel::mpsc::unbounded();
        state.run(SocketComponents {
            inspector: Some(inspector.clone()),
            ctl_tx,
        })?;

        // Pipeline IN: Data incoming from Upstream
        let (segments_tx, segments_rx) = futures::channel::mpsc::channel(cfg.capacity);
        let mut st_1 = state.clone();
        let upstream_frames_in = segments_tx
            //.sink_map_err(|_| SessionError::InvalidSegment)
            .with(move |segment| {
                // The segment_sent event is raised only for segments coming from Upstream,
                // not for the segments from the Control stream (= segment resends).
                if let Err(error) = st_1.segment_sent(&segment) {
                    tracing::debug!(%error, "outgoing segment state update failed");
                }
                future::ok::<_, futures::channel::mpsc::SendError>(SessionMessage::<C>::Segment(segment))
            })
            .segmenter_with_terminating_segment::<C>(frame_size);

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

        let last_emitted_frame = Arc::new(AtomicU32::new(0));
        let last_emitted_frame_clone = last_emitted_frame.clone();

        let (packets_in_abort_handle, packets_in_abort_reg) = AbortHandle::new_pair();

        // Pipeline OUT: Packets incoming from Downstream
        let mut st_1 = state.clone();
        let mut st_2 = state.clone();
        let mut st_3 = state.clone();

        // Continue receiving packets from downstream, unless we received a terminating frame.
        // Once the terminating frame is received, the `packets_in_abort_handle` is triggered, terminating the pipeline.
        let downstream_frames_out = futures::stream::Abortable::new(packets_in, packets_in_abort_reg)
            // Filter out Session control messages and update the State, pass only Segments onwards
            .filter_map(move |packet| {
                futures::future::ready(match packet {
                    Ok(packet) => {
                        if let Err(error) = match &packet {
                            SessionMessage::Segment(s) => st_1.incoming_segment(&s.id(), s.seq_flags),
                            SessionMessage::Request(r) => st_1.incoming_retransmission_request(r.clone()),
                            SessionMessage::Acknowledge(a) => st_1.incoming_acknowledged_frames(a.clone()),
                        } {
                            tracing::debug!(%error, "incoming message state update failed");
                        }
                        // Filter old frame ids to save space in the Reassembler
                        packet
                            .try_as_segment()
                            .filter(|s| s.frame_id > last_emitted_frame.load(std::sync::atomic::Ordering::Relaxed))
                    }
                    Err(error) => {
                        tracing::error!(%error, "unparseable packet");
                        None
                    }
                })
                .instrument(tracing::debug_span!(
                    "SessionSocket::packets_in::pre_reassembly",
                    session_id = st_1.session_id()
                ))
            })
            // Reassemble segments into frames
            .reassembler_with_inspector(cfg.frame_timeout, cfg.capacity, inspector)
            // Notify State once a frame has been reassembled, discard frames that we could not reassemble
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
                .instrument(tracing::debug_span!(
                    "SessionSocket::packets_in::pre_sequencing",
                    session_id = st_2.session_id()
                ))
            })
            // Put the frames into the correct sequence by Frame Ids
            .sequencer(cfg.frame_timeout, cfg.frame_size)
            // Discard frames missing from the sequence and
            // notify the State about emitted or discarded frames
            .filter_map(move |maybe_frame| {
                // Filter out discarded Frames and dispatch events to the State if needed
                future::ready(match maybe_frame {
                    Ok(frame) => {
                        if let Err(error) = st_3.frame_emitted(frame.0.frame_id) {
                            tracing::error!(%error, "frame received state update failed");
                        }
                        last_emitted_frame_clone.store(frame.0.frame_id, std::sync::atomic::Ordering::Relaxed);
                        if frame.0.is_terminating {
                            tracing::warn!("terminating frame received");
                            packets_in_abort_handle.abort();
                        }
                        Some(Ok(frame.0))
                    }
                    Err(SessionError::FrameDiscarded(frame_id)) | Err(SessionError::IncompleteFrame(frame_id)) => {
                        if let Err(error) = st_3.frame_discarded(frame_id) {
                            tracing::error!(%error, "frame discarded state update failed");
                        }
                        None // Downstream skips discarded frames
                    }
                    Err(err) => Some(Err(std::io::Error::other(err))),
                })
                .instrument(tracing::debug_span!(
                    "SessionSocket::packets_in::post_sequencing",
                    session_id = st_3.session_id()
                ))
            })
            .into_async_read();

        Ok(Self {
            state,
            upstream_frames_in: Box::pin(upstream_frames_in),
            downstream_frames_out: Box::pin(downstream_frames_out),
            write_state: if cfg.flush_immediately {
                WriteState::Writing
            } else {
                WriteState::WriteOnly
            },
        })
    }
}

impl<const C: usize, S: SocketState<C> + Clone + 'static> futures::io::AsyncRead for SessionSocket<C, S> {
    #[instrument(name = "SessionSocket::poll_read", level = "trace", skip(self, cx, buf), fields(session_id = self.state.session_id(), len = buf.len()))]
    fn poll_read(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut [u8]) -> Poll<std::io::Result<usize>> {
        self.project().downstream_frames_out.as_mut().poll_read(cx, buf)
    }
}

impl<const C: usize, S: SocketState<C> + Clone + 'static> futures::io::AsyncWrite for SessionSocket<C, S> {
    #[instrument(name = "SessionSocket::poll_write", level = "trace", skip(self, cx, buf), fields(session_id = self.state.session_id(), len = buf.len()))]
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<std::io::Result<usize>> {
        let this = self.project();
        loop {
            match this.write_state {
                WriteState::WriteOnly => {
                    return this.upstream_frames_in.as_mut().poll_write(cx, buf);
                }
                WriteState::Writing => {
                    let len = futures::ready!(this.upstream_frames_in.as_mut().poll_write(cx, buf))?;
                    *this.write_state = WriteState::Flushing(len);
                }
                WriteState::Flushing(len) => {
                    let res = futures::ready!(this.upstream_frames_in.as_mut().poll_flush(cx)).map(|_| *len);
                    *this.write_state = WriteState::Writing;
                    return Poll::Ready(res);
                }
            }
        }
    }

    #[instrument(name = "SessionSocket::poll_flush", level = "trace", skip(self, cx), fields(session_id = self.state.session_id()))]
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        self.project().upstream_frames_in.as_mut().poll_flush(cx)
    }

    #[instrument(name = "SessionSocket::poll_close", level = "trace", skip(self, cx), fields(session_id = self.state.session_id()))]
    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        let this = self.project();
        let _ = this.state.stop();
        this.upstream_frames_in.as_mut().poll_close(cx)
    }
}

#[cfg(feature = "runtime-tokio")]
impl<const C: usize, S: SocketState<C> + Clone + 'static> tokio::io::AsyncRead for SessionSocket<C, S> {
    #[instrument(name = "SessionSocket::poll_read", level = "trace", skip(self, cx, buf), fields(session_id = self.state.session_id()))]
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        let slice = buf.initialize_unfilled();
        let n = std::task::ready!(futures::AsyncRead::poll_read(self.as_mut(), cx, slice))?;
        buf.advance(n);
        Poll::Ready(Ok(()))
    }
}

#[cfg(feature = "runtime-tokio")]
impl<const C: usize, S: SocketState<C> + Clone + 'static> tokio::io::AsyncWrite for SessionSocket<C, S> {
    #[instrument(name = "SessionSocket::poll_write", level = "trace", skip(self, cx, buf), fields(session_id = self.state.session_id(), len = buf.len()))]
    fn poll_write(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize, std::io::Error>> {
        futures::AsyncWrite::poll_write(self.as_mut(), cx, buf)
    }

    #[instrument(name = "SessionSocket::poll_flush", level = "trace", skip(self, cx), fields(session_id = self.state.session_id()))]
    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), std::io::Error>> {
        futures::AsyncWrite::poll_flush(self.as_mut(), cx)
    }

    #[instrument(name = "SessionSocket::poll_shutdown", level = "trace", skip(self, cx), fields(session_id = self.state.session_id()))]
    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), std::io::Error>> {
        futures::AsyncWrite::poll_close(self.as_mut(), cx)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use futures::{AsyncReadExt, AsyncWriteExt};
    use futures_time::future::FutureExt;
    use hopr_crypto_packet::prelude::HoprPacket;

    use super::*;
    use crate::{AcknowledgementState, AcknowledgementStateConfig, utils::test::*};

    const MTU: usize = HoprPacket::PAYLOAD_SIZE;

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

        alice_socket
            .write_all(&data)
            .timeout(futures_time::time::Duration::from_secs(2))
            .await??;
        alice_socket.flush().await?;

        let mut bob_data = [0u8; DATA_SIZE];
        bob_socket
            .read_exact(&mut bob_data)
            .timeout(futures_time::time::Duration::from_secs(2))
            .await??;
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

        alice_socket
            .write_all(&data)
            .timeout(futures_time::time::Duration::from_secs(2))
            .await??;
        alice_socket.flush().await?;

        let mut bob_data = [0u8; DATA_SIZE];
        bob_socket
            .read_exact(&mut bob_data)
            .timeout(futures_time::time::Duration::from_secs(2))
            .await??;
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
        alice_socket
            .write_all(&alice_sent_data)
            .timeout(futures_time::time::Duration::from_secs(2))
            .await??;
        alice_socket.flush().await?;

        let bob_sent_data = hopr_crypto_random::random_bytes::<DATA_SIZE>();
        bob_socket
            .write_all(&bob_sent_data)
            .timeout(futures_time::time::Duration::from_secs(2))
            .await??;
        bob_socket.flush().await?;

        let mut bob_recv_data = [0u8; DATA_SIZE];
        bob_socket
            .read_exact(&mut bob_recv_data)
            .timeout(futures_time::time::Duration::from_secs(2))
            .await??;
        assert_eq!(alice_sent_data, bob_recv_data);

        let mut alice_recv_data = [0u8; DATA_SIZE];
        alice_socket
            .read_exact(&mut alice_recv_data)
            .timeout(futures_time::time::Duration::from_secs(2))
            .await??;
        assert_eq!(bob_sent_data, alice_recv_data);

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn stateful_socket_bidirectional_should_work() -> anyhow::Result<()> {
        let (alice, bob) = setup_alice_bob::<MTU>(FaultyNetworkConfig::default(), None, None);

        // use hopr_network_types::capture::PcapIoExt;
        // let (alice, bob) = (alice.capture("alice.pcap"), bob.capture("bob.pcap"));

        let sock_cfg = SessionSocketConfig {
            frame_size: FRAME_SIZE,
            ..Default::default()
        };

        let ack_cfg = AcknowledgementStateConfig {
            expected_packet_latency: Duration::from_millis(2),
            acknowledgement_delay: Duration::from_millis(10),
            ..Default::default()
        };

        let mut alice_socket =
            SessionSocket::<MTU, _>::new(alice, AcknowledgementState::new("alice", ack_cfg), sock_cfg)?;
        let mut bob_socket = SessionSocket::<MTU, _>::new(bob, AcknowledgementState::new("bob", ack_cfg), sock_cfg)?;

        let alice_sent_data = hopr_crypto_random::random_bytes::<DATA_SIZE>();
        alice_socket
            .write_all(&alice_sent_data)
            .timeout(futures_time::time::Duration::from_secs(2))
            .await??;
        alice_socket.flush().await?;

        let bob_sent_data = hopr_crypto_random::random_bytes::<DATA_SIZE>();
        bob_socket
            .write_all(&bob_sent_data)
            .timeout(futures_time::time::Duration::from_secs(2))
            .await??;
        bob_socket.flush().await?;

        let mut bob_recv_data = [0u8; DATA_SIZE];
        bob_socket
            .read_exact(&mut bob_recv_data)
            .timeout(futures_time::time::Duration::from_secs(2))
            .await??;
        assert_eq!(alice_sent_data, bob_recv_data);

        let mut alice_recv_data = [0u8; DATA_SIZE];
        alice_socket
            .read_exact(&mut alice_recv_data)
            .timeout(futures_time::time::Duration::from_secs(2))
            .await??;
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
        alice_socket
            .write_all(&data)
            .timeout(futures_time::time::Duration::from_secs(2))
            .await??;
        alice_socket.flush().await?;

        let mut bob_recv_data = [0u8; DATA_SIZE];
        bob_socket
            .read_exact(&mut bob_recv_data)
            .timeout(futures_time::time::Duration::from_secs(2))
            .await??;
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
        alice_socket
            .write_all(&data)
            .timeout(futures_time::time::Duration::from_secs(2))
            .await??;
        alice_socket.flush().await?;

        let mut bob_recv_data = [0u8; DATA_SIZE];
        bob_socket
            .read_exact(&mut bob_recv_data)
            .timeout(futures_time::time::Duration::from_secs(2))
            .await??;
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
        alice_socket
            .write_all(&alice_sent_data)
            .timeout(futures_time::time::Duration::from_secs(2))
            .await??;
        alice_socket.flush().await?;

        let bob_sent_data = hopr_crypto_random::random_bytes::<DATA_SIZE>();
        bob_socket
            .write_all(&bob_sent_data)
            .timeout(futures_time::time::Duration::from_secs(2))
            .await??;
        bob_socket.flush().await?;

        let mut bob_recv_data = [0u8; DATA_SIZE];
        bob_socket
            .read_exact(&mut bob_recv_data)
            .timeout(futures_time::time::Duration::from_secs(2))
            .await??;
        assert_eq!(alice_sent_data, bob_recv_data);

        let mut alice_recv_data = [0u8; DATA_SIZE];
        alice_socket
            .read_exact(&mut alice_recv_data)
            .timeout(futures_time::time::Duration::from_secs(2))
            .await??;
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
        alice_socket
            .write_all(&alice_sent_data)
            .timeout(futures_time::time::Duration::from_secs(2))
            .await??;
        alice_socket.flush().await?;

        let bob_sent_data = hopr_crypto_random::random_bytes::<DATA_SIZE>();
        bob_socket
            .write_all(&bob_sent_data)
            .timeout(futures_time::time::Duration::from_secs(2))
            .await??;
        bob_socket.flush().await?;

        let mut bob_recv_data = [0u8; DATA_SIZE];
        bob_socket
            .read_exact(&mut bob_recv_data)
            .timeout(futures_time::time::Duration::from_secs(2))
            .await??;
        assert_eq!(alice_sent_data, bob_recv_data);

        let mut alice_recv_data = [0u8; DATA_SIZE];
        alice_socket
            .read_exact(&mut alice_recv_data)
            .timeout(futures_time::time::Duration::from_secs(2))
            .await??;
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
        alice_socket
            .write_all(&data)
            .timeout(futures_time::time::Duration::from_secs(2))
            .await??;
        alice_socket.flush().await?;
        alice_socket.close().await?;

        let mut bob_data = Vec::with_capacity(DATA_SIZE);
        bob_socket
            .read_to_end(&mut bob_data)
            .timeout(futures_time::time::Duration::from_secs(2))
            .await??;

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

        let alice_jh = tokio::spawn(async move {
            alice_socket
                .write_all(&data)
                .timeout(futures_time::time::Duration::from_secs(5))
                .await??;

            alice_socket.flush().await?;

            // Alice has to keep reading so that it is ready for retransmitting
            let mut vec = Vec::new();
            alice_socket.read_to_end(&mut vec).await?;
            alice_socket.close().await?;

            Ok::<_, std::io::Error>(vec)
        });

        let mut bob_data = [0u8; DATA_SIZE];
        bob_socket
            .read_exact(&mut bob_data)
            .timeout(futures_time::time::Duration::from_secs(5))
            .await??;
        assert_eq!(data, bob_data);

        bob_socket.close().await?;

        let alice_recv = alice_jh.await??;
        assert!(alice_recv.is_empty());

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
        alice_socket
            .write_all(&alice_sent_data)
            .timeout(futures_time::time::Duration::from_secs(2))
            .await??;
        alice_socket.flush().await?;

        let bob_sent_data = hopr_crypto_random::random_bytes::<DATA_SIZE>();
        bob_socket
            .write_all(&bob_sent_data)
            .timeout(futures_time::time::Duration::from_secs(2))
            .await??;
        bob_socket.flush().await?;

        alice_socket.close().await?;
        bob_socket.close().await?;

        let mut alice_recv_data = Vec::with_capacity(DATA_SIZE);
        alice_socket
            .read_to_end(&mut alice_recv_data)
            .timeout(futures_time::time::Duration::from_secs(2))
            .await??;

        let mut bob_recv_data = Vec::with_capacity(DATA_SIZE);
        bob_socket
            .read_to_end(&mut bob_recv_data)
            .timeout(futures_time::time::Duration::from_secs(2))
            .await??;

        // The whole first frame is discarded due to the missing first segment
        assert_eq!(bob_sent_data.len() - 1500, alice_recv_data.len());
        assert_eq!(&bob_sent_data[1500..], &alice_recv_data);

        assert_eq!(alice_sent_data.len() - 1500, bob_recv_data.len());
        assert_eq!(&alice_sent_data[1500..], &bob_recv_data);

        Ok(())
    }

    //#[test_log::test(tokio::test)]
    #[test_log::test(tokio::test(flavor = "multi_thread"))]
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

        // use hopr_network_types::capture::PcapIoExt;
        // let (alice, bob) = (alice.capture("alice.pcap"), bob.capture("bob.pcap"));

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

        let (mut alice_rx, mut alice_tx) =
            SessionSocket::<MTU, _>::new(alice, AcknowledgementState::new("alice", ack_cfg), alice_cfg)?.split();

        let (mut bob_rx, mut bob_tx) =
            SessionSocket::<MTU, _>::new(bob, AcknowledgementState::new("bob", ack_cfg), bob_cfg)?.split();

        let alice_sent_data = hopr_crypto_random::random_bytes::<DATA_SIZE>();
        let (alice_data_tx, alice_recv_data) = futures::channel::oneshot::channel();
        let alice_rx_jh = tokio::spawn(async move {
            let mut alice_recv_data = vec![0u8; DATA_SIZE];
            alice_rx.read_exact(&mut alice_recv_data).await?;
            alice_data_tx
                .send(alice_recv_data)
                .map_err(|_| std::io::Error::other("tx error"))?;

            // Keep reading until the socket is closed
            alice_rx.read_to_end(&mut Vec::new()).await?;
            Ok::<_, std::io::Error>(())
        });

        let bob_sent_data = hopr_crypto_random::random_bytes::<DATA_SIZE>();
        let (bob_data_tx, bob_recv_data) = futures::channel::oneshot::channel();
        let bob_rx_jh = tokio::spawn(async move {
            let mut bob_recv_data = vec![0u8; DATA_SIZE];
            bob_rx.read_exact(&mut bob_recv_data).await?;
            bob_data_tx
                .send(bob_recv_data)
                .map_err(|_| std::io::Error::other("tx error"))?;

            // Keep reading until the socket is closed
            bob_rx.read_to_end(&mut Vec::new()).await?;
            Ok::<_, std::io::Error>(())
        });

        let alice_tx_jh = tokio::spawn(async move {
            alice_tx
                .write_all(&alice_sent_data)
                .timeout(futures_time::time::Duration::from_secs(2))
                .await??;
            alice_tx.flush().await?;

            // Once all data is sent, wait for the other side to receive it and close the socket
            let out = alice_recv_data.await.map_err(|_| std::io::Error::other("rx error"))?;
            alice_tx.close().await?;
            tracing::info!("alice closed");
            Ok::<_, std::io::Error>(out)
        });

        let bob_tx_jh = tokio::spawn(async move {
            bob_tx
                .write_all(&bob_sent_data)
                .timeout(futures_time::time::Duration::from_secs(2))
                .await??;
            bob_tx.flush().await?;

            // Once all data is sent, wait for the other side to receive it and close the socket
            let out = bob_recv_data.await.map_err(|_| std::io::Error::other("rx error"))?;
            bob_tx.close().await?;
            tracing::info!("bob closed");
            Ok::<_, std::io::Error>(out)
        });

        let (alice_recv_data, bob_recv_data, a, b) =
            futures::future::try_join4(alice_tx_jh, bob_tx_jh, alice_rx_jh, bob_rx_jh)
                .timeout(futures_time::time::Duration::from_secs(4))
                .await??;

        assert_eq!(&alice_sent_data, bob_recv_data?.as_slice());
        assert_eq!(&bob_sent_data, alice_recv_data?.as_slice());
        assert!(a.is_ok());
        assert!(b.is_ok());

        Ok(())
    }
}
