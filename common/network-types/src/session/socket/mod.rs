mod state;

use asynchronous_codec::Framed;
use futures::{pin_mut, FutureExt, SinkExt, TryStreamExt};
use futures::StreamExt;
use futures_concurrency::stream::Merge;
use state::SocketState;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

use crate::prelude::errors::SessionError;
use crate::prelude::{frame_reconstructor_with_inspector, FrameId, Segment};
use crate::prelude::protocol::{SessionCodec, SessionMessage};
use crate::session::segmenter::Segmenter;
use crate::session::socket::state::{SocketStateEvent, SocketStateEventDiscriminants};

pub struct SessionSocket<const C: usize> {
    // This is where upstream writes the to-be-segmented frame data to
    upstream_frames_in: Pin<Box<dyn futures::io::AsyncWrite + Send>>,
    // This is where upstream reads the reconstructed frame data from
    downstream_frames_out: Pin<Box<dyn futures::io::AsyncRead + Send>>,
}

/// Internal type that takes care of dispatching socket events into [`SocketState`].
#[derive(Debug, Clone)]
struct SocketEventManager<const C: usize> {
    frame_events_in: futures::channel::mpsc::UnboundedSender<SocketStateEvent<C>>,
    incoming_segment_events_in: futures::channel::mpsc::UnboundedSender<SocketStateEvent<C>>,
    outgoing_segment_events_in: futures::channel::mpsc::UnboundedSender<SocketStateEvent<C>>,
    message_events_in: futures::channel::mpsc::UnboundedSender<SocketStateEvent<C>>,
    session_id: String,
    subscribed_events: Vec<SocketStateEventDiscriminants>,
}

impl<const C: usize> SocketEventManager<C> {
    fn new<S>(state: S) -> Self
    where S: for<'a> SocketState<'a, C> + Send + 'static {

        let (frame_events_in, frame_events_out) = futures::channel::mpsc::unbounded();
        let (incoming_segment_events_in, incoming_segment_events_out) = futures::channel::mpsc::unbounded();
        let (outgoing_segment_events_in, outgoing_segment_events_out) = futures::channel::mpsc::unbounded();
        let (message_events_in, message_events_out) = futures::channel::mpsc::unbounded();

        let session_id = state.session_id().to_owned();
        let subscribed_events = state.event_subscriptions();

        // Start processing of events raised on the SocketEventManager to the SocketState
        // Merging the streams ensures yield-fairness to avoid starvation
        hopr_async_runtime::prelude::spawn((frame_events_out, incoming_segment_events_out, outgoing_segment_events_out,  message_events_out).merge().map(Ok).forward(state.into_sink()));

        Self {
            frame_events_in,
            incoming_segment_events_in,
            outgoing_segment_events_in,
            message_events_in,
            session_id,
            subscribed_events,
        }
    }

    async fn on_frame_received(&mut self, frame_id: FrameId) {
        if let Err(error) = self.frame_events_in.send(SocketStateEvent::FrameReceived(frame_id)).await {
            tracing::error!(session_id = self.session_id, %error, "cannot dispatch frame event to the state");
        }
    }

    async fn on_frame_discarded(&mut self, frame_id: FrameId) {
        if let Err(error) = self.frame_events_in.send(SocketStateEvent::FrameDiscarded(frame_id)).await {
            tracing::error!(session_id = self.session_id, %error, "cannot dispatch frame error event to the state");
        }
    }

    async fn on_segment_sent(&mut self, segment: &Segment) {
        if let Err(error) = self.outgoing_segment_events_in.send(SocketStateEvent::SegmentSent(segment.clone())).await {
            tracing::error!(session_id = self.session_id, %error, "failed to notify sent segment to the state");
        }
    }

    async fn on_message_received(&mut self, message: &SessionMessage<C>) {
        if let SessionMessage::Segment(_) = message {
            if let Err(error) = self.incoming_segment_events_in
                .send(SocketStateEvent::MessageReceived(message.clone()))
                .await
            {
                tracing::error!(session_id = self.session_id, %error, "cannot dispatch incoming segment to the state");
            }
        } else {
            if let Err(error) = self.message_events_in
                .send(SocketStateEvent::MessageReceived(message.clone()))
                .await
            {
                tracing::error!(session_id = self.session_id, %error, "cannot dispatch incoming message to the state");
            }
        }

    }
}

impl<const C: usize> SessionSocket<C> {
    pub fn new<T, S>(transport: T, mut state: S) -> Self
    where
        T: futures::io::AsyncRead + futures::io::AsyncWrite + Send + Unpin + 'static,
        S: for<'a> SocketState<'a, C> + Send + 'static,
    {
        // Downstream Segments get reconstructed into Frames
        let (downstream_segment_in, downstream_frames_out, frame_inspector) =
            frame_reconstructor_with_inspector(Duration::from_secs(10), 1024);

        // Upstream frames get segmented and are yielded by the data_rx stream
        let (upstream_frames_in, segmented_data_rx) = Segmenter::<C, 1500>::new(1024);

        // Downstream transport
        let (packets_out, packets_in) = Framed::new(transport, SessionCodec::<C>).split();

        let ctl_rx = state.control_stream();
        state.attach_frame_inspector(frame_inspector);

        let event_manager = SocketEventManager::new(state);

        // Messages incoming from Upstream and from the State go downstream as Packets
        // Segmented data coming from Upstream go out right away.
        let evt = event_manager.clone();
        let sid = event_manager.session_id.clone();
        hopr_async_runtime::prelude::spawn(
            (
                ctl_rx,
                segmented_data_rx
                    .then(move |s| {
                        let mut evt = evt.clone();
                        async move {
                            evt.on_segment_sent(&s).await;
                            SessionMessage::<C>::Segment(s)
                        }
                    }),
            )
            .merge()
            .map(Ok)
            .forward(packets_out)
            .map(move |result| {
                tracing::debug!(session_id = sid, ?result, "outgoing packet processing done");
            })
        );

        // Packets incoming from Downstream:
        // - if the State requires it, packets passed (cloned) into the State
        // - Packets that represent Segments (filtered out) are passed to the Reconstructor
        let evt = event_manager.clone();
        let sid = event_manager.session_id.clone();
        hopr_async_runtime::prelude::spawn(
            packets_in
                .try_filter_map(move |packet| {
                    let mut evt = evt.clone();
                    async move {
                        evt.on_message_received(&packet).await;
                        Ok(packet.try_as_segment())
                    }
                })
                .forward(downstream_segment_in)
                .map(move |result| {
                    tracing::debug!(session_id = sid, ?result, "incoming packet processing done");
                })
        );

        Self {
            upstream_frames_in: Box::pin(upstream_frames_in),
            downstream_frames_out: Box::pin(
                downstream_frames_out
                    .filter_map(move |maybe_frame| {
                        // Filter out discarded Frames and dispatch events to the State if needed
                        let mut evt = event_manager.clone();
                        async move {
                            match maybe_frame {
                                Ok(frame) => {
                                    evt.on_frame_received(frame.frame_id).await;
                                    Some(Ok(frame))
                                },
                                Err(SessionError::FrameDiscarded(frame_id)) | Err(SessionError::IncompleteFrame(frame_id)) => {
                                    evt.on_frame_discarded(frame_id).await;
                                    None // Downstream skips discarded frames
                                }
                                Err(err) => Some(Err(std::io::Error::other(err))),
                            }
                        }
                    })
                    .into_async_read(),
            )
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
