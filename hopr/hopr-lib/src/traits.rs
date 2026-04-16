/// Session-related traits.
pub mod session {
    use crate::{errors::Result, exports::transport::IncomingSession};

    /// Interface representing the HOPR server behavior for each incoming session instance
    /// supplied as an argument.
    #[async_trait::async_trait]
    pub trait HoprSessionServer {
        /// Fully process a single HOPR session
        async fn process(&self, session: IncomingSession) -> Result<()>;
    }
}

pub use session::HoprSessionServer;

/// Noop implementation for nodes that do not run a session server.
#[async_trait::async_trait]
impl HoprSessionServer for () {
    async fn process(&self, _session: crate::exports::transport::IncomingSession) -> crate::errors::Result<()> {
        tracing::warn!("incoming session received but no session server configured, dropping");
        Ok(())
    }
}
