//! ## Auto Redeeming Strategy
//! This strategy listens for newly added acknowledged tickets and automatically issues a redeem transaction on that
//! ticket. It can be configured to automatically redeem all tickets or only aggregated tickets (which results in far
//! fewer on-chain transactions being issued).
//!
//! For details on default parameters, see [AutoRedeemingStrategyConfig].
use std::{
    fmt::{Debug, Display, Formatter},
    str::FromStr,
};

use async_trait::async_trait;
use futures::StreamExt;
use hopr_api::{
    chain::{ChainReadChannelOperations, ChainWriteTicketOperations, ChannelSelector},
    db::TicketSelector,
};
use hopr_internal_types::{prelude::*, tickets::AcknowledgedTicket};
use hopr_primitive_types::prelude::*;
use serde::{Deserialize, Serialize};
use serde_with::{DisplayFromStr, serde_as};
use tracing::{debug, info};
use validator::Validate;

use crate::{
    Strategy,
    errors::{StrategyError, StrategyError::CriteriaNotSatisfied},
    strategy::SingularStrategy,
};

#[cfg(all(feature = "prometheus", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_COUNT_AUTO_REDEEMS:  hopr_metrics::SimpleCounter =
         hopr_metrics::SimpleCounter::new("hopr_strategy_auto_redeem_redeem_count", "Count of initiated automatic redemptions").unwrap();
}

fn just_true() -> bool {
    true
}

fn just_false() -> bool {
    false
}

fn min_redeem_hopr() -> HoprBalance {
    HoprBalance::from_str("1 wxHOPR").unwrap()
}

/// Configuration object for the `AutoRedeemingStrategy`
#[serde_as]
#[derive(Debug, Clone, Copy, PartialEq, Eq, smart_default::SmartDefault, Validate, Serialize, Deserialize)]
pub struct AutoRedeemingStrategyConfig {
    /// If set, the strategy will redeem only aggregated tickets.
    /// Otherwise, it redeems all acknowledged winning tickets.
    ///
    /// Default is `false`.
    #[serde(default = "just_false")]
    #[default = false]
    pub redeem_only_aggregated: bool,

    /// If set to true, will redeem all tickets in the channel (which are over the
    /// `minimum_redeem_ticket_value` threshold) once it transitions to `PendingToClose`.
    ///
    /// Default is `true`.
    #[serde(default = "just_true")]
    #[default = true]
    pub redeem_all_on_close: bool,

    /// The strategy will only redeem an acknowledged winning ticket if it has at least this value of HOPR.
    /// If 0 is given, the strategy will redeem tickets regardless of their value.
    ///
    /// Default is `1 wxHOPR`.
    #[serde(default = "min_redeem_hopr")]
    #[serde_as(as = "DisplayFromStr")]
    #[default(min_redeem_hopr())]
    pub minimum_redeem_ticket_value: HoprBalance,

    /// If set, the strategy will redeem each incoming winning ticket.
    /// Otherwise, it will try to redeem tickets in all channels periodically.
    ///
    /// Set this to `true` when winning tickets are not happening too often (e.g., when winning probability
    /// is below 1%).
    /// Set this to `false` when winning tickets are happening very often (e.g., when winning probability
    /// is above 1%).
    ///
    /// Default is `true`
    #[serde(default = "just_false")]
    #[default = false]
    pub redeem_on_winning: bool,
}

/// The `AutoRedeemingStrategy` automatically sends an acknowledged ticket
/// for redemption once encountered.
/// The strategy does not await the result of the redemption.
pub struct AutoRedeemingStrategy<A> {
    hopr_chain_actions: A,
    cfg: AutoRedeemingStrategyConfig,
}

impl<A> Debug for AutoRedeemingStrategy<A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", Strategy::AutoRedeeming(self.cfg))
    }
}

impl<A> Display for AutoRedeemingStrategy<A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Strategy::AutoRedeeming(self.cfg))
    }
}

impl<A> AutoRedeemingStrategy<A> {
    pub fn new(cfg: AutoRedeemingStrategyConfig, hopr_chain_actions: A) -> Self {
        Self {
            cfg,
            hopr_chain_actions,
        }
    }
}

#[async_trait]
impl<A> SingularStrategy for AutoRedeemingStrategy<A>
where
    A: ChainWriteTicketOperations + ChainReadChannelOperations + Send + Sync,
{
    async fn on_tick(&self) -> crate::errors::Result<()> {
        if !self.cfg.redeem_on_winning {
            debug!("trying to redeem all tickets in all channels");

            let all_channels = self
                .hopr_chain_actions
                .stream_channels(ChannelSelector {
                    direction: vec![ChannelDirection::Incoming],
                    allowed_states: vec![
                        ChannelStatusDiscriminants::Open,
                        ChannelStatusDiscriminants::PendingToClose,
                    ],
                    ..Default::default()
                })
                .await
                .map_err(|e| StrategyError::Other(e.into()))?
                .collect::<Vec<_>>()
                .await;

            if all_channels.is_empty() {
                return Err(CriteriaNotSatisfied);
            }

            let mut selector = TicketSelector::from(all_channels.first().unwrap())
                .with_amount(self.cfg.minimum_redeem_ticket_value..)
                .with_aggregated_only(self.cfg.redeem_only_aggregated);
            for channel in all_channels.iter().skip(1) {
                selector = selector.also_on_channel_entry(channel);
            }

            let count = self
                .hopr_chain_actions
                .redeem_tickets_via_selector(selector)
                .await
                .map_err(|e| StrategyError::Other(e.into()))?
                .len();
            if count > 0 {
                #[cfg(all(feature = "prometheus", not(test)))]
                METRIC_COUNT_AUTO_REDEEMS.increment_by(count as u64);

                info!(count, "strategy issued ticket redemptions");
            } else {
                debug!(count, "strategy issued no ticket redemptions");
            }

            Ok(())
        } else {
            Err(CriteriaNotSatisfied)
        }
    }

    async fn on_acknowledged_winning_ticket(&self, ack: &AcknowledgedTicket) -> crate::errors::Result<()> {
        if self.cfg.redeem_on_winning
            && ((!self.cfg.redeem_only_aggregated || ack.verified_ticket().is_aggregated())
                && ack.verified_ticket().amount.ge(&self.cfg.minimum_redeem_ticket_value))
        {
            if let Some(channel) = self
                .hopr_chain_actions
                .channel_by_id(&ack.verified_ticket().channel_id)
                .await
                .map_err(|e| StrategyError::Other(e.into()))?
            {
                info!(%ack, "redeeming");

                #[cfg(all(feature = "prometheus", not(test)))]
                METRIC_COUNT_AUTO_REDEEMS.increment();

                let rxs = self
                    .hopr_chain_actions
                    .redeem_tickets_via_selector(
                        TicketSelector::from(channel)
                            .with_amount(self.cfg.minimum_redeem_ticket_value..)
                            .with_aggregated_only(self.cfg.redeem_only_aggregated),
                    )
                    .await
                    .map_err(|e| StrategyError::Other(e.into()))?;

                std::mem::drop(rxs); // The Receiver is not intentionally awaited here and the oneshot Sender can fail safely

                Ok(())
            } else {
                Err(CriteriaNotSatisfied)
            }
        } else {
            Err(CriteriaNotSatisfied)
        }
    }

    async fn on_own_channel_changed(
        &self,
        channel: &ChannelEntry,
        direction: ChannelDirection,
        change: ChannelChange,
    ) -> crate::errors::Result<()> {
        if direction != ChannelDirection::Incoming || !self.cfg.redeem_all_on_close {
            return Ok(());
        }

        if let ChannelChange::Status { left: old, right: new } = change {
            if old != ChannelStatus::Open || !matches!(new, ChannelStatus::PendingToClose(_)) {
                debug!(?channel, "ignoring channel state change that's not in PendingToClose");
                return Ok(());
            }
            info!(%channel, "channel transitioned to PendingToClose, checking if it has tickets to redeem");

            let count = self
                .hopr_chain_actions
                .redeem_tickets_via_selector(
                    TicketSelector::from(channel)
                        .with_amount(self.cfg.minimum_redeem_ticket_value..)
                        .with_aggregated_only(self.cfg.redeem_only_aggregated),
                )
                .await
                .map_err(|e| StrategyError::Other(e.into()))?
                .len();

            #[cfg(all(feature = "prometheus", not(test)))]
            METRIC_COUNT_AUTO_REDEEMS.increment_by(count as u64);

            if count > 0 {
                info!(count, %channel, "tickets in channel being closed sent for redemption");
                Ok(())
            } else {
                info!(%channel, "no redeemable tickets with minimum amount in channel being closed");
                Err(CriteriaNotSatisfied)
            }
        } else {
            Err(CriteriaNotSatisfied)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        ops::Add,
        time::{Duration, SystemTime},
    };

    use async_trait::async_trait;
    use futures::{FutureExt, future::ok};
    use hex_literal::hex;
    use hopr_chain_types::{actions::Action, chain_events::ChainEventType};
    use hopr_crypto_random::{Randomizable, random_bytes};
    use hopr_crypto_types::prelude::*;
    use hopr_db_sql::info::HoprDbInfoOperations;
    use mockall::mock;

    use super::*;

    lazy_static::lazy_static! {
        static ref ALICE: ChainKeypair = ChainKeypair::from_secret(&hex!("492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775")).expect("lazy static keypair should be valid");
        static ref BOB: ChainKeypair = ChainKeypair::from_secret(&hex!("48680484c6fc31bc881a0083e6e32b6dc789f9eaba0f8b981429fd346c697f8c")).expect("lazy static keypair should be valid");
        static ref PRICE_PER_PACKET: U256 = 10000000000000000_u128.into(); // 0.01 HOPR
    }

    fn generate_random_ack_ticket(
        index: u64,
        idx_offset: u32,
        worth_packets: u32,
    ) -> anyhow::Result<AcknowledgedTicket> {
        let hk1 = HalfKey::random();
        let hk2 = HalfKey::random();

        let challenge = Response::from_half_keys(&hk1, &hk2)?.to_challenge()?;

        Ok(TicketBuilder::default()
            .addresses(&*BOB, &*ALICE)
            .amount(PRICE_PER_PACKET.div_f64(1.0f64)? * worth_packets)
            .index(index)
            .index_offset(idx_offset)
            .win_prob(WinningProbability::ALWAYS)
            .channel_epoch(4)
            .challenge(challenge)
            .build_signed(&BOB, &Hash::default())?
            .into_acknowledged(Response::from_half_keys(&hk1, &hk2)?))
    }

    mock! {
        TicketRedeemAct { }
        #[async_trait]
        impl TicketRedeemActions for TicketRedeemAct {
            async fn redeem_all_tickets(&self, min_value: HoprBalance, only_aggregated: bool) -> hopr_chain_actions::errors::Result<Vec<PendingAction>>;
            async fn redeem_tickets_with_counterparty(
                &self,
                counterparty: &Address,
                min_value: HoprBalance,
                only_aggregated: bool,
            ) -> hopr_chain_actions::errors::Result<Vec<PendingAction>>;
            async fn redeem_tickets_in_channel(
                &self,
                channel: &ChannelEntry,
                min_value: HoprBalance,
                only_aggregated: bool,
            ) -> hopr_chain_actions::errors::Result<Vec<PendingAction >>;
            async fn redeem_tickets(&self, selector: TicketSelector) -> hopr_chain_actions::errors::Result<Vec<PendingAction>>;
            async fn redeem_ticket(&self, ack: AcknowledgedTicket) -> hopr_chain_actions::errors::Result<PendingAction>;
        }
    }

    fn mock_action_confirmation(ack: AcknowledgedTicket) -> anyhow::Result<ActionConfirmation> {
        let random_hash = Hash::from(random_bytes::<{ Hash::SIZE }>());
        Ok(ActionConfirmation {
            tx_hash: random_hash,
            event: Some(ChainEventType::TicketRedeemed(
                ChannelEntry::new(
                    BOB.public().to_address(),
                    ALICE.public().to_address(),
                    10.into(),
                    U256::zero(),
                    ChannelStatus::Open,
                    U256::zero(),
                ),
                Some(ack.clone()),
            )),
            action: Action::RedeemTicket(ack.into_redeemable(&ALICE, &Hash::default())?),
        })
    }

    #[tokio::test]
    async fn test_auto_redeeming_strategy_redeem() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(ALICE.clone()).await?;
        db.set_domain_separator(None, DomainSeparator::Channel, Default::default())
            .await?;

        let ack_ticket = generate_random_ack_ticket(0, 1, 5)?;

        let mut actions = MockTicketRedeemAct::new();
        let mock_confirm = mock_action_confirmation(ack_ticket.clone())?;
        actions
            .expect_redeem_tickets_with_counterparty()
            .once()
            .with(
                mockall::predicate::eq(ack_ticket.ticket.verified_issuer().clone()),
                mockall::predicate::eq(HoprBalance::from(0)),
                mockall::predicate::eq(false),
            )
            .return_once(move |_, _, _| Ok(vec![ok(mock_confirm).boxed()]));

        let cfg = AutoRedeemingStrategyConfig {
            redeem_only_aggregated: false,
            minimum_redeem_ticket_value: 0.into(),
            redeem_on_winning: true,
            ..Default::default()
        };

        let ars = AutoRedeemingStrategy::new(cfg, actions);
        ars.on_acknowledged_winning_ticket(&ack_ticket).await?;
        assert!(ars.on_tick().await.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn test_auto_redeeming_strategy_redeem_on_tick() -> anyhow::Result<()> {
        let ack_ticket = generate_random_ack_ticket(0, 1, 5)?;

        let mut actions = MockTicketRedeemAct::new();
        let mock_confirm = mock_action_confirmation(ack_ticket.clone())?;
        actions
            .expect_redeem_all_tickets()
            .once()
            .with(
                mockall::predicate::eq(HoprBalance::from(*PRICE_PER_PACKET * 5)),
                mockall::predicate::eq(false),
            )
            .return_once(move |_, _| Ok(vec![ok(mock_confirm).boxed()]));

        let cfg = AutoRedeemingStrategyConfig {
            redeem_only_aggregated: false,
            minimum_redeem_ticket_value: HoprBalance::from(*PRICE_PER_PACKET * 5),
            redeem_on_winning: false,
            ..Default::default()
        };

        let ars = AutoRedeemingStrategy::new(cfg, actions);
        ars.on_tick().await?;
        assert!(ars.on_acknowledged_winning_ticket(&ack_ticket).await.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn test_auto_redeeming_strategy_should_not_redeem_unworthy_tickets_on_tick() -> anyhow::Result<()> {
        // Make the ticket worth less than the threshold
        let ack_ticket = generate_random_ack_ticket(0, 1, 4)?;

        let mut actions = MockTicketRedeemAct::new();
        let mock_confirm = mock_action_confirmation(ack_ticket.clone())?;
        actions
            .expect_redeem_all_tickets()
            .once()
            .with(
                mockall::predicate::eq(HoprBalance::from(*PRICE_PER_PACKET * 5)),
                mockall::predicate::eq(false),
            )
            .return_once(move |_, _| Ok(vec![ok(mock_confirm).boxed()]));

        let cfg = AutoRedeemingStrategyConfig {
            redeem_only_aggregated: false,
            minimum_redeem_ticket_value: HoprBalance::from(*PRICE_PER_PACKET * 5),
            redeem_on_winning: false,
            ..Default::default()
        };

        let ars = AutoRedeemingStrategy::new(cfg, actions);
        ars.on_tick().await?;
        assert!(ars.on_acknowledged_winning_ticket(&ack_ticket).await.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn test_auto_redeeming_strategy_redeem_agg_only() -> anyhow::Result<()> {
        let ack_ticket_unagg = generate_random_ack_ticket(0, 1, 5)?;
        let ack_ticket_agg = generate_random_ack_ticket(0, 3, 5)?;

        let mut actions = MockTicketRedeemAct::new();
        let mock_confirm = mock_action_confirmation(ack_ticket_agg.clone())?;
        actions
            .expect_redeem_tickets_with_counterparty()
            .once()
            .with(
                mockall::predicate::eq(ack_ticket_agg.ticket.verified_issuer().clone()),
                mockall::predicate::eq(HoprBalance::from(0)),
                mockall::predicate::eq(true),
            )
            .return_once(move |_, _, _| Ok(vec![ok(mock_confirm).boxed()]));

        let cfg = AutoRedeemingStrategyConfig {
            redeem_only_aggregated: true,
            minimum_redeem_ticket_value: 0.into(),
            redeem_on_winning: true,
            ..Default::default()
        };

        let ars = AutoRedeemingStrategy::new(cfg, actions);
        ars.on_acknowledged_winning_ticket(&ack_ticket_unagg)
            .await
            .expect_err("non-agg ticket should not satisfy");
        ars.on_acknowledged_winning_ticket(&ack_ticket_agg).await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_auto_redeeming_strategy_redeem_minimum_ticket_amount() -> anyhow::Result<()> {
        let ack_ticket_below = generate_random_ack_ticket(1, 1, 4)?;
        let ack_ticket_at = generate_random_ack_ticket(1, 1, 5)?;

        let mock_confirm = mock_action_confirmation(ack_ticket_at.clone())?;
        let mut actions = MockTicketRedeemAct::new();
        actions
            .expect_redeem_tickets_with_counterparty()
            .once()
            .with(
                mockall::predicate::eq(ack_ticket_at.ticket.verified_issuer().clone()),
                mockall::predicate::eq(HoprBalance::from(*PRICE_PER_PACKET * 5)),
                mockall::predicate::eq(false),
            )
            .return_once(move |_, _, _| Ok(vec![ok(mock_confirm).boxed()]));

        let cfg = AutoRedeemingStrategyConfig {
            redeem_only_aggregated: false,
            minimum_redeem_ticket_value: HoprBalance::from(*PRICE_PER_PACKET * 5),
            redeem_on_winning: true,
            ..Default::default()
        };

        let ars = AutoRedeemingStrategy::new(cfg, actions);
        ars.on_acknowledged_winning_ticket(&ack_ticket_below)
            .await
            .expect_err("ticket below threshold should not satisfy");
        ars.on_acknowledged_winning_ticket(&ack_ticket_at).await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_auto_redeeming_strategy_should_redeem_singular_ticket_on_close() -> anyhow::Result<()> {
        let channel = ChannelEntry::new(
            BOB.public().to_address(),
            ALICE.public().to_address(),
            10.into(),
            0.into(),
            ChannelStatus::PendingToClose(SystemTime::now().add(Duration::from_secs(100))),
            4.into(),
        );

        // Make the ticket worth exactly the threshold
        let ack_ticket = generate_random_ack_ticket(0, 1, 5)?;

        let mut actions = MockTicketRedeemAct::new();
        let mock_confirm = mock_action_confirmation(ack_ticket)?;
        actions
            .expect_redeem_tickets_in_channel()
            .once()
            .with(
                mockall::predicate::eq(channel),
                mockall::predicate::eq(HoprBalance::from(*PRICE_PER_PACKET * 5)),
                mockall::predicate::eq(true),
            )
            .return_once(move |_, _, _| Ok(vec![ok(mock_confirm).boxed()]));

        let cfg = AutoRedeemingStrategyConfig {
            redeem_only_aggregated: true,
            redeem_all_on_close: true,
            minimum_redeem_ticket_value: HoprBalance::from(*PRICE_PER_PACKET * 5),
            ..Default::default()
        };

        let ars = AutoRedeemingStrategy::new(cfg, actions);
        ars.on_own_channel_changed(
            &channel,
            ChannelDirection::Incoming,
            ChannelChange::Status {
                left: ChannelStatus::Open,
                right: channel.status,
            },
        )
        .await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_auto_redeeming_strategy_should_not_redeem_unworthy_tickets_on_close() -> anyhow::Result<()> {
        let channel = ChannelEntry::new(
            BOB.public().to_address(),
            ALICE.public().to_address(),
            10.into(),
            0.into(),
            ChannelStatus::PendingToClose(SystemTime::now().add(Duration::from_secs(100))),
            4.into(),
        );

        let ack_ticket = generate_random_ack_ticket(1, 1, 3)?;

        let mut actions = MockTicketRedeemAct::new();
        let mock_confirm = mock_action_confirmation(ack_ticket.clone())?;
        actions
            .expect_redeem_tickets_in_channel()
            .once()
            .with(
                mockall::predicate::eq(channel),
                mockall::predicate::eq(HoprBalance::from(*PRICE_PER_PACKET * 5)),
                mockall::predicate::eq(false),
            )
            .return_once(move |_, _, _| Ok(vec![ok(mock_confirm).boxed()]));

        let cfg = AutoRedeemingStrategyConfig {
            minimum_redeem_ticket_value: HoprBalance::from(*PRICE_PER_PACKET * 5),
            redeem_only_aggregated: false,
            redeem_all_on_close: true,
            ..Default::default()
        };

        let ars = AutoRedeemingStrategy::new(cfg, actions);
        ars.on_own_channel_changed(
            &channel,
            ChannelDirection::Incoming,
            ChannelChange::Status {
                left: ChannelStatus::Open,
                right: channel.status,
            },
        )
        .await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_auto_redeeming_strategy_redeem_multiple_tickets_in_channel() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(ALICE.clone()).await?;
        db.set_domain_separator(None, DomainSeparator::Channel, Default::default())
            .await?;

        let ack_ticket_0 = generate_random_ack_ticket(0, 1, 5)?;
        let ack_ticket_1 = generate_random_ack_ticket(0, 2, 5)?;

        let mut actions: MockTicketRedeemAct = MockTicketRedeemAct::new();
        let mock_confirms = vec![
            mock_action_confirmation(ack_ticket_0.clone())?,
            mock_action_confirmation(ack_ticket_1.clone())?,
        ];
        actions
            .expect_redeem_tickets_with_counterparty()
            .once()
            .with(
                mockall::predicate::eq(ack_ticket_1.ticket.verified_issuer().clone()),
                mockall::predicate::eq(HoprBalance::from(0)),
                mockall::predicate::eq(false),
            )
            .return_once(move |_, _, _| Ok(mock_confirms.into_iter().map(|c| ok(c).boxed()).collect()));

        let cfg = AutoRedeemingStrategyConfig {
            redeem_only_aggregated: false,
            minimum_redeem_ticket_value: 0.into(),
            redeem_on_winning: true,
            ..Default::default()
        };

        let ars = AutoRedeemingStrategy::new(cfg, actions);
        ars.on_acknowledged_winning_ticket(&ack_ticket_1).await?;
        assert!(ars.on_tick().await.is_err());

        Ok(())
    }
}
