use crate::connection::StreamCommand;
use crate::frame::Frame;
use crate::tagged_stream::TaggedStream;
use crate::Result;
use crate::{frame, StreamId};
use futures::channel::mpsc;
use futures::stream::{Fuse, SelectAll};
use futures::{ready, AsyncRead, AsyncWrite, SinkExt, StreamExt};
use std::collections::VecDeque;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

/// A [`Future`] that gracefully closes the yamux connection.
#[must_use]
pub struct Closing<T> {
    state: State,
    stream_receivers: SelectAll<TaggedStream<StreamId, mpsc::Receiver<StreamCommand>>>,
    pending_frames: VecDeque<Frame<()>>,
    socket: Fuse<frame::Io<T>>,
}

impl<T> Closing<T>
where
    T: AsyncRead + AsyncWrite + Unpin,
{
    pub(crate) fn new(
        stream_receivers: SelectAll<TaggedStream<StreamId, mpsc::Receiver<StreamCommand>>>,
        pending_frames: VecDeque<Frame<()>>,
        socket: Fuse<frame::Io<T>>,
    ) -> Self {
        Self {
            state: State::ClosingStreamReceiver,
            stream_receivers,
            pending_frames,
            socket,
        }
    }
}

impl<T> Future for Closing<T>
where
    T: AsyncRead + AsyncWrite + Unpin,
{
    type Output = Result<()>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();

        loop {
            match this.state {
                State::ClosingStreamReceiver => {
                    for stream in this.stream_receivers.iter_mut() {
                        stream.inner_mut().close();
                    }
                    this.state = State::DrainingStreamReceiver;
                }

                State::DrainingStreamReceiver => {
                    match this.stream_receivers.poll_next_unpin(cx) {
                        Poll::Ready(Some((_, Some(StreamCommand::SendFrame(frame))))) => {
                            this.pending_frames.push_back(frame.into())
                        }
                        Poll::Ready(Some((id, Some(StreamCommand::CloseStream { ack })))) => {
                            this.pending_frames
                                .push_back(Frame::close_stream(id, ack).into());
                        }
                        Poll::Ready(Some((_, None))) => {}
                        Poll::Pending | Poll::Ready(None) => {
                            // No more frames from streams, append `Term` frame and flush them all.
                            this.pending_frames.push_back(Frame::term().into());
                            this.state = State::FlushingPendingFrames;
                            continue;
                        }
                    }
                }
                State::FlushingPendingFrames => {
                    ready!(this.socket.poll_ready_unpin(cx))?;

                    match this.pending_frames.pop_front() {
                        Some(frame) => this.socket.start_send_unpin(frame)?,
                        None => this.state = State::ClosingSocket,
                    }
                }
                State::ClosingSocket => {
                    ready!(this.socket.poll_close_unpin(cx))?;

                    return Poll::Ready(Ok(()));
                }
            }
        }
    }
}

enum State {
    ClosingStreamReceiver,
    DrainingStreamReceiver,
    FlushingPendingFrames,
    ClosingSocket,
}
