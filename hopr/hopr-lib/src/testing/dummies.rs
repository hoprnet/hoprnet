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
impl crate::api::node::HoprSessionServer for EchoServer {
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
