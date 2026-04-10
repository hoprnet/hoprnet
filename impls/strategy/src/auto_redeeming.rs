//! ## Auto Redeeming Strategy
//! This strategy listens for newly added acknowledged tickets and automatically issues a redeem transaction on that
//! ticket.
//!
//! For details on default parameters, see [AutoRedeemingStrategyConfig].
use std::{
    fmt::{Debug, Display, Formatter},
    str::FromStr,
    time::Duration,
};

use async_trait::async_trait;
use futures::{StreamExt, TryStreamExt};
use hopr_async_runtime::{AbortableList, spawn_as_abortable};
use hopr_lib::{
    ChannelChange, ChannelDirection, ChannelEntry, ChannelId, ChannelStatus, HoprBalance, VerifiedTicket,
    api::{
        chain::{ChainReadChannelOperations, ChainWriteTicketOperations, ChannelSelector},
        tickets::TicketManagement,
    },
};
use parking_lot::lock_api::RwLockUpgradableReadGuard;
use serde::{Deserialize, Serialize};
use serde_with::{DisplayFromStr, serde_as};
use validator::Validate;

use crate::{
    Strategy,
    errors::{StrategyError, StrategyError::CriteriaNotSatisfied},
    strategy::SingularStrategy,
};

#[cfg(all(feature = "telemetry", not(test)))]
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
pub struct AutoRedeemingStrategy<A, T> {
    cfg: AutoRedeemingStrategyConfig,
    hopr_chain_actions: A,
    ticket_manager: T,
    // Makes sure all ongoing ticket redemptions to be terminated once the strategy is dropped.
    running_redemptions: std::sync::Arc<parking_lot::RwLock<AbortableList<ChannelId>>>,
}

impl<A, T> Debug for AutoRedeemingStrategy<A, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", Strategy::AutoRedeeming(self.cfg))
    }
}

impl<A, T> Display for AutoRedeemingStrategy<A, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", Strategy::AutoRedeeming(self.cfg))
    }
}

impl<A, T> AutoRedeemingStrategy<A, T>
where
    A: ChainReadChannelOperations + ChainWriteTicketOperations + Clone + Send + Sync + 'static,
    T: TicketManagement + Clone + Sync + Send + 'static,
{
    pub fn new(cfg: AutoRedeemingStrategyConfig, hopr_chain_actions: A, ticket_manager: T) -> Self {
        Self {
            cfg,
            hopr_chain_actions,
            ticket_manager,
            running_redemptions: std::sync::Arc::new(parking_lot::RwLock::new(AbortableList::default())),
        }
    }

    fn enqueue_redemption(&self, channel_id: &ChannelId) -> Result<(), StrategyError> {
        let redemptions = self.running_redemptions.upgradable_read();
        if !redemptions.contains(channel_id) {
            tracing::debug!(%channel_id, "attempting to start redemption in channel");

            let tmgr = self.ticket_manager.clone();
            let client = self.hopr_chain_actions.clone();
            let min_value = self.cfg.minimum_redeem_ticket_value;
            let channel_id = *channel_id;
            let redemptions_clone = self.running_redemptions.clone();

            RwLockUpgradableReadGuard::upgrade(redemptions).insert(
                channel_id,
                spawn_as_abortable!(async move {
                    let redeem_result = match tmgr
                        .redeem_stream(client.clone(), channel_id, min_value.into())
                        .map_err(StrategyError::other)
                    {
                        Ok(stream) => {
                            stream
                                .map_err(StrategyError::other)
                                .try_for_each(|res| {
                                    tracing::debug!(?res, %channel_id, "ticket redemption completed");
                                    futures::future::ok(())
                                })
                                .await
                        }
                        err => {
                            // Add small delay to avoid the write lock acquired for insertion
                            // still being held.
                            hopr_async_runtime::prelude::sleep(Duration::from_millis(100)).await;
                            err.map(|_| ())
                        }
                    };

                    tracing::debug!(?redeem_result, %channel_id, "redemption in channel complete");
                    redemptions_clone.write().abort_one(&channel_id);
                }),
            );
            Ok(())
        } else {
            tracing::debug!(%channel_id, "existing on-going redemption");
            Err(StrategyError::InProgress)
        }
    }
}

#[async_trait]
impl<A, T> SingularStrategy for AutoRedeemingStrategy<A, T>
where
    A: ChainReadChannelOperations + ChainWriteTicketOperations + Clone + Send + Sync + 'static,
    T: TicketManagement + Clone + Sync + Send + 'static,
{
    async fn on_tick(&self) -> crate::errors::Result<()> {
        if !self.cfg.redeem_on_winning {
            tracing::debug!("trying to redeem all tickets in all channels");

            self.hopr_chain_actions
                .stream_channels(
                    ChannelSelector::default()
                        .with_destination(*self.hopr_chain_actions.me())
                        .with_redeemable_channels(Duration::from_secs(30).into()),
                )
                .map_err(StrategyError::other)?
                .for_each(|channel| {
                    if let Err(error) = self.enqueue_redemption(channel.get_id()) {
                        tracing::error!(
                            %error,
                            channel_id = %channel.get_id(),
                            "cannot start redemption in channel"
                        );
                    }
                    futures::future::ready(())
                })
                .await;

            Ok(())
        } else {
            Err(CriteriaNotSatisfied)
        }
    }

    async fn on_acknowledged_winning_ticket(&self, ack: &VerifiedTicket) -> crate::errors::Result<()> {
        if self.cfg.redeem_on_winning && ack.verified_ticket().amount.ge(&self.cfg.minimum_redeem_ticket_value) {
            let chain_api = self.hopr_chain_actions.clone();
            let channel_id = *ack.channel_id();
            let maybe_channel = hopr_async_runtime::prelude::spawn_blocking(move || {
                chain_api.channel_by_id(&channel_id).map_err(StrategyError::other)
            })
            .await
            .map_err(StrategyError::other)??;

            if let Some(channel) = maybe_channel {
                tracing::info!(%ack, "redeeming");

                if ack.verified_ticket().index < channel.ticket_index {
                    tracing::error!(%ack, "acknowledged ticket is older than channel ticket index");
                    return Err(CriteriaNotSatisfied);
                }

                // Raises an error if redemption in this channel is ongoing
                self.enqueue_redemption(channel.get_id())?;

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
                tracing::debug!(?channel, "ignoring channel state change that's not in PendingToClose");
                return Ok(());
            }
            tracing::info!(%channel, "channel transitioned to PendingToClose, checking if it has tickets to redeem");

            // Raises an error if redemption in this channel is ongoing
            self.enqueue_redemption(channel.get_id())?;

            Ok(())
        } else {
            Err(CriteriaNotSatisfied)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        ops::Add,
        sync::Arc,
        time::{Duration, SystemTime},
    };

    use futures::stream::BoxStream;
    use futures_time::future::FutureExt as TimeExt;
    use hex_literal::hex;
    use hopr_api::{
        tickets::{ChannelStats, RedemptionResult},
        types::crypto_random::Randomizable,
    };
    use hopr_chain_connector::{HoprBlockchainSafeConnector, create_trustful_hopr_blokli_connector, testing::*};
    use hopr_lib::{
        Address, BytesRepresentable, ChainKeypair, HalfKey, Hash, Keypair, RedeemableTicket, Response, TicketBuilder,
        UnitaryFloatOps, WinningProbability, XDaiBalance,
    };

    use super::*;

    mockall::mock! {
        pub TicketMgmt {}
         #[allow(refining_impl_trait)]
        impl TicketManagement for TicketMgmt {
            type Error = std::io::Error;
            fn redeem_stream<C: ChainWriteTicketOperations + Send + Sync + 'static>(
                &self,
                client: C,
                channel_id: ChannelId,
                min_amount: Option<HoprBalance>,
            ) -> Result<BoxStream<'static, Result<RedemptionResult, std::io::Error>>, std::io::Error>;

            fn neglect_tickets(
                &self,
                channel_id: &ChannelId,
                max_ticket_index: Option<u64>,
            ) -> Result<Vec<VerifiedTicket>, std::io::Error>;

            fn ticket_stats<'a>(&self, channel_id: Option<&'a ChannelId>) -> Result<ChannelStats, std::io::Error>;
        }
    }

    lazy_static::lazy_static! {
        static ref ALICE: ChainKeypair = ChainKeypair::from_secret(&hex!("492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775")).expect("lazy static keypair should be valid");
        static ref BOB: ChainKeypair = ChainKeypair::from_secret(&hex!("48680484c6fc31bc881a0083e6e32b6dc789f9eaba0f8b981429fd346c697f8c")).expect("lazy static keypair should be valid");
        static ref CHARLIE: ChainKeypair = ChainKeypair::from_secret(&hex!("d39a926980d6fa96a9eba8f8058b2beb774bc11866a386e9ddf9dc1152557c26")).expect("lazy static keypair should be constructible");
        static ref PRICE_PER_PACKET: HoprBalance = 10000000000000000_u128.into(); // 0.01 HOPR

        static ref CHANNEL_1: ChannelEntry = ChannelEntry::builder()
            .between(&*ALICE, &*BOB)
            .balance(*PRICE_PER_PACKET * 10)
            .ticket_index(0)
            .status(ChannelStatus::Open)
            .epoch(4)
            .build()
            .unwrap();

        static ref CHANNEL_2: ChannelEntry = ChannelEntry::builder()
            .between(&*CHARLIE, &*BOB)
            .balance(*PRICE_PER_PACKET * 11)
            .ticket_index(1)
            .status(ChannelStatus::Open)
            .epoch(4)
            .build()
            .unwrap();

        static ref CHAIN_CLIENT: BlokliTestClient<StaticState> = BlokliTestStateBuilder::default()
            .with_generated_accounts(&[ALICE.public().as_ref(), BOB.public().as_ref(), CHARLIE.public().as_ref()], false, XDaiBalance::new_base(1), HoprBalance::new_base(1000))
            .with_channels([*CHANNEL_1, *CHANNEL_2])
            .build_static_client();
    }

    fn generate_random_ack_ticket(index: u64, worth_packets: u32) -> anyhow::Result<RedeemableTicket> {
        let hk1 = HalfKey::random();
        let hk2 = HalfKey::random();

        let challenge = Response::from_half_keys(&hk1, &hk2)?.to_challenge()?;

        Ok(TicketBuilder::default()
            .counterparty(&*BOB)
            .amount(PRICE_PER_PACKET.div_f64(1.0f64)?.amount() * worth_packets)
            .index(index)
            .win_prob(WinningProbability::ALWAYS)
            .channel_epoch(4)
            .challenge(challenge)
            .build_signed(&ALICE, &Hash::default())?
            .into_acknowledged(Response::from_half_keys(&hk1, &hk2)?)
            .into_redeemable(&BOB, &Hash::default())?)
    }

    type TestConnector = Arc<HoprBlockchainSafeConnector<BlokliTestClient<StaticState>>>;

    async fn await_redemption_queue_empty(redeems: Arc<parking_lot::RwLock<AbortableList<ChannelId>>>) {
        loop {
            hopr_async_runtime::prelude::sleep(Duration::from_millis(100)).await;

            if redeems.read().is_empty() {
                break;
            }
        }
    }

    #[test_log::test(tokio::test)]
    async fn test_auto_redeeming_strategy_redeem() -> anyhow::Result<()> {
        let ack_ticket = generate_random_ack_ticket(0, 5)?;

        let mut connector = create_trustful_hopr_blokli_connector(
            &BOB,
            Default::default(),
            CHAIN_CLIENT.clone(),
            [1u8; Address::SIZE].into(),
        )
        .await?;
        connector.connect().await?;

        let cfg = AutoRedeemingStrategyConfig {
            minimum_redeem_ticket_value: 0.into(),
            redeem_on_winning: true,
            ..Default::default()
        };

        let mut mock_tmgr = MockTicketMgmt::new();
        mock_tmgr
            .expect_redeem_stream()
            .once()
            .with(
                mockall::predicate::always(),
                mockall::predicate::eq(*CHANNEL_1.get_id()),
                mockall::predicate::eq(Some(cfg.minimum_redeem_ticket_value)),
            )
            .return_once(move |_: TestConnector, _, _| {
                Ok(futures::stream::once(futures::future::ok(RedemptionResult::Redeemed(ack_ticket.ticket))).boxed())
            });

        let ars = AutoRedeemingStrategy::new(cfg, Arc::new(connector), Arc::new(mock_tmgr));

        ars.on_acknowledged_winning_ticket(&ack_ticket.ticket).await?;
        assert!(ars.on_tick().await.is_err());

        await_redemption_queue_empty(ars.running_redemptions.clone())
            .timeout(futures_time::time::Duration::from_secs(5))
            .await?;

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn test_auto_redeeming_strategy_redeem_on_tick() -> anyhow::Result<()> {
        let mut connector = create_trustful_hopr_blokli_connector(
            &BOB,
            Default::default(),
            CHAIN_CLIENT.clone(),
            [1u8; Address::SIZE].into(),
        )
        .await?;
        connector.connect().await?;

        let cfg = AutoRedeemingStrategyConfig {
            minimum_redeem_ticket_value: HoprBalance::from(*PRICE_PER_PACKET * 5),
            redeem_on_winning: false,
            ..Default::default()
        };

        let mut mock_tmgr = MockTicketMgmt::new();
        mock_tmgr
            .expect_redeem_stream()
            .once()
            .with(
                mockall::predicate::always(),
                mockall::predicate::eq(*CHANNEL_1.get_id()),
                mockall::predicate::eq(Some(cfg.minimum_redeem_ticket_value)),
            )
            .return_once(|_: TestConnector, _, _| Ok(futures::stream::empty().boxed()));

        mock_tmgr
            .expect_redeem_stream()
            .once()
            .with(
                mockall::predicate::always(),
                mockall::predicate::eq(*CHANNEL_2.get_id()),
                mockall::predicate::eq(Some(cfg.minimum_redeem_ticket_value)),
            )
            .return_once(|_: TestConnector, _, _| Ok(futures::stream::empty().boxed()));

        let ars = AutoRedeemingStrategy::new(cfg, Arc::new(connector), Arc::new(mock_tmgr));
        ars.on_tick().await?;

        await_redemption_queue_empty(ars.running_redemptions.clone())
            .timeout(futures_time::time::Duration::from_secs(5))
            .await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_auto_redeeming_strategy_should_redeem_singular_ticket_on_close() -> anyhow::Result<()> {
        let mut channel = *CHANNEL_1;
        channel.status = ChannelStatus::PendingToClose(SystemTime::now().add(Duration::from_secs(100)));

        let mut connector = create_trustful_hopr_blokli_connector(
            &BOB,
            Default::default(),
            CHAIN_CLIENT.clone(),
            [1u8; Address::SIZE].into(),
        )
        .await?;
        connector.connect().await?;

        let cfg = AutoRedeemingStrategyConfig {
            redeem_all_on_close: true,
            minimum_redeem_ticket_value: HoprBalance::from(*PRICE_PER_PACKET * 5),
            ..Default::default()
        };

        let mut mock_tmgr = MockTicketMgmt::new();
        mock_tmgr
            .expect_redeem_stream()
            .once()
            .with(
                mockall::predicate::always(),
                mockall::predicate::eq(*CHANNEL_1.get_id()),
                mockall::predicate::eq(Some(cfg.minimum_redeem_ticket_value)),
            )
            .return_once(move |_: TestConnector, _, _| Ok(futures::stream::empty().boxed()));

        let ars = AutoRedeemingStrategy::new(cfg, Arc::new(connector), Arc::new(mock_tmgr));
        ars.on_own_channel_changed(
            &channel,
            ChannelDirection::Incoming,
            ChannelChange::Status {
                left: ChannelStatus::Open,
                right: channel.status,
            },
        )
        .await?;

        await_redemption_queue_empty(ars.running_redemptions.clone())
            .timeout(futures_time::time::Duration::from_secs(5))
            .await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_auto_redeeming_strategy_should_not_redeem_multiple_times_in_same_channel() -> anyhow::Result<()> {
        let ack_ticket_1 = generate_random_ack_ticket(0, 5)?;

        let mut connector = create_trustful_hopr_blokli_connector(
            &BOB,
            Default::default(),
            CHAIN_CLIENT.clone(),
            [1u8; Address::SIZE].into(),
        )
        .await?;
        connector.connect().await?;

        let cfg = AutoRedeemingStrategyConfig {
            minimum_redeem_ticket_value: 0.into(),
            redeem_on_winning: true,
            ..Default::default()
        };

        let mut mock_tmgr = MockTicketMgmt::new();
        mock_tmgr
            .expect_redeem_stream()
            .once()
            .with(
                mockall::predicate::always(),
                mockall::predicate::eq(*CHANNEL_1.get_id()),
                mockall::predicate::eq(Some(cfg.minimum_redeem_ticket_value)),
            )
            .return_once(move |_: TestConnector, _, _| {
                Ok(futures::stream::once(
                    futures::future::ok(RedemptionResult::Redeemed(ack_ticket_1.ticket))
                        .delay(futures_time::time::Duration::from_millis(500)),
                )
                .boxed())
            });

        let ars = AutoRedeemingStrategy::new(cfg, Arc::new(connector), Arc::new(mock_tmgr));
        ars.on_acknowledged_winning_ticket(&ack_ticket_1.ticket).await?;
        assert!(matches!(
            ars.on_acknowledged_winning_ticket(&ack_ticket_1.ticket).await,
            Err(StrategyError::InProgress)
        ));

        let mut channel = *CHANNEL_1;
        channel.status = ChannelStatus::PendingToClose(SystemTime::now().add(Duration::from_secs(100)));

        assert!(matches!(
            ars.on_own_channel_changed(
                &channel,
                ChannelDirection::Incoming,
                ChannelChange::Status {
                    left: ChannelStatus::Open,
                    right: channel.status,
                }
            )
            .await,
            Err(StrategyError::InProgress)
        ));
        assert!(ars.on_tick().await.is_err());

        await_redemption_queue_empty(ars.running_redemptions.clone())
            .timeout(futures_time::time::Duration::from_secs(5))
            .await?;

        Ok(())
    }
}
