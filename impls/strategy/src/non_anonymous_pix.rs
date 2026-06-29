//! ## Non-Anonymous PIX Strategy
//!
//! This strategy is responsible for handling non-anonymous PIX transactions.
//!
//! It is responsible for:
//! - Handling new deposit addresses
//! - Handling deposit address recovery
//! - Handling PIX transfers
//!
//! All of these are done in a **non-anonymous** way, using plain on-chain transactions.
//!
//! **DO NOT USE THIS STRATEGY IN PRODUCTION**

use std::{
    fmt::{Debug, Display, Formatter},
    sync::Arc,
    time::Duration,
};

use futures::StreamExt;
use hopr_api::{
    chain::{ChainReadChannelOperations, ChainWriteAccountOperations},
    node::{
        ActionableEvent, ActionableEventDiscriminant, ActionableEventSource, DepositUpdated, HasChainApi, PixAddressId,
        PixEvent,
    },
    types::primitive::prelude::*,
};
use hopr_api::chain::ChainValues;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::{errors::StrategyError, strategy::Strategy as StrategyTrait};

#[derive(Clone, Debug, Default, Serialize, Deserialize, Validate)]
pub struct NonAnonymousPixStrategyConfig {
    pub price_per_byte: HoprBalance,
    pub max_ssa_allocation: HoprBalance,
    pub max_deposit_tracking_time: std::time::Duration,
}

/// Builder for [`NonAnonymousPixStrategy`].
///
/// Call [`new`](NonAnonymousPixStrategy::new) with the strategy configuration,
/// then [`build`](NonAnonymousPixStrategy::build) to wire in a node and obtain a
/// runnable `Box<dyn StrategyTrait + Send>`.
pub struct NonAnonymousPixStrategy {
    cfg: NonAnonymousPixStrategyConfig,
    interval: Duration,
}

impl NonAnonymousPixStrategy {
    /// Create a new builder with the given configuration.
    pub fn new(cfg: NonAnonymousPixStrategyConfig, interval: Duration) -> Self {
        Self { cfg, interval }
    }

    /// Wire in a node and return a running-ready strategy.
    pub fn build<N>(self, node: Arc<N>) -> Box<dyn StrategyTrait + Send>
    where
        N: HasChainApi + ActionableEventSource + Send + Sync + 'static,
    {
        Box::new(NonAnonymousPixStrategyInner {
            cfg: self.cfg,
            interval: self.interval,
            node,
        })
    }
}

/// Private generic runner — constructed by [`NonAnonymousPixStrategy::build`].
struct NonAnonymousPixStrategyInner<N: HasChainApi> {
    node: Arc<N>,
    cfg: NonAnonymousPixStrategyConfig,
    interval: Duration,
}

impl<N> NonAnonymousPixStrategyInner<N>
where
    N: HasChainApi + ActionableEventSource + Send + Sync + 'static,
{
    /// Periodic task logic.
    async fn on_tick(&self) -> crate::errors::Result<()> {
        tracing::debug!("PixStrategy tick");
        Ok(())
    }

    /// Handle PIX event.
    async fn on_pix_event(&self, event: PixEvent) -> crate::errors::Result<()> {
        tracing::debug!(?event, "PixStrategy event");
        match event {
            PixEvent::NewDepositAddress(new_deposit_address) => {
                tracing::info!(?new_deposit_address, "new deposit address");

                let target_deposit = self.cfg.price_per_byte * new_deposit_address.quota;
                if target_deposit > self.cfg.max_ssa_allocation {
                    tracing::warn!(%target_deposit, max_deposit = self.cfg.max_ssa_allocation, "target deposit too high");
                    return Err(StrategyError::CriteriaNotSatisfied);
                }

                // TODO: do not allow parallel withdrawals to any address
                if let Err(error) = self.node.chain_api().withdraw(target_deposit, &new_deposit_address.address.into()).await?.await {
                    tracing::error!(%error, %target_deposit, ?new_deposit_address, "withdraw failed");
                    return Err(StrategyError::Other(error));
                }
                tracing::info!(%target_deposit, ?new_deposit_address, "deposit successful");
            }
            PixEvent::DepositAddressReceived(deposit_address_recv) => {
                tracing::info!(?deposit_address_recv, "deposit address received");

                let target_deposit = self.cfg.price_per_byte * deposit_address_recv.quota;


                let node_clone = self.node.clone();
                let deposit_addr: Address = deposit_address_recv.address.into();
                futures_time::stream::interval(futures_time::time::Duration::from_secs(self.cfg.max_deposit_tracking_time/10))
                    .then(|_| async move {
                        node_clone.chain_api().balance(deposit_addr).await
                    });
                    //.try_ta

                hopr_utils::runtime::prelude::spawn(async move {
                    self.node.chain_api().balance(&deposit_address_recv.address.into()).await?;
                    tokio::time::sleep(self.cfg.max_deposit_tracking_time).await;
                    self.node.chain_api().withdraw(target_deposit, &new_deposit_address.address.into()).await?;
                })

            }
            PixEvent::PrivateKeyRecovered(private_key_recovered) => {
                tracing::info!(?private_key_recovered, "private key recovered");


            }
        }

        Ok(())
    }
}

impl<N: HasChainApi> Debug for NonAnonymousPixStrategyInner<N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "NonAnonymousPixStrategy({:?})", self.cfg)
    }
}

impl<N: HasChainApi> Display for NonAnonymousPixStrategyInner<N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "non_anonymous_pix")
    }
}

#[async_trait::async_trait]
impl<N: HasChainApi> StrategyTrait for NonAnonymousPixStrategyInner<N>
where
    N: HasChainApi + ActionableEventSource + Send + Sync + 'static,
{
    async fn run(&mut self) -> crate::errors::Result<()> {
        enum Event {
            Tick,
            Pix(PixEvent),
        }

        // Run the first scan immediately at startup without waiting for the initial interval.
        if let Err(error) = self.on_tick().await
            && !matches!(error, StrategyError::CriteriaNotSatisfied)
        {
            tracing::error!(%error, "pix tick failed");
        }

        let tick_stream = futures_time::stream::interval(self.interval.into()).map(|_| Event::Tick);
        let event_stream = self
            .node
            .subscribe_to_actionable_events(Some(&[ActionableEventDiscriminant::Pix]))
            .map_err(|e| StrategyError::Other(anyhow::anyhow!(e)))?
            .filter_map(|event| futures::future::ready(event.try_as_pix().map(Event::Pix)));

        let mut combined = futures_concurrency::stream::Merge::merge((tick_stream, event_stream));
        let me = *self.node.chain_api().me();

        while let Some(event) = combined.next().await {
            match event {
                Event::Tick => {
                    if let Err(error) = self.on_tick().await
                        && !matches!(error, StrategyError::CriteriaNotSatisfied)
                    {
                        tracing::error!(%error, "pix tick failed");
                    }
                }
                Event::Pix(event) => {
                    if let Err(error) = self.on_pix_event(event).await {
                        tracing::error!(%error, "pix event failed");
                    }
                }
            }
        }

        Ok(())
    }
}
