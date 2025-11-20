pub use hopr_bindings::exports::alloy::rpc::types::TransactionRequest;
use hopr_bindings::{
    exports::alloy::{
        self,
        consensus::TxEnvelope,
        eips::Encodable2718,
        network::{EthereumWallet, TransactionBuilder},
        primitives::{
            B256, U256,
            aliases::{U24, U48, U56, U96},
        },
        signers::local::PrivateKeySigner,
        sol,
        sol_types::{SolCall, SolValue},
    },
    hopr_channels::{
        HoprChannels::{
            RedeemableTicket as OnChainRedeemableTicket, TicketData, closeIncomingChannelCall,
            closeIncomingChannelSafeCall, finalizeOutgoingChannelClosureCall, finalizeOutgoingChannelClosureSafeCall,
            fundChannelCall, fundChannelSafeCall, initiateOutgoingChannelClosureCall,
            initiateOutgoingChannelClosureSafeCall, redeemTicketCall, redeemTicketSafeCall,
        },
        HoprCrypto::{CompactSignature, VRFParameters},
    },
    hopr_node_management_module::HoprNodeManagementModule::execTransactionFromModuleCall,
    hopr_node_safe_registry::HoprNodeSafeRegistry::{deregisterNodeBySafeCall, registerSafeByNodeCall},
    hopr_token::HoprToken::{approveCall, sendCall, transferCall},
};
use hopr_crypto_types::prelude::*;
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::*;

use crate::{
    ContractAddresses,
    errors::ChainTypesError::{InvalidArguments, InvalidState, SigningError},
    payload,
    payload::{GasEstimation, PayloadGenerator, SignableTransaction},
};

const DEFAULT_TX_GAS: u64 = 400_000;

// Used instead of From implementation to avoid alloy being a dependency of the primitive crates
fn a2h(a: hopr_primitive_types::prelude::Address) -> alloy::primitives::Address {
    alloy::primitives::Address::from_slice(a.as_ref())
}

sol! {
    /// Mirrors the Solidity layout: (address, bytes32, bytes32, bytes32, string)
    struct KeyBindAndAnnouncePayload {
        address callerNode;
        bytes32 ed25519_sig_0;
        bytes32 ed25519_sig_1;
        bytes32 ed25519_pub_key;
        string  multiaddress;
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Operation {
    Call = 0,
    // Future use: DelegateCall = 1,
}

#[async_trait::async_trait]
impl SignableTransaction for TransactionRequest {
    async fn sign_and_encode_to_eip2718(
        self,
        nonce: u64,
        max_gas: Option<GasEstimation>,
        chain_keypair: &ChainKeypair,
    ) -> payload::Result<Box<[u8]>> {
        let max_gas = max_gas.unwrap_or_default();
        let signer: EthereumWallet = PrivateKeySigner::from_slice(chain_keypair.secret().as_ref())
            .map_err(|e| SigningError(e.into()))?
            .into();
        let signed: TxEnvelope = self
            .nonce(nonce)
            .gas_limit(max_gas.gas_limit)
            .max_fee_per_gas(max_gas.max_fee_per_gas)
            .max_priority_fee_per_gas(max_gas.max_priority_fee_per_gas)
            .build(&signer)
            .await
            .map_err(|e| SigningError(e.into()))?;

        Ok(signed.encoded_2718().into_boxed_slice())
    }
}

fn channels_payload(hopr_channels: Address, call_data: Vec<u8>) -> Vec<u8> {
    execTransactionFromModuleCall {
        to: a2h(hopr_channels),
        value: U256::ZERO,
        data: call_data.into(),
        operation: Operation::Call as u8,
    }
    .abi_encode()
}

fn approve_tx(spender: Address, amount: HoprBalance) -> TransactionRequest {
    TransactionRequest::default().with_input(
        approveCall {
            spender: a2h(spender),
            value: U256::from_be_bytes(amount.amount().to_be_bytes()),
        }
        .abi_encode(),
    )
}

fn transfer_tx<C: Currency>(destination: Address, amount: Balance<C>) -> TransactionRequest {
    let amount_u256 = U256::from_be_bytes(amount.amount().to_be_bytes());
    let tx = TransactionRequest::default();
    if WxHOPR::is::<C>() {
        tx.with_input(
            transferCall {
                recipient: a2h(destination),
                amount: amount_u256,
            }
            .abi_encode(),
        )
    } else if XDai::is::<C>() {
        tx.with_value(amount_u256)
    } else {
        unimplemented!("other currencies are currently not supported")
    }
}

fn register_safe_tx(safe_addr: Address) -> TransactionRequest {
    TransactionRequest::default().with_input(
        registerSafeByNodeCall {
            safeAddr: a2h(safe_addr),
        }
        .abi_encode(),
    )
}

/// Generates transaction payloads that do not use Safe-compliant ABI
#[derive(Debug, Clone, Copy)]
pub struct BasicPayloadGenerator {
    me: Address,
    contract_addrs: ContractAddresses,
}

impl BasicPayloadGenerator {
    pub fn new(me: Address, contract_addrs: ContractAddresses) -> Self {
        Self { me, contract_addrs }
    }
}

impl PayloadGenerator for BasicPayloadGenerator {
    type TxRequest = TransactionRequest;

    fn approve(&self, spender: Address, amount: HoprBalance) -> payload::Result<Self::TxRequest> {
        let tx = approve_tx(spender, amount).with_to(a2h(self.contract_addrs.token));
        Ok(tx)
    }

    fn transfer<C: Currency>(&self, destination: Address, amount: Balance<C>) -> payload::Result<Self::TxRequest> {
        let to = if XDai::is::<C>() {
            destination
        } else if WxHOPR::is::<C>() {
            self.contract_addrs.token
        } else {
            return Err(InvalidArguments("invalid currency"));
        };
        let tx = transfer_tx(destination, amount).with_to(a2h(to));
        Ok(tx)
    }

    fn announce(
        &self,
        announcement: AnnouncementData,
        key_binding_fee: HoprBalance,
    ) -> payload::Result<Self::TxRequest> {
        // when the keys have already bounded, now only try to announce without key binding
        // when keys are not bounded yet, bind keys and announce together
        let serialized_signature = announcement.key_binding().signature.as_ref();

        let inner_payload = KeyBindAndAnnouncePayload {
            callerNode: a2h(self.me),
            ed25519_sig_0: B256::from_slice(&serialized_signature[0..32]),
            ed25519_sig_1: B256::from_slice(&serialized_signature[32..64]),
            ed25519_pub_key: B256::from_slice(announcement.key_binding().packet_key.as_ref()),
            multiaddress: announcement
                .multiaddress()
                .as_ref()
                .map(ToString::to_string)
                .unwrap_or_default(), // "" if None
        }
        .abi_encode();

        let call_data = sendCall {
            recipient: a2h(self.contract_addrs.announcements),
            amount: alloy::primitives::U256::from_be_slice(&key_binding_fee.amount().to_be_bytes()),
            data: inner_payload[32..].to_vec().into(),
        }
        .abi_encode();

        Ok(TransactionRequest::default()
            .with_input(call_data)
            .with_to(a2h(self.contract_addrs.token)))
    }

    fn fund_channel(&self, dest: Address, amount: HoprBalance) -> payload::Result<Self::TxRequest> {
        if dest.eq(&self.me) {
            return Err(InvalidArguments("Cannot fund channel to self"));
        }

        let tx = TransactionRequest::default()
            .with_input(
                fundChannelCall {
                    account: a2h(dest),
                    amount: U96::from_be_slice(&amount.amount().to_be_bytes()[32 - 12..]),
                }
                .abi_encode(),
            )
            .with_to(a2h(self.contract_addrs.channels));
        Ok(tx)
    }

    fn close_incoming_channel(&self, source: Address) -> payload::Result<Self::TxRequest> {
        if source.eq(&self.me) {
            return Err(InvalidArguments("Cannot close incoming channel from self"));
        }

        let tx = TransactionRequest::default()
            .with_input(closeIncomingChannelCall { source: a2h(source) }.abi_encode())
            .with_to(a2h(self.contract_addrs.channels));
        Ok(tx)
    }

    fn initiate_outgoing_channel_closure(&self, destination: Address) -> payload::Result<Self::TxRequest> {
        if destination.eq(&self.me) {
            return Err(InvalidArguments("Cannot initiate closure of incoming channel to self"));
        }

        let tx = TransactionRequest::default()
            .with_input(
                initiateOutgoingChannelClosureCall {
                    destination: a2h(destination),
                }
                .abi_encode(),
            )
            .with_to(a2h(self.contract_addrs.channels));
        Ok(tx)
    }

    fn finalize_outgoing_channel_closure(&self, destination: Address) -> payload::Result<Self::TxRequest> {
        if destination.eq(&self.me) {
            return Err(InvalidArguments("Cannot initiate closure of incoming channel to self"));
        }

        let tx = TransactionRequest::default()
            .with_input(
                finalizeOutgoingChannelClosureCall {
                    destination: a2h(destination),
                }
                .abi_encode(),
            )
            .with_to(a2h(self.contract_addrs.channels));
        Ok(tx)
    }

    fn redeem_ticket(&self, acked_ticket: RedeemableTicket) -> payload::Result<Self::TxRequest> {
        let redeemable = convert_acknowledged_ticket(&acked_ticket)?;

        let params = convert_vrf_parameters(
            &acked_ticket.vrf_params,
            &self.me,
            acked_ticket.ticket.verified_hash(),
            &acked_ticket.channel_dst,
        );

        let tx = TransactionRequest::default()
            .with_input(redeemTicketCall { redeemable, params }.abi_encode())
            .with_to(a2h(self.contract_addrs.channels));
        Ok(tx)
    }

    fn register_safe_by_node(&self, safe_addr: Address) -> payload::Result<Self::TxRequest> {
        let tx = register_safe_tx(safe_addr).with_to(a2h(self.contract_addrs.node_safe_registry));
        Ok(tx)
    }

    fn deregister_node_by_safe(&self) -> payload::Result<Self::TxRequest> {
        Err(InvalidState("Can only deregister an address if Safe is activated"))
    }
}

/// Payload generator that generates Safe-compliant ABI
#[derive(Debug, Clone, Copy)]
pub struct SafePayloadGenerator {
    me: Address,
    contract_addrs: ContractAddresses,
    module: Address,
}

impl SafePayloadGenerator {
    pub fn new(chain_keypair: &ChainKeypair, contract_addrs: ContractAddresses, module: Address) -> Self {
        Self {
            me: chain_keypair.into(),
            contract_addrs,
            module,
        }
    }
}

impl PayloadGenerator for SafePayloadGenerator {
    type TxRequest = TransactionRequest;

    fn approve(&self, spender: Address, amount: HoprBalance) -> payload::Result<Self::TxRequest> {
        let tx = approve_tx(spender, amount)
            .with_to(a2h(self.contract_addrs.token))
            .with_gas_limit(DEFAULT_TX_GAS);

        Ok(tx)
    }

    fn transfer<C: Currency>(&self, destination: Address, amount: Balance<C>) -> payload::Result<Self::TxRequest> {
        let to = if XDai::is::<C>() {
            destination
        } else if WxHOPR::is::<C>() {
            self.contract_addrs.token
        } else {
            return Err(InvalidArguments("invalid currency"));
        };
        let tx = transfer_tx(destination, amount)
            .with_to(a2h(to))
            .with_gas_limit(DEFAULT_TX_GAS);

        Ok(tx)
    }

    fn announce(
        &self,
        announcement: AnnouncementData,
        key_binding_fee: HoprBalance,
    ) -> payload::Result<Self::TxRequest> {
        // when the keys have already bounded, now only try to announce without key binding
        // when keys are not bounded yet, bind keys and announce together
        let serialized_signature = announcement.key_binding().signature.as_ref();

        let inner_payload = KeyBindAndAnnouncePayload {
            callerNode: a2h(self.me),
            ed25519_sig_0: B256::from_slice(&serialized_signature[0..32]),
            ed25519_sig_1: B256::from_slice(&serialized_signature[32..64]),
            ed25519_pub_key: B256::from_slice(announcement.key_binding().packet_key.as_ref()),
            multiaddress: announcement
                .multiaddress()
                .as_ref()
                .map(ToString::to_string)
                .unwrap_or_default(),
        }
        .abi_encode();

        let call_data = sendCall {
            recipient: a2h(self.contract_addrs.announcements),
            amount: alloy::primitives::U256::from_be_slice(&key_binding_fee.amount().to_be_bytes()),
            data: inner_payload[32..].to_vec().into(),
        }
        .abi_encode();

        Ok(TransactionRequest::default()
            .with_input(
                execTransactionFromModuleCall {
                    to: a2h(self.contract_addrs.token),
                    value: U256::ZERO,
                    data: call_data.into(),
                    operation: Operation::Call as u8,
                }
                .abi_encode(),
            )
            .with_to(a2h(self.module))
            .with_gas_limit(DEFAULT_TX_GAS))
    }

    fn fund_channel(&self, dest: Address, amount: HoprBalance) -> payload::Result<Self::TxRequest> {
        if dest.eq(&self.me) {
            return Err(InvalidArguments("Cannot fund channel to self"));
        }

        if amount.amount() > hopr_primitive_types::prelude::U256::from(ChannelEntry::MAX_CHANNEL_BALANCE) {
            return Err(InvalidArguments("Cannot fund channel with amount larger than 96 bits"));
        }

        let call_data = fundChannelSafeCall {
            selfAddress: a2h(self.me),
            account: a2h(dest),
            amount: U96::from_be_slice(&amount.amount().to_be_bytes()[32 - 12..]),
        }
        .abi_encode();

        let tx = TransactionRequest::default()
            .with_input(channels_payload(self.contract_addrs.channels, call_data))
            .with_to(a2h(self.module))
            .with_gas_limit(DEFAULT_TX_GAS);

        Ok(tx)
    }

    fn close_incoming_channel(&self, source: Address) -> payload::Result<Self::TxRequest> {
        if source.eq(&self.me) {
            return Err(InvalidArguments("Cannot close incoming channel from self"));
        }

        let call_data = closeIncomingChannelSafeCall {
            selfAddress: a2h(self.me),
            source: a2h(source),
        }
        .abi_encode();

        let tx = TransactionRequest::default()
            .with_input(channels_payload(self.contract_addrs.channels, call_data))
            .with_to(a2h(self.module))
            .with_gas_limit(DEFAULT_TX_GAS);

        Ok(tx)
    }

    fn initiate_outgoing_channel_closure(&self, destination: Address) -> payload::Result<Self::TxRequest> {
        if destination.eq(&self.me) {
            return Err(InvalidArguments("Cannot initiate closure of incoming channel to self"));
        }

        let call_data = initiateOutgoingChannelClosureSafeCall {
            selfAddress: a2h(self.me),
            destination: a2h(destination),
        }
        .abi_encode();

        let tx = TransactionRequest::default()
            .with_input(channels_payload(self.contract_addrs.channels, call_data))
            .with_to(a2h(self.module))
            .with_gas_limit(DEFAULT_TX_GAS);

        Ok(tx)
    }

    fn finalize_outgoing_channel_closure(&self, destination: Address) -> payload::Result<Self::TxRequest> {
        if destination.eq(&self.me) {
            return Err(InvalidArguments("Cannot initiate closure of incoming channel to self"));
        }

        let call_data = finalizeOutgoingChannelClosureSafeCall {
            selfAddress: a2h(self.me),
            destination: a2h(destination),
        }
        .abi_encode();

        let tx = TransactionRequest::default()
            .with_input(channels_payload(self.contract_addrs.channels, call_data))
            .with_to(a2h(self.module))
            .with_gas_limit(DEFAULT_TX_GAS);

        Ok(tx)
    }

    fn redeem_ticket(&self, acked_ticket: RedeemableTicket) -> payload::Result<Self::TxRequest> {
        let redeemable = convert_acknowledged_ticket(&acked_ticket)?;

        let params = convert_vrf_parameters(
            &acked_ticket.vrf_params,
            &self.me,
            acked_ticket.ticket.verified_hash(),
            &acked_ticket.channel_dst,
        );

        let call_data = redeemTicketSafeCall {
            selfAddress: a2h(self.me),
            redeemable,
            params,
        }
        .abi_encode();

        let tx = TransactionRequest::default()
            .with_input(channels_payload(self.contract_addrs.channels, call_data))
            .with_to(a2h(self.module))
            .with_gas_limit(DEFAULT_TX_GAS);

        Ok(tx)
    }

    fn register_safe_by_node(&self, safe_addr: Address) -> payload::Result<Self::TxRequest> {
        let tx = register_safe_tx(safe_addr)
            .with_to(a2h(self.contract_addrs.node_safe_registry))
            .with_gas_limit(DEFAULT_TX_GAS);

        Ok(tx)
    }

    fn deregister_node_by_safe(&self) -> payload::Result<Self::TxRequest> {
        let tx = TransactionRequest::default()
            .with_input(deregisterNodeBySafeCall { nodeAddr: a2h(self.me) }.abi_encode())
            .with_to(a2h(self.module))
            .with_gas_limit(DEFAULT_TX_GAS);

        Ok(tx)
    }
}

/// Converts off-chain representation of VRF parameters into a representation
/// that the smart contract understands
///
/// Not implemented using From trait because logic fits better here
fn convert_vrf_parameters(
    off_chain: &VrfParameters,
    signer: &Address,
    ticket_hash: &Hash,
    domain_separator: &Hash,
) -> VRFParameters {
    // skip the secp256k1 curvepoint prefix
    let v = off_chain.get_v_encoded_point();
    let s_b = off_chain
        .get_s_b_witness(signer, &ticket_hash.into(), domain_separator.as_ref())
        // Safe: hash value is always in the allowed length boundaries,
        //       only fails for longer values
        // Safe: always encoding to secp256k1 whose field elements are in
        //       allowed length boundaries
        .expect("ticket hash exceeded hash2field boundaries or encoding to unsupported curve");

    let h_v = off_chain.get_h_v_witness();

    VRFParameters {
        vx: U256::from_be_slice(&v.as_bytes()[1..33]),
        vy: U256::from_be_slice(&v.as_bytes()[33..65]),
        s: U256::from_be_slice(&off_chain.s.to_bytes()),
        h: U256::from_be_slice(&off_chain.h.to_bytes()),
        sBx: U256::from_be_slice(&s_b.as_bytes()[1..33]),
        sBy: U256::from_be_slice(&s_b.as_bytes()[33..65]),
        hVx: U256::from_be_slice(&h_v.as_bytes()[1..33]),
        hVy: U256::from_be_slice(&h_v.as_bytes()[33..65]),
    }
}

/// Convert off-chain representation of an acknowledged ticket to representation
/// that the smart contract understands
///
/// Not implemented using From trait because logic fits better here
fn convert_acknowledged_ticket(off_chain: &RedeemableTicket) -> payload::Result<OnChainRedeemableTicket> {
    if let Some(ref signature) = off_chain.verified_ticket().signature {
        let serialized_signature = signature.as_ref();

        Ok(OnChainRedeemableTicket {
            data: TicketData {
                channelId: B256::from_slice(off_chain.verified_ticket().channel_id.as_ref()),
                amount: U96::from_be_slice(&off_chain.verified_ticket().amount.amount().to_be_bytes()[32 - 12..]), /* Extract only the last 12 bytes (lowest 96 bits) */
                ticketIndex: U48::from_be_slice(&off_chain.verified_ticket().index.to_be_bytes()[8 - 6..]),
                epoch: U24::from_be_slice(&off_chain.verified_ticket().channel_epoch.to_be_bytes()[4 - 3..]),
                winProb: U56::from_be_slice(&off_chain.verified_ticket().encoded_win_prob),
            },
            signature: CompactSignature {
                r: B256::from_slice(&serialized_signature[0..32]),
                vs: B256::from_slice(&serialized_signature[32..64]),
            },
            porSecret: U256::from_be_slice(off_chain.response.as_ref()),
        })
    } else {
        Err(InvalidArguments("Acknowledged ticket must be signed"))
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use std::str::FromStr;

    use hex_literal::hex;
    use hopr_crypto_types::prelude::*;
    use hopr_internal_types::prelude::*;
    use hopr_primitive_types::prelude::*;
    use multiaddr::Multiaddr;

    use crate::payload::{
        BasicPayloadGenerator, PayloadGenerator, SafePayloadGenerator, SignableTransaction, tests::CONTRACT_ADDRS,
    };

    const PRIVATE_KEY_1: [u8; 32] = hex!("c14b8faa0a9b8a5fa4453664996f23a7e7de606d42297d723fc4a794f375e260");
    const PRIVATE_KEY_2: [u8; 32] = hex!("492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775");

    lazy_static::lazy_static! {
        static ref REDEEMABLE_TICKET: RedeemableTicket = bincode::serde::decode_from_slice(&hex!(
            "20d7f3fa28f0caa757e81226e1ee86a9efdbe7991442286183797296ebaa4d292a0330783101ffffffffffffff01766fb51964b4b8797d0ad0d580481ba0024dcf3c01409f24b8d12487dd48340af4ebdc44eff0f4c85ee25b27aca72f50af361022b32232fb24b408a8ed17967c1d700acc3fccbce3408d885d5b8fc41f01c4cf103deb2010312e537a39dbd374fb8dbb7031f255a75ceab085d4789154830f90d9c3e297668430def9eacd2b5064acf85d73fb0b351a1c8c20b58f99c83ae0e7dd6a69f755305b38c7610c7687d2931ff3f70103f8f92b90bb61026ffdd80eb690f54e85a2d085cb4f0cabdcf96f545af546b0fa8d67402d8c14ef04274e009938aaa5eea217f4103349d4b6bc80cabdd3b8177d473b8b00872d3c20bbac4b6ad5c187d6a7e7118df6d2e63a04f194b54204c18f61372f5b78d154200000000000000000000000000000000000000000000000000000000000000000"
        ), bincode::config::standard()).unwrap().0;
    }

    #[tokio::test]
    async fn test_announce() -> anyhow::Result<()> {
        let test_multiaddr = Multiaddr::from_str("/ip4/1.2.3.4/tcp/56")?;

        let chain_key_0 = ChainKeypair::from_secret(&PRIVATE_KEY_1)?;

        let generator = BasicPayloadGenerator::new((&chain_key_0).into(), *CONTRACT_ADDRS);

        let kb = KeyBinding::new((&chain_key_0).into(), &OffchainKeypair::from_secret(&PRIVATE_KEY_1)?);

        let ad = AnnouncementData::new(kb, Some(test_multiaddr))?;

        let signed_tx = generator
            .announce(ad, 100_u32.into())?
            .sign_and_encode_to_eip2718(2, None, &chain_key_0)
            .await?;
        insta::assert_snapshot!("announce_basic", hex::encode(signed_tx));

        let test_multiaddr_reannounce = Multiaddr::from_str("/ip4/5.6.7.8/tcp/99")?;
        let ad_reannounce = AnnouncementData::new(kb, Some(test_multiaddr_reannounce))?;

        let signed_tx = generator
            .announce(ad_reannounce, 0_u32.into())?
            .sign_and_encode_to_eip2718(1, None, &chain_key_0)
            .await?;
        insta::assert_snapshot!("announce_safe", hex::encode(signed_tx.clone()));

        Ok(())
    }

    #[tokio::test]
    async fn redeem_ticket() -> anyhow::Result<()> {
        let chain_key_bob = ChainKeypair::from_secret(&PRIVATE_KEY_2)?;

        let acked_ticket = REDEEMABLE_TICKET.clone();

        // Bob redeems the ticket
        let generator = BasicPayloadGenerator::new((&chain_key_bob).into(), *CONTRACT_ADDRS);
        let redeem_ticket_tx = generator.redeem_ticket(acked_ticket.clone())?;
        let signed_tx = redeem_ticket_tx
            .sign_and_encode_to_eip2718(1, None, &chain_key_bob)
            .await?;

        insta::assert_snapshot!("redeem_ticket_basic", hex::encode(signed_tx));

        // Bob redeems the ticket
        let generator =
            SafePayloadGenerator::new((&chain_key_bob).into(), *CONTRACT_ADDRS, [1u8; Address::SIZE].into());
        let redeem_ticket_tx = generator.redeem_ticket(acked_ticket)?;
        let signed_tx = redeem_ticket_tx
            .sign_and_encode_to_eip2718(2, None, &chain_key_bob)
            .await?;

        insta::assert_snapshot!("redeem_ticket_safe", hex::encode(signed_tx));

        Ok(())
    }

    #[tokio::test]
    async fn withdraw_token() -> anyhow::Result<()> {
        let chain_key_alice = ChainKeypair::from_secret(&PRIVATE_KEY_1)?;
        let chain_key_bob = ChainKeypair::from_secret(&PRIVATE_KEY_2)?;

        let generator = BasicPayloadGenerator::new((&chain_key_alice).into(), *CONTRACT_ADDRS);
        let tx = generator.transfer((&chain_key_bob).into(), HoprBalance::from(100))?;

        let signed_tx = tx.sign_and_encode_to_eip2718(1, None, &chain_key_bob).await?;

        insta::assert_snapshot!("withdraw_basic", hex::encode(signed_tx));

        let generator =
            SafePayloadGenerator::new((&chain_key_alice).into(), *CONTRACT_ADDRS, [1u8; Address::SIZE].into());
        let tx = generator.transfer((&chain_key_bob).into(), HoprBalance::from(100))?;

        let signed_tx = tx.sign_and_encode_to_eip2718(2, None, &chain_key_bob).await?;

        insta::assert_snapshot!("withdraw_safe", hex::encode(signed_tx));

        Ok(())
    }
}
