pub use hopr_chain_types::chain_events::ChainEvent;

/// Allows subscribing to on-chain events.
#[auto_impl::auto_impl(&, Box, Arc)]
pub trait ChainEvents {
    type Error: std::error::Error + Send + Sync;

    /// Subscribe to on-chain events.
    fn subscribe(&self) -> Result<impl futures::Stream<Item = ChainEvent> + Send + 'static, Self::Error>;
}
