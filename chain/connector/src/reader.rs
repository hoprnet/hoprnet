use std::time::Duration;

use blokli_client::api::{BlokliQueryClient, BlokliSubscriptionClient};
use futures::{StreamExt, TryStreamExt};
use futures_time::future::FutureExt as FuturesTimeExt;
use hopr_api::chain::{ChainInfo, DeployedSafe, DomainSeparators, SafeSelector};
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::*;

use crate::{
    errors::ConnectorError,
    utils::{model_to_chain_info, model_to_deployed_safe},
};

/// A simplified version of [`HoprBlockchainConnector`](crate::HoprBlockchainConnector)
/// which only implements [HOPR Chain API](hopr_api::chain) partially, allowing for read-only operations.
///
/// This object specifically implements only the following traits:
///
/// - [`ChainValues`](hopr_api::chain::ChainValues)
/// - [`ChainReadSafeOperations`](hopr_api::chain::ChainReadSafeOperations)
///
/// The implementation is currently realized using the Blokli client and acts as a partial HOPR Chain API compatible
/// wrapper for [`blokli_client::BlokliClient`].
///
/// This object is useful for bootstrapping purposes that usually precede construction of the [full
/// connector](crate::HoprBlockchainConnector).
pub struct HoprBlockchainReader<C>(pub(crate) std::sync::Arc<C>);

impl<C> HoprBlockchainReader<C> {
    /// Creates new instance given the `client`.
    pub fn new(client: C) -> Self {
        Self(std::sync::Arc::new(client))
    }
}

impl<C> Clone for HoprBlockchainReader<C> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

#[async_trait::async_trait]
impl<C> hopr_api::chain::ChainValues for HoprBlockchainReader<C>
where
    C: BlokliQueryClient + Send + Sync,
{
    type Error = ConnectorError;

    async fn balance<Cy: Currency, A: Into<Address> + Send>(&self, address: A) -> Result<Balance<Cy>, Self::Error> {
        let address = address.into();
        if Cy::is::<WxHOPR>() {
            Ok(self.0.query_token_balance(&address.into()).await?.balance.0.parse()?)
        } else if Cy::is::<XDai>() {
            Ok(self.0.query_native_balance(&address.into()).await?.balance.0.parse()?)
        } else {
            Err(ConnectorError::InvalidState("unsupported currency"))
        }
    }

    async fn domain_separators(&self) -> Result<DomainSeparators, Self::Error> {
        let chain_info = self.0.query_chain_info().await?;
        Ok(model_to_chain_info(chain_info)?.domain_separators)
    }

    async fn minimum_incoming_ticket_win_prob(&self) -> Result<WinningProbability, Self::Error> {
        let chain_info = self.0.query_chain_info().await?;
        Ok(model_to_chain_info(chain_info)?.ticket_win_prob)
    }

    async fn minimum_ticket_price(&self) -> Result<HoprBalance, Self::Error> {
        let chain_info = self.0.query_chain_info().await?;
        Ok(model_to_chain_info(chain_info)?.ticket_price)
    }

    async fn key_binding_fee(&self) -> Result<HoprBalance, Self::Error> {
        let chain_info = self.0.query_chain_info().await?;
        Ok(model_to_chain_info(chain_info)?.key_binding_fee)
    }

    async fn channel_closure_notice_period(&self) -> Result<Duration, Self::Error> {
        let chain_info = self.0.query_chain_info().await?;
        Ok(model_to_chain_info(chain_info)?.channel_closure_grace_period)
    }

    async fn chain_info(&self) -> Result<ChainInfo, Self::Error> {
        let chain_info = self.0.query_chain_info().await?;
        Ok(model_to_chain_info(chain_info)?.info)
    }
}

#[async_trait::async_trait]
impl<C> hopr_api::chain::ChainReadSafeOperations for HoprBlockchainReader<C>
where
    C: BlokliQueryClient + BlokliSubscriptionClient + Send + Sync,
{
    type Error = ConnectorError;

    async fn safe_allowance<Cy: Currency, A: Into<Address> + Send>(
        &self,
        safe_address: A,
    ) -> Result<Balance<Cy>, Self::Error> {
        let address = safe_address.into();
        if Cy::is::<WxHOPR>() {
            Ok(self
                .0
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

    async fn safe_info(&self, selector: SafeSelector) -> Result<Option<DeployedSafe>, Self::Error> {
        let selector = match selector {
            SafeSelector::Owner(owner_address) => blokli_client::api::SafeSelector::ChainKey(owner_address.into()),
            SafeSelector::Address(safe_address) => blokli_client::api::SafeSelector::SafeAddress(safe_address.into()),
        };

        if let Some(safe) = self.0.query_safe(selector).await? {
            Ok(Some(model_to_deployed_safe(safe)?))
        } else {
            Ok(None)
        }
    }

    async fn await_safe_deployment(
        &self,
        selector: SafeSelector,
        timeout: Duration,
    ) -> Result<DeployedSafe, Self::Error> {
        if let Some(safe) = self.safe_info(selector).await? {
            return Ok(safe);
        }

        let res = self
            .0
            .subscribe_safe_deployments()?
            .map_err(ConnectorError::from)
            .and_then(|safe| futures::future::ready(model_to_deployed_safe(safe)))
            .try_skip_while(|deployed_safe| futures::future::ok(!selector.satisfies(deployed_safe)))
            .take(1)
            .try_collect::<Vec<_>>()
            .timeout(futures_time::time::Duration::from(timeout))
            .await??;

        res.into_iter()
            .next()
            .ok_or(ConnectorError::InvalidState("safe deployment stream closed"))
    }

    async fn predict_module_address(
        &self,
        nonce: u64,
        owner: &Address,
        safe_address: &Address,
    ) -> Result<Address, Self::Error> {
        Ok(self
            .0
            .query_module_address_prediction(blokli_client::api::ModulePredictionInput {
                nonce,
                owner: (*owner).into(),
                safe_address: (*safe_address).into(),
            })
            .await?
            .into())
    }
}
