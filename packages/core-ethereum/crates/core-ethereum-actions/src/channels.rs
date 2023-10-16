use async_lock::RwLock;
use core_crypto::types::Hash;
use core_ethereum_db::traits::HoprCoreEthereumDbActions;
use core_ethereum_misc::errors::CoreEthereumError::{InvalidArguments, InvalidState};
use core_types::channels::{ChannelDirection, ChannelStatus};
use std::sync::Arc;
use utils_log::{debug, error, info};
use utils_types::primitives::{Address, Balance, BalanceType};

use crate::{
    errors::{
        CoreEthereumActionsError::{
            ChannelAlreadyClosed, ChannelAlreadyExists, ChannelDoesNotExist, ClosureTimeHasNotElapsed,
            NotEnoughAllowance, PeerAccessDenied,
        },
        Result,
    },
    redeem::redeem_tickets_in_channel,
    transaction_queue::{Transaction, TransactionCompleted, TransactionSender},
};

#[cfg(all(feature = "wasm", not(test)))]
use utils_misc::time::wasm::current_timestamp;

#[cfg(any(not(feature = "wasm"), test))]
use utils_misc::time::native::current_timestamp;

pub async fn open_channel<Db>(
    db: Arc<RwLock<Db>>,
    tx_sender: TransactionSender,
    destination: Address,
    self_addr: Address,
    amount: Balance,
) -> Result<TransactionCompleted>
where
    Db: HoprCoreEthereumDbActions,
{
    if self_addr == destination {
        return Err(InvalidArguments("cannot open channel to self".into()).into());
    }

    if amount.eq(&amount.of_same("0")) || amount.balance_type() != BalanceType::HOPR {
        return Err(InvalidArguments("invalid balance or balance type given".into()).into());
    }

    let allowance = db.read().await.get_staking_safe_allowance().await?;
    debug!("current staking safe allowance is {allowance}");
    if allowance.lt(&amount) {
        return Err(NotEnoughAllowance);
    }

    if db.read().await.is_network_registry_enabled().await?
        && !db.read().await.is_allowed_to_access_network(&destination).await?
    {
        return Err(PeerAccessDenied);
    }

    let maybe_channel = db.read().await.get_channel_x(&self_addr, &destination).await?;
    if let Some(channel) = maybe_channel {
        debug!("already found existing {channel}");
        if channel.status != ChannelStatus::Closed {
            error!("channel to {destination} is already opened or pending to close");
            return Err(ChannelAlreadyExists);
        }
    }

    tx_sender.send(Transaction::OpenChannel(destination, amount)).await
}

pub async fn fund_channel<Db>(
    db: Arc<RwLock<Db>>,
    tx_sender: TransactionSender,
    channel_id: Hash,
    amount: Balance,
) -> Result<TransactionCompleted>
where
    Db: HoprCoreEthereumDbActions,
{
    if amount.eq(&amount.of_same("0")) || amount.balance_type() != BalanceType::HOPR {
        return Err(InvalidArguments("invalid balance or balance type given".into()).into());
    }

    let allowance = db.read().await.get_staking_safe_allowance().await?;
    debug!("current staking safe allowance is {allowance}");
    if allowance.lt(&amount) {
        return Err(NotEnoughAllowance);
    }

    let maybe_channel = db.read().await.get_channel(&channel_id).await?;
    match maybe_channel {
        Some(channel) => {
            if channel.status == ChannelStatus::Open {
                tx_sender.send(Transaction::FundChannel(channel, amount)).await
            } else {
                Err(InvalidState(format!("channel {channel_id} is not opened")).into())
            }
        }
        None => Err(ChannelDoesNotExist),
    }
}

pub async fn close_channel<Db>(
    db: Arc<RwLock<Db>>,
    tx_sender: TransactionSender,
    counterparty: Address,
    self_address: Address,
    direction: ChannelDirection,
    redeem_before_close: bool,
) -> Result<TransactionCompleted>
where
    Db: HoprCoreEthereumDbActions,
{
    let maybe_channel = match direction {
        ChannelDirection::Incoming => db.read().await.get_channel_x(&counterparty, &self_address).await?,
        ChannelDirection::Outgoing => db.read().await.get_channel_x(&self_address, &counterparty).await?,
    };

    match maybe_channel {
        Some(channel) => {
            match channel.status {
                ChannelStatus::Closed => Err(ChannelAlreadyClosed),
                ChannelStatus::PendingToClose => {
                    info!(
                        "{channel} - remaining closure time is {:?}",
                        channel.remaining_closure_time(current_timestamp())
                    );
                    if channel.closure_time_passed(current_timestamp()).unwrap_or(false) {
                        // TODO: emit "channel state change" event
                        tx_sender.send(Transaction::CloseChannel(channel, direction)).await
                    } else {
                        Err(ClosureTimeHasNotElapsed(
                            channel
                                .remaining_closure_time(current_timestamp())
                                .unwrap_or(u32::MAX as u64),
                        ))
                    }
                }
                ChannelStatus::Open => {
                    if redeem_before_close {
                        // TODO: trigger aggregation
                        // Do not await the redemption, just submit it to the queue
                        let redeemed = redeem_tickets_in_channel(db.clone(), &channel, false, tx_sender.clone())
                            .await?
                            .len();
                        info!("{redeemed} tickets will be redeemed before closing {channel}");
                    }

                    tx_sender.send(Transaction::CloseChannel(channel, direction)).await
                    // TODO: emit "channel state change" event
                }
            }
        }
        None => Err(ChannelDoesNotExist),
    }
}

pub async fn withdraw(
    tx_sender: TransactionSender,
    recipient: Address,
    amount: Balance,
) -> Result<TransactionCompleted> {
    if amount.eq(&amount.of_same("0")) {
        return Err(InvalidArguments("cannot withdraw zero amount".into()).into());
    }

    tx_sender.send(Transaction::Withdraw(recipient, amount)).await
}

#[cfg(test)]
mod tests {
    use crate::{
        channels::{close_channel, fund_channel, open_channel, withdraw},
        errors::CoreEthereumActionsError,
        transaction_queue::{MockTransactionExecutor, TransactionQueue, TransactionResult},
    };
    use async_lock::RwLock;
    use core_crypto::{random::random_bytes, types::Hash};
    use core_ethereum_db::{db::CoreEthereumDb, traits::HoprCoreEthereumDbActions};
    use core_types::channels::{generate_channel_id, ChannelDirection, ChannelEntry, ChannelStatus};
    use hex_literal::hex;
    use lazy_static::lazy_static;
    use mockall::Sequence;
    use std::{
        ops::{Add, Sub},
        sync::Arc,
        time::{Duration, SystemTime, UNIX_EPOCH},
    };
    use utils_db::{db::DB, rusty::RustyLevelDbShim};
    use utils_types::{
        primitives::{Address, Balance, BalanceType, Snapshot, U256},
        traits::BinarySerializable,
    };

    lazy_static! {
        static ref ALICE: Address = Address::from(hex!("86fa27add61fafc955e2da17329bba9f31692fe7"));
        static ref BOB: Address = Address::from(hex!("4c8bbd047c2130e702badb23b6b97a88b6562324"));
    }

    #[async_std::test]
    async fn test_open_channel() {
        let _ = env_logger::builder().is_test(true).try_init();

        let stake = Balance::new(10_u32.into(), BalanceType::HOPR);
        let random_hash = Hash::new(&random_bytes::<{ Hash::SIZE }>());

        let db = Arc::new(RwLock::new(CoreEthereumDb::new(
            DB::new(RustyLevelDbShim::new_in_memory()),
            *ALICE,
        )));
        db.write()
            .await
            .set_staking_safe_allowance(
                &Balance::new(10_000_000u64.into(), BalanceType::HOPR),
                &Snapshot::default(),
            )
            .await
            .unwrap();

        db.write()
            .await
            .set_network_registry(false, &Snapshot::default())
            .await
            .unwrap();

        let mut tx_exec = MockTransactionExecutor::new();
        tx_exec
            .expect_fund_channel()
            .times(1)
            .withf(move |dst, balance| BOB.eq(dst) && stake.eq(balance))
            .returning(move |_, _| TransactionResult::ChannelFunded { tx_hash: random_hash });

        let tx_queue = TransactionQueue::new(db.clone(), Box::new(tx_exec));
        let tx_sender = tx_queue.new_sender();
        async_std::task::spawn_local(async move {
            tx_queue.transaction_loop().await;
        });

        let tx_res = open_channel(db.clone(), tx_sender.clone(), *BOB, *ALICE, stake)
            .await
            .unwrap()
            .await
            .unwrap();

        match tx_res {
            TransactionResult::ChannelFunded { tx_hash } => {
                assert_eq!(random_hash, tx_hash, "tx hash must be equal");
            }
            _ => panic!("invalid or failed tx result"),
        }
    }

    #[async_std::test]
    async fn test_should_not_open_channel_again() {
        let _ = env_logger::builder().is_test(true).try_init();
        let stake = Balance::new(10_u32.into(), BalanceType::HOPR);

        let db = Arc::new(RwLock::new(CoreEthereumDb::new(
            DB::new(RustyLevelDbShim::new_in_memory()),
            *ALICE,
        )));
        let tx_queue = TransactionQueue::new(db.clone(), Box::new(MockTransactionExecutor::new()));

        let channel = ChannelEntry::new(
            *ALICE,
            *BOB,
            stake,
            U256::zero(),
            ChannelStatus::Open,
            U256::zero(),
            U256::zero(),
        );

        db.write()
            .await
            .set_staking_safe_allowance(
                &Balance::new(10_000_000u64.into(), BalanceType::HOPR),
                &Snapshot::default(),
            )
            .await
            .unwrap();
        db.write()
            .await
            .update_channel_and_snapshot(&channel.get_id(), &channel, &Snapshot::default())
            .await
            .unwrap();

        db.write()
            .await
            .set_network_registry(false, &Snapshot::default())
            .await
            .unwrap();

        assert!(
            matches!(
                open_channel(db.clone(), tx_queue.new_sender(), *BOB, *ALICE, stake)
                    .await
                    .err()
                    .unwrap(),
                CoreEthereumActionsError::ChannelAlreadyExists
            ),
            "should fail when channel exists"
        );
    }

    #[async_std::test]
    async fn test_should_not_open_channel_to_self() {
        let _ = env_logger::builder().is_test(true).try_init();

        let stake = Balance::new(10_u32.into(), BalanceType::HOPR);

        let db = Arc::new(RwLock::new(CoreEthereumDb::new(
            DB::new(RustyLevelDbShim::new_in_memory()),
            *ALICE,
        )));
        let tx_queue = TransactionQueue::new(db.clone(), Box::new(MockTransactionExecutor::new()));

        db.write()
            .await
            .set_staking_safe_allowance(
                &Balance::new(10_000_000u64.into(), BalanceType::HOPR),
                &Snapshot::default(),
            )
            .await
            .unwrap();

        assert!(
            matches!(
                open_channel(db.clone(), tx_queue.new_sender(), *ALICE, *ALICE, stake)
                    .await
                    .err()
                    .unwrap(),
                CoreEthereumActionsError::OtherError(_)
            ),
            "should not create channel to self"
        );
    }

    #[async_std::test]
    async fn test_open_should_not_allow_invalid_balance() {
        let _ = env_logger::builder().is_test(true).try_init();

        let self_addr = Address::random();
        let bob = Address::random();

        let db = Arc::new(RwLock::new(CoreEthereumDb::new(
            DB::new(RustyLevelDbShim::new_in_memory()),
            self_addr,
        )));
        let tx_queue = TransactionQueue::new(db.clone(), Box::new(MockTransactionExecutor::new()));

        db.write()
            .await
            .set_staking_safe_allowance(
                &Balance::new(10_000_000u64.into(), BalanceType::HOPR),
                &Snapshot::default(),
            )
            .await
            .unwrap();

        let stake = Balance::new(10_u32.into(), BalanceType::Native);
        assert!(
            matches!(
                open_channel(db.clone(), tx_queue.new_sender(), self_addr, bob, stake)
                    .await
                    .err()
                    .unwrap(),
                CoreEthereumActionsError::OtherError(_)
            ),
            "should not allow invalid balance"
        );

        let stake = Balance::new(0_u32.into(), BalanceType::HOPR);
        assert!(
            matches!(
                open_channel(db.clone(), tx_queue.new_sender(), self_addr, bob, stake)
                    .await
                    .err()
                    .unwrap(),
                CoreEthereumActionsError::OtherError(_)
            ),
            "should not allow invalid balance"
        );
    }

    #[async_std::test]
    async fn test_should_not_open_if_not_enough_allowance() {
        let _ = env_logger::builder().is_test(true).try_init();

        let self_addr = Address::random();
        let bob = Address::random();
        let stake = Balance::new(10_000_u32.into(), BalanceType::HOPR);

        let db = Arc::new(RwLock::new(CoreEthereumDb::new(
            DB::new(RustyLevelDbShim::new_in_memory()),
            self_addr,
        )));
        let tx_queue = TransactionQueue::new(db.clone(), Box::new(MockTransactionExecutor::new()));

        db.write()
            .await
            .set_staking_safe_allowance(&Balance::new(1000_u64.into(), BalanceType::HOPR), &Snapshot::default())
            .await
            .unwrap();

        assert!(
            matches!(
                open_channel(db.clone(), tx_queue.new_sender(), bob, self_addr, stake)
                    .await
                    .err()
                    .unwrap(),
                CoreEthereumActionsError::NotEnoughAllowance
            ),
            "should fail when not enough allowance"
        );
    }

    #[async_std::test]
    async fn test_fund_channel() {
        let _ = env_logger::builder().is_test(true).try_init();

        let self_addr = Address::random();
        let bob = Address::random();
        let stake = Balance::new(10_u32.into(), BalanceType::HOPR);
        let random_hash = Hash::new(&random_bytes::<{ Hash::SIZE }>());

        let db = Arc::new(RwLock::new(CoreEthereumDb::new(
            DB::new(RustyLevelDbShim::new_in_memory()),
            self_addr,
        )));
        db.write()
            .await
            .set_staking_safe_allowance(
                &Balance::new(10_000_000u64.into(), BalanceType::HOPR),
                &Snapshot::default(),
            )
            .await
            .unwrap();

        let channel = ChannelEntry::new(
            self_addr,
            bob,
            stake,
            U256::zero(),
            ChannelStatus::Open,
            U256::zero(),
            U256::zero(),
        );
        db.write()
            .await
            .update_channel_and_snapshot(&channel.get_id(), &channel, &Snapshot::default())
            .await
            .unwrap();

        let mut tx_exec = MockTransactionExecutor::new();
        tx_exec
            .expect_fund_channel()
            .times(1)
            .withf(move |dest, balance| channel.destination.eq(&dest) && stake.eq(balance))
            .returning(move |_, _| TransactionResult::ChannelFunded { tx_hash: random_hash });

        let tx_queue = TransactionQueue::new(db.clone(), Box::new(tx_exec));
        let tx_sender = tx_queue.new_sender();
        async_std::task::spawn_local(async move {
            tx_queue.transaction_loop().await;
        });

        let tx_res = fund_channel(db.clone(), tx_sender.clone(), channel.get_id(), stake)
            .await
            .unwrap()
            .await
            .unwrap();

        match tx_res {
            TransactionResult::ChannelFunded { tx_hash } => {
                assert_eq!(random_hash, tx_hash, "tx hash must be equal");
            }
            _ => panic!("invalid or failed tx result"),
        }
    }

    #[async_std::test]
    async fn test_should_not_fund_nonexistent_channel() {
        let _ = env_logger::builder().is_test(true).try_init();

        let self_addr = Address::random();
        let bob = Address::random();
        let channel_id = generate_channel_id(&self_addr, &bob);

        let db = Arc::new(RwLock::new(CoreEthereumDb::new(
            DB::new(RustyLevelDbShim::new_in_memory()),
            self_addr,
        )));
        let tx_queue = TransactionQueue::new(db.clone(), Box::new(MockTransactionExecutor::new()));

        db.write()
            .await
            .set_staking_safe_allowance(
                &Balance::new(10_000_000u64.into(), BalanceType::HOPR),
                &Snapshot::default(),
            )
            .await
            .unwrap();

        let stake = Balance::new(10_u32.into(), BalanceType::HOPR);
        assert!(
            matches!(
                fund_channel(db.clone(), tx_queue.new_sender(), channel_id, stake)
                    .await
                    .err()
                    .unwrap(),
                CoreEthereumActionsError::ChannelDoesNotExist
            ),
            "should fail when channel does not exist"
        );
    }

    #[async_std::test]
    async fn test_fund_should_not_allow_invalid_balance() {
        let _ = env_logger::builder().is_test(true).try_init();

        let self_addr = Address::random();
        let bob = Address::random();
        let channel_id = generate_channel_id(&self_addr, &bob);

        let db = Arc::new(RwLock::new(CoreEthereumDb::new(
            DB::new(RustyLevelDbShim::new_in_memory()),
            self_addr,
        )));
        let tx_queue = TransactionQueue::new(db.clone(), Box::new(MockTransactionExecutor::new()));

        db.write()
            .await
            .set_staking_safe_allowance(
                &Balance::new(10_000_000u64.into(), BalanceType::HOPR),
                &Snapshot::default(),
            )
            .await
            .unwrap();

        let stake = Balance::new(10_u32.into(), BalanceType::Native);
        assert!(
            matches!(
                open_channel(db.clone(), tx_queue.new_sender(), self_addr, bob, stake)
                    .await
                    .err()
                    .unwrap(),
                CoreEthereumActionsError::OtherError(_)
            ),
            "should not allow invalid balance"
        );

        let stake = Balance::new(0_u32.into(), BalanceType::HOPR);
        assert!(
            matches!(
                fund_channel(db.clone(), tx_queue.new_sender(), channel_id, stake)
                    .await
                    .err()
                    .unwrap(),
                CoreEthereumActionsError::OtherError(_)
            ),
            "should not allow invalid balance"
        );
    }

    #[async_std::test]
    async fn test_should_not_fund_if_not_enough_allowance() {
        let _ = env_logger::builder().is_test(true).try_init();

        let self_addr = Address::random();
        let bob = Address::random();
        let channel_id = generate_channel_id(&self_addr, &bob);

        let db = Arc::new(RwLock::new(CoreEthereumDb::new(
            DB::new(RustyLevelDbShim::new_in_memory()),
            self_addr,
        )));
        let tx_queue = TransactionQueue::new(db.clone(), Box::new(MockTransactionExecutor::new()));

        db.write()
            .await
            .set_staking_safe_allowance(&Balance::new(1000_u64.into(), BalanceType::HOPR), &Snapshot::default())
            .await
            .unwrap();

        let stake = Balance::new(10_000_u32.into(), BalanceType::HOPR);
        assert!(
            matches!(
                fund_channel(db.clone(), tx_queue.new_sender(), channel_id, stake)
                    .await
                    .err()
                    .unwrap(),
                CoreEthereumActionsError::NotEnoughAllowance
            ),
            "should fail when not enough allowance"
        );
    }

    #[async_std::test]
    async fn test_close_channel_outgoing() {
        let _ = env_logger::builder().is_test(true).try_init();

        let stake = Balance::new(10_u32.into(), BalanceType::HOPR);
        let random_hash = Hash::new(&random_bytes::<{ Hash::SIZE }>());

        let db = Arc::new(RwLock::new(CoreEthereumDb::new(
            DB::new(RustyLevelDbShim::new_in_memory()),
            *ALICE,
        )));

        let mut channel = ChannelEntry::new(
            *ALICE,
            *BOB,
            stake,
            U256::zero(),
            ChannelStatus::Open,
            U256::zero(),
            U256::zero(),
        );

        db.write()
            .await
            .update_channel_and_snapshot(&channel.get_id(), &channel, &Snapshot::default())
            .await
            .unwrap();

        let mut tx_exec = MockTransactionExecutor::new();
        let mut seq = Sequence::new();
        tx_exec
            .expect_initiate_outgoing_channel_closure()
            .times(1)
            .in_sequence(&mut seq)
            .withf(move |dst| BOB.eq(dst))
            .returning(move |_| TransactionResult::ChannelClosureInitiated { tx_hash: random_hash });

        tx_exec
            .expect_finalize_outgoing_channel_closure()
            .times(1)
            .in_sequence(&mut seq)
            .withf(move |dst| BOB.eq(dst))
            .returning(move |_| TransactionResult::ChannelClosed { tx_hash: random_hash });

        let tx_queue = TransactionQueue::new(db.clone(), Box::new(tx_exec));
        let tx_sender = tx_queue.new_sender();
        async_std::task::spawn_local(async move {
            tx_queue.transaction_loop().await;
        });

        let tx_res = close_channel(
            db.clone(),
            tx_sender.clone(),
            *BOB,
            *ALICE,
            ChannelDirection::Outgoing,
            false,
        )
        .await
        .unwrap()
        .await
        .unwrap();

        match tx_res {
            TransactionResult::ChannelClosureInitiated { tx_hash } => {
                assert_eq!(random_hash, tx_hash, "tx hash must be equal");
            }
            _ => panic!("invalid or failed tx result"),
        }

        // Transition the channel to the PendingToClose state with the closure time already elapsed
        channel.status = ChannelStatus::PendingToClose;
        channel.closure_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .sub(Duration::from_secs(10))
            .as_secs()
            .into();

        db.write()
            .await
            .update_channel_and_snapshot(&channel.get_id(), &channel, &Snapshot::default())
            .await
            .unwrap();

        let tx_res = close_channel(
            db.clone(),
            tx_sender.clone(),
            *BOB,
            *ALICE,
            ChannelDirection::Outgoing,
            false,
        )
        .await
        .unwrap()
        .await
        .unwrap();

        match tx_res {
            TransactionResult::ChannelClosed { tx_hash } => {
                assert_eq!(random_hash, tx_hash, "tx hash must be equal");
            }
            _ => panic!("invalid or failed tx result"),
        }
    }

    #[async_std::test]
    async fn test_close_channel_incoming() {
        let stake = Balance::new(10_u32.into(), BalanceType::HOPR);
        let random_hash = Hash::new(&random_bytes::<{ Hash::SIZE }>());

        let db = Arc::new(RwLock::new(CoreEthereumDb::new(
            DB::new(RustyLevelDbShim::new_in_memory()),
            *ALICE,
        )));

        let channel = ChannelEntry::new(
            *BOB,
            *ALICE,
            stake,
            U256::zero(),
            ChannelStatus::Open,
            U256::zero(),
            U256::zero(),
        );

        db.write()
            .await
            .update_channel_and_snapshot(&channel.get_id(), &channel, &Snapshot::default())
            .await
            .unwrap();

        let mut tx_exec = MockTransactionExecutor::new();
        let mut seq = Sequence::new();
        tx_exec
            .expect_close_incoming_channel()
            .times(1)
            .in_sequence(&mut seq)
            .withf(move |dst| BOB.eq(dst))
            .returning(move |_| TransactionResult::ChannelClosed { tx_hash: random_hash });

        let tx_queue = TransactionQueue::new(db.clone(), Box::new(tx_exec));
        let tx_sender = tx_queue.new_sender();
        async_std::task::spawn_local(async move {
            tx_queue.transaction_loop().await;
        });

        let tx_res = close_channel(
            db.clone(),
            tx_sender.clone(),
            *BOB,
            *ALICE,
            ChannelDirection::Incoming,
            false,
        )
        .await
        .unwrap()
        .await
        .unwrap();

        match tx_res {
            TransactionResult::ChannelClosed { tx_hash } => {
                assert_eq!(random_hash, tx_hash, "tx hash must be equal");
            }
            _ => panic!("invalid or failed tx result"),
        }
    }

    #[async_std::test]
    async fn test_should_not_close_when_closure_time_did_not_elapse() {
        let stake = Balance::new(10_u32.into(), BalanceType::HOPR);

        let db = Arc::new(RwLock::new(CoreEthereumDb::new(
            DB::new(RustyLevelDbShim::new_in_memory()),
            *ALICE,
        )));

        let channel = ChannelEntry::new(
            *ALICE,
            *BOB,
            stake,
            U256::zero(),
            ChannelStatus::PendingToClose,
            U256::zero(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .add(Duration::from_secs(100))
                .as_secs()
                .into(),
        );

        db.write()
            .await
            .update_channel_and_snapshot(&channel.get_id(), &channel, &Snapshot::default())
            .await
            .unwrap();

        let tx_queue = TransactionQueue::new(db.clone(), Box::new(MockTransactionExecutor::new()));

        assert!(
            matches!(
                close_channel(
                    db.clone(),
                    tx_queue.new_sender(),
                    *BOB,
                    *ALICE,
                    ChannelDirection::Outgoing,
                    false,
                )
                .await
                .err()
                .unwrap(),
                CoreEthereumActionsError::ClosureTimeHasNotElapsed(_)
            ),
            "should fail when the channel closure period did not elapse"
        );
    }

    #[async_std::test]
    async fn test_should_not_close_nonexistent_channel() {
        let _ = env_logger::builder().is_test(true).try_init();

        let db = Arc::new(RwLock::new(CoreEthereumDb::new(
            DB::new(RustyLevelDbShim::new_in_memory()),
            *ALICE,
        )));
        let tx_queue = TransactionQueue::new(db.clone(), Box::new(MockTransactionExecutor::new()));

        assert!(
            matches!(
                close_channel(
                    db.clone(),
                    tx_queue.new_sender(),
                    *BOB,
                    *ALICE,
                    ChannelDirection::Outgoing,
                    false
                )
                .await
                .err()
                .unwrap(),
                CoreEthereumActionsError::ChannelDoesNotExist
            ),
            "should fail when channel does not exist"
        );
    }

    #[async_std::test]
    async fn test_should_not_close_closed_channel() {
        let _ = env_logger::builder().is_test(true).try_init();

        let stake = Balance::new(10_u32.into(), BalanceType::HOPR);

        let db = Arc::new(RwLock::new(CoreEthereumDb::new(
            DB::new(RustyLevelDbShim::new_in_memory()),
            *ALICE,
        )));
        let tx_queue = TransactionQueue::new(db.clone(), Box::new(MockTransactionExecutor::new()));

        let channel = ChannelEntry::new(
            *ALICE,
            *BOB,
            stake,
            U256::zero(),
            ChannelStatus::Closed,
            U256::zero(),
            U256::zero(),
        );
        db.write()
            .await
            .update_channel_and_snapshot(&channel.get_id(), &channel, &Snapshot::default())
            .await
            .unwrap();

        assert!(
            matches!(
                close_channel(
                    db.clone(),
                    tx_queue.new_sender(),
                    *BOB,
                    *ALICE,
                    ChannelDirection::Outgoing,
                    false
                )
                .await
                .err()
                .unwrap(),
                CoreEthereumActionsError::ChannelAlreadyClosed
            ),
            "should fail when channel is already closed"
        );
    }

    #[async_std::test]
    async fn test_withdraw() {
        let _ = env_logger::builder().is_test(true).try_init();

        let stake = Balance::new(10_u32.into(), BalanceType::HOPR);
        let random_hash = Hash::new(&random_bytes::<{ Hash::SIZE }>());

        let db = Arc::new(RwLock::new(CoreEthereumDb::new(
            DB::new(RustyLevelDbShim::new_in_memory()),
            *ALICE,
        )));

        let mut tx_exec = MockTransactionExecutor::new();
        tx_exec
            .expect_withdraw()
            .times(1)
            .withf(move |dst, balance| BOB.eq(dst) && stake.eq(balance))
            .returning(move |_, _| TransactionResult::Withdrawn { tx_hash: random_hash });

        let tx_queue = TransactionQueue::new(db.clone(), Box::new(tx_exec));
        let tx_sender = tx_queue.new_sender();
        async_std::task::spawn_local(async move {
            tx_queue.transaction_loop().await;
        });

        let tx_res = withdraw(tx_sender.clone(), *BOB, stake).await.unwrap().await.unwrap();

        match tx_res {
            TransactionResult::Withdrawn { tx_hash } => {
                assert_eq!(random_hash, tx_hash, "tx hash must be equal");
            }
            _ => panic!("invalid or failed tx result"),
        }
    }

    #[async_std::test]
    async fn test_should_not_withdraw_zero_amount() {
        let _ = env_logger::builder().is_test(true).try_init();

        let db = Arc::new(RwLock::new(CoreEthereumDb::new(
            DB::new(RustyLevelDbShim::new_in_memory()),
            *ALICE,
        )));
        let tx_queue = TransactionQueue::new(db.clone(), Box::new(MockTransactionExecutor::new()));

        assert!(
            matches!(
                withdraw(tx_queue.new_sender(), *BOB, Balance::zero(BalanceType::HOPR))
                    .await
                    .err()
                    .unwrap(),
                CoreEthereumActionsError::OtherError(_)
            ),
            "should not allow to withdraw 0"
        );
    }
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use crate::transaction_queue::{TransactionResult, TransactionSender};
    use core_crypto::types::Hash;
    use core_ethereum_db::db::wasm::Database;
    use core_types::channels::{generate_channel_id, ChannelDirection, ChannelStatus};
    use utils_misc::utils::wasm::JsResult;
    use utils_types::primitives::{Address, Balance};
    use wasm_bindgen::prelude::wasm_bindgen;
    use wasm_bindgen::JsValue;

    #[wasm_bindgen(getter_with_clone)]
    pub struct OpenChannelResult {
        pub tx_hash: Hash,
        pub channel_id: Hash,
    }

    #[wasm_bindgen(getter_with_clone)]
    pub struct CloseChannelResult {
        pub tx_hash: Hash,
        pub status: ChannelStatus,
    }

    #[wasm_bindgen]
    pub async fn open_channel(
        db: &Database,
        destination: &Address,
        self_addr: &Address,
        amount: &Balance,
        on_chain_tx_sender: &TransactionSender,
    ) -> JsResult<OpenChannelResult> {
        let awaiter = super::open_channel(
            db.as_ref_counted(),
            on_chain_tx_sender.clone(),
            *destination,
            *self_addr,
            *amount,
        )
        .await?;

        let channel_id = generate_channel_id(self_addr, destination);
        match awaiter
            .await
            .map_err(|_| JsValue::from("transaction has been cancelled".to_string()))?
        {
            TransactionResult::ChannelFunded { tx_hash } => Ok(OpenChannelResult { tx_hash, channel_id }),
            _ => Err(JsValue::from("open channel transaction failed".to_string())),
        }
    }

    #[wasm_bindgen]
    pub async fn fund_channel(
        db: &Database,
        channel_id: &Hash,
        amount: &Balance,
        on_chain_tx_sender: &TransactionSender,
    ) -> JsResult<Hash> {
        let awaiter =
            super::fund_channel(db.as_ref_counted(), on_chain_tx_sender.clone(), *channel_id, *amount).await?;
        match awaiter
            .await
            .map_err(|_| JsValue::from("transaction has been cancelled".to_string()))?
        {
            TransactionResult::ChannelFunded { tx_hash } => Ok(tx_hash),
            _ => Err(JsValue::from("fund channel transaction failed".to_string())),
        }
    }

    #[wasm_bindgen]
    pub async fn close_channel(
        db: &Database,
        counterparty: &Address,
        self_addr: &Address,
        direction: ChannelDirection,
        redeem_before_close: bool,
        on_chain_tx_sender: &TransactionSender,
    ) -> JsResult<CloseChannelResult> {
        let awaiter = super::close_channel(
            db.as_ref_counted(),
            on_chain_tx_sender.clone(),
            *counterparty,
            *self_addr,
            direction,
            redeem_before_close,
        )
        .await?;
        match awaiter
            .await
            .map_err(|_| JsValue::from("transaction has been cancelled".to_string()))?
        {
            TransactionResult::ChannelClosureInitiated { tx_hash } => Ok(CloseChannelResult {
                tx_hash,
                status: ChannelStatus::PendingToClose,
            }),
            TransactionResult::ChannelClosed { tx_hash } => Ok(CloseChannelResult {
                tx_hash,
                status: ChannelStatus::Closed,
            }),
            _ => Err(JsValue::from("close channel transaction failed".to_string())),
        }
    }

    #[wasm_bindgen]
    pub async fn withdraw(
        recipient: &Address,
        amount: &Balance,
        on_chain_tx_sender: &TransactionSender,
    ) -> JsResult<Hash> {
        let awaiter = super::withdraw(on_chain_tx_sender.clone(), *recipient, *amount).await?;
        match awaiter
            .await
            .map_err(|_| JsValue::from("transaction has been cancelled".to_string()))?
        {
            TransactionResult::Withdrawn { tx_hash } => Ok(tx_hash),
            _ => Err(JsValue::from("withdraw transaction failed".to_string())),
        }
    }
}
