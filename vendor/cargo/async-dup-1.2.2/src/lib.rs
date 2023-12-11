//! Duplicate an async I/O handle.
//!
//! This crate provides two tools, [`Arc`] and [`Mutex`]:
//!
//! * [`Arc`] implements [`AsyncRead`], [`AsyncWrite`], and [`AsyncSeek`] if a reference to the
//!   inner type does.
//! * A reference to [`Mutex`] implements [`AsyncRead`], [`AsyncWrite`], and [`AsyncSeek`] if the
//!   inner type does.
//!
//! Wrap an async I/O handle in [`Arc`] or [`Mutex`] to clone it or share among tasks.
//!
//! # Examples
//!
//! Clone an async I/O handle:
//!
//! ```no_run
//! use async_dup::Arc;
//! use futures::io;
//! use smol::Async;
//! use std::net::TcpStream;
//!
//! # fn main() -> std::io::Result<()> { smol::run(async {
//! // A client that echoes messages back to the server.
//! let stream = Async::<TcpStream>::connect("127.0.0.1:8000").await?;
//!
//! // Create two handles to the stream.
//! let reader = Arc::new(stream);
//! let mut writer = reader.clone();
//!
//! // Echo data received from the reader back into the writer.
//! io::copy(reader, &mut writer).await?;
//! # Ok(()) }) }
//! ```
//!
//! Share an async I/O handle:
//!
//! ```
//! use async_dup::Mutex;
//! use futures::io;
//! use futures::prelude::*;
//!
//! // Reads data from a stream and echoes it back.
//! async fn echo(stream: impl AsyncRead + AsyncWrite + Unpin) -> io::Result<u64> {
//!     let stream = Mutex::new(stream);
//!     io::copy(&stream, &mut &stream).await
//! }
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs, missing_debug_implementations, rust_2018_idioms)]

use std::fmt;
use std::hash::{Hash, Hasher};
use std::io::{self, IoSlice, IoSliceMut, SeekFrom};
use std::ops::{Deref, DerefMut};
use std::pin::Pin;
use std::task::{Context, Poll};

use futures_io::{AsyncRead, AsyncSeek, AsyncWrite};

/// A reference-counted pointer that implements async I/O traits.
///
/// This is just a wrapper around [`std::sync::Arc`] that adds the following impls:
///
/// - `impl<T> AsyncRead for Arc<T> where &T: AsyncRead {}`
/// - `impl<T> AsyncWrite for Arc<T> where &T: AsyncWrite {}`
/// - `impl<T> AsyncSeek for Arc<T> where &T: AsyncSeek {}`
pub struct Arc<T>(pub std::sync::Arc<T>);

impl<T> Unpin for Arc<T> {}

impl<T> Arc<T> {
    /// Constructs a new `Arc<T>`.
    ///
    /// # Examples
    ///
    /// ```
    /// use async_dup::Arc;
    ///
    /// let a = Arc::new(7);
    /// ```
    pub fn new(data: T) -> Arc<T> {
        Arc(std::sync::Arc::new(data))
    }
}

impl<T> Clone for Arc<T> {
    fn clone(&self) -> Arc<T> {
        Arc(self.0.clone())
    }
}

impl<T> Deref for Arc<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: fmt::Debug> fmt::Debug for Arc<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl<T: fmt::Display> fmt::Display for Arc<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}

impl<T: Hash> Hash for Arc<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (**self).hash(state)
    }
}

impl<T> fmt::Pointer for Arc<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Pointer::fmt(&(&**self as *const T), f)
    }
}

impl<T: Default> Default for Arc<T> {
    fn default() -> Arc<T> {
        Arc::new(Default::default())
    }
}

impl<T> From<T> for Arc<T> {
    fn from(t: T) -> Arc<T> {
        Arc::new(t)
    }
}

// NOTE(stjepang): It would also make sense to have the following impls:
//
// - `impl<T> AsyncRead for &Arc<T> where &T: AsyncRead {}`
// - `impl<T> AsyncWrite for &Arc<T> where &T: AsyncWrite {}`
// - `impl<T> AsyncSeek for &Arc<T> where &T: AsyncSeek {}`
//
// However, those impls sometimes make Rust's type inference try too hard when types cannot be
// inferred. In the end, instead of complaining with a nice error message, the Rust compiler ends
// up overflowing and dumping a very long error message spanning multiple screens.
//
// Since those impls are not essential, I decided to err on the safe side and not include them.

impl<T> AsyncRead for Arc<T>
where
    for<'a> &'a T: AsyncRead,
{
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut &*self.0).poll_read(cx, buf)
    }

    fn poll_read_vectored(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &mut [IoSliceMut<'_>],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut &*self.0).poll_read_vectored(cx, bufs)
    }
}

impl<T> AsyncWrite for Arc<T>
where
    for<'a> &'a T: AsyncWrite,
{
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut &*self.0).poll_write(cx, buf)
    }

    fn poll_write_vectored(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &[IoSlice<'_>],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut &*self.0).poll_write_vectored(cx, bufs)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut &*self.0).poll_flush(cx)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut &*self.0).poll_close(cx)
    }
}

impl<T> AsyncSeek for Arc<T>
where
    for<'a> &'a T: AsyncSeek,
{
    fn poll_seek(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        pos: SeekFrom,
    ) -> Poll<io::Result<u64>> {
        Pin::new(&mut &*self.0).poll_seek(cx, pos)
    }
}

/// A mutex that implements async I/O traits.
///
/// This is a blocking mutex that adds the following impls:
///
/// - `impl<T> AsyncRead for Mutex<T> where T: AsyncRead + Unpin {}`
/// - `impl<T> AsyncRead for &Mutex<T> where T: AsyncRead + Unpin {}`
/// - `impl<T> AsyncWrite for Mutex<T> where T: AsyncWrite + Unpin {}`
/// - `impl<T> AsyncWrite for &Mutex<T> where T: AsyncWrite + Unpin {}`
/// - `impl<T> AsyncSeek for Mutex<T> where T: AsyncSeek + Unpin {}`
/// - `impl<T> AsyncSeek for &Mutex<T> where T: AsyncSeek + Unpin {}`
pub struct Mutex<T>(simple_mutex::Mutex<T>);

impl<T> Mutex<T> {
    /// Creates a new mutex.
    ///
    /// # Examples
    ///
    /// ```
    /// use async_dup::Mutex;
    ///
    /// let mutex = Mutex::new(10);
    /// ```
    pub fn new(data: T) -> Mutex<T> {
        Mutex(data.into())
    }

    /// Acquires the mutex, blocking the current thread until it is able to do so.
    ///
    /// Returns a guard that releases the mutex when dropped.
    ///
    /// # Examples
    ///
    /// ```
    /// use async_dup::Mutex;
    ///
    /// let mutex = Mutex::new(10);
    /// let guard = mutex.lock();
    /// assert_eq!(*guard, 10);
    /// ```
    pub fn lock(&self) -> MutexGuard<'_, T> {
        MutexGuard(self.0.lock())
    }

    /// Attempts to acquire the mutex.
    ///
    /// If the mutex could not be acquired at this time, then [`None`] is returned. Otherwise, a
    /// guard is returned that releases the mutex when dropped.
    ///
    /// [`None`]: https://doc.rust-lang.org/std/option/enum.Option.html#variant.None
    ///
    /// # Examples
    ///
    /// ```
    /// use async_dup::Mutex;
    ///
    /// let mutex = Mutex::new(10);
    /// if let Some(guard) = mutex.try_lock() {
    ///     assert_eq!(*guard, 10);
    /// }
    /// # ;
    /// ```
    pub fn try_lock(&self) -> Option<MutexGuard<'_, T>> {
        if let Some(guard) = self.0.try_lock() {
            Some(MutexGuard(guard))
        } else {
            None
        }
    }

    /// Consumes the mutex, returning the underlying data.
    ///
    /// # Examples
    ///
    /// ```
    /// use async_dup::Mutex;
    ///
    /// let mutex = Mutex::new(10);
    /// assert_eq!(mutex.into_inner(), 10);
    /// ```
    pub fn into_inner(self) -> T {
        self.0.into_inner()
    }

    /// Returns a mutable reference to the underlying data.
    ///
    /// Since this call borrows the mutex mutably, no actual locking takes place -- the mutable
    /// borrow statically guarantees the mutex is not already acquired.
    ///
    /// # Examples
    ///
    /// ```
    /// use async_dup::Mutex;
    ///
    /// let mut mutex = Mutex::new(0);
    /// *mutex.get_mut() = 10;
    /// assert_eq!(*mutex.lock(), 10);
    /// ```
    pub fn get_mut(&mut self) -> &mut T {
        self.0.get_mut()
    }
}

impl<T: fmt::Debug> fmt::Debug for Mutex<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        struct Locked;
        impl fmt::Debug for Locked {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("<locked>")
            }
        }

        match self.try_lock() {
            None => f.debug_struct("Mutex").field("data", &Locked).finish(),
            Some(guard) => f.debug_struct("Mutex").field("data", &&*guard).finish(),
        }
    }
}

impl<T> From<T> for Mutex<T> {
    fn from(val: T) -> Mutex<T> {
        Mutex::new(val)
    }
}

impl<T: Default> Default for Mutex<T> {
    fn default() -> Mutex<T> {
        Mutex::new(Default::default())
    }
}

impl<T: AsyncRead + Unpin> AsyncRead for Mutex<T> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut *self.lock()).poll_read(cx, buf)
    }

    fn poll_read_vectored(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &mut [IoSliceMut<'_>],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut *self.lock()).poll_read_vectored(cx, bufs)
    }
}

impl<T: AsyncRead + Unpin> AsyncRead for &Mutex<T> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut *self.lock()).poll_read(cx, buf)
    }

    fn poll_read_vectored(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &mut [IoSliceMut<'_>],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut *self.lock()).poll_read_vectored(cx, bufs)
    }
}

impl<T: AsyncWrite + Unpin> AsyncWrite for Mutex<T> {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut *self.lock()).poll_write(cx, buf)
    }

    fn poll_write_vectored(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &[IoSlice<'_>],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut *self.lock()).poll_write_vectored(cx, bufs)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut *self.lock()).poll_flush(cx)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut *self.lock()).poll_close(cx)
    }
}

impl<T: AsyncWrite + Unpin> AsyncWrite for &Mutex<T> {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut *self.lock()).poll_write(cx, buf)
    }

    fn poll_write_vectored(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &[IoSlice<'_>],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut *self.lock()).poll_write_vectored(cx, bufs)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut *self.lock()).poll_flush(cx)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut *self.lock()).poll_close(cx)
    }
}

impl<T: AsyncSeek + Unpin> AsyncSeek for Mutex<T> {
    fn poll_seek(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        pos: SeekFrom,
    ) -> Poll<io::Result<u64>> {
        Pin::new(&mut *self.lock()).poll_seek(cx, pos)
    }
}

impl<T: AsyncSeek + Unpin> AsyncSeek for &Mutex<T> {
    fn poll_seek(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        pos: SeekFrom,
    ) -> Poll<io::Result<u64>> {
        Pin::new(&mut *self.lock()).poll_seek(cx, pos)
    }
}

/// A guard that releases the mutex when dropped.
pub struct MutexGuard<'a, T>(simple_mutex::MutexGuard<'a, T>);

impl<T: fmt::Debug> fmt::Debug for MutexGuard<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl<T: fmt::Display> fmt::Display for MutexGuard<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (**self).fmt(f)
    }
}

impl<T> Deref for MutexGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> DerefMut for MutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}
