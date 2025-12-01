use std::str::FromStr;

use blokli_client::api::{BlokliQueryClient, BlokliTransactionClient};
use futures::{FutureExt, StreamExt, TryFutureExt, future::BoxFuture, stream::BoxStream};
use hopr_api::chain::{AccountSelector, AnnouncementError, ChainReceipt, Multiaddr, SafeRegistrationError};
use hopr_chain_types::prelude::*;
use hopr_crypto_types::prelude::*;
use hopr_internal_types::{
    account::AccountEntry,
    prelude::{AnnouncementData, KeyBinding},
};
use hopr_primitive_types::prelude::*;

use crate::{backend::Backend, connector::HoprBlockchainConnector, errors::ConnectorError};

impl<B, C, P, R> HoprBlockchainConnector<C, B, P, R>
where
    B: Backend + Send + Sync + 'static,
{
    pub(crate) fn build_account_stream(
        &self,
        selector: AccountSelector,
    ) -> Result<impl futures::Stream<Item = AccountEntry> + Send + 'static, ConnectorError> {
        let accounts = self.graph.read().nodes().collect::<Vec<_>>();
        let backend = self.backend.clone();
        Ok(futures::stream::iter(accounts).filter_map(move |account_id| {
            let backend = backend.clone();
            let selector = selector.clone();
            // This avoids the cache on purpose so it does not get spammed
            async move {
                match hopr_async_runtime::prelude::spawn_blocking(move || backend.get_account_by_id(&account_id)).await
                {
                    Ok(Ok(value)) => value.filter(|c| selector.satisfies(c)),
                    Ok(Err(error)) => {
                        tracing::error!(%error, %account_id, "backend error when looking up account");
                        None
                    }
                    Err(error) => {
                        tracing::error!(%error, %account_id, "join error when looking up account");
                        None
                    }
                }
            }
        }))
    }
}

#[async_trait::async_trait]
impl<B, C, P, R> hopr_api::chain::ChainReadAccountOperations for HoprBlockchainConnector<C, B, P, R>
where
    B: Backend + Send + Sync + 'static,
    C: BlokliQueryClient + Send + Sync + 'static,
    P: Send + Sync + 'static,
    R: Send + Sync,
{
    type Error = ConnectorError;

    async fn get_balance<Cy: Currency, A: Into<Address> + Send>(&self, address: A) -> Result<Balance<Cy>, Self::Error> {
        self.check_connection_state()?;

        let address = address.into();
        if Cy::is::<WxHOPR>() {
            Ok(self
                .client
                .query_token_balance(&address.into())
                .await?
                .balance
                .0
                .parse()?)
        } else if Cy::is::<XDai>() {
            Ok(self
                .client
                .query_native_balance(&address.into())
                .await?
                .balance
                .0
                .parse()?)
        } else {
            Err(ConnectorError::InvalidState("unsupported currency"))
        }
    }

    async fn safe_allowance<Cy: Currency, A: Into<Address> + Send>(
        &self,
        address: A,
    ) -> Result<Balance<Cy>, Self::Error> {
        self.check_connection_state()?;

        let address = address.into();
        if Cy::is::<WxHOPR>() {
            Ok(self
                .client
                .query_safe_allowance(&address.into())
                .await?
                .allowance
                .0
                .parse()?)
        } else if Cy::is::<XDai>() {
            Err(ConnectorError::InvalidState("cannot query allowance on xDai"))
        } else {
            Err(ConnectorError::InvalidState("unsupported currency"))
        }
    }

    async fn stream_accounts<'a>(
        &'a self,
        selector: AccountSelector,
    ) -> Result<BoxStream<'a, AccountEntry>, Self::Error> {
        self.check_connection_state()?;

        Ok(self.build_account_stream(selector)?.boxed())
    }

    async fn count_accounts(&self, selector: AccountSelector) -> Result<usize, Self::Error> {
        self.check_connection_state()?;

        Ok(self.stream_accounts(selector).await?.count().await)
    }
}

#[async_trait::async_trait]
impl<B, C, P> hopr_api::chain::ChainWriteAccountOperations for HoprBlockchainConnector<C, B, P, P::TxRequest>
where
    B: Send + Sync,
    C: BlokliTransactionClient + BlokliQueryClient + Send + Sync + 'static,
    P: PayloadGenerator + Send + Sync + 'static,
    P::TxRequest: Send + Sync + 'static,
{
    type Error = ConnectorError;

    async fn announce(
        &self,
        multiaddrs: &[Multiaddr],
        key: &OffchainKeypair,
    ) -> Result<BoxFuture<'_, Result<ChainReceipt, Self::Error>>, AnnouncementError<Self::Error>> {
        self.check_connection_state()
            .map_err(AnnouncementError::ProcessingError)?;

        let new_announced_addrs = ahash::HashSet::from_iter(multiaddrs.iter().map(|a| a.to_string()));

        let existing_account = self
            .client
            .query_accounts(blokli_client::api::v1::AccountSelector::Address(
                self.chain_key.public().to_address().into(),
            ))
            .await
            .map_err(|e| AnnouncementError::ProcessingError(ConnectorError::from(e)))?
            .into_iter()
            .find(|account| OffchainPublicKey::from_str(&account.packet_key).is_ok_and(|k| &k == key.public()));

        if let Some(account) = &existing_account {
            let old_announced_addrs = ahash::HashSet::from_iter(account.multi_addresses.iter().cloned());
            if old_announced_addrs == new_announced_addrs || old_announced_addrs.is_superset(&new_announced_addrs) {
                return Err(AnnouncementError::AlreadyAnnounced);
            }
        }

        let key_binding = KeyBinding::new(self.chain_key.public().to_address(), key);

        let tx_req = self
            .payload_generator
            .announce(
                AnnouncementData::new(key_binding, multiaddrs.first().cloned())
                    .map_err(|e| AnnouncementError::ProcessingError(ConnectorError::OtherError(e.into())))?,
                existing_account
                    .map(|_| HoprBalance::zero())
                    .unwrap_or(self.cfg.new_key_binding_fee),
            )
            .map_err(|e| AnnouncementError::ProcessingError(ConnectorError::from(e)))?;

        Ok(self
            .send_tx(tx_req)
            .map_err(AnnouncementError::ProcessingError)
            .await?
            .boxed())
    }

    async fn withdraw<Cy: Currency + Send>(
        &self,
        balance: Balance<Cy>,
        recipient: &Address,
    ) -> Result<BoxFuture<'_, Result<ChainReceipt, Self::Error>>, Self::Error> {
        self.check_connection_state()?;

        let tx_req = self.payload_generator.transfer(*recipient, balance)?;

        Ok(self.send_tx(tx_req).await?.boxed())
    }

    async fn register_safe(
        &self,
        safe_address: &Address,
    ) -> Result<BoxFuture<'_, Result<ChainReceipt, Self::Error>>, SafeRegistrationError<Self::Error>> {
        self.check_connection_state()
            .map_err(SafeRegistrationError::ProcessingError)?;

        if let Some(safe) = self
            .client
            .query_accounts(blokli_client::api::v1::AccountSelector::Address(
                self.chain_key.public().to_address().into(),
            ))
            .await
            .map_err(|e| SafeRegistrationError::ProcessingError(ConnectorError::from(e)))?
            .iter()
            .find_map(|account| account.safe_address.clone())
        {
            return Err(SafeRegistrationError::AlreadyRegistered(safe.parse().unwrap_or_else(
                |e| {
                    tracing::error!("failed to parse safe {safe} address: {e}");
                    Address::default()
                },
            )));
        }

        let tx_req = self
            .payload_generator
            .register_safe_by_node(*safe_address)
            .map_err(|e| SafeRegistrationError::ProcessingError(ConnectorError::from(e)))?;

        Ok(self
            .send_tx(tx_req)
            .map_err(SafeRegistrationError::ProcessingError)
            .await?
            .boxed())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::time::Duration;
    use hex_literal::hex;
    use hopr_api::chain::{ChainReadAccountOperations, ChainWriteAccountOperations};
    use hopr_internal_types::account::AccountType;

    use crate::{
        connector::tests::create_connector,
        testing::BlokliTestStateBuilder
    };
    use crate::connector::tests::{PRIVATE_KEY_1, MODULE_ADDR};

    #[tokio::test]
    async fn connector_should_stream_and_count_accounts() -> anyhow::Result<()> {
        let account = AccountEntry {
            public_key: *OffchainKeypair::random().public(),
            chain_addr: [1u8; Address::SIZE].into(),
            entry_type: AccountType::NotAnnounced,
            safe_address: Some([2u8; Address::SIZE].into()),
            key_id: 1.into(),
        };

        let blokli_client = BlokliTestStateBuilder::default()
            .with_accounts([(
                account.clone(),
                HoprBalance::new_base(100),
                XDaiBalance::new_base(1),
                )
            ])
            .build_static_client();

        let mut connector = create_connector(blokli_client)?;
        connector.connect(Duration::from_secs(2)).await?;

        let accounts = connector
            .stream_accounts(AccountSelector::default())
            .await?
            .collect::<Vec<_>>().await;

        let count = connector
            .count_accounts(AccountSelector::default())
            .await?;

        assert_eq!(accounts.len(), 1);
        assert_eq!(count, 1);
        assert_eq!(&accounts[0], &account);

        Ok(())
    }

    #[tokio::test]
    async fn connector_should_stream_and_count_accounts_with_selector() -> anyhow::Result<()> {
        let account_1 = AccountEntry {
            public_key: *OffchainKeypair::random().public(),
            chain_addr: [1u8; Address::SIZE].into(),
            entry_type: AccountType::NotAnnounced,
            safe_address: Some([2u8; Address::SIZE].into()),
            key_id: 1.into(),
        };

        let account_2 = AccountEntry {
            public_key: *OffchainKeypair::random().public(),
            chain_addr: [2u8; Address::SIZE].into(),
            entry_type: AccountType::Announced(vec!["/ip4/1.2.3.4/tcp/1234".parse()?]),
            safe_address: Some([3u8; Address::SIZE].into()),
            key_id: 2.into(),
        };

        let blokli_client = BlokliTestStateBuilder::default()
            .with_accounts([
                (
                    account_1.clone(),
                    HoprBalance::new_base(100),
                    XDaiBalance::new_base(1),
                ),
                (
                    account_2.clone(),
                    HoprBalance::new_base(100),
                    XDaiBalance::new_base(1),
                )
            ])
            .build_static_client();

        let mut connector = create_connector(blokli_client)?;
        connector.connect(Duration::from_secs(2)).await?;

        let selector = AccountSelector::default()
            .with_chain_key(account_1.chain_addr);
        let accounts = connector
            .stream_accounts(selector)
            .await?
            .collect::<Vec<_>>()
            .await;
        let count = connector
            .count_accounts(selector)
            .await?;

        assert_eq!(count, 1);
        assert_eq!(accounts.len(), 1);
        assert_eq!(&accounts[0], &account_1);

        let selector = AccountSelector::default()
            .with_offchain_key(account_1.public_key);
        let accounts = connector
            .stream_accounts(selector)
            .await?
            .collect::<Vec<_>>().await;
        let count = connector
            .count_accounts(selector)
            .await?;

        assert_eq!(count, 1);
        assert_eq!(accounts.len(), 1);
        assert_eq!(&accounts[0], &account_1);

        let selector = AccountSelector::default()
            .with_public_only(true);
        let accounts = connector
            .stream_accounts(selector)
            .await?
            .collect::<Vec<_>>().await;
        let count = connector
            .count_accounts(selector)
            .await?;

        assert_eq!(count, 1);
        assert_eq!(accounts.len(), 1);
        assert_eq!(&accounts[0], &account_2);

        let selector = AccountSelector::default()
            .with_chain_key(account_1.chain_addr)
            .with_public_only(true);
        let accounts = connector
            .stream_accounts(selector)
            .await?
            .collect::<Vec<_>>().await;
        let count = connector
            .count_accounts(selector)
            .await?;

        assert_eq!(count, 0);
        assert!(accounts.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn connector_should_get_balance_and_safe_allowance() -> anyhow::Result<()> {
        let account = AccountEntry {
            public_key: *OffchainKeypair::random().public(),
            chain_addr: [1u8; Address::SIZE].into(),
            entry_type: AccountType::NotAnnounced,
            safe_address: Some([2u8; Address::SIZE].into()),
            key_id: 1.into(),
        };

        let blokli_client = BlokliTestStateBuilder::default()
            .with_accounts([(
                account.clone(),
                HoprBalance::new_base(100),
                XDaiBalance::new_base(1),
            )
            ])
            .with_safe_allowances([(account.safe_address.unwrap(), HoprBalance::new_base(10000))])
            .build_static_client();

        let mut connector = create_connector(blokli_client)?;
        connector.connect(Duration::from_secs(2)).await?;

        assert_eq!(connector.get_balance(account.safe_address.unwrap()).await?, HoprBalance::new_base(100));
        assert_eq!(connector.get_balance(account.chain_addr).await?, XDaiBalance::new_base(1));
        assert_eq!(connector.safe_allowance(account.safe_address.unwrap()).await?, HoprBalance::new_base(10000));

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn connector_should_announce_new_account_with_multiaddresses() -> anyhow::Result<()> {
        let blokli_client = BlokliTestStateBuilder::default()
            .with_balances([(ChainKeypair::from_secret(&PRIVATE_KEY_1)?.public().to_address(), XDaiBalance::new_base(1))])
            .with_hopr_network_chain_info(1, "rotsee")
            .build_dynamic_client(MODULE_ADDR.into());

        let mut connector = create_connector(blokli_client)?;
        connector.connect(Duration::from_secs(2)).await?;

        let offchain_key = OffchainKeypair::from_secret(&hex!("60741b83b99e36aa0c1331578156e16b8e21166d01834abb6c64b103f885734d"))?;
        let multiaddress = Multiaddr::from_str("/ip4/127.0.0.1/tcp/1234")?;

        connector.announce(&[multiaddress], &offchain_key)
            .await?
            .await?;

        insta::assert_yaml_snapshot!(*connector.client.snapshot());

        let accounts = connector.stream_accounts(AccountSelector::default().with_public_only(true))
            .await?
            .collect::<Vec<_>>()
            .await;

        assert_eq!(accounts.len(), 1);
        assert_eq!(
            accounts[0].get_multiaddrs(),
            &[Multiaddr::from_str("/ip4/127.0.0.1/tcp/1234")?]
        );

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn connector_should_announce_new_account_without_multiaddresses() -> anyhow::Result<()> {
        let blokli_client = BlokliTestStateBuilder::default()
            .with_hopr_network_chain_info(1, "rotsee")
            .with_balances([(ChainKeypair::from_secret(&PRIVATE_KEY_1)?.public().to_address(), XDaiBalance::new_base(1))])
            .build_dynamic_client(MODULE_ADDR.into());

        let mut connector = create_connector(blokli_client)?;
        connector.connect(Duration::from_secs(2)).await?;

        let offchain_key = OffchainKeypair::from_secret(&hex!("60741b83b99e36aa0c1331578156e16b8e21166d01834abb6c64b103f885734d"))?;

        connector.announce(&[], &offchain_key)
            .await?
            .await?;

        insta::assert_yaml_snapshot!(*connector.client.snapshot());

        let accounts = connector.stream_accounts(AccountSelector::default())
            .await?
            .collect::<Vec<_>>()
            .await;

        assert_eq!(accounts.len(), 1);
        assert!(accounts[0].get_multiaddrs().is_empty());

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn connector_should_not_reannounce_when_existing_account_has_same_multiaddresses() -> anyhow::Result<()> {
        let offchain_key = OffchainKeypair::from_secret(&hex!("60741b83b99e36aa0c1331578156e16b8e21166d01834abb6c64b103f885734d"))?;
        let multiaddr: Multiaddr = "/ip4/127.0.0.1/tcp/1234".parse()?;
        let account = AccountEntry {
            public_key: *offchain_key.public(),
            chain_addr: ChainKeypair::from_secret(&PRIVATE_KEY_1)?.public().to_address(),
            entry_type: AccountType::Announced(vec![multiaddr.clone()]),
            safe_address: Some([2u8; Address::SIZE].into()),
            key_id: 1.into(),
        };

        let blokli_client = BlokliTestStateBuilder::default()
            .with_accounts([
                (account.clone(), HoprBalance::new_base(100), XDaiBalance::new_base(1))
            ])
            .with_hopr_network_chain_info(1, "rotsee")
            .build_dynamic_client(MODULE_ADDR.into());

        let mut connector = create_connector(blokli_client)?;
        connector.connect(Duration::from_secs(2)).await?;

        assert!(matches!(connector.announce(&[], &offchain_key).await, Err(AnnouncementError::AlreadyAnnounced)));

        assert!(matches!(connector.announce(&[multiaddr], &offchain_key).await, Err(AnnouncementError::AlreadyAnnounced)));

        insta::assert_yaml_snapshot!(*connector.client.snapshot());

        Ok(())
    }

    #[tokio::test]
    async fn connector_should_reannounce_when_existing_account_has_no_multiaddresses() -> anyhow::Result<()> {
        let offchain_key = OffchainKeypair::from_secret(&hex!("60741b83b99e36aa0c1331578156e16b8e21166d01834abb6c64b103f885734d"))?;
        let multiaddr: Multiaddr = "/ip4/127.0.0.1/tcp/1234".parse()?;
        let account = AccountEntry {
            public_key: *offchain_key.public(),
            chain_addr: ChainKeypair::from_secret(&PRIVATE_KEY_1)?.public().to_address(),
            entry_type: AccountType::NotAnnounced,
            safe_address: Some([2u8; Address::SIZE].into()),
            key_id: 1.into(),
        };

        let blokli_client = BlokliTestStateBuilder::default()
            .with_accounts([
                (account.clone(), HoprBalance::new_base(100), XDaiBalance::new_base(1))
            ])
            .with_hopr_network_chain_info(1, "rotsee")
            .build_dynamic_client(MODULE_ADDR.into());

        let mut connector = create_connector(blokli_client)?;
        connector.connect(Duration::from_secs(2)).await?;

        assert!(matches!(connector.announce(&[], &offchain_key).await, Err(AnnouncementError::AlreadyAnnounced)));

        connector.announce(&[multiaddr.clone()], &offchain_key).await?.await?;

        insta::assert_yaml_snapshot!(*connector.client.snapshot());

        let accounts = connector.stream_accounts(AccountSelector::default().with_public_only(true))
            .await?
            .collect::<Vec<_>>()
            .await;

        assert_eq!(accounts.len(), 1);
        assert_eq!(
            accounts[0].get_multiaddrs(),
            &[multiaddr]
        );

        Ok(())
    }

    #[tokio::test]
    async fn connector_should_withdraw() -> anyhow::Result<()> {
        let blokli_client = BlokliTestStateBuilder::default()
            .with_balances([([1u8; Address::SIZE].into(), HoprBalance::zero())])
            .with_balances([([1u8; Address::SIZE].into(), XDaiBalance::zero())])
            .with_balances([(ChainKeypair::from_secret(&PRIVATE_KEY_1)?.public().to_address(), XDaiBalance::new_base(10))])
            .with_balances([(ChainKeypair::from_secret(&PRIVATE_KEY_1)?.public().to_address(), HoprBalance::new_base(1000))])
            .with_hopr_network_chain_info(1, "rotsee")
            .build_dynamic_client(MODULE_ADDR.into());

        let mut connector = create_connector(blokli_client)?;
        connector.connect(Duration::from_secs(2)).await?;

        connector.withdraw(HoprBalance::new_base(10), &[1u8; Address::SIZE].into()).await?.await?;
        connector.withdraw(XDaiBalance::new_base(1), &[1u8; Address::SIZE].into()).await?.await?;

        insta::assert_yaml_snapshot!(*connector.client.snapshot());

        Ok(())
    }

    #[tokio::test]
    async fn connector_should_register_safe() -> anyhow::Result<()> {
        let blokli_client = BlokliTestStateBuilder::default()
            .with_balances([(ChainKeypair::from_secret(&PRIVATE_KEY_1)?.public().to_address(), XDaiBalance::new_base(10))])
            .with_hopr_network_chain_info(1, "rotsee")
            .build_dynamic_client(MODULE_ADDR.into());

        let mut connector = create_connector(blokli_client)?;
        connector.connect(Duration::from_secs(2)).await?;

        connector.register_safe(&[1u8; Address::SIZE].into()).await?.await?;

        insta::assert_yaml_snapshot!(*connector.client.snapshot());

        Ok(())
    }

    #[tokio::test]
    async fn connector_should_not_re_register_safe() -> anyhow::Result<()> {
        let offchain_key = OffchainKeypair::from_secret(&hex!("60741b83b99e36aa0c1331578156e16b8e21166d01834abb6c64b103f885734d"))?;
        let safe_addr: Address = [2u8; Address::SIZE].into();
        let account = AccountEntry {
            public_key: *offchain_key.public(),
            chain_addr: ChainKeypair::from_secret(&PRIVATE_KEY_1)?.public().to_address(),
            entry_type: AccountType::NotAnnounced,
            safe_address: Some(safe_addr),
            key_id: 1.into(),
        };

        let blokli_client = BlokliTestStateBuilder::default()
            .with_accounts([(account, HoprBalance::new_base(100), XDaiBalance::new_base(1))])
            .with_balances([(ChainKeypair::from_secret(&PRIVATE_KEY_1)?.public().to_address(), XDaiBalance::new_base(10))])
            .with_hopr_network_chain_info(1, "rotsee")
            .build_dynamic_client(MODULE_ADDR.into());

        let mut connector = create_connector(blokli_client)?;
        connector.connect(Duration::from_secs(2)).await?;

        assert!(matches!(connector.register_safe(&[1u8; Address::SIZE].into()).await, Err(SafeRegistrationError::AlreadyRegistered(a)) if a == safe_addr));
        assert!(matches!(connector.register_safe(&safe_addr).await, Err(SafeRegistrationError::AlreadyRegistered(a)) if a == safe_addr));

        insta::assert_yaml_snapshot!(*connector.client.snapshot());

        Ok(())
    }
}
