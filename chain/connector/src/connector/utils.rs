use std::str::FromStr;
use std::time::Duration;
use blokli_client::api::{BlokliTransactionClient};
use hopr_api::chain::ChainReceipt;
use hopr_crypto_types::prelude::Hash;
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::{Address, ToHex};
use futures::TryFutureExt;
use hopr_chain_types::chain_events::ChainEvent;
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

pub(crate) async fn process_account_changes_into_events(new_account: AccountEntry, maybe_old_account: Option<AccountEntry>, event_tx: &async_broadcast::Sender<ChainEvent>) {
    // We only track public accounts as events
    if new_account.has_announced() {
        tracing::debug!(account = %new_account, "new announcement");
        let _ = event_tx.broadcast_direct(ChainEvent::Announcement {
            peer: new_account.public_key.into(),
            address: new_account.chain_addr,
            multiaddresses: new_account.get_multiaddr().into_iter().collect(),
        }).await;
    }
    if let Some(safe_addr) = new_account.safe_address {
        // We only emit this event when there was an account previously,
        // but it didn't have a safe address yet.
        if maybe_old_account.is_some_and(|a| a.safe_address.is_none()) {
            tracing::debug!(account = %new_account, "registered safe address");
            let _ = event_tx.broadcast_direct(ChainEvent::NodeSafeRegistered(safe_addr)).await;
        }
    }
}

pub(crate)  async fn process_channel_changes_into_events(
    new_channel: ChannelEntry,
    changes: Vec<ChannelChange>,
    me: &Address,
    event_tx: &async_broadcast::Sender<ChainEvent>,
    redeemed_ticket_queue: &moka::future::Cache<TicketId, Box<VerifiedTicket>, ahash::RandomState>,
) {
    for change in changes {
        tracing::trace!(id = %new_channel.get_id(), %change, "channel updated");
        match change {
            ChannelChange::Status { left: ChannelStatus::Open, right: ChannelStatus::PendingToClose(_) } => {
                tracing::debug!(id = %new_channel.get_id(), "channel pending to close");
                let _ = event_tx.broadcast_direct(ChainEvent::ChannelClosureInitiated(new_channel.clone())).await;
            }
            ChannelChange::Status {left: ChannelStatus::PendingToClose(_), right: ChannelStatus::Closed} => {
                tracing::debug!(id = %new_channel.get_id(), "channel closed");
                let _ = event_tx.broadcast_direct(ChainEvent::ChannelClosed(new_channel.clone())).await;
            }
            ChannelChange::Status { left: ChannelStatus::Closed, right: ChannelStatus::Open } => {
                tracing::debug!(id = %new_channel.get_id(), "channel reopened");
                let _ = event_tx.broadcast_direct(ChainEvent::ChannelOpened(new_channel.clone())).await;
            }
            ChannelChange::Balance { left, right } => {
                if left > right {
                    tracing::debug!(id = %new_channel.get_id(), "channel balance decreased");
                    let _ = event_tx.broadcast_direct(ChainEvent::ChannelBalanceDecreased(new_channel.clone(), left - right)).await;
                } else {
                    tracing::debug!(id = %new_channel.get_id(), "channel balance increased");
                    let _ = event_tx.broadcast_direct(ChainEvent::ChannelBalanceIncreased(new_channel.clone(), right - left)).await;
                }
            }
            // Ticket index can wrap (left > right) on a channel re-open,
            // but we're not interested in that here
            ChannelChange::TicketIndex { left, right } if left < right => {
                match new_channel.direction(me) {
                    Some(ChannelDirection::Incoming) => {
                        if let Some(redeemed_ticket) = redeemed_ticket_queue.remove(&TicketId {
                            id: new_channel.get_id(),
                            epoch: new_channel.channel_epoch.as_u32(),
                            index: right,
                        }).await {
                            tracing::debug!(id = %new_channel.get_id(), "ticket redeemed on own channel");
                            let _ = event_tx.broadcast_direct(ChainEvent::TicketRedeemed(new_channel.clone(), Some(redeemed_ticket))).await;
                        } else {
                            tracing::error!("got ticket redemption event on ticket that's no longer in the redemption cache");
                        }
                    },
                    Some(ChannelDirection::Outgoing) => {
                        tracing::debug!(id = %new_channel.get_id(), "counterparty has redeemed ticket on our channel");
                        let _ = event_tx.broadcast_direct(ChainEvent::TicketRedeemed(new_channel.clone(), None)).await;
                    },
                    None => {
                        tracing::debug!(id = %new_channel.get_id(), "ticket redeemed on foreign channel");
                        let _ = event_tx.broadcast_direct(ChainEvent::TicketRedeemed(new_channel.clone(), None)).await;
                    }
                }
            }
            _ => {}
        }
    }
}