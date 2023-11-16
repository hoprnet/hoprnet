use crate::errors::{CoreEthereumActionsError::InvalidArguments, Result};
use crate::transaction_queue::{Transaction, TransactionCompleted};
use crate::CoreEthereumActions;
use async_trait::async_trait;
use core_crypto::keypairs::OffchainKeypair;
use core_ethereum_db::traits::HoprCoreEthereumDbActions;
use core_types::announcement::{AnnouncementData, KeyBinding};
use multiaddr::Multiaddr;
use utils_log::info;
use utils_types::primitives::{Address, Balance};

/// Contains all on-chain calls specific to HOPR node itself.
#[async_trait(? Send)]
pub trait NodeActions {
    /// Withdraws the specified `amount` of tokens to the given `recipient`.
    async fn withdraw(&self, recipient: Address, amount: Balance) -> Result<TransactionCompleted>;

    /// Announces node on-chain with key binding
    async fn announce(&self, multiaddr: &Multiaddr, offchain_key: &OffchainKeypair) -> Result<TransactionCompleted>;

    async fn register_safe_by_node(&self, safe_address: Address) -> Result<TransactionCompleted>;
}

#[async_trait(? Send)]
impl<Db: HoprCoreEthereumDbActions + Clone> NodeActions for CoreEthereumActions<Db> {
    async fn withdraw(&self, recipient: Address, amount: Balance) -> Result<TransactionCompleted> {
        if amount.eq(&amount.of_same("0")) {
            return Err(InvalidArguments("cannot withdraw zero amount".into()).into());
        }

        // TODO: should we check native/token balance here before withdrawing ?

        info!("initiating withdrawal of {amount} to {recipient}");
        self.tx_sender.send(Transaction::Withdraw(recipient, amount)).await
    }

    async fn announce(&self, multiaddr: &Multiaddr, offchain_key: &OffchainKeypair) -> Result<TransactionCompleted> {
        let announcement_data = AnnouncementData::new(multiaddr, Some(KeyBinding::new(self.me, offchain_key)))?;

        info!("initiating announcement {announcement_data}");
        self.tx_sender.send(Transaction::Announce(announcement_data)).await
    }

    async fn register_safe_by_node(&self, safe_address: Address) -> Result<TransactionCompleted> {
        info!("initiating safe address registration of {safe_address}");
        self.tx_sender.send(Transaction::RegisterSafe(safe_address)).await
    }
}

#[cfg(test)]
mod tests {
    use crate::errors::CoreEthereumActionsError;
    use crate::node::NodeActions;
    use crate::transaction_queue::{MockTransactionExecutor, TransactionQueue, TransactionResult};
    use crate::CoreEthereumActions;
    use async_lock::RwLock;
    use core_crypto::random::random_bytes;
    use core_crypto::types::Hash;
    use core_ethereum_db::db::CoreEthereumDb;
    use std::sync::Arc;
    use utils_db::db::DB;
    use utils_db::rusty::RustyLevelDbShim;
    use utils_types::primitives::{Address, Balance, BalanceType};
    use utils_types::traits::BinarySerializable;

    #[async_std::test]
    async fn test_withdraw() {
        let _ = env_logger::builder().is_test(true).try_init();

        let self_addr = Address::random();
        let bob = Address::random();
        let stake = Balance::new(10_u32.into(), BalanceType::HOPR);
        let random_hash = Hash::new(&random_bytes::<{ Hash::SIZE }>());

        let db = Arc::new(RwLock::new(CoreEthereumDb::new(
            DB::new(RustyLevelDbShim::new_in_memory()),
            self_addr,
        )));

        let mut tx_exec = MockTransactionExecutor::new();
        tx_exec
            .expect_withdraw()
            .times(1)
            .withf(move |dst, balance| bob.eq(dst) && stake.eq(balance))
            .returning(move |_, _| TransactionResult::Withdrawn { tx_hash: random_hash });

        let tx_queue = TransactionQueue::new(db.clone(), Box::new(tx_exec));
        let tx_sender = tx_queue.new_sender();
        async_std::task::spawn_local(async move {
            tx_queue.transaction_loop().await;
        });

        let actions = CoreEthereumActions::new(self_addr, db.clone(), tx_sender.clone());

        let tx_res = actions.withdraw(bob, stake).await.unwrap().await;

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

        let self_addr = Address::random();
        let bob = Address::random();

        let db = Arc::new(RwLock::new(CoreEthereumDb::new(
            DB::new(RustyLevelDbShim::new_in_memory()),
            self_addr,
        )));
        let tx_queue = TransactionQueue::new(db.clone(), Box::new(MockTransactionExecutor::new()));
        let actions = CoreEthereumActions::new(self_addr, db.clone(), tx_queue.new_sender());

        assert!(
            matches!(
                actions
                    .withdraw(bob, Balance::zero(BalanceType::HOPR))
                    .await
                    .err()
                    .unwrap(),
                CoreEthereumActionsError::OtherError(_)
            ),
            "should not allow to withdraw 0"
        );
    }
}
