use std::sync::Arc;
use async_lock::RwLock;
use core_crypto::types::Hash;
use core_ethereum_db::traits::HoprCoreEthereumDbActions;
use core_ethereum_misc::errors::CoreEthereumError::{ChannelAlreadyClosed, ChannelAlreadyExists, ChannelDoesNotExist, InvalidArguments, InvalidState};
use crate::transaction_queue::{Transaction, TransactionCompleted, TransactionSender};
use core_ethereum_misc::errors::Result;
use core_types::channels::{ChannelDirection, ChannelStatus};
use utils_log::info;
use utils_types::primitives::{Address, Balance, BalanceType};

pub async fn open_channel<Db>(db: Arc<RwLock<Db>>, tx_sender: TransactionSender, destination: Address, self_addr: Address, amount: Balance) -> Result<TransactionCompleted>
where Db: HoprCoreEthereumDbActions {
    if amount.eq(&amount.of_same("0")) || amount.balance_type() != BalanceType::HOPR {
        return Err(InvalidArguments("invalid balance or balance type given".into()))
    }

    if db.read().await.get_channel_x(&self_addr, &destination).await?.is_some() {
        return Err(ChannelAlreadyExists)
    }

    tx_sender.send(Transaction::OpenChannel(destination, amount)).await
}

pub async fn fund_channel<Db>(db: Arc<RwLock<Db>>, tx_sender: TransactionSender, channel_id: Hash, amount: Balance) -> Result<TransactionCompleted>
where Db: HoprCoreEthereumDbActions {

    if amount.eq(&amount.of_same("0")) || amount.balance_type() != BalanceType::HOPR {
        return Err(InvalidArguments("invalid balance or balance type given".into()))
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
        None => Err(ChannelDoesNotExist)
    }
}

pub async fn close_channel<Db>(db: Arc<RwLock<Db>>, tx_sender: TransactionSender, counterparty: Address, self_address: Address, direction: ChannelDirection) -> Result<TransactionCompleted>
where Db: HoprCoreEthereumDbActions {

    let maybe_channel = match direction {
        ChannelDirection::Incoming => db.read().await.get_channel_x(&counterparty, &self_address).await?,
        ChannelDirection::Outgoing => db.read().await.get_channel_x(&self_address, &counterparty).await?,
    };

    match maybe_channel {
        Some(channel) => {
            let channel_id = channel.get_id();
            match channel.status {
                ChannelStatus::Closed => {
                    Err(ChannelAlreadyClosed)
                }
                ChannelStatus::Open | ChannelStatus::PendingToClose => {
                    if channel.status == ChannelStatus::Open {
                        info!("closing channel {channel_id}");
                        // TODO: emit "channel will close" event
                    }

                    tx_sender.send(Transaction::CloseChannel(channel)).await
                }
            }
        },
        None => Err(ChannelDoesNotExist)
    }

}

pub async fn withdraw(tx_sender: TransactionSender, recipient: Address, amount: Balance) -> Result<TransactionCompleted> {
    if amount.eq(&amount.of_same("0")) {
        return Err(InvalidArguments("cannot withdraw zero amount".into()))
    }

    tx_sender.send(Transaction::Withdraw(recipient, amount)).await
}

