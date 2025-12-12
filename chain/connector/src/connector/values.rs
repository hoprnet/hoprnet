use std::time::Duration;

use blokli_client::api::BlokliQueryClient;
use futures::TryFutureExt;
use hopr_api::chain::{ChainInfo, DomainSeparators};
use hopr_internal_types::prelude::WinningProbability;
use hopr_primitive_types::prelude::*;

use crate::{
    HoprBlockchainReader,
    connector::HoprBlockchainConnector,
    errors::ConnectorError,
    utils::{ParsedChainInfo, model_to_chain_info},
};

pub(crate) const CHAIN_INFO_CACHE_KEY: u32 = 0;

impl<B, C, P, R> HoprBlockchainConnector<C, R, B, P>
where
    C: BlokliQueryClient + Send + Sync + 'static,
{
    pub(crate) async fn query_cached_chain_info(&self) -> Result<ParsedChainInfo, ConnectorError> {
        Ok(self
            .values
            .try_get_with(
                CHAIN_INFO_CACHE_KEY,
                self.client
                    .query_chain_info()
                    .map_err(ConnectorError::from)
                    .and_then(|model| futures::future::ready(model_to_chain_info(model))),
            )
            .await?)
    }
}

#[async_trait::async_trait]
impl<B, R, C, P> hopr_api::chain::ChainValues for HoprBlockchainConnector<C, B, P, R>
where
    B: Send + Sync,
    C: BlokliQueryClient + Send + Sync + 'static,
    P: Send + Sync,
    R: Send + Sync,
{
    type Error = ConnectorError;

    // NOTE: these APIs can be called without calling `connect` first

    #[inline]
    async fn balance<Cy: Currency, A: Into<Address> + Send>(&self, address: A) -> Result<Balance<Cy>, Self::Error> {
        HoprBlockchainReader(self.client.clone()).balance(address).await
    }

    async fn domain_separators(&self) -> Result<DomainSeparators, Self::Error> {
        Ok(self.query_cached_chain_info().await?.domain_separators)
    }

    async fn minimum_incoming_ticket_win_prob(&self) -> Result<WinningProbability, Self::Error> {
        Ok(self.query_cached_chain_info().await?.ticket_win_prob)
    }

    async fn minimum_ticket_price(&self) -> Result<HoprBalance, Self::Error> {
        Ok(self.query_cached_chain_info().await?.ticket_price)
    }

    async fn key_binding_fee(&self) -> Result<HoprBalance, Self::Error> {
        Ok(self.query_cached_chain_info().await?.key_binding_fee)
    }

    async fn channel_closure_notice_period(&self) -> Result<Duration, Self::Error> {
        Ok(self.query_cached_chain_info().await?.channel_closure_grace_period)
    }

    async fn chain_info(&self) -> Result<ChainInfo, Self::Error> {
        Ok(self.query_cached_chain_info().await?.info)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use hopr_api::chain::ChainValues;
    use hopr_crypto_types::prelude::*;
    use hopr_internal_types::account::{AccountEntry, AccountType};

    use super::*;
    use crate::{connector::tests::create_connector, testing::BlokliTestStateBuilder};

    #[tokio::test]
    async fn connector_should_get_balance() -> anyhow::Result<()> {
        let account = AccountEntry {
            public_key: *OffchainKeypair::random().public(),
            chain_addr: [1u8; Address::SIZE].into(),
            entry_type: AccountType::NotAnnounced,
            safe_address: Some([2u8; Address::SIZE].into()),
            key_id: 1.into(),
        };

        let blokli_client = BlokliTestStateBuilder::default()
            .with_accounts([(account.clone(), HoprBalance::new_base(100), XDaiBalance::new_base(1))])
            .with_safe_allowances([(account.safe_address.unwrap(), HoprBalance::new_base(10000))])
            .build_static_client();

        let mut connector = create_connector(blokli_client)?;
        connector.connect().await?;

        assert_eq!(
            connector.balance(account.safe_address.unwrap()).await?,
            HoprBalance::new_base(100)
        );
        assert_eq!(connector.balance(account.chain_addr).await?, XDaiBalance::new_base(1));

        Ok(())
    }

    #[tokio::test]
    async fn connector_should_query_chain_info() -> anyhow::Result<()> {
        let blokli_client = BlokliTestStateBuilder::default()
            .with_hopr_network_chain_info("rotsee")
            .build_static_client();

        let mut connector = create_connector(blokli_client)?;
        connector.connect().await?;

        let chain_info = connector.chain_info().await?;

        assert_eq!(100, chain_info.chain_id);
        assert_eq!("rotsee", &chain_info.hopr_network_name);

        assert_eq!(Duration::from_mins(5), connector.channel_closure_notice_period().await?);
        assert_eq!(HoprBalance::new_base(1), connector.minimum_ticket_price().await?);
        assert!(WinningProbability::ALWAYS.approx_eq(&connector.minimum_incoming_ticket_win_prob().await?));
        assert_eq!(Hash::default(), connector.domain_separators().await?.channel);
        assert_eq!(
            HoprBalance::from_str("0.01 wxHOPR")?,
            connector.key_binding_fee().await?
        );

        Ok(())
    }

    #[tokio::test]
    async fn connector_should_query_chain_info_without_calling_connect_first() -> anyhow::Result<()> {
        let blokli_client = BlokliTestStateBuilder::default()
            .with_hopr_network_chain_info("rotsee")
            .build_static_client();

        let connector = create_connector(blokli_client)?;

        let chain_info = connector.chain_info().await?;

        assert_eq!(100, chain_info.chain_id);
        assert_eq!("rotsee", &chain_info.hopr_network_name);

        assert_eq!(Duration::from_mins(5), connector.channel_closure_notice_period().await?);
        assert_eq!(HoprBalance::new_base(1), connector.minimum_ticket_price().await?);
        assert!(WinningProbability::ALWAYS.approx_eq(&connector.minimum_incoming_ticket_win_prob().await?));
        assert_eq!(Hash::default(), connector.domain_separators().await?.channel);
        assert_eq!(
            HoprBalance::from_str("0.01 wxHOPR")?,
            connector.key_binding_fee().await?
        );

        Ok(())
    }
}
