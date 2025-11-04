use std::time::Duration;

use blokli_client::api::BlokliQueryClient;
use futures::TryFutureExt;
use hopr_api::chain::{ChainInfo, DomainSeparators};
use hopr_chain_types::ContractAddresses;
use hopr_crypto_types::types::Hash;
use hopr_internal_types::prelude::WinningProbability;
use hopr_primitive_types::prelude::*;

use crate::{connector::HoprBlockchainConnector, errors::ConnectorError};

impl<B, C, P> HoprBlockchainConnector<B, C, P>
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
        self.check_connection_state()?;

        let info = self.query_cached_chain_info().await?;

        Ok(DomainSeparators {
            ledger: Hash::from_hex(
                &info
                    .ledger_dst
                    .ok_or(ConnectorError::InvalidState("ledger DST not found"))?,
            )?,
            safe_registry: Hash::from_hex(
                &info
                    .safe_registry_dst
                    .ok_or(ConnectorError::InvalidState("safe registry DST not found"))?,
            )?,
            channel: Hash::from_hex(
                &info
                    .channel_dst
                    .ok_or(ConnectorError::InvalidState("channel DST not found"))?,
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

        Ok(self.query_cached_chain_info().await?.ticket_price.0.parse()?)
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
                .map_err(|_| ConnectorError::TypeConversion("channel closure grace period not a number".into()))?,
        ))
    }

    async fn chain_info(&self) -> Result<ChainInfo, Self::Error> {
        self.check_connection_state()?;

        // TODO: update this once Blokli provides this information
        let info = self.query_cached_chain_info().await?;

        Ok(ChainInfo {
            chain_id: info.chain_id as u64,
            hopr_network_name: "dufour".into(),
            contract_addresses: ContractAddresses {
                announcements: "0x619eabE23FD0E2291B50a507719aa633fE6069b8".parse()?,
                channels: "0x693Bac5ce61c720dDC68533991Ceb41199D8F8ae".parse()?,
                network_registry: "0x582b4b586168621dAf83bEb2AeADb5fb20F8d50d".parse()?,
                network_registry_proxy: "0x2bc6b78B0aA892e97714F0e3b1c74487f92C5884".parse()?,
                token: "0xD4fdec44DB9D44B8f2b6d529620f9C0C7066A2c1".parse()?,
                node_stake_v2_factory: "0x098B275485c406573D042848D66eb9d63fca311C".parse()?,
                node_safe_registry: "0xe15C24a0910311c83aC78B5930d771089E93077b".parse()?,
                module_implementation: "0xB7397C218766eBe6A1A634df523A1a7e412e67eA".parse()?,
                ticket_price_oracle: "0xcA5656Fe6F2d847ACA32cf5f38E51D2054cA1273".parse()?,
                winning_probability_oracle: "0x7Eb8d762fe794A108e568aD2097562cc5D3A1359".parse()?
            }
        })
    }
}
