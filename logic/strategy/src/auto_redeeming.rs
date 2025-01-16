//! ## Auto Redeeming Strategy
//! This strategy listens for newly added acknowledged tickets and automatically issues a redeem transaction on that ticket.
//! It can be configured to automatically redeem all tickets or only aggregated tickets (which results in far fewer on-chain transactions being issued).
//!
//! For details on default parameters, see [AutoRedeemingStrategyConfig].
use async_trait::async_trait;
use chain_actions::redeem::TicketRedeemActions;
use hopr_db_sql::api::tickets::HoprDbTicketOperations;
use hopr_db_sql::prelude::TicketSelector;
use hopr_internal_types::prelude::*;
use hopr_internal_types::tickets::{AcknowledgedTicket, AcknowledgedTicketStatus};
use hopr_primitive_types::prelude::*;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use std::fmt::{Debug, Display, Formatter};
use tracing::{debug, error, info};
use validator::Validate;

use crate::errors::StrategyError::CriteriaNotSatisfied;
use crate::strategy::SingularStrategy;
use crate::Strategy;

#[cfg(all(feature = "prometheus", not(test)))]
use hopr_metrics::metrics::SimpleCounter;

#[cfg(all(feature = "prometheus", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_COUNT_AUTO_REDEEMS: SimpleCounter =
        SimpleCounter::new("hopr_strategy_auto_redeem_redeem_count", "Count of initiated automatic redemptions").unwrap();
}

fn just_true() -> bool {
    true
}

fn min_redeem_hopr() -> Balance {
    Balance::new_from_str("90000000000000000", BalanceType::HOPR)
}

/// Configuration object for the `AutoRedeemingStrategy`
#[serde_as]
#[derive(Debug, Clone, Copy, PartialEq, Eq, smart_default::SmartDefault, Validate, Serialize, Deserialize)]
pub struct AutoRedeemingStrategyConfig {
    /// If set, the strategy will redeem only aggregated tickets.
    /// Otherwise, it redeems all acknowledged winning tickets.
    ///
    /// Default is true.
    #[serde(default = "just_true")]
    #[default = true]
    pub redeem_only_aggregated: bool,

    /// If set to true, will redeem all tickets in the channel (which are over the
    /// `minimum_redeem_ticket_value` threshold) once it transitions to `PendingToClose`.
    ///
    /// Default is true.
    #[serde(default = "just_true")]
    #[default = true]
    pub redeem_all_on_close: bool,

    /// The strategy will only redeem an acknowledged winning ticket if it has at least this value of HOPR.
    /// If 0 is given, the strategy will redeem tickets regardless of their value.
    /// This is not used for cases where `on_close_redeem_single_tickets_value_min` applies.
    ///
    /// Default is 0.09 HOPR.
    #[serde(default = "min_redeem_hopr")]
    #[serde_as(as = "DisplayFromStr")]
    #[default(Balance::new_from_str("90000000000000000", BalanceType::HOPR))]
    pub minimum_redeem_ticket_value: Balance,
}

/// The `AutoRedeemingStrategy` automatically sends an acknowledged ticket
/// for redemption once encountered.
/// The strategy does not await the result of the redemption.
pub struct AutoRedeemingStrategy<A: TicketRedeemActions, Db: HoprDbTicketOperations> {
    chain_actions: A,
    db: Db,
    cfg: AutoRedeemingStrategyConfig,
}

impl<A: TicketRedeemActions, Db: HoprDbTicketOperations> Debug for AutoRedeemingStrategy<A, Db> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", Strategy::AutoRedeeming(self.cfg))
    }
}

impl<A: TicketRedeemActions, Db: HoprDbTicketOperations> Display for AutoRedeemingStrategy<A, Db> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Strategy::AutoRedeeming(self.cfg))
    }
}

impl<A: TicketRedeemActions, Db: HoprDbTicketOperations> AutoRedeemingStrategy<A, Db> {
    pub fn new(cfg: AutoRedeemingStrategyConfig, db: Db, chain_actions: A) -> Self {
        Self { cfg, db, chain_actions }
    }
}

#[async_trait]
impl<A, Db> SingularStrategy for AutoRedeemingStrategy<A, Db>
where
    A: TicketRedeemActions + Send + Sync,
    Db: HoprDbTicketOperations + Send + Sync,
{
    async fn on_acknowledged_winning_ticket(&self, ack: &AcknowledgedTicket) -> crate::errors::Result<()> {
        if (!self.cfg.redeem_only_aggregated || ack.verified_ticket().is_aggregated())
            && ack.verified_ticket().amount.ge(&self.cfg.minimum_redeem_ticket_value)
        {
            info!(%ack, "redeeming");

            #[cfg(all(feature = "prometheus", not(test)))]
            METRIC_COUNT_AUTO_REDEEMS.increment();

            let rx = self.chain_actions.redeem_ticket(ack.clone()).await?;
            std::mem::drop(rx); // The Receiver is not intentionally awaited here and the oneshot Sender can fail safely
            Ok(())
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

            let selector = TicketSelector::from(channel)
                .with_state(AcknowledgedTicketStatus::Untouched)
                .with_amount(self.cfg.minimum_redeem_ticket_value..);

            let (redeem_sent_ok, redeem_sent_failed) =
                futures::future::join_all(self.db.get_tickets(selector).await?.into_iter().map(|ack| {
                    info!(%ack, worth = %ack.verified_ticket().amount, "redeeming ticket on channel close");

                    #[cfg(all(feature = "prometheus", not(test)))]
                    METRIC_COUNT_AUTO_REDEEMS.increment();

                    self.chain_actions.redeem_ticket(ack.clone())
                }))
                .await
                .into_iter()
                .map(|submission_res| {
                    if let Err(err) = submission_res {
                        error!(%err, "error while submitting ticket for redemption");
                        (0_usize, 1_usize)
                    } else {
                        // We intentionally do not await the ticket redemption until confirmation
                        (1, 0)
                    }
                })
                .reduce(|(sum_ok, sum_fail), (ok, fail)| (sum_ok + ok, sum_fail + fail))
                .unwrap_or((0, 0));

            if redeem_sent_ok > 0 || redeem_sent_failed > 0 {
                info!(redeem_sent_ok, redeem_sent_failed, %channel, "tickets channel being closed sent for redemption");
                Ok(())
            } else {
                debug!(%channel, "no redeemable tickets with minimum amount in channel being closed");
                Err(CriteriaNotSatisfied)
            }
        } else {
            Err(CriteriaNotSatisfied)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use chain_actions::action_queue::{ActionConfirmation, PendingAction};
    use chain_actions::redeem::TicketRedeemActions;
    use chain_types::actions::Action;
    use chain_types::chain_events::ChainEventType;
    use futures::{future::ok, FutureExt};
    use hex_literal::hex;
    use hopr_crypto_random::random_bytes;
    use hopr_crypto_types::prelude::*;
    use hopr_db_sql::api::tickets::TicketSelector;
    use hopr_db_sql::channels::HoprDbChannelOperations;
    use hopr_db_sql::db::HoprDb;
    use hopr_db_sql::{api::info::DomainSeparator, info::HoprDbInfoOperations};
    use hopr_db_sql::{HoprDbGeneralModelOperations, TargetDb};
    use mockall::mock;
    use std::ops::Add;
    use std::time::{Duration, SystemTime};

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

        let cp1: CurvePoint = hk1.to_challenge().try_into()?;
        let cp2: CurvePoint = hk2.to_challenge().try_into()?;
        let cp_sum = CurvePoint::combine(&[&cp1, &cp2]);

        Ok(TicketBuilder::default()
            .addresses(&*BOB, &*ALICE)
            .amount(PRICE_PER_PACKET.div_f64(1.0f64)? * worth_packets)
            .index(index)
            .index_offset(idx_offset)
            .win_prob(1.0)
            .channel_epoch(4)
            .challenge(Challenge::from(cp_sum).into())
            .build_signed(&BOB, &Hash::default())?
            .into_acknowledged(Response::from_half_keys(&hk1, &hk2)?))
    }

    mock! {
        TicketRedeemAct { }
        #[async_trait]
        impl TicketRedeemActions for TicketRedeemAct {
            async fn redeem_all_tickets(&self, only_aggregated: bool) -> chain_actions::errors::Result<Vec<PendingAction>>;
            async fn redeem_tickets_with_counterparty(
                &self,
                counterparty: &Address,
                only_aggregated: bool,
            ) -> chain_actions::errors::Result<Vec<PendingAction>>;
            async fn redeem_tickets_in_channel(
                &self,
                channel: &ChannelEntry,
                only_aggregated: bool,
            ) -> chain_actions::errors::Result<Vec<PendingAction >>;
            async fn redeem_tickets(&self, selector: TicketSelector) -> chain_actions::errors::Result<Vec<PendingAction>>;
            async fn redeem_ticket(&self, ack: AcknowledgedTicket) -> chain_actions::errors::Result<PendingAction>;
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
                    Balance::new_from_str("10", BalanceType::HOPR),
                    U256::zero(),
                    ChannelStatus::Open,
                    U256::zero(),
                ),
                Some(ack.clone()),
            )),
            action: Action::RedeemTicket(ack.into_redeemable(&ALICE, &Hash::default())?),
        })
    }

    #[async_std::test]
    async fn test_auto_redeeming_strategy_redeem() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(ALICE.clone()).await?;
        db.set_domain_separator(None, DomainSeparator::Channel, Default::default())
            .await?;

        let ack_ticket = generate_random_ack_ticket(0, 1, 5)?;
        let ack_clone = ack_ticket.clone();
        let ack_clone_2 = ack_ticket.clone();

        let mut actions = MockTicketRedeemAct::new();
        let mock_confirm = mock_action_confirmation(ack_clone_2)?;
        actions
            .expect_redeem_ticket()
            .once()
            .withf(move |ack| ack_clone.ticket.eq(&ack.ticket))
            .return_once(move |_| Ok(ok(mock_confirm).boxed()));

        let cfg = AutoRedeemingStrategyConfig {
            redeem_only_aggregated: false,
            minimum_redeem_ticket_value: BalanceType::HOPR.zero(),
            ..Default::default()
        };

        let ars = AutoRedeemingStrategy::new(cfg, db, actions);
        ars.on_acknowledged_winning_ticket(&ack_ticket).await?;

        Ok(())
    }

    #[async_std::test]
    async fn test_auto_redeeming_strategy_redeem_agg_only() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(ALICE.clone()).await?;
        db.set_domain_separator(None, DomainSeparator::Channel, Default::default())
            .await?;

        let ack_ticket_unagg = generate_random_ack_ticket(0, 1, 5)?;
        let ack_ticket_agg = generate_random_ack_ticket(0, 3, 5)?;

        let ack_clone_agg = ack_ticket_agg.clone();
        let ack_clone_agg_2 = ack_ticket_agg.clone();
        let mut actions = MockTicketRedeemAct::new();
        let mock_confirm = mock_action_confirmation(ack_clone_agg_2)?;
        actions
            .expect_redeem_ticket()
            .once()
            .withf(move |ack| ack_clone_agg.ticket.eq(&ack.ticket))
            .return_once(|_| Ok(ok(mock_confirm).boxed()));

        let cfg = AutoRedeemingStrategyConfig {
            redeem_only_aggregated: true,
            minimum_redeem_ticket_value: BalanceType::HOPR.zero(),
            ..Default::default()
        };

        let ars = AutoRedeemingStrategy::new(cfg, db, actions);
        ars.on_acknowledged_winning_ticket(&ack_ticket_unagg)
            .await
            .expect_err("non-agg ticket should not satisfy");
        ars.on_acknowledged_winning_ticket(&ack_ticket_agg).await?;

        Ok(())
    }

    #[async_std::test]
    async fn test_auto_redeeming_strategy_redeem_minimum_ticket_amount() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(ALICE.clone()).await?;
        db.set_domain_separator(None, DomainSeparator::Channel, Default::default())
            .await?;

        let ack_ticket_below = generate_random_ack_ticket(1, 1, 4)?;
        let ack_ticket_at = generate_random_ack_ticket(1, 1, 5)?;

        let ack_clone_at = ack_ticket_at.clone();
        let ack_clone_at_2 = ack_ticket_at.clone();
        let mock_confirm = mock_action_confirmation(ack_clone_at_2)?;
        let mut actions = MockTicketRedeemAct::new();
        actions
            .expect_redeem_ticket()
            .once()
            .withf(move |ack| ack_clone_at.ticket.eq(&ack.ticket))
            .return_once(|_| Ok(ok(mock_confirm).boxed()));

        let cfg = AutoRedeemingStrategyConfig {
            redeem_only_aggregated: false,
            minimum_redeem_ticket_value: BalanceType::HOPR.balance(*PRICE_PER_PACKET * 5),
            ..Default::default()
        };

        let ars = AutoRedeemingStrategy::new(cfg, db, actions);
        ars.on_acknowledged_winning_ticket(&ack_ticket_below)
            .await
            .expect_err("ticket below threshold should not satisfy");
        ars.on_acknowledged_winning_ticket(&ack_ticket_at).await?;

        Ok(())
    }

    #[async_std::test]
    async fn test_auto_redeeming_strategy_should_redeem_singular_ticket_on_close() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(ALICE.clone()).await?;
        db.set_domain_separator(None, DomainSeparator::Channel, Default::default())
            .await?;

        let channel = ChannelEntry::new(
            BOB.public().to_address(),
            ALICE.public().to_address(),
            BalanceType::HOPR.balance(10),
            0.into(),
            ChannelStatus::PendingToClose(SystemTime::now().add(Duration::from_secs(100))),
            4.into(),
        );

        // Make ticket worth exactly the threshold
        let ack_ticket = generate_random_ack_ticket(0, 1, 5)?;

        db.upsert_channel(None, channel).await?;
        db.upsert_ticket(None, ack_ticket.clone()).await?;

        let ack_clone = ack_ticket.clone();
        let ack_clone_2 = ack_ticket.clone();

        let mut actions = MockTicketRedeemAct::new();
        let mock_confirm = mock_action_confirmation(ack_clone_2)?;
        actions
            .expect_redeem_ticket()
            .once()
            .withf(move |ack| ack_clone.ticket.eq(&ack.ticket))
            .return_once(move |_| Ok(ok(mock_confirm).boxed()));

        let cfg = AutoRedeemingStrategyConfig {
            redeem_only_aggregated: true,
            redeem_all_on_close: true,
            minimum_redeem_ticket_value: BalanceType::HOPR.balance(*PRICE_PER_PACKET * 5),
        };

        let ars = AutoRedeemingStrategy::new(cfg, db, actions);
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

    #[async_std::test]
    async fn test_auto_redeeming_strategy_should_not_redeem_singular_ticket_worth_less_on_close() -> anyhow::Result<()>
    {
        let db = HoprDb::new_in_memory(ALICE.clone()).await?;
        db.set_domain_separator(None, DomainSeparator::Channel, Default::default())
            .await?;

        let channel = ChannelEntry::new(
            BOB.public().to_address(),
            ALICE.public().to_address(),
            BalanceType::HOPR.balance(10),
            0.into(),
            ChannelStatus::PendingToClose(SystemTime::now().add(Duration::from_secs(100))),
            0.into(),
        );

        // Make this ticket worth less than the threshold
        let ack_ticket = generate_random_ack_ticket(0, 1, 3)?;

        db.upsert_channel(None, channel).await?;
        db.upsert_ticket(None, ack_ticket).await?;

        let mut actions = MockTicketRedeemAct::new();
        actions.expect_redeem_ticket().never();

        let cfg = AutoRedeemingStrategyConfig {
            minimum_redeem_ticket_value: BalanceType::HOPR.balance(*PRICE_PER_PACKET * 5),
            redeem_only_aggregated: false,
            redeem_all_on_close: true,
        };

        let ars = AutoRedeemingStrategy::new(cfg, db, actions);
        ars.on_own_channel_changed(
            &channel,
            ChannelDirection::Incoming,
            ChannelChange::Status {
                left: ChannelStatus::Open,
                right: channel.status,
            },
        )
        .await
        .expect_err("event should not satisfy criteria");
        Ok(())
    }

    #[async_std::test]
    async fn test_auto_redeeming_strategy_should_not_redeem_unworthy_tickets_on_close() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(ALICE.clone()).await?;
        db.set_domain_separator(None, DomainSeparator::Channel, Default::default())
            .await?;

        let channel = ChannelEntry::new(
            BOB.public().to_address(),
            ALICE.public().to_address(),
            BalanceType::HOPR.balance(10),
            0.into(),
            ChannelStatus::PendingToClose(SystemTime::now().add(Duration::from_secs(100))),
            4.into(),
        );

        db.upsert_channel(None, channel).await?;

        let db_clone = db.clone();
        let ack_ticket_1 = generate_random_ack_ticket(1, 1, 3)?;
        let ack_ticket_2 = generate_random_ack_ticket(2, 1, 5)?;

        let ack_ticket_2_clone = ack_ticket_2.clone();

        db.begin_transaction_in_db(TargetDb::Tickets)
            .await?
            .perform(move |tx| {
                Box::pin(async move {
                    db_clone.upsert_ticket(Some(tx), ack_ticket_1).await?;
                    db_clone.upsert_ticket(Some(tx), ack_ticket_2).await
                })
            })
            .await?;

        let mut actions = MockTicketRedeemAct::new();
        let mock_confirm = mock_action_confirmation(ack_ticket_2_clone.clone())?;
        actions
            .expect_redeem_ticket()
            .once()
            .withf(move |ack| ack_ticket_2_clone.ticket.eq(&ack.ticket))
            .return_once(move |_| Ok(ok(mock_confirm).boxed()));

        let cfg = AutoRedeemingStrategyConfig {
            minimum_redeem_ticket_value: BalanceType::HOPR.balance(*PRICE_PER_PACKET * 5),
            redeem_only_aggregated: false,
            redeem_all_on_close: true,
        };

        let ars = AutoRedeemingStrategy::new(cfg, db, actions);
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
}
