//! Builder, `Display`, and `Strategy` trait implementation.

use std::{
    fmt::{Debug, Display, Formatter},
    sync::Arc,
    time::Duration,
};

use async_trait::async_trait;
use dashmap::{DashMap, DashSet};
use futures::StreamExt as _;
use hopr_api::{
    chain::{
        ChainEvent, ChainReadAccountOperations, ChainReadChannelOperations, ChainReadSafeOperations, ChainValues,
        ChainWriteChannelOperations,
    },
    node::{
        ActionableEvent, ActionableEventDiscriminant, ActionableEventSource, HasChainApi, HasGraphView, HasNetworkView,
    },
};

use super::{
    ChannelLifecycleConfig, ChannelLifecycleStrategyInner,
    selector::{DefaultSelector, MultiObjectiveSelector},
};
use crate::{errors::StrategyError, strategy::Strategy as StrategyTrait};

/// Builder for [`ChannelLifecycleStrategy`].
///
/// Cadence (`tick_interval`, `jitter`) lives in the config struct — operators
/// can tune it without touching wiring code.
///
/// Call [`new`](ChannelLifecycleStrategy::new) with the strategy configuration,
/// then [`build`](ChannelLifecycleStrategy::build) to wire in a node and obtain
/// a runnable `Box<dyn Strategy + Send>`.
pub struct ChannelLifecycleStrategy {
    cfg: ChannelLifecycleConfig,
}

impl ChannelLifecycleStrategy {
    /// Create a new builder with the given configuration.
    pub fn new(cfg: ChannelLifecycleConfig) -> Self {
        Self { cfg }
    }

    /// Wire in a node and return a running-ready strategy.
    ///
    /// The generic `N` is erased at construction time; the returned
    /// `Box<dyn Strategy + Send>` can be held and spawned without knowledge
    /// of the concrete node type.
    pub fn build<N>(self, node: Arc<N>) -> Box<dyn StrategyTrait + Send>
    where
        N: HasChainApi + HasNetworkView + HasGraphView + ActionableEventSource + Send + Sync + 'static,
        N::ChainApi: ChainReadChannelOperations
            + ChainReadSafeOperations
            + ChainReadAccountOperations
            + ChainValues
            + ChainWriteChannelOperations
            + Clone
            + Send
            + Sync
            + 'static,
    {
        let selector: Arc<dyn super::selector::Selector> = match self.cfg.selector.multi_objective_config() {
            Some(mo_cfg) => {
                mo_cfg
                    .validate_trust_weights()
                    .expect("invalid selector config: trust inner weights must sum to ~1.0");
                Arc::new(MultiObjectiveSelector::new(mo_cfg))
            }
            None => Arc::new(DefaultSelector),
        };

        Box::new(ChannelLifecycleStrategyInner {
            cfg: self.cfg,
            node,
            selector,
            open_in_flight: Arc::new(DashSet::new()),
            fund_in_flight: Arc::new(DashSet::new()),
            close_in_flight: Arc::new(DashSet::new()),
            finalize_in_flight: Arc::new(DashSet::new()),
            cooldown: Arc::new(DashMap::new()),
            start_epoch: std::time::Instant::now(),
            last_observed: Arc::new(DashMap::new()),
            peer_ticket_activity: Arc::new(DashMap::new()),
            peer_addr_cache: Arc::new(parking_lot::Mutex::new(None)),
        })
    }
}

impl<N> Debug for ChannelLifecycleStrategyInner<N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "ChannelLifecycleStrategy({:?})", self.cfg)
    }
}

impl<N> Display for ChannelLifecycleStrategyInner<N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "channel_lifecycle")
    }
}

#[async_trait]
impl<N> StrategyTrait for ChannelLifecycleStrategyInner<N>
where
    N: HasChainApi + HasNetworkView + HasGraphView + ActionableEventSource + Send + Sync + 'static,
    N::ChainApi: ChainReadChannelOperations
        + ChainReadSafeOperations
        + ChainReadAccountOperations
        + ChainValues
        + ChainWriteChannelOperations
        + Clone
        + Send
        + Sync
        + 'static,
{
    async fn run(&mut self) -> crate::errors::Result<()> {
        tracing::info!(
            target = self.cfg.population.target_open_channels,
            min = self.cfg.population.min_open_channels,
            tick_interval_secs = self.cfg.tick_interval.as_secs(),
            initial_capacity = %self.cfg.funding.initial_capacity,
            "channel-lifecycle: strategy started"
        );
        self.run_pipeline().await;

        let me = *self.node.chain_api().me();

        // Derive a fixed per-run jitter offset from system-time nanoseconds so
        // nodes restarted simultaneously spread out their ticks.  Use the full
        // nanosecond reading (not just the sub-second component) so the jitter
        // window is not silently capped at <1 s.
        let jitter_ns = self.cfg.jitter.as_nanos();
        let jitter_offset = if jitter_ns > 0 {
            let now_ns = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos();
            Duration::from_nanos((now_ns % jitter_ns) as u64)
        } else {
            Duration::ZERO
        };
        let effective_interval = self.cfg.tick_interval + jitter_offset;

        let tick_stream = futures_time::stream::interval(effective_interval.into()).map(|_| LoopEvent::Tick);

        let event_stream = self
            .node
            .subscribe_to_actionable_events(Some(&[ActionableEventDiscriminant::Chain]))
            .map_err(|e| StrategyError::Other(anyhow::anyhow!(e)))?
            .filter_map(|ev| {
                futures::future::ready(match ev {
                    ActionableEvent::Chain(e) => Some(LoopEvent::Chain(Box::new(e))),
                    _ => None,
                })
            });

        let mut driver = futures_concurrency::stream::Merge::merge((tick_stream, event_stream));

        while let Some(evt) = driver.next().await {
            match evt {
                LoopEvent::Tick => {
                    self.run_pipeline().await;
                }
                LoopEvent::Chain(e) => match *e {
                    ChainEvent::ChannelBalanceDecreased(ch, _) => {
                        self.on_balance_decreased(ch, me).await;
                    }
                    ChainEvent::ChannelBalanceIncreased(ch, _) => {
                        self.on_balance_increased(ch);
                    }
                    ChainEvent::ChannelOpened(ch) => {
                        self.on_channel_opened(ch);
                    }
                    ChainEvent::ChannelClosureInitiated(ch) => {
                        self.on_channel_closure_initiated(ch);
                    }
                    ChainEvent::ChannelClosed(ch) => {
                        self.on_channel_closed(ch);
                    }
                    ChainEvent::TicketRedeemed(ch, _) => {
                        self.on_ticket_redeemed(ch);
                    }
                    _ => {}
                },
            }
        }

        Ok(())
    }
}

enum LoopEvent {
    Tick,
    Chain(Box<ChainEvent>),
}
