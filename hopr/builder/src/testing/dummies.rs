use futures::AsyncReadExt;
use hopr_lib::{IncomingSession, errors::HoprLibError, traits::session::HoprSessionServer};
use tokio_util::compat::{FuturesAsyncReadCompatExt, FuturesAsyncWriteCompatExt};

#[derive(Debug, Clone, Default)]
pub struct EchoServer {}

impl EchoServer {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait::async_trait]
impl HoprSessionServer for EchoServer {
    async fn process(&self, session: IncomingSession) -> std::result::Result<(), HoprLibError> {
        tokio::spawn(async move {
            let (r, w) = session.session.split();

            if let Err(error) = tokio::io::copy(&mut r.compat(), &mut w.compat_write()).await {
                tracing::debug!(?error, "Echo server session ended with error:");
            }
        });

        Ok(())
    }
}
