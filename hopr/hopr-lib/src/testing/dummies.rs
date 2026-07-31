use std::sync::{
    Arc, Mutex,
    atomic::{AtomicU64, Ordering},
};

use futures::AsyncReadExt as _;
use hopr_transport::IncomingSession;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_util::compat::{FuturesAsyncReadCompatExt, FuturesAsyncWriteCompatExt};

use crate::errors::HoprLibError;

/// Session echo server that copies every received byte back to the sender.
///
/// If `received_bytes` is set, the counter is incremented progressively as
/// bytes flow in — making it suitable for throughput sampling.
#[derive(Debug, Clone)]
pub struct EchoServer {
    pub received_bytes: Option<Arc<AtomicU64>>,
    /// When `false`, received bytes are only tallied and never written back.
    ///
    /// Echoing requires SURBs on the return path. The throughput harness sends
    /// one-way traffic without them, so the write would fail and break the read
    /// loop, stalling delivery — hence count-only mode for those runs.
    pub echo: bool,
}

impl Default for EchoServer {
    fn default() -> Self {
        Self {
            received_bytes: None,
            echo: true,
        }
    }
}

impl EchoServer {
    pub fn new() -> Self {
        Self::default()
    }

    /// Count received bytes without echoing (used by the throughput harness sampler).
    pub fn with_counter(counter: Arc<AtomicU64>) -> Self {
        Self {
            received_bytes: Some(counter),
            echo: false,
        }
    }

    /// Count received bytes *and* echo them back, for bidirectional tests that
    /// also want delivery accounting.
    pub fn with_counter_and_echo(counter: Arc<AtomicU64>) -> Self {
        Self {
            received_bytes: Some(counter),
            echo: true,
        }
    }
}

#[async_trait::async_trait]
impl hopr_api::node::HoprSessionServer for EchoServer {
    type Error = HoprLibError;
    type Session = IncomingSession;

    async fn process(&self, session: IncomingSession) -> Result<(), HoprLibError> {
        let counter = self.received_bytes.clone();
        let echo = self.echo;
        tokio::spawn(async move {
            let (r, w) = session.session.split();
            let mut r = r.compat();
            let mut w = w.compat_write();
            let mut buf = vec![0u8; 8192];
            loop {
                let n = match r.read(&mut buf).await {
                    Ok(0) => break,
                    Ok(n) => n,
                    Err(error) => {
                        tracing::debug!(?error, "Echo server read error");
                        break;
                    }
                };
                if let Some(c) = &counter {
                    c.fetch_add(n as u64, Ordering::Relaxed);
                }
                // In count-only mode the bytes are never written back: echoing needs
                // SURBs on the return path, and without them the write fails and
                // breaks the read loop, stalling delivery.
                if echo {
                    if let Err(error) = w.write_all(&buf[..n]).await {
                        tracing::debug!(?error, "Echo server write error");
                        break;
                    }
                    // `write_all` only fills the session's frame buffer. Nothing polls the
                    // write half again once this task parks on the next `read`, so without an
                    // explicit flush a partial frame is never emitted and the reply never
                    // leaves — the peer then sees the session idle out. (The previous
                    // `tokio::io::copy` implementation flushed on its own.)
                    if let Err(error) = w.flush().await {
                        tracing::debug!(?error, "Echo server flush error");
                        break;
                    }
                }
            }
        });

        Ok(())
    }
}

/// A [`HoprSessionServer`] that captures the first incoming [`IncomingSession`]
/// into a shared `Arc<Mutex<Option<IncomingSession>>>` for test access.
///
/// After capturing, it holds the session open indefinitely via `futures::future::pending()`
/// so the test can read and write data through it.
#[derive(Clone)]
pub struct SessionCaptureServer {
    /// Shared slot for the captured incoming session.
    pub captured: Arc<Mutex<Option<IncomingSession>>>,
}

impl SessionCaptureServer {
    /// Create a new [`SessionCaptureServer`] and return a handle to the captured session.
    pub fn new() -> (Self, Arc<Mutex<Option<IncomingSession>>>) {
        let captured = Arc::new(Mutex::new(None));
        (
            Self {
                captured: captured.clone(),
            },
            captured,
        )
    }
}

#[async_trait::async_trait]
impl hopr_api::node::HoprSessionServer for SessionCaptureServer {
    type Error = HoprLibError;
    type Session = IncomingSession;

    async fn process(&self, session: IncomingSession) -> Result<(), HoprLibError> {
        {
            let mut captured = self.captured.lock().unwrap();
            if captured.is_some() {
                // Preserve the first session — ignore subsequent ones
                return Ok(());
            }
            captured.replace(session);
        }
        tracing::debug!("SessionCaptureServer captured incoming session");
        // Keep the future alive to stop the session from being dropped
        let () = futures::future::pending().await;
        Ok(())
    }
}
