use std::sync::{Arc, Mutex};

use futures::AsyncReadExt;
use hopr_transport::IncomingSession;
use tokio_util::compat::{FuturesAsyncReadCompatExt, FuturesAsyncWriteCompatExt};

use crate::errors::HoprLibError;

#[derive(Debug, Clone, Default)]
pub struct EchoServer {}

impl EchoServer {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait::async_trait]
impl hopr_api::node::HoprSessionServer for EchoServer {
    type Error = HoprLibError;
    type Session = IncomingSession;

    async fn process(&self, session: IncomingSession) -> Result<(), HoprLibError> {
        tokio::spawn(async move {
            let (r, w) = session.session.split();

            if let Err(error) = tokio::io::copy(&mut r.compat(), &mut w.compat_write()).await {
                tracing::debug!(?error, "Echo server session ended with error:");
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
