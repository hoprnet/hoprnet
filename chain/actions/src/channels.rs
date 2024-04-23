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
//! See the details in [ActionQueue](crate::action_queue::ActionQueue) on how the confirmation is realized by awaiting the respective [SignificantChainEvent](chain_types::chain_events::SignificantChainEvent)
//! by the Indexer.
use async_trait::async_trait;
use chain_types::actions::Action;
use hopr_crypto_types::types::Hash;
use hopr_db_api::HoprDbAllOperations;
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::*;
use std::time::Duration;
use tracing::{debug, error, info};

use crate::action_queue::PendingAction;
use crate::errors::ChainActionsError::{
    BalanceTooLow, ClosureTimeHasNotElapsed, InvalidArguments, InvalidState, NotEnoughAllowance, PeerAccessDenied,
};
use crate::errors::{
    ChainActionsError::{ChannelAlreadyClosed, ChannelAlreadyExists, ChannelDoesNotExist},
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
                    debug!("current staking safe allowance is {allowance}");
                    if allowance.lt(&amount) {
                        return Err(NotEnoughAllowance);
                    }

                    let hopr_balance = db_clone.get_safe_hopr_balance(Some(tx)).await?;
                    debug!("current Safe HOPR balance is {hopr_balance}");
                    if hopr_balance.lt(&amount) {
                        return Err(BalanceTooLow);
                    }

                    if db_clone.get_indexer_data(Some(tx)).await?.nr_enabled
                        && !db_clone.is_allowed_in_network_registry(Some(tx), destination).await?
                    {
                        return Err(PeerAccessDenied);
                    }

                    let maybe_channel = db_clone
                        .get_channel_by_parties(Some(tx), &self_addr, &destination)
                        .await?;
                    if let Some(channel) = maybe_channel {
                        debug!("already found existing {channel}");
                        if channel.status != ChannelStatus::Closed {
                            error!("channel to {destination} is already opened or pending to close");
                            return Err(ChannelAlreadyExists);
                        }
                    }
                    Ok(())
                })
            })
            .await?;

        info!("initiating channel open to {destination} with {amount}");
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
                    debug!("current staking safe allowance is {allowance}");
                    if allowance.lt(&amount) {
                        return Err(NotEnoughAllowance);
                    }

                    let hopr_balance = db_clone.get_safe_hopr_balance(Some(tx)).await?;
                    debug!("current Safe HOPR balance is {hopr_balance}");
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
                    .get_channel_by_parties(None, &counterparty, &self.self_address())
                    .await?
            }
            ChannelDirection::Outgoing => {
                self.db
                    .get_channel_by_parties(None, &self.self_address(), &counterparty)
                    .await?
            }
        };

        match maybe_channel {
            Some(channel) => {
                match channel.status {
                    ChannelStatus::Closed => Err(ChannelAlreadyClosed),
                    ChannelStatus::PendingToClose(_) => {
                        let remaining_closure_time = channel.remaining_closure_time(current_time());
                        info!("{channel} - remaining closure time is {remaining_closure_time:?}");
                        match remaining_closure_time {
                            Some(Duration::ZERO) => {
                                info!("initiating finalization of channel closure of {channel} in {direction}");
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
                            info!("{redeemed} tickets will be redeemed before closing {channel}");
                        }

                        info!("initiating channel closure of {channel} in {direction}");
                        self.tx_sender.send(Action::CloseChannel(channel, direction)).await
                    }
                }
            }
            None => Err(ChannelDoesNotExist),
        }
    }
}
#[cfg(test)]
mod tests {
    use crate::action_queue::{ActionQueue, MockTransactionExecutor};
    use crate::action_state::MockActionState;
    use crate::channels::ChannelActions;
    use crate::errors::ChainActionsError;
    use crate::ChainActions;
    use chain_types::actions::Action;
    use chain_types::chain_events::{ChainEventType, SignificantChainEvent};
    use futures::FutureExt;
    use hex_literal::hex;
    use hopr_crypto_random::random_bytes;
    use hopr_crypto_types::prelude::*;
    use hopr_db_api::channels::HoprDbChannelOperations;
    use hopr_db_api::db::HoprDb;
    use hopr_db_api::info::{DomainSeparator, HoprDbInfoOperations};
    use hopr_db_api::HoprDbGeneralModelOperations;
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
        .unwrap();
        static ref BOB_KP: ChainKeypair = ChainKeypair::from_secret(&hex!(
            "48680484c6fc31bc881a0083e6e32b6dc789f9eaba0f8b981429fd346c697f8c"
        ))
        .unwrap();
        static ref ALICE: Address = ALICE_KP.public().to_address();
        static ref BOB: Address = BOB_KP.public().to_address();
    }

    #[async_std::test]
    async fn test_open_channel() {
        let _ = env_logger::builder().is_test(true).try_init();

        let stake = Balance::new(10_u32, BalanceType::HOPR);
        let random_hash = Hash::from(random_bytes::<{ Hash::SIZE }>());

        let db = HoprDb::new_in_memory(ALICE_KP.clone()).await;
        let db_clone = db.clone();
        db.begin_transaction()
            .await
            .unwrap()
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
            .await
            .expect("must initialize db");

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
        async_std::task::spawn(async move {
            tx_queue.action_loop().await;
        });

        let actions = ChainActions::new(&ALICE_KP, db.clone(), tx_sender.clone());

        let tx_res = actions
            .open_channel(*BOB, stake)
            .await
            .unwrap()
            .await
            .expect("must resolve confirmation");

        assert_eq!(tx_res.tx_hash, random_hash, "tx hashes must be equal");
        assert!(
            matches!(tx_res.action, Action::OpenChannel(_, _)),
            "must be open channel action"
        );
        assert!(
            matches!(tx_res.event, Some(ChainEventType::ChannelOpened(_))),
            "must correspond to open channel chain event"
        );
    }

    #[async_std::test]
    async fn test_should_not_open_channel_again() {
        let _ = env_logger::builder().is_test(true).try_init();
        let stake = Balance::new(10_u32, BalanceType::HOPR);

        let channel = ChannelEntry::new(*ALICE, *BOB, stake, U256::zero(), ChannelStatus::Open, U256::zero());

        let db = HoprDb::new_in_memory(ALICE_KP.clone()).await;
        let db_clone = db.clone();
        db.begin_transaction()
            .await
            .unwrap()
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
            .await
            .expect("must initialize db");

        let tx_queue = ActionQueue::new(
            db.clone(),
            MockActionState::new(),
            MockTransactionExecutor::new(),
            Default::default(),
        );

        let actions = ChainActions::new(&ALICE_KP, db.clone(), tx_queue.new_sender());

        assert!(
            matches!(
                actions.open_channel(*BOB, stake).await.err().unwrap(),
                ChainActionsError::ChannelAlreadyExists
            ),
            "should fail when channel exists"
        );
    }

    #[async_std::test]
    async fn test_should_not_open_channel_to_self() {
        let _ = env_logger::builder().is_test(true).try_init();

        let stake = Balance::new(10_u32, BalanceType::HOPR);

        let db = HoprDb::new_in_memory(ALICE_KP.clone()).await;
        let db_clone = db.clone();
        db.begin_transaction()
            .await
            .unwrap()
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
            .await
            .expect("must initialize db");

        let tx_queue = ActionQueue::new(
            db.clone(),
            MockActionState::new(),
            MockTransactionExecutor::new(),
            Default::default(),
        );

        let actions = ChainActions::new(&ALICE_KP, db.clone(), tx_queue.new_sender());

        assert!(
            matches!(
                actions.open_channel(*ALICE, stake).await.err().unwrap(),
                ChainActionsError::InvalidArguments(_)
            ),
            "should not create channel to self"
        );
    }

    #[async_std::test]
    async fn test_open_should_not_allow_invalid_balance() {
        let _ = env_logger::builder().is_test(true).try_init();

        let db = HoprDb::new_in_memory(ALICE_KP.clone()).await;
        let db_clone = db.clone();
        db.begin_transaction()
            .await
            .unwrap()
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
            .await
            .expect("must initialize db");

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
                actions.open_channel(*BOB, stake).await.err().unwrap(),
                ChainActionsError::InvalidArguments(_)
            ),
            "should not allow invalid balance"
        );

        let stake = Balance::new(0_u32, BalanceType::HOPR);

        assert!(
            matches!(
                actions.open_channel(*BOB, stake).await.err().unwrap(),
                ChainActionsError::InvalidArguments(_)
            ),
            "should not allow invalid balance"
        );
    }

    #[async_std::test]
    async fn test_should_not_open_if_not_enough_allowance() {
        let _ = env_logger::builder().is_test(true).try_init();

        let stake = Balance::new(10_000_u32, BalanceType::HOPR);

        let db = HoprDb::new_in_memory(ALICE_KP.clone()).await;
        let db_clone = db.clone();
        db.begin_transaction()
            .await
            .unwrap()
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
            .await
            .expect("must initialize db");

        let tx_queue = ActionQueue::new(
            db.clone(),
            MockActionState::new(),
            MockTransactionExecutor::new(),
            Default::default(),
        );

        let actions = ChainActions::new(&ALICE_KP, db.clone(), tx_queue.new_sender());

        assert!(
            matches!(
                actions.open_channel(*BOB, stake).await.err().unwrap(),
                ChainActionsError::NotEnoughAllowance
            ),
            "should fail when not enough allowance"
        );
    }

    #[async_std::test]
    async fn test_should_not_open_if_not_enough_token_balance() {
        let _ = env_logger::builder().is_test(true).try_init();

        let stake = Balance::new(10_000_u32, BalanceType::HOPR);

        let db = HoprDb::new_in_memory(ALICE_KP.clone()).await;
        let db_clone = db.clone();
        db.begin_transaction()
            .await
            .unwrap()
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
            .await
            .expect("must initialize db");

        let tx_queue = ActionQueue::new(
            db.clone(),
            MockActionState::new(),
            MockTransactionExecutor::new(),
            Default::default(),
        );

        let actions = ChainActions::new(&ALICE_KP, db.clone(), tx_queue.new_sender());

        assert!(
            matches!(
                actions.open_channel(*BOB, stake).await.err().unwrap(),
                ChainActionsError::BalanceTooLow
            ),
            "should fail when not enough token balance"
        );
    }

    #[async_std::test]
    async fn test_fund_channel() {
        let _ = env_logger::builder().is_test(true).try_init();

        let stake = Balance::new(10_u32, BalanceType::HOPR);
        let random_hash = Hash::from(random_bytes::<{ Hash::SIZE }>());
        let channel = ChannelEntry::new(*ALICE, *BOB, stake, U256::zero(), ChannelStatus::Open, U256::zero());

        let db = HoprDb::new_in_memory(ALICE_KP.clone()).await;
        let db_clone = db.clone();
        db.begin_transaction()
            .await
            .unwrap()
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
            .await
            .expect("must initialize db");

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
            tx_queue.action_loop().await;
        });

        let actions = ChainActions::new(&ALICE_KP, db.clone(), tx_sender.clone());

        let tx_res = actions
            .fund_channel(channel.get_id(), stake)
            .await
            .unwrap()
            .await
            .expect("must resolve confirmation");

        assert_eq!(tx_res.tx_hash, random_hash, "tx hashes must be equal");
        assert!(
            matches!(tx_res.action, Action::FundChannel(_, _)),
            "must be open channel action"
        );
        assert!(
            matches!(tx_res.event, Some(ChainEventType::ChannelBalanceIncreased(_, _))),
            "must correspond to channel chain event"
        );
    }

    #[async_std::test]
    async fn test_should_not_fund_nonexistent_channel() {
        let _ = env_logger::builder().is_test(true).try_init();

        let channel_id = generate_channel_id(&*ALICE, &*BOB);

        let db = HoprDb::new_in_memory(ALICE_KP.clone()).await;
        let db_clone = db.clone();
        db.begin_transaction()
            .await
            .unwrap()
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
            .await
            .expect("must initialize db");

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
                actions.fund_channel(channel_id, stake).await.err().unwrap(),
                ChainActionsError::ChannelDoesNotExist
            ),
            "should fail when channel does not exist"
        );
    }

    #[async_std::test]
    async fn test_fund_should_not_allow_invalid_balance() {
        let _ = env_logger::builder().is_test(true).try_init();

        let channel_id = generate_channel_id(&*ALICE, &*BOB);

        let db = HoprDb::new_in_memory(ALICE_KP.clone()).await;
        let db_clone = db.clone();
        db.begin_transaction()
            .await
            .unwrap()
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
            .await
            .expect("must initialize db");

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
                actions.open_channel(*BOB, stake).await.err().unwrap(),
                ChainActionsError::InvalidArguments(_)
            ),
            "should not allow invalid balance"
        );

        let stake = Balance::new(0_u32, BalanceType::HOPR);
        assert!(
            matches!(
                actions.fund_channel(channel_id, stake).await.err().unwrap(),
                ChainActionsError::InvalidArguments(_)
            ),
            "should not allow invalid balance"
        );
    }

    #[async_std::test]
    async fn test_should_not_fund_if_not_enough_allowance() {
        let _ = env_logger::builder().is_test(true).try_init();

        let channel_id = generate_channel_id(&*ALICE, &*BOB);

        let db = HoprDb::new_in_memory(ALICE_KP.clone()).await;
        let db_clone = db.clone();
        db.begin_transaction()
            .await
            .unwrap()
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
            .await
            .expect("must initialize db");

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
                actions.fund_channel(channel_id, stake).await.err().unwrap(),
                ChainActionsError::NotEnoughAllowance
            ),
            "should fail when not enough allowance"
        );
    }

    #[async_std::test]
    async fn test_should_not_fund_if_not_enough_balance() {
        let _ = env_logger::builder().is_test(true).try_init();

        let channel_id = generate_channel_id(&*ALICE, &*BOB);

        let db = HoprDb::new_in_memory(ALICE_KP.clone()).await;
        let db_clone = db.clone();
        db.begin_transaction()
            .await
            .unwrap()
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
            .await
            .expect("must initialize db");

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
                actions.fund_channel(channel_id, stake).await.err().unwrap(),
                ChainActionsError::BalanceTooLow
            ),
            "should fail when not enough balance"
        );
    }

    #[async_std::test]
    async fn test_close_channel_outgoing() {
        let _ = env_logger::builder().is_test(true).try_init();

        let stake = Balance::new(10_u32, BalanceType::HOPR);
        let random_hash = Hash::from(random_bytes::<{ Hash::SIZE }>());

        let mut channel = ChannelEntry::new(*ALICE, *BOB, stake, U256::zero(), ChannelStatus::Open, U256::zero());

        let db = HoprDb::new_in_memory(ALICE_KP.clone()).await;
        let db_clone = db.clone();
        db.begin_transaction()
            .await
            .unwrap()
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
            .await
            .expect("must initialize db");

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
            tx_queue.action_loop().await;
        });

        let actions = ChainActions::new(&ALICE_KP, db.clone(), tx_sender.clone());

        let tx_res = actions
            .close_channel(*BOB, ChannelDirection::Outgoing, false)
            .await
            .unwrap()
            .await
            .expect("must resolve confirmation");

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

        db.upsert_channel(None, channel).await.unwrap();

        let tx_res = actions
            .close_channel(*BOB, ChannelDirection::Outgoing, false)
            .await
            .unwrap()
            .await
            .expect("must resolve confirmation");

        assert_eq!(tx_res.tx_hash, random_hash, "tx hashes must be equal");
        assert!(
            matches!(tx_res.action, Action::CloseChannel(_, _)),
            "must be close channel action"
        );
        assert!(
            matches!(tx_res.event, Some(ChainEventType::ChannelClosed(_))),
            "must correspond to channel chain event"
        );
    }

    #[async_std::test]
    async fn test_close_channel_incoming() {
        let stake = Balance::new(10_u32, BalanceType::HOPR);
        let random_hash = Hash::from(random_bytes::<{ Hash::SIZE }>());

        let channel = ChannelEntry::new(*BOB, *ALICE, stake, U256::zero(), ChannelStatus::Open, U256::zero());

        let db = HoprDb::new_in_memory(ALICE_KP.clone()).await;
        let db_clone = db.clone();
        db.begin_transaction()
            .await
            .unwrap()
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
            .await
            .expect("must initialize db");

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
            tx_queue.action_loop().await;
        });

        let actions = ChainActions::new(&ALICE_KP, db.clone(), tx_sender.clone());

        let tx_res = actions
            .close_channel(*BOB, ChannelDirection::Incoming, false)
            .await
            .unwrap()
            .await
            .expect("must resolve confirmation");

        assert_eq!(tx_res.tx_hash, random_hash, "tx hashes must be equal");
        assert!(
            matches!(tx_res.action, Action::CloseChannel(_, _)),
            "must be close channel action"
        );
        assert!(
            matches!(tx_res.event, Some(ChainEventType::ChannelClosed(_))),
            "must correspond to channel chain event"
        );
    }

    #[async_std::test]
    async fn test_should_not_close_when_closure_time_did_not_elapse() {
        let stake = Balance::new(10_u32, BalanceType::HOPR);

        let channel = ChannelEntry::new(
            *ALICE,
            *BOB,
            stake,
            U256::zero(),
            ChannelStatus::PendingToClose(SystemTime::now().add(Duration::from_secs(100))),
            U256::zero(),
        );

        let db = HoprDb::new_in_memory(ALICE_KP.clone()).await;
        let db_clone = db.clone();
        db.begin_transaction()
            .await
            .unwrap()
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
            .await
            .expect("must initialize db");

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
                    .unwrap(),
                ChainActionsError::ClosureTimeHasNotElapsed(_)
            ),
            "should fail when the channel closure period did not elapse"
        );
    }

    #[async_std::test]
    async fn test_should_not_close_nonexistent_channel() {
        let _ = env_logger::builder().is_test(true).try_init();

        let db = HoprDb::new_in_memory(ALICE_KP.clone()).await;
        let db_clone = db.clone();
        db.begin_transaction()
            .await
            .unwrap()
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
            .await
            .expect("must initialize db");

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
                    .unwrap(),
                ChainActionsError::ChannelDoesNotExist
            ),
            "should fail when channel does not exist"
        );
    }

    #[async_std::test]
    async fn test_should_not_close_closed_channel() {
        let _ = env_logger::builder().is_test(true).try_init();

        let stake = Balance::new(10_u32, BalanceType::HOPR);
        let channel = ChannelEntry::new(*ALICE, *BOB, stake, U256::zero(), ChannelStatus::Closed, U256::zero());

        let db = HoprDb::new_in_memory(ALICE_KP.clone()).await;
        let db_clone = db.clone();
        db.begin_transaction()
            .await
            .unwrap()
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
            .await
            .expect("must initialize db");

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
                    .unwrap(),
                ChainActionsError::ChannelAlreadyClosed
            ),
            "should fail when channel is already closed"
        );
    }
}
