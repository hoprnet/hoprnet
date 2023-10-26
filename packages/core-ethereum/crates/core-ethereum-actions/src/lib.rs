pub mod channels;
pub mod errors;
pub mod node;
pub mod payload;
pub mod redeem;
pub mod transaction_queue;
pub mod rpc_tx_executor;

use crate::transaction_queue::TransactionSender;
use async_lock::RwLock;
use core_ethereum_db::traits::HoprCoreEthereumDbActions;
use std::sync::Arc;
use utils_types::primitives::Address;

/// Contains all actions that a node can execute on-chain.
#[derive(Clone)]
pub struct CoreEthereumActions<Db: HoprCoreEthereumDbActions + Clone> {
    me: Address,
    db: Arc<RwLock<Db>>,
    tx_sender: TransactionSender,
}

impl<Db: HoprCoreEthereumDbActions + Clone> CoreEthereumActions<Db> {
    /// Creates new instance.
    pub fn new(me: Address, db: Arc<RwLock<Db>>, tx_sender: TransactionSender) -> Self {
        Self { me, db, tx_sender }
    }

    /// On-chain address of this node
    pub fn self_address(&self) -> Address {
        self.me
    }
}
