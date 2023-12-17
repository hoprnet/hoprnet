use async_lock::RwLock;
use core_ethereum_db::traits::HoprCoreEthereumDbActions;
use std::sync::Arc;
use utils_types::primitives::Address;

use crate::action_queue::ActionSender;

pub mod action_queue;
pub mod action_state;
pub mod channels;
pub mod errors;
pub mod node;
pub mod payload;
pub mod redeem;

/// Contains all actions that a node can execute on-chain.
#[derive(Debug, Clone)]
pub struct CoreEthereumActions<Db: HoprCoreEthereumDbActions + Clone + Send + Sync> {
    me: Address,
    db: Arc<RwLock<Db>>,
    tx_sender: ActionSender,
}

impl<Db: HoprCoreEthereumDbActions + Clone + Send + Sync> CoreEthereumActions<Db> {
    /// Creates new instance.
    pub fn new(me: Address, db: Arc<RwLock<Db>>, tx_sender: ActionSender) -> Self {
        Self { me, db, tx_sender }
    }

    /// On-chain address of this node
    pub fn self_address(&self) -> Address {
        self.me
    }
}
