//! This module contains on-chain actions in the `NodeActions` trait, which related to HOPR node itself.
//! An implementation of this trait is added to [ChainActions] which realizes the redemption
//! operations via [ActionQueue].
//!
//! There are 3 functions that can be used to redeem tickets in the `NodeActions` trait:
//! - [withdraw](NodeActions::withdraw)
//! - [announce](NodeActions::announce)
//! - [register_safe_by_node](NodeActions::register_safe_by_node)
//!
//! All necessary pre-requisites are checked by the implementation before the respective [Action] is submitted
//! to the [ActionQueue].
use async_trait::async_trait;
use chain_db::traits::HoprCoreEthereumDbActions;
use chain_types::actions::Action;
use hopr_crypto_types::keypairs::OffchainKeypair;
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::*;
use log::info;
use multiaddr::Multiaddr;

use crate::action_queue::PendingAction;
use crate::errors::{ChainActionsError::InvalidArguments, Result};
use crate::ChainActions;

/// Contains all on-chain calls specific to HOPR node itself.
#[async_trait]
pub trait NodeActions {
    /// Withdraws the specified `amount` of tokens to the given `recipient`.
    async fn withdraw(&self, recipient: Address, amount: Balance) -> Result<PendingAction>;

    /// Announces node on-chain with key binding
    async fn announce(&self, multiaddr: &Multiaddr, offchain_key: &OffchainKeypair) -> Result<PendingAction>;

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

    async fn announce(&self, multiaddr: &Multiaddr, offchain_key: &OffchainKeypair) -> Result<PendingAction> {
        let announcement_data = AnnouncementData::new(multiaddr, Some(KeyBinding::new(self.me, offchain_key)))?;

        info!("initiating announcement {announcement_data}");
        self.tx_sender.send(Action::Announce(announcement_data)).await
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
    use chain_types::actions::Action;
    use hopr_crypto_random::random_bytes;
    use hopr_crypto_types::types::Hash;
    use hopr_primitive_types::prelude::*;
    use std::sync::Arc;
    use utils_db::db::DB;
    use utils_db::CurrentDbShim;

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

        let actions = ChainActions::new(self_addr, db.clone(), tx_sender.clone());

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
        let actions = ChainActions::new(self_addr, db.clone(), tx_queue.new_sender());

        assert!(
            matches!(
                actions
                    .withdraw(bob, Balance::zero(BalanceType::HOPR))
                    .await
                    .err()
                    .unwrap(),
                ChainActionsError::InvalidArguments(_)
            ),
            "should not allow to withdraw 0"
        );
    }
}
