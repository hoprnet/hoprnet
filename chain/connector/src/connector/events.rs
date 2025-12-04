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

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use futures::StreamExt;
    use hex_literal::hex;
    use hopr_api::chain::{
        ChainEvent, ChainEvents, ChainWriteAccountOperations, ChainWriteChannelOperations, StateSyncOptions,
    };
    use hopr_crypto_types::prelude::*;
    use hopr_internal_types::prelude::*;
    use hopr_primitive_types::prelude::*;

    use crate::{
        connector::tests::{MODULE_ADDR, PRIVATE_KEY_1, PRIVATE_KEY_2, create_connector},
        testing::BlokliTestStateBuilder,
    };

    #[tokio::test]
    async fn connector_should_stream_new_events() -> anyhow::Result<()> {
        let offchain_key_2 = OffchainKeypair::from_secret(&hex!(
            "71bf1f42ebbfcd89c3e197a3fd7cda79b92499e509b6fefa0fe44d02821d146a"
        ))?;
        let account_2 = AccountEntry {
            public_key: *offchain_key_2.public(),
            chain_addr: ChainKeypair::from_secret(&PRIVATE_KEY_2)?.public().to_address(),
            entry_type: AccountType::NotAnnounced,
            safe_address: Some([2u8; Address::SIZE].into()),
            key_id: 1.into(),
        };

        let blokli_client = BlokliTestStateBuilder::default()
            .with_accounts([(account_2.clone(), HoprBalance::new_base(100), XDaiBalance::new_base(1))])
            .with_balances([(
                ChainKeypair::from_secret(&PRIVATE_KEY_1)?.public().to_address(),
                XDaiBalance::new_base(1),
            )])
            .with_balances([([3u8; Address::SIZE].into(), HoprBalance::new_base(100))])
            .with_safe_allowances([([3u8; Address::SIZE].into(), HoprBalance::new_base(10000))])
            .with_hopr_network_chain_info("rotsee")
            .build_dynamic_client(MODULE_ADDR.into())
            .with_tx_simulation_delay(Duration::from_millis(100));

        let mut connector = create_connector(blokli_client)?;
        connector.connect().await?;

        let jh = tokio::task::spawn(connector.subscribe()?.take(2).collect::<Vec<_>>());

        let offchain_key_1 = OffchainKeypair::from_secret(&hex!(
            "60741b83b99e36aa0c1331578156e16b8e21166d01834abb6c64b103f885734d"
        ))?;
        let multiaddress: Multiaddr = "/ip4/127.0.0.1/tcp/1234".parse()?;

        connector.register_safe(&[3u8; Address::SIZE].into()).await?.await?;

        connector
            .announce(&[multiaddress.clone()], &offchain_key_1)
            .await?
            .await?;

        connector.open_channel(&account_2.chain_addr, 10.into()).await?.await?;

        let events = jh.await?;

        assert!(
            matches!(&events[0], ChainEvent::Announcement(acc) if &acc.public_key == offchain_key_1.public() && acc.entry_type == AccountType::Announced(vec![multiaddress]))
        );
        assert!(
            matches!(&events[1], ChainEvent::ChannelOpened(channel) if channel.get_id() == &generate_channel_id(&ChainKeypair::from_secret(&PRIVATE_KEY_1)?.public().to_address(), &account_2.chain_addr))
        );

        Ok(())
    }

    #[tokio::test]
    async fn connector_should_stream_existing_state() -> anyhow::Result<()> {
        let offchain_key_1 = OffchainKeypair::from_secret(&hex!(
            "60741b83b99e36aa0c1331578156e16b8e21166d01834abb6c64b103f885734d"
        ))?;
        let account_1 = AccountEntry {
            public_key: *offchain_key_1.public(),
            chain_addr: ChainKeypair::from_secret(&PRIVATE_KEY_1)?.public().to_address(),
            entry_type: AccountType::Announced(vec!["/ip4/1.2.3.4/tcp/1234".parse()?]),
            safe_address: Some([1u8; Address::SIZE].into()),
            key_id: 1.into(),
        };
        let offchain_key_2 = OffchainKeypair::from_secret(&hex!(
            "71bf1f42ebbfcd89c3e197a3fd7cda79b92499e509b6fefa0fe44d02821d146a"
        ))?;
        let account_2 = AccountEntry {
            public_key: *offchain_key_2.public(),
            chain_addr: ChainKeypair::from_secret(&PRIVATE_KEY_2)?.public().to_address(),
            entry_type: AccountType::NotAnnounced,
            safe_address: Some([2u8; Address::SIZE].into()),
            key_id: 2.into(),
        };

        let channel_1 = ChannelEntry::new(
            ChainKeypair::from_secret(&PRIVATE_KEY_1)?.public().to_address(),
            ChainKeypair::from_secret(&PRIVATE_KEY_2)?.public().to_address(),
            10.into(),
            1,
            ChannelStatus::Open,
            1,
        );

        let channel_2 = ChannelEntry::new(
            ChainKeypair::from_secret(&PRIVATE_KEY_2)?.public().to_address(),
            ChainKeypair::from_secret(&PRIVATE_KEY_1)?.public().to_address(),
            15.into(),
            2,
            ChannelStatus::PendingToClose(std::time::SystemTime::UNIX_EPOCH + Duration::from_mins(10)),
            1,
        );

        let blokli_client = BlokliTestStateBuilder::default()
            .with_accounts([
                (account_1.clone(), HoprBalance::new_base(100), XDaiBalance::new_base(1)),
                (account_2.clone(), HoprBalance::new_base(100), XDaiBalance::new_base(1)),
            ])
            .with_channels([channel_1, channel_2])
            .with_hopr_network_chain_info("rotsee")
            .build_static_client();

        let mut connector = create_connector(blokli_client)?;
        connector.connect().await?;

        let accounts = connector
            .subscribe_with_state_sync([StateSyncOptions::PublicAccounts])?
            .take(1)
            .collect::<Vec<_>>()
            .await;
        assert!(matches!(&accounts[0], ChainEvent::Announcement(acc) if acc == &account_1));

        let accounts = connector
            .subscribe_with_state_sync([StateSyncOptions::AllAccounts])?
            .take(2)
            .collect::<Vec<_>>()
            .await;
        assert!(matches!(&accounts[0], ChainEvent::Announcement(acc) if acc == &account_1));
        assert!(matches!(&accounts[1], ChainEvent::Announcement(acc) if acc == &account_2));

        let channels = connector
            .subscribe_with_state_sync([StateSyncOptions::OpenedChannels])?
            .take(1)
            .collect::<Vec<_>>()
            .await;
        assert!(matches!(&channels[0], ChainEvent::ChannelOpened(ch) if ch == &channel_1));

        Ok(())
    }
}
