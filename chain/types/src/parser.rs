use alloy::consensus::Transaction;
use alloy::eips::Decodable2718;
use alloy::sol_types::SolCall;
use hopr_crypto_types::prelude::OffchainPublicKey;
use hopr_internal_types::prelude::{generate_channel_id, ChannelId};
use hopr_primitive_types::prelude::*;
use multiaddr::Multiaddr;
use hopr_bindings::hoprchannels::HoprChannels::{finalizeOutgoingChannelClosureCall, fundChannelSafeCall, initiateOutgoingChannelClosureCall, redeemTicketCall};
use hopr_bindings::hoprnodesaferegistry::HoprNodeSafeRegistry::registerSafeByNodeCall;
use hopr_bindings::hoprtoken::HoprToken::transferCall;

use crate::ContractAddresses;
use crate::errors::ChainTypesError;

/// Represents the action previously parsed from an EIP-2718 transaction.
///
/// This is effectively inverse of a [`PayloadGenerator`](crate::payload::PayloadGenerator).
pub enum ParsedHoprChainAction {
    RegisterSafeAddress(Address),
    Announce {
        packet_key: OffchainPublicKey,
        multiaddress: Option<Multiaddr>,
    },
    WithdrawNative(Address, XDaiBalance),
    WithdrawToken(Address, HoprBalance),
    FundChannel(Address, HoprBalance),
    InitializeChannelClosure(ChannelId),
    FinalizeChannelClosure(ChannelId),
    RedeemTicket {
        channel_id: ChannelId,
        ticket_index: u64,
        ticket_amount: HoprBalance,
    },
}

impl ParsedHoprChainAction {
    /// Attempts to parse a signed EIP-2718 transaction previously generated via a
    /// [`PayloadGenerator`](crate::payload::PayloadGenerator).
    pub fn parse_from_eip2718(signed_tx: &[u8], contract_addresses: &ContractAddresses) -> Result<(Self, Address), ChainTypesError> {
        let tx = alloy::consensus::TxEnvelope::decode_2718_exact(signed_tx.as_ref())
            .map_err(|e| ChainTypesError::ParseError(e.into()))?
            .into_signed();

        let signer = Address::from(tx.recover_signer().map_err(|e| ChainTypesError::ParseError(e.into()))?.0.0);

        let target_contract = tx
            .to()
            .map(|to| Address::from(to.0.0))
            .ok_or(ChainTypesError::ParseError(anyhow::anyhow!("transaction has no recipient")))?;

        if target_contract == contract_addresses.node_safe_registry {
            let register_call = registerSafeByNodeCall::abi_decode(tx.input().as_ref())
                .map_err(|e| ChainTypesError::ParseError(e.into()))?;

            Ok((Self::RegisterSafeAddress(register_call.safeAddr.0.0.into()), signer))
        } else if  target_contract == contract_addresses.announcements {
            todo!()
            /*Ok((Self::Announce {
                packet_key: (),
                multiaddress: None,
            }, signer))*/

        } else if target_contract == contract_addresses.channels {
            if let Ok(fund) = fundChannelSafeCall::abi_decode(tx.input().as_ref()) {
                Ok((Self::FundChannel(fund.account.0.0.into(), HoprBalance::from_be_bytes(fund.amount.to_be_bytes::<32>())), signer))
            } else {
                if let Ok(initiate) = initiateOutgoingChannelClosureCall::abi_decode(tx.input().as_ref()) {
                    Ok((Self::InitializeChannelClosure(generate_channel_id(&signer, &initiate.destination.0.0.into())), signer))
                } else {
                    if let Ok(finalize) = finalizeOutgoingChannelClosureCall::abi_decode(tx.input().as_ref()) {
                        Ok((Self::FinalizeChannelClosure(generate_channel_id(&signer, &finalize.destination.0.0.into())), signer))
                    } else {
                        if let Ok(redeem) = redeemTicketCall::abi_decode(tx.input().as_ref()) {
                            let ticket_data = redeem.redeemable.data;
                            Ok((
                                    Self::RedeemTicket {
                                        channel_id: ticket_data.channelId.0.into(),
                                        ticket_index: U256::from_be_bytes(&ticket_data.ticketIndex.to_be_bytes::<6>()).as_u64(),
                                        ticket_amount: HoprBalance::from_be_bytes(&ticket_data.amount.to_be_bytes::<12>())
                                    },
                                    signer
                                )
                            )
                        } else {
                            Err(ChainTypesError::ParseError(anyhow::anyhow!("channel transaction has invalid type")))?
                        }
                    }
                }
            }
        } else if target_contract == contract_addresses.token {
            let transfer = transferCall::abi_decode(tx.input().as_ref())
                .map_err(|e| ChainTypesError::ParseError(e.into()))?;

            Ok((Self::WithdrawToken(transfer.recipient.0.0.into(), HoprBalance::from_be_bytes(transfer.amount.to_be_bytes::<32>())), signer))
        } else if tx.value() > 0 {
            Ok((Self::WithdrawNative(signer, XDaiBalance::from_be_bytes(tx.value().to_be_bytes::<32>())), signer))
        } else {
            Err(ChainTypesError::ParseError(anyhow::anyhow!("transaction has invalid contract address")))?
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    
}