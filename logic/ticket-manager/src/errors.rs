use hopr_api::chain::TicketRedeemError;

#[derive(Debug, thiserror::Error)]
pub enum TicketManagerError<Cerr, Qerr> {
    #[error("channel not found")]
    ChannelNotFound,

    #[error("already redeeming in this channel")]
    AlreadyRedeeming,

    #[error("ticket redemption error: {0}")]
    RedeemError(#[from] TicketRedeemError<Cerr>),

    #[error("queue error: {0}")]
    QueueError(Qerr),
}
