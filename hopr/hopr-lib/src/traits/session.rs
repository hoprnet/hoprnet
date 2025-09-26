#[cfg(feature = "session-server")]
use crate::{errors::Result, exports::transport::session::IncomingSession};

/// Interface representing the HOPR server behavior for each incoming session instance
/// supplied as an argument.
#[cfg(feature = "session-server")]
#[async_trait::async_trait]
pub trait HoprSessionServer {
    /// Fully process a single HOPR session
    async fn process(&self, session: IncomingSession) -> Result<()>;
}
