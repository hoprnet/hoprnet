use crate::errors::CoreEthereumError::TransactionSubmissionFailed;
use crate::errors::Result;
use async_lock::RwLock;
use async_std::channel::{bounded, Receiver, Sender};
use async_trait::async_trait;
use core_ethereum_db::traits::HoprCoreEthereumDbActions;
use core_types::acknowledgement::AcknowledgedTicket;
use core_types::acknowledgement::AcknowledgedTicketStatus::BeingRedeemed;
use std::sync::Arc;
use utils_log::{error, warn};

/// Enumerates all possible outgoing transactions
#[derive(Clone, PartialEq, Debug)]
pub enum Transaction {
    RedeemTicket(AcknowledgedTicket),
}

/// Implements execution of each `Transaction` and also **awaits** its confirmation.
#[cfg_attr(test, mockall::automock)]
#[async_trait(? Send)]
pub trait TransactionExecutor {
    async fn redeem_ticket(&self, ticket: AcknowledgedTicket) -> Result<()>;
}

/// Represents a result of an Ethereum transaction after it has been confirmed.
#[derive(Clone, Copy, Debug)]
pub struct TransactionResult;

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
    tx_exec: Box<dyn TransactionExecutor>,
}

impl<Db: HoprCoreEthereumDbActions> TransactionQueue<Db> {
    /// Number of pending transactions in the queue
    pub const ETHEREUM_TX_QUEUE_SIZE: usize = 2048;

    /// Creates a new instance with the given `TransactionExecutor` implementation.
    pub fn new(db: Arc<RwLock<Db>>, tx_exec: Box<dyn TransactionExecutor>) -> Self {
        let (queue_send, queue_recv) = bounded(Self::ETHEREUM_TX_QUEUE_SIZE);
        Self {
            db,
            queue_send,
            queue_recv,
            tx_exec,
        }
    }

    /// Creates a new producer of transactions for this queue.
    pub fn new_sender(&self) -> TransactionSender {
        TransactionSender(self.queue_send.clone())
    }

    /// Runs the main queue processing loop until the queue is closed.
    pub async fn transaction_loop(&self) {
        while let Ok(req) = self.queue_recv.recv().await {
            match req.0 {
                Transaction::RedeemTicket(ack) => {
                    match ack.status {
                        BeingRedeemed { .. } => {
                            if let Err(e) = self.tx_exec.redeem_ticket(ack.clone()).await {
                                error!("redeem tx of {ack} failed: {e}")
                            } else if let Err(e) = self.db.write().await.mark_redeemed(&ack).await {
                                error!("failed to mark {ack} as redeemed: {e}");
                            }
                        }
                        _ => error!("invalid state of {ack}"),
                    };
                    let _ = req.1.send(TransactionResult);
                }
            }
        }
        warn!("transaction queue has finished");
    }
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use crate::errors::CoreEthereumError::TransactionSubmissionFailed;
    use crate::transaction_queue::TransactionExecutor;
    use async_trait::async_trait;
    use core_types::acknowledgement::AcknowledgedTicket;
    use js_sys::Promise;
    use utils_log::debug;
    use utils_misc::utils::wasm::js_value_to_error_msg;
    use wasm_bindgen::prelude::wasm_bindgen;
    use wasm_bindgen::JsValue;
    use wasm_bindgen_futures::JsFuture;

    #[wasm_bindgen]
    pub struct WasmTxExecutor {
        redeem_ticket_fn: js_sys::Function,
    }

    #[wasm_bindgen]
    impl WasmTxExecutor {
        #[wasm_bindgen(constructor)]
        pub fn new(redeem_ticket_fn: js_sys::Function) -> Self {
            Self { redeem_ticket_fn }
        }
    }

    #[async_trait(? Send)]
    impl TransactionExecutor for WasmTxExecutor {
        async fn redeem_ticket(&self, ticket: AcknowledgedTicket) -> crate::errors::Result<()> {
            let wasm_ack: core_types::acknowledgement::wasm::AcknowledgedTicket = ticket.into();
            match self.redeem_ticket_fn.call1(&JsValue::null(), &JsValue::from(wasm_ack)) {
                Ok(ret) => {
                    let promise = Promise::from(ret);
                    match JsFuture::from(promise).await {
                        Ok(res) => {
                            debug!("JS redeem transaction completed: {:?}", res.as_string());
                            Ok(())
                        }
                        Err(e) => Err(TransactionSubmissionFailed(
                            js_value_to_error_msg(e).unwrap_or("unknown error".to_string()),
                        )),
                    }
                }
                Err(e) => Err(TransactionSubmissionFailed(
                    js_value_to_error_msg(e).unwrap_or("unknown error".to_string()),
                )),
            }
        }
    }
}
