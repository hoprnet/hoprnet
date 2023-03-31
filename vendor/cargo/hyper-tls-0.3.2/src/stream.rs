use std::fmt;
use std::io::{self, Read, Write};

use bytes::{Buf, BufMut};
use futures::Poll;
use native_tls;
use tokio_io::{AsyncRead, AsyncWrite};

/// A stream that might be protected with TLS.
pub enum MaybeHttpsStream<T> {
    /// A stream over plain text.
    Http(T),
    /// A stream protected with TLS.
    Https(TlsStream<T>),
}

/// A stream protected with TLS.
pub struct TlsStream<T> {
    inner: native_tls::TlsStream<T>,
}

// ===== impl MaybeHttpsStream =====

impl<T: fmt::Debug> fmt::Debug for MaybeHttpsStream<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            MaybeHttpsStream::Http(ref s) => {
                f.debug_tuple("Http")
                    .field(s)
                    .finish()
            },
            MaybeHttpsStream::Https(ref s) => {
                f.debug_tuple("Https")
                    .field(s)
                    .finish()
            },
        }
    }
}

impl<T> From<native_tls::TlsStream<T>> for MaybeHttpsStream<T> {
    fn from(inner: native_tls::TlsStream<T>) -> Self {
        MaybeHttpsStream::Https(TlsStream::from(inner))
    }
}

impl<T> From<T> for MaybeHttpsStream<T> {
    fn from(inner: T) -> Self {
        MaybeHttpsStream::Http(inner)
    }
}

impl<T> From<TlsStream<T>> for MaybeHttpsStream<T> {
    fn from(inner: TlsStream<T>) -> Self {
        MaybeHttpsStream::Https(inner)
    }
}

impl<T: Read + Write> Read for MaybeHttpsStream<T> {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match *self {
            MaybeHttpsStream::Http(ref mut s) => s.read(buf),
            MaybeHttpsStream::Https(ref mut s) => s.read(buf),
        }
    }
}

impl<T: Read + Write> Write for MaybeHttpsStream<T> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match *self {
            MaybeHttpsStream::Http(ref mut s) => s.write(buf),
            MaybeHttpsStream::Https(ref mut s) => s.write(buf),
        }
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        match *self {
            MaybeHttpsStream::Http(ref mut s) => s.flush(),
            MaybeHttpsStream::Https(ref mut s) => s.flush(),
        }
    }
}

impl<T: AsyncRead + AsyncWrite> AsyncRead for MaybeHttpsStream<T> {
    #[inline]
    unsafe fn prepare_uninitialized_buffer(&self, buf: &mut [u8]) -> bool {
        match *self {
            MaybeHttpsStream::Http(ref s) => s.prepare_uninitialized_buffer(buf),
            MaybeHttpsStream::Https(ref s) => s.prepare_uninitialized_buffer(buf),
        }
    }

    #[inline]
    fn read_buf<B: BufMut>(&mut self, buf: &mut B) -> Poll<usize, io::Error> {
        match *self {
            MaybeHttpsStream::Http(ref mut s) => s.read_buf(buf),
            MaybeHttpsStream::Https(ref mut s) => s.read_buf(buf),
        }
    }
}

impl<T: AsyncWrite + AsyncRead> AsyncWrite for MaybeHttpsStream<T> {
    #[inline]
    fn shutdown(&mut self) -> Poll<(), io::Error> {
        match *self {
            MaybeHttpsStream::Http(ref mut s) => s.shutdown(),
            MaybeHttpsStream::Https(ref mut s) => s.shutdown(),
        }
    }

    #[inline]
    fn write_buf<B: Buf>(&mut self, buf: &mut B) -> Poll<usize, io::Error> {
        match *self {
            MaybeHttpsStream::Http(ref mut s) => s.write_buf(buf),
            MaybeHttpsStream::Https(ref mut s) => s.write_buf(buf),
        }
    }
}

// ===== impl TlsStream =====

impl<T> TlsStream<T> {
    pub(crate) fn new(inner: native_tls::TlsStream<T>) -> Self {
        TlsStream {
            inner,
        }
    }

    /// Get access to the internal `native_tls::TlsStream` stream which also
    /// transitively allows access to `T`.
    pub fn get_ref(&self) -> &native_tls::TlsStream<T> {
        &self.inner
    }


    /// Get mutable access to the internal `native_tls::TlsStream` stream which
    /// also transitively allows mutable access to `T`.
    pub fn get_mut(&mut self) -> &mut native_tls::TlsStream<T> {
        &mut self.inner
    }
}

impl<T: fmt::Debug> fmt::Debug for TlsStream<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self.inner, f)
    }
}

impl<T> From<native_tls::TlsStream<T>> for TlsStream<T> {
    fn from(stream: native_tls::TlsStream<T>) -> Self {
        TlsStream { inner: stream }
    }
}

impl<T: Read + Write> Read for TlsStream<T> {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf)
    }
}

impl<T: Read + Write> Write for TlsStream<T> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.inner.write(buf)
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

impl<T: AsyncRead + AsyncWrite> AsyncRead for TlsStream<T> {}

impl<T: AsyncWrite + AsyncRead> AsyncWrite for TlsStream<T> {
    fn shutdown(&mut self) -> Poll<(), io::Error> {
        try_nb!(self.inner.shutdown());
        self.inner.get_mut().shutdown()
    }
}
