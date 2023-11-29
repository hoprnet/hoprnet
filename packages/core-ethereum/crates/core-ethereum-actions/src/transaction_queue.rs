use async_lock::RwLock;
use async_std::channel::{bounded, Receiver, Sender};
use async_trait::async_trait;
use core_crypto::types::Hash;
use core_ethereum_db::traits::HoprCoreEthereumDbActions;
use core_ethereum_types::actions::Action;
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
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::Arc;
use utils_log::{debug, error, info, warn};
use utils_types::primitives::{Address, Balance};

use crate::errors::CoreEthereumActionsError::TransactionSubmissionFailed;
use crate::errors::{CoreEthereumActionsError, Result};
use crate::transaction_queue::TransactionResult::Failure;

#[cfg(any(not(feature = "wasm"), test))]
use async_std::task::{sleep, spawn_local};

#[cfg(all(feature = "wasm", not(test)))]
use wasm_bindgen_futures::spawn_local;

#[cfg(all(feature = "wasm", not(test)))]
use gloo_timers::future::sleep;

#[cfg(all(feature = "prometheus", not(test)))]
use utils_metrics::metrics::SimpleCounter;

#[cfg(all(feature = "prometheus", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_COUNT_SUCCESSFUL_TXS: SimpleCounter = SimpleCounter::new(
        "core_ethereum_counter_successful_transactions",
        "Number of successful transactions"
    )
    .unwrap();
    static ref METRIC_COUNT_FAILED_TXS: SimpleCounter = SimpleCounter::new(
        "core_ethereum_counter_failed_transactions",
        "Number of failed transactions"
    )
    .unwrap();
    static ref METRIC_COUNT_TIMEOUT_TXS: SimpleCounter = SimpleCounter::new(
        "core_ethereum_counter_timeout_transactions",
        "Number of timed out transactions"
    )
    .unwrap();
}

/// Implements execution of each `Transaction` and also **awaits** its confirmation.
/// Each operation must return the corresponding `TransactionResult` variant or `Failure`.
#[cfg_attr(test, mockall::automock)]
#[async_trait(? Send)]
pub trait TransactionExecutor {
    async fn redeem_ticket(&self, ticket: AcknowledgedTicket) -> TransactionResult;
    async fn fund_channel(&self, destination: Address, balance: Balance) -> TransactionResult;
    async fn initiate_outgoing_channel_closure(&self, dst: Address) -> TransactionResult;
    async fn finalize_outgoing_channel_closure(&self, dst: Address) -> TransactionResult;
    async fn close_incoming_channel(&self, src: Address) -> TransactionResult;
    async fn withdraw(&self, recipient: Address, amount: Balance) -> TransactionResult;
    async fn announce(&self, data: AnnouncementData) -> TransactionResult;
    async fn register_safe(&self, safe_address: Address) -> TransactionResult;
}

/// Represents a result of an Ethereum transaction after it has been confirmed.
#[derive(Clone, Debug)]
pub enum TransactionResult {
    TicketRedeemed { tx_hash: Hash },
    ChannelFunded { tx_hash: Hash },
    ChannelClosureInitiated { tx_hash: Hash },
    ChannelClosed { tx_hash: Hash },
    Withdrawn { tx_hash: Hash },
    Announced { tx_hash: Hash },
    SafeRegistered { tx_hash: Hash },
    Failure(String),
}

impl From<CoreEthereumActionsError> for TransactionResult {
    fn from(value: CoreEthereumActionsError) -> Self {
        Failure(format!("tx failed with local error: {value}"))
    }
}

/// Notifies about completion of a transaction (success or failure).
pub type TransactionCompleted = Pin<Box<dyn Future<Output = TransactionResult> + Send>>;

/// Future that resolves once the transaction has been confirmed by the Indexer.
type TransactionFinisher = futures::channel::oneshot::Sender<TransactionResult>;

/// Sends a future Ethereum transaction into the `TransactionQueue`.
#[derive(Clone)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct TransactionSender(Sender<(Action, TransactionFinisher)>);

impl TransactionSender {
    /// Delivers the future transaction into the `TransactionQueue` for processing.
    pub async fn send(&self, transaction: Action) -> Result<TransactionCompleted> {
        let completer = futures::channel::oneshot::channel();
        self.0
            .send((transaction, completer.0))
            .await
            .map(|_| {
                completer
                    .1
                    .map(|r| r.unwrap_or(TransactionResult::Failure("channel cancelled".into())))
                    .boxed()
            })
            .map_err(|_| TransactionSubmissionFailed("ethereum tx queue is closed".into()))
    }
}

/// A queue of outgoing Ethereum transactions.
/// This queue awaits new transactions to arrive and calls the corresponding
/// method of the `TransactionExecutor` to execute it and await its confirmation.
pub struct TransactionQueue<Db: HoprCoreEthereumDbActions> {
    db: Arc<RwLock<Db>>,
    queue_send: Sender<(Action, TransactionFinisher)>,
    queue_recv: Receiver<(Action, TransactionFinisher)>,
    tx_exec: Rc<Box<dyn TransactionExecutor>>, // TODO: Make this Arc once TransactionExecutor is Send
}

impl<Db: HoprCoreEthereumDbActions + 'static> TransactionQueue<Db> {
    /// Number of pending transactions in the queue
    pub const ETHEREUM_TX_QUEUE_SIZE: usize = 2048;

    /// Maximum time (in seconds) to wait for the transaction to be confirmed on-chain and indexed
    pub const MAX_TX_CONFIRMATION_WAIT_SECS: usize = 180;

    /// Creates a new instance with the given `TransactionExecutor` implementation.
    pub fn new(db: Arc<RwLock<Db>>, tx_exec: Box<dyn TransactionExecutor>) -> Self {
        let (queue_send, queue_recv) = bounded(Self::ETHEREUM_TX_QUEUE_SIZE);
        Self {
            db,
            queue_send,
            queue_recv,
            tx_exec: Rc::new(tx_exec),
        }
    }

    /// Creates a new producer of transactions for this queue.
    pub fn new_sender(&self) -> TransactionSender {
        TransactionSender(self.queue_send.clone())
    }

    async fn execute_transaction(
        db: Arc<RwLock<Db>>,
        tx_exec: Rc<Box<dyn TransactionExecutor>>,
        tx: Action,
    ) -> TransactionResult {
        match tx {
            Action::RedeemTicket(mut ack) => match &ack.status {
                BeingRedeemed { .. } => {
                    let res = tx_exec.redeem_ticket(ack.clone()).await;
                    if let Failure(e) = &res {
                        // TODO: once we can distinguish EVM execution failure from `e`, we can mark ticket as losing instead

                        error!("marking the acknowledged ticket as untouched - redeem tx failed: {e}");
                        ack.status = AcknowledgedTicketStatus::Untouched;
                        if let Err(e) = db.write().await.update_acknowledged_ticket(&ack).await {
                            error!("cannot mark {ack} as untouched: {e}");
                        }
                    }
                    res
                }
                _ => Failure(format!("invalid state of {ack}")),
            },

            Action::OpenChannel(address, stake) => tx_exec.fund_channel(address, stake).await,

            Action::FundChannel(channel, amount) => {
                if channel.status == Open {
                    tx_exec.fund_channel(channel.destination, amount).await
                } else {
                    Failure(format!("cannot fund {channel} because it is not opened"))
                }
            }

            Action::CloseChannel(channel, direction) => match direction {
                ChannelDirection::Incoming => match channel.status {
                    Open | PendingToClose => tx_exec.close_incoming_channel(channel.source).await,
                    Closed => {
                        warn!("channel {} is already closed", channel.get_id());
                        TransactionResult::ChannelClosed {
                            tx_hash: Hash::default(),
                        }
                    }
                },
                ChannelDirection::Outgoing => match channel.status {
                    Open => {
                        debug!("initiating closure of {channel}");
                        tx_exec.initiate_outgoing_channel_closure(channel.destination).await
                    }

                    PendingToClose => {
                        debug!("finalizing closure of {channel}");
                        tx_exec.finalize_outgoing_channel_closure(channel.destination).await
                    }

                    Closed => {
                        warn!("channel {} is already closed", channel.get_id());
                        TransactionResult::ChannelClosed {
                            tx_hash: Hash::default(),
                        }
                    }
                },
            },

            Action::Withdraw(recipient, amount) => tx_exec.withdraw(recipient, amount).await,
            Action::Announce(data) => tx_exec.announce(data).await,
            Action::RegisterSafe(safe_address) => tx_exec.register_safe(safe_address).await,
        }
    }

    /// Consumes self and runs the main queue processing loop until the queue is closed.
    pub async fn transaction_loop(self) {
        while let Ok((tx, tx_finisher)) = self.queue_recv.recv().await {
            let db_clone = self.db.clone();
            let tx_exec_clone = self.tx_exec.clone();
            let tx_id = tx.to_string();

            spawn_local(async move {
                let tx_fut = Self::execute_transaction(db_clone, tx_exec_clone, tx).fuse();

                // Put an upper bound on the transaction to get confirmed and indexed
                let timeout = sleep(std::time::Duration::from_secs(
                    Self::MAX_TX_CONFIRMATION_WAIT_SECS as u64,
                ))
                .fuse();

                pin_mut!(timeout, tx_fut);

                debug!("start executing {tx_id}");
                let tx_result = match futures::future::select(tx_fut, timeout).await {
                    Either::Left((result, _)) => {
                        if let Failure(err) = &result {
                            error!("{tx_id} failed: {err}");

                            #[cfg(all(feature = "prometheus", not(test)))]
                            METRIC_COUNT_FAILED_TXS.increment();
                        } else {
                            info!("transaction {tx_id} succeeded");

                            #[cfg(all(feature = "prometheus", not(test)))]
                            METRIC_COUNT_SUCCESSFUL_TXS.increment();
                        }
                        result
                    }
                    Either::Right((_, _)) => {
                        #[cfg(all(feature = "prometheus", not(test)))]
                        METRIC_COUNT_TIMEOUT_TXS.increment();

                        Failure(format!(
                            "awaiting for confirmation of {tx_id} timed out after {} seconds",
                            Self::MAX_TX_CONFIRMATION_WAIT_SECS
                        ))
                    }
                };

                let _ = tx_finisher.send(tx_result);
            });
        }
        warn!("transaction queue has finished");
    }
}
