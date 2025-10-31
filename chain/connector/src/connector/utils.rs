use std::str::FromStr;
use std::time::Duration;
use blokli_client::api::{BlokliTransactionClient};
use hopr_api::chain::ChainReceipt;
use hopr_crypto_types::prelude::Hash;
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::{Address, ToHex};
use futures::TryFutureExt;

use crate::errors::ConnectorError;

const CLIENT_TX_TIMEOUT: Duration = Duration::from_secs(30);

pub(crate) fn model_to_account_entry(model: blokli_client::api::types::Account) -> Result<AccountEntry, ConnectorError> {
    Ok(AccountEntry {
        public_key: model.packet_key.parse()?,
        chain_addr: model.chain_key.parse()?,
        key_id: (model.keyid as u32).into(),
        entry_type: if let Some(maddr) = model.multi_addresses.first() {
            AccountType::Announced {
                multiaddr: maddr.parse().map_err(|e| ConnectorError::TypeConversion(format!("invalid multiaddress {maddr}: {e}")))?,
                updated_block: 0,
            }
        } else {
            AccountType::NotAnnounced
        },
        safe_address: model.safe_address.map(|addr| Address::from_hex(&addr)).transpose()?,
    })
}

pub(crate) fn model_to_graph_entry(model: blokli_client::api::types::OpenedChannelsGraphEntry) -> Result<(AccountEntry, AccountEntry, ChannelEntry), ConnectorError> {
    let src = model_to_account_entry(model.source)?;
    let dst = model_to_account_entry(model.destination)?;
    let channel = ChannelEntry::new(
        src.chain_addr,
        dst.chain_addr,
        model.channel.balance.0.parse()?,
        model.channel.ticket_index.0.parse().map_err(|e| ConnectorError::TypeConversion(format!("invalid ticket index: {e}")))?,
        match model.channel.status {
            blokli_client::api::types::ChannelStatus::Open => ChannelStatus::Open,
            blokli_client::api::types::ChannelStatus::PendingToClose => ChannelStatus::PendingToClose(
                model.channel
                    .closure_time
                    .as_ref()
                    .ok_or(ConnectorError::TypeConversion("invalid closure time".into()))
                    .and_then(|t| hopr_api::chain::DateTime::from_str(&t.0)
                        .map_err(|e| ConnectorError::TypeConversion(format!("invalid closure time: {e}")))
                    )?
                .into()),
            blokli_client::api::types::ChannelStatus::Closed => ChannelStatus::Closed
        },
        (model.channel.epoch as u32).into()
    );
    Ok((src, dst, channel))
}

pub(crate) fn track_transaction<'a, B: BlokliTransactionClient + Send + Sync + 'static>(client: &'a B, tx_id: blokli_client::api::TxId)
    -> Result<impl futures::Future<Output = Result<ChainReceipt, ConnectorError>> + Send + 'a, ConnectorError> {
    Ok(client
        .track_transaction(tx_id, CLIENT_TX_TIMEOUT)
        .map_err(ConnectorError::from)
        .and_then(|tx| futures::future::ready(
            tx.transaction_hash
                .ok_or(ConnectorError::ClientError(blokli_client::errors::ErrorKind::NoData.into()))
                .and_then(|hash| Hash::from_hex(&hash.0).map_err(ConnectorError::from))
        )))

}