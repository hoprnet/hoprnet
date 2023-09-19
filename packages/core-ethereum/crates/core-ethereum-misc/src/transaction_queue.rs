use crate::errors::CoreEthereumError::TransactionSubmissionFailed;
use crate::errors::Result;
use crate::transaction_queue::TransactionResult::Failure;
use async_lock::RwLock;
use async_std::channel::{bounded, Receiver, Sender};
use async_trait::async_trait;
use core_crypto::types::Hash;
use core_ethereum_db::traits::HoprCoreEthereumDbActions;
use core_types::acknowledgement::AcknowledgedTicket;
use core_types::acknowledgement::AcknowledgedTicketStatus::BeingRedeemed;
use core_types::channels::ChannelStatus;
use core_types::channels::ChannelStatus::{Closed, Open, PendingToClose};
use std::sync::Arc;
use utils_log::{error, info, warn};
use utils_types::primitives::{Address, Balance};

/// Enumerates all possible outgoing transactions
#[derive(Clone, PartialEq, Debug)]
pub enum Transaction {
    /// Redeem the given acknowledged ticket
    RedeemTicket(AcknowledgedTicket),

    /// Open channel to the given destination with the given stake
    OpenChannel(Address, Balance),

    /// Fund channel with the given ID and amount
    FundChannel(Hash, Balance),

    /// Close channel with the given source and destination
    CloseChannel(Address, Address),

    /// Withdraw given balance to the given address
    Withdraw(Address, Balance),
}

/// Implements execution of each `Transaction` and also **awaits** its confirmation.
#[cfg_attr(test, mockall::automock)]
#[async_trait(? Send)]
pub trait TransactionExecutor {
    async fn redeem_ticket(&self, ticket: AcknowledgedTicket) -> Result<Hash>;
    async fn open_channel(&self, destination: Address, balance: Balance) -> Result<(Hash, Hash)>;
    async fn fund_channel(&self, channel_id: Hash, amount: Balance) -> Result<Hash>;
    async fn close_channel_initialize(&self, src: Address, dst: Address) -> Result<Hash>;
    async fn close_channel_finalize(&self, src: Address, dst: Address) -> Result<Hash>;
    async fn withdraw(&self, recipient: Address, amount: Balance) -> Result<Hash>;
}

/// Represents a result of an Ethereum transaction after it has been confirmed.
#[derive(Clone, Copy, Debug)]
pub enum TransactionResult {
    RedeemTicket { tx_hash: Hash },
    OpenChannel { tx_hash: Hash, channel_id: Hash },
    FundChannel { tx_hash: Hash },
    CloseChannel { tx_hash: Hash, status: ChannelStatus },
    Withdraw { tx_hash: Hash },
    Failure,
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
            let mut tx_result = Failure;
            match req.0 {
                Transaction::RedeemTicket(ack) => {
                    match ack.status {
                        BeingRedeemed { .. } => match self.tx_exec.redeem_ticket(ack.clone()).await {
                            Ok(tx_hash) => {
                                if let Err(e) = self.db.write().await.mark_redeemed(&ack).await {
                                    error!("failed to mark {ack} as redeemed: {e}");
                                }
                                tx_result = TransactionResult::RedeemTicket { tx_hash }
                            }
                            Err(e) => error!("failed to redeem {ack}: {e}"),
                        },
                        _ => error!("invalid state of {ack}"),
                    };
                }

                Transaction::OpenChannel(address, stake) => match self.tx_exec.open_channel(address, stake).await {
                    Ok((tx_hash, channel_id)) => {
                        tx_result = TransactionResult::OpenChannel { tx_hash, channel_id };
                    }
                    Err(e) => error!("failed to open channel to {address} with {stake}: {e}"),
                },

                Transaction::FundChannel(channel_id, amount) => {
                    let maybe_channel = self.db.read().await.get_channel(&channel_id).await.ok().flatten();
                    if let Some(channel) = maybe_channel {
                        if channel.status == Open {
                            match self.tx_exec.fund_channel(channel_id, amount).await {
                                Ok(tx_hash) => tx_result = TransactionResult::FundChannel { tx_hash },
                                Err(e) => error!("failed to fund channel {channel_id} with {amount}: {e}"),
                            }
                        } else {
                            error!("cannot fund channel {channel_id} that is not opened");
                        }
                    } else {
                        error!("cannot obtain channel {channel_id} for funding");
                    }
                }

                Transaction::CloseChannel(source, destination) => {
                    let maybe_channel = self
                        .db
                        .read()
                        .await
                        .get_channel_x(&source, &destination)
                        .await
                        .ok()
                        .flatten();

                    if let Some(channel) = maybe_channel {
                        let channel_id = channel.get_id();
                        match channel.status {
                            Open => match self.tx_exec.close_channel_initialize(source, destination).await {
                                Ok(tx_hash) => {
                                    info!("closure of channel {channel_id} initialized: {tx_hash}");
                                    tx_result = TransactionResult::CloseChannel {
                                        tx_hash,
                                        status: channel.status,
                                    }
                                }
                                Err(e) => {
                                    error!("failed to initialize closure of channel {channel_id}: {e}");
                                }
                            },
                            PendingToClose => {
                                if channel.closure_time_passed().unwrap_or(false) {
                                    match self.tx_exec.close_channel_finalize(source, destination).await {
                                        Ok(tx_hash) => {
                                            info!("closure of channel {channel_id} finalized: {tx_hash}");
                                            tx_result = TransactionResult::CloseChannel {
                                                tx_hash,
                                                status: channel.status,
                                            }
                                        }
                                        Err(e) => {
                                            error!("failed to initialize closure of channel {channel_id}: {e}");
                                        }
                                    }
                                } else {
                                    warn!("cannot close channel {channel_id} because closure time has not passed, remaining {} seconds", channel.remaining_closure_time().unwrap_or(u32::MAX as u64));
                                    tx_result = TransactionResult::CloseChannel {
                                        tx_hash: Hash::default(),
                                        status: channel.status,
                                    }
                                }
                            }
                            Closed => {
                                warn!("channel {channel_id} is already closed");
                            }
                        }
                    }
                }

                Transaction::Withdraw(recipient, amount) => match self.tx_exec.withdraw(recipient, amount).await {
                    Ok(tx_hash) => {
                        info!("successfully withdrawn {amount} to {recipient}");
                        tx_result = TransactionResult::Withdraw { tx_hash }
                    }
                    Err(e) => error!("failed to withdraw {amount} to {recipient}: {e}"),
                },
            }
            let _ = req.1.send(tx_result);
        }
        warn!("transaction queue has finished");
    }
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use crate::errors::CoreEthereumError::TransactionSubmissionFailed;
    use crate::transaction_queue::TransactionExecutor;
    use async_trait::async_trait;
    use core_crypto::types::Hash;
    use core_types::acknowledgement::AcknowledgedTicket;
    use js_sys::Promise;
    use serde::{Deserialize, Serialize};
    use utils_misc::utils::wasm::js_value_to_error_msg;
    use utils_types::errors::GeneralError::ParseError;
    use utils_types::primitives::{Address, Balance};
    use utils_types::traits::ToHex;
    use wasm_bindgen::prelude::wasm_bindgen;
    use wasm_bindgen::JsValue;
    use wasm_bindgen_futures::JsFuture;

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
        async fn redeem_ticket(&self, ticket: AcknowledgedTicket) -> crate::errors::Result<Hash> {
            let wasm_ack: core_types::acknowledgement::wasm::AcknowledgedTicket = ticket.into();
            await_js_promise(self.redeem_ticket_fn.call1(&JsValue::null(), &JsValue::from(wasm_ack)))
                .await
                .map(|r| r.as_string().and_then(|s| Hash::from_hex(&s).ok()).unwrap_or_default())
                .map_err(|s| TransactionSubmissionFailed(s))
        }

        async fn open_channel(&self, destination: Address, balance: Balance) -> crate::errors::Result<(Hash, Hash)> {
            await_js_promise(self.open_channel_fn.call2(
                &JsValue::null(),
                &JsValue::from(destination),
                &JsValue::from(balance),
            ))
            .await
            .map_err(|s| TransactionSubmissionFailed(s))
            .and_then(|v| {
                serde_wasm_bindgen::from_value::<OpenChannelResult>(v)
                    .map_err(|_| crate::errors::CoreEthereumError::OtherError(ParseError))
            })
            .map(|v| {
                (
                    Hash::from_hex(&v.receipt).unwrap_or_default(),
                    Hash::from_hex(&v.receipt).unwrap_or_default(),
                )
            })
        }

        async fn fund_channel(&self, channel_id: Hash, amount: Balance) -> crate::errors::Result<Hash> {
            await_js_promise(self.fund_channel_fn.call2(
                &JsValue::null(),
                &JsValue::from(channel_id),
                &JsValue::from(amount),
            ))
            .await
            .map(|r| r.as_string().and_then(|s| Hash::from_hex(&s).ok()).unwrap_or_default())
            .map_err(|s| TransactionSubmissionFailed(s))
        }

        async fn close_channel_initialize(&self, src: Address, dst: Address) -> crate::errors::Result<Hash> {
            await_js_promise(self.close_channel_init_fn.call2(
                &JsValue::null(),
                &JsValue::from(src),
                &JsValue::from(dst),
            ))
            .await
            .map(|r| r.as_string().and_then(|s| Hash::from_hex(&s).ok()).unwrap_or_default())
            .map_err(|s| TransactionSubmissionFailed(s))
        }

        async fn close_channel_finalize(&self, src: Address, dst: Address) -> crate::errors::Result<Hash> {
            await_js_promise(self.close_channel_finish_fn.call2(
                &JsValue::null(),
                &JsValue::from(src),
                &JsValue::from(dst),
            ))
            .await
            .map(|r| r.as_string().and_then(|s| Hash::from_hex(&s).ok()).unwrap_or_default())
            .map_err(|s| TransactionSubmissionFailed(s))
        }

        async fn withdraw(&self, recipient: Address, amount: Balance) -> crate::errors::Result<Hash> {
            await_js_promise(self.withdraw_fn.call2(
                &JsValue::null(),
                &JsValue::from(recipient),
                &JsValue::from(amount),
            ))
            .await
            .map(|r| r.as_string().and_then(|s| Hash::from_hex(&s).ok()).unwrap_or_default())
            .map_err(|s| TransactionSubmissionFailed(s))
        }
    }
}
