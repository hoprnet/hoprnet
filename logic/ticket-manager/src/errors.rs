use hopr_api::types::internal::tickets::Ticket;

#[derive(Debug, thiserror::Error)]
pub enum TicketManagerError {
    #[error("channel not found")]
    ChannelNotFound,

    #[error("already redeeming in this channel")]
    AlreadyRedeeming,

    #[error("ticket value too low for redeeming: {0}")]
    TicketValueLow(Box<Ticket>),

    #[error("ticket redemption error: {0}")]
    RedeemError(#[source] anyhow::Error),

    #[error("storage error: {0}")]
    StoreError(#[source] anyhow::Error),

    #[error(transparent)]
    Other(anyhow::Error),
}

impl TicketManagerError {
    pub fn redeem<E: Into<anyhow::Error>>(error: E) -> Self {
        Self::RedeemError(error.into())
    }

    pub fn store<E: Into<anyhow::Error>>(error: E) -> Self {
        Self::StoreError(error.into())
    }
}
