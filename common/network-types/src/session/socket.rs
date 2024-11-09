/*use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Duration;
use asynchronous_codec::Framed;
use futures::{pin_mut, Sink, StreamExt, TryStreamExt};
use futures_concurrency::stream::Merge;
use crate::prelude::errors::SessionError;
use crate::prelude::{frame_reconstructor, Frame};
use crate::prelude::protocol::{SessionCodec, SessionMessage};

#[derive(Clone)]
struct SocketState<const C: usize> {
    data_tx: futures::channel::mpsc::Sender<SessionMessage<C>>,
    ctl_tx: futures::channel::mpsc::Sender<SessionMessage<C>>,
    seg_in: Arc<dyn Sink<SessionMessage<C>, Error = SessionError> + Send>,
}

pub struct SessionSocket<const C: usize> {
    state: SocketState<C>,
    frames_out: Box<dyn futures::io::AsyncRead + Send + Unpin>
}

impl<const C: usize> SessionSocket<C> {
    pub fn new<T, I>(id: I, transport: T) -> Self
    where T: futures::io::AsyncRead + futures::io::AsyncWrite + Send + 'static{
        let (segment_in, frames_out) = frame_reconstructor(Duration::from_secs(10), 1024);

        let (packets_out, packets_in) = Framed::new(transport, SessionCodec::<C>).split();

        let (data_tx, data_rx) = futures::channel::mpsc::unbounded::<SessionMessage<C>>();
        let (ctl_tx, ctl_rx) = futures::channel::mpsc::unbounded::<SessionMessage<C>>();

        let state = SocketState {
            data_tx,
            ctl_tx,
            seg_in: Arc::new(segment_in)
        };

        // Data coming out from the `state` go downstream as packets
        hopr_async_runtime::prelude::spawn(async move {
            (ctl_rx, data_rx).merge().forward(packets_out)
        });

        // Data coming in from downstream go into the `state`
        let state_clone = state.clone();
        hopr_async_runtime::prelude::spawn(async move {
            packets_in.forward(state_clone);
        });

        // Frames coming out from the Reconstructor can be read upstream
        let frames_out = Box::new(frames_out
            .filter_map(move |maybe_frame| {
                // TODO
                match maybe_frame {
                    Ok(frame) => {
                        futures::future::ready(Some(Ok(frame)))
                    }
                    Err(err) => {
                        futures::future::ready(Some(Err(std::io::Error::other(err))))
                    }
                }
            }).into_async_read());

        Self { state, frames_out }
    }

}

impl<const C: usize> futures::io::AsyncRead for SessionSocket<C> {
    fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut [u8]) -> Poll<std::io::Result<usize>> {
        let inner = &mut self.frames_out;
        pin_mut!(inner);
        inner.poll_read(cx, buf)
    }
}

impl<const C: usize> futures::io::AsyncWrite for SessionSocket<C> {
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<std::io::Result<usize>> {
        todo!()
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        todo!()
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        todo!()
    }
}*/