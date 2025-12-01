use blokli_client::api::{BlokliQueryClient, BlokliTransactionClient};
use futures::{FutureExt, StreamExt, TryFutureExt, future::BoxFuture, stream::BoxStream};
use hopr_api::chain::{ChainReceipt, ChannelSelector};
use hopr_chain_types::prelude::*;
use hopr_crypto_types::prelude::Keypair;
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::*;

use crate::{backend::Backend, connector::HoprBlockchainConnector, errors::ConnectorError};

impl<B, C, P, R> HoprBlockchainConnector<C, B, P, R>
where
    B: Backend + Send + Sync + 'static,
{
    pub(crate) fn build_channel_stream(
        &self,
        selector: ChannelSelector,
    ) -> Result<impl futures::Stream<Item = ChannelEntry> + Send + 'static, ConnectorError> {
        // Note: Since the graph does not contain Closed channels, they cannot
        // be selected if requested solely via the ChannelSelector.
        if selector.allowed_states == [ChannelStatusDiscriminants::Closed] {
            return Err(ConnectorError::InvalidArguments("cannot stream closed channels only"));
        }

        let mut channels = self
            .graph
            .read()
            .all_edges()
            .map(|(_, _, e)| e)
            .copied()
            .collect::<Vec<_>>();

        // Ensure the returned channels are always perfectly ordered by their id.
        channels.sort_unstable();

        let backend = self.backend.clone();
        Ok(futures::stream::iter(channels).filter_map(move |channel_id| {
            let backend = backend.clone();
            let selector = selector.clone();
            // This avoids the cache on purpose so it does not get spammed
            async move {
                match hopr_async_runtime::prelude::spawn_blocking(move || backend.get_channel_by_id(&channel_id)).await
                {
                    Ok(Ok(value)) => value.filter(|c| selector.satisfies(c)),
                    Ok(Err(error)) => {
                        tracing::error!(%error, %channel_id, "backend error when looking up channel");
                        None
                    }
                    Err(error) => {
                        tracing::error!(%error, %channel_id, "join error when looking up channel");
                        None
                    }
                }
            }
        }))
    }
}

#[async_trait::async_trait]
impl<B, C, P, R> hopr_api::chain::ChainReadChannelOperations for HoprBlockchainConnector<C, B, P, R>
where
    B: Backend + Send + Sync + 'static,
    C: Send + Sync,
    P: Send + Sync,
    R: Send + Sync,
{
    type Error = ConnectorError;

    fn me(&self) -> &Address {
        self.chain_key.public().as_ref()
    }

    async fn channel_by_parties(&self, src: &Address, dst: &Address) -> Result<Option<ChannelEntry>, Self::Error> {
        self.check_connection_state()?;

        let backend = self.backend.clone();
        let src = *src;
        let dst = *dst;
        Ok(self
            .channel_by_parties
            .try_get_with(ChannelParties::new(src, dst), async move {
                tracing::warn!(%src, %dst, "cache miss on channel_by_parties");
                match hopr_async_runtime::prelude::spawn_blocking(move || {
                    let channel_id = generate_channel_id(&src, &dst);
                    backend.get_channel_by_id(&channel_id)
                })
                .await
                {
                    Ok(Ok(value)) => Ok(value),
                    Ok(Err(e)) => Err(ConnectorError::BackendError(e.into())),
                    Err(e) => Err(ConnectorError::BackendError(e.into())),
                }
            })
            .await?)
    }

    async fn channel_by_id(&self, channel_id: &ChannelId) -> Result<Option<ChannelEntry>, Self::Error> {
        self.check_connection_state()?;

        let channel_id = *channel_id;
        let backend = self.backend.clone();
        Ok(self
            .channel_by_id
            .try_get_with_by_ref(&channel_id, async move {
                tracing::warn!(%channel_id, "cache miss on channel_by_id");
                match hopr_async_runtime::prelude::spawn_blocking(move || backend.get_channel_by_id(&channel_id)).await
                {
                    Ok(Ok(value)) => Ok(value),
                    Ok(Err(e)) => Err(ConnectorError::BackendError(e.into())),
                    Err(e) => Err(ConnectorError::BackendError(e.into())),
                }
            })
            .await?)
    }

    async fn stream_channels<'a>(
        &'a self,
        selector: ChannelSelector,
    ) -> Result<BoxStream<'a, ChannelEntry>, Self::Error> {
        self.check_connection_state()?;

        Ok(self.build_channel_stream(selector)?.boxed())
    }
}

#[async_trait::async_trait]
impl<B, C, P> hopr_api::chain::ChainWriteChannelOperations for HoprBlockchainConnector<C, B, P, P::TxRequest>
where
    B: Backend + Send + Sync + 'static,
    C: BlokliQueryClient + BlokliTransactionClient + Send + Sync + 'static,
    P: PayloadGenerator + Send + Sync + 'static,
    P::TxRequest: Send + Sync + 'static,
{
    type Error = ConnectorError;

    async fn open_channel<'a>(
        &'a self,
        dst: &'a Address,
        amount: HoprBalance,
    ) -> Result<BoxFuture<'a, Result<(ChannelId, ChainReceipt), Self::Error>>, Self::Error> {
        self.check_connection_state()?;

        let id = generate_channel_id(self.chain_key.public().as_ref(), dst);
        let tx_req = self.payload_generator.fund_channel(*dst, amount)?;
        tracing::debug!(channel_id = %id, %dst, %amount, "opening channel");

        Ok(self
            .send_tx(tx_req)
            .await?
            .and_then(move |tx_hash| futures::future::ok((id, tx_hash)))
            .boxed())
    }

    async fn fund_channel<'a>(
        &'a self,
        channel_id: &'a ChannelId,
        amount: HoprBalance,
    ) -> Result<BoxFuture<'a, Result<ChainReceipt, Self::Error>>, Self::Error> {
        self.check_connection_state()?;

        use hopr_api::chain::ChainReadChannelOperations;

        let channel = self
            .channel_by_id(channel_id)
            .await?
            .ok_or_else(|| ConnectorError::ChannelDoesNotExist(*channel_id))?;
        let tx_req = self.payload_generator.fund_channel(channel.destination, amount)?;
        tracing::debug!(%channel_id, %amount, "funding channel");

        Ok(self.send_tx(tx_req).await?.boxed())
    }

    async fn close_channel<'a>(
        &'a self,
        channel_id: &'a ChannelId,
    ) -> Result<BoxFuture<'a, Result<ChainReceipt, Self::Error>>, Self::Error> {
        self.check_connection_state()?;

        use hopr_api::chain::ChainReadChannelOperations;

        let channel = self
            .channel_by_id(channel_id)
            .await?
            .ok_or_else(|| ConnectorError::ChannelDoesNotExist(*channel_id))?;

        let direction = channel.direction(self.me()).ok_or(ConnectorError::InvalidArguments(
            "cannot close channels that is not own",
        ))?;

        let tx_req = match channel.status {
            ChannelStatus::Closed => return Err(ConnectorError::ChannelClosed(*channel_id)),
            ChannelStatus::Open => {
                if direction == ChannelDirection::Outgoing {
                    tracing::debug!(%channel_id, "initiating outgoing channel closure");
                    self.payload_generator
                        .initiate_outgoing_channel_closure(channel.destination)?
                } else {
                    tracing::debug!(%channel_id, "closing incoming channel");
                    self.payload_generator.close_incoming_channel(channel.source)?
                }
            }
            c if c.closure_time_elapsed(&std::time::SystemTime::now()) => {
                if direction == ChannelDirection::Outgoing {
                    tracing::debug!(%channel_id, "finalizing outgoing channel closure");
                    self.payload_generator
                        .finalize_outgoing_channel_closure(channel.destination)?
                } else {
                    tracing::debug!(%channel_id, "closing incoming channel");
                    self.payload_generator.close_incoming_channel(channel.source)?
                }
            }
            _ => return Err(ConnectorError::InvalidState("channel closure time has not elapsed")),
        };

        Ok(self.send_tx(tx_req).await?.boxed())
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;
    use hex_literal::hex;
    use hopr_api::chain::{ChainReadChannelOperations, ChainWriteChannelOperations};
    use hopr_crypto_types::keypairs::{ChainKeypair, OffchainKeypair};
    use crate::connector::tests::{create_connector, MODULE_ADDR, PRIVATE_KEY_1, PRIVATE_KEY_2};
    use crate::testing::BlokliTestStateBuilder;
    use super::*;

    #[tokio::test]
    async fn connector_should_get_and_stream_channels() -> anyhow::Result<()> {
        let offchain_key_1 = OffchainKeypair::from_secret(&hex!("60741b83b99e36aa0c1331578156e16b8e21166d01834abb6c64b103f885734d"))?;
        let account_1 = AccountEntry {
            public_key: *offchain_key_1.public(),
            chain_addr: ChainKeypair::from_secret(&PRIVATE_KEY_1)?.public().to_address(),
            entry_type: AccountType::NotAnnounced,
            safe_address: Some([1u8; Address::SIZE].into()),
            key_id: 1.into(),
        };
        let offchain_key_2 = OffchainKeypair::from_secret(&hex!("71bf1f42ebbfcd89c3e197a3fd7cda79b92499e509b6fefa0fe44d02821d146a"))?;
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
            1
        );

        let channel_2 = ChannelEntry::new(
            ChainKeypair::from_secret(&PRIVATE_KEY_2)?.public().to_address(),
            ChainKeypair::from_secret(&PRIVATE_KEY_1)?.public().to_address(),
            15.into(),
            2,
            ChannelStatus::PendingToClose(std::time::SystemTime::UNIX_EPOCH + Duration::from_mins(10)),
            1
        );

        let blokli_client = BlokliTestStateBuilder::default()
            .with_accounts([
                (account_1, HoprBalance::new_base(100), XDaiBalance::new_base(1)),
                (account_2, HoprBalance::new_base(100), XDaiBalance::new_base(1)),
            ])
            .with_channels([channel_1, channel_2])
            .with_hopr_network_chain_info(1, "rotsee")
            .build_dynamic_client(MODULE_ADDR.into());

        let mut connector = create_connector(blokli_client)?;
        connector.connect(Duration::from_secs(2)).await?;


        assert_eq!(Some(channel_1), connector.channel_by_id(channel_1.get_id()).await?);
        assert_eq!(Some(channel_1), connector.channel_by_parties(&channel_1.source, &channel_1.destination).await?);
        assert_eq!(Some(channel_2), connector.channel_by_id(channel_2.get_id()).await?);
        assert_eq!(Some(channel_2), connector.channel_by_parties(&channel_2.source, &channel_2.destination).await?);

        assert_eq!(vec![channel_1, channel_2], connector.stream_channels(ChannelSelector::default()).await?.collect::<Vec<_>>().await);

        assert_eq!(vec![channel_1], connector.stream_channels(ChannelSelector::default()
            .with_allowed_states(&[ChannelStatusDiscriminants::Open]))
            .await?
            .collect::<Vec<_>>()
            .await
        );
        assert_eq!(vec![channel_2], connector.stream_channels(ChannelSelector::default()
            .with_allowed_states(&[ChannelStatusDiscriminants::PendingToClose]))
            .await?
            .collect::<Vec<_>>()
            .await
        );
        assert_eq!(Vec::<ChannelEntry>::new(), connector.stream_channels(ChannelSelector::default()
            .with_allowed_states(&[ChannelStatusDiscriminants::PendingToClose])
            .with_closure_time_range(DateTime::from(std::time::SystemTime::UNIX_EPOCH + Duration::from_mins(11))..))
            .await?
            .collect::<Vec<_>>()
            .await
        );

        Ok(())
    }

    #[tokio::test]
    async fn connector_should_open_channel() -> anyhow::Result<()> {
        let offchain_key_1 = OffchainKeypair::from_secret(&hex!("60741b83b99e36aa0c1331578156e16b8e21166d01834abb6c64b103f885734d"))?;
        let account_1 = AccountEntry {
            public_key: *offchain_key_1.public(),
            chain_addr: ChainKeypair::from_secret(&PRIVATE_KEY_1)?.public().to_address(),
            entry_type: AccountType::NotAnnounced,
            safe_address: Some([1u8; Address::SIZE].into()),
            key_id: 1.into(),
        };
        let offchain_key_2 = OffchainKeypair::from_secret(&hex!("71bf1f42ebbfcd89c3e197a3fd7cda79b92499e509b6fefa0fe44d02821d146a"))?;
        let account_2 = AccountEntry {
            public_key: *offchain_key_2.public(),
            chain_addr: ChainKeypair::from_secret(&PRIVATE_KEY_2)?.public().to_address(),
            entry_type: AccountType::NotAnnounced,
            safe_address: Some([2u8; Address::SIZE].into()),
            key_id: 2.into(),
        };

        let blokli_client = BlokliTestStateBuilder::default()
            .with_accounts([
                (account_1.clone(), HoprBalance::new_base(100), XDaiBalance::new_base(1)),
                (account_2.clone(), HoprBalance::new_base(100), XDaiBalance::new_base(1)),
            ])
            .with_hopr_network_chain_info(1, "rotsee")
            .build_dynamic_client(MODULE_ADDR.into());

        let mut connector = create_connector(blokli_client)?;
        connector.connect(Duration::from_secs(2)).await?;

        connector.open_channel(&account_2.chain_addr, 10.into()).await?.await?;

        insta::assert_yaml_snapshot!(*connector.client().snapshot());

        Ok(())
    }

    #[tokio::test]
    async fn connector_should_fund_channel() -> anyhow::Result<()> {
        let offchain_key_1 = OffchainKeypair::from_secret(&hex!("60741b83b99e36aa0c1331578156e16b8e21166d01834abb6c64b103f885734d"))?;
        let account_1 = AccountEntry {
            public_key: *offchain_key_1.public(),
            chain_addr: ChainKeypair::from_secret(&PRIVATE_KEY_1)?.public().to_address(),
            entry_type: AccountType::NotAnnounced,
            safe_address: Some([1u8; Address::SIZE].into()),
            key_id: 1.into(),
        };
        let offchain_key_2 = OffchainKeypair::from_secret(&hex!("71bf1f42ebbfcd89c3e197a3fd7cda79b92499e509b6fefa0fe44d02821d146a"))?;
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
            1
        );

        let blokli_client = BlokliTestStateBuilder::default()
            .with_accounts([
                (account_1, HoprBalance::new_base(100), XDaiBalance::new_base(1)),
                (account_2, HoprBalance::new_base(100), XDaiBalance::new_base(1)),
            ])
            .with_channels([channel_1])
            .with_hopr_network_chain_info(1, "rotsee")
            .build_dynamic_client(MODULE_ADDR.into());

        let mut connector = create_connector(blokli_client)?;
        connector.connect(Duration::from_secs(2)).await?;

        connector.fund_channel(&channel_1.get_id(), 5.into()).await?.await?;

        insta::assert_yaml_snapshot!(*connector.client().snapshot());

        Ok(())
    }

    #[tokio::test]
    async fn connector_should_initiate_channel_closure() -> anyhow::Result<()> {
        let offchain_key_1 = OffchainKeypair::from_secret(&hex!("60741b83b99e36aa0c1331578156e16b8e21166d01834abb6c64b103f885734d"))?;
        let account_1 = AccountEntry {
            public_key: *offchain_key_1.public(),
            chain_addr: ChainKeypair::from_secret(&PRIVATE_KEY_1)?.public().to_address(),
            entry_type: AccountType::NotAnnounced,
            safe_address: Some([1u8; Address::SIZE].into()),
            key_id: 1.into(),
        };
        let offchain_key_2 = OffchainKeypair::from_secret(&hex!("71bf1f42ebbfcd89c3e197a3fd7cda79b92499e509b6fefa0fe44d02821d146a"))?;
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
            1
        );

        let blokli_client = BlokliTestStateBuilder::default()
            .with_accounts([
                (account_1, HoprBalance::new_base(100), XDaiBalance::new_base(1)),
                (account_2, HoprBalance::new_base(100), XDaiBalance::new_base(1)),
            ])
            .with_channels([channel_1])
            .with_hopr_network_chain_info(1, "rotsee")
            .build_dynamic_client(MODULE_ADDR.into());

        let mut connector = create_connector(blokli_client)?;
        connector.connect(Duration::from_secs(2)).await?;

        connector.close_channel(&channel_1.get_id()).await?.await?;

        let mut snapshot = (*connector.client().snapshot()).clone();

        // Replace the closure time value to make the snapshot deterministic
        snapshot.channels.get_mut(&hex::encode(channel_1.get_id())).unwrap().closure_time = Some(blokli_client::api::types::DateTime{ 0: "dummy".into() });

        insta::assert_yaml_snapshot!(snapshot);

        Ok(())
    }

    #[tokio::test]
    async fn connector_should_finalize_channel_closure() -> anyhow::Result<()> {
        let offchain_key_1 = OffchainKeypair::from_secret(&hex!("60741b83b99e36aa0c1331578156e16b8e21166d01834abb6c64b103f885734d"))?;
        let account_1 = AccountEntry {
            public_key: *offchain_key_1.public(),
            chain_addr: ChainKeypair::from_secret(&PRIVATE_KEY_1)?.public().to_address(),
            entry_type: AccountType::NotAnnounced,
            safe_address: Some([1u8; Address::SIZE].into()),
            key_id: 1.into(),
        };
        let offchain_key_2 = OffchainKeypair::from_secret(&hex!("71bf1f42ebbfcd89c3e197a3fd7cda79b92499e509b6fefa0fe44d02821d146a"))?;
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
            ChannelStatus::PendingToClose(std::time::SystemTime::UNIX_EPOCH + Duration::from_mins(10)),
            1
        );

        let blokli_client = BlokliTestStateBuilder::default()
            .with_accounts([
                (account_1, HoprBalance::new_base(100), XDaiBalance::new_base(1)),
                (account_2, HoprBalance::new_base(100), XDaiBalance::new_base(1)),
            ])
            .with_channels([channel_1])
            .with_hopr_network_chain_info(1, "rotsee")
            .build_dynamic_client(MODULE_ADDR.into());

        let mut connector = create_connector(blokli_client)?;
        connector.connect(Duration::from_secs(2)).await?;

        connector.close_channel(&channel_1.get_id()).await?.await?;

        insta::assert_yaml_snapshot!(*connector.client().snapshot());

        Ok(())
    }

    #[tokio::test]
    async fn connector_should_close_incoming_channel() -> anyhow::Result<()> {
        let offchain_key_1 = OffchainKeypair::from_secret(&hex!("60741b83b99e36aa0c1331578156e16b8e21166d01834abb6c64b103f885734d"))?;
        let account_1 = AccountEntry {
            public_key: *offchain_key_1.public(),
            chain_addr: ChainKeypair::from_secret(&PRIVATE_KEY_1)?.public().to_address(),
            entry_type: AccountType::NotAnnounced,
            safe_address: Some([1u8; Address::SIZE].into()),
            key_id: 1.into(),
        };
        let offchain_key_2 = OffchainKeypair::from_secret(&hex!("71bf1f42ebbfcd89c3e197a3fd7cda79b92499e509b6fefa0fe44d02821d146a"))?;
        let account_2 = AccountEntry {
            public_key: *offchain_key_2.public(),
            chain_addr: ChainKeypair::from_secret(&PRIVATE_KEY_2)?.public().to_address(),
            entry_type: AccountType::NotAnnounced,
            safe_address: Some([2u8; Address::SIZE].into()),
            key_id: 2.into(),
        };

        let channel_1 = ChannelEntry::new(
            ChainKeypair::from_secret(&PRIVATE_KEY_2)?.public().to_address(),
            ChainKeypair::from_secret(&PRIVATE_KEY_1)?.public().to_address(),
            10.into(),
            1,
            ChannelStatus::Open,
            1
        );

        let blokli_client = BlokliTestStateBuilder::default()
            .with_accounts([
                (account_1, HoprBalance::new_base(100), XDaiBalance::new_base(1)),
                (account_2, HoprBalance::new_base(100), XDaiBalance::new_base(1)),
            ])
            .with_channels([channel_1])
            .with_hopr_network_chain_info(1, "rotsee")
            .build_dynamic_client(MODULE_ADDR.into());

        let mut connector = create_connector(blokli_client)?;
        connector.connect(Duration::from_secs(2)).await?;

        connector.close_channel(&channel_1.get_id()).await?.await?;

        insta::assert_yaml_snapshot!(*connector.client().snapshot());

        Ok(())
    }
}