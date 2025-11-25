use std::{str::FromStr, time::Duration};

use hopr_api::chain::{ChainInfo, DomainSeparators};
use hopr_chain_types::chain_events::ChainEvent;
use hopr_crypto_types::types::Hash;
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::*;

use crate::errors::ConnectorError;

pub(crate) fn model_to_account_entry(
    model: blokli_client::api::types::Account,
) -> Result<AccountEntry, ConnectorError> {
    let entry_type = if !model.multi_addresses.is_empty() {
        AccountType::Announced(
            model
                .multi_addresses
                .into_iter()
                .filter_map(|addr| match Multiaddr::from_str(&addr) {
                    Ok(addr) => Some(addr),
                    Err(_) => {
                        tracing::error!(%addr, "invalid multiaddress");
                        None
                    }
                })
                .collect(),
        )
    } else {
        AccountType::NotAnnounced
    };

    Ok(AccountEntry {
        public_key: model.packet_key.parse()?,
        chain_addr: model.chain_key.parse()?,
        key_id: (model.keyid as u32).into(),
        entry_type,
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

pub(crate) fn model_to_ticket_params(
    model: blokli_client::api::types::TicketParameters,
) -> Result<(HoprBalance, WinningProbability), ConnectorError> {
    Ok((
        model.ticket_price.0.parse()?,
        WinningProbability::try_from_f64(model.min_ticket_winning_probability)?,
    ))
}

#[derive(Debug, Clone)]
pub(crate) struct ParsedChainInfo {
    pub channel_closure_grace_period: Duration,
    pub domain_separators: DomainSeparators,
    pub info: ChainInfo,
    pub ticket_win_prob: WinningProbability,
    pub ticket_price: HoprBalance,
}

pub(crate) fn model_to_chain_info(
    model: blokli_client::api::types::ChainInfo,
) -> Result<ParsedChainInfo, ConnectorError> {
    Ok(ParsedChainInfo {
        channel_closure_grace_period: model
            .channel_closure_grace_period
            .ok_or(ConnectorError::TypeConversion("missing channel grace period".into()))
            .and_then(|v| {
                v.0.parse()
                    .map(Duration::from_secs)
                    .map_err(|e| ConnectorError::TypeConversion(format!("invalid channel grace period: {e}")))
            })?,
        domain_separators: DomainSeparators {
            ledger: model
                .ledger_dst
                .ok_or(ConnectorError::TypeConversion("missing ledger dst".into()))
                .and_then(|v| {
                    Hash::from_hex(&v).map_err(|e| ConnectorError::TypeConversion(format!("invalid ledger dst: {e}")))
                })?,
            safe_registry: model
                .safe_registry_dst
                .ok_or(ConnectorError::TypeConversion("missing safe registry dst".into()))
                .and_then(|v| {
                    Hash::from_hex(&v)
                        .map_err(|e| ConnectorError::TypeConversion(format!("invalid safe registry dst: {e}")))
                })?,
            channel: model
                .channel_dst
                .ok_or(ConnectorError::TypeConversion("missing channel dst".into()))
                .and_then(|v| {
                    Hash::from_hex(&v).map_err(|e| ConnectorError::TypeConversion(format!("invalid channel dst: {e}")))
                })?,
        },
        info: ChainInfo {
            chain_id: model.chain_id as u64,
            hopr_network_name: model.network,
            contract_addresses: serde_json::from_str(&model.contract_addresses.0)
                .map_err(|e| ConnectorError::TypeConversion(format!("invalid contract addresses JSON: {e}")))?,
        },
        ticket_win_prob: WinningProbability::try_from_f64(model.min_ticket_winning_probability)
            .map_err(|e| ConnectorError::TypeConversion(format!("invalid winning probability info: {e}")))?,
        ticket_price: model
            .ticket_price
            .0
            .parse()
            .map_err(|e| ConnectorError::TypeConversion(format!("invalid ticket price: {e}")))?,
    })
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
