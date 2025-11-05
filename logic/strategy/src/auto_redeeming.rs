//! ## Auto Redeeming Strategy
//! This strategy listens for newly added acknowledged tickets and automatically issues a redeem transaction on that
//! ticket.
//!
//! For details on default parameters, see [AutoRedeemingStrategyConfig].
use std::{
    fmt::{Debug, Display, Formatter},
    str::FromStr,
};

use async_trait::async_trait;
use futures::StreamExt;
use hopr_api::{
    chain::{
        ChainReadChannelOperations, ChainValues, ChainWriteTicketOperations, ChannelSelector,
        redeem_tickets_via_selector,
    },
    db::{HoprDbTicketOperations, TicketSelector},
};
use hopr_internal_types::{prelude::*, tickets::AcknowledgedTicket};
use hopr_primitive_types::prelude::*;
use serde::{Deserialize, Serialize};
use serde_with::{DisplayFromStr, serde_as};
use tracing::{debug, error, info};
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
pub struct AutoRedeemingStrategy<A, Db> {
    hopr_chain_actions: A,
    node_db: Db,
    cfg: AutoRedeemingStrategyConfig,
}

/// Number of concurrent channel redemption tasks.
const REDEEMING_CONCURRENCY: usize = 20;

impl<A, Db> Debug for AutoRedeemingStrategy<A, Db> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", Strategy::AutoRedeeming(self.cfg))
    }
}

impl<A, Db> Display for AutoRedeemingStrategy<A, Db> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Strategy::AutoRedeeming(self.cfg))
    }
}

impl<A, Db> AutoRedeemingStrategy<A, Db> {
    pub fn new(cfg: AutoRedeemingStrategyConfig, hopr_chain_actions: A, node_db: Db) -> Self {
        Self {
            cfg,
            hopr_chain_actions,
            node_db,
        }
    }
}

#[async_trait]
impl<A, Db> SingularStrategy for AutoRedeemingStrategy<A, Db>
where
    A: ChainWriteTicketOperations + ChainReadChannelOperations + ChainValues + Clone + Send + Sync,
    Db: HoprDbTicketOperations + Clone + Send + Sync,
{
    async fn on_tick(&self) -> crate::errors::Result<()> {
        if !self.cfg.redeem_on_winning {
            debug!("trying to redeem all tickets in all channels");

            let chain_actions = self.hopr_chain_actions.clone();
            let node_db = self.node_db.clone();
            self.hopr_chain_actions
                .stream_channels(
                    ChannelSelector::default()
                        .with_destination(*self.hopr_chain_actions.me())
                        .with_allowed_states(&[
                            ChannelStatusDiscriminants::Open,
                            ChannelStatusDiscriminants::PendingToClose,
                        ])
                        .with_closure_time_range(Utc::now()..),
                )
                .await
                .map_err(|e| StrategyError::Other(e.into()))?
                .for_each_concurrent(REDEEMING_CONCURRENCY, |channel| {
                    let chain_actions_clone = chain_actions.clone();
                    let node_db = node_db.clone();
                    async move {
                        let selector = TicketSelector::from(&channel)
                            .with_amount(self.cfg.minimum_redeem_ticket_value..)
                            .with_index_range(channel.ticket_index.as_u64()..)
                            .with_state(AcknowledgedTicketStatus::Untouched);

                        match redeem_tickets_via_selector(selector, &node_db, &chain_actions_clone)
                            .await
                            .map_err(|e| StrategyError::Other(e.into()))
                        {
                            Ok((count, _)) => {
                                // We don't await the result of the redemption here
                                if count > 0 {
                                    #[cfg(all(feature = "prometheus", not(test)))]
                                    METRIC_COUNT_AUTO_REDEEMS.increment_by(count as u64);

                                    info!(count, %channel, "strategy issued ticket redemptions");
                                } else {
                                    debug!(count, %channel, "strategy issued no ticket redemptions");
                                }
                            }
                            Err(error) => error!(%error, %channel, "strategy failed to issue ticket redemptions"),
                        }
                    }
                })
                .await;

            Ok(())
        } else {
            Err(CriteriaNotSatisfied)
        }
    }

    async fn on_acknowledged_winning_ticket(&self, ack: &AcknowledgedTicket) -> crate::errors::Result<()> {
        if self.cfg.redeem_on_winning && ack.verified_ticket().amount.ge(&self.cfg.minimum_redeem_ticket_value) {
            if let Some(channel) = self
                .hopr_chain_actions
                .channel_by_id(&ack.verified_ticket().channel_id)
                .await
                .map_err(|e| StrategyError::Other(e.into()))?
            {
                info!(%ack, "redeeming");

                if ack.ticket.verified_ticket().index < channel.ticket_index.as_u64() {
                    error!(%ack, "acknowledged ticket is older than channel ticket index");
                    return Err(CriteriaNotSatisfied);
                }

                #[cfg(all(feature = "prometheus", not(test)))]
                METRIC_COUNT_AUTO_REDEEMS.increment();

                let _ = redeem_tickets_via_selector(
                    TicketSelector::from(channel)
                        .with_amount(self.cfg.minimum_redeem_ticket_value..)
                        .with_index_range(channel.ticket_index.as_u64()..=ack.ticket.verified_ticket().index)
                        .with_state(AcknowledgedTicketStatus::Untouched),
                    &self.node_db,
                    &self.hopr_chain_actions,
                )
                .await
                .map_err(|e| StrategyError::Other(e.into()))?;

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

            let (count, _) = redeem_tickets_via_selector(
                TicketSelector::from(channel)
                    .with_amount(self.cfg.minimum_redeem_ticket_value..)
                    .with_index_range(channel.ticket_index.as_u64()..)
                    .with_state(AcknowledgedTicketStatus::Untouched),
                &self.node_db,
                &self.hopr_chain_actions,
            )
            .await
            .map_err(|e| StrategyError::Other(e.into()))?;

            #[cfg(all(feature = "prometheus", not(test)))]
            METRIC_COUNT_AUTO_REDEEMS.increment();

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

    use hex_literal::hex;
    use hopr_api::chain::ChainReceipt;
    use hopr_crypto_random::Randomizable;
    use hopr_crypto_types::prelude::*;

    use super::*;
    use crate::tests::{MockChainActions, MockTestActions};

    lazy_static::lazy_static! {
        static ref ALICE: ChainKeypair = ChainKeypair::from_secret(&hex!("492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775")).expect("lazy static keypair should be valid");
        static ref BOB: ChainKeypair = ChainKeypair::from_secret(&hex!("48680484c6fc31bc881a0083e6e32b6dc789f9eaba0f8b981429fd346c697f8c")).expect("lazy static keypair should be valid");
        static ref CHARLIE: ChainKeypair = ChainKeypair::from_secret(&hex!("d39a926980d6fa96a9eba8f8058b2beb774bc11866a386e9ddf9dc1152557c26")).expect("lazy static keypair should be constructible");
        static ref PRICE_PER_PACKET: HoprBalance = 10000000000000000_u128.into(); // 0.01 HOPR

        static ref CHANNEL_1: ChannelEntry = ChannelEntry::new(
            BOB.public().to_address(),
            ALICE.public().to_address(),
            *PRICE_PER_PACKET * 10,
            0.into(),
            ChannelStatus::Open,
            4.into()
        );
        static ref CHANNEL_2: ChannelEntry = ChannelEntry::new(
            BOB.public().to_address(),
            CHARLIE.public().to_address(),
            *PRICE_PER_PACKET * 11,
            1.into(),
            ChannelStatus::Open,
            4.into()
        );
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
            .amount(PRICE_PER_PACKET.div_f64(1.0f64)?.amount() * worth_packets)
            .index(index)
            .win_prob(WinningProbability::ALWAYS)
            .channel_epoch(4)
            .challenge(challenge)
            .build_signed(&BOB, &Hash::default())?
            .into_acknowledged(Response::from_half_keys(&hk1, &hk2)?))
    }

    #[tokio::test]
    async fn test_auto_redeeming_strategy_redeem() -> anyhow::Result<()> {
        let ack_ticket = generate_random_ack_ticket(0, 1, 5)?;

        let mut mock = MockTestActions::new();
        mock.expect_channel_by_id()
            .once()
            .with(mockall::predicate::eq(CHANNEL_1.get_id()))
            .return_once(|_| Some(CHANNEL_1.clone()));

        mock.expect_redeem_with_selector()
            .once()
            .with(mockall::predicate::eq(
                TicketSelector::from(CHANNEL_1.clone())
                    .with_amount(HoprBalance::zero()..)
                    .with_index_range(
                        ack_ticket.ticket.verified_ticket().index..=ack_ticket.ticket.verified_ticket().index,
                    )
                    .with_state(AcknowledgedTicketStatus::Untouched),
            ))
            .return_once(|_| vec![ChainReceipt::default()]);

        let cfg = AutoRedeemingStrategyConfig {
            minimum_redeem_ticket_value: 0.into(),
            redeem_on_winning: true,
            ..Default::default()
        };

        let ars = AutoRedeemingStrategy::new(cfg, MockChainActions(mock.into()));
        ars.on_acknowledged_winning_ticket(&ack_ticket).await?;
        assert!(ars.on_tick().await.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn test_auto_redeeming_strategy_redeem_on_tick() -> anyhow::Result<()> {
        let ack_ticket = generate_random_ack_ticket(0, 1, 5)?;

        let mut mock = MockTestActions::new();
        mock.expect_me().return_const(BOB.public().to_address());

        mock.expect_stream_channels()
            .once()
            .withf(move |selector| {
                selector.source.is_none()
                    && selector.destination == Some(BOB.public().to_address())
                    && selector.allowed_states
                        == &[
                            ChannelStatusDiscriminants::Open,
                            ChannelStatusDiscriminants::PendingToClose,
                        ]
            })
            .return_once(|_| futures::stream::iter([CHANNEL_1.clone(), CHANNEL_2.clone()]).boxed());

        mock.expect_redeem_with_selector()
            .once()
            .with(mockall::predicate::eq(
                TicketSelector::from(CHANNEL_1.clone())
                    .with_amount(HoprBalance::from(*PRICE_PER_PACKET * 5)..)
                    .with_index_range(CHANNEL_1.ticket_index.as_u64()..)
                    .with_state(AcknowledgedTicketStatus::Untouched),
            ))
            .returning(|_| vec![ChainReceipt::default()]);

        mock.expect_redeem_with_selector()
            .once()
            .with(mockall::predicate::eq(
                TicketSelector::from(CHANNEL_2.clone())
                    .with_amount(HoprBalance::from(*PRICE_PER_PACKET * 5)..)
                    .with_index_range(CHANNEL_2.ticket_index.as_u64()..)
                    .with_state(AcknowledgedTicketStatus::Untouched),
            ))
            .returning(|_| vec![ChainReceipt::default()]);

        let cfg = AutoRedeemingStrategyConfig {
            minimum_redeem_ticket_value: HoprBalance::from(*PRICE_PER_PACKET * 5),
            redeem_on_winning: false,
            ..Default::default()
        };

        let ars = AutoRedeemingStrategy::new(cfg, MockChainActions(mock.into()));
        ars.on_tick().await?;
        assert!(ars.on_acknowledged_winning_ticket(&ack_ticket).await.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn test_auto_redeeming_strategy_redeem_minimum_ticket_amount() -> anyhow::Result<()> {
        let ack_ticket_below = generate_random_ack_ticket(1, 1, 4)?;
        let ack_ticket_at = generate_random_ack_ticket(1, 1, 5)?;

        let mut mock = MockTestActions::new();
        mock.expect_channel_by_id()
            .once()
            .with(mockall::predicate::eq(CHANNEL_1.get_id()))
            .return_once(|_| Some(CHANNEL_1.clone()));

        mock.expect_redeem_with_selector()
            .once()
            .with(mockall::predicate::eq(
                TicketSelector::from(CHANNEL_1.clone())
                    .with_amount(HoprBalance::from(*PRICE_PER_PACKET * 5)..)
                    .with_index_range(CHANNEL_1.ticket_index.as_u64()..=ack_ticket_at.ticket.verified_ticket().index)
                    .with_state(AcknowledgedTicketStatus::Untouched),
            ))
            .return_once(|_| vec![ChainReceipt::default()]);

        let cfg = AutoRedeemingStrategyConfig {
            minimum_redeem_ticket_value: HoprBalance::from(*PRICE_PER_PACKET * 5),
            redeem_on_winning: true,
            ..Default::default()
        };

        let ars = AutoRedeemingStrategy::new(cfg, MockChainActions(mock.into()));
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

        let mut mock = MockTestActions::new();
        mock.expect_redeem_with_selector()
            .once()
            .with(mockall::predicate::eq(
                TicketSelector::from(CHANNEL_1.clone())
                    .with_amount(HoprBalance::from(*PRICE_PER_PACKET * 5)..)
                    .with_index_range(CHANNEL_1.ticket_index.as_u64()..)
                    .with_state(AcknowledgedTicketStatus::Untouched),
            ))
            .return_once(|_| vec![ChainReceipt::default()]);

        let cfg = AutoRedeemingStrategyConfig {
            redeem_all_on_close: true,
            minimum_redeem_ticket_value: HoprBalance::from(*PRICE_PER_PACKET * 5),
            ..Default::default()
        };

        let ars = AutoRedeemingStrategy::new(cfg, MockChainActions(mock.into()));
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
        let ack_ticket_1 = generate_random_ack_ticket(0, 2, 5)?;

        let mut mock = MockTestActions::new();
        mock.expect_channel_by_id()
            .once()
            .with(mockall::predicate::eq(CHANNEL_1.get_id()))
            .return_once(|_| Some(CHANNEL_1.clone()));

        mock.expect_redeem_with_selector()
            .once()
            .with(mockall::predicate::eq(
                TicketSelector::from(CHANNEL_1.clone())
                    .with_amount(HoprBalance::zero()..)
                    .with_index_range(CHANNEL_1.ticket_index.as_u64()..=ack_ticket_1.ticket.verified_ticket().index)
                    .with_state(AcknowledgedTicketStatus::Untouched),
            ))
            .return_once(|_| vec![ChainReceipt::default()]);

        let cfg = AutoRedeemingStrategyConfig {
            minimum_redeem_ticket_value: 0.into(),
            redeem_on_winning: true,
            ..Default::default()
        };

        let ars = AutoRedeemingStrategy::new(cfg, MockChainActions(mock.into()));
        ars.on_acknowledged_winning_ticket(&ack_ticket_1).await?;
        assert!(ars.on_tick().await.is_err());

        Ok(())
    }
}
