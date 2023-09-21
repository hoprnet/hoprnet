use async_lock::RwLock;
use async_std::channel::{bounded, Receiver, Sender};
use async_trait::async_trait;
use core_crypto::types::Hash;
use core_ethereum_db::traits::HoprCoreEthereumDbActions;
use core_types::acknowledgement::AcknowledgedTicket;
use core_types::acknowledgement::AcknowledgedTicketStatus::BeingRedeemed;
use core_types::channels::ChannelStatus::{Closed, Open, PendingToClose};
use core_types::channels::{ChannelEntry, ChannelStatus};
use std::fmt::{Display, Formatter};
use std::sync::Arc;
use utils_log::{debug, error, info, warn};
use utils_types::primitives::{Address, Balance};

use crate::errors::CoreEthereumActionsError::TransactionSubmissionFailed;
use crate::errors::Result;
use crate::transaction_queue::TransactionResult::{Failure, RedeemTicket};

#[cfg(any(not(feature = "wasm"), test))]
use async_std::task::spawn_local;

#[cfg(all(feature = "wasm", not(test)))]
use wasm_bindgen_futures::spawn_local;

/// Enumerates all possible outgoing transactions
#[derive(Clone, PartialEq, Debug)]
pub enum Transaction {
    /// Redeem the given acknowledged ticket
    RedeemTicket(AcknowledgedTicket),

    /// Open channel to the given destination with the given stake
    OpenChannel(Address, Balance),

    /// Fund channel with the given ID and amount
    FundChannel(ChannelEntry, Balance),

    /// Close channel with the given source and destination
    CloseChannel(ChannelEntry),

    /// Withdraw given balance to the given address
    Withdraw(Address, Balance),
}

impl Display for Transaction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Transaction::RedeemTicket(ack) => write!(f, "redeem tx of {ack}"),
            Transaction::OpenChannel(dst, amount) => write!(f, "open channel tx to {dst} with {amount}"),
            Transaction::FundChannel(channel, amount) => write!(
                f,
                "fund channel tx for channel from {} to {} with {amount}",
                channel.source, channel.destination
            ),
            Transaction::CloseChannel(channel) => write!(
                f,
                "closure tx of channel from {} to {}",
                channel.source, channel.destination
            ),
            Transaction::Withdraw(dst, amount) => write!(f, "withdraw tx of {amount} to {dst}"),
        }
    }
}

/// Implements execution of each `Transaction` and also **awaits** its confirmation.
/// Each operation must return the corresponding `TransactionResult` variant or `Failure`.
#[cfg_attr(test, mockall::automock)]
#[async_trait(? Send)]
pub trait TransactionExecutor {
    async fn redeem_ticket(&self, ticket: AcknowledgedTicket) -> TransactionResult;
    async fn open_channel(&self, destination: Address, balance: Balance) -> TransactionResult;
    async fn fund_channel(&self, channel_id: Hash, amount: Balance) -> TransactionResult;
    async fn close_channel_initialize(&self, src: Address, dst: Address) -> TransactionResult;
    async fn close_channel_finalize(&self, src: Address, dst: Address) -> TransactionResult;
    async fn withdraw(&self, recipient: Address, amount: Balance) -> TransactionResult;
}

/// Represents a result of an Ethereum transaction after it has been confirmed.
#[derive(Clone, Debug)]
pub enum TransactionResult {
    RedeemTicket { tx_hash: Hash },
    OpenChannel { tx_hash: Hash, channel_id: Hash },
    FundChannel { tx_hash: Hash },
    CloseChannel { tx_hash: Hash, status: ChannelStatus },
    Withdraw { tx_hash: Hash },
    Failure(String),
}

/// Notifies about completion of a transaction (success or failure).
pub type TransactionCompleted = futures::channel::oneshot::Receiver<TransactionResult>;

type TransactionFinisher = futures::channel::oneshot::Sender<TransactionResult>;

/// Sends a future Ethereum transaction into the `TransactionQueue`.
#[derive(Clone)]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct TransactionSender(Sender<(Transaction, TransactionFinisher)>);

impl TransactionSender {
    /// Delivers the future transaction into the `TransactionQueue` for processing.
    pub async fn send(&self, transaction: Transaction) -> Result<TransactionCompleted> {
        let completer = futures::channel::oneshot::channel();
        self.0
            .send((transaction, completer.0))
            .await
            .map(|_| completer.1)
            .map_err(|_| TransactionSubmissionFailed("ethereum tx queue is closed".into()))
    }
}

/// A queue of outgoing Ethereum transactions.
/// This queue awaits new transactions to arrive and calls the corresponding
/// method of the `TransactionExecutor` to execute it and await its confirmation.
pub struct TransactionQueue<Db: HoprCoreEthereumDbActions> {
    db: Arc<RwLock<Db>>,
    queue_send: Sender<(Transaction, TransactionFinisher)>,
    queue_recv: Receiver<(Transaction, TransactionFinisher)>,
    tx_exec: Arc<Box<dyn TransactionExecutor>>,
}

impl<Db: HoprCoreEthereumDbActions + 'static> TransactionQueue<Db> {
    /// Number of pending transactions in the queue
    pub const ETHEREUM_TX_QUEUE_SIZE: usize = 2048;

    /// Creates a new instance with the given `TransactionExecutor` implementation.
    pub fn new(db: Arc<RwLock<Db>>, tx_exec: Box<dyn TransactionExecutor>) -> Self {
        let (queue_send, queue_recv) = bounded(Self::ETHEREUM_TX_QUEUE_SIZE);
        Self {
            db,
            queue_send,
            queue_recv,
            tx_exec: Arc::new(tx_exec),
        }
    }

    /// Creates a new producer of transactions for this queue.
    pub fn new_sender(&self) -> TransactionSender {
        TransactionSender(self.queue_send.clone())
    }

    async fn execute_transaction(
        db: Arc<RwLock<Db>>,
        tx_exec: Arc<Box<dyn TransactionExecutor>>,
        tx: Transaction,
        tx_finisher: TransactionFinisher,
    ) {
        let tx_id = tx.to_string();

        let tx_result = match tx {
            Transaction::RedeemTicket(ack) => match ack.status {
                BeingRedeemed { .. } => {
                    let res = tx_exec.redeem_ticket(ack.clone()).await;
                    match res {
                        RedeemTicket { .. } => {
                            if let Err(e) = db.write().await.mark_redeemed(&ack).await {
                                error!("failed to mark {ack} as redeemed: {e}");
                                // Still declare the TX a success
                            }
                        }
                        Failure(_) => {
                            // TODO: if we know that the transaction failed due to on-chain execution, mark the ticket as losing here!
                        }
                        _ => panic!("invalid tx result from ticket redeem"),
                    }
                    res
                }
                _ => Failure(format!("invalid state of {ack}")),
            },

            Transaction::OpenChannel(address, stake) => tx_exec.open_channel(address, stake).await,

            Transaction::FundChannel(channel, amount) => {
                let channel_id = channel.get_id();
                if channel.status == Open {
                    tx_exec.fund_channel(channel_id, amount).await
                } else {
                    Failure(format!("cannot fund channel {channel_id} that is not opened"))
                }
            }

            Transaction::CloseChannel(channel) => {
                let channel_id = channel.get_id();
                match channel.status {
                    Open => {
                        debug!("initiating closure of {channel}");
                        tx_exec
                            .close_channel_initialize(channel.source, channel.destination)
                            .await
                    }

                    PendingToClose => {
                        debug!("finalizing closure of {channel}");
                        tx_exec
                            .close_channel_finalize(channel.source, channel.destination)
                            .await
                    }

                    Closed => {
                        warn!("channel {channel_id} is already closed");
                        TransactionResult::CloseChannel {
                            tx_hash: Hash::default(),
                            status: Closed,
                        }
                    }
                }
            }

            Transaction::Withdraw(recipient, amount) => tx_exec.withdraw(recipient, amount).await,
        };

        if let Failure(err) = &tx_result {
            error!("{tx_id} failed: {err}");
        } else {
            info!("transaction {tx_id} succeeded");
        }

        let _ = tx_finisher.send(tx_result);
    }

    /// Consumes self and runs the main queue processing loop until the queue is closed.
    pub async fn transaction_loop(self) {
        while let Ok((tx, tx_finisher)) = self.queue_recv.recv().await {
            let db_clone = self.db.clone();
            let tx_exec_clone = self.tx_exec.clone();

            spawn_local(async move { Self::execute_transaction(db_clone, tx_exec_clone, tx, tx_finisher).await });
        }
        warn!("transaction queue has finished");
    }
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use crate::transaction_queue::{TransactionExecutor, TransactionResult, TransactionSender};
    use async_trait::async_trait;
    use core_crypto::types::Hash;
    use core_types::acknowledgement::AcknowledgedTicket;
    use core_types::channels::ChannelStatus;
    use js_sys::Promise;
    use serde::{Deserialize, Serialize};
    use utils_misc::utils::wasm::js_value_to_error_msg;
    use utils_types::primitives::{Address, Balance};
    use utils_types::traits::ToHex;
    use wasm_bindgen::prelude::wasm_bindgen;
    use wasm_bindgen::JsValue;
    use wasm_bindgen_futures::JsFuture;

    #[wasm_bindgen]
    impl TransactionSender {
        #[wasm_bindgen(js_name = "clone")]
        pub fn _clone(&self) -> TransactionSender {
            self.clone()
        }
    }

    #[wasm_bindgen]
    pub struct WasmTxExecutor {
        redeem_ticket_fn: js_sys::Function,
        open_channel_fn: js_sys::Function,
        fund_channel_fn: js_sys::Function,
        close_channel_init_fn: js_sys::Function,
        close_channel_finish_fn: js_sys::Function,
        withdraw_fn: js_sys::Function,
    }

    #[wasm_bindgen]
    impl WasmTxExecutor {
        #[wasm_bindgen(constructor)]
        pub fn new(
            redeem_ticket_fn: js_sys::Function,
            open_channel_fn: js_sys::Function,
            fund_channel_fn: js_sys::Function,
            close_channel_init_fn: js_sys::Function,
            close_channel_finish_fn: js_sys::Function,
            withdraw_fn: js_sys::Function,
        ) -> Self {
            Self {
                redeem_ticket_fn,
                open_channel_fn,
                fund_channel_fn,
                close_channel_init_fn,
                close_channel_finish_fn,
                withdraw_fn,
            }
        }
    }

    async fn await_js_promise(result: Result<JsValue, JsValue>) -> Result<JsValue, String> {
        match result {
            Ok(ret) => {
                let promise = Promise::from(ret);
                match JsFuture::from(promise).await {
                    Ok(res) => Ok(res),
                    Err(e) => Err(js_value_to_error_msg(e).unwrap_or("unknown error".to_string())),
                }
            }
            Err(e) => Err(js_value_to_error_msg(e).unwrap_or("unknown error".to_string())),
        }
    }

    // TODO: update JS to return this object
    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct OpenChannelResult {
        channel_id: String,
        receipt: String,
    }

    #[async_trait(? Send)]
    impl TransactionExecutor for WasmTxExecutor {
        async fn redeem_ticket(&self, ticket: AcknowledgedTicket) -> TransactionResult {
            let wasm_ack: core_types::acknowledgement::wasm::AcknowledgedTicket = ticket.into();
            match await_js_promise(self.redeem_ticket_fn.call1(&JsValue::null(), &JsValue::from(wasm_ack))).await {
                Ok(r) => TransactionResult::RedeemTicket {
                    tx_hash: r.as_string().and_then(|s| Hash::from_hex(&s).ok()).unwrap_or_default(),
                },
                Err(e) => TransactionResult::Failure(e),
            }
        }

        async fn open_channel(&self, destination: Address, balance: Balance) -> TransactionResult {
            match await_js_promise(self.open_channel_fn.call2(
                &JsValue::null(),
                &JsValue::from(destination),
                &JsValue::from(balance),
            ))
            .await
            .and_then(|v| serde_wasm_bindgen::from_value::<OpenChannelResult>(v).map_err(|_| "parse error".to_string()))
            {
                Ok(v) => TransactionResult::OpenChannel {
                    tx_hash: Hash::from_hex(&v.receipt).unwrap_or_default(),
                    channel_id: Hash::from_hex(&v.channel_id).unwrap_or_default(),
                },
                Err(e) => TransactionResult::Failure(e),
            }
        }

        async fn fund_channel(&self, channel_id: Hash, amount: Balance) -> TransactionResult {
            match await_js_promise(self.fund_channel_fn.call2(
                &JsValue::null(),
                &JsValue::from(channel_id),
                &JsValue::from(amount),
            ))
            .await
            {
                Ok(r) => TransactionResult::FundChannel {
                    tx_hash: r.as_string().and_then(|s| Hash::from_hex(&s).ok()).unwrap_or_default(),
                },
                Err(e) => TransactionResult::Failure(e),
            }
        }

        async fn close_channel_initialize(&self, src: Address, dst: Address) -> TransactionResult {
            match await_js_promise(self.close_channel_init_fn.call2(
                &JsValue::null(),
                &JsValue::from(src),
                &JsValue::from(dst),
            ))
            .await
            {
                Ok(r) => TransactionResult::CloseChannel {
                    tx_hash: r.as_string().and_then(|s| Hash::from_hex(&s).ok()).unwrap_or_default(),
                    status: ChannelStatus::PendingToClose,
                },
                Err(e) => TransactionResult::Failure(e),
            }
        }

        async fn close_channel_finalize(&self, src: Address, dst: Address) -> TransactionResult {
            match await_js_promise(self.close_channel_finish_fn.call2(
                &JsValue::null(),
                &JsValue::from(src),
                &JsValue::from(dst),
            ))
            .await
            {
                Ok(r) => TransactionResult::CloseChannel {
                    tx_hash: r.as_string().and_then(|s| Hash::from_hex(&s).ok()).unwrap_or_default(),
                    status: ChannelStatus::Closed,
                },
                Err(e) => TransactionResult::Failure(e),
            }
        }

        async fn withdraw(&self, recipient: Address, amount: Balance) -> TransactionResult {
            match await_js_promise(self.withdraw_fn.call2(
                &JsValue::null(),
                &JsValue::from(recipient),
                &JsValue::from(amount),
            ))
            .await
            {
                Ok(r) => TransactionResult::Withdraw {
                    tx_hash: r.as_string().and_then(|s| Hash::from_hex(&s).ok()).unwrap_or_default(),
                },
                Err(e) => TransactionResult::Failure(e),
            }
        }
    }
}
