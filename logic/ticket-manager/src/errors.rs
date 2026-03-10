#[derive(Debug, thiserror::Error)]
pub enum TicketManagerError {
    #[error("channel not found")]
    ChannelNotFound,
    #[error("already redeeming in this channel")]
    AlreadyRedeeming,
    #[error("queue error: {0}")]
    QueueError(#[from] std::io::Error),
}
