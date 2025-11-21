use std::{ops::Add, str::FromStr};

use blokli_client::{BlokliTestState, BlokliTestStateMutator};
use hopr_chain_types::{ContractAddresses, ParsedHoprChainAction};
use hopr_internal_types::channels::generate_channel_id;
use hopr_primitive_types::{
    balance::{HoprBalance, XDaiBalance},
    prelude::Address,
};

/// A [`BlokliTestStateMutator`] that does not update the state.
///
/// Any attempt for a state change will raise an error.
#[derive(Clone, Debug)]
pub struct StaticState;

impl BlokliTestStateMutator for StaticState {
    fn update_state(&self, _: &[u8], _: &mut BlokliTestState) -> Result<(), blokli_client::errors::BlokliClientError> {
        Err(
            blokli_client::errors::ErrorKind::MockClientError(anyhow::anyhow!("static client must not update state"))
                .into(),
        )
    }
}

/// A [`BlokliTestStateMutator`] that updates the stated based on the actions parsed from the signed transaction.
/// This tries to emulate the behavior of the HOPR smart contracts on-chain.
#[derive(Clone, Debug)]
pub struct FullStateEmulator(
    pub(crate) Address,
    pub(crate) Option<futures::channel::mpsc::UnboundedSender<ParsedHoprChainAction>>,
);

const EMULATED_TX_PRICE: u128 = 1_u128;

impl FullStateEmulator {
    pub fn new(module: Address) -> Self {
        Self(module, None)
    }

    pub fn new_with_chain_events_interceptor(
        module: Address,
    ) -> (Self, impl futures::Stream<Item = ParsedHoprChainAction>) {
        let (sender, receiver) = futures::channel::mpsc::unbounded();
        (Self(module, Some(sender)), receiver)
    }
}

impl BlokliTestStateMutator for FullStateEmulator {
    fn update_state(
        &self,
        signed_tx: &[u8],
        state: &mut BlokliTestState,
    ) -> Result<(), blokli_client::errors::BlokliClientError> {
        let addresses: ContractAddresses =
            serde_json::from_str(&state.chain_info.contract_addresses.0).map_err(|_| {
                blokli_client::errors::ErrorKind::MockClientError(anyhow::anyhow!("failed to parse contract addresses"))
            })?;

        let (action, sender) = ParsedHoprChainAction::parse_from_eip2718(signed_tx, &self.0, &addresses)
            .map_err(|e| blokli_client::errors::ErrorKind::MockClientError(e.into()))?;
        tracing::debug!(%sender, ?action, "parsed action from signed transaction");

        match &action {
            ParsedHoprChainAction::RegisterSafeAddress(safe_address) => {
                if let Some(account) = state.get_account_mut(&sender.into()) {
                    account.safe_address = Some(hex::encode(safe_address));
                    tracing::debug!(%sender, %safe_address, "registered safe address to account");
                } else {
                    state
                        .unpaired_safes
                        .insert(hex::encode(sender), hex::encode(safe_address));
                    tracing::debug!(%sender, %safe_address, "registered safe address without account");
                }
            }
            ParsedHoprChainAction::Announce {
                packet_key,
                multiaddress,
            } => {
                if let Some(account) = state.get_account_mut(&sender.into()) {
                    account.packet_key = hex::encode(packet_key);
                    if let Some(multiaddress) = multiaddress.clone() {
                        if !multiaddress.is_empty() {
                            account.multi_addresses.push(multiaddress.to_string());
                        } else {
                            return Err(blokli_client::errors::ErrorKind::MockClientError(anyhow::anyhow!(
                                "multiaddress must not be empty"
                            ))
                            .into());
                        }
                    }
                    tracing::debug!(%sender, %packet_key, ?multiaddress, "node re-announced");
                } else {
                    let next_key_id = state.accounts.keys().max().map(|k| k + 1).unwrap_or(1);
                    state.accounts.insert(
                        next_key_id,
                        blokli_client::api::types::Account {
                            chain_key: hex::encode(sender),
                            keyid: next_key_id as i32,
                            multi_addresses: multiaddress.iter().map(|a| a.to_string()).collect(),
                            packet_key: hex::encode(packet_key),
                            safe_address: state.unpaired_safes.shift_remove(&hex::encode(sender)),
                        },
                    );
                    tracing::debug!(%sender, %packet_key, ?multiaddress, "node announced");
                }
            }
            ParsedHoprChainAction::WithdrawNative(destination, amount) => {
                let balance = state.native_balances.get_mut(&hex::encode(sender)).ok_or(
                    blokli_client::errors::ErrorKind::MockClientError(anyhow::anyhow!(
                        "missing native balance for {sender}"
                    )),
                )?;

                let balance_num = balance.balance.0.parse::<XDaiBalance>().map_err(|_| {
                    blokli_client::errors::ErrorKind::MockClientError(anyhow::anyhow!(
                        "failed to parse token balance for {sender}"
                    ))
                })?;

                if &balance_num < amount {
                    return Err(blokli_client::errors::ErrorKind::MockClientError(anyhow::anyhow!(
                        "balance {balance_num} for {sender} is lower than amount {amount}"
                    ))
                    .into());
                }

                balance.balance = blokli_client::api::types::TokenValueString((balance_num - *amount).to_string());

                if let Some(dst_balance) = state.native_balances.get_mut(&hex::encode(destination)) {
                    let new_balance = dst_balance.balance.0.parse::<XDaiBalance>().map_err(|_| {
                        blokli_client::errors::ErrorKind::MockClientError(anyhow::anyhow!(
                            "failed to parse native balance for {destination}"
                        ))
                    })? + *amount;
                    dst_balance.balance = blokli_client::api::types::TokenValueString(new_balance.to_string());

                    tracing::debug!(%sender, %amount, %destination, "xdai withdrawn to an existing account");
                } else {
                    tracing::debug!(%sender, %amount, %destination, "xdai withdrawn");
                }
            }
            ParsedHoprChainAction::WithdrawToken(destination, amount) => {
                let balance = state.token_balances.get_mut(&hex::encode(sender)).ok_or(
                    blokli_client::errors::ErrorKind::MockClientError(anyhow::anyhow!(
                        "missing token balance for {sender}"
                    )),
                )?;

                let balance_num = balance.balance.0.parse::<HoprBalance>().map_err(|_| {
                    blokli_client::errors::ErrorKind::MockClientError(anyhow::anyhow!(
                        "failed to parse token balance for {sender}"
                    ))
                })?;

                if &balance_num < amount {
                    return Err(blokli_client::errors::ErrorKind::MockClientError(anyhow::anyhow!(
                        "balance {balance_num} for {sender} is lower than amount {amount}"
                    ))
                    .into());
                }

                balance.balance = blokli_client::api::types::TokenValueString((balance_num - *amount).to_string());

                if let Some(dst_balance) = state.token_balances.get_mut(&hex::encode(destination)) {
                    let new_balance = dst_balance.balance.0.parse::<HoprBalance>().map_err(|_| {
                        blokli_client::errors::ErrorKind::MockClientError(anyhow::anyhow!(
                            "failed to parse token balance for {destination}"
                        ))
                    })? + *amount;
                    dst_balance.balance = blokli_client::api::types::TokenValueString(new_balance.to_string());

                    tracing::debug!(%sender, %amount, %destination, "wxhopr withdrawn to an existing account");
                } else {
                    tracing::debug!(%sender, %amount, %destination, "wxhopr withdrawn");
                }
            }
            ParsedHoprChainAction::FundChannel(dst_addr, stake) => {
                let source = state.get_account(&sender.into()).cloned().ok_or(
                    blokli_client::errors::ErrorKind::MockClientError(anyhow::anyhow!("missing account for {sender}")),
                )?;
                let destination = state.get_account(&(*dst_addr).into()).cloned().ok_or(
                    blokli_client::errors::ErrorKind::MockClientError(anyhow::anyhow!(
                        "missing account for {dst_addr}"
                    )),
                )?;

                if stake.is_zero() {
                    return Err(blokli_client::errors::ErrorKind::MockClientError(anyhow::anyhow!(
                        "stake must be greater than zero"
                    ))
                    .into());
                }

                {
                    let safe_balance = state.get_account_safe_token_balance_mut(&sender.into()).ok_or(
                        blokli_client::errors::ErrorKind::MockClientError(anyhow::anyhow!(
                            "missing safe balance for {sender}"
                        )),
                    )?;

                    let safe_balance_num = safe_balance.balance.0.parse::<HoprBalance>().map_err(|_| {
                        blokli_client::errors::ErrorKind::MockClientError(anyhow::anyhow!(
                            "failed to parse safe balance for safe of {sender}"
                        ))
                    })?;

                    if &safe_balance_num < stake {
                        return Err(blokli_client::errors::ErrorKind::MockClientError(anyhow::anyhow!(
                            "safe balance of {sender} for {sender} is lower than stake {stake}"
                        ))
                        .into());
                    }

                    safe_balance.balance =
                        blokli_client::api::types::TokenValueString((safe_balance_num - *stake).to_string());
                }

                {
                    let safe_allowance = state.get_account_safe_allowance_mut(&sender.into()).ok_or(
                        blokli_client::errors::ErrorKind::MockClientError(anyhow::anyhow!(
                            "missing safe allowance for {sender}"
                        )),
                    )?;

                    let safe_allowance_num = safe_allowance.allowance.0.parse::<HoprBalance>().map_err(|_| {
                        blokli_client::errors::ErrorKind::MockClientError(anyhow::anyhow!(
                            "failed to parse safe allowance for {sender}"
                        ))
                    })?;

                    if &safe_allowance_num < stake {
                        return Err(blokli_client::errors::ErrorKind::MockClientError(anyhow::anyhow!(
                            "safe allowance for {sender} is lower than stake {stake}"
                        ))
                        .into());
                    }

                    safe_allowance.allowance =
                        blokli_client::api::types::TokenValueString((safe_allowance_num - *stake).to_string());
                }

                if let Some(existing_channel) = state
                    .channels
                    .values_mut()
                    .find(|c| c.source == source.keyid && c.destination == destination.keyid)
                {
                    if existing_channel.status == blokli_client::api::types::ChannelStatus::Closed {
                        existing_channel.status = blokli_client::api::types::ChannelStatus::Open;
                        existing_channel.ticket_index = blokli_client::api::types::Uint64("0".into());
                        existing_channel.closure_time = None;
                        existing_channel.epoch += 1;

                        tracing::debug!(%sender, %dst_addr, %stake, "channel re-opened");
                    }

                    let balance = existing_channel.balance.0.parse::<HoprBalance>().map_err(|_| {
                        blokli_client::errors::ErrorKind::MockClientError(anyhow::anyhow!(
                            "failed to parse balance on channel {sender} -> {dst_addr}"
                        ))
                    })?;

                    existing_channel.balance =
                        blokli_client::api::types::TokenValueString((balance + *stake).to_string());

                    tracing::debug!(%sender, %dst_addr, %stake, "channel funded");
                } else {
                    let new_id = generate_channel_id(&sender, &dst_addr);
                    state.channels.insert(
                        hex::encode(new_id),
                        blokli_client::api::types::Channel {
                            balance: blokli_client::api::types::TokenValueString(stake.to_string()),
                            closure_time: None,
                            concrete_channel_id: hex::encode(new_id),
                            destination: destination.keyid,
                            epoch: 1,
                            source: source.keyid,
                            status: blokli_client::api::types::ChannelStatus::Open,
                            ticket_index: blokli_client::api::types::Uint64("0".into()),
                        },
                    );

                    tracing::debug!(%sender, %dst_addr, %stake, "channel opened");
                }
            }
            ParsedHoprChainAction::InitializeChannelClosure(channel_id) => {
                let grace_period = state
                    .chain_info
                    .channel_closure_grace_period
                    .clone()
                    .map(|p| p.0)
                    .unwrap_or("10".into());
                let grace_period = u64::from_str(&grace_period)
                    .map(|p| std::time::Duration::from_secs(p).max(std::time::Duration::from_secs(2)))
                    .map_err(|_| {
                        blokli_client::errors::ErrorKind::MockClientError(anyhow::anyhow!(
                            "failed to parse channel closure grace period"
                        ))
                    })?;

                let channel = state.get_channel_by_id_mut(&channel_id.into()).ok_or(
                    blokli_client::errors::ErrorKind::MockClientError(anyhow::anyhow!("missing channel {channel_id}")),
                )?;

                channel.status = blokli_client::api::types::ChannelStatus::PendingToClose;
                channel.closure_time = Some(blokli_client::api::types::DateTime(
                    hopr_api::chain::DateTime::from(std::time::SystemTime::now().add(grace_period)).to_rfc3339(),
                ));

                tracing::debug!(%channel_id, "channel closure initialized");
            }
            ParsedHoprChainAction::FinalizeChannelClosure(channel_id) => {
                let channel = state.get_channel_by_id_mut(&channel_id.into()).ok_or(
                    blokli_client::errors::ErrorKind::MockClientError(anyhow::anyhow!("missing channel {channel_id}")),
                )?;

                if channel.status != blokli_client::api::types::ChannelStatus::PendingToClose {
                    return Err(blokli_client::errors::ErrorKind::MockClientError(anyhow::anyhow!(
                        "channel {channel_id} is not pending to close"
                    ))
                    .into());
                }

                channel.status = blokli_client::api::types::ChannelStatus::Closed;

                tracing::debug!(%channel_id, "channel closure finalized");
            }
            ParsedHoprChainAction::IncomingChannelClosure(channel_id) => {
                let channel = state.get_channel_by_id_mut(&channel_id.into()).ok_or(
                    blokli_client::errors::ErrorKind::MockClientError(anyhow::anyhow!("missing channel {channel_id}")),
                )?;

                channel.status = blokli_client::api::types::ChannelStatus::Closed;

                tracing::debug!(%channel_id, "incoming channel closed");
            }
            ParsedHoprChainAction::RedeemTicket {
                channel_id,
                ticket_index,
                ticket_amount,
            } => {
                let channel = state.get_channel_by_id_mut(&channel_id.into()).ok_or(
                    blokli_client::errors::ErrorKind::MockClientError(anyhow::anyhow!("missing channel {channel_id}")),
                )?;

                if channel.status == blokli_client::api::types::ChannelStatus::Closed {
                    return Err(blokli_client::errors::ErrorKind::MockClientError(anyhow::anyhow!(
                        "channel {channel_id} is closed"
                    ))
                    .into());
                }

                let channel_ticket_index = u64::from_str(&channel.ticket_index.0).map_err(|_| {
                    blokli_client::errors::ErrorKind::MockClientError(anyhow::anyhow!(
                        "failed to parse ticket index of {channel_id}"
                    ))
                })?;

                if &channel_ticket_index > ticket_index {
                    return Err(blokli_client::errors::ErrorKind::MockClientError(anyhow::anyhow!(
                        "ticket index of {channel_id} ({channel_ticket_index}) is greater than redeemed ticket index \
                         {ticket_index}"
                    ))
                    .into());
                }

                let balance = channel.balance.0.parse::<HoprBalance>().map_err(|_| {
                    blokli_client::errors::ErrorKind::MockClientError(anyhow::anyhow!(
                        "failed to parse balance on channel {channel_id}"
                    ))
                })?;

                if &balance < ticket_amount {
                    return Err(blokli_client::errors::ErrorKind::MockClientError(anyhow::anyhow!(
                        "balance of channel {channel_id} ({balance}) is lower than ticket amount {ticket_amount}"
                    ))
                    .into());
                }

                channel.ticket_index = blokli_client::api::types::Uint64(ticket_index.to_string());
                channel.balance = blokli_client::api::types::TokenValueString((balance - *ticket_amount).to_string());

                let channel = channel.clone();
                if let Some(opposite_channel) = state
                    .channels
                    .values_mut()
                    .find(|c| c.source == channel.destination && c.destination == channel.source)
                {
                    let balance = opposite_channel.balance.0.parse::<HoprBalance>().map_err(|_| {
                        blokli_client::errors::ErrorKind::MockClientError(anyhow::anyhow!(
                            "failed to parse balance on opposite channel {channel_id}"
                        ))
                    })?;
                    opposite_channel.balance =
                        blokli_client::api::types::TokenValueString((balance + *ticket_amount).to_string());

                    tracing::debug!(%channel_id, %ticket_index, other_id = channel.concrete_channel_id, "ticket redeemed with channel rebalance");
                } else if let Some((safe_addr, safe_balance)) = state
                    .accounts
                    .get_mut(&(channel.destination as u32))
                    .and_then(|a| a.safe_address.clone())
                    .and_then(|safe_addr| state.token_balances.get_mut(&safe_addr).map(|b| (safe_addr, b)))
                {
                    let balance = safe_balance.balance.0.parse::<HoprBalance>().map_err(|_| {
                        blokli_client::errors::ErrorKind::MockClientError(anyhow::anyhow!(
                            "failed to parse balance on safe {safe_addr}"
                        ))
                    })?;
                    safe_balance.balance =
                        blokli_client::api::types::TokenValueString((balance + *ticket_amount).to_string());

                    tracing::debug!(%channel_id, %ticket_index, %safe_addr, "ticket redeemed into safe");
                } else {
                    tracing::debug!(%channel_id, %ticket_index, "ticket redeemed");
                }
            }
        }

        *state.tx_counts.entry(hex::encode(sender)).or_default() += 1;

        let balance = state.native_balances.get_mut(&hex::encode(sender)).ok_or(
            blokli_client::errors::ErrorKind::MockClientError(anyhow::anyhow!("missing native balance for {sender}")),
        )?;

        let balance_num = balance.balance.0.parse::<XDaiBalance>().map_err(|_| {
            blokli_client::errors::ErrorKind::MockClientError(anyhow::anyhow!(
                "failed to parse native balance for {sender}"
            ))
        })?;

        if balance_num.amount() < EMULATED_TX_PRICE.into() {
            return Err(blokli_client::errors::ErrorKind::MockClientError(anyhow::anyhow!(
                "insufficient native funds for tx"
            ))
            .into());
        }

        balance.balance = blokli_client::api::types::TokenValueString((balance_num - EMULATED_TX_PRICE).to_string());

        if let Some(sender) = &self.1 {
            sender.unbounded_send(action).map_err(|_| {
                blokli_client::errors::ErrorKind::MockClientError(anyhow::anyhow!(
                    "failed to send tx to tx interceptor"
                ))
            })?;
        }

        Ok(())
    }
}
