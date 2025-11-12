use std::str::FromStr;

use hopr_chain_types::chain_events::ChainEvent;
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::{Address, ToHex};

use crate::errors::ConnectorError;

pub(crate) fn model_to_account_entry(
    model: blokli_client::api::types::Account,
) -> Result<AccountEntry, ConnectorError> {
    Ok(AccountEntry {
        public_key: model.packet_key.parse()?,
        chain_addr: model.chain_key.parse()?,
        key_id: (model.keyid as u32).into(),
        entry_type: if let Some(maddr) = model.multi_addresses.first() {
            AccountType::Announced(
                maddr
                    .parse()
                    .map_err(|e| ConnectorError::TypeConversion(format!("invalid multiaddress {maddr}: {e}")))?,
            )
        } else {
            AccountType::NotAnnounced
        },
        safe_address: model.safe_address.map(|addr| Address::from_hex(&addr)).transpose()?,
    })
}

pub(crate) fn model_to_graph_entry(
    model: blokli_client::api::types::OpenedChannelsGraphEntry,
) -> Result<(AccountEntry, AccountEntry, ChannelEntry), ConnectorError> {
    let src = model_to_account_entry(model.source)?;
    let dst = model_to_account_entry(model.destination)?;
    let channel = ChannelEntry::new(
        src.chain_addr,
        dst.chain_addr,
        model.channel.balance.0.parse()?,
        model
            .channel
            .ticket_index
            .0
            .parse()
            .map_err(|e| ConnectorError::TypeConversion(format!("invalid ticket index: {e}")))?,
        match model.channel.status {
            blokli_client::api::types::ChannelStatus::Open => ChannelStatus::Open,
            blokli_client::api::types::ChannelStatus::PendingToClose => ChannelStatus::PendingToClose(
                model
                    .channel
                    .closure_time
                    .as_ref()
                    .ok_or(ConnectorError::TypeConversion("invalid closure time".into()))
                    .and_then(|t| {
                        hopr_api::chain::DateTime::from_str(&t.0)
                            .map_err(|e| ConnectorError::TypeConversion(format!("invalid closure time: {e}")))
                    })?
                    .into(),
            ),
            blokli_client::api::types::ChannelStatus::Closed => ChannelStatus::Closed,
        },
        (model.channel.epoch as u32).into(),
    );
    Ok((src, dst, channel))
}

pub(crate) async fn process_channel_changes_into_events(
    new_channel: ChannelEntry,
    changes: Vec<ChannelChange>,
    me: &Address,
    event_tx: &async_broadcast::Sender<ChainEvent>,
) {
    for change in changes {
        tracing::trace!(id = %new_channel.get_id(), %change, "channel updated");
        match change {
            ChannelChange::Status {
                left: ChannelStatus::Open,
                right: ChannelStatus::PendingToClose(_),
            } => {
                tracing::debug!(id = %new_channel.get_id(), "channel pending to close");
                let _ = event_tx
                    .broadcast_direct(ChainEvent::ChannelClosureInitiated(new_channel))
                    .await;
            }
            ChannelChange::Status {
                left: ChannelStatus::PendingToClose(_),
                right: ChannelStatus::Closed,
            } => {
                tracing::debug!(id = %new_channel.get_id(), "channel closed");
                let _ = event_tx.broadcast_direct(ChainEvent::ChannelClosed(new_channel)).await;
            }
            ChannelChange::Status {
                left: ChannelStatus::Closed,
                right: ChannelStatus::Open,
            } => {
                tracing::debug!(id = %new_channel.get_id(), "channel reopened");
                let _ = event_tx.broadcast_direct(ChainEvent::ChannelOpened(new_channel)).await;
            }
            ChannelChange::Balance { left, right } => {
                if left > right {
                    tracing::debug!(id = %new_channel.get_id(), "channel balance decreased");
                    let _ = event_tx
                        .broadcast_direct(ChainEvent::ChannelBalanceDecreased(new_channel, left - right))
                        .await;
                } else {
                    tracing::debug!(id = %new_channel.get_id(), "channel balance increased");
                    let _ = event_tx
                        .broadcast_direct(ChainEvent::ChannelBalanceIncreased(new_channel, right - left))
                        .await;
                }
            }
            // Ticket index can wrap (left > right) on a channel re-open,
            // but we're not interested in that here
            ChannelChange::TicketIndex { left, right } if left < right => match new_channel.direction(me) {
                Some(ChannelDirection::Incoming) => {
                    // The corresponding event is raised in the ticket redeem tracker,
                    // as the failure must be tracked there too.
                    tracing::debug!(id = %new_channel.get_id(), "ticket redemption succeeded");
                }
                Some(ChannelDirection::Outgoing) => {
                    tracing::debug!(id = %new_channel.get_id(), "counterparty has redeemed ticket on our channel");
                    let _ = event_tx
                        .broadcast_direct(ChainEvent::TicketRedeemed(new_channel, None))
                        .await;
                }
                None => {
                    tracing::debug!(id = %new_channel.get_id(), "ticket redeemed on foreign channel");
                    let _ = event_tx
                        .broadcast_direct(ChainEvent::TicketRedeemed(new_channel, None))
                        .await;
                }
            },
            _ => {}
        }
    }
}
