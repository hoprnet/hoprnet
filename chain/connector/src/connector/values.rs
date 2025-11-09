use std::{str::FromStr, time::Duration};

use blokli_client::api::{AccountSelector, BlokliQueryClient};
use futures::TryFutureExt;
use hopr_api::chain::{ChainInfo, DomainSeparators};
use hopr_crypto_types::{prelude::Keypair, types::Hash};
use hopr_internal_types::prelude::WinningProbability;
use hopr_primitive_types::prelude::*;

use crate::{connector::HoprBlockchainConnector, errors::ConnectorError};

impl<B, C, P> HoprBlockchainConnector<C, B, P>
where
    B: Send + Sync,
    C: BlokliQueryClient + Send + Sync + 'static,
    P: Send + Sync,
{
    async fn query_cached_chain_info(&self) -> Result<blokli_client::api::types::ChainInfo, ConnectorError> {
        Ok(self
            .values
            .try_get_with(0, self.client.query_chain_info().map_err(ConnectorError::from))
            .await?)
    }

    pub(crate) async fn query_next_nonce(&self) -> Result<u64, ConnectorError> {
        let accounts = self
            .client
            .query_accounts(AccountSelector::Address(self.chain_key.public().to_address().into()))
            .await?;

        if accounts.len() > 1 {
            return Err(ConnectorError::InvalidState("more than one account found".into()));
        }

        accounts
            .first()
            .cloned()
            .and_then(|a| a.safe_transaction_count)
            .ok_or(ConnectorError::InvalidState("no safe transaction count found".into()))
            .and_then(|v| {
                u64::from_str(&v.0)
                    .map(|v| v + 1)
                    .map_err(|_| ConnectorError::TypeConversion("invalid safe transaction count".into()))
            })
    }
}

#[async_trait::async_trait]
impl<B, C, P> hopr_api::chain::ChainValues for HoprBlockchainConnector<C, B, P>
where
    B: Send + Sync,
    C: BlokliQueryClient + Send + Sync + 'static,
    P: Send + Sync,
{
    type Error = ConnectorError;

    async fn domain_separators(&self) -> Result<DomainSeparators, Self::Error> {
        self.check_connection_state()?;

        let info = self.query_cached_chain_info().await?;

        Ok(DomainSeparators {
            ledger: Hash::from_hex(
                &info
                    .ledger_dst
                    .ok_or(ConnectorError::InvalidState("ledger DST not found"))
                    .inspect_err(|_| self.values.invalidate_all())?,
            )?,
            safe_registry: Hash::from_hex(
                &info
                    .safe_registry_dst
                    .ok_or(ConnectorError::InvalidState("safe registry DST not found"))
                    .inspect_err(|_| self.values.invalidate_all())?,
            )?,
            channel: Hash::from_hex(
                &info
                    .channel_dst
                    .ok_or(ConnectorError::InvalidState("channel DST not found"))
                    .inspect_err(|_| self.values.invalidate_all())?,
            )?,
        })
    }

    async fn minimum_incoming_ticket_win_prob(&self) -> Result<WinningProbability, Self::Error> {
        self.check_connection_state()?;

        Ok(WinningProbability::try_from_f64(
            self.query_cached_chain_info().await?.min_ticket_winning_probability,
        )?)
    }

    async fn minimum_ticket_price(&self) -> Result<HoprBalance, Self::Error> {
        self.check_connection_state()?;

        Ok(self
            .query_cached_chain_info()
            .await?
            .ticket_price
            .0
            .parse()
            .inspect_err(|_| self.values.invalidate_all())?)
    }

    async fn channel_closure_notice_period(&self) -> Result<Duration, Self::Error> {
        self.check_connection_state()?;

        Ok(Duration::from_millis(
            self.query_cached_chain_info()
                .await?
                .channel_closure_grace_period
                .ok_or(ConnectorError::InvalidState("channel closure grace period not found"))?
                .0
                .parse()
                .map_err(|_| ConnectorError::TypeConversion("channel closure grace period not a number".into()))
                .inspect_err(|_| self.values.invalidate_all())?,
        ))
    }

    async fn chain_info(&self) -> Result<ChainInfo, Self::Error> {
        self.check_connection_state()?;

        let info = self.query_cached_chain_info().await?;

        Ok(ChainInfo {
            chain_id: info.chain_id as u64,
            hopr_network_name: "<not set>".into(),
            contract_addresses: serde_json::from_str(&info.contract_addresses.0)
                .map_err(|e| ConnectorError::TypeConversion(format!("contract addresses not a valid JSON: {e}")))
                .inspect_err(|_| self.values.invalidate_all())?,
        })
    }
}
