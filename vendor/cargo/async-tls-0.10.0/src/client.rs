//! The client end of a TLS connection.

use crate::common::tls_state::TlsState;
use crate::rusttls::stream::Stream;
use futures_core::ready;
use futures_io::{AsyncRead, AsyncWrite};
use rustls::ClientSession;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::{io, mem};

use rustls::Session;

/// The client end of a TLS connection. Can be used like any other bidirectional IO stream.
/// Wraps the underlying TCP stream.
#[derive(Debug)]
pub struct TlsStream<IO> {
    pub(crate) io: IO,
    pub(crate) session: ClientSession,
    pub(crate) state: TlsState,

    #[cfg(feature = "early-data")]
    pub(crate) early_data: (usize, Vec<u8>),
}

pub(crate) enum MidHandshake<IO> {
    Handshaking(TlsStream<IO>),
    #[cfg(feature = "early-data")]
    EarlyData(TlsStream<IO>),
    End,
}

impl<IO> TlsStream<IO> {
    /// Returns a reference to the underlying IO stream.
    pub fn get_ref(&self) -> &IO {
        &self.io
    }

    /// Returns a mutuable reference to the underlying IO stream.
    pub fn get_mut(&mut self) -> &mut IO {
        &mut self.io
    }
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
            #[cfg(feature = "early-data")]
            MidHandshake::EarlyData(stream) => Poll::Ready(Ok(stream)),
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
        match self.state {
            #[cfg(feature = "early-data")]
            TlsState::EarlyData => {
                let this = self.get_mut();

                let mut stream =
                    Stream::new(&mut this.io, &mut this.session).set_eof(!this.state.readable());
                let (pos, data) = &mut this.early_data;

                // complete handshake
                if stream.session.is_handshaking() {
                    ready!(stream.complete_io(cx))?;
                }

                // write early data (fallback)
                if !stream.session.is_early_data_accepted() {
                    while *pos < data.len() {
                        let len = ready!(stream.as_mut_pin().poll_write(cx, &data[*pos..]))?;
                        *pos += len;
                    }
                }

                // end
                this.state = TlsState::Stream;
                data.clear();

                Pin::new(this).poll_read(cx, buf)
            }
            TlsState::Stream | TlsState::WriteShutdown => {
                let this = self.get_mut();
                let mut stream =
                    Stream::new(&mut this.io, &mut this.session).set_eof(!this.state.readable());

                match stream.as_mut_pin().poll_read(cx, buf) {
                    Poll::Ready(Ok(0)) => {
                        this.state.shutdown_read();
                        Poll::Ready(Ok(0))
                    }
                    Poll::Ready(Ok(n)) => Poll::Ready(Ok(n)),
                    Poll::Ready(Err(ref e)) if e.kind() == io::ErrorKind::ConnectionAborted => {
                        this.state.shutdown_read();
                        if this.state.writeable() {
                            stream.session.send_close_notify();
                            this.state.shutdown_write();
                        }
                        Poll::Ready(Ok(0))
                    }
                    Poll::Ready(Err(err)) => Poll::Ready(Err(err)),
                    Poll::Pending => Poll::Pending,
                }
            }
            TlsState::ReadShutdown | TlsState::FullyShutdown => Poll::Ready(Ok(0)),
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

        match this.state {
            #[cfg(feature = "early-data")]
            TlsState::EarlyData => {
                use std::io::Write;

                let (pos, data) = &mut this.early_data;

                // write early data
                if let Some(mut early_data) = stream.session.early_data() {
                    let len = match early_data.write(buf) {
                        Ok(n) => n,
                        Err(ref err) if err.kind() == io::ErrorKind::WouldBlock => {
                            return Poll::Pending
                        }
                        Err(err) => return Poll::Ready(Err(err)),
                    };
                    data.extend_from_slice(&buf[..len]);
                    return Poll::Ready(Ok(len));
                }

                // complete handshake
                if stream.session.is_handshaking() {
                    ready!(stream.complete_io(cx))?;
                }

                // write early data (fallback)
                if !stream.session.is_early_data_accepted() {
                    while *pos < data.len() {
                        let len = ready!(stream.as_mut_pin().poll_write(cx, &data[*pos..]))?;
                        *pos += len;
                    }
                }

                // end
                this.state = TlsState::Stream;
                data.clear();
                stream.as_mut_pin().poll_write(cx, buf)
            }
            _ => stream.as_mut_pin().poll_write(cx, buf),
        }
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
