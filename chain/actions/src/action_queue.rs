//! Defines the main FIFO MPSC queue for actions - the [ActionQueue] type.
//!
//! The [ActionQueue] acts as a MPSC queue of [Actions](chain_types::actions::Action) which are executed one-by-one
//! as they are being popped up from the queue by a runner task.
use async_trait::async_trait;
use chain_types::actions::Action;
use chain_types::chain_events::ChainEventType;
use futures::channel::mpsc::{channel, Receiver, Sender};
use futures::future::Either;
use futures::{pin_mut, FutureExt, StreamExt};
use hopr_crypto_types::types::Hash;
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::future::{poll_fn, Future};
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, info, trace, warn};

use crate::action_state::{ActionState, IndexerExpectation};
use crate::errors::ChainActionsError::{ChannelAlreadyClosed, InvalidState, Timeout, TransactionSubmissionFailed};
use crate::errors::{ChainActionsError, Result};

use hopr_db_api::info::HoprDbInfoOperations;
use hopr_db_api::tickets::HoprDbTicketOperations;

#[cfg(all(feature = "prometheus", not(test)))]
use hopr_metrics::metrics::MultiCounter;

#[cfg(all(feature = "prometheus", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_COUNT_ACTIONS: MultiCounter = MultiCounter::new(
        "hopr_chain_actions_count",
        "Number of different chain actions and their results",
        &["action", "result"]
    )
    .unwrap();
}

/// Implements execution of transactions underlying each `Action`
/// Each operation returns a transaction hash and may time out.
#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait TransactionExecutor {
    /// Executes ticket redemption transaction given a ticket.
    async fn redeem_ticket(&self, ticket: AcknowledgedTicket, domain_separator: Hash) -> Result<Hash>;

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

/// Sends a future Ethereum transaction into the `ActionQueue`.
#[derive(Debug, Clone)]
pub struct ActionSender(Sender<(Action, ActionFinisher)>);

impl ActionSender {
    /// Delivers the future action into the `ActionQueue` for processing.
    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn send(&self, action: Action) -> Result<PendingAction> {
        let completer = futures::channel::oneshot::channel();
        let mut sender = self.0.clone();
        poll_fn(|cx| Pin::new(&mut sender).poll_ready(cx))
            .await
            .and_then(move |_| sender.start_send((action, completer.0)))
            .map(|_| {
                completer
                    .1
                    .map(|r| r.unwrap_or(Err(InvalidState("channel cancelled".into()))))
                    .boxed()
            })
            .map_err(|_| TransactionSubmissionFailed("ethereum tx queue is closed".into()))
    }
}

/// Configuration for the [ActionQueue]
#[derive(Debug, Clone, Copy, PartialEq, smart_default::SmartDefault, Serialize, Deserialize)]
pub struct ActionQueueConfig {
    /// Maximum time (in seconds) to wait for the action to be confirmed on-chain and indexed
    ///
    /// Defaults to 150 seconds.
    #[default(Duration::from_secs(150))]
    pub max_action_confirmation_wait: Duration,
}

struct ExecutionContext<S, TxExec>
where
    S: ActionState,
    TxExec: TransactionExecutor,
{
    action_state: Arc<S>,
    tx_exec: Arc<TxExec>,
    cfg: ActionQueueConfig,
}

// Needs manual implementation, so we don't need to impose Clone restrictions on the generic args
impl<S, TxExec> Clone for ExecutionContext<S, TxExec>
where
    S: ActionState,
    TxExec: TransactionExecutor,
{
    fn clone(&self) -> Self {
        Self {
            action_state: self.action_state.clone(),
            tx_exec: self.tx_exec.clone(),
            cfg: self.cfg,
        }
    }
}

impl<S, TxExec> ExecutionContext<S, TxExec>
where
    S: ActionState,
    TxExec: TransactionExecutor,
{
    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn execute_action(self, action: Action, channel_dst: Hash) -> Result<ActionConfirmation> {
        let expectation = match action.clone() {
            Action::RedeemTicket(ack) => match ack.status {
                AcknowledgedTicketStatus::BeingRedeemed { .. } => {
                    let tx_hash = self.tx_exec.redeem_ticket(ack.clone(), channel_dst).await?;
                    IndexerExpectation::new(
                        tx_hash,
                        move |event| matches!(event, ChainEventType::TicketRedeemed(channel, _) if ack.ticket.channel_id == channel.get_id()),
                    )
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
                if channel.status == ChannelStatus::Open {
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
                    ChannelStatus::Open | ChannelStatus::PendingToClose(_) => {
                        let tx_hash = self.tx_exec.close_incoming_channel(channel.source).await?;
                        IndexerExpectation::new(
                            tx_hash,
                            move |event| matches!(event, ChainEventType::ChannelClosed(r_channel) if r_channel.get_id() == channel.get_id()),
                        )
                    }
                    ChannelStatus::Closed => {
                        warn!("channel {} is already closed", channel.get_id());
                        return Err(ChannelAlreadyClosed);
                    }
                },
                ChannelDirection::Outgoing => match channel.status {
                    ChannelStatus::Open => {
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
                    ChannelStatus::PendingToClose(_) => {
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
                    ChannelStatus::Closed => {
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

/// A queue of [Actions](Action) to be executed.
///
/// This queue awaits new Actions to arrive, translates them into Ethereum
/// transactions via [TransactionExecutor] to execute them and await their confirmation
/// by registering their corresponding expectations in [ActionState].
pub struct ActionQueue<Db, S, TxExec>
where
    Db: HoprDbInfoOperations + HoprDbTicketOperations + Send + Sync,
    S: ActionState + Send + Sync,
    TxExec: TransactionExecutor + Send + Sync,
{
    db: Db,
    queue_send: Sender<(Action, ActionFinisher)>,
    queue_recv: Receiver<(Action, ActionFinisher)>,
    ctx: ExecutionContext<S, TxExec>,
}

impl<Db, S, TxExec> ActionQueue<Db, S, TxExec>
where
    Db: HoprDbInfoOperations + HoprDbTicketOperations + Clone + Send + Sync + 'static,
    S: ActionState + Send + Sync + 'static,
    TxExec: TransactionExecutor + Send + Sync + 'static,
{
    /// Number of pending transactions in the queue
    pub const ACTION_QUEUE_SIZE: usize = 2048;

    /// Creates a new instance with the given [TransactionExecutor] and [ActionState] implementations.
    pub fn new(db: Db, action_state: S, tx_exec: TxExec, cfg: ActionQueueConfig) -> Self {
        let (queue_send, queue_recv) = channel(Self::ACTION_QUEUE_SIZE);
        Self {
            db,
            ctx: ExecutionContext {
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
    ///
    /// The method will panic if Channel Domain Separator is not yet populated in the DB.
    #[allow(clippy::async_yields_async)]
    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn start(mut self) -> async_std::task::JoinHandle<()> {
        async_std::task::spawn(async move {
            while let Some((act, tx_finisher)) = self.queue_recv.next().await {
                // Some minimum separation to avoid batching txs
                futures_timer::Delay::new(Duration::from_millis(100)).await;

                let exec_context = self.ctx.clone();
                let db_clone = self.db.clone();
                let channel_dst = self
                    .db
                    .get_indexer_data(None)
                    .await
                    .map_err(ChainActionsError::from)
                    .and_then(|data| data.channels_dst.ok_or(InvalidState("missing channels dst".into())))
                    .unwrap();

                async_std::task::spawn(async move {
                    let act_id = act.to_string();
                    let act_name: &'static str = (&act).into();
                    trace!("start executing {act_id} ({act_name})");

                    let tx_result = exec_context.execute_action(act.clone(), channel_dst).await;
                    match &tx_result {
                        Ok(confirmation) => {
                            info!("successful {confirmation}");

                            #[cfg(all(feature = "prometheus", not(test)))]
                            METRIC_COUNT_ACTIONS.increment(&[act_name, "success"]);
                        }
                        Err(err) => {
                            // On error in Ticket redeem action, we also need to reset ack ticket state
                            if let Action::RedeemTicket(ack) = act {
                                error!("marking the acknowledged ticket as untouched - redeem action failed: {err}");

                                if let Err(e) = db_clone
                                    .update_ticket_states((&ack).into(), AcknowledgedTicketStatus::Untouched)
                                    .await
                                {
                                    error!("cannot mark {ack} as untouched: {e}");
                                }
                            }

                            // Timeout are accounted in different metric
                            if let Timeout = err {
                                error!("timeout while waiting for confirmation of {act_id}");

                                #[cfg(all(feature = "prometheus", not(test)))]
                                METRIC_COUNT_ACTIONS.increment(&[act_name, "timeout"]);
                            } else {
                                error!("{act_id} failed: {err}");

                                #[cfg(all(feature = "prometheus", not(test)))]
                                METRIC_COUNT_ACTIONS.increment(&[act_name, "failure"]);
                            }
                        }
                    }

                    let _ = tx_finisher.send(tx_result);
                });
            }
            error!("action queue has finished, it should be running for the node to be able to process chain actions");
        })
    }
}
