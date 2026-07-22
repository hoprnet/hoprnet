//! Fast-lane event handlers that react to `ActionableEvent::Chain` events
//! without waiting for the next full pipeline tick.

use std::time::Instant;

use hopr_api::{
    chain::{
        ChainReadAccountOperations, ChainReadChannelOperations, ChainReadSafeOperations, ChainValues,
        ChainWriteChannelOperations,
    },
    node::{ActionableEventSource, HasChainApi, HasGraphView, HasNetworkView},
    types::{
        internal::prelude::{ChannelDirection, ChannelEntry, ChannelStatus},
        primitive::prelude::Address,
    },
};

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

        // Reuse the economics resolved by the most-recent pipeline tick rather
        // than issuing fresh chain RPC calls on every balance-decrease event.
        let Some(funding) = *self.last_resolved_funding.lock() else {
            tracing::debug!(%ch, "channel-lifecycle: event-driven funding skipped: no tick-resolved economics yet");
            return;
        };

        if ch.balance < funding.lower_balance_threshold {
            match self.safe_balance_budget().await {
                Ok(budget) if budget >= funding.topup_balance => {
                    self.try_fund_channel(&ch, funding.topup_balance);
                }
                Ok(budget) => {
                    tracing::debug!(%ch, %budget, "channel-lifecycle: event-driven funding skipped: safe too low");
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
        tracing::info!(%ch, "channel-lifecycle: channel balance increased");
    }

    pub(super) fn on_channel_opened(&self, ch: ChannelEntry) {
        self.open_in_flight.remove(&ch.destination);
        tracing::info!(%ch, "channel-lifecycle: channel opened");
    }

    pub(super) fn on_channel_closure_initiated(&self, ch: ChannelEntry) {
        self.close_in_flight.remove(ch.get_id());
        tracing::info!(%ch, "channel-lifecycle: channel closure initiated");
    }

    /// Starts the peer cooldown so the channel is not immediately re-opened.
    pub(super) fn on_channel_closed(&self, ch: ChannelEntry) {
        self.finalize_in_flight.remove(ch.get_id());
        self.last_observed.remove(ch.get_id());
        self.peer_ticket_activity.remove(&ch.destination);
        let until = Instant::now() + self.cfg.population.peer_reopen_cooldown;
        self.cooldown.insert(ch.destination, until);
        tracing::info!(%ch, "channel-lifecycle: channel closed");
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
