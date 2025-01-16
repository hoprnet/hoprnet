mod state;

use std::cmp::Ordering;
use asynchronous_codec::Framed;
use futures::{pin_mut, Sink, SinkExt, StreamExt, TryStreamExt};
use futures_concurrency::stream::Merge;
use governor::prelude::StreamRateLimitExt;
use governor::{Quota, RateLimiter};
use std::num::NonZeroU32;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;
use futures::stream::BoxStream;
use crate::prelude::errors::SessionError;
use crate::prelude::protocol::{FrameAcknowledgements, SessionCodec, SessionMessage};
use crate::prelude::{frame_reconstructor_with_inspector, FrameId, Segment, SegmentId};
use crate::session::frame::SeqNum;
use crate::session::protocol::SegmentRequest;
use crate::session::segmenter::Segmenter;
use crate::session::utils::{offloaded_ringbuffer, OffloadedRbConsumer};
use crate::session::utils::skip_queue::{skip_delay_channel, DelayedItem, Skip, SkipDelaySender};


#[derive(Debug, Copy, Clone, Eq)]
struct RetriedFrameId {
    pub frame_id: FrameId,
    pub retry_count: u32,
}

impl From<FrameId> for RetriedFrameId {
    fn from(value: FrameId) -> Self {
        Self::new(value)
    }
}

impl RetriedFrameId {
    pub fn new(frame_id: FrameId) -> Self {
        Self {
            frame_id,
            retry_count: 0,
        }
    }
    pub fn next(self, max_retries: u32) -> Option<Self> {
        if self.retry_count < max_retries {
            Some(Self {
                frame_id: self.frame_id,
                retry_count: self.retry_count + 1,
            })
        } else {
            None
        }
    }

    pub fn at(&self) -> std::time::Instant {
        // TODO: add backoff impl here
        std::time::Instant::now()
    }
}

impl PartialEq<Self> for RetriedFrameId {
    fn eq(&self, other: &Self) -> bool {
        self.frame_id.eq(&other.frame_id)
    }
}

impl PartialOrd<Self> for RetriedFrameId {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for RetriedFrameId {
    fn cmp(&self, other: &Self) -> Ordering {
        self.frame_id.cmp(&other.frame_id)
    }
}

struct SocketStateImpl<const C: usize> {
    rb: OffloadedRbConsumer<Segment>,
    ctl_tx: futures::channel::mpsc::UnboundedSender<SessionMessage<C>>,
    downstream_segment_in: Pin<Box<dyn Sink<Segment, Error = SessionError> + Send>>,
    incoming_frame_retries_tx: SkipDelaySender<RetriedFrameId>,
    outgoing_frame_retries_tx: SkipDelaySender<RetriedFrameId>,
}

impl<const C: usize> SocketStateImpl<C> {
    async fn incoming_segment(&mut self, segment: Segment) -> Result<(), SessionError> {
        let id = segment.id();
        tracing::trace!(%id, "RECEIVED: segment");



        // TODO: condition this below only when retransmission is enabled

        // Register future requesting of segments for this frame
        let first_request = Duration::from_secs(1); // TODO: take this from the config
        if let Err(error) = self.incoming_frame_retries_tx.send_one((id.0.into(), first_request)) {
            tracing::error!(frame_id = id.0, %error, "failed to register incoming retry for frame");
        }

        Ok(())
    }

    async fn incoming_acknowledged_frames(&mut self, ack: FrameAcknowledgements<C>) -> Result<(), SessionError> {
        // Frame acknowledged, we will not need to resend it
        if let Err(error)= self.outgoing_frame_retries_tx.send_many(ack
                .into_iter()
                .inspect(|frame_id| tracing::trace!(frame_id, "frame acknowledged"))
                .map(|frame_id| DelayedItem::Cancel(RetriedFrameId::new(frame_id)))) {
            tracing::error!(%error, "failed to cancel frame resend");
        }

        Ok(())
    }

    async fn incoming_retransmission_request(&mut self, retransmit: SegmentRequest<C>) -> Result<(), SessionError> {
        let missing = retransmit.into_iter().collect::<Vec<_>>();

        // Perform a single find to lock the RB only once
        let segments = self.rb.find(|s| missing.contains(&s.id()));

        // Partially acknowledged frames will not need to be fully resent in the future
        if let Err(error) = self.outgoing_frame_retries_tx
            .send_many(missing.into_iter().map(|seg_id| (RetriedFrameId::new(seg_id.0), Duration::from_secs(1)).into())) {
            tracing::error!(%error, "failed to cancel frame resend of partially acknowledged frames");
        }

        self.downstream_segment_in
            .send_all(&mut futures::stream::iter(segments).map(Ok))
            .await
    }

    pub async fn handle_incoming_message(&mut self, message: SessionMessage<C>) -> Result<(), SessionError> {
        match message {
            SessionMessage::Segment(s) => self.incoming_segment(s).await,
            SessionMessage::Request(r) => self.incoming_retransmission_request(r).await,
            SessionMessage::Acknowledge(a) => self.incoming_acknowledged_frames(a).await,
        }
    }
}

pub struct SessionSocket<const C: usize> {
    session_id: String,
    // This is where upstream writes frame data to
    upstream_frames_in: Pin<Box<dyn futures::io::AsyncWrite + Send>>,
    // This is where upstream reads frame data from
    downstream_frames_out: Pin<Box<dyn futures::io::AsyncRead + Send>>,
}

/// Abstraction of the [SessionSocket] state.
pub trait SocketState<'a, const C: usize>: Clone + Send + Sync {
    /// Called when the Socket receives a new segment from Downstream.
    fn incoming_segment(&mut self, id: &SegmentId, segment_count: SeqNum) -> Result<(), SessionError>;

    /// Called when [segment retransmission request](SegmentRequest) is received from Downstream.
    fn incoming_retransmission_request(&mut self, request: SegmentRequest<C>) -> Result<(), SessionError>;

    /// Called when an [acknowledgement of frames](FrameAcknowledgements) is received from Downstream.
    fn incoming_acknowledged_frames(&mut self, ack: FrameAcknowledgements<C>) -> Result<(), SessionError>;

    /// Called when a complete Frame has been finalized from segments received from Downstream.
    fn frame_received(&mut self, id: FrameId) -> Result<(), SessionError>;

    /// Called when a frame could not be completed from the segments received from Downstream.
    fn frame_discarded(&mut self, id: FrameId) -> Result<(), SessionError>;

    /// Called when a segment of a Frame was sent to the Downstream.
    fn segment_sent(&mut self, segment: &Segment) -> Result<(), SessionError>;

    fn control_message_stream(&self) -> Option<BoxStream<'a, SessionMessage<C>>>;
}


impl<const C: usize> SessionSocket<C> {
    pub fn new<T, I, S>(id: I, transport: T, state: S) -> Self
    where
        T: futures::io::AsyncRead + futures::io::AsyncWrite + Send + Unpin + 'static,
        I: std::fmt::Display,
        S: for<'a> SocketState<'a, C> + 'static
    {
        // Downstream Segments get reconstructed into Frames
        let (mut downstream_segment_in, downstream_frames_out, frame_inspector) = frame_reconstructor_with_inspector(Duration::from_secs(10), 1024);

        // Upstream frames get segmented and are yielded by the data_rx stream
        let (upstream_frames_in, segmented_data_rx) = Segmenter::<C, 1500>::new(1024);

        // Control messages (resend requests, acknowledgements) are generated by the Socket
        let (ctl_tx, ctl_rx) = futures::channel::mpsc::unbounded::<SessionMessage<C>>();

        // Acknowledgements get a special channel with fixed capacity
        //let (mut ack_tx, ack_rx) = futures::channel::mpsc::channel(200_000);

        //let (incoming_frame_retries_tx, incoming_frame_retries_rx) = skip_delay_channel();
        //let (outgoing_frame_retries_tx, outgoing_frame_retries_rx) = skip_delay_channel();

        // Frames coming out from the Reconstructor can be read Upstream
        let session_id = id.to_string();
        let mut state_clone = state.clone();
        let downstream_frames_out = Box::pin(
            downstream_frames_out
                .filter_map(move |maybe_frame| {
                    match maybe_frame {
                        Ok(frame) => {
                            tracing::trace!(session_id, frame_id = frame.frame_id, "frame complete");
                            if let Err(error) = state_clone.frame_received(frame.frame_id) {
                                tracing::error!(session_id, frame_id = frame.frame_id, %error, "failed to notify frame retrieval");
                            }
                            futures::future::ready(Some(Ok(frame)))
                        },
                        Err(SessionError::FrameDiscarded(frame_id)) | Err(SessionError::IncompleteFrame(frame_id)) => {
                            tracing::warn!(session_id, frame_id, "frame discarded");
                            if let Err(error) = state_clone.frame_discarded(frame_id) {
                                tracing::error!(session_id, frame_id, %error, "failed to notify frame retrieval");
                            }

                            // Downstream skips discarded frames
                            futures::future::ready(None)
                        }
                        Err(err) => {
                            futures::future::ready(Some(Err(std::io::Error::other(err))))
                        },
                    }
                })
                .into_async_read(),
        );

        let (packets_out, packets_in) =
            StreamExt::split::<SessionMessage<C>>(Framed::new(transport, SessionCodec::<C>));

        //let (rb_tx, rb_rx) = offloaded_ringbuffer(1024);

        // Messages incoming from Upstream and from the State go downstream as Packets
        let mut state_clone = state.clone();
        let session_id = id.to_string();
        let ctl_rx = state.control_message_stream().unwrap_or(futures::stream::empty().boxed());
        hopr_async_runtime::prelude::spawn(
            (
                ctl_rx, // TODO: refactor
                segmented_data_rx
                    .map(move |s| {
                        if let Err(error) = state_clone.segment_sent(&s) {
                            tracing::error!(session_id, %error, "failed to notify sent segment to the state");
                        }
                        SessionMessage::<C>::Segment(s)
                    }),
            )
            .merge()
            .map(Ok)
            .forward(packets_out),
        );

        // Forward acknowledged frames chunked as a Control messages
        /*let ctl_tx_clone = ctl_tx.clone();
        let id = session_id.clone();
        hopr_async_runtime::prelude::spawn(async move {
            // TODO: chunk size and rate limit should be configurable
            let ack_rate_limiter = RateLimiter::direct(Quota::per_second(NonZeroU32::MIN));
            if let Err(error) = ack_rx
                .ready_chunks(FrameAcknowledgements::<C>::MAX_ACK_FRAMES)
                .map(|v| Ok(SessionMessage::Acknowledge(v.into())))
                .ratelimit_stream(&ack_rate_limiter)
                .forward(ctl_tx_clone)
                .await
            {
                tracing::error!(session_id = id, %error, "acknowledgement forwarding failed");
            } else {
                tracing::trace!(session_id = id, "acknowledgement forwarding done");
            }
        });

        // Outgoing frame resends (that haven't been acknowledged by the counterparty)
        let rb_rx_clone = rb_rx.clone();
        let mut outgoing_frame_retries_tx_clone = outgoing_frame_retries_tx.clone();
        let ctl_tx_clone = ctl_tx.clone();
        hopr_async_runtime::prelude::spawn(async move {
           if let Err(error) = outgoing_frame_retries_rx
               .map(move |rf| {
                   // TODO: take frame max_retries from the config
                   // Find out if the frame can be retried
                   let frame_id = rf.frame_id;
                   if let Some(next) = rf.next(5) {
                       // Register the next retry if still possible
                       let retry_at = next.at();
                       if let Err(error) = outgoing_frame_retries_tx_clone.send_one((next, retry_at)) {
                           tracing::error!(frame_id, %error, "failed to register next retry of frame");
                       } else {
                           tracing::debug!(frame_id, ?retry_at, "next resend of outgoing frame");
                       }
                   }
                   frame_id
               })
               .flat_map(move |frame_id|
                   // Find out all the segments of that frame to be retransmitted
                   futures::stream::iter(rb_rx_clone.find(|s| s.id().0 == frame_id))
                   .map(|s| Ok(SessionMessage::<C>::Segment(s)))
               )
               .forward(ctl_tx_clone) // Retransmit all the segments
               .await {
               tracing::error!(%error, "error while processing outgoing frame retries");
           } else {
               tracing::trace!("outgoing frame retries processing done");
           }
        });

        // Incoming segment resend requests (that are incomplete for too long)
        let mut incoming_frame_retries_tx_clone = incoming_frame_retries_tx.clone();
        let ctl_tx_clone = ctl_tx.clone();
        hopr_async_runtime::prelude::spawn(async move {
            if let Err(error) = incoming_frame_retries_rx
                .map(|rf| {
                    let frame_id = rf.frame_id;
                    // TODO: take frame max_retries from the config
                    // Find out if the frame's segments should be requested
                    if let Some(next) = rf.next(5) {
                        // Register the next retry if still possible
                        let retry_at = next.at();
                        if let Err(error) = incoming_frame_retries_tx_clone.send_one((next, retry_at)) {
                            tracing::error!(frame_id, %error, "failed to register next resend of incoming frame");
                        } else {
                            tracing::debug!(frame_id, ?retry_at, "next resend request of incoming frame segments");
                        }
                    }
                    (frame_id, frame_inspector.missing_segments(&frame_id).unwrap_or_default())
                })
                .ready_chunks(SegmentRequest::<C>::MAX_ENTRIES)
                .map(|a| Ok(SessionMessage::<C>::Request(a.into_iter().collect())))
                .forward(ctl_tx_clone)
                .await {
                tracing::error!(%error, "error while processing incoming frame resends");
            } else {
                tracing::trace!("incoming segment resends processing done");
            }
        });

        let mut state = SocketStateImpl {
            rb: rb_rx,
            ctl_tx,
            downstream_segment_in: Box::pin(downstream_segment_in),
            incoming_frame_retries_tx,
            outgoing_frame_retries_tx,
        };*/

        // Packets incoming from Downstream
        let session_id = id.to_string();
        let state_clone = state.clone();
        hopr_async_runtime::prelude::spawn({
            let mut state_clone = state_clone.clone();
            async move {
                if let Err(error) = packets_in
                    // TODO
                    /*.then(|maybe_packet| async {
                        if let Ok(SessionMessage::Segment(s)) = maybe_packet {
                            downstream_segment_in
                                .send(s.clone())
                                .await
                                .map_err(|e| SessionError::ProcessingError(e.to_string()))?;
                            Ok(SessionMessage::Segment(s))
                        } else {
                            maybe_packet
                        }
                    })*/
                    .try_for_each_concurrent(Some(10), |maybe_packet| {
                        futures::future::ready(match maybe_packet {
                            SessionMessage::Segment(s) => state_clone.incoming_segment(&s.id(), s.seq_len),
                            SessionMessage::Request(r) => state_clone.incoming_retransmission_request(r),
                            SessionMessage::Acknowledge(a) => state_clone.incoming_acknowledged_frames(a),
                        })
                    })
                    .await {
                    tracing::error!(session_id, %error, "downstream packet processing completed with error");
                } else {
                    tracing::debug!(session_id, "incoming downstream completed");
                }
            }
        });

        Self {
            session_id: id.to_string(),
            upstream_frames_in: Box::pin(upstream_frames_in),
            downstream_frames_out,
        }
    }
}

impl<const C: usize> futures::io::AsyncRead for SessionSocket<C> {
    fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut [u8]) -> Poll<std::io::Result<usize>> {
        let inner = &mut self.downstream_frames_out;
        pin_mut!(inner);
        inner.poll_read(cx, buf)
    }
}

impl<const C: usize> futures::io::AsyncWrite for SessionSocket<C> {
    fn poll_write(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<std::io::Result<usize>> {
        let inner = &mut self.upstream_frames_in;
        pin_mut!(inner);
        inner.poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        let inner = &mut self.upstream_frames_in;
        pin_mut!(inner);
        inner.poll_flush(cx)
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        let inner = &mut self.upstream_frames_in;
        pin_mut!(inner);
        inner.poll_close(cx)
    }
}
