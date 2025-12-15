use hopr_bindings::{
    exports::alloy::{
        consensus::Transaction,
        eips::Decodable2718,
        sol_types::{SolCall, SolType},
    },
    hopr_channels::HoprChannels::{
        closeIncomingChannelCall, closeIncomingChannelSafeCall, finalizeOutgoingChannelClosureCall,
        finalizeOutgoingChannelClosureSafeCall, fundChannelCall, fundChannelSafeCall,
        initiateOutgoingChannelClosureCall, initiateOutgoingChannelClosureSafeCall, redeemTicketCall,
        redeemTicketSafeCall,
    },
    hopr_node_management_module::HoprNodeManagementModule::execTransactionFromModuleCall,
    hopr_node_safe_registry::HoprNodeSafeRegistry::registerSafeByNodeCall,
    hopr_token::HoprToken::{sendCall, transferCall},
};
use hopr_crypto_types::prelude::OffchainPublicKey;
use hopr_internal_types::prelude::{ChannelId, generate_channel_id};
use hopr_primitive_types::prelude::*;
use multiaddr::Multiaddr;

use crate::{ContractAddresses, a2al, errors::ChainTypesError, payload::KeyBindAndAnnouncePayload};

/// Represents the action previously parsed from an EIP-2718 transaction.
///
/// This is effectively inverse of a [`PayloadGenerator`](crate::payload::PayloadGenerator).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParsedHoprChainAction {
    /// Registration of a Safe address.
    RegisterSafeAddress(Address),
    /// Announcement of a packet key and optional multiaddress.
    Announce {
        /// Announced packet key (key-binding).
        packet_key: OffchainPublicKey,
        /// Optional multiaddress to announce.
        multiaddress: Option<Multiaddr>,
    },
    /// Withdrawal of native XDai to an address.
    WithdrawNative(Address, XDaiBalance),
    /// Withdrawal of HOPR token to an address.
    WithdrawToken(Address, HoprBalance),
    /// Funding of a payment channel to a given destination with a given amount.
    FundChannel(Address, HoprBalance),
    /// Payment channel closure initiation with the given ID.
    InitializeChannelClosure(ChannelId),
    /// Payment channel closure finalization with the given ID.
    FinalizeChannelClosure(ChannelId),
    /// Incoming payment channel closure with the given ID.
    IncomingChannelClosure(ChannelId),
    /// Redemption of ticket.
    RedeemTicket {
        /// ID of the channel the ticket was issued on.
        channel_id: ChannelId,
        /// Index of the ticket within the channel.
        ticket_index: u64,
        /// Amount HOPR tokens on the ticket (value to be redeemed).
        ticket_amount: HoprBalance,
    },
}

impl ParsedHoprChainAction {
    /// Attempts to parse a signed EIP-2718 transaction previously generated via a
    /// [`PayloadGenerator`](crate::payload::PayloadGenerator).
    pub fn parse_from_eip2718(
        signed_tx: &[u8],
        module: &Address,
        contract_addresses: &ContractAddresses,
    ) -> Result<(Self, Address), ChainTypesError> {
        let tx = hopr_bindings::exports::alloy::consensus::TxEnvelope::decode_2718_exact(signed_tx)
            .map_err(|e| ChainTypesError::ParseError(e.into()))?
            .into_signed();

        let signer = Address::from(
            tx.recover_signer()
                .map_err(|e| ChainTypesError::ParseError(e.into()))?
                .0
                .0,
        );

        let tx_target = tx
            .to()
            .map(|to| Address::from(to.0.0))
            .ok_or(ChainTypesError::ParseError(anyhow::anyhow!(
                "transaction has no recipient"
            )))?;

        let (target_contract, input, module_call) = if &tx_target == module {
            let module_call = execTransactionFromModuleCall::abi_decode(tx.input().as_ref())
                .map_err(|e| ChainTypesError::ParseError(e.into()))?;
            (module_call.to.0.0.into(), module_call.data, true)
        } else if contract_addresses.into_iter().any(|addr| addr == a2al(tx_target)) {
            (tx_target, tx.input().clone(), false)
        } else if tx.value() > 0 {
            return Ok((
                Self::WithdrawNative(tx_target, XDaiBalance::from_be_bytes(tx.value().to_be_bytes::<32>())),
                signer,
            ));
        } else {
            return Err(ChainTypesError::ParseError(anyhow::anyhow!(
                "failed to determine type of transaction"
            )));
        };

        let target_contract = a2al(target_contract);

        if target_contract == contract_addresses.node_safe_registry {
            let register_call = registerSafeByNodeCall::abi_decode(input.as_ref())
                .map_err(|e| ChainTypesError::ParseError(e.into()))?;

            Ok((Self::RegisterSafeAddress(register_call.safeAddr.0.0.into()), signer))
        } else if target_contract == contract_addresses.channels && module_call {
            if let Ok(fund) = fundChannelSafeCall::abi_decode(input.as_ref()) {
                return Ok((
                    Self::FundChannel(
                        fund.account.0.0.into(),
                        HoprBalance::from_be_bytes(fund.amount.to_be_bytes::<12>()),
                    ),
                    signer,
                ));
            }

            if let Ok(initiate) = initiateOutgoingChannelClosureSafeCall::abi_decode(input.as_ref()) {
                return Ok((
                    Self::InitializeChannelClosure(generate_channel_id(&signer, &initiate.destination.0.0.into())),
                    signer,
                ));
            }

            if let Ok(finalize) = finalizeOutgoingChannelClosureSafeCall::abi_decode(input.as_ref()) {
                return Ok((
                    Self::FinalizeChannelClosure(generate_channel_id(&signer, &finalize.destination.0.0.into())),
                    signer,
                ));
            }

            if let Ok(close_incoming) = closeIncomingChannelSafeCall::abi_decode(input.as_ref()) {
                return Ok((
                    Self::IncomingChannelClosure(generate_channel_id(&close_incoming.source.0.0.into(), &signer)),
                    signer,
                ));
            }

            if let Ok(redeem) = redeemTicketSafeCall::abi_decode(input.as_ref()) {
                let ticket_data = redeem.redeemable.data;
                return Ok((
                    Self::RedeemTicket {
                        channel_id: ticket_data.channelId.0.into(),
                        ticket_index: U256::from_be_bytes(ticket_data.ticketIndex.to_be_bytes::<6>()).as_u64(),
                        ticket_amount: HoprBalance::from_be_bytes(ticket_data.amount.to_be_bytes::<12>()),
                    },
                    signer,
                ));
            }

            Err(ChainTypesError::ParseError(anyhow::anyhow!(
                "channel transaction has invalid type"
            )))?
        } else if target_contract == contract_addresses.channels && !module_call {
            if let Ok(fund) = fundChannelCall::abi_decode(input.as_ref()) {
                return Ok((
                    Self::FundChannel(
                        fund.account.0.0.into(),
                        HoprBalance::from_be_bytes(fund.amount.to_be_bytes::<12>()),
                    ),
                    signer,
                ));
            }

            if let Ok(initiate) = initiateOutgoingChannelClosureCall::abi_decode(input.as_ref()) {
                return Ok((
                    Self::InitializeChannelClosure(generate_channel_id(&signer, &initiate.destination.0.0.into())),
                    signer,
                ));
            }

            if let Ok(finalize) = finalizeOutgoingChannelClosureCall::abi_decode(input.as_ref()) {
                return Ok((
                    Self::FinalizeChannelClosure(generate_channel_id(&signer, &finalize.destination.0.0.into())),
                    signer,
                ));
            }

            if let Ok(close_incoming) = closeIncomingChannelCall::abi_decode(input.as_ref()) {
                return Ok((
                    Self::IncomingChannelClosure(generate_channel_id(&close_incoming.source.0.0.into(), &signer)),
                    signer,
                ));
            }

            if let Ok(redeem) = redeemTicketCall::abi_decode(input.as_ref()) {
                let ticket_data = redeem.redeemable.data;
                return Ok((
                    Self::RedeemTicket {
                        channel_id: ticket_data.channelId.0.into(),
                        ticket_index: U256::from_be_bytes(ticket_data.ticketIndex.to_be_bytes::<6>()).as_u64(),
                        ticket_amount: HoprBalance::from_be_bytes(ticket_data.amount.to_be_bytes::<12>()),
                    },
                    signer,
                ));
            }
            Err(ChainTypesError::ParseError(anyhow::anyhow!(
                "channel transaction has invalid type"
            )))?
        } else if target_contract == contract_addresses.token {
            if let Ok(send) = sendCall::abi_decode(input.as_ref()) {
                if send.recipient == contract_addresses.announcements {
                    let mut data = vec![0u8; 32 + send.data.len()];
                    data[31] = 32;
                    data[32..].copy_from_slice(&send.data);

                    let kb = KeyBindAndAnnouncePayload::abi_decode(&data)
                        .map_err(|e| ChainTypesError::ParseError(e.into()))?;

                    return Ok((
                        Self::Announce {
                            packet_key: kb.ed25519_pub_key.0.try_into().map_err(|_| {
                                ChainTypesError::ParseError(anyhow::anyhow!("failed to parse packet key"))
                            })?,
                            multiaddress: if kb.multiaddress.is_empty() {
                                None
                            } else {
                                Some(kb.multiaddress.parse().map_err(|_| {
                                    ChainTypesError::ParseError(anyhow::anyhow!("failed to parse multiaddress"))
                                })?)
                            },
                        },
                        signer,
                    ));
                } else {
                    Err(ChainTypesError::ParseError(anyhow::anyhow!(
                        "token send transaction transaction has invalid type"
                    )))?
                }
            }

            let transfer =
                transferCall::abi_decode(input.as_ref()).map_err(|e| ChainTypesError::ParseError(e.into()))?;

            Ok((
                Self::WithdrawToken(
                    transfer.recipient.0.0.into(),
                    HoprBalance::from_be_bytes(transfer.amount.to_be_bytes::<32>()),
                ),
                signer,
            ))
        } else {
            Err(ChainTypesError::ParseError(anyhow::anyhow!(
                "transaction has invalid contract address"
            )))?
        }
    }
}

#[cfg(test)]
mod tests {
    use hex_literal::hex;
    use hopr_crypto_types::{
        crypto_traits::Randomizable,
        prelude::{ChainKeypair, HalfKey, Hash, Keypair, OffchainKeypair, Response},
    };
    use hopr_internal_types::prelude::{AnnouncementData, KeyBinding, TicketBuilder};

    use super::*;
    use crate::payload::{
        BasicPayloadGenerator, PayloadGenerator, SafePayloadGenerator, SignableTransaction, tests::CONTRACT_ADDRS,
    };

    const PRIVATE_KEY_1: [u8; 32] = hex!("c14b8faa0a9b8a5fa4453664996f23a7e7de606d42297d723fc4a794f375e260");
    const PRIVATE_KEY_2: [u8; 32] = hex!("492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775");

    #[tokio::test]
    async fn announce_safe_action_should_decode() -> anyhow::Result<()> {
        let cp = ChainKeypair::from_secret(&PRIVATE_KEY_1)?;
        let ocp = OffchainKeypair::random();

        let ad = AnnouncementData::new(
            KeyBinding::new(cp.public().to_address(), &ocp),
            Some("/ip4/127.0.0.1/tcp/10000".parse()?),
        )?;

        let safe_gen = SafePayloadGenerator::new(&cp, *CONTRACT_ADDRS, [1u8; Address::SIZE].into());
        let signed_tx = safe_gen
            .announce(ad, 10_u32.into())?
            .sign_and_encode_to_eip2718(1, 1, None, &cp)
            .await?;

        let (action, signer) =
            ParsedHoprChainAction::parse_from_eip2718(&signed_tx, &[1u8; Address::SIZE].into(), &CONTRACT_ADDRS)?;
        assert_eq!(
            action,
            ParsedHoprChainAction::Announce {
                packet_key: *ocp.public(),
                multiaddress: Some("/ip4/127.0.0.1/tcp/10000".parse()?),
            }
        );
        assert_eq!(signer, cp.public().to_address());

        Ok(())
    }

    #[tokio::test]
    async fn fund_channel_action_should_decode() -> anyhow::Result<()> {
        let cp = ChainKeypair::from_secret(&PRIVATE_KEY_1)?;

        let basic_gen = BasicPayloadGenerator::new(cp.public().to_address(), *CONTRACT_ADDRS);
        let signed_tx = basic_gen
            .fund_channel([2u8; Address::SIZE].into(), 123_u32.into())?
            .sign_and_encode_to_eip2718(1, 1, None, &cp)
            .await?;

        let (action, signer) =
            ParsedHoprChainAction::parse_from_eip2718(&signed_tx, &[1u8; Address::SIZE].into(), &CONTRACT_ADDRS)?;
        assert_eq!(
            action,
            ParsedHoprChainAction::FundChannel([2u8; Address::SIZE].into(), 123_u32.into())
        );
        assert_eq!(signer, cp.public().to_address());

        let safe_gen = SafePayloadGenerator::new(&cp, *CONTRACT_ADDRS, [1u8; Address::SIZE].into());
        let signed_tx = safe_gen
            .fund_channel([2u8; Address::SIZE].into(), 123_u32.into())?
            .sign_and_encode_to_eip2718(1, 1, None, &cp)
            .await?;

        let (action, signer) =
            ParsedHoprChainAction::parse_from_eip2718(&signed_tx, &[1u8; Address::SIZE].into(), &CONTRACT_ADDRS)?;
        assert_eq!(
            action,
            ParsedHoprChainAction::FundChannel([2u8; Address::SIZE].into(), 123_u32.into())
        );
        assert_eq!(signer, cp.public().to_address());

        Ok(())
    }

    #[tokio::test]
    async fn initiate_channel_closure_action_should_decode() -> anyhow::Result<()> {
        let cp = ChainKeypair::from_secret(&PRIVATE_KEY_1)?;

        let basic_gen = BasicPayloadGenerator::new(cp.public().to_address(), *CONTRACT_ADDRS);
        let signed_tx = basic_gen
            .initiate_outgoing_channel_closure([2u8; Address::SIZE].into())?
            .sign_and_encode_to_eip2718(1, 1, None, &cp)
            .await?;

        let channel_id = generate_channel_id(&cp.public().to_address(), &[2u8; Address::SIZE].into());

        let (action, signer) =
            ParsedHoprChainAction::parse_from_eip2718(&signed_tx, &[1u8; Address::SIZE].into(), &CONTRACT_ADDRS)?;
        assert_eq!(action, ParsedHoprChainAction::InitializeChannelClosure(channel_id));
        assert_eq!(signer, cp.public().to_address());

        let safe_gen = SafePayloadGenerator::new(&cp, *CONTRACT_ADDRS, [1u8; Address::SIZE].into());
        let signed_tx = safe_gen
            .initiate_outgoing_channel_closure([2u8; Address::SIZE].into())?
            .sign_and_encode_to_eip2718(1, 1, None, &cp)
            .await?;

        let (action, signer) =
            ParsedHoprChainAction::parse_from_eip2718(&signed_tx, &[1u8; Address::SIZE].into(), &CONTRACT_ADDRS)?;
        assert_eq!(action, ParsedHoprChainAction::InitializeChannelClosure(channel_id));
        assert_eq!(signer, cp.public().to_address());

        Ok(())
    }

    #[tokio::test]
    async fn finalize_channel_closure_action_should_decode() -> anyhow::Result<()> {
        let cp = ChainKeypair::from_secret(&PRIVATE_KEY_1)?;

        let basic_gen = BasicPayloadGenerator::new(cp.public().to_address(), *CONTRACT_ADDRS);
        let signed_tx = basic_gen
            .finalize_outgoing_channel_closure([2u8; Address::SIZE].into())?
            .sign_and_encode_to_eip2718(1, 1, None, &cp)
            .await?;

        let channel_id = generate_channel_id(&cp.public().to_address(), &[2u8; Address::SIZE].into());

        let (action, signer) =
            ParsedHoprChainAction::parse_from_eip2718(&signed_tx, &[1u8; Address::SIZE].into(), &CONTRACT_ADDRS)?;
        assert_eq!(action, ParsedHoprChainAction::FinalizeChannelClosure(channel_id));
        assert_eq!(signer, cp.public().to_address());

        let safe_gen = SafePayloadGenerator::new(&cp, *CONTRACT_ADDRS, [1u8; Address::SIZE].into());
        let signed_tx = safe_gen
            .finalize_outgoing_channel_closure([2u8; Address::SIZE].into())?
            .sign_and_encode_to_eip2718(1, 1, None, &cp)
            .await?;

        let (action, signer) =
            ParsedHoprChainAction::parse_from_eip2718(&signed_tx, &[1u8; Address::SIZE].into(), &CONTRACT_ADDRS)?;
        assert_eq!(action, ParsedHoprChainAction::FinalizeChannelClosure(channel_id));
        assert_eq!(signer, cp.public().to_address());

        Ok(())
    }

    #[tokio::test]
    async fn incoming_channel_closure_action_should_decode() -> anyhow::Result<()> {
        let cp = ChainKeypair::from_secret(&PRIVATE_KEY_1)?;

        let basic_gen = BasicPayloadGenerator::new(cp.public().to_address(), *CONTRACT_ADDRS);
        let signed_tx = basic_gen
            .close_incoming_channel([2u8; Address::SIZE].into())?
            .sign_and_encode_to_eip2718(1, 1, None, &cp)
            .await?;

        let channel_id = generate_channel_id(&[2u8; Address::SIZE].into(), &cp.public().to_address());

        let (action, signer) =
            ParsedHoprChainAction::parse_from_eip2718(&signed_tx, &[1u8; Address::SIZE].into(), &CONTRACT_ADDRS)?;
        assert_eq!(action, ParsedHoprChainAction::IncomingChannelClosure(channel_id));
        assert_eq!(signer, cp.public().to_address());

        let safe_gen = SafePayloadGenerator::new(&cp, *CONTRACT_ADDRS, [1u8; Address::SIZE].into());
        let signed_tx = safe_gen
            .close_incoming_channel([2u8; Address::SIZE].into())?
            .sign_and_encode_to_eip2718(1, 1, None, &cp)
            .await?;

        let (action, signer) =
            ParsedHoprChainAction::parse_from_eip2718(&signed_tx, &[1u8; Address::SIZE].into(), &CONTRACT_ADDRS)?;
        assert_eq!(action, ParsedHoprChainAction::IncomingChannelClosure(channel_id));
        assert_eq!(signer, cp.public().to_address());

        Ok(())
    }

    #[tokio::test]
    async fn register_safe_action_should_decode() -> anyhow::Result<()> {
        let cp = ChainKeypair::from_secret(&PRIVATE_KEY_1)?;

        let basic_gen = BasicPayloadGenerator::new(cp.public().to_address(), *CONTRACT_ADDRS);
        let signed_tx = basic_gen
            .register_safe_by_node([2u8; Address::SIZE].into())?
            .sign_and_encode_to_eip2718(1, 1, None, &cp)
            .await?;

        let (action, signer) =
            ParsedHoprChainAction::parse_from_eip2718(&signed_tx, &[1u8; Address::SIZE].into(), &CONTRACT_ADDRS)?;
        assert_eq!(
            action,
            ParsedHoprChainAction::RegisterSafeAddress([2u8; Address::SIZE].into())
        );
        assert_eq!(signer, cp.public().to_address());

        let safe_gen = SafePayloadGenerator::new(&cp, *CONTRACT_ADDRS, [1u8; Address::SIZE].into());
        let signed_tx = safe_gen
            .register_safe_by_node([2u8; Address::SIZE].into())?
            .sign_and_encode_to_eip2718(1, 1, None, &cp)
            .await?;

        let (action, signer) =
            ParsedHoprChainAction::parse_from_eip2718(&signed_tx, &[1u8; Address::SIZE].into(), &CONTRACT_ADDRS)?;
        assert_eq!(
            action,
            ParsedHoprChainAction::RegisterSafeAddress([2u8; Address::SIZE].into())
        );
        assert_eq!(signer, cp.public().to_address());

        Ok(())
    }

    #[tokio::test]
    async fn redeem_ticket_safe_action_should_decode() -> anyhow::Result<()> {
        let cp_1 = ChainKeypair::from_secret(&PRIVATE_KEY_1)?;
        let cp_2 = ChainKeypair::from_secret(&PRIVATE_KEY_2)?;
        let hk1 = HalfKey::random();
        let hk2 = HalfKey::random();
        let resp = Response::from_half_keys(&hk1, &hk2)?;

        let ticket = TicketBuilder::default()
            .counterparty(&cp_2)
            .amount(123_u32)
            .index(7)
            .challenge(resp.to_challenge()?)
            .build_signed(&cp_1, &Hash::default())?
            .into_acknowledged(resp)
            .into_redeemable(&cp_2, &Hash::default())?;

        let basic_gen = BasicPayloadGenerator::new(cp_2.public().to_address(), *CONTRACT_ADDRS);
        let signed_tx = basic_gen
            .redeem_ticket(ticket.clone())?
            .sign_and_encode_to_eip2718(1, 1, None, &cp_2)
            .await?;

        let (action, signer) =
            ParsedHoprChainAction::parse_from_eip2718(&signed_tx, &[1u8; Address::SIZE].into(), &CONTRACT_ADDRS)?;
        assert_eq!(
            action,
            ParsedHoprChainAction::RedeemTicket {
                channel_id: generate_channel_id(&cp_1.public().to_address(), &cp_2.public().to_address()),
                ticket_index: 7,
                ticket_amount: 123_u32.into()
            }
        );
        assert_eq!(signer, cp_2.public().to_address());

        let safe_gen = SafePayloadGenerator::new(&cp_2, *CONTRACT_ADDRS, [1u8; Address::SIZE].into());
        let signed_tx = safe_gen
            .redeem_ticket(ticket.clone())?
            .sign_and_encode_to_eip2718(1, 1, None, &cp_2)
            .await?;

        let (action, signer) =
            ParsedHoprChainAction::parse_from_eip2718(&signed_tx, &[1u8; Address::SIZE].into(), &CONTRACT_ADDRS)?;
        assert_eq!(
            action,
            ParsedHoprChainAction::RedeemTicket {
                channel_id: generate_channel_id(&cp_1.public().to_address(), &cp_2.public().to_address()),
                ticket_index: 7,
                ticket_amount: 123_u32.into()
            }
        );
        assert_eq!(signer, cp_2.public().to_address());

        Ok(())
    }

    #[tokio::test]
    async fn withdraw_native_action_should_decode() -> anyhow::Result<()> {
        let cp = ChainKeypair::from_secret(&PRIVATE_KEY_1)?;

        let basic_gen = BasicPayloadGenerator::new(cp.public().to_address(), *CONTRACT_ADDRS);
        let signed_tx = basic_gen
            .transfer::<XDai>([2u8; Address::SIZE].into(), 123_u32.into())?
            .sign_and_encode_to_eip2718(1, 1, None, &cp)
            .await?;

        let (action, signer) =
            ParsedHoprChainAction::parse_from_eip2718(&signed_tx, &[1u8; Address::SIZE].into(), &CONTRACT_ADDRS)?;
        assert_eq!(
            action,
            ParsedHoprChainAction::WithdrawNative([2u8; Address::SIZE].into(), 123_u32.into())
        );
        assert_eq!(signer, cp.public().to_address());

        let safe_gen = SafePayloadGenerator::new(&cp, *CONTRACT_ADDRS, [1u8; Address::SIZE].into());
        let signed_tx = safe_gen
            .transfer::<XDai>([2u8; Address::SIZE].into(), 123_u32.into())?
            .sign_and_encode_to_eip2718(1, 1, None, &cp)
            .await?;

        let (action, signer) =
            ParsedHoprChainAction::parse_from_eip2718(&signed_tx, &[1u8; Address::SIZE].into(), &CONTRACT_ADDRS)?;
        assert_eq!(
            action,
            ParsedHoprChainAction::WithdrawNative([2u8; Address::SIZE].into(), 123_u32.into())
        );
        assert_eq!(signer, cp.public().to_address());

        Ok(())
    }

    #[tokio::test]
    async fn withdraw_token_safe_action_should_decode() -> anyhow::Result<()> {
        let cp = ChainKeypair::from_secret(&PRIVATE_KEY_1)?;

        let basic_gen = BasicPayloadGenerator::new(cp.public().to_address(), *CONTRACT_ADDRS);
        let signed_tx = basic_gen
            .transfer::<WxHOPR>([2u8; Address::SIZE].into(), 123_u32.into())?
            .sign_and_encode_to_eip2718(1, 1, None, &cp)
            .await?;

        let (action, signer) =
            ParsedHoprChainAction::parse_from_eip2718(&signed_tx, &[1u8; Address::SIZE].into(), &CONTRACT_ADDRS)?;
        assert_eq!(
            action,
            ParsedHoprChainAction::WithdrawToken([2u8; Address::SIZE].into(), 123_u32.into())
        );
        assert_eq!(signer, cp.public().to_address());

        let safe_gen = SafePayloadGenerator::new(&cp, *CONTRACT_ADDRS, [1u8; Address::SIZE].into());
        let signed_tx = safe_gen
            .transfer::<WxHOPR>([2u8; Address::SIZE].into(), 123_u32.into())?
            .sign_and_encode_to_eip2718(1, 1, None, &cp)
            .await?;

        let (action, signer) =
            ParsedHoprChainAction::parse_from_eip2718(&signed_tx, &[1u8; Address::SIZE].into(), &CONTRACT_ADDRS)?;
        assert_eq!(
            action,
            ParsedHoprChainAction::WithdrawToken([2u8; Address::SIZE].into(), 123_u32.into())
        );
        assert_eq!(signer, cp.public().to_address());

        Ok(())
    }
}
