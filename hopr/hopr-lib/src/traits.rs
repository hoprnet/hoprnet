/// Session-related traits.
#[cfg(feature = "session-server")]
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

#[cfg(feature = "session-server")]
pub use session::HoprSessionServer;
