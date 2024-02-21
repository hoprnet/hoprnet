//! This module contains the [NodeActions] trait defining action which relate to HOPR node itself.
//!
//! An implementation of this trait is added to [ChainActions] which realizes the redemption
//! operations via [ActionQueue](crate::action_queue::ActionQueue).
//!
//! There are 3 functions that can be used to redeem tickets in the [NodeActions] trait:
//! - [withdraw](NodeActions::withdraw)
//! - [announce](NodeActions::announce)
//! - [register_safe_by_node](NodeActions::register_safe_by_node)
//!
//! All necessary pre-requisites are checked by the implementation before the respective [Action] is submitted
//! to the [ActionQueue](crate::action_queue::ActionQueue).
use async_trait::async_trait;
use chain_db::traits::HoprCoreEthereumDbActions;
use chain_types::actions::Action;
use hopr_crypto_types::prelude::*;
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::*;
use multiaddr::Multiaddr;
use tracing::info;

use crate::action_queue::PendingAction;
use crate::errors::{
    ChainActionsError::{AlreadyAnnounced, InvalidArguments},
    Result,
};
use crate::ChainActions;

/// Contains all on-chain calls specific to HOPR node itself.
#[async_trait]
pub trait NodeActions {
    /// Withdraws the specified `amount` of tokens to the given `recipient`.
    async fn withdraw(&self, recipient: Address, amount: Balance) -> Result<PendingAction>;

    /// Announces node on-chain with key binding.
    /// The operation should also check if such announcement has not been already made on-chain.
    async fn announce(&self, multiaddrs: &[Multiaddr], offchain_key: &OffchainKeypair) -> Result<PendingAction>;

    /// Registers the safe address with the node
    async fn register_safe_by_node(&self, safe_address: Address) -> Result<PendingAction>;
}

#[async_trait]
impl<Db: HoprCoreEthereumDbActions + Clone + Send + Sync> NodeActions for ChainActions<Db> {
    async fn withdraw(&self, recipient: Address, amount: Balance) -> Result<PendingAction> {
        if amount.eq(&amount.of_same("0")) {
            return Err(InvalidArguments("cannot withdraw zero amount".into()));
        }

        // TODO: should we check native/token balance here before withdrawing ?

        info!("initiating withdrawal of {amount} to {recipient}");
        self.tx_sender.send(Action::Withdraw(recipient, amount)).await
    }

    async fn announce(&self, multiaddrs: &[Multiaddr], offchain_key: &OffchainKeypair) -> Result<PendingAction> {
        // TODO: allow announcing all addresses once that option is supported
        let announcement_data = AnnouncementData::new(
            multiaddrs[0].clone(),
            Some(KeyBinding::new(self.self_address(), offchain_key)),
        )?;

        if !self
            .db
            .read()
            .await
            .get_public_node_accounts()
            .await?
            .into_iter()
            .any(|account| {
                account.public_key.eq(offchain_key.public())
                    && account
                        .get_multiaddr()
                        .is_some_and(|ma| decapsulate_multiaddress(ma).eq(announcement_data.multiaddress()))
            })
        {
            info!("initiating announcement {announcement_data}");
            self.tx_sender.send(Action::Announce(announcement_data)).await
        } else {
            Err(AlreadyAnnounced)
        }
    }

    async fn register_safe_by_node(&self, safe_address: Address) -> Result<PendingAction> {
        info!("initiating safe address registration of {safe_address}");
        self.tx_sender.send(Action::RegisterSafe(safe_address)).await
    }
}

#[cfg(test)]
mod tests {
    use crate::action_queue::{ActionQueue, MockTransactionExecutor};
    use crate::action_state::MockActionState;
    use crate::errors::ChainActionsError;
    use crate::node::NodeActions;
    use crate::ChainActions;
    use async_lock::RwLock;
    use chain_db::db::CoreEthereumDb;
    use chain_db::traits::HoprCoreEthereumDbActions;
    use chain_types::actions::Action;
    use chain_types::chain_events::*;
    use futures::FutureExt;
    use hex_literal::hex;
    use hopr_crypto_random::random_bytes;
    use hopr_crypto_types::prelude::*;
    use hopr_internal_types::prelude::*;
    use hopr_primitive_types::prelude::*;
    use multiaddr::Multiaddr;
    use std::str::FromStr;
    use std::sync::Arc;
    use utils_db::db::DB;
    use utils_db::CurrentDbShim;

    lazy_static::lazy_static! {
        static ref ALICE: ChainKeypair = ChainKeypair::from_secret(&hex!("cdb0e2219b75ef23a0167e8f3196f0a292c92699e562a4533df658b8fac196d0")).unwrap();
        static ref ALICE_ADDR: Address = ALICE.public().to_address();
        static ref BOB: ChainKeypair = ChainKeypair::from_secret(&hex!("92019229229fff4c36c52fb1257f3ca710c73502ec7f6111eda4c1b5b8e84810")).unwrap();
        static ref BOB_ADDR: Address = BOB.public().to_address();
        static ref ALICE_OFFCHAIN: OffchainKeypair = OffchainKeypair::from_secret(&hex!("e0bf93e9c916104da00b1850adc4608bd7e9087bbd3f805451f4556aa6b3fd6e")).unwrap();
    }

    #[async_std::test]
    async fn test_announce() {
        let _ = env_logger::builder().is_test(true).try_init();

        let random_hash = Hash::new(&random_bytes::<{ Hash::SIZE }>());
        let announce_multiaddr = Multiaddr::from_str("/ip4/1.2.3.4/tcp/9009").unwrap();

        let db = Arc::new(RwLock::new(CoreEthereumDb::new(
            DB::new(CurrentDbShim::new_in_memory().await),
            *ALICE_ADDR,
        )));

        let ma = announce_multiaddr.clone();
        let pubkey_clone = ALICE_OFFCHAIN.public().clone();
        let mut tx_exec = MockTransactionExecutor::new();
        tx_exec
            .expect_announce()
            .once()
            .withf(move |ad| {
                let kb = ad.key_binding.clone().unwrap();
                ma.eq(ad.multiaddress()) && kb.packet_key == pubkey_clone && kb.chain_key == *ALICE_ADDR
            })
            .returning(move |_| Ok(random_hash));

        let ma = announce_multiaddr.clone();
        let pk = ALICE_OFFCHAIN.public().clone();
        let mut indexer_action_tracker = MockActionState::new();
        indexer_action_tracker
            .expect_register_expectation()
            .once()
            .returning(move |_| {
                Ok(futures::future::ok(SignificantChainEvent {
                    tx_hash: random_hash,
                    event_type: ChainEventType::Announcement {
                        peer: pk.into(),
                        multiaddresses: vec![ma.clone()],
                        address: *ALICE_ADDR,
                    },
                })
                .boxed())
            });

        let tx_queue = ActionQueue::new(db.clone(), indexer_action_tracker, tx_exec, Default::default());
        let tx_sender = tx_queue.new_sender();
        async_std::task::spawn(async move {
            tx_queue.action_loop().await;
        });

        let actions = ChainActions::new(ALICE.clone(), db.clone(), tx_sender.clone());
        let tx_res = actions
            .announce(&[announce_multiaddr], &ALICE_OFFCHAIN)
            .await
            .expect("announcement call should not fail")
            .await
            .expect("announcement should be confirmed");

        assert_eq!(tx_res.tx_hash, random_hash, "tx hashes must be equal");
        assert!(matches!(tx_res.action, Action::Announce(_)), "must be announce action");
        assert!(
            matches!(tx_res.event, Some(ChainEventType::Announcement { .. })),
            "must correspond to announcement chain event"
        );
    }

    #[async_std::test]
    async fn test_announce_should_not_allow_reannouncing_with_same_multiaddress() {
        let _ = env_logger::builder().is_test(true).try_init();

        let announce_multiaddr = Multiaddr::from_str("/ip4/1.2.3.4/tcp/9009").unwrap();

        let db = Arc::new(RwLock::new(CoreEthereumDb::new(
            DB::new(CurrentDbShim::new_in_memory().await),
            *ALICE_ADDR,
        )));

        db.write()
            .await
            .update_account_and_snapshot(
                &AccountEntry::new(
                    *ALICE_OFFCHAIN.public(),
                    *ALICE_ADDR,
                    AccountType::Announced {
                        multiaddr: announce_multiaddr.clone(),
                        updated_block: 0,
                    },
                ),
                &Snapshot::default(),
            )
            .await
            .unwrap();

        let tx_queue = ActionQueue::new(
            db.clone(),
            MockActionState::new(),
            MockTransactionExecutor::new(),
            Default::default(),
        );
        let tx_sender = tx_queue.new_sender();

        let actions = ChainActions::new(ALICE.clone(), db.clone(), tx_sender.clone());

        let res = actions.announce(&[announce_multiaddr], &*ALICE_OFFCHAIN).await;
        assert!(
            matches!(res, Err(ChainActionsError::AlreadyAnnounced)),
            "must not be able to re-announce with same address"
        );
    }

    #[async_std::test]
    async fn test_withdraw() {
        let _ = env_logger::builder().is_test(true).try_init();

        let stake = Balance::new(10_u32, BalanceType::HOPR);
        let random_hash = Hash::new(&random_bytes::<{ Hash::SIZE }>());

        let db = Arc::new(RwLock::new(CoreEthereumDb::new(
            DB::new(CurrentDbShim::new_in_memory().await),
            *ALICE_ADDR,
        )));

        let mut tx_exec = MockTransactionExecutor::new();
        tx_exec
            .expect_withdraw()
            .times(1)
            .withf(move |dst, balance| BOB_ADDR.eq(dst) && stake.eq(balance))
            .returning(move |_, _| Ok(random_hash));

        let mut indexer_action_tracker = MockActionState::new();
        indexer_action_tracker.expect_register_expectation().never();

        let tx_queue = ActionQueue::new(db.clone(), indexer_action_tracker, tx_exec, Default::default());
        let tx_sender = tx_queue.new_sender();
        async_std::task::spawn(async move {
            tx_queue.action_loop().await;
        });

        let actions = ChainActions::new(ALICE.clone(), db.clone(), tx_sender.clone());

        let tx_res = actions
            .withdraw(*BOB_ADDR, stake)
            .await
            .unwrap()
            .await
            .expect("must resolve confirmation");

        assert_eq!(tx_res.tx_hash, random_hash, "tx hashes must be equal");
        assert!(
            matches!(tx_res.action, Action::Withdraw(_, _)),
            "must be withdraw action"
        );
        assert!(
            tx_res.event.is_none(),
            "withdraw tx must not connect to any chain event"
        );
    }

    #[async_std::test]
    async fn test_should_not_withdraw_zero_amount() {
        let _ = env_logger::builder().is_test(true).try_init();

        let db = Arc::new(RwLock::new(CoreEthereumDb::new(
            DB::new(CurrentDbShim::new_in_memory().await),
            *ALICE_ADDR,
        )));
        let tx_queue = ActionQueue::new(
            db.clone(),
            MockActionState::new(),
            MockTransactionExecutor::new(),
            Default::default(),
        );
        let actions = ChainActions::new(ALICE.clone(), db.clone(), tx_queue.new_sender());

        assert!(
            matches!(
                actions
                    .withdraw(*BOB_ADDR, Balance::zero(BalanceType::HOPR))
                    .await
                    .err()
                    .unwrap(),
                ChainActionsError::InvalidArguments(_)
            ),
            "should not allow to withdraw 0"
        );
    }
}
