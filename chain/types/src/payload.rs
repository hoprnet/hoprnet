//! Module defining various Ethereum transaction payload generators for the actions.
//!
//! This module defines the basic [`PayloadGenerator`] trait that describes how an action
//! is translated into a [`TransactionRequest`] that can be submitted on-chain.
//!
//! There are two main implementations:
//! - [`BasicPayloadGenerator`] which implements generation of a direct EIP1559 transaction payload. This is currently
//!   not used by a HOPR node.
//! - [`SafePayloadGenerator`] which implements generation of a payload that embeds the transaction data into the SAFE
//!   transaction. This is currently the main mode of HOPR node operation.

use alloy::{
    consensus::TxEnvelope,
    eips::Encodable2718,
    network::{EthereumWallet, TransactionBuilder},
    primitives::{
        B256, U256,
        aliases::{U24, U48, U56, U96},
    },
    rpc::types::TransactionRequest,
    signers::local::PrivateKeySigner,
    sol_types::SolCall,
};
use hopr_bindings::{
    hoprannouncements::HoprAnnouncements::{
        announceCall, announceSafeCall, bindKeysAnnounceCall, bindKeysAnnounceSafeCall,
    },
    hoprchannels::{
        HoprChannels::{
            RedeemableTicket as OnChainRedeemableTicket, TicketData, closeIncomingChannelCall,
            closeIncomingChannelSafeCall, finalizeOutgoingChannelClosureCall, finalizeOutgoingChannelClosureSafeCall,
            fundChannelCall, fundChannelSafeCall, initiateOutgoingChannelClosureCall,
            initiateOutgoingChannelClosureSafeCall, redeemTicketCall, redeemTicketSafeCall,
        },
        HoprCrypto::{CompactSignature, VRFParameters},
    },
    hoprnodemanagementmodule::HoprNodeManagementModule::execTransactionFromModuleCall,
    hoprnodesaferegistry::HoprNodeSafeRegistry::{deregisterNodeBySafeCall, registerSafeByNodeCall},
    hoprtoken::HoprToken::{approveCall, transferCall},
};
use hopr_crypto_types::prelude::*;
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::*;

use crate::{
    ContractAddresses,
    errors::ChainTypesError::{InvalidArguments, InvalidState, SigningError},
};

type Result<T> = std::result::Result<T, crate::errors::ChainTypesError>;

// Used instead of From implementation to avoid alloy being a dependency of the primitive crates
fn a2h(a: hopr_primitive_types::prelude::Address) -> alloy::primitives::Address {
    alloy::primitives::Address::from_slice(a.as_ref())
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Operation {
    Call = 0,
    // Future use: DelegateCall = 1,
}

#[async_trait::async_trait]
pub trait SignableTransaction {
    async fn sign_and_encode_to_eip2718(self, chain_keypair: &ChainKeypair) -> Result<Box<[u8]>>;
}

#[async_trait::async_trait]
impl SignableTransaction for TransactionRequest {
    async fn sign_and_encode_to_eip2718(self, chain_keypair: &ChainKeypair) -> Result<Box<[u8]>> {
        let signer: EthereumWallet = PrivateKeySigner::from_slice(chain_keypair.secret().as_ref())
            .map_err(|e| SigningError(e.into()))?
            .into();
        let signed: TxEnvelope = self.build(&signer).await.map_err(|e| SigningError(e.into()))?;

        Ok(signed.encoded_2718().into_boxed_slice())
    }
}

/// Trait for various implementations of common on-chain transaction payloads generators.
pub trait PayloadGenerator {
    type TxRequest: SignableTransaction + Send;

    /// Create an ERC20 approve transaction payload. Pre-requisite to open payment channels.
    /// The `spender` address is typically the HOPR Channels contract address.
    fn approve(&self, spender: Address, amount: HoprBalance) -> Result<Self::TxRequest>;

    /// Create a ERC20 transfer transaction payload
    fn transfer<C: Currency>(&self, destination: Address, amount: Balance<C>) -> Result<Self::TxRequest>;

    /// Creates the transaction payload to announce a node on-chain.
    fn announce(&self, announcement: AnnouncementData) -> Result<Self::TxRequest>;

    /// Creates the transaction payload to open a payment channel
    fn fund_channel(&self, dest: Address, amount: HoprBalance) -> Result<Self::TxRequest>;

    /// Creates the transaction payload to immediately close an incoming payment channel
    fn close_incoming_channel(&self, source: Address) -> Result<Self::TxRequest>;

    /// Creates the transaction payload that initiates the closure of a payment channel.
    /// Once the notice period is due, the funds can be withdrawn using a
    /// finalizeChannelClosure transaction.
    fn initiate_outgoing_channel_closure(&self, destination: Address) -> Result<Self::TxRequest>;

    /// Creates a transaction payload that withdraws funds from
    /// an outgoing payment channel. This will succeed once the closure
    /// notice period is due.
    fn finalize_outgoing_channel_closure(&self, destination: Address) -> Result<Self::TxRequest>;

    /// Used to create the payload to claim incentives for relaying a mixnet packet.
    fn redeem_ticket(&self, acked_ticket: RedeemableTicket) -> Result<Self::TxRequest>;

    /// Creates a transaction payload to register a Safe instance which is used
    /// to manage the node's funds
    fn register_safe_by_node(&self, safe_addr: Address) -> Result<Self::TxRequest>;

    /// Creates a transaction payload to remove the Safe instance. Once succeeded,
    /// the node no longer manages the funds.
    fn deregister_node_by_safe(&self) -> Result<Self::TxRequest>;
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

    fn approve(&self, spender: Address, amount: HoprBalance) -> Result<Self::TxRequest> {
        let tx = approve_tx(spender, amount).with_to(a2h(self.contract_addrs.token));
        Ok(tx)
    }

    fn transfer<C: Currency>(&self, destination: Address, amount: Balance<C>) -> Result<Self::TxRequest> {
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

    fn announce(&self, announcement: AnnouncementData) -> Result<Self::TxRequest> {
        let payload = match &announcement.key_binding {
            Some(binding) => {
                let serialized_signature = binding.signature.as_ref();

                bindKeysAnnounceCall {
                    ed25519_sig_0: B256::from_slice(&serialized_signature[0..32]),
                    ed25519_sig_1: B256::from_slice(&serialized_signature[32..64]),
                    ed25519_pub_key: B256::from_slice(binding.packet_key.as_ref()),
                    baseMultiaddr: announcement.multiaddress().to_string(),
                }
                .abi_encode()
            }
            None => announceCall {
                baseMultiaddr: announcement.multiaddress().to_string(),
            }
            .abi_encode(),
        };

        let tx = TransactionRequest::default()
            .with_input(payload)
            .with_to(a2h(self.contract_addrs.announcements));
        Ok(tx)
    }

    fn fund_channel(&self, dest: Address, amount: HoprBalance) -> Result<Self::TxRequest> {
        if dest.eq(&self.me) {
            return Err(InvalidArguments("Cannot fund channel to self".into()));
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

    fn close_incoming_channel(&self, source: Address) -> Result<Self::TxRequest> {
        if source.eq(&self.me) {
            return Err(InvalidArguments("Cannot close incoming channel from self".into()));
        }

        let tx = TransactionRequest::default()
            .with_input(closeIncomingChannelCall { source: a2h(source) }.abi_encode())
            .with_to(a2h(self.contract_addrs.channels));
        Ok(tx)
    }

    fn initiate_outgoing_channel_closure(&self, destination: Address) -> Result<Self::TxRequest> {
        if destination.eq(&self.me) {
            return Err(InvalidArguments(
                "Cannot initiate closure of incoming channel to self".into(),
            ));
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

    fn finalize_outgoing_channel_closure(&self, destination: Address) -> Result<Self::TxRequest> {
        if destination.eq(&self.me) {
            return Err(InvalidArguments(
                "Cannot initiate closure of incoming channel to self".into(),
            ));
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

    fn redeem_ticket(&self, acked_ticket: RedeemableTicket) -> Result<Self::TxRequest> {
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

    fn register_safe_by_node(&self, safe_addr: Address) -> Result<Self::TxRequest> {
        let tx = register_safe_tx(safe_addr).with_to(a2h(self.contract_addrs.node_safe_registry));
        Ok(tx)
    }

    fn deregister_node_by_safe(&self) -> Result<Self::TxRequest> {
        Err(InvalidState(
            "Can only deregister an address if Safe is activated".into(),
        ))
    }
}

/// Payload generator that generates Safe-compliant ABI
#[derive(Debug, Clone, Copy)]
pub struct SafePayloadGenerator {
    me: Address,
    contract_addrs: ContractAddresses,
    module: Address,
}

const DEFAULT_TX_GAS: u64 = 400_000;

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

    fn approve(&self, spender: Address, amount: HoprBalance) -> Result<Self::TxRequest> {
        let tx = approve_tx(spender, amount)
            .with_to(a2h(self.contract_addrs.token))
            .with_gas_limit(DEFAULT_TX_GAS);

        Ok(tx)
    }

    fn transfer<C: Currency>(&self, destination: Address, amount: Balance<C>) -> Result<Self::TxRequest> {
        let to = if XDai::is::<C>() {
            destination
        } else if WxHOPR::is::<C>() {
            self.contract_addrs.token
        } else {
            return Err(InvalidArguments("invalid currency".into()));
        };
        let tx = transfer_tx(destination, amount)
            .with_to(a2h(to))
            .with_gas_limit(DEFAULT_TX_GAS);

        Ok(tx)
    }

    fn announce(&self, announcement: AnnouncementData) -> Result<Self::TxRequest> {
        let call_data = match &announcement.key_binding {
            Some(binding) => {
                let serialized_signature = binding.signature.as_ref();

                bindKeysAnnounceSafeCall {
                    selfAddress: a2h(self.me),
                    ed25519_sig_0: B256::from_slice(&serialized_signature[0..32]),
                    ed25519_sig_1: B256::from_slice(&serialized_signature[32..64]),
                    ed25519_pub_key: B256::from_slice(binding.packet_key.as_ref()),
                    baseMultiaddr: announcement.multiaddress().to_string(),
                }
                .abi_encode()
            }
            None => announceSafeCall {
                selfAddress: a2h(self.me),
                baseMultiaddr: announcement.multiaddress().to_string(),
            }
            .abi_encode(),
        };

        let tx = TransactionRequest::default()
            .with_input(
                execTransactionFromModuleCall {
                    to: a2h(self.contract_addrs.announcements),
                    value: U256::ZERO,
                    data: call_data.into(),
                    operation: Operation::Call as u8,
                }
                .abi_encode(),
            )
            .with_to(a2h(self.module))
            .with_gas_limit(DEFAULT_TX_GAS);

        Ok(tx)
    }

    fn fund_channel(&self, dest: Address, amount: HoprBalance) -> Result<Self::TxRequest> {
        if dest.eq(&self.me) {
            return Err(InvalidArguments("Cannot fund channel to self".into()));
        }

        if amount.amount() > hopr_primitive_types::prelude::U256::from(ChannelEntry::MAX_CHANNEL_BALANCE) {
            return Err(InvalidArguments(
                "Cannot fund channel with amount larger than 96 bits".into(),
            ));
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

    fn close_incoming_channel(&self, source: Address) -> Result<Self::TxRequest> {
        if source.eq(&self.me) {
            return Err(InvalidArguments("Cannot close incoming channel from self".into()));
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

    fn initiate_outgoing_channel_closure(&self, destination: Address) -> Result<Self::TxRequest> {
        if destination.eq(&self.me) {
            return Err(InvalidArguments(
                "Cannot initiate closure of incoming channel to self".into(),
            ));
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

    fn finalize_outgoing_channel_closure(&self, destination: Address) -> Result<Self::TxRequest> {
        if destination.eq(&self.me) {
            return Err(InvalidArguments(
                "Cannot initiate closure of incoming channel to self".into(),
            ));
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

    fn redeem_ticket(&self, acked_ticket: RedeemableTicket) -> Result<Self::TxRequest> {
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

    fn register_safe_by_node(&self, safe_addr: Address) -> Result<Self::TxRequest> {
        let tx = register_safe_tx(safe_addr)
            .with_to(a2h(self.contract_addrs.node_safe_registry))
            .with_gas_limit(DEFAULT_TX_GAS);

        Ok(tx)
    }

    fn deregister_node_by_safe(&self) -> Result<Self::TxRequest> {
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
fn convert_acknowledged_ticket(off_chain: &RedeemableTicket) -> Result<OnChainRedeemableTicket> {
    if let Some(ref signature) = off_chain.verified_ticket().signature {
        let serialized_signature = signature.as_ref();

        Ok(OnChainRedeemableTicket {
            data: TicketData {
                channelId: B256::from_slice(off_chain.verified_ticket().channel_id.as_ref()),
                amount: U96::from_be_slice(&off_chain.verified_ticket().amount.amount().to_be_bytes()[32 - 12..]), /* Extract only the last 12 bytes (lowest 96 bits) */
                ticketIndex: U48::from_be_slice(&off_chain.verified_ticket().index.to_be_bytes()[8 - 6..]),
                indexOffset: 1,
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
        Err(InvalidArguments("Acknowledged ticket must be signed".into()))
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use alloy::providers::{
        Identity, Provider, RootProvider,
        fillers::{BlobGasFiller, ChainIdFiller, FillProvider, GasFiller, JoinFill, NonceFiller, WalletFiller},
    };
    use hex_literal::hex;
    use hopr_crypto_types::prelude::*;
    use hopr_internal_types::prelude::*;
    use hopr_primitive_types::prelude::{Address, BytesRepresentable, HoprBalance};
    use multiaddr::Multiaddr;

    use super::{BasicPayloadGenerator, PayloadGenerator, SafePayloadGenerator, SignableTransaction};
    use crate::ContractAddresses;

    const PRIVATE_KEY_1: [u8; 32] = hex!("c14b8faa0a9b8a5fa4453664996f23a7e7de606d42297d723fc4a794f375e260");
    const PRIVATE_KEY_2: [u8; 32] = hex!("492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775");
    const RESPONSE_TO_CHALLENGE: [u8; 32] = hex!("b58f99c83ae0e7dd6a69f755305b38c7610c7687d2931ff3f70103f8f92b90bb");

    lazy_static::lazy_static! {
        static ref CONTRACT_ADDRS: ContractAddresses = serde_json::from_str(r#"{
            "announcements": "0xf1c143B1bA20C7606d56aA2FA94502D25744b982",
            "channels": "0x77C9414043d27fdC98A6A2d73fc77b9b383092a7",
            "module_implementation": "0x32863c4974fBb6253E338a0cb70C382DCeD2eFCb",
            "network_registry": "0x15a315E1320cFF0de84671c0139042EE320CE38d",
            "network_registry_proxy": "0x20559cbD3C2eDcD0b396431226C00D2Cd102eB3F",
            "node_safe_registry": "0x4F7C7dE3BA2B29ED8B2448dF2213cA43f94E45c0",
            "node_stake_v2_factory": "0x791d190b2c95397F4BcE7bD8032FD67dCEA7a5F2",
            "token": "0xD4fdec44DB9D44B8f2b6d529620f9C0C7066A2c1",
            "ticket_price_oracle": "0x442df1d946303fB088C9377eefdaeA84146DA0A6",
            "winning_probability_oracle": "0xC15675d4CCa538D91a91a8D3EcFBB8499C3B0471"
        }"#).unwrap();
    }

    #[tokio::test]
    async fn test_announce() -> anyhow::Result<()> {
        let test_multiaddr = Multiaddr::from_str("/ip4/1.2.3.4/tcp/56")?;

        let chain_key_0 = ChainKeypair::from_secret(&PRIVATE_KEY_1)?;

        let generator = BasicPayloadGenerator::new((&chain_key_0).into(), *CONTRACT_ADDRS);

        let ad = AnnouncementData::new(
            test_multiaddr,
            Some(KeyBinding::new(
                (&chain_key_0).into(),
                &OffchainKeypair::from_secret(&PRIVATE_KEY_1)?,
            )),
        )?;

        let signed_tx = generator.announce(ad)?.sign_and_encode_to_eip2718(&chain_key_0).await?;
        insta::assert_snapshot!(hex::encode(signed_tx));

        let test_multiaddr_reannounce = Multiaddr::from_str("/ip4/5.6.7.8/tcp/99")?;
        let ad_reannounce = AnnouncementData::new(test_multiaddr_reannounce, None)?;

        let signed_tx = generator
            .announce(ad_reannounce)?
            .sign_and_encode_to_eip2718(&chain_key_0)
            .await?;
        insta::assert_snapshot!(hex::encode(signed_tx));

        Ok(())
    }

    #[tokio::test]
    async fn redeem_ticket() -> anyhow::Result<()> {
        let domain_separator = Hash::default();
        let chain_key_alice = ChainKeypair::from_secret(&PRIVATE_KEY_1)?;
        let chain_key_bob = ChainKeypair::from_secret(&PRIVATE_KEY_2)?;

        let response = Response::try_from(RESPONSE_TO_CHALLENGE.as_ref())?;

        // Alice issues a ticket to Bob
        let ticket = TicketBuilder::default()
            .addresses(&chain_key_alice, &chain_key_bob)
            .amount(1)
            .index(1)
            .win_prob(1.0.try_into()?)
            .channel_epoch(1)
            .challenge(response.to_challenge()?)
            .build_signed(&chain_key_alice, &domain_separator)?;

        // Bob acknowledges the ticket using the HalfKey from the Response
        let acked_ticket = ticket
            .into_acknowledged(response)
            .into_redeemable(&chain_key_bob, &domain_separator)?;

        // Bob redeems the ticket
        let generator = BasicPayloadGenerator::new((&chain_key_bob).into(), *CONTRACT_ADDRS);
        let redeem_ticket_tx = generator.redeem_ticket(acked_ticket.clone())?;
        let signed_tx = redeem_ticket_tx.sign_and_encode_to_eip2718(&chain_key_bob).await?;

        insta::assert_snapshot!(hex::encode(signed_tx));

        let generator =
            SafePayloadGenerator::new((&chain_key_bob).into(), *CONTRACT_ADDRS, [1u8; Address::SIZE].into());
        let redeem_ticket_tx = generator.redeem_ticket(acked_ticket)?;
        let signed_tx = redeem_ticket_tx.sign_and_encode_to_eip2718(&chain_key_bob).await?;

        insta::assert_snapshot!(hex::encode(signed_tx));

        Ok(())
    }

    #[tokio::test]
    async fn withdraw_token() -> anyhow::Result<()> {
        let chain_key_alice = ChainKeypair::from_secret(&PRIVATE_KEY_1)?;
        let chain_key_bob = ChainKeypair::from_secret(&PRIVATE_KEY_2)?;

        let generator = BasicPayloadGenerator::new((&chain_key_alice).into(), *CONTRACT_ADDRS);
        let tx = generator.transfer((&chain_key_bob).into(), HoprBalance::from(100))?;

        let signed_tx = tx.sign_and_encode_to_eip2718(&chain_key_bob).await?;

        insta::assert_snapshot!(hex::encode(signed_tx));

        let generator =
            SafePayloadGenerator::new((&chain_key_alice).into(), *CONTRACT_ADDRS, [1u8; Address::SIZE].into());
        let tx = generator.transfer((&chain_key_bob).into(), HoprBalance::from(100))?;

        let signed_tx = tx.sign_and_encode_to_eip2718(&chain_key_bob).await?;

        insta::assert_snapshot!(hex::encode(signed_tx));

        Ok(())
    }
}
