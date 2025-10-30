use std::time::Duration;
use blokli_client::api::BlokliQueryClient;
use futures::TryFutureExt;
use hopr_api::chain::DomainSeparators;
use hopr_crypto_types::types::Hash;
use hopr_internal_types::prelude::WinningProbability;
use hopr_primitive_types::prelude::*;

use crate::connector::HoprBlockchainConnector;
use crate::errors::ConnectorError;

impl<B,C, P> HoprBlockchainConnector<B,C, P>
where B: Send + Sync,
      C: BlokliQueryClient + Send + Sync + 'static,
      P: Send + Sync
{
    async fn query_cached_chain_info(&self) -> Result<blokli_client::api::types::ChainInfo, ConnectorError> {
        Ok(self.values.try_get_with(0, self.client.query_chain_info().map_err(ConnectorError::from)).await?)
    }
}

#[async_trait::async_trait]
impl<B, C, P> hopr_api::chain::ChainValues for HoprBlockchainConnector<B, C, P>
where
    B: Send + Sync,
    C: BlokliQueryClient + Send + Sync + 'static,
    P: Send + Sync,
{
    type Error = ConnectorError;

    async fn domain_separators(&self) -> Result<DomainSeparators, Self::Error> {
        let info = self.query_cached_chain_info()
            .await?;

        Ok(DomainSeparators {
            ledger: Hash::from_hex(&info.ledger_dst
                .ok_or(ConnectorError::InvalidState("ledger DST not found"))?
            )?,
            safe_registry: Hash::from_hex(&info.safe_registry_dst.ok_or(ConnectorError::InvalidState("safe registry DST not found"))?)?,
            channel: Hash::from_hex(&info.channel_dst.ok_or(ConnectorError::InvalidState("channel DST not found"))?)?,
        })
    }

    async fn minimum_incoming_ticket_win_prob(&self) -> Result<WinningProbability, Self::Error> {
        Ok(WinningProbability::try_from_f64(self
            .query_cached_chain_info()
            .await?
            .min_ticket_winning_probability
        )?)
    }

    async fn minimum_ticket_price(&self) -> Result<HoprBalance, Self::Error> {
        Ok(self.query_cached_chain_info().await?.ticket_price.0.parse()?)
    }

    async fn channel_closure_notice_period(&self) -> Result<Duration, Self::Error> {
        Ok(Duration::from_millis(self.query_cached_chain_info().await?
            .channel_closure_grace_period
            .ok_or(ConnectorError::InvalidState("channel closure grace period not found"))?
            .0
            .parse()
            .map_err(|_| ConnectorError::TypeConversion("channel closure grace period not a number".into()))?
        ))
    }
}