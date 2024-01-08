use crate::{async_trait, Result, Session};

/// An async session backend.
#[async_trait]
pub trait SessionStore: std::fmt::Debug + Send + Sync + Clone + 'static {
    /// Get a session from the storage backend.
    ///
    /// The input is expected to be the value of an identifying
    /// cookie. This will then be parsed by the session middleware
    /// into a session if possible
    async fn load_session(&self, cookie_value: String) -> Result<Option<Session>>;

    /// Store a session on the storage backend.
    ///
    /// The return value is the value of the cookie to store for the
    /// user that represents this session
    async fn store_session(&self, session: Session) -> Result<Option<String>>;

    /// Remove a session from the session store
    async fn destroy_session(&self, session: Session) -> Result;

    /// Empties the entire store, destroying all sessions
    async fn clear_store(&self) -> Result;
}
