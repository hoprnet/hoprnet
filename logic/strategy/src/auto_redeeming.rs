//! ## Auto Redeeming Strategy
//! This strategy listens for newly added acknowledged tickets and automatically issues a redeem transaction on that ticket.
//! It can be configured to automatically redeem all tickets or only aggregated tickets (which results in far less on-chain transactions being issued).
//!
//! For details on default parameters see [AutoRedeemingStrategyConfig].
use async_trait::async_trait;
use chain_actions::redeem::TicketRedeemActions;
use hopr_internal_types::acknowledgement::AcknowledgedTicket;
use tracing::info;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display, Formatter};
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, smart_default::SmartDefault, Validate, Serialize, Deserialize)]
pub struct AutoRedeemingStrategyConfig {
    /// If set, the strategy will redeem only aggregated tickets.
    #[default = true]
    pub redeem_only_aggregated: bool,
}

/// The `AutoRedeemingStrategy` automatically sends an acknowledged ticket
/// for redemption once encountered.
/// The strategy does not await the result of the redemption.
pub struct AutoRedeemingStrategy<A: TicketRedeemActions> {
    chain_actions: A,
    cfg: AutoRedeemingStrategyConfig,
}

impl<A: TicketRedeemActions> Debug for AutoRedeemingStrategy<A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", Strategy::AutoRedeeming(self.cfg))
    }
}

impl<A: TicketRedeemActions> Display for AutoRedeemingStrategy<A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Strategy::AutoRedeeming(self.cfg))
    }
}

impl<A: TicketRedeemActions> AutoRedeemingStrategy<A> {
    pub fn new(cfg: AutoRedeemingStrategyConfig, chain_actions: A) -> Self {
        Self { cfg, chain_actions }
    }
}

#[async_trait]
impl<A: TicketRedeemActions + Send + Sync> SingularStrategy for AutoRedeemingStrategy<A> {
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
    use hopr_internal_types::prelude::*;
    use hopr_primitive_types::prelude::*;
    use mockall::mock;

    lazy_static::lazy_static! {
        static ref ALICE: ChainKeypair = ChainKeypair::from_secret(&hex!("492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775")).unwrap();
        static ref BOB: ChainKeypair = ChainKeypair::from_secret(&hex!("48680484c6fc31bc881a0083e6e32b6dc789f9eaba0f8b981429fd346c697f8c")).unwrap();
    }

    fn generate_random_ack_ticket(idx_offset: u32) -> AcknowledgedTicket {
        let counterparty = &BOB;
        let hk1 = HalfKey::random();
        let hk2 = HalfKey::random();

        let cp1: CurvePoint = hk1.to_challenge().into();
        let cp2: CurvePoint = hk2.to_challenge().into();
        let cp_sum = CurvePoint::combine(&[&cp1, &cp2]);

        let price_per_packet: U256 = 10000000000000000u128.into(); // 0.01 HOPR

        let ticket = Ticket::new(
            &ALICE.public().to_address(),
            &Balance::new(price_per_packet.div_f64(1.0f64).unwrap() * 5u32, BalanceType::HOPR),
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
        let ack_ticket = generate_random_ack_ticket(1);
        let ack_clone = ack_ticket.clone();
        let ack_clone_2 = ack_ticket.clone();

        let mut actions = MockTicketRedeemAct::new();
        actions
            .expect_redeem_ticket()
            .times(1)
            .withf(move |ack| ack_clone.ticket.eq(&ack.ticket))
            .return_once(|_| Ok(ok(mock_action_confirmation(ack_clone_2)).boxed()));

        let cfg = AutoRedeemingStrategyConfig {
            redeem_only_aggregated: false,
        };

        let ars = AutoRedeemingStrategy::new(cfg, actions);
        ars.on_acknowledged_winning_ticket(&ack_ticket).await.unwrap();
    }

    #[async_std::test]
    async fn test_auto_redeeming_strategy_redeem_agg_only() {
        let ack_ticket_unagg = generate_random_ack_ticket(1);
        let ack_ticket_agg = generate_random_ack_ticket(3);

        let ack_clone_agg = ack_ticket_agg.clone();
        let ack_clone_agg_2 = ack_ticket_agg.clone();
        let mut actions = MockTicketRedeemAct::new();
        actions
            .expect_redeem_ticket()
            .times(1)
            .withf(move |ack| ack_clone_agg.ticket.eq(&ack.ticket))
            .return_once(|_| Ok(ok(mock_action_confirmation(ack_clone_agg_2)).boxed()));

        let cfg = AutoRedeemingStrategyConfig {
            redeem_only_aggregated: true,
        };

        let ars = AutoRedeemingStrategy::new(cfg, actions);
        ars.on_acknowledged_winning_ticket(&ack_ticket_unagg)
            .await
            .expect_err("non-agg ticket should not satisfy");
        ars.on_acknowledged_winning_ticket(&ack_ticket_agg)
            .await
            .expect("agg ticket should satisfy");
    }
}
