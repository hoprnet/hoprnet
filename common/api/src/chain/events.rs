pub use hopr_chain_types::chain_events::{ChainEventType, SignificantChainEvent};

/// Allows subscribing to on-chain events.
pub trait ChainEvents {
    type Error: std::error::Error + Send + Sync;

    /// Subscribe to on-chain events.
    fn subscribe(&self) -> Result<impl futures::Stream<Item = SignificantChainEvent> + Send + 'static, Self::Error>;
}
