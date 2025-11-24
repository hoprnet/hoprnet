use ahash::HashSet;
use futures::StreamExt;
use hopr_api::chain::{AccountSelector, ChainEvent, ChannelSelector, StateSyncOptions};
use hopr_internal_types::channels::ChannelStatusDiscriminants;

use crate::{Backend, connector::HoprBlockchainConnector, errors::ConnectorError};

impl<B, C, P, R> hopr_api::chain::ChainEvents for HoprBlockchainConnector<C, B, P, R>
where
    B: Backend + Send + Sync + 'static,
{
    type Error = ConnectorError;

    fn subscribe_with_state_sync<I: IntoIterator<Item = StateSyncOptions>>(
        &self,
        options: I,
    ) -> Result<impl futures::Stream<Item = ChainEvent> + Send + 'static, Self::Error> {
        self.check_connection_state()?;

        let options = options.into_iter().collect::<HashSet<_>>();

        let mut state_stream = futures_concurrency::stream::StreamGroup::new();
        if options.contains(&StateSyncOptions::PublicAccounts) && !options.contains(&StateSyncOptions::AllAccounts) {
            let stream = self
                .build_account_stream(AccountSelector::default().with_public_only(true))?
                .map(ChainEvent::Announcement);
            state_stream.insert(stream.boxed());
        }

        if options.contains(&StateSyncOptions::AllAccounts) {
            let stream = self
                .build_account_stream(AccountSelector::default().with_public_only(false))?
                .map(ChainEvent::Announcement);
            state_stream.insert(stream.boxed());
        }

        if options.contains(&StateSyncOptions::OpenedChannels) {
            let stream = self
                .build_channel_stream(
                    ChannelSelector::default().with_allowed_states(&[ChannelStatusDiscriminants::Open]),
                )?
                .map(ChainEvent::ChannelOpened);
            state_stream.insert(stream.boxed());
        }

        Ok(state_stream.chain(self.events.1.activate_cloned()))
    }
}
