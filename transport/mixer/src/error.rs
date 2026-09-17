//! Error types shared across the mixer implementations.

/// Error returned by a mixer channel sender.
#[derive(Clone, Debug, thiserror::Error)]
pub enum SenderError {
    /// The channel is closed because the receiver was dropped.
    #[error("Channel is closed")]
    Closed,
}
