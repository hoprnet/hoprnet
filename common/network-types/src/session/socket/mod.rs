mod state;

use futures::StreamExt;
use asynchronous_codec::Framed;
use futures::{pin_mut, SinkExt, TryStreamExt};
use futures_concurrency::stream::Merge;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;
use state::SocketState;

use crate::prelude::errors::SessionError;
use crate::prelude::protocol::{SessionCodec, SessionMessage};
use crate::prelude::frame_reconstructor_with_inspector;
use crate::session::segmenter::Segmenter;
use crate::session::socket::state::{SocketStateEvent, SocketStateQueue, StateManager};


pub struct SessionSocket<const C: usize> {
    // This is where upstream writes frame data to
    upstream_frames_in: Pin<Box<dyn futures::io::AsyncWrite + Send>>,
    // This is where upstream reads frame data from
    downstream_frames_out: Pin<Box<dyn futures::io::AsyncRead + Send>>,
}


impl<const C: usize> SessionSocket<C> {
    pub fn new<T, S>(transport: T, state: S) -> Self
    where
        T: futures::io::AsyncRead + futures::io::AsyncWrite + Send + Unpin + 'static,
        S: for<'a> SocketState<'a, C> + Send + Sync + 'static, <<S as SocketStateQueue<C>>::EventsIn as futures::Sink<SocketStateEvent<C>>>::Error: std::fmt::Display
    {
        let sid = state.session_id().to_owned();

        // Downstream Segments get reconstructed into Frames
        let (downstream_segment_in, downstream_frames_out, frame_inspector) = frame_reconstructor_with_inspector(Duration::from_secs(10), 1024);

        // Upstream frames get segmented and are yielded by the data_rx stream
        let (upstream_frames_in, segmented_data_rx) = Segmenter::<C, 1500>::new(1024);

        let mut state_mgr = StateManager::new(state);

        // Frames coming out from the Reconstructor can be read Upstream
        let state_events = state_mgr.state_events();
        let session_id = sid.clone();
        let downstream_frames_out = Box::pin(
            downstream_frames_out
                .filter_map(move |maybe_frame| {
                    let mut state_events = state_events.clone();
                    let session_id = session_id.clone();
                    async move {
                        match maybe_frame {
                            Ok(frame) => {
                                if let Err(error) = state_events.send(SocketStateEvent::FrameReceived(frame.frame_id)).await {
                                    tracing::error!(session_id, %error, "cannot dispatch frame event to the state");
                                }
                                Some(Ok(frame))
                            },
                            Err(SessionError::FrameDiscarded(frame_id)) | Err(SessionError::IncompleteFrame(frame_id)) => {
                                if let Err(error) = state_events.send(SocketStateEvent::FrameDiscarded(frame_id)).await {
                                    tracing::error!(session_id, %error, "cannot dispatch frame error event to the state");
                                }
                                None // Downstream skips discarded frames
                            }
                            Err(err) => {
                                Some(Err(std::io::Error::other(err)))
                            },
                        }
                    }
                })
                .into_async_read(),
        );

        let (packets_out, packets_in) =
            StreamExt::split::<SessionMessage<C>>(Framed::new(transport, SessionCodec::<C>));

        // Messages incoming from Upstream and from the State go downstream as Packets
        let ctl_rx = state_mgr.control_stream().unwrap_or(futures::stream::empty().boxed());
        let state_events = state_mgr.state_events();
        let session_id = sid.clone();
        hopr_async_runtime::prelude::spawn(
            (
                ctl_rx, // TODO: refactor
                segmented_data_rx
                    .then(move |s| {
                        let mut state_events = state_events.clone();
                        let session_id = session_id.clone();
                        async move {
                            if let Err(error) = state_events.send(SocketStateEvent::SegmentSent(s.clone())).await {
                                tracing::error!(session_id, %error, "failed to notify sent segment to the state");
                            }
                            SessionMessage::<C>::Segment(s)
                        }
                    }),
            )
            .merge()
            .map(Ok)
            .forward(packets_out),
        );

        // Packets incoming from Downstream
        let state_events = state_mgr.state_events();
        let session_id = sid.clone();
        hopr_async_runtime::prelude::spawn(
            packets_in
                .try_filter_map(move |packet| {
                    let mut state_events = state_events.clone();
                    let session_id = session_id.clone();
                    async move {
                        if let Err(error) = state_events
                            .send(SocketStateEvent::MessageReceived(packet.clone())).await {
                            tracing::error!(session_id, %error, "cannot dispatch incoming message to the state");
                            Err(SessionError::ProcessingError(error.to_string()))
                        } else {
                            if let SessionMessage::Segment(s) = packet {
                                Ok(Some(s))
                            } else {
                                Ok(None)
                            }
                        }
                    }
                })
                .forward(downstream_segment_in)
        );

        Self {
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
