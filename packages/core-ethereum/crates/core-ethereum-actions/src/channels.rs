use crate::transaction_queue::{Transaction, TransactionCompleted, TransactionSender};
use async_lock::RwLock;
use core_crypto::types::Hash;
use core_ethereum_db::traits::HoprCoreEthereumDbActions;
use core_ethereum_misc::errors::CoreEthereumError::{
    ChannelAlreadyClosed, ChannelAlreadyExists, ChannelDoesNotExist, InvalidArguments, InvalidState,
};
use core_ethereum_misc::errors::Result;
use core_types::channels::{ChannelDirection, ChannelStatus};
use std::sync::Arc;
use utils_log::{error, info};
use utils_types::primitives::{Address, Balance, BalanceType};

pub async fn open_channel<Db>(
    db: Arc<RwLock<Db>>,
    tx_sender: TransactionSender,
    destination: Address,
    self_addr: Address,
    amount: Balance,
) -> Result<TransactionCompleted>
where
    Db: HoprCoreEthereumDbActions,
{
    if amount.eq(&amount.of_same("0")) || amount.balance_type() != BalanceType::HOPR {
        return Err(InvalidArguments("invalid balance or balance type given".into()));
    }

    let maybe_channel = db.read().await.get_channel_x(&self_addr, &destination).await?;
    if let Some(channel) = maybe_channel {
        if channel.status != ChannelStatus::Closed {
            error!("channel to {destination} is already opened or pending to close");
            return Err(ChannelAlreadyExists);
        }
    }

    tx_sender.send(Transaction::OpenChannel(destination, amount)).await
}

pub async fn fund_channel<Db>(
    db: Arc<RwLock<Db>>,
    tx_sender: TransactionSender,
    channel_id: Hash,
    amount: Balance,
) -> Result<TransactionCompleted>
where
    Db: HoprCoreEthereumDbActions,
{
    if amount.eq(&amount.of_same("0")) || amount.balance_type() != BalanceType::HOPR {
        return Err(InvalidArguments("invalid balance or balance type given".into()));
    }

    let maybe_channel = db.read().await.get_channel(&channel_id).await?;
    match maybe_channel {
        Some(channel) => {
            if channel.status == ChannelStatus::Open {
                tx_sender.send(Transaction::FundChannel(channel, amount)).await
            } else {
                Err(InvalidState(format!("channel {channel_id} is not opened")))
            }
        }
        None => Err(ChannelDoesNotExist),
    }
}

pub async fn close_channel<Db>(
    db: Arc<RwLock<Db>>,
    tx_sender: TransactionSender,
    counterparty: Address,
    self_address: Address,
    direction: ChannelDirection,
) -> Result<TransactionCompleted>
where
    Db: HoprCoreEthereumDbActions,
{
    let maybe_channel = match direction {
        ChannelDirection::Incoming => db.read().await.get_channel_x(&counterparty, &self_address).await?,
        ChannelDirection::Outgoing => db.read().await.get_channel_x(&self_address, &counterparty).await?,
    };

    match maybe_channel {
        Some(channel) => {
            let channel_id = channel.get_id();
            match channel.status {
                ChannelStatus::Closed => Err(ChannelAlreadyClosed),
                ChannelStatus::Open | ChannelStatus::PendingToClose => {
                    if channel.status == ChannelStatus::Open {
                        info!("closing channel {channel_id}");
                        // TODO: emit "channel will close" event
                    }

                    tx_sender.send(Transaction::CloseChannel(channel)).await
                }
            }
        }
        None => Err(ChannelDoesNotExist),
    }
}

pub async fn withdraw(
    tx_sender: TransactionSender,
    recipient: Address,
    amount: Balance,
) -> Result<TransactionCompleted> {
    if amount.eq(&amount.of_same("0")) {
        return Err(InvalidArguments("cannot withdraw zero amount".into()));
    }

    tx_sender.send(Transaction::Withdraw(recipient, amount)).await
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use crate::transaction_queue::{TransactionResult, TransactionSender};
    use core_crypto::types::Hash;
    use core_ethereum_db::db::wasm::Database;
    use core_types::channels::{ChannelDirection, ChannelStatus};
    use utils_misc::utils::wasm::JsResult;
    use utils_types::primitives::{Address, Balance};
    use wasm_bindgen::prelude::wasm_bindgen;
    use wasm_bindgen::JsValue;

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
    pub async fn open_channel(
        db: &Database,
        destination: &Address,
        self_addr: &Address,
        amount: &Balance,
        on_chain_tx_sender: &TransactionSender,
    ) -> JsResult<OpenChannelResult> {
        let awaiter = super::open_channel(
            db.as_ref_counted(),
            on_chain_tx_sender.clone(),
            *destination,
            *self_addr,
            *amount,
        )
        .await?;
        match awaiter
            .await
            .map_err(|_| JsValue::from("transaction has been cancelled".to_string()))?
        {
            TransactionResult::OpenChannel { tx_hash, channel_id } => Ok(OpenChannelResult { tx_hash, channel_id }),
            _ => Err(JsValue::from("open channel transaction failed".to_string())),
        }
    }

    #[wasm_bindgen]
    pub async fn fund_channel(
        db: &Database,
        channel_id: &Hash,
        amount: &Balance,
        on_chain_tx_sender: &TransactionSender,
    ) -> JsResult<Hash> {
        let awaiter =
            super::fund_channel(db.as_ref_counted(), on_chain_tx_sender.clone(), *channel_id, *amount).await?;
        match awaiter
            .await
            .map_err(|_| JsValue::from("transaction has been cancelled".to_string()))?
        {
            TransactionResult::FundChannel { tx_hash } => Ok(tx_hash),
            _ => Err(JsValue::from("fund channel transaction failed".to_string())),
        }
    }

    #[wasm_bindgen]
    pub async fn close_channel(
        db: &Database,
        counterparty: &Address,
        self_addr: &Address,
        direction: ChannelDirection,
        on_chain_tx_sender: &TransactionSender,
    ) -> JsResult<CloseChannelResult> {
        let awaiter = super::close_channel(
            db.as_ref_counted(),
            on_chain_tx_sender.clone(),
            *counterparty,
            *self_addr,
            direction,
        )
        .await?;
        match awaiter
            .await
            .map_err(|_| JsValue::from("transaction has been cancelled".to_string()))?
        {
            TransactionResult::CloseChannel { tx_hash, status } => Ok(CloseChannelResult { tx_hash, status }),
            _ => Err(JsValue::from("close channel transaction failed".to_string())),
        }
    }

    #[wasm_bindgen]
    pub async fn withdraw(
        recipient: &Address,
        amount: &Balance,
        on_chain_tx_sender: &TransactionSender,
    ) -> JsResult<Hash> {
        let awaiter = super::withdraw(on_chain_tx_sender.clone(), *recipient, *amount).await?;
        match awaiter
            .await
            .map_err(|_| JsValue::from("transaction has been cancelled".to_string()))?
        {
            TransactionResult::Withdraw { tx_hash } => Ok(tx_hash),
            _ => Err(JsValue::from("withdraw transaction failed".to_string())),
        }
    }
}
