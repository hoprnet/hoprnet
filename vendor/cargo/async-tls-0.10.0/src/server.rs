//! The server end of a TLS connection.

use crate::common::tls_state::TlsState;
use crate::rusttls::stream::Stream;

use futures_core::ready;
use futures_io::{AsyncRead, AsyncWrite};
use rustls::ServerSession;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::{io, mem};

use rustls::Session;

/// The server end of a TLS connection. Can be used like any other bidirectional IO stream.
/// Wraps the underlying TCP stream.
#[derive(Debug)]
pub struct TlsStream<IO> {
    pub(crate) io: IO,
    pub(crate) session: ServerSession,
    pub(crate) state: TlsState,
}

pub(crate) enum MidHandshake<IO> {
    Handshaking(TlsStream<IO>),
    End,
}

impl<IO> Future for MidHandshake<IO>
where
    IO: AsyncRead + AsyncWrite + Unpin,
{
    type Output = io::Result<TlsStream<IO>>;

    #[inline]
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();

        if let MidHandshake::Handshaking(stream) = this {
            let eof = !stream.state.readable();
            let (io, session) = (&mut stream.io, &mut stream.session);
            let mut stream = Stream::new(io, session).set_eof(eof);

            if stream.session.is_handshaking() {
                ready!(stream.complete_io(cx))?;
            }

            if stream.session.wants_write() {
                ready!(stream.complete_io(cx))?;
            }
        }

        match mem::replace(this, MidHandshake::End) {
            MidHandshake::Handshaking(stream) => Poll::Ready(Ok(stream)),
            MidHandshake::End => panic!(),
        }
    }
}

impl<IO> AsyncRead for TlsStream<IO>
where
    IO: AsyncRead + AsyncWrite + Unpin,
{
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        let this = self.get_mut();
        let mut stream =
            Stream::new(&mut this.io, &mut this.session).set_eof(!this.state.readable());

        match this.state {
            TlsState::Stream | TlsState::WriteShutdown => {
                match stream.as_mut_pin().poll_read(cx, buf) {
                    Poll::Ready(Ok(0)) => {
                        this.state.shutdown_read();
                        Poll::Ready(Ok(0))
                    }
                    Poll::Ready(Ok(n)) => Poll::Ready(Ok(n)),
                    Poll::Ready(Err(ref err)) if err.kind() == io::ErrorKind::ConnectionAborted => {
                        this.state.shutdown_read();
                        if this.state.writeable() {
                            stream.session.send_close_notify();
                            this.state.shutdown_write();
                        }
                        Poll::Ready(Ok(0))
                    }
                    Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
                    Poll::Pending => Poll::Pending,
                }
            }
            TlsState::ReadShutdown | TlsState::FullyShutdown => Poll::Ready(Ok(0)),
            #[cfg(feature = "early-data")]
            s => unreachable!("server TLS can not hit this state: {:?}", s),
        }
    }
}

impl<IO> AsyncWrite for TlsStream<IO>
where
    IO: AsyncRead + AsyncWrite + Unpin,
{
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        let this = self.get_mut();
        let mut stream =
            Stream::new(&mut this.io, &mut this.session).set_eof(!this.state.readable());
        stream.as_mut_pin().poll_write(cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        let this = self.get_mut();
        let mut stream =
            Stream::new(&mut this.io, &mut this.session).set_eof(!this.state.readable());
        stream.as_mut_pin().poll_flush(cx)
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        if self.state.writeable() {
            self.session.send_close_notify();
            self.state.shutdown_write();
        }

        let this = self.get_mut();
        let mut stream =
            Stream::new(&mut this.io, &mut this.session).set_eof(!this.state.readable());
        stream.as_mut_pin().poll_close(cx)
    }
}
