use std::time::Duration;
use hopr_api::chain::DomainSeparators;
use hopr_internal_types::prelude::WinningProbability;
use hopr_primitive_types::balance::HoprBalance;
use crate::connector::HoprBlockchainConnector;
use crate::errors::ConnectorError;

#[async_trait::async_trait]
impl<B, C> hopr_api::chain::ChainValues for HoprBlockchainConnector<B, C>
where
    B: Send + Sync,
    C: Send + Sync + 'static 
{
    type Error = ConnectorError;

    async fn domain_separators(&self) -> Result<DomainSeparators, Self::Error> {
        todo!()
    }

    async fn minimum_incoming_ticket_win_prob(&self) -> Result<WinningProbability, Self::Error> {
        todo!()
    }

    async fn minimum_ticket_price(&self) -> Result<HoprBalance, Self::Error> {
        todo!()
    }

    async fn channel_closure_notice_period(&self) -> Result<Duration, Self::Error> {
        todo!()
    }
}