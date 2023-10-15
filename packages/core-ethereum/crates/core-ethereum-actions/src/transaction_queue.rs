use async_lock::RwLock;
use async_std::channel::{bounded, Receiver, Sender};
use async_trait::async_trait;
use core_crypto::types::Hash;
use core_ethereum_db::traits::HoprCoreEthereumDbActions;
use core_types::{
    acknowledgement::{AcknowledgedTicket, AcknowledgedTicketStatus::BeingRedeemed},
    channels::{
        ChannelDirection, ChannelEntry,
        ChannelStatus::{Closed, Open, PendingToClose},
    },
};
use futures::future::Either;
use futures::{pin_mut, FutureExt};
use std::{
    fmt::{Display, Formatter},
    sync::Arc,
};
use utils_log::{debug, error, info, warn};
use utils_types::primitives::{Address, Balance};

use crate::errors::CoreEthereumActionsError::TransactionSubmissionFailed;
use crate::errors::Result;
use crate::transaction_queue::TransactionResult::{Failure, TicketRedeemed};

#[cfg(any(not(feature = "wasm"), test))]
use async_std::task::{sleep, spawn_local};

#[cfg(all(feature = "wasm", not(test)))]
use wasm_bindgen_futures::spawn_local;

#[cfg(all(feature = "wasm", not(test)))]
use gloo_timers::future::sleep;

/// Enumerates all possible on-chain state change requests
#[derive(Clone, PartialEq, Debug)]
pub enum Transaction {
    /// Redeem the given acknowledged ticket
    RedeemTicket(AcknowledgedTicket),

    /// Open channel to the given destination with the given stake
    OpenChannel(Address, Balance),

    /// Fund channel with the given ID and amount
    FundChannel(ChannelEntry, Balance),

    /// Close channel with the given source and destination
    CloseChannel(ChannelEntry, ChannelDirection),

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
            Transaction::CloseChannel(channel, direction) => write!(
                f,
                "closure tx of {} channel from {} to {}",
                direction, channel.source, channel.destination
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
    async fn fund_channel(&self, destination: Address, balance: Balance) -> TransactionResult;
    async fn initiate_outgoing_channel_closure(&self, dst: Address) -> TransactionResult;
    async fn finalize_outgoing_channel_closure(&self, dst: Address) -> TransactionResult;
    async fn close_incoming_channel(&self, src: Address) -> TransactionResult;
    async fn withdraw(&self, recipient: Address, amount: Balance) -> TransactionResult;
}

/// Represents a result of an Ethereum transaction after it has been confirmed.
#[derive(Clone, Debug)]
pub enum TransactionResult {
    TicketRedeemed { tx_hash: Hash },
    ChannelFunded { tx_hash: Hash },
    ChannelClosureInitiated { tx_hash: Hash },
    ChannelClosed { tx_hash: Hash },
    Withdrawn { tx_hash: Hash },
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

    /// Maximum time (in seconds) to wait for the transaction to be confirmed on-chain and indexed
    pub const MAX_TX_CONFIRMATION_WAIT_SECS: usize = 180;

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
    ) -> TransactionResult {
        match tx {
            Transaction::RedeemTicket(ack) => match ack.status {
                BeingRedeemed { .. } => {
                    let res = tx_exec.redeem_ticket(ack.clone()).await;
                    match res {
                        TicketRedeemed { .. } => {
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

            Transaction::OpenChannel(address, stake) => tx_exec.fund_channel(address, stake).await,

            Transaction::FundChannel(channel, amount) => {
                if channel.status == Open {
                    tx_exec.fund_channel(channel.destination, amount).await
                } else {
                    Failure(format!("cannot fund {channel} because it is not opened"))
                }
            }

            Transaction::CloseChannel(channel, direction) => match direction {
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
                        let res = tx_exec.initiate_outgoing_channel_closure(channel.source).await;
                        if let TransactionResult::ChannelClosureInitiated { .. } = res {
                            debug!("deleting pending balance of {channel} after initiating closure");
                            if let Err(e) = db.write().await.reset_pending_balance_to(&channel.destination).await {
                                error!(
                                    "failed to reset pending balance to {} in channel {channel}: {e}",
                                    channel.destination
                                );
                            }
                        }
                        res
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

            Transaction::Withdraw(recipient, amount) => tx_exec.withdraw(recipient, amount).await,
        }
    }

    /// Consumes self and runs the main queue processing loop until the queue is closed.
    pub async fn transaction_loop(self) {
        while let Ok((tx, tx_finisher)) = self.queue_recv.recv().await {
            let db_clone = self.db.clone();
            let tx_exec_clone = self.tx_exec.clone();
            let tx_id = tx.to_string();

            spawn_local(async move {
                let tx_fut = Self::execute_transaction(db_clone, tx_exec_clone, tx);

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
                        } else {
                            info!("transaction {tx_id} succeeded");
                        }
                        result
                    }
                    Either::Right((_, _)) => Failure(format!(
                        "awaiting for confirmation of {tx_id} timed out after {} seconds",
                        Self::MAX_TX_CONFIRMATION_WAIT_SECS
                    )),
                };

                let _ = tx_finisher.send(tx_result);
            });
        }
        warn!("transaction queue has finished");
    }
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use crate::{
        errors::CoreEthereumActionsError,
        payload::PayloadGenerator,
        transaction_queue::{TransactionExecutor, TransactionResult, TransactionSender},
    };
    use async_trait::async_trait;
    use core_crypto::{keypairs::ChainKeypair, types::Hash};
    use core_types::acknowledgement::AcknowledgedTicket;
    use ethers::types::{transaction::eip2718::TypedTransaction, Eip1559TransactionRequest, NameOrAddress, H160, U256};
    use hex;
    use js_sys::{JsString, Promise};
    use utils_misc::utils::wasm::js_value_to_error_msg;
    use utils_types::{
        primitives::{Address, Balance, BalanceType},
        traits::ToHex,
    };
    use wasm_bindgen::{prelude::wasm_bindgen, JsValue};
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
        send_transaction: js_sys::Function,
        generator: PayloadGenerator,
        hopr_channels: Address,
        hopr_token: Address,
        // TODO: migrate announce function
        _hopr_announcements: Address,
    }

    #[wasm_bindgen]
    impl WasmTxExecutor {
        #[wasm_bindgen(constructor)]
        pub fn new(
            send_transaction: js_sys::Function,
            chain_keypair: &ChainKeypair,
            hopr_channels: Address,
            hopr_announcements: Address,
            hopr_token: Address,
        ) -> Self {
            Self {
                // TODO: migrate announce function
                _hopr_announcements: hopr_announcements,
                hopr_channels,
                hopr_token,
                send_transaction,
                generator: PayloadGenerator::new(chain_keypair, hopr_channels, hopr_announcements),
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

    #[wasm_bindgen(getter_with_clone)]
    struct TransactionPayload {
        #[allow(unused)]
        pub data: String,
        #[allow(unused)]
        pub to: String,
        #[allow(unused)]
        pub value: String,
    }

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen]
        pub type SendTransactionReturnTx;

        #[wasm_bindgen(method, getter)]
        pub fn hash(this: &SendTransactionReturnTx) -> String;
    }

    #[wasm_bindgen]
    extern "C" {
        // copied from JS, to be removed soon
        #[wasm_bindgen]
        pub type SendTransactionReturn;

        #[wasm_bindgen(method, getter)]
        pub fn code(this: &SendTransactionReturn) -> String;

        #[wasm_bindgen(method, getter)]
        pub fn tx(this: &SendTransactionReturn) -> Option<SendTransactionReturnTx>;
    }

    impl TryFrom<TypedTransaction> for TransactionPayload {
        type Error = CoreEthereumActionsError;

        fn try_from(value: TypedTransaction) -> Result<Self, Self::Error> {
            Ok(Self {
                data: match value.data() {
                    Some(data) => format!("0x{}", hex::encode(data)),
                    None => "0x".into(),
                },
                to: match value.to() {
                    Some(NameOrAddress::Address(addr)) => format!("0x{}", hex::encode(addr)),
                    Some(NameOrAddress::Name(_)) => todo!("ens names are not yet supported"),
                    None => {
                        return Err(CoreEthereumActionsError::InvalidArguments(
                            "cannot convert transaction destination (\"to\") to hex".into(),
                        ))
                    }
                },
                value: match value.value() {
                    Some(x) => x.to_string(),
                    None => "".into(),
                },
            })
        }
    }

    #[async_trait(? Send)]
    impl TransactionExecutor for WasmTxExecutor {
        async fn redeem_ticket(&self, acked_ticket: AcknowledgedTicket) -> TransactionResult {
            let mut tx = TypedTransaction::Eip1559(Eip1559TransactionRequest::new());

            tx.set_data(match self.generator.redeem_ticket(&acked_ticket) {
                Ok(payload) => payload.into(),
                Err(e) => return TransactionResult::Failure(e.to_string()),
            });
            tx.set_to(H160::from(self.hopr_channels));

            match await_js_promise(self.send_transaction.call2(
                &JsValue::undefined(),
                &JsValue::from(match TransactionPayload::try_from(tx) {
                    Ok(tx) => tx,
                    Err(e) => return TransactionResult::Failure(e.to_string()),
                }),
                &JsString::from("channel-updated-").into(),
            ))
            .await
            {
                Ok(v) => {
                    let result = SendTransactionReturn::from(v.clone());

                    if result.code().eq("SUCCESS") {
                        return match Hash::from_hex(result.tx().unwrap().hash().as_str()) {
                            Ok(tx_hash) => TransactionResult::TicketRedeemed { tx_hash },
                            Err(_) => TransactionResult::Failure(format!("Could not convert js object. {:?}", v)),
                        };
                    } else if result.code().eq("DUPLICATE") {
                        return TransactionResult::Failure(format!("ticket redeem transaction is a duplicate."));
                    } else {
                        return TransactionResult::Failure(format!(
                            "ticket redeem transaction failed with unknown error {:?}",
                            v
                        ));
                    }
                }
                Err(e) => TransactionResult::Failure(e),
            }
        }

        async fn fund_channel(&self, destination: Address, balance: Balance) -> TransactionResult {
            let mut tx = TypedTransaction::Eip1559(Eip1559TransactionRequest::new());

            tx.set_data(match self.generator.fund_channel(&destination, &balance) {
                Ok(payload) => payload.into(),
                Err(e) => return TransactionResult::Failure(e.to_string()),
            });
            tx.set_to(H160::from(self.hopr_channels));

            match await_js_promise(self.send_transaction.call2(
                &JsValue::undefined(),
                &JsValue::from(match TransactionPayload::try_from(tx) {
                    Ok(tx) => tx,
                    Err(e) => return TransactionResult::Failure(e.to_string()),
                }),
                &JsString::from("channel-updated-").into(),
            ))
            .await
            {
                Ok(v) => {
                    let result = SendTransactionReturn::from(v.clone());

                    if result.code().eq("SUCCESS") {
                        return match Hash::from_hex(result.tx().unwrap().hash().as_str()) {
                            Ok(tx_hash) => TransactionResult::ChannelFunded { tx_hash },
                            Err(_) => TransactionResult::Failure(format!("Could not convert js object. {:?}", v)),
                        };
                    } else if result.code().eq("DUPLICATE") {
                        return TransactionResult::Failure(format!("fund channel transaction is a duplicate."));
                    } else {
                        return TransactionResult::Failure(format!(
                            "fund channel transaction failed with unknown error {:?}",
                            v
                        ));
                    }
                }
                Err(e) => TransactionResult::Failure(e),
            }
        }

        async fn initiate_outgoing_channel_closure(&self, dst: Address) -> TransactionResult {
            let mut tx = TypedTransaction::Eip1559(Eip1559TransactionRequest::new());

            tx.set_data(match self.generator.initiate_outgoing_channel_closure(&dst) {
                Ok(payload) => payload.into(),
                Err(e) => return TransactionResult::Failure(e.to_string()),
            });
            tx.set_to(H160::from(self.hopr_channels));

            match await_js_promise(self.send_transaction.call2(
                &JsValue::undefined(),
                &JsValue::from(match TransactionPayload::try_from(tx) {
                    Ok(tx) => tx,
                    Err(e) => return TransactionResult::Failure(e.to_string()),
                }),
                &JsString::from("channel-updated-").into(),
            ))
            .await
            {
                Ok(v) => {
                    let result = SendTransactionReturn::from(v.clone());

                    if result.code().eq("SUCCESS") {
                        return match Hash::from_hex(result.tx().unwrap().hash().as_str()) {
                            Ok(tx_hash) => TransactionResult::ChannelClosureInitiated { tx_hash },
                            Err(_) => TransactionResult::Failure(format!("Could not convert js object. {:?}", v)),
                        };
                    } else if result.code().eq("DUPLICATE") {
                        return TransactionResult::Failure(format!(
                            "initiate channel closure transaction is a duplicate."
                        ));
                    } else {
                        return TransactionResult::Failure(format!(
                            "initiate channel closure transaction failed with unknown error {:?}",
                            v
                        ));
                    }
                }
                Err(e) => TransactionResult::Failure(e),
            }
        }

        async fn finalize_outgoing_channel_closure(&self, dst: Address) -> TransactionResult {
            let mut tx = TypedTransaction::Eip1559(Eip1559TransactionRequest::new());

            tx.set_data(match self.generator.finalize_outgoing_channel_closure(&dst) {
                Ok(payload) => payload.into(),
                Err(e) => return TransactionResult::Failure(e.to_string()),
            });
            tx.set_to(H160::from(self.hopr_channels));

            match await_js_promise(self.send_transaction.call2(
                &JsValue::undefined(),
                &JsValue::from(match TransactionPayload::try_from(tx) {
                    Ok(tx) => tx,
                    Err(e) => return TransactionResult::Failure(e.to_string()),
                }),
                &JsString::from("channel-updated-").into(),
            ))
            .await
            {
                Ok(v) => {
                    let result = SendTransactionReturn::from(v.clone());

                    if result.code().eq("SUCCESS") {
                        return match Hash::from_hex(result.tx().unwrap().hash().as_str()) {
                            Ok(tx_hash) => TransactionResult::ChannelClosed { tx_hash },
                            Err(_) => TransactionResult::Failure(format!("Could not convert js object. {:?}", v)),
                        };
                    } else if result.code().eq("DUPLICATE") {
                        return TransactionResult::Failure(format!(
                            "finalize channel close transaction is a duplicate."
                        ));
                    } else {
                        return TransactionResult::Failure(format!(
                            "finalize channel close transaction failed with unknown error {:?}",
                            v
                        ));
                    }
                }
                Err(e) => TransactionResult::Failure(e),
            }
        }

        async fn close_incoming_channel(&self, src: Address) -> TransactionResult {
            let mut tx = TypedTransaction::Eip1559(Eip1559TransactionRequest::new());

            tx.set_data(match self.generator.close_incoming_channel(&src) {
                Ok(payload) => payload.into(),
                Err(e) => return TransactionResult::Failure(e.to_string()),
            });
            tx.set_to(H160::from(self.hopr_channels));

            match await_js_promise(self.send_transaction.call2(
                &JsValue::undefined(),
                &JsValue::from(match TransactionPayload::try_from(tx) {
                    Ok(tx) => tx,
                    Err(e) => return TransactionResult::Failure(e.to_string()),
                }),
                &JsString::from("channel-updated-").into(),
            ))
            .await
            {
                Ok(v) => {
                    let result = SendTransactionReturn::from(v.clone());

                    if result.code().eq("SUCCESS") {
                        return match Hash::from_hex(result.tx().unwrap().hash().as_str()) {
                            Ok(tx_hash) => TransactionResult::ChannelClosed { tx_hash },
                            Err(_) => TransactionResult::Failure(format!("Could not convert js object. {:?}", v)),
                        };
                    } else if result.code().eq("DUPLICATE") {
                        return TransactionResult::Failure(format!(
                            "close incoming channel transaction is a duplicate."
                        ));
                    } else {
                        return TransactionResult::Failure(format!(
                            "close incoming channel transaction failed with unknown error {:?}",
                            v
                        ));
                    }
                }
                Err(e) => TransactionResult::Failure(e),
            }
        }

        async fn withdraw(&self, recipient: Address, amount: Balance) -> TransactionResult {
            let mut tx = TypedTransaction::Eip1559(Eip1559TransactionRequest::new());

            let event_string: String;
            match amount.balance_type() {
                BalanceType::HOPR => {
                    tx.set_data(match self.generator.transfer(&recipient, &amount) {
                        Ok(payload) => payload.into(),
                        Err(e) => return TransactionResult::Failure(e.to_string()),
                    });
                    tx.set_to(H160::from(self.hopr_token));

                    event_string = "withdraw-hopr-".into();
                }
                BalanceType::Native => {
                    tx.set_to(H160::from(recipient));
                    tx.set_value(U256(primitive_types::U256::from(amount.value()).0));

                    event_string = "withdraw-native-".into();
                }
            }

            match await_js_promise(self.send_transaction.call2(
                &JsValue::undefined(),
                &JsValue::from(match TransactionPayload::try_from(tx) {
                    Ok(tx) => tx,
                    Err(e) => return TransactionResult::Failure(e.to_string()),
                }),
                &JsString::from(event_string),
            ))
            .await
            {
                Ok(v) => {
                    let result = SendTransactionReturn::from(v.clone());

                    if result.code().eq("SUCCESS") {
                        return match Hash::from_hex(result.tx().unwrap().hash().as_str()) {
                            Ok(tx_hash) => TransactionResult::Withdrawn { tx_hash },
                            Err(_) => TransactionResult::Failure(format!("Could not convert js object. {:?}", v)),
                        };
                    } else if result.code().eq("DUPLICATE") {
                        return TransactionResult::Failure(format!("withdraw transaction is a duplicate."));
                    } else {
                        return TransactionResult::Failure(format!(
                            "withdraw transaction failed with unknown error {:?}",
                            v
                        ));
                    }
                }
                Err(e) => TransactionResult::Failure(e),
            }
        }
    }
}
