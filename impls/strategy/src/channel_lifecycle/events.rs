//! Fast-lane event handlers that react to `ActionableEvent::Chain` events
//! without waiting for the next full pipeline tick.

use std::time::Instant;

use hopr_api::{
    chain::{
        ChainReadAccountOperations, ChainReadChannelOperations, ChainReadSafeOperations, ChainValues,
        ChainWriteChannelOperations, WinningProbability,
    },
    node::{ActionableEventSource, HasChainApi, HasGraphView, HasNetworkView},
    types::{
        internal::prelude::{ChannelDirection, ChannelEntry, ChannelStatus},
        primitive::prelude::{Address, HoprBalance},
    },
};
use tracing::{debug, info};

use super::{ChannelLifecycleStrategyInner, ChannelObservation};

impl<N> ChannelLifecycleStrategyInner<N>
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
    /// Funds the channel immediately if balance dropped below threshold.
    pub(super) async fn on_balance_decreased(&self, ch: ChannelEntry, me: Address) {
        if ch.direction(&me) != Some(ChannelDirection::Outgoing) {
            return;
        }
        if ch.status != ChannelStatus::Open {
            return;
        }
        if self.fund_in_flight.contains(ch.get_id()) {
            return;
        }

        self.last_observed
            .entry(*ch.get_id())
            .and_modify(|obs| {
                obs.balance = ch.balance;
                obs.at = Instant::now();
            })
            .or_insert_with(|| ChannelObservation {
                balance: ch.balance,
                ticket_index: ch.ticket_index,
                at: Instant::now(),
            });

        // Resolve the funding thresholds from the current ticket economics.
        let chain = self.node.chain_api();
        let price = chain
            .minimum_ticket_price()
            .await
            .unwrap_or_else(|e| {
                debug!(%e, "channel-lifecycle: event-driven funding: ticket price unavailable, using zero");
                HoprBalance::zero()
            });
        let win_prob = chain
            .minimum_incoming_ticket_win_prob()
            .await
            .unwrap_or_else(|e| {
                debug!(%e, "channel-lifecycle: event-driven funding: win_prob unavailable, using ALWAYS");
                WinningProbability::ALWAYS
            });
        let funding = self.cfg.funding.resolve(price, win_prob);

        if ch.balance <= funding.lower_balance_threshold {
            match self.safe_balance_budget().await {
                Ok(budget) if budget >= funding.topup_balance => {
                    self.try_fund_channel(&ch, funding.topup_balance);
                }
                Ok(budget) => {
                    debug!(%ch, %budget, "channel-lifecycle: event-driven funding skipped: safe too low");
                }
                Err(e) => {
                    tracing::warn!(%ch, %e, "channel-lifecycle: event-driven funding: could not fetch safe balance");
                }
            }
        }
    }

    pub(super) fn on_balance_increased(&self, ch: ChannelEntry) {
        self.fund_in_flight.remove(ch.get_id());
        self.last_observed.entry(*ch.get_id()).and_modify(|obs| {
            obs.balance = ch.balance;
        });
        info!(%ch, "channel-lifecycle: channel balance increased");
    }

    pub(super) fn on_channel_opened(&self, ch: ChannelEntry) {
        self.open_in_flight.remove(&ch.destination);
        info!(%ch, "channel-lifecycle: channel opened");
    }

    pub(super) fn on_channel_closure_initiated(&self, ch: ChannelEntry) {
        self.close_in_flight.remove(ch.get_id());
        info!(%ch, "channel-lifecycle: channel closure initiated");
    }

    /// Starts the peer cooldown so the channel is not immediately re-opened.
    pub(super) fn on_channel_closed(&self, ch: ChannelEntry) {
        self.finalize_in_flight.remove(ch.get_id());
        self.last_observed.remove(ch.get_id());
        self.peer_ticket_activity.remove(&ch.destination);
        let until = Instant::now() + self.cfg.population.peer_reopen_cooldown;
        self.cooldown.insert(ch.destination, until);
        info!(%ch, "channel-lifecycle: channel closed");
    }

    /// Records ticket activity for the proactive drain-rate estimate.
    pub(super) fn on_ticket_redeemed(&self, ch: ChannelEntry) {
        self.peer_ticket_activity
            .entry(ch.destination)
            .and_modify(|v| *v += 1)
            .or_insert(1);
        self.last_observed.entry(*ch.get_id()).and_modify(|obs| {
            obs.ticket_index = ch.ticket_index;
        });
    }
}
