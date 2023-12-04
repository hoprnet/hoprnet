use async_lock::RwLock;
use async_std::channel::{bounded, Receiver, Sender};
use async_trait::async_trait;
use core_crypto::types::Hash;
use core_ethereum_db::traits::HoprCoreEthereumDbActions;
use core_ethereum_types::actions::Action;
use core_ethereum_types::chain_events::ChainEventType;
use core_types::acknowledgement::AcknowledgedTicketStatus;
use core_types::announcement::AnnouncementData;
use core_types::{
    acknowledgement::{AcknowledgedTicket, AcknowledgedTicketStatus::BeingRedeemed},
    channels::{
        ChannelDirection,
        ChannelStatus::{Closed, Open, PendingToClose},
    },
};
use futures::future::Either;
use futures::{pin_mut, FutureExt};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use utils_log::{debug, error, info, warn};
use utils_types::primitives::{Address, Balance};

use crate::action_state::{ActionState, IndexerExpectation};
use crate::errors::CoreEthereumActionsError::{
    ChannelAlreadyClosed, InvalidState, Timeout, TransactionSubmissionFailed,
};
use crate::errors::Result;

#[cfg(any(not(feature = "wasm"), test))]
use async_std::task::spawn_local;

#[cfg(all(feature = "wasm", not(test)))]
use wasm_bindgen_futures::spawn_local;

#[cfg(all(feature = "prometheus", not(test)))]
use utils_metrics::metrics::SimpleCounter;

#[cfg(all(feature = "prometheus", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_COUNT_SUCCESSFUL_ACTIONS: SimpleCounter = SimpleCounter::new(
        "core_ethereum_counter_successful_actions",
        "Number of successful actions"
    )
    .unwrap();
    static ref METRIC_COUNT_FAILED_ACTIONS: SimpleCounter = SimpleCounter::new(
        "core_ethereum_counter_failed_actions",
        "Number of failed actions"
    )
    .unwrap();
    static ref METRIC_COUNT_TIMEOUT_ACTIONS: SimpleCounter = SimpleCounter::new(
        "core_ethereum_counter_timeout_actions",
        "Number of timed out actions"
    )
    .unwrap();
}

/// Implements execution of transactions underlying each `Action`
/// Each operation returns a transaction hash and may timeout.
#[cfg_attr(test, mockall::automock)]
#[async_trait(? Send)]
pub trait TransactionExecutor {
    /// Executes ticket redemption transaction given a ticket.
    async fn redeem_ticket(&self, ticket: AcknowledgedTicket) -> Result<Hash>;

    /// Executes channel funding transaction (or channel opening) to the given `destination` and stake.
    /// Channel funding and channel opening are both same transactions.
    async fn fund_channel(&self, destination: Address, balance: Balance) -> Result<Hash>;

    /// Initiates closure of an outgoing channel.
    async fn initiate_outgoing_channel_closure(&self, dst: Address) -> Result<Hash>;

    /// Finalizes closure of an outgoing channel.
    async fn finalize_outgoing_channel_closure(&self, dst: Address) -> Result<Hash>;

    /// Closes incoming channel.
    async fn close_incoming_channel(&self, src: Address) -> Result<Hash>;

    /// Performs withdrawal of a certain amount from an address.
    /// Note that this transaction is typically awaited via polling and is not tracked
    /// by the Indexer.
    async fn withdraw(&self, recipient: Address, amount: Balance) -> Result<Hash>;

    /// Announces the node on-chain given the `AnnouncementData`
    async fn announce(&self, data: AnnouncementData) -> Result<Hash>;

    /// Registers Safe with the node.
    async fn register_safe(&self, safe_address: Address) -> Result<Hash>;
}

/// Represents confirmation of the `Action` execution.
#[derive(Debug, Clone, PartialEq)]
pub struct ActionConfirmation {
    /// Hash of the transaction that executed this action
    pub tx_hash: Hash,

    /// Corresponding chain event if any
    pub event: Option<ChainEventType>,

    /// Action that was executed
    pub action: Action,
}

impl Display for ActionConfirmation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} confirmed in tx {}", self.action, self.tx_hash)
    }
}

/// Notifies about completion of a transaction (success or failure).
pub type PendingAction = Pin<Box<dyn Future<Output = Result<ActionConfirmation>> + Send>>;

/// Future that resolves once the transaction has been confirmed by the Indexer.
type ActionFinisher = futures::channel::oneshot::Sender<Result<ActionConfirmation>>;

/// Sends a future Ethereum transaction into the `TransactionQueue`.
#[derive(Clone)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct ActionSender(Sender<(Action, ActionFinisher)>);

impl ActionSender {
    /// Delivers the future action into the `ActionQueue` for processing.
    pub async fn send(&self, action: Action) -> Result<PendingAction> {
        let completer = futures::channel::oneshot::channel();
        self.0
            .send((action, completer.0))
            .await
            .map(|_| {
                completer
                    .1
                    .map(|r| r.unwrap_or(Err(InvalidState("channel cancelled".into()))))
                    .boxed()
            })
            .map_err(|_| TransactionSubmissionFailed("ethereum tx queue is closed".into()))
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActionQueueConfig {
    /// Maximum time (in seconds) to wait for the action to be confirmed on-chain and indexed
    /// Defaults to 150 seconds.
    pub max_action_confirmation_wait: Duration,
}

impl Default for ActionQueueConfig {
    fn default() -> Self {
        Self {
            max_action_confirmation_wait: Duration::from_secs(150),
        }
    }
}

struct ExecutionContext<Db, S, TxExec>
where
    Db: HoprCoreEthereumDbActions,
    S: ActionState,
    TxExec: TransactionExecutor,
{
    db: Arc<RwLock<Db>>,
    action_state: Arc<S>,
    tx_exec: Arc<TxExec>,
    cfg: ActionQueueConfig,
}

// Needs manual implementation, so we don't need to impose Clone restrictions on the generic args
impl<Db, S, TxExec> Clone for ExecutionContext<Db, S, TxExec>
where
    Db: HoprCoreEthereumDbActions,
    S: ActionState,
    TxExec: TransactionExecutor,
{
    fn clone(&self) -> Self {
        Self {
            db: self.db.clone(),
            action_state: self.action_state.clone(),
            tx_exec: self.tx_exec.clone(),
            cfg: self.cfg.clone(),
        }
    }
}

impl<Db, S, TxExec> ExecutionContext<Db, S, TxExec>
where
    Db: HoprCoreEthereumDbActions,
    S: ActionState,
    TxExec: TransactionExecutor,
{
    pub async fn execute_action(self, action: Action) -> Result<ActionConfirmation> {
        let expectation = match action.clone() {
            Action::RedeemTicket(mut ack) => match ack.status {
                BeingRedeemed { .. } => {
                    match self.tx_exec.redeem_ticket(ack.clone()).await {
                        Ok(tx_hash) => IndexerExpectation::new(
                            tx_hash,
                            move |event| matches!(event, ChainEventType::TicketRedeemed(_, Some(ticket)) if ack.eq(ticket)),
                        ),
                        Err(e) => {
                            // TODO: once we can distinguish EVM execution failure from `e`, we can mark ticket as losing instead

                            error!("marking the acknowledged ticket as untouched - redeem tx failed: {e}");
                            ack.status = AcknowledgedTicketStatus::Untouched;
                            if let Err(e) = self.db.write().await.update_acknowledged_ticket(&ack).await {
                                error!("cannot mark {ack} as untouched: {e}");
                            }

                            return Err(e);
                        }
                    }
                }
                _ => return Err(InvalidState(ack.to_string())),
            },

            Action::OpenChannel(address, stake) => {
                let tx_hash = self.tx_exec.fund_channel(address, stake).await?;
                IndexerExpectation::new(
                    tx_hash,
                    move |event| matches!(event, ChainEventType::ChannelOpened(channel) if channel.destination == address),
                )
            }

            Action::FundChannel(channel, amount) => {
                if channel.status == Open {
                    let tx_hash = self.tx_exec.fund_channel(channel.destination, amount).await?;
                    IndexerExpectation::new(
                        tx_hash,
                        move |event| matches!(event, ChainEventType::ChannelBalanceIncreased(r_channel, diff) if r_channel.get_id() == channel.get_id() && amount.eq(diff)),
                    )
                } else {
                    return Err(InvalidState(format!("cannot fund {channel} because it is not opened")));
                }
            }

            Action::CloseChannel(channel, direction) => match direction {
                ChannelDirection::Incoming => match channel.status {
                    Open | PendingToClose => {
                        let tx_hash = self.tx_exec.close_incoming_channel(channel.source).await?;
                        IndexerExpectation::new(
                            tx_hash,
                            move |event| matches!(event, ChainEventType::ChannelClosed(r_channel) if r_channel.get_id() == channel.get_id()),
                        )
                    }
                    Closed => {
                        warn!("channel {} is already closed", channel.get_id());
                        return Err(ChannelAlreadyClosed);
                    }
                },
                ChannelDirection::Outgoing => match channel.status {
                    Open => {
                        debug!("initiating closure of {channel}");
                        let tx_hash = self
                            .tx_exec
                            .initiate_outgoing_channel_closure(channel.destination)
                            .await?;
                        IndexerExpectation::new(
                            tx_hash,
                            move |event| matches!(event, ChainEventType::ChannelClosureInitiated(r_channel) if r_channel.get_id() == channel.get_id()),
                        )
                    }

                    PendingToClose => {
                        debug!("finalizing closure of {channel}");
                        let tx_hash = self
                            .tx_exec
                            .finalize_outgoing_channel_closure(channel.destination)
                            .await?;
                        IndexerExpectation::new(
                            tx_hash,
                            move |event| matches!(event, ChainEventType::ChannelClosed(r_channel) if r_channel.get_id() == channel.get_id()),
                        )
                    }

                    Closed => {
                        warn!("channel {} is already closed", channel.get_id());
                        return Err(ChannelAlreadyClosed);
                    }
                },
            },

            Action::Withdraw(recipient, amount) => {
                // Withdrawal is not awaited via the Indexer, but polled for completion
                // so no indexer event stream expectation awaiting is needed.
                // So simply return once the future completes
                return Ok(ActionConfirmation {
                    tx_hash: self.tx_exec.withdraw(recipient, amount).await?,
                    event: None,
                    action: action.clone(),
                });
            }
            Action::Announce(data) => {
                let tx_hash = self.tx_exec.announce(data.clone()).await?;
                IndexerExpectation::new(
                    tx_hash,
                    move |event| matches!(event, ChainEventType::Announcement{multiaddresses,..} if multiaddresses.contains(data.multiaddress())),
                )
            }
            Action::RegisterSafe(safe_address) => {
                let tx_hash = self.tx_exec.register_safe(safe_address).await?;
                IndexerExpectation::new(
                    tx_hash,
                    move |event| matches!(event, ChainEventType::NodeSafeRegistered(address) if safe_address.eq(address)),
                )
            }
        };

        let tx_hash = expectation.tx_hash;
        debug!("action {action} submitted via tx {tx_hash}, registering expectation");

        // Register new expectation and await it with timeout
        let confirmation = self.action_state.register_expectation(expectation).await?.fuse();
        let timeout = futures_timer::Delay::new(self.cfg.max_action_confirmation_wait).fuse();

        pin_mut!(confirmation, timeout);

        match futures::future::select(confirmation, timeout).await {
            Either::Left((Ok(chain_event), _)) => Ok(ActionConfirmation {
                tx_hash: chain_event.tx_hash,
                event: Some(chain_event.event_type),
                action,
            }),
            Either::Left((Err(_), _)) => {
                self.action_state.unregister_expectation(tx_hash).await;
                Err(InvalidState("action expectation was removed before resolving".into()))
            }
            Either::Right(_) => {
                self.action_state.unregister_expectation(tx_hash).await;
                Err(Timeout)
            }
        }
    }
}

/// A queue of outgoing Ethereum transactions.
/// This queue awaits new transactions to arrive and calls the corresponding
/// method of the `TransactionExecutor` to execute it and await its confirmation.
pub struct ActionQueue<Db, S, TxExec>
where
    Db: HoprCoreEthereumDbActions,
    S: ActionState,
    TxExec: TransactionExecutor,
{
    queue_send: Sender<(Action, ActionFinisher)>,
    queue_recv: Receiver<(Action, ActionFinisher)>,
    ctx: ExecutionContext<Db, S, TxExec>,
}

impl<Db, S, TxExec> ActionQueue<Db, S, TxExec>
where
    Db: HoprCoreEthereumDbActions + 'static,
    TxExec: TransactionExecutor + 'static,
    S: ActionState + 'static,
{
    /// Number of pending transactions in the queue
    pub const ACTION_QUEUE_SIZE: usize = 2048;

    /// Creates a new instance with the given `TransactionExecutor` implementation.
    pub fn new(db: Arc<RwLock<Db>>, action_state: S, tx_exec: TxExec, cfg: ActionQueueConfig) -> Self {
        let (queue_send, queue_recv) = bounded(Self::ACTION_QUEUE_SIZE);
        Self {
            ctx: ExecutionContext {
                db,
                action_state: Arc::new(action_state),
                tx_exec: Arc::new(tx_exec),
                cfg,
            },
            queue_send,
            queue_recv,
        }
    }

    /// Creates a new producer of actions for this queue.
    pub fn new_sender(&self) -> ActionSender {
        ActionSender(self.queue_send.clone())
    }

    /// Clones the `ActionState` implementation.
    pub fn action_state(&self) -> Arc<S> {
        self.ctx.action_state.clone()
    }

    /// Consumes self and runs the main queue processing loop until the queue is closed.
    pub async fn transaction_loop(self) {
        while let Ok((act, tx_finisher)) = self.queue_recv.recv().await {
            let exec_context = self.ctx.clone();
            spawn_local(async move {
                let act_id = act.to_string();
                debug!("start executing {act_id}");

                let tx_result = exec_context.execute_action(act).await;
                match &tx_result {
                    Ok(confirmation) => {
                        info!("successful {confirmation}");

                        #[cfg(all(feature = "prometheus", not(test)))]
                        METRIC_COUNT_SUCCESSFUL_ACTIONS.increment();
                    }
                    Err(Timeout) => {
                        error!("timeout while waiting for confirmation of {act_id}");

                        #[cfg(all(feature = "prometheus", not(test)))]
                        METRIC_COUNT_TIMEOUT_ACTIONS.increment();
                    }
                    Err(err) => {
                        error!("{act_id} failed: {err}");

                        #[cfg(all(feature = "prometheus", not(test)))]
                        METRIC_COUNT_FAILED_ACTIONS.increment();
                    }
                }

                let _ = tx_finisher.send(tx_result);
            });
        }
        warn!("transaction queue has finished");
    }
}
