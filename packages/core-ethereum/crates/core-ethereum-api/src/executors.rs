use async_lock::Mutex;
use async_trait::async_trait;
use core_crypto::types::Hash;
use core_ethereum_actions::payload::PayloadGenerator;
use core_ethereum_actions::transaction_queue::{TransactionExecutor, TransactionResult};
use core_ethereum_rpc::{HoprRpcOperations, TypedTransaction};
use core_types::acknowledgement::AcknowledgedTicket;
use core_types::announcement::AnnouncementData;
use std::marker::PhantomData;
use std::sync::Arc;
use utils_types::primitives::{Address, Balance};

use crate::errors::Result;

/// Represents an abstract client that is capable of submitting
/// an Ethereum transaction-like object to the blockchain.
#[async_trait(? Send)]
pub trait EthereumClient<T: Into<TypedTransaction>> {
    /// Sends transaction to the blockchain and returns its hash.
    /// Does not poll for transaction completion.
    async fn post_transaction(&self, tx: T) -> Result<Hash>;
}

/// Instantiation of `EthereumClient` using `HoprRpcOperations`.
pub struct RpcEthereumClient<Rpc: HoprRpcOperations> {
    rpc: Arc<Mutex<Rpc>>,
}

impl<Rpc: HoprRpcOperations> RpcEthereumClient<Rpc> {
    pub fn new(rpc: Arc<Mutex<Rpc>>) -> Self {
        Self { rpc }
    }
}

#[async_trait(? Send)]
impl<Rpc: HoprRpcOperations> EthereumClient<TypedTransaction> for RpcEthereumClient<Rpc> {
    async fn post_transaction(&self, tx: TypedTransaction) -> Result<Hash> {
        let res = self.rpc.lock().await.send_transaction(tx.into()).await?;
        Ok(res)
    }
}

/// Implementation of `TransactionExecutor` using the given `EthereumClient` and corresponding
/// `PayloadGenerator`.
#[derive(Clone, Debug)]
pub struct EthereumTransactionExecutor<T, C, PGen>
where
    T: Into<TypedTransaction>,
    C: EthereumClient<T> + Clone,
    PGen: PayloadGenerator<T> + Clone,
{
    client: C,
    payload_generator: PGen,
    _data: PhantomData<T>,
}

impl<T, C, PGen> EthereumTransactionExecutor<T, C, PGen>
where
    T: Into<TypedTransaction>,
    C: EthereumClient<T> + Clone,
    PGen: PayloadGenerator<T> + Clone,
{
    pub fn new(client: C, payload_generator: PGen) -> Self {
        Self {
            client,
            payload_generator,
            _data: PhantomData,
        }
    }
}

#[async_trait(? Send)]
impl<T, C, PGen> TransactionExecutor for EthereumTransactionExecutor<T, C, PGen>
where
    T: Into<TypedTransaction>,
    C: EthereumClient<T> + Clone,
    PGen: PayloadGenerator<T> + Clone,
{
    async fn redeem_ticket(&self, acked_ticket: AcknowledgedTicket) -> TransactionResult {
        match self.payload_generator.redeem_ticket(acked_ticket) {
            Ok(tx) => match self.client.post_transaction(tx).await {
                Ok(tx_hash) => TransactionResult::TicketRedeemed { tx_hash },
                Err(e) => TransactionResult::Failure(e.to_string()),
            },
            Err(e) => e.into(),
        }
    }

    async fn fund_channel(&self, destination: Address, balance: Balance) -> TransactionResult {
        match self.payload_generator.fund_channel(destination, balance) {
            Ok(tx) => match self.client.post_transaction(tx).await {
                Ok(tx_hash) => TransactionResult::ChannelFunded { tx_hash },
                Err(e) => TransactionResult::Failure(e.to_string()),
            },
            Err(e) => e.into(),
        }
    }

    async fn initiate_outgoing_channel_closure(&self, dst: Address) -> TransactionResult {
        match self.payload_generator.initiate_outgoing_channel_closure(dst) {
            Ok(tx) => match self.client.post_transaction(tx).await {
                Ok(tx_hash) => TransactionResult::ChannelClosureInitiated { tx_hash },
                Err(e) => TransactionResult::Failure(e.to_string()),
            },
            Err(e) => e.into(),
        }
    }

    async fn finalize_outgoing_channel_closure(&self, dst: Address) -> TransactionResult {
        match self.payload_generator.finalize_outgoing_channel_closure(dst) {
            Ok(tx) => match self.client.post_transaction(tx).await {
                Ok(tx_hash) => TransactionResult::ChannelClosed { tx_hash },
                Err(e) => TransactionResult::Failure(e.to_string()),
            },
            Err(e) => e.into(),
        }
    }

    async fn close_incoming_channel(&self, src: Address) -> TransactionResult {
        match self.payload_generator.close_incoming_channel(src) {
            Ok(tx) => match self.client.post_transaction(tx).await {
                Ok(tx_hash) => TransactionResult::ChannelClosed { tx_hash },
                Err(e) => TransactionResult::Failure(e.to_string()),
            },
            Err(e) => e.into(),
        }
    }

    async fn withdraw(&self, recipient: Address, amount: Balance) -> TransactionResult {
        match self.payload_generator.transfer(recipient, amount) {
            Ok(tx) => match self.client.post_transaction(tx).await {
                Ok(tx_hash) => TransactionResult::Withdrawn { tx_hash },
                Err(e) => TransactionResult::Failure(e.to_string()),
            },
            Err(e) => e.into(),
        }
    }

    async fn announce(&self, data: AnnouncementData) -> TransactionResult {
        match self.payload_generator.announce(data) {
            Ok(tx) => match self.client.post_transaction(tx).await {
                Ok(tx_hash) => TransactionResult::Announced { tx_hash },
                Err(e) => TransactionResult::Failure(e.to_string()),
            },
            Err(e) => e.into(),
        }
    }

    async fn register_safe(&self, safe_address: Address) -> TransactionResult {
        match self.payload_generator.register_safe_by_node(safe_address) {
            Ok(tx) => match self.client.post_transaction(tx).await {
                Ok(tx_hash) => TransactionResult::SafeRegistered { tx_hash },
                Err(e) => TransactionResult::Failure(e.to_string()),
            },
            Err(e) => e.into(),
        }
    }
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use crate::errors::HoprChainError;
    use crate::executors::{EthereumClient, EthereumTransactionExecutor};
    use async_trait::async_trait;
    use core_crypto::types::Hash;
    use core_ethereum_actions::payload::PayloadGenerator;
    use core_ethereum_rpc::TypedTransaction;
    use core_types::acknowledgement::AcknowledgedTicket;
    use core_types::announcement::AnnouncementData;
    use js_sys::{JsString, Promise};
    use serde::{Deserialize, Serialize};
    use utils_misc::utils::wasm::js_value_to_error_msg;
    use utils_types::primitives::{Address, Balance, BalanceType};
    use utils_types::traits::ToHex;
    use wasm_bindgen::prelude::wasm_bindgen;
    use wasm_bindgen::JsValue;
    use wasm_bindgen_futures::JsFuture;

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

    pub struct TypedTransactionWithTag(TypedTransaction, String);

    impl From<TypedTransactionWithTag> for TypedTransaction {
        fn from(value: TypedTransactionWithTag) -> Self {
            value.0
        }
    }

    #[wasm_bindgen]
    #[derive(Clone)]
    pub struct WasmEthereumClient {
        send_transaction: js_sys::Function,
    }

    #[wasm_bindgen]
    impl WasmEthereumClient {
        #[wasm_bindgen(constructor)]
        pub fn new(send_transaction: js_sys::Function) -> Self {
            Self { send_transaction }
        }
    }

    #[async_trait(? Send)]
    impl EthereumClient<TypedTransactionWithTag> for WasmEthereumClient {
        async fn post_transaction(&self, tx: TypedTransactionWithTag) -> crate::errors::Result<Hash> {
            let payload = WasmTransactionPayload {
                data: match tx.0.data() {
                    Some(data) => format!("0x{}", hex::encode(data)),
                    None => "0x".into(),
                },
                to: match tx.0.to_addr() {
                    Some(addr) => format!("0x{}", hex::encode(addr)),
                    None => return Err(HoprChainError::Api("cannot set transaction target".into())),
                },
                value: match tx.0.value() {
                    Some(x) => x.to_string(),
                    None => "".into(),
                },
            };

            match await_js_promise(self.send_transaction.call2(
                &JsValue::undefined(),
                &JsValue::from(payload),
                &JsString::from(tx.1).into(),
            ))
            .await
            {
                Ok(v) => {
                    if let Ok(result) = serde_wasm_bindgen::from_value::<WasmSendTransactionResult>(v) {
                        let val = result.code.to_uppercase();
                        match val.as_str() {
                            "SUCCESS" => Ok(result
                                .tx
                                .and_then(|tx| Hash::from_hex(&tx).ok())
                                .expect("invalid tx hash returned")),
                            _ => Err(HoprChainError::Api(format!("tx sender error: {result:?}"))),
                        }
                    } else {
                        Err(HoprChainError::Api("serde deserialization error".into()))
                    }
                }
                Err(e) => Err(HoprChainError::Api(e)),
            }
        }
    }

    #[derive(Clone)]
    pub struct WasmTaggingPayloadGenerator<Wrapped: PayloadGenerator<TypedTransaction>>(pub Wrapped);

    impl<Wrapped: PayloadGenerator<TypedTransaction>> PayloadGenerator<TypedTransactionWithTag>
        for WasmTaggingPayloadGenerator<Wrapped>
    {
        fn approve(
            &self,
            spender: Address,
            amount: Balance,
        ) -> core_ethereum_actions::errors::Result<TypedTransactionWithTag> {
            self.0
                .approve(spender, amount)
                .map(|tx| TypedTransactionWithTag(tx, "approve-".into()))
        }

        fn transfer(
            &self,
            destination: Address,
            amount: Balance,
        ) -> core_ethereum_actions::errors::Result<TypedTransactionWithTag> {
            match amount.balance_type() {
                BalanceType::HOPR => self
                    .0
                    .transfer(destination, amount)
                    .map(|tx| TypedTransactionWithTag(tx, "withdraw-hopr-".into())),
                BalanceType::Native => self
                    .0
                    .transfer(destination, amount)
                    .map(|tx| TypedTransactionWithTag(tx, "withdraw-native-".into())),
            }
        }

        fn announce(
            &self,
            announcement: AnnouncementData,
        ) -> core_ethereum_actions::errors::Result<TypedTransactionWithTag> {
            self.0
                .announce(announcement)
                .map(|tx| TypedTransactionWithTag(tx, "announce-".into()))
        }

        fn fund_channel(
            &self,
            dest: Address,
            amount: Balance,
        ) -> core_ethereum_actions::errors::Result<TypedTransactionWithTag> {
            self.0
                .fund_channel(dest, amount)
                .map(|tx| TypedTransactionWithTag(tx, "channel-updated-".into()))
        }

        fn close_incoming_channel(
            &self,
            source: Address,
        ) -> core_ethereum_actions::errors::Result<TypedTransactionWithTag> {
            self.0
                .close_incoming_channel(source)
                .map(|tx| TypedTransactionWithTag(tx, "channel-updated-".into()))
        }

        fn initiate_outgoing_channel_closure(
            &self,
            destination: Address,
        ) -> core_ethereum_actions::errors::Result<TypedTransactionWithTag> {
            self.0
                .initiate_outgoing_channel_closure(destination)
                .map(|tx| TypedTransactionWithTag(tx, "channel-updated-".into()))
        }

        fn finalize_outgoing_channel_closure(
            &self,
            destination: Address,
        ) -> core_ethereum_actions::errors::Result<TypedTransactionWithTag> {
            self.0
                .finalize_outgoing_channel_closure(destination)
                .map(|tx| TypedTransactionWithTag(tx, "channel-updated-".into()))
        }

        fn redeem_ticket(
            &self,
            acked_ticket: AcknowledgedTicket,
        ) -> core_ethereum_actions::errors::Result<TypedTransactionWithTag> {
            self.0
                .redeem_ticket(acked_ticket)
                .map(|tx| TypedTransactionWithTag(tx, "channel-updated-".into()))
        }

        fn register_safe_by_node(
            &self,
            safe_addr: Address,
        ) -> core_ethereum_actions::errors::Result<TypedTransactionWithTag> {
            self.0
                .register_safe_by_node(safe_addr)
                .map(|tx| TypedTransactionWithTag(tx, "node-safe-registered-".into()))
        }

        fn deregister_node_by_safe(&self) -> core_ethereum_actions::errors::Result<TypedTransactionWithTag> {
            self.0
                .deregister_node_by_safe()
                .map(|tx| TypedTransactionWithTag(tx, "node-safe-deregistered-".into()))
        }
    }

    pub type WasmEthereumTransactionExecutor<PGen> =
        EthereumTransactionExecutor<TypedTransactionWithTag, WasmEthereumClient, WasmTaggingPayloadGenerator<PGen>>;
}
