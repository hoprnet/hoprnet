//! Contains high-level Core-Ethereum traits that translate to on-chain transactions
//!
//! ## Actions
//! The main concept is an "action", which a node can perform and results into an on-chain
//! operation. These actions are all on the external interface of this crate which is represented
//! by the [ChainActions<Db>] type.
//! There are 3 classes of actions implemented in submodules of this crate:
//! - [channel actions](channels)
//! - [ticket redeem actions](redeem)
//! - [node actions](node)
//!
//! Each action is represented by a method (or methods) that are imported into the [ChainActions<Db>] type
//! through a trait from the respective module (e.g. [ChannelActions](channels::ChannelActions) trait for channel actions).
//! Each action will eventually translate to an on-chain transaction.
//! An action will always return a [PendingAction](action_queue::PendingAction) future. This
//! future can be awaited or not, depending if the caller wishes to obtain the [ActionConfirmation](action_queue::ActionConfirmation)
//! of the submitted action.
//! If the action's caller  wishes to await the confirmation, this process can end in one of 3 possible states:
//!
//! 1. the action gets confirmed, meaning it has been successfully executed.
//! 2. awaiting the confirmation returns an error, which typically means a failure during action prerequisite checks,
//! an invalid state to execute the action or invalid arguments given to the action
//! 3. awaiting the confirmation times out, meaning the execution failed on-chain and the "action expectations"
//! did not yield (see below for details on how "action expectations work").
//!
//! Not awaiting the returned [PendingAction](action_queue::PendingAction) future does not give the caller any guarantees
//! on how the action has executed, although this might be perfectly fine for certain cases (fire & forget).
//!
//! ## How are actions executed and make it on-chain ?
//! Call to any [ChainAction's](ChainActions<Db>) method is eventually represented as an [Action](chain_types::actions::Action)
//! enum with parameters that already closely resemble the required on-chain transaction input for that action to be
//! The [Action](chain_types::actions::Action) enum instance is then passed via
//! an [ActionSender] into the [ActionQueue](action_queue::ActionQueue).
//! The [ActionQueue](action_queue::ActionQueue) takes care of ensuring the FIFO order of the
//! actions which is driven by a standalone [action loop](`ActionQueue::action_loop()`) and must be instantiated
//! before [ChainActions], so that it can provide it with an [ActionSender].
//!
//! ### Queueing of actions
//! The [ActionQueue](action_queue::ActionQueue) operates a MPSC queue, which picks up the [Actions](chain_types::actions::Action) submitted
//! to it one by one. With each such action it will:
//! 1. transform [Action](chain_types::actions::Action) into a [TypedTransaction](chain_types::TypedTransaction)
//! via a [PayloadGenerator](payload::PayloadGenerator<T>)
//! 2. submit the [TypedTransaction](chain_types::TypedTransaction) on-chain via a [TransactionExecutor](action_queue::TransactionExecutor)
//! 3. generate an [IndexerExpectation](action_state::IndexerExpectation) from the submitted action
//! 4. submit the [IndexerExpectation](action_state::IndexerExpectation) in an [ActionState](action_state::ActionState) implementation
//! 5. wait for expectation to be resolved (or timeout, see [ActionQueueConfig](action_queue::ActionQueueConfig)
//! and resolve the action submitter's [PendingAction](action_queue::PendingAction).
//!
//! In other words, the [ActionQueue](action_queue::ActionQueue) takes care of two important mappings:
//! 1. [Action](chain_types::actions::Action) to [TypedTransaction](chain_types::TypedTransaction)
//! 2. [Action](chain_types::actions::Action) to [IndexerExpectation](action_state::IndexerExpectation)
//! The first one makes it possible for an action to make it on-chain, the second one allows to
//! be informed of the action's result and effect.
//!
//! See the [action_queue] module for details.
//!
//! ### On-chain expectations after an action is submitted
//! The [action_state] module defines the [ActionState](action_state::ActionState) trait which is responsible
//! for registering [IndexerExpectation](action_state::IndexerExpectation) done by the [ActionQueue](action_queue::ActionQueue).
//! The implementor of the [ActionState](action_state::ActionState) should be monitoring the state of the block chain (reading
//! event logs contained inside newly mined blocks) and with each on-chain event that matches a registered expectation,
//! mark it as resolved to allow the bound action to be confirmed in the [ActionQueue](action_queue::ActionQueue).
//! Therefore, the two components which interact with an [ActionState](action_state::ActionState) implementation
//! are the [ActionQueue](action_queue::ActionQueue) and the on-chain Indexer (see `chain-indexer` crate for details).
//!
//! There's an exception on construction of an expectation for the `withdraw` [NodeAction](node::NodeActions):
//! Since the current implementation of the Indexer does not track native token transfers, but only smart contract
//! events, it is impossible to detect the native token's transfer confirmation.
//! Since the `withdraw` action is not very common, its confirmation is tracked via direct polling of the RPC
//! provider until the transaction is confirmed. The confirmation horizon is set in the [TransactionExecutor](action_queue::TransactionExecutor).
//!
//! See the [action_state] module for details.
//!
//! ## Payload generators
//! As described above, the [ActionQueue](action_queue::ActionQueue) needs a [PayloadGenerator](payload::PayloadGenerator<T>)
//! implementation to be able to translate the [Action](chain_types::actions::Action) into the [TypedTransaction](chain_types::TypedTransaction).
//! There are currently two possible ways of constructing the action's transaction:
//! - via plain EIP1559 payload
//! - via EIP1559 payload containing a SAFE transaction that embeds the actual payload
//!
//! The former one is implemented via [BasicPayloadGenerator](payload::BasicPayloadGenerator), the latter
//! is implemented in [SafePayloadGenerator](payload::SafePayloadGenerator).
//!
//! See the [payload] module for details.
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
pub struct ChainActions<Db: HoprCoreEthereumDbActions + Clone> {
    me: Address,
    db: Arc<RwLock<Db>>,
    tx_sender: ActionSender,
}

impl<Db: HoprCoreEthereumDbActions + Clone> ChainActions<Db> {
    ///! Creates new instance.
    pub fn new(me: Address, db: Arc<RwLock<Db>>, tx_sender: ActionSender) -> Self {
        Self { me, db, tx_sender }
    }

    ///! On-chain address of this node
    pub fn self_address(&self) -> Address {
        self.me
    }
}
