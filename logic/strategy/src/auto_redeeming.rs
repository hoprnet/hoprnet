//! ## Auto Redeeming Strategy
//! This strategy listens for newly added acknowledged tickets and automatically issues a redeem transaction on that ticket.
//! It can be configured to automatically redeem all tickets or only aggregated tickets (which results in far less on-chain transactions being issued).
//!
//! For details on default parameters see [AutoRedeemingStrategyConfig].
use async_trait::async_trait;
use chain_actions::redeem::TicketRedeemActions;
use hopr_db_sql::tickets::HoprDbTicketOperations;
use hopr_internal_types::prelude::*;
use hopr_internal_types::tickets::{AcknowledgedTicket, AcknowledgedTicketStatus};
use hopr_primitive_types::prelude::*;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use std::fmt::{Debug, Display, Formatter};
use tracing::{debug, info};
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

/// Configuration object for the `AutoRedeemingStrategy`
#[serde_as]
#[derive(Debug, Clone, Copy, PartialEq, Eq, smart_default::SmartDefault, Validate, Serialize, Deserialize)]
pub struct AutoRedeemingStrategyConfig {
    /// If set, the strategy will redeem only aggregated tickets.
    #[default = true]
    pub redeem_only_aggregated: bool,

    /// The strategy will automatically redeem if there's a single ticket left when a channel
    /// transitions to `PendingToClose` and it has at least this value of HOPR.
    /// This happens regardless the `redeem_only_aggregated` setting.
    ///
    /// Default is 2 HOPR.
    #[serde_as(as = "DisplayFromStr")]
    #[default(Balance::new_from_str("2000000000000000000", BalanceType::HOPR))]
    pub on_close_redeem_single_tickets_value_min: Balance,
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
        if !self.cfg.redeem_only_aggregated || ack.verified_ticket().is_aggregated() {
            info!("redeeming {ack}");

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
        if direction != ChannelDirection::Incoming {
            return Ok(());
        }

        if let ChannelChange::Status { left: old, right: new } = change {
            if old != ChannelStatus::Open || !matches!(new, ChannelStatus::PendingToClose(_)) {
                debug!("ignoring channel {channel} state change that's not in PendingToClose");
                return Ok(());
            }
            info!("checking to redeem a singular ticket in {channel} because it's now PendingToClose");

            let mut ack_ticket_in_db = self
                .db
                .get_tickets(channel.into())
                .await?
                .into_iter()
                .filter(|t| {
                    t.status == AcknowledgedTicketStatus::Untouched
                        && t.verified_ticket().amount >= self.cfg.on_close_redeem_single_tickets_value_min
                })
                .collect::<Vec<_>>();

            if ack_ticket_in_db.len() == 1 {
                let ack = ack_ticket_in_db.pop().unwrap();
                info!("redeeming single {ack} worth {}", ack.verified_ticket().amount);

                #[cfg(all(feature = "prometheus", not(test)))]
                METRIC_COUNT_AUTO_REDEEMS.increment();

                let rx = self.chain_actions.redeem_ticket(ack.clone()).await?;
                std::mem::drop(rx); // The Receiver is not intentionally awaited here and the oneshot Sender can fail safely
                Ok(())
            } else {
                debug!(
                    "not auto-redeeming single ticket in {channel}: there are {} redeemable tickets worth >= {}",
                    ack_ticket_in_db.len(),
                    self.cfg.on_close_redeem_single_tickets_value_min
                );
                Err(CriteriaNotSatisfied)
            }
        } else {
            Err(CriteriaNotSatisfied)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::auto_redeeming::{AutoRedeemingStrategy, AutoRedeemingStrategyConfig};
    use crate::strategy::SingularStrategy;
    use async_trait::async_trait;
    use chain_actions::action_queue::{ActionConfirmation, PendingAction};
    use chain_actions::redeem::TicketRedeemActions;
    use chain_types::actions::Action;
    use chain_types::chain_events::ChainEventType;
    use futures::{future::ok, FutureExt};
    use hex_literal::hex;
    use hopr_crypto_random::random_bytes;
    use hopr_crypto_types::prelude::*;
    use hopr_db_sql::channels::HoprDbChannelOperations;
    use hopr_db_sql::db::HoprDb;
    use hopr_db_sql::info::{DomainSeparator, HoprDbInfoOperations};
    use hopr_db_sql::{HoprDbGeneralModelOperations, TargetDb};
    use hopr_internal_types::prelude::*;
    use hopr_primitive_types::prelude::*;
    use mockall::mock;
    use std::ops::Add;
    use std::time::{Duration, SystemTime};

    lazy_static::lazy_static! {
        static ref ALICE: ChainKeypair = ChainKeypair::from_secret(&hex!("492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775")).unwrap();
        static ref BOB: ChainKeypair = ChainKeypair::from_secret(&hex!("48680484c6fc31bc881a0083e6e32b6dc789f9eaba0f8b981429fd346c697f8c")).unwrap();
        static ref PRICE_PER_PACKET: U256 = 10000000000000000_u128.into(); // 0.01 HOPR
    }

    fn generate_random_ack_ticket(index: u64, idx_offset: u32, worth_packets: u32) -> AcknowledgedTicket {
        let hk1 = HalfKey::random();
        let hk2 = HalfKey::random();

        let cp1: CurvePoint = hk1.to_challenge().try_into().unwrap();
        let cp2: CurvePoint = hk2.to_challenge().try_into().unwrap();
        let cp_sum = CurvePoint::combine(&[&cp1, &cp2]);

        TicketBuilder::default()
            .addresses(&*BOB, &*ALICE)
            .amount(PRICE_PER_PACKET.div_f64(1.0f64).unwrap() * worth_packets)
            .index(index)
            .index_offset(idx_offset)
            .win_prob(1.0)
            .channel_epoch(4)
            .challenge(Challenge::from(cp_sum).into())
            .build_signed(&BOB, &Hash::default())
            .unwrap()
            .into_acknowledged(Response::from_half_keys(&hk1, &hk2).unwrap())
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
            ) -> chain_actions::errors::Result<Vec<PendingAction >>;
            async fn redeem_tickets_in_channel(
                &self,
                channel: &ChannelEntry,
                only_aggregated: bool,
            ) -> chain_actions::errors::Result<Vec<PendingAction >>;
            async fn redeem_ticket(&self, ack: AcknowledgedTicket) -> chain_actions::errors::Result<PendingAction>;
        }
    }

    fn mock_action_confirmation(ack: AcknowledgedTicket) -> ActionConfirmation {
        let random_hash = Hash::from(random_bytes::<{ Hash::SIZE }>());
        ActionConfirmation {
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
            action: Action::RedeemTicket(ack.into_redeemable(&ALICE, &Hash::default()).unwrap()),
        }
    }

    #[async_std::test]
    async fn test_auto_redeeming_strategy_redeem() {
        let db = HoprDb::new_in_memory(ALICE.clone()).await;
        db.set_domain_separator(None, DomainSeparator::Channel, Default::default())
            .await
            .unwrap();

        let ack_ticket = generate_random_ack_ticket(0, 1, 5);
        let ack_clone = ack_ticket.clone();
        let ack_clone_2 = ack_ticket.clone();

        let mut actions = MockTicketRedeemAct::new();
        actions
            .expect_redeem_ticket()
            .once()
            .withf(move |ack| ack_clone.ticket.eq(&ack.ticket))
            .return_once(|_| Ok(ok(mock_action_confirmation(ack_clone_2)).boxed()));

        let cfg = AutoRedeemingStrategyConfig {
            redeem_only_aggregated: false,
            ..Default::default()
        };

        let ars = AutoRedeemingStrategy::new(cfg, db, actions);
        ars.on_acknowledged_winning_ticket(&ack_ticket).await.unwrap();
    }

    #[async_std::test]
    async fn test_auto_redeeming_strategy_redeem_agg_only() {
        let db = HoprDb::new_in_memory(ALICE.clone()).await;
        db.set_domain_separator(None, DomainSeparator::Channel, Default::default())
            .await
            .unwrap();

        let ack_ticket_unagg = generate_random_ack_ticket(0, 1, 5);
        let ack_ticket_agg = generate_random_ack_ticket(0, 3, 5);

        let ack_clone_agg = ack_ticket_agg.clone();
        let ack_clone_agg_2 = ack_ticket_agg.clone();
        let mut actions = MockTicketRedeemAct::new();
        actions
            .expect_redeem_ticket()
            .once()
            .withf(move |ack| ack_clone_agg.ticket.eq(&ack.ticket))
            .return_once(|_| Ok(ok(mock_action_confirmation(ack_clone_agg_2)).boxed()));

        let cfg = AutoRedeemingStrategyConfig {
            redeem_only_aggregated: true,
            ..Default::default()
        };

        let ars = AutoRedeemingStrategy::new(cfg, db, actions);
        ars.on_acknowledged_winning_ticket(&ack_ticket_unagg)
            .await
            .expect_err("non-agg ticket should not satisfy");
        ars.on_acknowledged_winning_ticket(&ack_ticket_agg)
            .await
            .expect("agg ticket should satisfy");
    }

    #[async_std::test]
    async fn test_auto_redeeming_strategy_should_redeem_singular_ticket_on_close() {
        let db = HoprDb::new_in_memory(ALICE.clone()).await;
        db.set_domain_separator(None, DomainSeparator::Channel, Default::default())
            .await
            .unwrap();

        let channel = ChannelEntry::new(
            BOB.public().to_address(),
            ALICE.public().to_address(),
            BalanceType::HOPR.balance(10),
            0.into(),
            ChannelStatus::PendingToClose(SystemTime::now().add(Duration::from_secs(100))),
            4.into(),
        );

        // Make ticket worth exactly the threshold
        let ack_ticket = generate_random_ack_ticket(0, 1, 5);

        db.upsert_channel(None, channel).await.unwrap();
        db.upsert_ticket(None, ack_ticket.clone()).await.unwrap();

        let ack_clone = ack_ticket.clone();
        let ack_clone_2 = ack_ticket.clone();

        let mut actions = MockTicketRedeemAct::new();
        actions
            .expect_redeem_ticket()
            .once()
            .withf(move |ack| ack_clone.ticket.eq(&ack.ticket))
            .return_once(|_| Ok(ok(mock_action_confirmation(ack_clone_2)).boxed()));

        let cfg = AutoRedeemingStrategyConfig {
            redeem_only_aggregated: true,
            on_close_redeem_single_tickets_value_min: BalanceType::HOPR.balance(*PRICE_PER_PACKET * 5),
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
        .expect("event should be satisfied");
    }

    #[async_std::test]
    async fn test_auto_redeeming_strategy_should_not_redeem_singular_ticket_worth_less_on_close() {
        let db = HoprDb::new_in_memory(ALICE.clone()).await;
        db.set_domain_separator(None, DomainSeparator::Channel, Default::default())
            .await
            .unwrap();

        let channel = ChannelEntry::new(
            BOB.public().to_address(),
            ALICE.public().to_address(),
            BalanceType::HOPR.balance(10),
            0.into(),
            ChannelStatus::PendingToClose(SystemTime::now().add(Duration::from_secs(100))),
            0.into(),
        );

        // Make this ticket worth less than the threshold
        let ack_ticket = generate_random_ack_ticket(0, 1, 3);

        db.upsert_channel(None, channel).await.unwrap();
        db.upsert_ticket(None, ack_ticket).await.unwrap();

        let mut actions = MockTicketRedeemAct::new();
        actions.expect_redeem_ticket().never();

        let cfg = AutoRedeemingStrategyConfig {
            redeem_only_aggregated: false,
            on_close_redeem_single_tickets_value_min: BalanceType::HOPR.balance(*PRICE_PER_PACKET * 5),
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
    }

    #[async_std::test]
    async fn test_auto_redeeming_strategy_should_not_redeem_non_singular_tickets_on_close() {
        let db = HoprDb::new_in_memory(ALICE.clone()).await;
        db.set_domain_separator(None, DomainSeparator::Channel, Default::default())
            .await
            .unwrap();

        let channel = ChannelEntry::new(
            BOB.public().to_address(),
            ALICE.public().to_address(),
            BalanceType::HOPR.balance(10),
            0.into(),
            ChannelStatus::PendingToClose(SystemTime::now().add(Duration::from_secs(100))),
            0.into(),
        );

        db.upsert_channel(None, channel).await.unwrap();

        let db_clone = db.clone();
        db.begin_transaction_in_db(TargetDb::Tickets)
            .await
            .unwrap()
            .perform(|tx| {
                Box::pin(async move {
                    let ack_ticket = generate_random_ack_ticket(1, 1, 5);
                    db_clone.upsert_ticket(Some(tx), ack_ticket).await?;

                    let ack_ticket = generate_random_ack_ticket(2, 1, 5);
                    db_clone.upsert_ticket(Some(tx), ack_ticket).await
                })
            })
            .await
            .unwrap();

        let mut actions = MockTicketRedeemAct::new();
        actions.expect_redeem_ticket().never();

        let cfg = AutoRedeemingStrategyConfig {
            redeem_only_aggregated: false,
            on_close_redeem_single_tickets_value_min: BalanceType::HOPR.balance(*PRICE_PER_PACKET * 5),
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
    }
}
