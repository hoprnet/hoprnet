// TODO: docs need updating
//! Contains high-level Core-Ethereum traits that translate to on-chain transactions
//!
//! ## `action_queue`
//! The `ActionQueue` object acts as general outgoing on-chain action MPSC queue. The queue is picked up
//! one-by-one in an infinite loop that's executed in `core-transport`. Any component that gets a `ActionSender` type,
//! can send new action requests to the queue via its `send` method.
//! A new `ActionSender` can be obtained by calling `new_sender` method on the `ActionQueue` and can be subsequently cloned.
//! The possible actions that can be sent into the queue are declared in the `Action` enum.
//! The `send` method of `ActionSender` returns a `ActionComplete` future that can be awaited if the caller
//! wishes to await the underlying transaction being confirmed.
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
