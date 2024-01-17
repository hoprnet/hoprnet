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
//! ## `redeem`
//! There are 4 functions that can be used to redeem tickets in the `TicketRedeemActions` trait:
//! - `redeem_all_tickets`
//! - `redeem_tickets_in_channel`
//! - `redeem_tickets_by_counterparty`
//! - `redeem_ticket`
//!
//! The method first checks if the tickets are redeemable (= they are not in `BeingRedeemed` or `BeginAggregated` in the DB),
//! and if they are, their state is changed to `BeingRedeemed` (while having acquired the exclusive DB write lock).
//! Subsequently, the ticket in such state is transmitted into the `ActionQueue` so the redemption soon is executed on-chain.
//! The functions return immediately, but provide futures that can be awaited in case the callers wishes to await the on-chain
//! confirmation of each ticket redemption.
//!
//! ## `channels`
//! This submodule adds 4 basic high-level on-chain functions in the `ChannelActions` trait:
//! - `open_channel`
//! - `fund_channel`
//! - `close_channel`
//!
//! All the functions do the necessary validations using the DB and then post the corresponding transaction
//! into the `ActionQueue`.
//! The functions return immediately, but provide futures that can be awaited in case the callers wishes to await the on-chain
//! confirmation of the corresponding operation.
//!
//! ## `node`
//! Submodule containing high-level on-chain actions in the `NodeActions` trait, which related to HOPR node itself.
//! - `withdraw`

use async_lock::RwLock;
use chain_db::traits::HoprCoreEthereumDbActions;
use hopr_primitive_types::primitives::Address;
use std::sync::Arc;

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
