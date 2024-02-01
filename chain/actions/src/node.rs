use crate::action_queue::PendingAction;
use crate::errors::CoreEthereumActionsError::AlreadyAnnounced;
use crate::errors::{CoreEthereumActionsError::InvalidArguments, Result};
use crate::CoreEthereumActions;
use async_trait::async_trait;
use chain_db::traits::HoprCoreEthereumDbActions;
use chain_types::actions::Action;
use hopr_crypto_types::keypairs::OffchainKeypair;
use hopr_crypto_types::prelude::Keypair;
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::*;
use log::info;
use multiaddr::Multiaddr;

/// Contains all on-chain calls specific to HOPR node itself.
#[async_trait]
pub trait NodeActions {
    /// Withdraws the specified `amount` of tokens to the given `recipient`.
    async fn withdraw(&self, recipient: Address, amount: Balance) -> Result<PendingAction>;

    /// Announces node on-chain with key binding.
    /// The operation should also check if such announcement has not been already made on-chain.
    async fn announce(&self, multiaddr: &Multiaddr, offchain_key: &OffchainKeypair) -> Result<PendingAction>;

    /// Registers the safe address with the node
    async fn register_safe_by_node(&self, safe_address: Address) -> Result<PendingAction>;
}

#[async_trait]
impl<Db: HoprCoreEthereumDbActions + Clone + Send + Sync> NodeActions for CoreEthereumActions<Db> {
    async fn withdraw(&self, recipient: Address, amount: Balance) -> Result<PendingAction> {
        if amount.eq(&amount.of_same("0")) {
            return Err(InvalidArguments("cannot withdraw zero amount".into()));
        }

        // TODO: should we check native/token balance here before withdrawing ?

        info!("initiating withdrawal of {amount} to {recipient}");
        self.tx_sender.send(Action::Withdraw(recipient, amount)).await
    }

    async fn announce(&self, multiaddr: &Multiaddr, offchain_key: &OffchainKeypair) -> Result<PendingAction> {
        if !self
            .db
            .read()
            .await
            .get_public_node_accounts()
            .await?
            .into_iter()
            .any(|account| {
                account.get_multiaddr().is_some_and(|ma| ma.eq(multiaddr))
                    && account.public_key.eq(offchain_key.public())
            })
        {
            let announcement_data = AnnouncementData::new(multiaddr, Some(KeyBinding::new(self.me, offchain_key)))?;

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
    use crate::errors::CoreEthereumActionsError;
    use crate::node::NodeActions;
    use crate::CoreEthereumActions;
    use async_lock::RwLock;
    use chain_db::db::CoreEthereumDb;
    use chain_db::traits::HoprCoreEthereumDbActions;
    use chain_types::actions::Action;
    use chain_types::chain_events::{ChainEventType, SignificantChainEvent};
    use futures::FutureExt;
    use hopr_crypto_random::random_bytes;
    use hopr_crypto_types::keypairs::OffchainKeypair;
    use hopr_crypto_types::prelude::Keypair;
    use hopr_crypto_types::types::Hash;
    use hopr_internal_types::account::AccountType;
    use hopr_internal_types::prelude::AccountEntry;
    use hopr_primitive_types::prelude::*;
    use multiaddr::Multiaddr;
    use std::str::FromStr;
    use std::sync::Arc;
    use utils_db::db::DB;
    use utils_db::CurrentDbShim;

    #[async_std::test]
    async fn test_announce() {
        let _ = env_logger::builder().is_test(true).try_init();

        let self_addr = Address::random();
        let random_hash = Hash::new(&random_bytes::<{ Hash::SIZE }>());
        let keypair = OffchainKeypair::random();
        let announce_multiaddr = Multiaddr::from_str("/ip4/1.2.3.4/tcp/9009").unwrap();

        let db = Arc::new(RwLock::new(CoreEthereumDb::new(
            DB::new(CurrentDbShim::new_in_memory().await),
            self_addr,
        )));

        let ma = announce_multiaddr.clone();
        let pubkey_clone = keypair.public().clone();
        let mut tx_exec = MockTransactionExecutor::new();
        tx_exec
            .expect_announce()
            .once()
            .withf(move |ad| {
                let kb = ad.key_binding.clone().unwrap();
                ma.eq(ad.multiaddress()) && kb.packet_key == pubkey_clone && kb.chain_key == self_addr
            })
            .returning(move |_| Ok(random_hash));

        let ma = announce_multiaddr.clone();
        let pk = keypair.public().clone();
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
                        address: self_addr,
                    },
                })
                .boxed())
            });

        let tx_queue = ActionQueue::new(db.clone(), indexer_action_tracker, tx_exec, Default::default());
        let tx_sender = tx_queue.new_sender();
        async_std::task::spawn(async move {
            tx_queue.action_loop().await;
        });

        let actions = CoreEthereumActions::new(self_addr, db.clone(), tx_sender.clone());
        let tx_res = actions
            .announce(&announce_multiaddr, &keypair)
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

        let self_addr = Address::random();
        let keypair = OffchainKeypair::random();
        let announce_multiaddr = Multiaddr::from_str("/ip4/1.2.3.4/tcp/9009").unwrap();

        let db = Arc::new(RwLock::new(CoreEthereumDb::new(
            DB::new(CurrentDbShim::new_in_memory().await),
            self_addr,
        )));

        db.write()
            .await
            .update_account_and_snapshot(
                &AccountEntry::new(
                    *keypair.public(),
                    self_addr,
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

        let actions = CoreEthereumActions::new(self_addr, db.clone(), tx_sender.clone());

        let res = actions.announce(&announce_multiaddr, &keypair).await;
        assert!(
            matches!(res, Err(CoreEthereumActionsError::AlreadyAnnounced)),
            "must not be able to re-announce with same address"
        );
    }

    #[async_std::test]
    async fn test_withdraw() {
        let _ = env_logger::builder().is_test(true).try_init();

        let self_addr = Address::random();
        let bob = Address::random();
        let stake = Balance::new(10_u32, BalanceType::HOPR);
        let random_hash = Hash::new(&random_bytes::<{ Hash::SIZE }>());

        let db = Arc::new(RwLock::new(CoreEthereumDb::new(
            DB::new(CurrentDbShim::new_in_memory().await),
            self_addr,
        )));

        let mut tx_exec = MockTransactionExecutor::new();
        tx_exec
            .expect_withdraw()
            .times(1)
            .withf(move |dst, balance| bob.eq(dst) && stake.eq(balance))
            .returning(move |_, _| Ok(random_hash));

        let mut indexer_action_tracker = MockActionState::new();
        indexer_action_tracker.expect_register_expectation().never();

        let tx_queue = ActionQueue::new(db.clone(), indexer_action_tracker, tx_exec, Default::default());
        let tx_sender = tx_queue.new_sender();
        async_std::task::spawn(async move {
            tx_queue.action_loop().await;
        });

        let actions = CoreEthereumActions::new(self_addr, db.clone(), tx_sender.clone());

        let tx_res = actions
            .withdraw(bob, stake)
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

        let self_addr = Address::random();
        let bob = Address::random();

        let db = Arc::new(RwLock::new(CoreEthereumDb::new(
            DB::new(CurrentDbShim::new_in_memory().await),
            self_addr,
        )));
        let tx_queue = ActionQueue::new(
            db.clone(),
            MockActionState::new(),
            MockTransactionExecutor::new(),
            Default::default(),
        );
        let actions = CoreEthereumActions::new(self_addr, db.clone(), tx_queue.new_sender());

        assert!(
            matches!(
                actions
                    .withdraw(bob, Balance::zero(BalanceType::HOPR))
                    .await
                    .err()
                    .unwrap(),
                CoreEthereumActionsError::InvalidArguments(_)
            ),
            "should not allow to withdraw 0"
        );
    }
}
