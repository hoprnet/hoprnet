//! ## Auto Funding Strategy
//! Watches for balance-decrease events and periodically scans outgoing channels
//! to re-stake any that have dropped below `min_stake_threshold` HOPR.
//!
//! ### In-flight tracking
//! Prevents duplicate funding when multiple balance-decrease events arrive in
//! quick succession. See [`AutoFundingStrategy`] for details.
//!
//! ### Metrics
//! - `hopr_strategy_auto_funding_funding_count` — incremented on successful enqueue
//! - `hopr_strategy_auto_funding_failure_count` — incremented on enqueue/confirm failure
use std::{
    fmt::{Debug, Display, Formatter},
    sync::Arc,
    time::Duration,
};

use async_trait::async_trait;
use dashmap::DashSet;
use futures::StreamExt;
use hopr_api::{
    chain::{
        ChainReadChannelOperations, ChainReadSafeOperations, ChainValues, ChainWriteChannelOperations, ChannelSelector,
        SafeSelector,
    },
    node::{ActionableEventDiscriminant, ActionableEventSource, HasChainApi},
    types::{
        internal::prelude::{ChannelDirection, ChannelEntry, ChannelId, ChannelStatus, ChannelStatusDiscriminants},
        primitive::prelude::HoprBalance,
    },
};
use serde::{Deserialize, Serialize};
use serde_with::{DisplayFromStr, serde_as};
use tracing::{debug, info, warn};
use validator::{Validate, ValidationError};

use crate::{errors::StrategyError, strategy::Strategy as StrategyTrait};

#[cfg(all(feature = "telemetry", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_COUNT_AUTO_FUNDINGS: hopr_api::types::telemetry::SimpleCounter =
        hopr_api::types::telemetry::SimpleCounter::new("hopr_strategy_auto_funding_funding_count", "Count of initiated automatic fundings").unwrap();
    static ref METRIC_COUNT_AUTO_FUNDING_FAILURES: hopr_api::types::telemetry::SimpleCounter =
        hopr_api::types::telemetry::SimpleCounter::new("hopr_strategy_auto_funding_failure_count", "Count of failed automatic funding attempts").unwrap();
}

fn validate_funding_amount(amount: &HoprBalance) -> std::result::Result<(), ValidationError> {
    if amount.is_zero() {
        return Err(ValidationError::new("funding_amount must be greater than zero"));
    }
    Ok(())
}

/// Configuration for `AutoFundingStrategy`.
#[serde_as]
#[derive(Debug, Clone, Copy, PartialEq, Eq, smart_default::SmartDefault, Validate, Serialize, Deserialize)]
pub struct AutoFundingStrategyConfig {
    /// Minimum stake that a channel's balance must not go below.
    ///
    /// Default is 1 wxHOPR.
    #[serde_as(as = "DisplayFromStr")]
    #[default(HoprBalance::new_base(1))]
    pub min_stake_threshold: HoprBalance,

    /// Funding amount. Must be greater than zero.
    ///
    /// Default is 10 wxHOPR.
    #[serde_as(as = "DisplayFromStr")]
    #[default(HoprBalance::new_base(10))]
    #[validate(custom(function = "validate_funding_amount"))]
    pub funding_amount: HoprBalance,
}

/// Builder for [`AutoFundingStrategy`].
///
/// Call [`new`](AutoFundingStrategy::new) with the strategy configuration,
/// then [`build`](AutoFundingStrategy::build) to wire in a node and obtain a
/// runnable `Box<dyn Strategy + Send>`.
pub struct AutoFundingStrategy {
    cfg: AutoFundingStrategyConfig,
    interval: Duration,
}

impl AutoFundingStrategy {
    /// Create a new builder with the given configuration.
    pub fn new(cfg: AutoFundingStrategyConfig, interval: Duration) -> Self {
        if cfg.funding_amount.le(&cfg.min_stake_threshold) {
            warn!(
                funding_amount = %cfg.funding_amount,
                min_stake_threshold = %cfg.min_stake_threshold,
                "funding_amount is not greater than min_stake_threshold; \
                 successful funding may re-trigger the threshold check"
            );
        }
        Self { cfg, interval }
    }

    /// Wire in a node and return a running-ready strategy.
    ///
    /// The generic `N` is erased at construction time; the returned
    /// `Box<dyn Strategy + Send>` can be held and spawned without knowledge
    /// of the concrete node type.
    pub fn build<N>(self, node: Arc<N>) -> Box<dyn StrategyTrait + Send>
    where
        N: HasChainApi + ActionableEventSource + Send + Sync + 'static,
        N::ChainApi: ChainReadChannelOperations
            + ChainReadSafeOperations
            + ChainValues
            + ChainWriteChannelOperations
            + Clone
            + Send
            + Sync
            + 'static,
    {
        Box::new(AutoFundingStrategyInner {
            cfg: self.cfg,
            interval: self.interval,
            node,
            in_flight: Arc::new(DashSet::new()),
        })
    }
}

/// Private generic runner — constructed by [`AutoFundingStrategy::build`].
struct AutoFundingStrategyInner<N: HasChainApi> {
    node: Arc<N>,
    cfg: AutoFundingStrategyConfig,
    interval: Duration,
    /// Channels with in-flight funding transactions.
    in_flight: Arc<DashSet<ChannelId>>,
}

impl<N> AutoFundingStrategyInner<N>
where
    N: HasChainApi + ActionableEventSource + Send + Sync + 'static,
    <N as HasChainApi>::ChainApi: ChainReadChannelOperations
        + ChainReadSafeOperations
        + ChainValues
        + ChainWriteChannelOperations
        + Clone
        + Send
        + Sync
        + 'static,
{
    /// Attempt to fund a channel if not already in-flight.
    ///
    /// Returns `Ok(true)` if a funding task was spawned, `Ok(false)` if skipped.
    /// This function is synchronous: the actual chain call is dispatched to a background task.
    fn try_fund_channel(&self, channel: &ChannelEntry) -> crate::errors::Result<bool> {
        let channel_id = *channel.get_id();

        if !self.in_flight.insert(channel_id) {
            debug!(%channel, "skipping channel with in-flight funding");
            return Ok(false);
        }

        info!(
            %channel,
            balance = %channel.balance,
            threshold = %self.cfg.min_stake_threshold,
            "stake on channel at or below threshold"
        );

        let chain = self.node.chain_api().clone();
        let funding_amount = self.cfg.funding_amount;
        let in_flight = Arc::clone(&self.in_flight);

        hopr_utils::runtime::prelude::spawn(async move {
            match chain.fund_channel(&channel_id, funding_amount).await {
                Ok(confirmation) => {
                    #[cfg(all(feature = "telemetry", not(test)))]
                    METRIC_COUNT_AUTO_FUNDINGS.increment();

                    info!(%channel_id, %funding_amount, "issued re-staking of channel");

                    if let Err(e) = confirmation.await {
                        warn!(%channel_id, error = %e, "funding transaction failed");
                        in_flight.remove(&channel_id);

                        #[cfg(all(feature = "telemetry", not(test)))]
                        METRIC_COUNT_AUTO_FUNDING_FAILURES.increment();
                    }
                    // On success: ChannelBalanceIncreased event clears the in-flight entry.
                }
                Err(e) => {
                    warn!(%channel_id, error = %e, "failed to enqueue funding transaction");
                    in_flight.remove(&channel_id);

                    #[cfg(all(feature = "telemetry", not(test)))]
                    METRIC_COUNT_AUTO_FUNDING_FAILURES.increment();
                }
            }
        });

        Ok(true)
    }

    /// Returns the available safe balance, or zero if the safe is not registered.
    async fn safe_balance_budget(&self) -> crate::errors::Result<HoprBalance> {
        let me = *self.node.chain_api().me();
        let chain = self.node.chain_api().clone();

        let safe = chain
            .safe_info(SafeSelector::NodeAddress(me))
            .await
            .map_err(|e| StrategyError::Other(e.into()))?;

        let Some(safe) = safe else {
            warn!(%me, "auto-funding on_tick skipped: safe is not registered. Should never happen.");
            return Ok(HoprBalance::zero());
        };

        chain
            .balance(safe.address)
            .await
            .map_err(|e| StrategyError::Other(e.into()))
    }

    /// Periodic scan: fund any outgoing open channels below the threshold.
    async fn on_tick(&self) -> crate::errors::Result<()> {
        let mut safe_balance_budget = self.safe_balance_budget().await?;
        if safe_balance_budget < self.cfg.funding_amount {
            debug!(
                %safe_balance_budget,
                funding_amount = %self.cfg.funding_amount,
                "auto-funding on_tick skipped: safe balance below funding amount"
            );
            return Ok(());
        }

        let me = *self.node.chain_api().me();
        let mut channels = self
            .node
            .chain_api()
            .stream_channels(
                ChannelSelector::default()
                    .with_source(me)
                    .with_allowed_states(&[ChannelStatusDiscriminants::Open]),
            )
            .map_err(|e| StrategyError::Other(e.into()))?;

        while let Some(channel) = channels.next().await {
            if channel.balance.le(&self.cfg.min_stake_threshold) {
                if safe_balance_budget < self.cfg.funding_amount {
                    break;
                }

                match self.try_fund_channel(&channel) {
                    Ok(true) => safe_balance_budget -= self.cfg.funding_amount,
                    Ok(false) => {}
                    Err(e) => warn!(%channel, error = %e, "on_tick: failed to fund channel"),
                }
            } else {
                // Channel above threshold; clear any stale in-flight entry.
                self.in_flight.remove(channel.get_id());
            }
        }

        debug!("auto-funding on_tick scan complete");
        Ok(())
    }
}

impl<N: HasChainApi> Debug for AutoFundingStrategyInner<N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "AutoFundingStrategy({:?})", self.cfg)
    }
}

impl<N: HasChainApi> Display for AutoFundingStrategyInner<N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "auto_funding")
    }
}

#[async_trait]
impl<N> StrategyTrait for AutoFundingStrategyInner<N>
where
    N: HasChainApi + ActionableEventSource + Send + Sync + 'static,
    <N as HasChainApi>::ChainApi: ChainReadChannelOperations
        + ChainReadSafeOperations
        + ChainValues
        + ChainWriteChannelOperations
        + Clone
        + Send
        + Sync
        + 'static,
{
    async fn run(&mut self) -> crate::errors::Result<()> {
        use hopr_api::{chain::ChainEvent, node::ActionableEvent};

        enum Event {
            Tick,
            Actionable(Box<ActionableEvent>),
        }

        // Run the first scan immediately at startup without waiting for the initial interval.
        if let Err(e) = self.on_tick().await {
            tracing::warn!(%e, "auto-funding tick failed");
        }

        let tick_stream = futures_time::stream::interval(self.interval.into()).map(|_| Event::Tick);
        let event_stream = self
            .node
            .subscribe_to_actionable_events(Some(&[ActionableEventDiscriminant::Chain]))
            .map_err(|e| StrategyError::Other(anyhow::anyhow!(e)))?
            .map(|e| Event::Actionable(Box::new(e)));

        let mut combined = futures_concurrency::stream::Merge::merge((tick_stream, event_stream));
        let me = *self.node.chain_api().me();

        while let Some(event) = combined.next().await {
            match event {
                Event::Tick => {
                    if let Err(e) = self.on_tick().await {
                        tracing::warn!(%e, "auto-funding tick failed");
                    }
                }
                Event::Actionable(ev) => match *ev {
                    ActionableEvent::Chain(ChainEvent::ChannelBalanceDecreased(ch, _))
                        if ch.direction(&me) == Some(ChannelDirection::Outgoing)
                            && ch.balance.le(&self.cfg.min_stake_threshold)
                            && ch.status == ChannelStatus::Open =>
                    {
                        // Guard against over-commitment: skip if the safe cannot
                        // cover even a single funding round.
                        match self.safe_balance_budget().await {
                            Ok(budget) if budget < self.cfg.funding_amount => {
                                debug!(%ch, %budget, "event-driven funding skipped: safe balance below funding amount");
                            }
                            Ok(_) => {
                                if let Err(e) = self.try_fund_channel(&ch) {
                                    warn!(%ch, %e, "failed to fund channel on balance decrease event");
                                }
                            }
                            Err(e) => {
                                warn!(%ch, %e, "event-driven funding skipped: could not fetch safe balance");
                            }
                        }
                    }
                    ActionableEvent::Chain(ChainEvent::ChannelBalanceIncreased(ch, _))
                        if self.in_flight.remove(ch.get_id()).is_some() =>
                    {
                        debug!(%ch, "cleared in-flight funding state after balance increase");
                    }
                    _ => {}
                },
            }
        }

        Ok(())
    }
}

/// Test-only helpers for driving `AutoFundingStrategyInner` from unit tests
/// without going through the event stream.
#[cfg(test)]
impl<N> AutoFundingStrategyInner<N>
where
    N: HasChainApi + ActionableEventSource + Send + Sync + 'static,
    <N as HasChainApi>::ChainApi: ChainReadChannelOperations
        + ChainReadSafeOperations
        + ChainValues
        + ChainWriteChannelOperations
        + Clone
        + Send
        + Sync
        + 'static,
{
    /// Handle a balance change event on an outgoing channel.
    async fn on_own_channel_changed_balance(
        &self,
        channel: &ChannelEntry,
        direction: ChannelDirection,
        old: HoprBalance,
        new: HoprBalance,
    ) -> crate::errors::Result<()> {
        if direction != ChannelDirection::Outgoing {
            return Ok(());
        }

        // If balance increased, clear in-flight state for this channel
        if new > old && self.in_flight.remove(channel.get_id()).is_some() {
            debug!(%channel, "cleared in-flight funding state after balance increase");
        }

        if new.le(&self.cfg.min_stake_threshold) && channel.status == ChannelStatus::Open {
            self.try_fund_channel(channel)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use anyhow::Context;
    use futures::StreamExt;
    use futures_time::future::FutureExt;
    use hex_literal::hex;
    use hopr_api::{
        chain::{
            AccountSelector, ChainEvent, ChainEvents, ChainReadAccountOperations, ChainWriteAccountOperations,
            HoprChainApi,
        },
        node::{
            ActionableEvent, ComponentStatus, ComponentStatusReporter, EventWaitResult, HasChainApi,
            NodeOnchainIdentity,
        },
        types::{
            crypto::{keypairs::Keypair, prelude::ChainKeypair},
            internal::prelude::{ChannelDirection, ChannelEntry, ChannelStatus},
            primitive::prelude::{Address, BytesRepresentable, HoprBalance, XDaiBalance},
        },
    };
    use hopr_chain_connector::{create_trustful_hopr_blokli_connector, testing::BlokliTestStateBuilder};

    use super::*;

    lazy_static::lazy_static! {
        static ref BOB_KP: ChainKeypair = ChainKeypair::from_secret(&hex!(
            "492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775"
        ))
        .expect("lazy static keypair should be valid");

        static ref ALICE: Address = hex!("18f8ae833c85c51fbeba29cef9fbfb53b3bad950").into();
        static ref BOB: Address = BOB_KP.public().to_address();
        static ref CHRIS: Address = hex!("b6021e0860dd9d96c9ff0a73e2e5ba3a466ba234").into();
        static ref DAVE: Address = hex!("68499f50ff68d523385dc60686069935d17d762a").into();
    }

    /// Wraps a chain API implementor as a minimal node for strategy tests.
    ///
    /// The connector itself is a chain API, not a node. This newtype implements the
    /// `HasChainApi` and `ActionableEventSource` node traits so integration tests can
    /// drive strategies without a full `Hopr` node.
    struct ChainNode<C>(C);

    impl<C> HasChainApi for ChainNode<C>
    where
        C: HoprChainApi + ComponentStatusReporter + Clone + Send + Sync + 'static,
    {
        type ChainApi = C;
        type ChainError = <C as HoprChainApi>::ChainError;

        fn identity(&self) -> &NodeOnchainIdentity {
            static IDENTITY: std::sync::OnceLock<NodeOnchainIdentity> = std::sync::OnceLock::new();
            IDENTITY.get_or_init(NodeOnchainIdentity::default)
        }

        fn chain_api(&self) -> &C {
            &self.0
        }

        fn status(&self) -> ComponentStatus {
            self.0.component_status()
        }

        fn wait_for_on_chain_event<F>(
            &self,
            _predicate: F,
            _context: String,
            _timeout: std::time::Duration,
        ) -> EventWaitResult<<C as HoprChainApi>::ChainError, <C as HoprChainApi>::ChainError>
        where
            F: Fn(&ChainEvent) -> bool + Send + Sync + 'static,
        {
            unimplemented!("tests do not call wait_for_on_chain_event")
        }
    }

    impl<C> ActionableEventSource for ChainNode<C>
    where
        C: ChainEvents + Send + Sync + 'static,
    {
        fn subscribe_to_actionable_events(
            &self,
            _filter: Option<&[ActionableEventDiscriminant]>,
        ) -> Result<futures::stream::BoxStream<'static, ActionableEvent>, String> {
            Ok(self
                .0
                .subscribe()
                .map_err(|e| e.to_string())?
                .map(ActionableEvent::Chain)
                .boxed())
        }
    }

    async fn register_test_safe<C>(chain_connector: &C, node_address: Address) -> anyhow::Result<()>
    where
        C: HoprChainApi + ChainReadAccountOperations + ChainWriteAccountOperations,
    {
        let account = chain_connector
            .stream_accounts(AccountSelector::default().with_chain_key(node_address))?
            .next()
            .await
            .context("missing test account for node")?;
        let safe_address = account.safe_address.context("missing test safe address for node")?;

        chain_connector.register_safe(&safe_address).await?.await?;

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn test_auto_funding_strategy() -> anyhow::Result<()> {
        let stake_limit = HoprBalance::from(7_u32);
        let fund_amount = HoprBalance::from(5_u32);

        let c1 = ChannelEntry::builder()
            .between(*ALICE, *BOB)
            .amount(10_u32)
            .ticket_index(0)
            .status(ChannelStatus::Open)
            .epoch(0)
            .build()?;

        let c2 = ChannelEntry::builder()
            .between(*BOB, *CHRIS)
            .amount(5_u32)
            .ticket_index(0_u32.into())
            .status(ChannelStatus::Open)
            .epoch(0)
            .build()?;

        let c3 = ChannelEntry::builder()
            .between(*CHRIS, *DAVE)
            .amount(5)
            .ticket_index(0)
            .status(ChannelStatus::PendingToClose(
                chrono::DateTime::<chrono::Utc>::from_str("2025-11-10T00:00:00+00:00")?.into(),
            ))
            .epoch(0)
            .build()?;

        let blokli_sim = BlokliTestStateBuilder::default()
            .with_generated_accounts(
                &[&*ALICE, &*BOB, &*CHRIS, &*DAVE],
                false,
                XDaiBalance::new_base(1),
                HoprBalance::new_base(1000),
            )
            .with_channels([c1, c2, c3])
            .build_dynamic_client([1; Address::SIZE].into());

        let snapshot = blokli_sim.snapshot();

        let mut chain_connector =
            create_trustful_hopr_blokli_connector(&BOB_KP, Default::default(), blokli_sim, [1; Address::SIZE].into())
                .await?;
        chain_connector.connect().await?;
        let chain_connector = Arc::new(chain_connector);
        let events = chain_connector.subscribe()?;

        let cfg = AutoFundingStrategyConfig {
            min_stake_threshold: stake_limit,
            funding_amount: fund_amount,
        };

        let afs = AutoFundingStrategyInner {
            cfg,
            interval: std::time::Duration::from_secs(60),
            node: Arc::new(ChainNode(Arc::clone(&chain_connector))),
            in_flight: Arc::new(DashSet::new()),
        };

        afs.on_own_channel_changed_balance(&c1, ChannelDirection::Outgoing, HoprBalance::zero(), c1.balance)
            .await?;

        afs.on_own_channel_changed_balance(&c2, ChannelDirection::Outgoing, HoprBalance::zero(), c2.balance)
            .await?;

        afs.on_own_channel_changed_balance(&c3, ChannelDirection::Outgoing, HoprBalance::zero(), c3.balance)
            .await?;

        events
            .filter(|event| futures::future::ready(matches!(event, ChainEvent::ChannelBalanceIncreased(c, amount) if c.get_id() == c2.get_id() && amount == &fund_amount)))
            .next()
            .timeout(futures_time::time::Duration::from_secs(2))
            .await?;

        insta::assert_yaml_snapshot!(*snapshot.refresh());

        Ok(())
    }

    #[test]
    fn test_config_validation_rejects_zero_funding_amount() {
        let cfg = AutoFundingStrategyConfig {
            min_stake_threshold: HoprBalance::new_base(1),
            funding_amount: HoprBalance::zero(),
        };
        assert!(
            cfg.validate().is_err(),
            "config with zero funding_amount should fail validation"
        );
    }

    #[test]
    fn test_config_validation_accepts_valid_config() {
        let cfg = AutoFundingStrategyConfig {
            min_stake_threshold: HoprBalance::new_base(1),
            funding_amount: HoprBalance::new_base(10),
        };
        assert!(
            cfg.validate().is_ok(),
            "config with valid funding_amount should pass validation"
        );
    }

    #[test]
    fn test_default_config_passes_validation() {
        let cfg = AutoFundingStrategyConfig::default();
        assert!(cfg.validate().is_ok(), "default config should pass validation");
    }

    #[test_log::test(tokio::test)]
    async fn test_on_tick_funds_underfunded_channels() -> anyhow::Result<()> {
        let stake_limit = HoprBalance::from(7_u32);
        let fund_amount = HoprBalance::from(5_u32);

        let c1 = ChannelEntry::builder()
            .between(*BOB, *CHRIS)
            .amount(3)
            .ticket_index(0)
            .status(ChannelStatus::Open)
            .epoch(0_u32)
            .build()?;

        let c2 = ChannelEntry::builder()
            .between(*BOB, *DAVE)
            .amount(10)
            .ticket_index(0)
            .status(ChannelStatus::Open)
            .epoch(0_u32)
            .build()?;

        let blokli_sim = BlokliTestStateBuilder::default()
            .with_generated_accounts(
                &[&*ALICE, &*BOB, &*CHRIS, &*DAVE],
                false,
                XDaiBalance::new_base(1),
                HoprBalance::new_base(1000),
            )
            .with_channels([c1, c2])
            .build_dynamic_client([1; Address::SIZE].into());

        let mut chain_connector =
            create_trustful_hopr_blokli_connector(&BOB_KP, Default::default(), blokli_sim, [1; Address::SIZE].into())
                .await?;
        chain_connector.connect().await?;
        register_test_safe(&chain_connector, *BOB).await?;
        let chain_connector = Arc::new(chain_connector);
        let events = chain_connector.subscribe()?;

        let cfg = AutoFundingStrategyConfig {
            min_stake_threshold: stake_limit,
            funding_amount: fund_amount,
        };

        let afs = AutoFundingStrategyInner {
            cfg,
            interval: std::time::Duration::from_secs(60),
            node: Arc::new(ChainNode(Arc::clone(&chain_connector))),
            in_flight: Arc::new(DashSet::new()),
        };

        afs.on_tick().await?;

        events
            .filter(|event| {
                futures::future::ready(
                    matches!(event, ChainEvent::ChannelBalanceIncreased(c, amount) if c.get_id() == c1.get_id() && amount == &fund_amount),
                )
            })
            .next()
            .timeout(futures_time::time::Duration::from_secs(2))
            .await?;

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn test_on_tick_skips_when_safe_balance_below_funding_amount() -> anyhow::Result<()> {
        let stake_limit = HoprBalance::from(7_u32);
        let fund_amount = HoprBalance::from(5_u32);

        let c1 = ChannelEntry::builder()
            .between(*BOB, *CHRIS)
            .amount(3)
            .ticket_index(0)
            .status(ChannelStatus::Open)
            .epoch(0)
            .build()?;

        let blokli_sim = BlokliTestStateBuilder::default()
            .with_generated_accounts(
                &[&*ALICE, &*BOB, &*CHRIS, &*DAVE],
                false,
                XDaiBalance::new_base(1),
                HoprBalance::from(1_u32),
            )
            .with_channels([c1])
            .build_dynamic_client([1; Address::SIZE].into());

        let mut chain_connector =
            create_trustful_hopr_blokli_connector(&BOB_KP, Default::default(), blokli_sim, [1; Address::SIZE].into())
                .await?;
        chain_connector.connect().await?;
        register_test_safe(&chain_connector, *BOB).await?;
        let chain_connector = Arc::new(chain_connector);
        let events = chain_connector.subscribe()?;

        let cfg = AutoFundingStrategyConfig {
            min_stake_threshold: stake_limit,
            funding_amount: fund_amount,
        };

        let afs = AutoFundingStrategyInner {
            cfg,
            interval: std::time::Duration::from_secs(60),
            node: Arc::new(ChainNode(Arc::clone(&chain_connector))),
            in_flight: Arc::new(DashSet::new()),
        };

        afs.on_tick().await?;

        let no_funding_event = events
            .filter(|event| {
                futures::future::ready(
                    matches!(event, ChainEvent::ChannelBalanceIncreased(c, amount) if c.get_id() == c1.get_id() && amount == &fund_amount),
                )
            })
            .next()
            .timeout(futures_time::time::Duration::from_secs(1))
            .await;

        assert!(
            no_funding_event.is_err(),
            "on_tick should skip funding when safe balance is below funding_amount"
        );

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn test_on_tick_funds_only_channels_affordable_by_safe_balance() -> anyhow::Result<()> {
        let stake_limit = HoprBalance::from(7_u32);
        let fund_amount = HoprBalance::from(5_u32);

        let c1 = ChannelEntry::builder()
            .between(*BOB, *CHRIS)
            .amount(3)
            .ticket_index(0)
            .status(ChannelStatus::Open)
            .epoch(0_u32)
            .build()?;
        let c2 = ChannelEntry::builder()
            .between(*BOB, *DAVE)
            .amount(2)
            .ticket_index(0)
            .status(ChannelStatus::Open)
            .epoch(0_u32)
            .build()?;
        let c3 = ChannelEntry::builder()
            .between(*BOB, *ALICE)
            .amount(1)
            .ticket_index(0)
            .status(ChannelStatus::Open)
            .epoch(0_u32)
            .build()?;
        let tracked_channels = [*c1.get_id(), *c2.get_id(), *c3.get_id()];

        let blokli_sim = BlokliTestStateBuilder::default()
            .with_generated_accounts(
                &[&*ALICE, &*BOB, &*CHRIS, &*DAVE],
                false,
                XDaiBalance::new_base(1),
                HoprBalance::from(11_u32),
            )
            .with_channels([c1, c2, c3])
            .build_dynamic_client([1; Address::SIZE].into());

        let mut chain_connector =
            create_trustful_hopr_blokli_connector(&BOB_KP, Default::default(), blokli_sim, [1; Address::SIZE].into())
                .await?;
        chain_connector.connect().await?;
        register_test_safe(&chain_connector, *BOB).await?;
        let chain_connector = Arc::new(chain_connector);
        let events = chain_connector.subscribe()?;

        let cfg = AutoFundingStrategyConfig {
            min_stake_threshold: stake_limit,
            funding_amount: fund_amount,
        };

        let afs = AutoFundingStrategyInner {
            cfg,
            interval: std::time::Duration::from_secs(60),
            node: Arc::new(ChainNode(Arc::clone(&chain_connector))),
            in_flight: Arc::new(DashSet::new()),
        };

        afs.on_tick().await?;

        let mut funding_events = events.filter(|event| {
            futures::future::ready(matches!(
                event,
                ChainEvent::ChannelBalanceIncreased(c, amount)
                    if tracked_channels.contains(c.get_id()) && amount == &fund_amount
            ))
        });

        let first_two = funding_events
            .by_ref()
            .take(2)
            .collect::<Vec<_>>()
            .timeout(futures_time::time::Duration::from_secs(2))
            .await?;
        assert_eq!(first_two.len(), 2, "on_tick should fund exactly two channels");

        let third = funding_events
            .next()
            .timeout(futures_time::time::Duration::from_secs(1))
            .await;
        assert!(third.is_err(), "on_tick should not fund a third channel");

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn test_in_flight_prevents_duplicate_funding() -> anyhow::Result<()> {
        let stake_limit = HoprBalance::from(7_u32);
        let fund_amount = HoprBalance::from(5_u32);

        let c1 = ChannelEntry::builder()
            .between(*BOB, *CHRIS)
            .amount(3)
            .ticket_index(0)
            .status(ChannelStatus::Open)
            .epoch(0_u32)
            .build()?;

        let blokli_sim = BlokliTestStateBuilder::default()
            .with_generated_accounts(
                &[&*ALICE, &*BOB, &*CHRIS, &*DAVE],
                false,
                XDaiBalance::new_base(1),
                HoprBalance::new_base(1000),
            )
            .with_channels([c1])
            .build_dynamic_client([1; Address::SIZE].into());

        let mut chain_connector =
            create_trustful_hopr_blokli_connector(&BOB_KP, Default::default(), blokli_sim, [1; Address::SIZE].into())
                .await?;
        chain_connector.connect().await?;
        register_test_safe(&chain_connector, *BOB).await?;
        let chain_connector = Arc::new(chain_connector);
        let _events = chain_connector.subscribe()?;

        let cfg = AutoFundingStrategyConfig {
            min_stake_threshold: stake_limit,
            funding_amount: fund_amount,
        };

        let afs = AutoFundingStrategyInner {
            cfg,
            interval: std::time::Duration::from_secs(60),
            node: Arc::new(ChainNode(Arc::clone(&chain_connector))),
            in_flight: Arc::new(DashSet::new()),
        };

        afs.on_own_channel_changed_balance(&c1, ChannelDirection::Outgoing, HoprBalance::from(10_u32), c1.balance)
            .await?;

        assert!(
            afs.in_flight.contains(c1.get_id()),
            "channel should be in-flight after funding"
        );

        afs.on_own_channel_changed_balance(&c1, ChannelDirection::Outgoing, HoprBalance::from(10_u32), c1.balance)
            .await?;

        assert_eq!(
            afs.in_flight.len(),
            1,
            "in-flight set should still have exactly one entry"
        );
        assert!(afs.in_flight.contains(c1.get_id()), "channel should still be in-flight");

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn test_balance_increase_clears_in_flight() -> anyhow::Result<()> {
        let stake_limit = HoprBalance::from(7_u32);
        let fund_amount = HoprBalance::from(5_u32);

        let c1 = ChannelEntry::builder()
            .between(*BOB, *CHRIS)
            .amount(3)
            .ticket_index(0)
            .status(ChannelStatus::Open)
            .epoch(0_u32)
            .build()?;

        let blokli_sim = BlokliTestStateBuilder::default()
            .with_generated_accounts(
                &[&*ALICE, &*BOB, &*CHRIS, &*DAVE],
                false,
                XDaiBalance::new_base(1),
                HoprBalance::new_base(1000),
            )
            .with_channels([c1])
            .build_dynamic_client([1; Address::SIZE].into());

        let mut chain_connector =
            create_trustful_hopr_blokli_connector(&BOB_KP, Default::default(), blokli_sim, [1; Address::SIZE].into())
                .await?;
        chain_connector.connect().await?;
        register_test_safe(&chain_connector, *BOB).await?;
        let chain_connector = Arc::new(chain_connector);
        let _events = chain_connector.subscribe()?;

        let cfg = AutoFundingStrategyConfig {
            min_stake_threshold: stake_limit,
            funding_amount: fund_amount,
        };

        let afs = AutoFundingStrategyInner {
            cfg,
            interval: std::time::Duration::from_secs(60),
            node: Arc::new(ChainNode(Arc::clone(&chain_connector))),
            in_flight: Arc::new(DashSet::new()),
        };

        afs.on_own_channel_changed_balance(&c1, ChannelDirection::Outgoing, HoprBalance::from(10_u32), c1.balance)
            .await?;

        assert!(afs.in_flight.contains(c1.get_id()));

        afs.on_own_channel_changed_balance(
            &c1,
            ChannelDirection::Outgoing,
            HoprBalance::from(3),
            HoprBalance::from(8),
        )
        .await?;

        assert!(
            !afs.in_flight.contains(c1.get_id()),
            "channel should be cleared from in-flight after balance increase"
        );

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn test_on_tick_skips_in_flight_channels() -> anyhow::Result<()> {
        let stake_limit = HoprBalance::from(7_u32);
        let fund_amount = HoprBalance::from(5_u32);

        let c1 = ChannelEntry::builder()
            .between(*BOB, *CHRIS)
            .amount(3)
            .ticket_index(0)
            .status(ChannelStatus::Open)
            .epoch(0)
            .build()?;

        let blokli_sim = BlokliTestStateBuilder::default()
            .with_generated_accounts(
                &[&*ALICE, &*BOB, &*CHRIS, &*DAVE],
                false,
                XDaiBalance::new_base(1),
                HoprBalance::new_base(1000),
            )
            .with_channels([c1])
            .build_dynamic_client([1; Address::SIZE].into());

        let mut chain_connector =
            create_trustful_hopr_blokli_connector(&BOB_KP, Default::default(), blokli_sim, [1; Address::SIZE].into())
                .await?;
        chain_connector.connect().await?;
        register_test_safe(&chain_connector, *BOB).await?;
        let chain_connector = Arc::new(chain_connector);
        let _events = chain_connector.subscribe()?;

        let cfg = AutoFundingStrategyConfig {
            min_stake_threshold: stake_limit,
            funding_amount: fund_amount,
        };

        let afs = AutoFundingStrategyInner {
            cfg,
            interval: std::time::Duration::from_secs(60),
            node: Arc::new(ChainNode(Arc::clone(&chain_connector))),
            in_flight: Arc::new(DashSet::new()),
        };

        afs.on_own_channel_changed_balance(&c1, ChannelDirection::Outgoing, HoprBalance::from(10_u32), c1.balance)
            .await?;

        assert!(afs.in_flight.contains(c1.get_id()), "channel should be in-flight");

        afs.on_tick().await?;

        assert_eq!(afs.in_flight.len(), 1, "in-flight set should still have one entry");
        assert!(afs.in_flight.contains(c1.get_id()), "channel should still be in-flight");

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn test_on_own_channel_changed_balance_skips_incoming_direction() -> anyhow::Result<()> {
        let stake_limit = HoprBalance::from(7_u32);
        let fund_amount = HoprBalance::from(5_u32);

        let c1 = ChannelEntry::builder()
            .between(*CHRIS, *BOB) // CHRIS → BOB: from BOB's perspective this is Incoming
            .amount(3)
            .ticket_index(0)
            .status(ChannelStatus::Open)
            .epoch(0_u32)
            .build()?;

        let blokli_sim = BlokliTestStateBuilder::default()
            .with_generated_accounts(
                &[&*ALICE, &*BOB, &*CHRIS, &*DAVE],
                false,
                XDaiBalance::new_base(1),
                HoprBalance::new_base(1000),
            )
            .with_channels([c1])
            .build_dynamic_client([1; Address::SIZE].into());

        let mut chain_connector =
            create_trustful_hopr_blokli_connector(&BOB_KP, Default::default(), blokli_sim, [1; Address::SIZE].into())
                .await?;
        chain_connector.connect().await?;

        let afs = AutoFundingStrategyInner {
            cfg: AutoFundingStrategyConfig {
                min_stake_threshold: stake_limit,
                funding_amount: fund_amount,
            },
            interval: std::time::Duration::from_secs(60),
            node: Arc::new(ChainNode(Arc::new(chain_connector))),
            in_flight: Arc::new(DashSet::new()),
        };

        // Incoming direction: should be a no-op regardless of balance.
        afs.on_own_channel_changed_balance(&c1, ChannelDirection::Incoming, HoprBalance::zero(), c1.balance)
            .await?;
        assert!(afs.in_flight.is_empty(), "incoming channel should never be funded");
        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn test_on_own_channel_changed_balance_skips_when_above_threshold() -> anyhow::Result<()> {
        let stake_limit = HoprBalance::from(7_u32);
        let fund_amount = HoprBalance::from(5_u32);

        let c1 = ChannelEntry::builder()
            .between(*BOB, *CHRIS)
            .amount(10) // balance 10 > stake_limit 7, so no funding
            .ticket_index(0)
            .status(ChannelStatus::Open)
            .epoch(0_u32)
            .build()?;

        let blokli_sim = BlokliTestStateBuilder::default()
            .with_generated_accounts(
                &[&*ALICE, &*BOB, &*CHRIS, &*DAVE],
                false,
                XDaiBalance::new_base(1),
                HoprBalance::new_base(1000),
            )
            .with_channels([c1])
            .build_dynamic_client([1; Address::SIZE].into());

        let mut chain_connector =
            create_trustful_hopr_blokli_connector(&BOB_KP, Default::default(), blokli_sim, [1; Address::SIZE].into())
                .await?;
        chain_connector.connect().await?;

        let afs = AutoFundingStrategyInner {
            cfg: AutoFundingStrategyConfig {
                min_stake_threshold: stake_limit,
                funding_amount: fund_amount,
            },
            interval: std::time::Duration::from_secs(60),
            node: Arc::new(ChainNode(Arc::new(chain_connector))),
            in_flight: Arc::new(DashSet::new()),
        };

        // Balance (10) > min_stake_threshold (7): no funding should be triggered.
        afs.on_own_channel_changed_balance(&c1, ChannelDirection::Outgoing, HoprBalance::zero(), c1.balance)
            .await?;
        assert!(afs.in_flight.is_empty(), "channel above threshold should not be funded");
        Ok(())
    }

    /// Tests the public builder API: `AutoFundingStrategy::new(...).build(node)` must
    /// return a `Box<dyn Strategy + Send>` with the expected Display string.
    #[tokio::test]
    async fn test_build_returns_strategy_trait_object() -> anyhow::Result<()> {
        let blokli_sim = BlokliTestStateBuilder::default()
            .with_generated_accounts(
                &[&*ALICE, &*BOB],
                false,
                XDaiBalance::new_base(1),
                HoprBalance::new_base(1000),
            )
            .with_channels([])
            .build_dynamic_client([1; Address::SIZE].into());

        let mut chain_connector =
            create_trustful_hopr_blokli_connector(&BOB_KP, Default::default(), blokli_sim, [1; Address::SIZE].into())
                .await?;
        chain_connector.connect().await?;
        let node = Arc::new(ChainNode(Arc::new(chain_connector)));

        let strategy: Box<dyn crate::strategy::Strategy + Send> =
            super::AutoFundingStrategy::new(AutoFundingStrategyConfig::default(), std::time::Duration::from_secs(60))
                .build(node);

        assert_eq!(strategy.to_string(), "auto_funding");
        // Verify the box is Send (compile-time check via trait object)
        fn assert_send<T: Send>(_: T) {}
        assert_send(strategy);

        Ok(())
    }
}
