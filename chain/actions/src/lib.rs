// TODO: docs need updating
//! Contains high-level Core-Ethereum traits that translate to on-chain transactions
//!
//!

use async_lock::RwLock;
use chain_db::traits::HoprCoreEthereumDbActions;
use hopr_primitive_types::primitives::Address;
use std::sync::Arc;

use crate::action_queue::ActionSender;

pub mod action_queue;
///
pub mod action_state;
/// Actions related to HOPR channels.
pub mod channels;
/// Contains all errors used in this crate.
pub mod errors;
/// Actions related to a HOPR node itself.
pub mod node;
/// Ethereum transaction payload generators for the actions.
pub mod payload;
/// Ticket redemption related actions.
pub mod redeem;

/// Contains all actions that a node can execute on-chain.
#[derive(Debug, Clone)]
pub struct CoreEthereumActions<Db: HoprCoreEthereumDbActions + Clone> {
    me: Address,
    db: Arc<RwLock<Db>>,
    tx_sender: ActionSender,
}

impl<Db: HoprCoreEthereumDbActions + Clone> CoreEthereumActions<Db> {
    ///! Creates new instance.
    pub fn new(me: Address, db: Arc<RwLock<Db>>, tx_sender: ActionSender) -> Self {
        Self { me, db, tx_sender }
    }

    ///! On-chain address of this node
    pub fn self_address(&self) -> Address {
        self.me
    }
}
