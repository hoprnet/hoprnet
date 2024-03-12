//! ## Auto Redeeming Strategy
//! This strategy listens for newly added acknowledged tickets and automatically issues a redeem transaction on that ticket.
//! It can be configured to automatically redeem all tickets or only aggregated tickets (which results in far less on-chain transactions being issued).
//!
//! For details on default parameters see [AutoRedeemingStrategyConfig].
use async_lock::RwLock;
use async_trait::async_trait;
use chain_actions::redeem::TicketRedeemActions;
use hopr_internal_types::acknowledgement::{AcknowledgedTicket, AcknowledgedTicketStatus};
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::*;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use std::fmt::{Debug, Display, Formatter};
use std::sync::Arc;
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
pub struct AutoRedeemingStrategy<A: TicketRedeemActions, Db: HoprCoreEthereumDbActions> {
    chain_actions: A,
    db: Arc<RwLock<Db>>,
    cfg: AutoRedeemingStrategyConfig,
}

impl<A: TicketRedeemActions, Db: HoprCoreEthereumDbActions> Debug for AutoRedeemingStrategy<A, Db> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", Strategy::AutoRedeeming(self.cfg))
    }
}

impl<A: TicketRedeemActions, Db: HoprCoreEthereumDbActions> Display for AutoRedeemingStrategy<A, Db> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Strategy::AutoRedeeming(self.cfg))
    }
}

impl<A: TicketRedeemActions, Db: HoprCoreEthereumDbActions> AutoRedeemingStrategy<A, Db> {
    pub fn new(cfg: AutoRedeemingStrategyConfig, db: Arc<RwLock<Db>>, chain_actions: A) -> Self {
        Self { cfg, db, chain_actions }
    }
}

#[async_trait]
impl<A, Db> SingularStrategy for AutoRedeemingStrategy<A, Db>
where
    A: TicketRedeemActions + Send + Sync,
    Db: HoprCoreEthereumDbActions + Send + Sync,
{
    async fn on_acknowledged_winning_ticket(&self, ack: &AcknowledgedTicket) -> crate::errors::Result<()> {
        if !self.cfg.redeem_only_aggregated || ack.ticket.is_aggregated() {
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
                .read()
                .await
                .get_acknowledged_tickets(Some(*channel))
                .await?
                .into_iter()
                .filter(|t| {
                    t.status == AcknowledgedTicketStatus::Untouched
                        && t.ticket.amount >= self.cfg.on_close_redeem_single_tickets_value_min
                })
                .collect::<Vec<_>>();

            if ack_ticket_in_db.len() == 1 {
                let ack = ack_ticket_in_db.pop().unwrap();
                info!("redeeming single {ack} worth {}", ack.ticket.amount);

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
    use async_lock::RwLock;
    use async_trait::async_trait;
    use chain_actions::action_queue::{ActionConfirmation, PendingAction};
    use chain_actions::redeem::TicketRedeemActions;
    use chain_db::db::CoreEthereumDb;
    use chain_db::traits::HoprCoreEthereumDbActions;
    use chain_types::actions::Action;
    use chain_types::chain_events::ChainEventType;
    use futures::{future::ok, FutureExt};
    use hex_literal::hex;
    use hopr_crypto_random::random_bytes;
    use hopr_crypto_types::prelude::*;
    use hopr_internal_types::prelude::*;
    use hopr_primitive_types::prelude::*;
    use mockall::mock;
    use std::ops::Add;
    use std::sync::Arc;
    use std::time::{Duration, SystemTime};
    use utils_db::constants::ACKNOWLEDGED_TICKETS_PREFIX;
    use utils_db::db::DB;
    use utils_db::CurrentDbShim;

    lazy_static::lazy_static! {
        static ref ALICE: ChainKeypair = ChainKeypair::from_secret(&hex!("492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775")).unwrap();
        static ref BOB: ChainKeypair = ChainKeypair::from_secret(&hex!("48680484c6fc31bc881a0083e6e32b6dc789f9eaba0f8b981429fd346c697f8c")).unwrap();
        static ref PRICE_PER_PACKET: U256 = 10000000000000000_u128.into(); // 0.01 HOPR
    }

    fn generate_random_ack_ticket(idx_offset: u32, worth_packets: u32) -> AcknowledgedTicket {
        let counterparty = &BOB;
        let hk1 = HalfKey::random();
        let hk2 = HalfKey::random();

        let cp1: CurvePoint = hk1.to_challenge().into();
        let cp2: CurvePoint = hk2.to_challenge().into();
        let cp_sum = CurvePoint::combine(&[&cp1, &cp2]);

        let ticket = Ticket::new(
            &ALICE.public().to_address(),
            &Balance::new(
                PRICE_PER_PACKET.div_f64(1.0f64).unwrap() * worth_packets,
                BalanceType::HOPR,
            ),
            0_u32.into(),
            idx_offset.into(),
            1.0f64,
            4u64.into(),
            Challenge::from(cp_sum).to_ethereum_challenge(),
            counterparty,
            &Hash::default(),
        )
        .unwrap();

        let unacked_ticket = UnacknowledgedTicket::new(ticket, hk1, counterparty.public().to_address());
        unacked_ticket.acknowledge(&hk2, &ALICE, &Hash::default()).unwrap()
    }

    fn ack_ticket_key(ack_ticket: &AcknowledgedTicket) -> utils_db::db::Key {
        let mut ack_key = Vec::new();

        ack_key.extend_from_slice(&ack_ticket.ticket.channel_id.to_bytes());
        ack_key.extend_from_slice(&ack_ticket.ticket.channel_epoch.to_be_bytes());
        ack_key.extend_from_slice(&ack_ticket.ticket.index.to_be_bytes());

        utils_db::db::Key::new_bytes_with_prefix(&ack_key, ACKNOWLEDGED_TICKETS_PREFIX).unwrap()
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
        let random_hash = Hash::new(&random_bytes::<{ Hash::SIZE }>());
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
            action: Action::RedeemTicket(ack.clone()),
        }
    }

    #[async_std::test]
    async fn test_auto_redeeming_strategy_redeem() {
        let db = Arc::new(RwLock::new(CoreEthereumDb::new(
            DB::new(CurrentDbShim::new_in_memory().await),
            ALICE.public().to_address(),
        )));

        let ack_ticket = generate_random_ack_ticket(1, 5);
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
        let db = Arc::new(RwLock::new(CoreEthereumDb::new(
            DB::new(CurrentDbShim::new_in_memory().await),
            ALICE.public().to_address(),
        )));

        let ack_ticket_unagg = generate_random_ack_ticket(1, 5);
        let ack_ticket_agg = generate_random_ack_ticket(3, 5);

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
        let db = Arc::new(RwLock::new(CoreEthereumDb::new(
            DB::new(CurrentDbShim::new_in_memory().await),
            ALICE.public().to_address(),
        )));

        let channel = ChannelEntry::new(
            BOB.public().to_address(),
            ALICE.public().to_address(),
            BalanceType::HOPR.balance(10),
            0.into(),
            ChannelStatus::PendingToClose(SystemTime::now().add(Duration::from_secs(100))),
            0.into(),
        );
        db.write()
            .await
            .update_channel_and_snapshot(&channel.get_id(), &channel, &Default::default())
            .await
            .unwrap();

        // Make ticket worth exactly the threshold
        let ack_ticket = generate_random_ack_ticket(1, 5);
        db.write()
            .await
            .db
            .set(ack_ticket_key(&ack_ticket), &ack_ticket)
            .await
            .unwrap();

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
        let db = Arc::new(RwLock::new(CoreEthereumDb::new(
            DB::new(CurrentDbShim::new_in_memory().await),
            ALICE.public().to_address(),
        )));

        let channel = ChannelEntry::new(
            BOB.public().to_address(),
            ALICE.public().to_address(),
            BalanceType::HOPR.balance(10),
            0.into(),
            ChannelStatus::PendingToClose(SystemTime::now().add(Duration::from_secs(100))),
            0.into(),
        );
        db.write()
            .await
            .update_channel_and_snapshot(&channel.get_id(), &channel, &Default::default())
            .await
            .unwrap();

        // Make this ticket worth less than the threshold
        let ack_ticket = generate_random_ack_ticket(1, 3);
        db.write()
            .await
            .db
            .set(ack_ticket_key(&ack_ticket), &ack_ticket)
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

    #[async_std::test]
    async fn test_auto_redeeming_strategy_should_not_redeem_non_singular_tickets_on_close() {
        let db = Arc::new(RwLock::new(CoreEthereumDb::new(
            DB::new(CurrentDbShim::new_in_memory().await),
            ALICE.public().to_address(),
        )));

        let channel = ChannelEntry::new(
            BOB.public().to_address(),
            ALICE.public().to_address(),
            BalanceType::HOPR.balance(10),
            0.into(),
            ChannelStatus::PendingToClose(SystemTime::now().add(Duration::from_secs(100))),
            0.into(),
        );
        db.write()
            .await
            .update_channel_and_snapshot(&channel.get_id(), &channel, &Default::default())
            .await
            .unwrap();

        // Make this ticket worth exactly the threshold
        let mut ack_ticket = generate_random_ack_ticket(1, 5);
        ack_ticket.ticket.index = 1;
        db.write()
            .await
            .db
            .set(ack_ticket_key(&ack_ticket), &ack_ticket)
            .await
            .unwrap();

        // Make one more ticket worth exactly the threshold
        let mut ack_ticket = generate_random_ack_ticket(1, 5);
        ack_ticket.ticket.index = 2;
        db.write()
            .await
            .db
            .set(ack_ticket_key(&ack_ticket), &ack_ticket)
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
