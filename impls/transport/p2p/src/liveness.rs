//! Per-peer connection liveness tracking for libp2p streams.
//!
//! [`LivenessStream`] wraps any `AsyncRead + AsyncWrite` value and checks an
//! `Arc<AtomicBool>` liveness flag **before every poll**. When the swarm event
//! loop clears the flag on `ConnectionClosed` / `OutgoingConnectionError`, the
//! very next read or write on any wrapped substream for that peer returns
//! `Err(io::ErrorKind::ConnectionAborted)`.
//!
//! This makes dead streams self-signal to their consumers (the per-peer reader
//! and writer tasks in `hopr-transport`) without any changes to the
//! `NetworkStreamControl` trait or the protocol layer.

use std::{
    io,
    pin::Pin,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    task::{Context, Poll},
};

use dashmap::DashMap;
use futures::{AsyncRead, AsyncWrite};
use libp2p::PeerId;
use pin_project::pin_project;

/// Shared registry mapping each peer to its connection-liveness flag.
///
/// A clone of this registry is held by both [`crate::HoprNetwork`] and the
/// swarm event-loop closure. When libp2p reports a peer fully disconnected,
/// the loop calls [`mark_peer_disconnected`], which removes the entry and
/// clears the flag. Any [`LivenessStream`] that already cloned the old
/// `Arc<AtomicBool>` will see `false` on its next poll.
pub(crate) type LivenessRegistry = Arc<DashMap<PeerId, Arc<AtomicBool>>>;

/// Returns the liveness flag for `peer`, creating a fresh `true` flag if none exists.
pub(crate) fn peer_liveness(registry: &LivenessRegistry, peer: &PeerId) -> Arc<AtomicBool> {
    registry
        .entry(*peer)
        .or_insert_with(|| Arc::new(AtomicBool::new(true)))
        .clone()
}

/// Marks a peer as disconnected: sets its flag to `false` and removes the registry entry.
///
/// A subsequent `open()` call mints a fresh `true` flag via [`peer_liveness`].
/// Any [`LivenessStream`] that still holds the old `Arc` will error on its next poll.
pub(crate) fn mark_peer_disconnected(registry: &LivenessRegistry, peer: &PeerId) {
    if let Some((_, flag)) = registry.remove(peer) {
        flag.store(false, Ordering::Relaxed);
    }
}

/// An `AsyncRead + AsyncWrite` wrapper that errors once its connection-liveness flag is cleared.
///
/// Every `poll_read`, `poll_write`, `poll_flush`, and `poll_close` performs a
/// single `Relaxed` load of the flag. If the flag is `false`, the poll returns
/// `Err(io::ErrorKind::ConnectionAborted)` immediately, without touching the
/// inner stream. This surfaces naturally to `FramedRead`/`FramedWrite` above it,
/// ending the per-peer reader/writer forward-futures and triggering cache invalidation.
#[pin_project]
pub(crate) struct LivenessStream<S> {
    #[pin]
    inner: S,
    alive: Arc<AtomicBool>,
}

impl<S> LivenessStream<S> {
    pub(crate) fn new(inner: S, alive: Arc<AtomicBool>) -> Self {
        Self { inner, alive }
    }

    fn dead_error() -> io::Error {
        io::Error::new(io::ErrorKind::ConnectionAborted, "connection to peer was closed")
    }
}

impl<S: AsyncRead> AsyncRead for LivenessStream<S> {
    fn poll_read(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut [u8]) -> Poll<io::Result<usize>> {
        let this = self.project();
        if !this.alive.load(Ordering::Relaxed) {
            return Poll::Ready(Err(Self::dead_error()));
        }
        this.inner.poll_read(cx, buf)
    }
}

impl<S: AsyncWrite> AsyncWrite for LivenessStream<S> {
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<io::Result<usize>> {
        let this = self.project();
        if !this.alive.load(Ordering::Relaxed) {
            return Poll::Ready(Err(Self::dead_error()));
        }
        this.inner.poll_write(cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        let this = self.project();
        if !this.alive.load(Ordering::Relaxed) {
            return Poll::Ready(Err(Self::dead_error()));
        }
        this.inner.poll_flush(cx)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        let this = self.project();
        if !this.alive.load(Ordering::Relaxed) {
            return Poll::Ready(Err(Self::dead_error()));
        }
        this.inner.poll_close(cx)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::AtomicBool;

    use anyhow::Context;
    use async_channel_io::pipe;
    use futures::{AsyncReadExt, AsyncWriteExt};

    use super::*;

    /// Build a connected in-memory full-duplex pair for testing.
    ///
    /// Returns `(writer_end, reader_end)` where bytes written to `writer_end`
    /// can be read from `reader_end`.
    fn in_memory_pipe() -> (impl AsyncWrite, impl AsyncRead) {
        let (writer, reader) = pipe();
        (writer, reader)
    }

    // ---------------------------------------------------------------------------
    // LivenessStream unit tests
    // ---------------------------------------------------------------------------

    #[tokio::test]
    async fn liveness_stream_should_pass_through_reads_when_alive() -> anyhow::Result<()> {
        let (mut raw_write, raw_read) = in_memory_pipe();
        let alive = Arc::new(AtomicBool::new(true));
        let mut stream = LivenessStream::new(raw_read, alive.clone());

        raw_write.write_all(b"hello").await.context("write failed")?;
        drop(raw_write);

        let mut buf = vec![0u8; 5];
        stream.read_exact(&mut buf).await.context("read failed")?;
        assert_eq!(&buf, b"hello");
        Ok(())
    }

    #[tokio::test]
    async fn liveness_stream_should_pass_through_writes_when_alive() -> anyhow::Result<()> {
        let (raw_write, mut raw_read) = in_memory_pipe();
        let alive = Arc::new(AtomicBool::new(true));
        let mut stream = LivenessStream::new(raw_write, alive.clone());

        stream.write_all(b"world").await.context("write failed")?;
        // flush is needed for the channel to yield
        stream.flush().await.context("flush failed")?;
        drop(stream);

        let mut buf = vec![0u8; 5];
        raw_read.read_exact(&mut buf).await.context("read failed")?;
        assert_eq!(&buf, b"world");
        Ok(())
    }

    #[tokio::test]
    async fn liveness_stream_should_error_on_read_when_flag_cleared() -> anyhow::Result<()> {
        let (_raw_write, raw_read) = in_memory_pipe();
        let alive = Arc::new(AtomicBool::new(true));
        let mut stream = LivenessStream::new(raw_read, alive.clone());

        alive.store(false, Ordering::Relaxed);

        let mut buf = vec![0u8; 4];
        let result = stream.read(&mut buf).await;

        assert!(
            matches!(result, Err(ref e) if e.kind() == io::ErrorKind::ConnectionAborted),
            "expected ConnectionAborted, got {result:?}"
        );
        Ok(())
    }

    #[tokio::test]
    async fn liveness_stream_should_error_on_write_when_flag_cleared() -> anyhow::Result<()> {
        let (raw_write, _raw_read) = in_memory_pipe();
        let alive = Arc::new(AtomicBool::new(true));
        let mut stream = LivenessStream::new(raw_write, alive.clone());

        alive.store(false, Ordering::Relaxed);

        let result = stream.write(b"data").await;

        assert!(
            matches!(result, Err(ref e) if e.kind() == io::ErrorKind::ConnectionAborted),
            "expected ConnectionAborted, got {result:?}"
        );
        Ok(())
    }

    #[tokio::test]
    async fn liveness_stream_should_error_on_flush_when_flag_cleared() -> anyhow::Result<()> {
        let (raw_write, _raw_read) = in_memory_pipe();
        let alive = Arc::new(AtomicBool::new(true));
        let mut stream = LivenessStream::new(raw_write, alive.clone());

        alive.store(false, Ordering::Relaxed);

        let result = stream.flush().await;

        assert!(
            matches!(result, Err(ref e) if e.kind() == io::ErrorKind::ConnectionAborted),
            "expected ConnectionAborted, got {result:?}"
        );
        Ok(())
    }

    // ---------------------------------------------------------------------------
    // LivenessRegistry helpers
    // ---------------------------------------------------------------------------

    #[test]
    fn peer_liveness_should_create_alive_flag_for_new_peer() {
        let registry: LivenessRegistry = Default::default();
        let peer = PeerId::random();

        let flag = peer_liveness(&registry, &peer);

        assert!(flag.load(Ordering::Relaxed), "new flag must be alive");
        assert!(registry.contains_key(&peer));
    }

    #[test]
    fn peer_liveness_should_return_same_flag_for_same_peer() {
        let registry: LivenessRegistry = Default::default();
        let peer = PeerId::random();

        let flag1 = peer_liveness(&registry, &peer);
        let flag2 = peer_liveness(&registry, &peer);

        // Both arcs must point to the same allocation.
        assert!(Arc::ptr_eq(&flag1, &flag2), "should return the same Arc");
    }

    #[test]
    fn mark_peer_disconnected_should_clear_flag_and_remove_entry() {
        let registry: LivenessRegistry = Default::default();
        let peer = PeerId::random();

        let flag = peer_liveness(&registry, &peer);
        assert!(flag.load(Ordering::Relaxed));

        mark_peer_disconnected(&registry, &peer);

        // The flag is now dead.
        assert!(!flag.load(Ordering::Relaxed), "flag must be cleared after disconnect");
        // The entry is removed, so the next open() mints a fresh flag.
        assert!(!registry.contains_key(&peer));
    }

    #[test]
    fn mark_peer_disconnected_should_be_safe_when_peer_not_in_registry() {
        let registry: LivenessRegistry = Default::default();
        let peer = PeerId::random();

        // Should not panic.
        mark_peer_disconnected(&registry, &peer);
    }

    #[test]
    fn subsequent_peer_liveness_after_disconnect_should_return_fresh_alive_flag() {
        let registry: LivenessRegistry = Default::default();
        let peer = PeerId::random();

        let old_flag = peer_liveness(&registry, &peer);
        mark_peer_disconnected(&registry, &peer);

        let new_flag = peer_liveness(&registry, &peer);

        assert!(!old_flag.load(Ordering::Relaxed), "old flag must be dead");
        assert!(new_flag.load(Ordering::Relaxed), "new flag must be alive");
        assert!(!Arc::ptr_eq(&old_flag, &new_flag), "must be a distinct Arc");
    }
}
