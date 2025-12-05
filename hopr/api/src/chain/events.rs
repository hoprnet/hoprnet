pub use hopr_chain_types::chain_events::ChainEvent;

/// Indicates if the current state should be emitted in the form of events
/// in the [subscription stream](ChainEvents::subscribe).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum StateSyncOptions {
    /// The current state of channels is emitted as [`ChainEvent::ChannelOpened`] events
    /// in the stream, preceding all the future events.
    OpenedChannels,
    /// The current state of **public** accounts is emitted as [`ChainEvent::Announcement`] events
    /// in the stream, preceding all the future events of that kind.
    PublicAccounts,
    /// The current state of all accounts (also private ones) is emitted as [`ChainEvent::Announcement`] events
    /// in the stream, preceding all the future events of that kind.
    AllAccounts,
}

/// Allows subscribing to on-chain events.
#[auto_impl::auto_impl(&, Box, Arc)]
pub trait ChainEvents {
    type Error: std::error::Error + Send + Sync;

    /// Convenience method for subscribing to on-chain events without specifying any state sync options.
    ///
    /// See [`ChainEvents::subscribe_with_state_sync`].
    fn subscribe(&self) -> Result<impl futures::Stream<Item = ChainEvent> + Send + 'static, Self::Error> {
        self.subscribe_with_state_sync(None)
    }

    /// Subscribe to on-chain events.
    ///
    /// The [`options`](StateSyncOptions) specify which parts of the current state should be streamed
    /// in the form on [`ChainEvents`](ChainEvent) before any future events are streamed.
    ///
    /// When an empty iterator (or simply `None`) is specified, only all future events are streamed from this point.
    fn subscribe_with_state_sync<I: IntoIterator<Item = StateSyncOptions>>(
        &self,
        options: I,
    ) -> Result<impl futures::Stream<Item = ChainEvent> + Send + 'static, Self::Error>;
}
