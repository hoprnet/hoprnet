use std::sync::{Arc, atomic::{AtomicU64, Ordering}};

use futures::AsyncReadExt as _;
use hopr_transport::IncomingSession;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_util::compat::{FuturesAsyncReadCompatExt, FuturesAsyncWriteCompatExt};

use crate::errors::HoprLibError;

/// Session echo server that copies every received byte back to the sender.
///
/// If `received_bytes` is set, the counter is incremented progressively as
/// bytes flow in — making it suitable for throughput sampling.
#[derive(Debug, Clone, Default)]
pub struct EchoServer {
    pub received_bytes: Option<Arc<AtomicU64>>,
}

impl EchoServer {
    pub fn new() -> Self {
        Self::default()
    }

    /// Attach a shared receive-byte counter (used by the stress harness sampler).
    pub fn with_counter(counter: Arc<AtomicU64>) -> Self {
        Self { received_bytes: Some(counter) }
    }
}

#[async_trait::async_trait]
impl hopr_api::node::HoprSessionServer for EchoServer {
    type Error = HoprLibError;
    type Session = IncomingSession;

    async fn process(&self, session: IncomingSession) -> Result<(), HoprLibError> {
        let counter = self.received_bytes.clone();
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
                    // Count-only mode: just tally bytes received, no echo.
                    // Echoing requires SURBs for the return path; without them the
                    // write fails and breaks the read loop, causing delivery to stall.
                    c.fetch_add(n as u64, Ordering::Relaxed);
                } else if let Err(error) = w.write_all(&buf[..n]).await {
                    tracing::debug!(?error, "Echo server write error");
                    break;
                }
            }
        });

        Ok(())
    }
}
