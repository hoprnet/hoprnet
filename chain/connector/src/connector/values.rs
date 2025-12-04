use std::time::Duration;

use blokli_client::api::BlokliQueryClient;
use futures::TryFutureExt;
use hopr_api::chain::{ChainInfo, DomainSeparators};
use hopr_internal_types::prelude::WinningProbability;
use hopr_primitive_types::prelude::*;

use crate::{
    connector::{
        HoprBlockchainConnector,
        utils::{ParsedChainInfo, model_to_chain_info},
    },
    errors::ConnectorError,
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

    async fn domain_separators(&self) -> Result<DomainSeparators, Self::Error> {
        self.check_connection_state()?;

        Ok(self.query_cached_chain_info().await?.domain_separators)
    }

    async fn minimum_incoming_ticket_win_prob(&self) -> Result<WinningProbability, Self::Error> {
        self.check_connection_state()?;

        Ok(self.query_cached_chain_info().await?.ticket_win_prob)
    }

    async fn minimum_ticket_price(&self) -> Result<HoprBalance, Self::Error> {
        self.check_connection_state()?;

        Ok(self.query_cached_chain_info().await?.ticket_price)
    }

    async fn key_binding_fee(&self) -> Result<HoprBalance, Self::Error> {
        self.check_connection_state()?;
        Ok(self.query_cached_chain_info().await?.key_binding_fee)
    }

    async fn channel_closure_notice_period(&self) -> Result<Duration, Self::Error> {
        self.check_connection_state()?;

        Ok(self.query_cached_chain_info().await?.channel_closure_grace_period)
    }

    async fn chain_info(&self) -> Result<ChainInfo, Self::Error> {
        self.check_connection_state()?;

        Ok(self.query_cached_chain_info().await?.info)
    }
}

#[cfg(test)]
mod tests {
    use hopr_api::chain::ChainValues;
    use hopr_crypto_types::types::Hash;

    use super::*;
    use crate::{connector::tests::create_connector, testing::BlokliTestStateBuilder};

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

        Ok(())
    }
}
