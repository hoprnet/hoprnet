use crate::upgrade::Connection;

/// The sending half of a channel to send an upgraded connection.
///
/// Unlike `async_channel::Sender` the `send` method on this type can only be
/// called once, and cannot be cloned. That's because only a single instance of
/// `Connection` should be created.
#[derive(Debug)]
pub struct Sender {
    sender: async_channel::Sender<Connection>,
}

impl Sender {
    /// Create a new instance of `Sender`.
    #[doc(hidden)]
    pub fn new(sender: async_channel::Sender<Connection>) -> Self {
        Self { sender }
    }

    /// Send a `Connection`.
    ///
    /// The channel will be consumed after having sent the connection.
    pub async fn send(self, conn: Connection) {
        let _ = self.sender.send(conn).await;
    }
}
