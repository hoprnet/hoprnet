use async_lock::RwLock;
use async_std::channel::{bounded, Receiver, Sender};
use async_trait::async_trait;
use core_crypto::types::Hash;
use core_ethereum_db::traits::HoprCoreEthereumDbActions;
use core_types::acknowledgement::AcknowledgedTicketStatus;
use core_types::announcement::AnnouncementData;
use core_types::{
    acknowledgement::{AcknowledgedTicket, AcknowledgedTicketStatus::BeingRedeemed},
    channels::{
        ChannelDirection, ChannelEntry,
        ChannelStatus::{Closed, Open, PendingToClose},
    },
};
use futures::future::Either;
use futures::{pin_mut, FutureExt};
use std::rc::Rc;
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

    /// Announce node on-chain
    Announce(AnnouncementData, bool),

    /// Register safe address with this node
    RegisterSafe(Address),
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
            Transaction::Withdraw(destination, amount) => write!(f, "withdraw tx of {amount} to {destination}"),
            Transaction::Announce(data, safe) => {
                if *safe {
                    write!(f, "announce tx via safe of {}", data.to_multiaddress_str())
                } else {
                    write!(f, "announce tx of {}", data.to_multiaddress_str())
                }
            }
            Transaction::RegisterSafe(safe_address) => write!(f, "register safe tx {safe_address}"),
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
    async fn announce(&self, data: AnnouncementData, use_safe: bool) -> TransactionResult;
    async fn register_safe(&self, safe_address: Address) -> TransactionResult;
}

/// Represents a result of an Ethereum transaction after it has been confirmed.
/// These are counter parts to the `Transaction` type.
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

/// Notifies about completion of a transaction (success or failure).
pub type TransactionCompleted = futures::channel::oneshot::Receiver<TransactionResult>;

/// Future that resolves once the transaction has been confirmed by the Indexer.
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
        tx: Transaction,
    ) -> TransactionResult {
        match tx {
            Transaction::RedeemTicket(mut ack) => match &ack.status {
                BeingRedeemed { .. } => {
                    let res = tx_exec.redeem_ticket(ack.clone()).await;
                    match &res {
                        TicketRedeemed { .. } => {
                            if let Err(e) = db.write().await.mark_redeemed(&ack).await {
                                // Still declare the TX a success
                                error!("failed to mark {ack} as redeemed: {e}");
                            }
                        }
                        Failure(e) => {
                            // TODO: once we can distinguish EVM execution failure from `e`, we can mark ticket as losing instead

                            error!("marking the acknowledged ticket as untouched - edeem tx failed: {e}");
                            ack.status = AcknowledgedTicketStatus::Untouched;
                            if let Err(e) = db.write().await.update_acknowledged_ticket(&ack).await {
                                error!("cannot mark {ack} as untouched: {e}");
                            }
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
                        let res = tx_exec.initiate_outgoing_channel_closure(channel.destination).await;
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
            Transaction::Announce(data, use_safe) => tx_exec.announce(data, use_safe).await,
            Transaction::RegisterSafe(safe_address) => tx_exec.register_safe(safe_address).await,
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

#[cfg(feature = "wasm")]
pub mod wasm {
    use crate::payload::{BasicPayloadGenerator, PayloadGenerator};
    use crate::{
        payload::SafePayloadGenerator,
        transaction_queue::{TransactionExecutor, TransactionResult, TransactionSender},
    };
    use async_trait::async_trait;
    use core_crypto::{keypairs::ChainKeypair, keypairs::Keypair, types::Hash};
    use core_types::acknowledgement::AcknowledgedTicket;
    use core_types::announcement::AnnouncementData;
    use ethers::types::{transaction::eip2718::TypedTransaction, Eip1559TransactionRequest, NameOrAddress, H160, U256};
    use hex;
    use js_sys::{JsString, Promise};
    use serde::{Deserialize, Serialize};
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
    pub struct WasmTransactionPayload {
        pub data: String,
        pub to: String,
        pub value: String,
    }

    #[wasm_bindgen(getter_with_clone)]
    #[derive(Serialize, Deserialize, Debug)]
    pub struct WasmSendTransactionResult {
        pub code: String,
        pub tx: Option<String>,
    }

    enum SendTransactionResult {
        Success(Hash),
        Duplicate,
        Failure(String),
    }

    #[wasm_bindgen]
    pub struct WasmTxExecutor {
        send_transaction: js_sys::Function,
        safe_generator: SafePayloadGenerator,
        basic_generator: BasicPayloadGenerator,
        hopr_channels: Address,
        hopr_token: Address,
        hopr_announcements: Address,
        module_address: Address,
        node_safe_registry: Address,
    }

    #[wasm_bindgen]
    impl WasmTxExecutor {
        #[wasm_bindgen(constructor)]
        pub fn new(
            send_transaction: js_sys::Function,
            chain_keypair: &ChainKeypair,
            hopr_channels: Address,
            hopr_announcements: Address,
            module_address: Address,
            node_safe_registry: Address,
            hopr_token: Address,
        ) -> Self {
            Self {
                hopr_announcements,
                module_address,
                node_safe_registry,
                hopr_channels,
                hopr_token,
                send_transaction,
                basic_generator: BasicPayloadGenerator::new(chain_keypair.public().to_address()),
                safe_generator: SafePayloadGenerator::new(chain_keypair, hopr_channels, hopr_announcements),
            }
        }
    }

    impl From<WasmSendTransactionResult> for SendTransactionResult {
        fn from(value: WasmSendTransactionResult) -> Self {
            let val = value.code.to_uppercase();
            match val.as_str() {
                "SUCCESS" => SendTransactionResult::Success(
                    value
                        .tx
                        .and_then(|tx| Hash::from_hex(&tx).ok())
                        .expect("invalid tx hash returned"),
                ),
                "DUPLICATE" => SendTransactionResult::Duplicate,
                _ => SendTransactionResult::Failure(format!("tx sender error: {value:?}")),
            }
        }
    }

    impl WasmTxExecutor {
        async fn send_transaction(&self, tx: TypedTransaction, confirmation_prefix: &str) -> SendTransactionResult {
            let payload = WasmTransactionPayload {
                data: match tx.data() {
                    Some(data) => format!("0x{}", hex::encode(data)),
                    None => "0x".into(),
                },
                to: match tx.to() {
                    Some(NameOrAddress::Address(addr)) => format!("0x{}", hex::encode(addr)),
                    Some(NameOrAddress::Name(_)) => todo!("ens names are not yet supported"),
                    None => return SendTransactionResult::Failure("cannot set transaction target".into()),
                },
                value: match tx.value() {
                    Some(x) => x.to_string(),
                    None => "".into(),
                },
            };

            match await_js_promise(self.send_transaction.call2(
                &JsValue::undefined(),
                &JsValue::from(payload),
                &JsString::from(confirmation_prefix).into(),
            ))
            .await
            {
                Ok(v) => {
                    if let Ok(result) = serde_wasm_bindgen::from_value::<WasmSendTransactionResult>(v) {
                        result.into()
                    } else {
                        SendTransactionResult::Failure("serde deserialization error".into())
                    }
                }
                Err(e) => SendTransactionResult::Failure(e),
            }
        }
    }

    #[async_trait(? Send)]
    impl TransactionExecutor for WasmTxExecutor {
        async fn redeem_ticket(&self, acked_ticket: AcknowledgedTicket) -> TransactionResult {
            let mut tx = TypedTransaction::Eip1559(Eip1559TransactionRequest::new());

            tx.set_data(match self.safe_generator.redeem_ticket(&acked_ticket) {
                Ok(payload) => payload.into(),
                Err(e) => return TransactionResult::Failure(e.to_string()),
            });
            tx.set_to(H160::from(self.hopr_channels));

            match self.send_transaction(tx, "channel-updated-").await {
                SendTransactionResult::Success(tx_hash) => TransactionResult::TicketRedeemed { tx_hash },
                SendTransactionResult::Duplicate => {
                    TransactionResult::Failure("ticket redeem transaction is a duplicate".into())
                }
                SendTransactionResult::Failure(e) => {
                    TransactionResult::Failure(format!("ticket redeem send transaction failed: {e}"))
                }
            }
        }

        async fn fund_channel(&self, destination: Address, balance: Balance) -> TransactionResult {
            let mut tx = TypedTransaction::Eip1559(Eip1559TransactionRequest::new());

            tx.set_data(match self.safe_generator.fund_channel(&destination, &balance) {
                Ok(payload) => payload.into(),
                Err(e) => return TransactionResult::Failure(e.to_string()),
            });
            tx.set_to(H160::from(self.hopr_channels));

            match self.send_transaction(tx, "channel-updated-").await {
                SendTransactionResult::Success(tx_hash) => TransactionResult::ChannelFunded { tx_hash },
                SendTransactionResult::Duplicate => {
                    TransactionResult::Failure("fund channel transaction is a duplicate".into())
                }
                SendTransactionResult::Failure(e) => {
                    TransactionResult::Failure(format!("fund channel send transaction failed: {e}"))
                }
            }
        }

        async fn initiate_outgoing_channel_closure(&self, dst: Address) -> TransactionResult {
            let mut tx = TypedTransaction::Eip1559(Eip1559TransactionRequest::new());

            tx.set_data(match self.safe_generator.initiate_outgoing_channel_closure(&dst) {
                Ok(payload) => payload.into(),
                Err(e) => return TransactionResult::Failure(e.to_string()),
            });
            tx.set_to(H160::from(self.hopr_channels));

            match self.send_transaction(tx, "channel-updated-").await {
                SendTransactionResult::Success(tx_hash) => TransactionResult::ChannelClosureInitiated { tx_hash },
                SendTransactionResult::Duplicate => {
                    TransactionResult::Failure("init close outgoing channel transaction is a duplicate".into())
                }
                SendTransactionResult::Failure(e) => {
                    TransactionResult::Failure(format!("init close outgoing channel send transaction failed: {e}"))
                }
            }
        }

        async fn finalize_outgoing_channel_closure(&self, dst: Address) -> TransactionResult {
            let mut tx = TypedTransaction::Eip1559(Eip1559TransactionRequest::new());

            tx.set_data(match self.safe_generator.finalize_outgoing_channel_closure(&dst) {
                Ok(payload) => payload.into(),
                Err(e) => return TransactionResult::Failure(e.to_string()),
            });
            tx.set_to(H160::from(self.hopr_channels));

            match self.send_transaction(tx, "channel-updated-").await {
                SendTransactionResult::Success(tx_hash) => TransactionResult::ChannelClosed { tx_hash },
                SendTransactionResult::Duplicate => {
                    TransactionResult::Failure("finalize close outgoing channel transaction is a duplicate".into())
                }
                SendTransactionResult::Failure(e) => {
                    TransactionResult::Failure(format!("finalize close outgoing channel send transaction failed: {e}"))
                }
            }
        }

        async fn close_incoming_channel(&self, src: Address) -> TransactionResult {
            let mut tx = TypedTransaction::Eip1559(Eip1559TransactionRequest::new());

            tx.set_data(match self.safe_generator.close_incoming_channel(&src) {
                Ok(payload) => payload.into(),
                Err(e) => return TransactionResult::Failure(e.to_string()),
            });
            tx.set_to(H160::from(self.hopr_channels));

            match self.send_transaction(tx, "channel-updated-").await {
                SendTransactionResult::Success(tx_hash) => TransactionResult::ChannelClosed { tx_hash },
                SendTransactionResult::Duplicate => {
                    TransactionResult::Failure("close incoming channel transaction is a duplicate".into())
                }
                SendTransactionResult::Failure(e) => {
                    TransactionResult::Failure(format!("close incoming channel send transaction failed: {e}"))
                }
            }
        }

        async fn withdraw(&self, recipient: Address, amount: Balance) -> TransactionResult {
            let mut tx = TypedTransaction::Eip1559(Eip1559TransactionRequest::new());

            let event_string = match amount.balance_type() {
                BalanceType::HOPR => {
                    tx.set_data(match self.safe_generator.transfer(&recipient, &amount) {
                        Ok(payload) => payload.into(),
                        Err(e) => return TransactionResult::Failure(e.to_string()),
                    });
                    tx.set_to(H160::from(self.hopr_token));

                    "withdraw-hopr-"
                }
                BalanceType::Native => {
                    tx.set_to(H160::from(recipient));
                    tx.set_value(U256(primitive_types::U256::from(amount.value()).0));

                    "withdraw-native-"
                }
            };

            match self.send_transaction(tx, event_string).await {
                SendTransactionResult::Success(tx_hash) => TransactionResult::Withdrawn { tx_hash },
                SendTransactionResult::Duplicate => {
                    TransactionResult::Failure("withdraw transaction is a duplicate".into())
                }
                SendTransactionResult::Failure(e) => {
                    TransactionResult::Failure(format!("withdraw send transaction failed: {e}"))
                }
            }
        }

        async fn announce(&self, data: AnnouncementData, use_safe: bool) -> TransactionResult {
            let mut tx = TypedTransaction::Eip1559(Eip1559TransactionRequest::new());

            if !use_safe {
                tx.set_data(match self.basic_generator.announce(&data) {
                    Ok(payload) => payload.into(),
                    Err(e) => return TransactionResult::Failure(e.to_string()),
                });

                tx.set_to(H160::from(self.hopr_announcements));
            } else {
                tx.set_data(match self.safe_generator.announce(&data) {
                    Ok(payload) => payload.into(),
                    Err(e) => return TransactionResult::Failure(e.to_string()),
                });

                tx.set_to(H160::from(self.module_address));
            }

            match self.send_transaction(tx, "announce-").await {
                SendTransactionResult::Success(tx_hash) => TransactionResult::Announced { tx_hash },
                SendTransactionResult::Duplicate => {
                    TransactionResult::Failure("announce transaction is a duplicate".into())
                }
                SendTransactionResult::Failure(e) => {
                    TransactionResult::Failure(format!("announce send transaction failed: {e}"))
                }
            }
        }

        async fn register_safe(&self, safe_address: Address) -> TransactionResult {
            let mut tx = TypedTransaction::Eip1559(Eip1559TransactionRequest::new());

            tx.set_data(match self.basic_generator.register_safe_by_node(&safe_address) {
                Ok(payload) => payload.into(),
                Err(e) => return TransactionResult::Failure(e.to_string()),
            });
            tx.set_to(H160::from(self.node_safe_registry));

            match self.send_transaction(tx, "node-safe-registered-").await {
                SendTransactionResult::Success(tx_hash) => TransactionResult::SafeRegistered { tx_hash },
                SendTransactionResult::Duplicate => TransactionResult::Failure("safe register is a duplicate".into()),
                SendTransactionResult::Failure(e) => {
                    TransactionResult::Failure(format!("safe register send transaction failed: {e}"))
                }
            }
        }
    }
}
