#[derive(Debug, thiserror::Error)]
pub enum TicketManagerError {
    #[error("channel not found")]
    ChannelNotFound,

    #[error("already redeeming in this channel")]
    AlreadyRedeeming,

    #[error("ticket redemption error: {0}")]
    RedeemError(#[source] anyhow::Error),

    #[error("queue error: {0}")]
    QueueError(#[source] anyhow::Error),

    #[error(transparent)]
    Other(anyhow::Error),
}

impl TicketManagerError {
    pub fn redeem<E: Into<anyhow::Error>>(error: E) -> Self {
        Self::RedeemError(error.into())
    }

    pub fn queue<E: Into<anyhow::Error>>(error: E) -> Self {
        Self::QueueError(error.into())
    }
}
