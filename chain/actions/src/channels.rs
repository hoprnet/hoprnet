//! This module contains the [ChannelActions] trait defining HOPR channels operations.
//!
//! An implementation of this trait is added to [ChainActions] which realizes the redemption
//! operations via [ActionQueue](crate::action_queue::ActionQueue).
//! There are 4 basic high-level on-chain functions in the [ChannelActions] trait:
//! - [open_channel](ChannelActions::open_channel)
//! - [fund_channel](ChannelActions::fund_channel)
//! - [close_channel](ChannelActions::close_channel)
//!
//! All the functions do the necessary validations using the DB and then post the corresponding action
//! into the [ActionQueue](crate::action_queue::ActionQueue).
//! The functions return immediately, but provide futures that can be awaited in case the callers wishes to await the on-chain
//! confirmation of the corresponding operation.
//! See the details in [ActionQueue](crate::action_queue::ActionQueue) on how the confirmation is realized by awaiting the respective [SignificantChainEvent](hopr_chain_types::chain_events::SignificantChainEvent)
//! by the Indexer.
use async_trait::async_trait;
use futures::future::join_all;
use hopr_chain_types::actions::Action;
use hopr_crypto_types::types::Hash;
use hopr_db_sql::HoprDbAllOperations;
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::*;
use std::time::Duration;
use tracing::{debug, error, info};

use crate::action_queue::PendingAction;
use crate::errors::ChainActionsError::{
    BalanceTooLow, ClosureTimeHasNotElapsed, InvalidArguments, InvalidState, NotEnoughAllowance, PeerAccessDenied,
};
use crate::errors::{
    ChainActionsError::{ChannelAlreadyClosed, ChannelAlreadyExists, ChannelDoesNotExist, NoChannelToClose},
    Result,
};
use crate::redeem::TicketRedeemActions;
use crate::ChainActions;

use hopr_platform::time::native::current_time;

/// Gathers all channel related on-chain actions.
#[async_trait]
pub trait ChannelActions {
    /// Opens a channel to the given `destination` with the given `amount` staked.
    async fn open_channel(&self, destination: Address, amount: Balance) -> Result<PendingAction>;

    /// Funds the given channel with the given `amount`
    async fn fund_channel(&self, channel_id: Hash, amount: Balance) -> Result<PendingAction>;

    /// Closes the channel to counterparty in the given direction. Optionally can issue redeeming of all tickets in that channel.
    async fn close_channel(
        &self,
        counterparty: Address,
        direction: ChannelDirection,
        redeem_before_close: bool,
    ) -> Result<PendingAction>;

    /// Closes all channels in the given direction. Optionally can issue redeeming of all tickets in those channels.
    async fn close_all_channels(&self, direction: ChannelDirection, redeem_before_close: bool)
        -> Result<PendingAction>;
}

#[async_trait]
impl<Db> ChannelActions for ChainActions<Db>
where
    Db: HoprDbAllOperations + Clone + Send + Sync + std::fmt::Debug + 'static,
{
    #[tracing::instrument(level = "debug", skip(self))]
    async fn open_channel(&self, destination: Address, amount: Balance) -> Result<PendingAction> {
        if self.self_address() == destination {
            return Err(InvalidArguments("cannot open channel to self".into()));
        }

        if amount.eq(&amount.of_same("0")) || amount.balance_type() != BalanceType::HOPR {
            return Err(InvalidArguments("invalid balance or balance type given".into()));
        }

        // Perform all checks
        let db_clone = self.db.clone();
        let self_addr = self.self_address();
        self.db
            .begin_transaction()
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    let allowance = db_clone.get_safe_hopr_allowance(Some(tx)).await?;
                    debug!(%allowance, "current staking safe allowance");
                    if allowance.lt(&amount) {
                        return Err(NotEnoughAllowance);
                    }

                    let hopr_balance = db_clone.get_safe_hopr_balance(Some(tx)).await?;
                    debug!(balance = %hopr_balance, "current Safe HOPR balance");
                    if hopr_balance.lt(&amount) {
                        return Err(BalanceTooLow);
                    }

                    if db_clone.get_indexer_data(Some(tx)).await?.nr_enabled
                        && !db_clone.is_allowed_in_network_registry(Some(tx), destination).await?
                    {
                        return Err(PeerAccessDenied);
                    }

                    let maybe_channel = db_clone
                        .get_channel_by_parties(Some(tx), &self_addr, &destination, false)
                        .await?;
                    if let Some(channel) = maybe_channel {
                        debug!(%channel, "already found existing channel");
                        if channel.status != ChannelStatus::Closed {
                            error!(
                                %destination,
                                "channel to destination is already opened or pending to close"
                            );
                            return Err(ChannelAlreadyExists);
                        }
                    }
                    Ok(())
                })
            })
            .await?;

        info!(%destination, %amount, "initiating channel open");
        self.tx_sender.send(Action::OpenChannel(destination, amount)).await
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn fund_channel(&self, channel_id: Hash, amount: Balance) -> Result<PendingAction> {
        if amount.eq(&amount.of_same("0")) || amount.balance_type() != BalanceType::HOPR {
            return Err(InvalidArguments("invalid balance or balance type given".into()));
        }

        let db_clone = self.db.clone();
        let maybe_channel = self
            .db
            .begin_transaction()
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    let allowance = db_clone.get_safe_hopr_allowance(Some(tx)).await?;
                    debug!(%allowance, "current staking safe allowance");
                    if allowance.lt(&amount) {
                        return Err(NotEnoughAllowance);
                    }

                    let hopr_balance = db_clone.get_safe_hopr_balance(Some(tx)).await?;
                    debug!(balance = %hopr_balance, "current Safe HOPR balance");
                    if hopr_balance.lt(&amount) {
                        return Err(BalanceTooLow);
                    }

                    Ok(db_clone.get_channel_by_id(Some(tx), &channel_id).await?)
                })
            })
            .await?;

        match maybe_channel {
            Some(channel) => {
                if channel.status == ChannelStatus::Open {
                    info!("initiating funding of {channel} with {amount}");
                    self.tx_sender.send(Action::FundChannel(channel, amount)).await
                } else {
                    Err(InvalidState(format!("channel {channel_id} is not opened")))
                }
            }
            None => Err(ChannelDoesNotExist),
        }
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn close_channel(
        &self,
        counterparty: Address,
        direction: ChannelDirection,
        redeem_before_close: bool,
    ) -> Result<PendingAction> {
        let maybe_channel = match direction {
            ChannelDirection::Incoming => {
                self.db
                    .get_channel_by_parties(None, &counterparty, &self.self_address(), false)
                    .await?
            }
            ChannelDirection::Outgoing => {
                self.db
                    .get_channel_by_parties(None, &self.self_address(), &counterparty, false)
                    .await?
            }
        };

        match maybe_channel {
            Some(channel) => {
                match channel.status {
                    ChannelStatus::Closed => Err(ChannelAlreadyClosed),
                    ChannelStatus::PendingToClose(_) => {
                        let remaining_closure_time = channel.remaining_closure_time(current_time());
                        info!(%channel, ?remaining_closure_time, "remaining closure time update for a channel");
                        match remaining_closure_time {
                            Some(Duration::ZERO) => {
                                info!(%channel, %direction, "initiating finalization of channel closure");
                                self.tx_sender.send(Action::CloseChannel(channel, direction)).await
                            }
                            _ => Err(ClosureTimeHasNotElapsed(
                                channel
                                    .remaining_closure_time(current_time())
                                    .expect("impossible: closure time has not passed but no remaining closure time")
                                    .as_secs(),
                            )),
                        }
                    }
                    ChannelStatus::Open => {
                        if redeem_before_close {
                            // TODO: trigger aggregation
                            // Do not await the redemption, just submit it to the queue
                            let redeemed = self.redeem_tickets_in_channel(&channel, false).await?.len();
                            info!(count = redeemed, %channel, "redeemed tickets before channel closing");
                        }

                        info!(%channel, ?direction, "initiating channel closure");
                        self.tx_sender.send(Action::CloseChannel(channel, direction)).await
                    }
                }
            }
            None => Err(ChannelDoesNotExist),
        }
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn close_all_channels(
        &self,
        direction: ChannelDirection,
        redeem_before_close: bool,
    ) -> Result<PendingAction> {
        let channels = self
            .db
            .get_channels_via(None, direction, &self.self_address())
            .await?
            .into_iter()
            .filter(|channel| matches!(channel.status, ChannelStatus::PendingToClose(_) | ChannelStatus::Open))
            .collect::<Vec<_>>();

        // Filter out channels that are in pending to close state but not ready to be finalized
        let channels = channels
            .into_iter()
            .filter(|channel| {
                if let ChannelStatus::PendingToClose(closure_time) = channel.status {
                    let remaining_closure_time = channel.remaining_closure_time(closure_time);
                    info!(%channel, ?remaining_closure_time, "remaining closure time update for a channel");
                    remaining_closure_time == Some(Duration::ZERO)
                } else {
                    true
                }
            })
            .collect::<Vec<_>>();

        if channels.is_empty() {
            return Err(NoChannelToClose);
        }

        if redeem_before_close {
            join_all(channels.iter().map(|channel| async {
                // Do not await the redemption, just submit it to the queue
                self.redeem_tickets_in_channel(channel, false).await
            }))
            .await;
        }
        self.tx_sender.send(Action::CloseChannels(channels, direction)).await
    }
}
#[cfg(test)]
mod tests {
    use crate::action_queue::{ActionQueue, MockTransactionExecutor};
    use crate::action_state::MockActionState;
    use crate::channels::ChannelActions;
    use crate::errors::ChainActionsError;
    use crate::ChainActions;
    use futures::FutureExt;
    use hex_literal::hex;
    use hopr_chain_types::actions::Action;
    use hopr_chain_types::chain_events::{ChainEventType, SignificantChainEvent};
    use hopr_crypto_random::random_bytes;
    use hopr_crypto_types::prelude::*;
    use hopr_db_sql::channels::HoprDbChannelOperations;
    use hopr_db_sql::db::HoprDb;
    use hopr_db_sql::HoprDbGeneralModelOperations;
    use hopr_db_sql::{api::info::DomainSeparator, info::HoprDbInfoOperations};
    use hopr_internal_types::prelude::*;
    use hopr_primitive_types::prelude::*;
    use lazy_static::lazy_static;
    use mockall::Sequence;
    use std::{
        ops::{Add, Sub},
        time::{Duration, SystemTime},
    };

    lazy_static! {
        static ref ALICE_KP: ChainKeypair = ChainKeypair::from_secret(&hex!(
            "492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775"
        ))
        .expect("lazy static keypair should be constructible");
        static ref BOB_KP: ChainKeypair = ChainKeypair::from_secret(&hex!(
            "48680484c6fc31bc881a0083e6e32b6dc789f9eaba0f8b981429fd346c697f8c"
        ))
        .expect("lazy static keypair should be constructible");
        static ref ALICE: Address = ALICE_KP.public().to_address();
        static ref BOB: Address = BOB_KP.public().to_address();
    }

    #[async_std::test]
    async fn test_open_channel() -> anyhow::Result<()> {
        let stake = Balance::new(10_u32, BalanceType::HOPR);
        let random_hash = Hash::from(random_bytes::<{ Hash::SIZE }>());

        let db = HoprDb::new_in_memory(ALICE_KP.clone()).await?;
        let db_clone = db.clone();
        db.begin_transaction()
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    db_clone
                        .set_safe_hopr_allowance(Some(tx), Balance::new(10_000_000_u64, BalanceType::HOPR))
                        .await?;
                    db_clone
                        .set_safe_hopr_balance(Some(tx), Balance::new(5_000_000_u64, BalanceType::HOPR))
                        .await?;
                    db_clone
                        .set_domain_separator(Some(tx), DomainSeparator::Channel, Default::default())
                        .await?;
                    db_clone.set_network_registry_enabled(Some(tx), false).await
                })
            })
            .await?;

        let mut tx_exec = MockTransactionExecutor::new();
        tx_exec
            .expect_fund_channel()
            .times(1)
            .withf(move |dst, balance| BOB.eq(dst) && stake.eq(balance))
            .returning(move |_, _| Ok(random_hash));

        let new_channel = ChannelEntry::new(*ALICE, *BOB, stake, U256::zero(), ChannelStatus::Open, U256::zero());

        let mut indexer_action_tracker = MockActionState::new();
        indexer_action_tracker
            .expect_register_expectation()
            .once()
            .returning(move |_| {
                Ok(futures::future::ok(SignificantChainEvent {
                    tx_hash: random_hash,
                    event_type: ChainEventType::ChannelOpened(new_channel),
                })
                .boxed())
            });

        let tx_queue = ActionQueue::new(db.clone(), indexer_action_tracker, tx_exec, Default::default());

        let tx_sender = tx_queue.new_sender();
        async_std::task::spawn(async move { tx_queue.start().await });

        let actions = ChainActions::new(&ALICE_KP, db.clone(), tx_sender.clone());

        let tx_res = actions.open_channel(*BOB, stake).await?.await?;

        assert_eq!(tx_res.tx_hash, random_hash, "tx hashes must be equal");
        assert!(
            matches!(tx_res.action, Action::OpenChannel(_, _)),
            "must be open channel action"
        );
        assert!(
            matches!(tx_res.event, Some(ChainEventType::ChannelOpened(_))),
            "must correspond to open channel chain event"
        );

        Ok(())
    }

    #[async_std::test]
    async fn test_should_not_open_channel_again() -> anyhow::Result<()> {
        let stake = Balance::new(10_u32, BalanceType::HOPR);

        let channel = ChannelEntry::new(*ALICE, *BOB, stake, U256::zero(), ChannelStatus::Open, U256::zero());

        let db = HoprDb::new_in_memory(ALICE_KP.clone()).await?;
        let db_clone = db.clone();
        db.begin_transaction()
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    db_clone
                        .set_safe_hopr_allowance(Some(tx), Balance::new(10_000_000_u64, BalanceType::HOPR))
                        .await?;
                    db_clone
                        .set_safe_hopr_balance(Some(tx), Balance::new(5_000_000_u64, BalanceType::HOPR))
                        .await?;
                    db_clone.set_network_registry_enabled(Some(tx), false).await?;
                    db_clone
                        .set_domain_separator(Some(tx), DomainSeparator::Channel, Default::default())
                        .await?;
                    db_clone.upsert_channel(Some(tx), channel).await
                })
            })
            .await?;

        let tx_queue = ActionQueue::new(
            db.clone(),
            MockActionState::new(),
            MockTransactionExecutor::new(),
            Default::default(),
        );

        let actions = ChainActions::new(&ALICE_KP, db.clone(), tx_queue.new_sender());

        assert!(
            matches!(
                actions
                    .open_channel(*BOB, stake)
                    .await
                    .err()
                    .expect("should be an error"),
                ChainActionsError::ChannelAlreadyExists
            ),
            "should fail when channel exists"
        );

        Ok(())
    }

    #[async_std::test]
    async fn test_should_not_open_channel_to_self() -> anyhow::Result<()> {
        let stake = Balance::new(10_u32, BalanceType::HOPR);

        let db = HoprDb::new_in_memory(ALICE_KP.clone()).await?;
        let db_clone = db.clone();
        db.begin_transaction()
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    db_clone
                        .set_safe_hopr_allowance(Some(tx), Balance::new(10_000_000_u64, BalanceType::HOPR))
                        .await?;
                    db_clone
                        .set_safe_hopr_balance(Some(tx), Balance::new(5_000_000_u64, BalanceType::HOPR))
                        .await?;
                    db_clone.set_network_registry_enabled(Some(tx), false).await?;
                    db_clone
                        .set_domain_separator(Some(tx), DomainSeparator::Channel, Default::default())
                        .await
                })
            })
            .await?;

        let tx_queue = ActionQueue::new(
            db.clone(),
            MockActionState::new(),
            MockTransactionExecutor::new(),
            Default::default(),
        );

        let actions = ChainActions::new(&ALICE_KP, db.clone(), tx_queue.new_sender());

        assert!(
            matches!(
                actions
                    .open_channel(*ALICE, stake)
                    .await
                    .err()
                    .expect("should be an error"),
                ChainActionsError::InvalidArguments(_)
            ),
            "should not create channel to self"
        );
        Ok(())
    }

    #[async_std::test]
    async fn test_open_should_not_allow_invalid_balance() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(ALICE_KP.clone()).await?;
        let db_clone = db.clone();
        db.begin_transaction()
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    db_clone
                        .set_safe_hopr_allowance(Some(tx), Balance::new(10_000_000_u64, BalanceType::HOPR))
                        .await?;
                    db_clone
                        .set_safe_hopr_balance(Some(tx), Balance::new(5_000_000_u64, BalanceType::HOPR))
                        .await?;
                    db_clone.set_network_registry_enabled(Some(tx), false).await?;
                    db_clone
                        .set_domain_separator(Some(tx), DomainSeparator::Channel, Default::default())
                        .await
                })
            })
            .await?;

        let tx_queue = ActionQueue::new(
            db.clone(),
            MockActionState::new(),
            MockTransactionExecutor::new(),
            Default::default(),
        );

        let actions = ChainActions::new(&ALICE_KP, db.clone(), tx_queue.new_sender());
        let stake = Balance::new(10_u32, BalanceType::Native);
        assert!(
            matches!(
                actions
                    .open_channel(*BOB, stake)
                    .await
                    .err()
                    .expect("should be an error"),
                ChainActionsError::InvalidArguments(_)
            ),
            "should not allow invalid balance"
        );

        let stake = Balance::new(0_u32, BalanceType::HOPR);

        assert!(
            matches!(
                actions
                    .open_channel(*BOB, stake)
                    .await
                    .err()
                    .expect("should be an error"),
                ChainActionsError::InvalidArguments(_)
            ),
            "should not allow invalid balance"
        );
        Ok(())
    }

    #[async_std::test]
    async fn test_should_not_open_if_not_enough_allowance() -> anyhow::Result<()> {
        let stake = Balance::new(10_000_u32, BalanceType::HOPR);

        let db = HoprDb::new_in_memory(ALICE_KP.clone()).await?;
        let db_clone = db.clone();
        db.begin_transaction()
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    db_clone
                        .set_safe_hopr_allowance(Some(tx), Balance::new(1_000_u64, BalanceType::HOPR))
                        .await?;
                    db_clone
                        .set_safe_hopr_balance(Some(tx), Balance::new(5_000_000_u64, BalanceType::HOPR))
                        .await?;
                    db_clone.set_network_registry_enabled(Some(tx), false).await?;
                    db_clone
                        .set_domain_separator(Some(tx), DomainSeparator::Channel, Default::default())
                        .await
                })
            })
            .await?;

        let tx_queue = ActionQueue::new(
            db.clone(),
            MockActionState::new(),
            MockTransactionExecutor::new(),
            Default::default(),
        );

        let actions = ChainActions::new(&ALICE_KP, db.clone(), tx_queue.new_sender());

        assert!(
            matches!(
                actions
                    .open_channel(*BOB, stake)
                    .await
                    .err()
                    .expect("should be an error"),
                ChainActionsError::NotEnoughAllowance
            ),
            "should fail when not enough allowance"
        );
        Ok(())
    }

    #[async_std::test]
    async fn test_should_not_open_if_not_enough_token_balance() -> anyhow::Result<()> {
        let stake = Balance::new(10_000_u32, BalanceType::HOPR);

        let db = HoprDb::new_in_memory(ALICE_KP.clone()).await?;
        let db_clone = db.clone();
        db.begin_transaction()
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    db_clone
                        .set_safe_hopr_allowance(Some(tx), Balance::new(10_000_000_u64, BalanceType::HOPR))
                        .await?;
                    db_clone
                        .set_safe_hopr_balance(Some(tx), Balance::new(1_u64, BalanceType::HOPR))
                        .await?;
                    db_clone.set_network_registry_enabled(Some(tx), false).await?;
                    db_clone
                        .set_domain_separator(Some(tx), DomainSeparator::Channel, Default::default())
                        .await
                })
            })
            .await?;

        let tx_queue = ActionQueue::new(
            db.clone(),
            MockActionState::new(),
            MockTransactionExecutor::new(),
            Default::default(),
        );

        let actions = ChainActions::new(&ALICE_KP, db.clone(), tx_queue.new_sender());

        assert!(
            matches!(
                actions
                    .open_channel(*BOB, stake)
                    .await
                    .err()
                    .expect("should be an error"),
                ChainActionsError::BalanceTooLow
            ),
            "should fail when not enough token balance"
        );
        Ok(())
    }

    #[async_std::test]
    async fn test_fund_channel() -> anyhow::Result<()> {
        let stake = Balance::new(10_u32, BalanceType::HOPR);
        let random_hash = Hash::from(random_bytes::<{ Hash::SIZE }>());
        let channel = ChannelEntry::new(*ALICE, *BOB, stake, U256::zero(), ChannelStatus::Open, U256::zero());

        let db = HoprDb::new_in_memory(ALICE_KP.clone()).await?;
        let db_clone = db.clone();
        db.begin_transaction()
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    db_clone
                        .set_safe_hopr_allowance(Some(tx), Balance::new(10_000_000_u64, BalanceType::HOPR))
                        .await?;
                    db_clone
                        .set_safe_hopr_balance(Some(tx), Balance::new(5_000_000_u64, BalanceType::HOPR))
                        .await?;
                    db_clone.set_network_registry_enabled(Some(tx), false).await?;
                    db_clone
                        .set_domain_separator(Some(tx), DomainSeparator::Channel, Default::default())
                        .await?;
                    db_clone.upsert_channel(Some(tx), channel).await
                })
            })
            .await?;

        let mut tx_exec = MockTransactionExecutor::new();
        tx_exec
            .expect_fund_channel()
            .times(1)
            .withf(move |dest, balance| channel.destination.eq(dest) && stake.eq(balance))
            .returning(move |_, _| Ok(random_hash));

        let mut indexer_action_tracker = MockActionState::new();
        indexer_action_tracker
            .expect_register_expectation()
            .once()
            .returning(move |_| {
                Ok(futures::future::ok(SignificantChainEvent {
                    tx_hash: random_hash,
                    event_type: ChainEventType::ChannelBalanceIncreased(channel, stake),
                })
                .boxed())
            });

        let tx_queue = ActionQueue::new(db.clone(), indexer_action_tracker, tx_exec, Default::default());
        let tx_sender = tx_queue.new_sender();
        async_std::task::spawn(async move {
            tx_queue.start().await;
        });

        let actions = ChainActions::new(&ALICE_KP, db.clone(), tx_sender.clone());

        let tx_res = actions.fund_channel(channel.get_id(), stake).await?.await?;

        assert_eq!(tx_res.tx_hash, random_hash, "tx hashes must be equal");
        assert!(
            matches!(tx_res.action, Action::FundChannel(_, _)),
            "must be open channel action"
        );
        assert!(
            matches!(tx_res.event, Some(ChainEventType::ChannelBalanceIncreased(_, _))),
            "must correspond to channel chain event"
        );
        Ok(())
    }

    #[async_std::test]
    async fn test_should_not_fund_nonexistent_channel() -> anyhow::Result<()> {
        let channel_id = generate_channel_id(&*ALICE, &*BOB);

        let db = HoprDb::new_in_memory(ALICE_KP.clone()).await?;
        let db_clone = db.clone();
        db.begin_transaction()
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    db_clone
                        .set_safe_hopr_allowance(Some(tx), Balance::new(10_000_000_u64, BalanceType::HOPR))
                        .await?;
                    db_clone
                        .set_safe_hopr_balance(Some(tx), Balance::new(5_000_000_u64, BalanceType::HOPR))
                        .await?;
                    db_clone.set_network_registry_enabled(Some(tx), false).await?;
                    db_clone
                        .set_domain_separator(Some(tx), DomainSeparator::Channel, Default::default())
                        .await
                })
            })
            .await?;

        let tx_queue = ActionQueue::new(
            db.clone(),
            MockActionState::new(),
            MockTransactionExecutor::new(),
            Default::default(),
        );

        let actions = ChainActions::new(&ALICE_KP, db.clone(), tx_queue.new_sender());
        let stake = Balance::new(10_u32, BalanceType::HOPR);
        assert!(
            matches!(
                actions
                    .fund_channel(channel_id, stake)
                    .await
                    .err()
                    .expect("should be an error"),
                ChainActionsError::ChannelDoesNotExist
            ),
            "should fail when channel does not exist"
        );
        Ok(())
    }

    #[async_std::test]
    async fn test_fund_should_not_allow_invalid_balance() -> anyhow::Result<()> {
        let channel_id = generate_channel_id(&*ALICE, &*BOB);

        let db = HoprDb::new_in_memory(ALICE_KP.clone()).await?;
        let db_clone = db.clone();
        db.begin_transaction()
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    db_clone
                        .set_safe_hopr_allowance(Some(tx), Balance::new(10_000_000_u64, BalanceType::HOPR))
                        .await?;
                    db_clone
                        .set_safe_hopr_balance(Some(tx), Balance::new(5_000_000_u64, BalanceType::HOPR))
                        .await?;
                    db_clone.set_network_registry_enabled(Some(tx), false).await?;
                    db_clone
                        .set_domain_separator(Some(tx), DomainSeparator::Channel, Default::default())
                        .await
                })
            })
            .await?;

        let tx_queue = ActionQueue::new(
            db.clone(),
            MockActionState::new(),
            MockTransactionExecutor::new(),
            Default::default(),
        );

        let actions = ChainActions::new(&ALICE_KP, db.clone(), tx_queue.new_sender());
        let stake = Balance::new(10_u32, BalanceType::Native);
        assert!(
            matches!(
                actions
                    .open_channel(*BOB, stake)
                    .await
                    .err()
                    .expect("should be an error"),
                ChainActionsError::InvalidArguments(_)
            ),
            "should not allow invalid balance"
        );

        let stake = Balance::new(0_u32, BalanceType::HOPR);
        assert!(
            matches!(
                actions
                    .fund_channel(channel_id, stake)
                    .await
                    .err()
                    .expect("should be an error"),
                ChainActionsError::InvalidArguments(_)
            ),
            "should not allow invalid balance"
        );
        Ok(())
    }

    #[async_std::test]
    async fn test_should_not_fund_if_not_enough_allowance() -> anyhow::Result<()> {
        let channel_id = generate_channel_id(&*ALICE, &*BOB);

        let db = HoprDb::new_in_memory(ALICE_KP.clone()).await?;
        let db_clone = db.clone();
        db.begin_transaction()
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    db_clone
                        .set_safe_hopr_allowance(Some(tx), Balance::new(1_000_u64, BalanceType::HOPR))
                        .await?;
                    db_clone
                        .set_safe_hopr_balance(Some(tx), Balance::new(5_000_000_u64, BalanceType::HOPR))
                        .await?;
                    db_clone.set_network_registry_enabled(Some(tx), false).await?;
                    db_clone
                        .set_domain_separator(Some(tx), DomainSeparator::Channel, Default::default())
                        .await
                })
            })
            .await?;

        let tx_queue = ActionQueue::new(
            db.clone(),
            MockActionState::new(),
            MockTransactionExecutor::new(),
            Default::default(),
        );

        let actions = ChainActions::new(&ALICE_KP, db.clone(), tx_queue.new_sender());
        let stake = Balance::new(10_000_u32, BalanceType::HOPR);
        assert!(
            matches!(
                actions
                    .fund_channel(channel_id, stake)
                    .await
                    .err()
                    .expect("should be an error"),
                ChainActionsError::NotEnoughAllowance
            ),
            "should fail when not enough allowance"
        );
        Ok(())
    }

    #[async_std::test]
    async fn test_should_not_fund_if_not_enough_balance() -> anyhow::Result<()> {
        let channel_id = generate_channel_id(&*ALICE, &*BOB);

        let db = HoprDb::new_in_memory(ALICE_KP.clone()).await?;
        let db_clone = db.clone();
        db.begin_transaction()
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    db_clone
                        .set_safe_hopr_allowance(Some(tx), Balance::new(100_000_u64, BalanceType::HOPR))
                        .await?;
                    db_clone
                        .set_safe_hopr_balance(Some(tx), Balance::new(1_u64, BalanceType::HOPR))
                        .await?;
                    db_clone.set_network_registry_enabled(Some(tx), false).await?;
                    db_clone
                        .set_domain_separator(Some(tx), DomainSeparator::Channel, Default::default())
                        .await
                })
            })
            .await?;

        let tx_queue = ActionQueue::new(
            db.clone(),
            MockActionState::new(),
            MockTransactionExecutor::new(),
            Default::default(),
        );

        let actions = ChainActions::new(&ALICE_KP, db.clone(), tx_queue.new_sender());
        let stake = Balance::new(10_000_u32, BalanceType::HOPR);
        assert!(
            matches!(
                actions
                    .fund_channel(channel_id, stake)
                    .await
                    .err()
                    .expect("should be an error"),
                ChainActionsError::BalanceTooLow
            ),
            "should fail when not enough balance"
        );
        Ok(())
    }

    #[async_std::test]
    async fn test_close_channel_outgoing() -> anyhow::Result<()> {
        let stake = Balance::new(10_u32, BalanceType::HOPR);
        let random_hash = Hash::from(random_bytes::<{ Hash::SIZE }>());

        let mut channel = ChannelEntry::new(*ALICE, *BOB, stake, U256::zero(), ChannelStatus::Open, U256::zero());

        let db = HoprDb::new_in_memory(ALICE_KP.clone()).await?;
        let db_clone = db.clone();
        db.begin_transaction()
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    db_clone
                        .set_safe_hopr_allowance(Some(tx), Balance::new(1_000_u64, BalanceType::HOPR))
                        .await?;
                    db_clone
                        .set_safe_hopr_balance(Some(tx), Balance::new(5_000_000_u64, BalanceType::HOPR))
                        .await?;
                    db_clone.set_network_registry_enabled(Some(tx), false).await?;
                    db_clone
                        .set_domain_separator(Some(tx), DomainSeparator::Channel, Default::default())
                        .await?;
                    db_clone.upsert_channel(Some(tx), channel).await
                })
            })
            .await?;

        let mut tx_exec = MockTransactionExecutor::new();
        let mut seq = Sequence::new();
        tx_exec
            .expect_initiate_outgoing_channel_closure()
            .times(1)
            .in_sequence(&mut seq)
            .withf(move |dst| BOB.eq(dst))
            .returning(move |_| Ok(random_hash));

        tx_exec
            .expect_finalize_outgoing_channel_closure()
            .times(1)
            .in_sequence(&mut seq)
            .withf(move |dst| BOB.eq(dst))
            .returning(move |_| Ok(random_hash));

        let mut indexer_action_tracker = MockActionState::new();
        let mut seq2 = Sequence::new();
        indexer_action_tracker
            .expect_register_expectation()
            .once()
            .in_sequence(&mut seq2)
            .returning(move |_| {
                Ok(futures::future::ok(SignificantChainEvent {
                    tx_hash: random_hash,
                    event_type: ChainEventType::ChannelClosureInitiated(channel),
                })
                .boxed())
            });

        indexer_action_tracker
            .expect_register_expectation()
            .once()
            .in_sequence(&mut seq2)
            .returning(move |_| {
                Ok(futures::future::ok(SignificantChainEvent {
                    tx_hash: random_hash,
                    event_type: ChainEventType::ChannelClosed(channel),
                })
                .boxed())
            });

        let tx_queue = ActionQueue::new(db.clone(), indexer_action_tracker, tx_exec, Default::default());
        let tx_sender = tx_queue.new_sender();
        async_std::task::spawn(async move {
            tx_queue.start().await;
        });

        let actions = ChainActions::new(&ALICE_KP, db.clone(), tx_sender.clone());

        let tx_res = actions
            .close_channel(*BOB, ChannelDirection::Outgoing, false)
            .await?
            .await?;

        assert_eq!(tx_res.tx_hash, random_hash, "tx hashes must be equal");
        assert!(
            matches!(tx_res.action, Action::CloseChannel(_, _)),
            "must be close channel action"
        );
        assert!(
            matches!(tx_res.event, Some(ChainEventType::ChannelClosureInitiated(_))),
            "must correspond to channel chain event"
        );

        // Transition the channel to the PendingToClose state with the closure time already elapsed
        channel.status = ChannelStatus::PendingToClose(SystemTime::now().sub(Duration::from_secs(10)));

        db.upsert_channel(None, channel).await?;

        let tx_res = actions
            .close_channel(*BOB, ChannelDirection::Outgoing, false)
            .await?
            .await?;

        assert_eq!(tx_res.tx_hash, random_hash, "tx hashes must be equal");
        assert!(
            matches!(tx_res.action, Action::CloseChannel(_, _)),
            "must be close channel action"
        );
        assert!(
            matches!(tx_res.event, Some(ChainEventType::ChannelClosed(_))),
            "must correspond to channel chain event"
        );
        Ok(())
    }

    #[async_std::test]
    async fn test_close_channel_incoming() -> anyhow::Result<()> {
        let stake = Balance::new(10_u32, BalanceType::HOPR);
        let random_hash = Hash::from(random_bytes::<{ Hash::SIZE }>());

        let channel = ChannelEntry::new(*BOB, *ALICE, stake, U256::zero(), ChannelStatus::Open, U256::zero());

        let db = HoprDb::new_in_memory(ALICE_KP.clone()).await?;
        let db_clone = db.clone();
        db.begin_transaction()
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    db_clone
                        .set_safe_hopr_allowance(Some(tx), Balance::new(1_000_u64, BalanceType::HOPR))
                        .await?;
                    db_clone
                        .set_safe_hopr_balance(Some(tx), Balance::new(5_000_000_u64, BalanceType::HOPR))
                        .await?;
                    db_clone.set_network_registry_enabled(Some(tx), false).await?;
                    db_clone
                        .set_domain_separator(Some(tx), DomainSeparator::Channel, Default::default())
                        .await?;
                    db_clone.upsert_channel(Some(tx), channel).await
                })
            })
            .await?;

        let mut tx_exec = MockTransactionExecutor::new();
        let mut seq = Sequence::new();
        tx_exec
            .expect_close_incoming_channel()
            .times(1)
            .in_sequence(&mut seq)
            .withf(move |dst| BOB.eq(dst))
            .returning(move |_| Ok(random_hash));

        let mut indexer_action_tracker = MockActionState::new();
        indexer_action_tracker
            .expect_register_expectation()
            .returning(move |_| {
                Ok(futures::future::ok(SignificantChainEvent {
                    tx_hash: random_hash,
                    event_type: ChainEventType::ChannelClosed(channel),
                })
                .boxed())
            });

        let tx_queue = ActionQueue::new(db.clone(), indexer_action_tracker, tx_exec, Default::default());
        let tx_sender = tx_queue.new_sender();
        async_std::task::spawn(async move {
            tx_queue.start().await;
        });

        let actions = ChainActions::new(&ALICE_KP, db.clone(), tx_sender.clone());

        let tx_res = actions
            .close_channel(*BOB, ChannelDirection::Incoming, false)
            .await?
            .await?;

        assert_eq!(tx_res.tx_hash, random_hash, "tx hashes must be equal");
        assert!(
            matches!(tx_res.action, Action::CloseChannel(_, _)),
            "must be close channel action"
        );
        assert!(
            matches!(tx_res.event, Some(ChainEventType::ChannelClosed(_))),
            "must correspond to channel chain event"
        );
        Ok(())
    }

    #[async_std::test]
    async fn test_should_not_close_when_closure_time_did_not_elapse() -> anyhow::Result<()> {
        let stake = Balance::new(10_u32, BalanceType::HOPR);

        let channel = ChannelEntry::new(
            *ALICE,
            *BOB,
            stake,
            U256::zero(),
            ChannelStatus::PendingToClose(SystemTime::now().add(Duration::from_secs(100))),
            U256::zero(),
        );

        let db = HoprDb::new_in_memory(ALICE_KP.clone()).await?;
        let db_clone = db.clone();
        db.begin_transaction()
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    db_clone
                        .set_safe_hopr_allowance(Some(tx), Balance::new(1_000_u64, BalanceType::HOPR))
                        .await?;
                    db_clone
                        .set_safe_hopr_balance(Some(tx), Balance::new(5_000_000_u64, BalanceType::HOPR))
                        .await?;
                    db_clone.set_network_registry_enabled(Some(tx), false).await?;
                    db_clone
                        .set_domain_separator(Some(tx), DomainSeparator::Channel, Default::default())
                        .await?;
                    db_clone.upsert_channel(Some(tx), channel).await
                })
            })
            .await?;

        let tx_queue = ActionQueue::new(
            db.clone(),
            MockActionState::new(),
            MockTransactionExecutor::new(),
            Default::default(),
        );

        let actions = ChainActions::new(&ALICE_KP, db.clone(), tx_queue.new_sender());

        assert!(
            matches!(
                actions
                    .close_channel(*BOB, ChannelDirection::Outgoing, false)
                    .await
                    .err()
                    .expect("should be an error"),
                ChainActionsError::ClosureTimeHasNotElapsed(_)
            ),
            "should fail when the channel closure period did not elapse"
        );
        Ok(())
    }

    #[async_std::test]
    async fn test_should_not_close_nonexistent_channel() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(ALICE_KP.clone()).await?;
        let db_clone = db.clone();
        db.begin_transaction()
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    db_clone
                        .set_safe_hopr_allowance(Some(tx), Balance::new(1_000_u64, BalanceType::HOPR))
                        .await?;
                    db_clone
                        .set_safe_hopr_balance(Some(tx), Balance::new(5_000_000_u64, BalanceType::HOPR))
                        .await?;
                    db_clone.set_network_registry_enabled(Some(tx), false).await?;
                    db_clone
                        .set_domain_separator(Some(tx), DomainSeparator::Channel, Default::default())
                        .await
                })
            })
            .await?;

        let tx_queue = ActionQueue::new(
            db.clone(),
            MockActionState::new(),
            MockTransactionExecutor::new(),
            Default::default(),
        );
        let actions = ChainActions::new(&ALICE_KP, db.clone(), tx_queue.new_sender());

        assert!(
            matches!(
                actions
                    .close_channel(*BOB, ChannelDirection::Outgoing, false)
                    .await
                    .err()
                    .expect("should be an error"),
                ChainActionsError::ChannelDoesNotExist
            ),
            "should fail when channel does not exist"
        );
        Ok(())
    }

    #[async_std::test]
    async fn test_should_not_close_closed_channel() -> anyhow::Result<()> {
        let stake = Balance::new(10_u32, BalanceType::HOPR);
        let channel = ChannelEntry::new(*ALICE, *BOB, stake, U256::zero(), ChannelStatus::Closed, U256::zero());

        let db = HoprDb::new_in_memory(ALICE_KP.clone()).await?;
        let db_clone = db.clone();
        db.begin_transaction()
            .await?
            .perform(|tx| {
                Box::pin(async move {
                    db_clone
                        .set_safe_hopr_allowance(Some(tx), Balance::new(1_000_u64, BalanceType::HOPR))
                        .await?;
                    db_clone
                        .set_safe_hopr_balance(Some(tx), Balance::new(5_000_000_u64, BalanceType::HOPR))
                        .await?;
                    db_clone.set_network_registry_enabled(Some(tx), false).await?;
                    db_clone
                        .set_domain_separator(Some(tx), DomainSeparator::Channel, Default::default())
                        .await?;
                    db_clone.upsert_channel(Some(tx), channel).await
                })
            })
            .await?;

        let tx_queue = ActionQueue::new(
            db.clone(),
            MockActionState::new(),
            MockTransactionExecutor::new(),
            Default::default(),
        );

        let actions = ChainActions::new(&ALICE_KP, db.clone(), tx_queue.new_sender());

        assert!(
            matches!(
                actions
                    .close_channel(*BOB, ChannelDirection::Outgoing, false)
                    .await
                    .err()
                    .expect("should be an error"),
                ChainActionsError::ChannelAlreadyClosed
            ),
            "should fail when channel is already closed"
        );
        Ok(())
    }
}
