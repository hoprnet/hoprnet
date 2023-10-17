pub mod channels;
pub mod errors;
pub mod node;
pub mod redeem;
pub mod transaction_queue;

use crate::transaction_queue::TransactionSender;
use async_lock::RwLock;
use core_ethereum_db::traits::HoprCoreEthereumDbActions;
use std::sync::Arc;
use utils_types::primitives::Address;

/// Contains all actions that a node can execute on-chain.
#[derive(Clone)]
pub struct CoreEthereumActions<Db: HoprCoreEthereumDbActions + Clone> {
    me: Address,
    db: Arc<RwLock<Db>>,
    tx_sender: TransactionSender,
}

impl<Db: HoprCoreEthereumDbActions + Clone> CoreEthereumActions<Db> {
    /// Creates new instance.
    pub fn new(me: Address, db: Arc<RwLock<Db>>, tx_sender: TransactionSender) -> Self {
        Self { me, db, tx_sender }
    }

    /// On-chain address of this node
    pub fn self_address(&self) -> Address {
        self.me
    }
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use crate::channels::ChannelActions;
    use crate::node::NodeActions;
    use crate::redeem::TicketRedeemActions;
    use crate::transaction_queue::{TransactionResult, TransactionSender};
    use crate::CoreEthereumActions;
    use core_crypto::types::Hash;
    use core_ethereum_db::db::wasm::Database;
    use core_ethereum_db::db::CoreEthereumDb;
    use core_types::acknowledgement::wasm::AcknowledgedTicket;
    use core_types::channels::{ChannelDirection, ChannelEntry, ChannelStatus};
    use utils_db::rusty::RustyLevelDbShim;
    use utils_misc::utils::wasm::JsResult;
    use utils_types::primitives::{Address, Balance};
    use wasm_bindgen::prelude::wasm_bindgen;
    use wasm_bindgen::JsValue;

    #[derive(Clone)]
    #[wasm_bindgen]
    pub struct WasmCoreEthereumActions {
        w: CoreEthereumActions<CoreEthereumDb<RustyLevelDbShim>>,
    }

    impl WasmCoreEthereumActions {
        pub fn inner_ref(&self) -> &CoreEthereumActions<CoreEthereumDb<RustyLevelDbShim>> {
            &self.w
        }

        pub fn new_from_actions(actions: CoreEthereumActions<CoreEthereumDb<RustyLevelDbShim>>) -> Self {
            Self { w: actions }
        }
    }

    #[wasm_bindgen(getter_with_clone)]
    pub struct OpenChannelResult {
        pub tx_hash: Hash,
        pub channel_id: Hash,
    }

    #[wasm_bindgen(getter_with_clone)]
    pub struct CloseChannelResult {
        pub tx_hash: Hash,
        pub status: ChannelStatus,
    }

    #[wasm_bindgen]
    impl WasmCoreEthereumActions {
        #[wasm_bindgen(constructor)]
        pub fn new(me: Address, db: &Database, tx_sender: &TransactionSender) -> Self {
            Self {
                w: CoreEthereumActions::new(me, db.as_ref_counted(), tx_sender.clone()),
            }
        }

        pub async fn open_channel(&self, destination: &Address, amount: &Balance) -> JsResult<OpenChannelResult> {
            let awaiter = self.w.open_channel(*destination, *amount).await?;
            match awaiter
                .await
                .map_err(|_| JsValue::from("transaction has been cancelled".to_string()))?
            {
                TransactionResult::OpenChannel { tx_hash, channel_id } => Ok(OpenChannelResult { tx_hash, channel_id }),
                _ => Err(JsValue::from("open channel transaction failed".to_string())),
            }
        }

        pub async fn fund_channel(&self, channel_id: &Hash, amount: &Balance) -> JsResult<Hash> {
            let awaiter = self.w.fund_channel(*channel_id, *amount).await?;
            match awaiter
                .await
                .map_err(|_| JsValue::from("transaction has been cancelled".to_string()))?
            {
                TransactionResult::FundChannel { tx_hash } => Ok(tx_hash),
                _ => Err(JsValue::from("fund channel transaction failed".to_string())),
            }
        }

        pub async fn close_channel(
            &self,
            counterparty: &Address,
            direction: ChannelDirection,
            redeem_before_close: bool,
        ) -> JsResult<CloseChannelResult> {
            let awaiter = self
                .w
                .close_channel(*counterparty, direction, redeem_before_close)
                .await?;
            match awaiter
                .await
                .map_err(|_| JsValue::from("transaction has been cancelled".to_string()))?
            {
                TransactionResult::CloseChannel { tx_hash, status } => Ok(CloseChannelResult { tx_hash, status }),
                _ => Err(JsValue::from("close channel transaction failed".to_string())),
            }
        }

        pub async fn withdraw(&self, recipient: &Address, amount: &Balance) -> JsResult<Hash> {
            let awaiter = self.w.withdraw(*recipient, *amount).await?;
            match awaiter
                .await
                .map_err(|_| JsValue::from("transaction has been cancelled".to_string()))?
            {
                TransactionResult::Withdraw { tx_hash } => Ok(tx_hash),
                _ => Err(JsValue::from("withdraw transaction failed".to_string())),
            }
        }

        pub async fn redeem_all_tickets(&self, only_aggregated: bool) -> JsResult<()> {
            // We do not await the on-chain confirmation
            self.w.redeem_all_tickets(only_aggregated).await?;
            Ok(())
        }

        pub async fn redeem_tickets_with_counterparty(
            &self,
            counterparty: &Address,
            only_aggregated: bool,
        ) -> JsResult<()> {
            // We do not await the on-chain confirmation
            self.w
                .redeem_tickets_with_counterparty(counterparty, only_aggregated)
                .await?;
            Ok(())
        }

        pub async fn redeem_tickets_in_channel(&self, channel: &ChannelEntry, only_aggregated: bool) -> JsResult<()> {
            // We do not await the on-chain confirmation
            self.w.redeem_tickets_in_channel(channel, only_aggregated).await?;
            Ok(())
        }

        pub async fn redeem_ticket(&self, ack_ticket: &AcknowledgedTicket) -> JsResult<()> {
            // We do not await the on-chain confirmation
            let _ = self.w.redeem_ticket(ack_ticket.into()).await?;
            Ok(())
        }
    }
}
