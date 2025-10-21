//! Module defining various Ethereum transaction payload generators for the actions.
//!
//! This module defines the basic [`PayloadGenerator`] trait that describes how an action
//! is translated into a [`TransactionRequest`] that can be submitted on-chain.
//!
//! There are two main implementations:
//! - [`BasicPayloadGenerator`] which implements generation of a direct EIP1559 transaction payload. This is currently not
//!   used by a HOPR node.
//! - [`SafePayloadGenerator`] which implements generation of a payload that embeds the transaction data into the SAFE
//!   transaction. This is currently the main mode of HOPR node operation.

use alloy::{
    network::TransactionBuilder,
    primitives::{
        B256, U256,
        aliases::{U24, U48, U56, U96},
    },
    rpc::types::TransactionRequest,
    sol_types::SolCall,
};
use alloy::consensus::TxEnvelope;
use alloy::eips::Encodable2718;
use alloy::network::EthereumWallet;
use alloy::signers::local::PrivateKeySigner;
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
use hopr_chain_types::ContractAddresses;
use hopr_crypto_types::prelude::*;
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::*;
use crate::errors::ConnectorError;
use crate::errors::ConnectorError::{InvalidArguments, InvalidState, SigningError};

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Operation {
    Call = 0,
    // Future use: DelegateCall = 1,
}

type Result<T> = std::result::Result<T, ConnectorError>;

/// Trait for various implementations of common on-chain transaction payloads generators.
pub trait PayloadGenerator<T: Into<TransactionRequest>> {
    /// Create an ERC20 approve transaction payload. Pre-requisite to open payment channels.
    /// The `spender` address is typically the HOPR Channels contract address.
    fn approve(&self, spender: Address, amount: HoprBalance) -> Result<T>;

    /// Create a ERC20 transfer transaction payload
    fn transfer<C: Currency>(&self, destination: Address, amount: Balance<C>) -> Result<T>;

    /// Creates the transaction payload to announce a node on-chain.
    fn announce(&self, announcement: AnnouncementData) -> Result<T>;

    /// Creates the transaction payload to open a payment channel
    fn fund_channel(&self, dest: Address, amount: HoprBalance) -> Result<T>;

    /// Creates the transaction payload to immediately close an incoming payment channel
    fn close_incoming_channel(&self, source: Address) -> Result<T>;

    /// Creates the transaction payload that initiates the closure of a payment channel.
    /// Once the notice period is due, the funds can be withdrawn using a
    /// finalizeChannelClosure transaction.
    fn initiate_outgoing_channel_closure(&self, destination: Address) -> Result<T>;

    /// Creates a transaction payload that withdraws funds from
    /// an outgoing payment channel. This will succeed once the closure
    /// notice period is due.
    fn finalize_outgoing_channel_closure(&self, destination: Address) -> Result<T>;

    /// Used to create the payload to claim incentives for relaying a mixnet packet.
    fn redeem_ticket(&self, acked_ticket: RedeemableTicket) -> Result<T>;

    /// Creates a transaction payload to register a Safe instance which is used
    /// to manage the node's funds
    fn register_safe_by_node(&self, safe_addr: Address) -> Result<T>;

    /// Creates a transaction payload to remove the Safe instance. Once succeeded,
    /// the node no longer manages the funds.
    fn deregister_node_by_safe(&self) -> Result<T>;
}

/// Signs the given `tx_payload` (possibly generated via some [`PayloadGenerator`]) using the given [`ChainKeypair`],
/// and returns the serialized EIP2718 format.
pub async fn sign_payload<T: Into<TransactionRequest>>(tx_payload: T, keypair: &ChainKeypair) -> Result<Box<[u8]>> {
    let payload = tx_payload.into();
    let signer: EthereumWallet = PrivateKeySigner::from_slice(keypair.secret().as_ref()).map_err(|e| SigningError(e.into()))?.into();
    let signed: TxEnvelope = payload.build(&signer).await.map_err(|e| SigningError(e.into()))?;

    Ok(signed.encoded_2718().into_boxed_slice())
}

fn channels_payload(hopr_channels: Address, call_data: Vec<u8>) -> Vec<u8> {
    execTransactionFromModuleCall {
        to: hopr_channels.into(),
        value: U256::ZERO,
        data: call_data.into(),
        operation: Operation::Call as u8,
    }
        .abi_encode()
}

fn approve_tx(spender: Address, amount: HoprBalance) -> TransactionRequest {
    TransactionRequest::default().with_input(
        approveCall {
            spender: spender.into(),
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
                recipient: destination.into(),
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
            safeAddr: safe_addr.into(),
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

impl PayloadGenerator<TransactionRequest> for BasicPayloadGenerator {
    fn approve(&self, spender: Address, amount: HoprBalance) -> Result<TransactionRequest> {
        let tx = approve_tx(spender, amount).with_to(self.contract_addrs.token.into());
        Ok(tx)
    }

    fn transfer<C: Currency>(&self, destination: Address, amount: Balance<C>) -> Result<TransactionRequest> {
        let to = if XDai::is::<C>() {
            destination
        } else if WxHOPR::is::<C>() {
            self.contract_addrs.token
        } else {
            return Err(InvalidArguments("invalid currency"));
        };
        let tx = transfer_tx(destination, amount).with_to(to.into());
        Ok(tx)
    }

    fn announce(&self, announcement: AnnouncementData) -> Result<TransactionRequest> {
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
            .with_to(self.contract_addrs.announcements.into());
        Ok(tx)
    }

    fn fund_channel(&self, dest: Address, amount: HoprBalance) -> Result<TransactionRequest> {
        if dest.eq(&self.me) {
            return Err(InvalidArguments("Cannot fund channel to self".into()));
        }

        let tx = TransactionRequest::default()
            .with_input(
                fundChannelCall {
                    account: dest.into(),
                    amount: U96::from_be_slice(&amount.amount().to_be_bytes()[32 - 12..]),
                }
                    .abi_encode(),
            )
            .with_to(self.contract_addrs.channels.into());
        Ok(tx)
    }

    fn close_incoming_channel(&self, source: Address) -> Result<TransactionRequest> {
        if source.eq(&self.me) {
            return Err(InvalidArguments("Cannot close incoming channel from self".into()));
        }

        let tx = TransactionRequest::default()
            .with_input(closeIncomingChannelCall { source: source.into() }.abi_encode())
            .with_to(self.contract_addrs.channels.into());
        Ok(tx)
    }

    fn initiate_outgoing_channel_closure(&self, destination: Address) -> Result<TransactionRequest> {
        if destination.eq(&self.me) {
            return Err(InvalidArguments(
                "Cannot initiate closure of incoming channel to self".into(),
            ));
        }

        let tx = TransactionRequest::default()
            .with_input(
                initiateOutgoingChannelClosureCall {
                    destination: destination.into(),
                }
                    .abi_encode(),
            )
            .with_to(self.contract_addrs.channels.into());
        Ok(tx)
    }

    fn finalize_outgoing_channel_closure(&self, destination: Address) -> Result<TransactionRequest> {
        if destination.eq(&self.me) {
            return Err(InvalidArguments(
                "Cannot initiate closure of incoming channel to self".into(),
            ));
        }

        let tx = TransactionRequest::default()
            .with_input(
                finalizeOutgoingChannelClosureCall {
                    destination: destination.into(),
                }
                    .abi_encode(),
            )
            .with_to(self.contract_addrs.channels.into());
        Ok(tx)
    }

    fn redeem_ticket(&self, acked_ticket: RedeemableTicket) -> Result<TransactionRequest> {
        let redeemable = convert_acknowledged_ticket(&acked_ticket)?;

        let params = convert_vrf_parameters(
            &acked_ticket.vrf_params,
            &self.me,
            acked_ticket.ticket.verified_hash(),
            &acked_ticket.channel_dst,
        );

        let tx = TransactionRequest::default()
            .with_input(redeemTicketCall { redeemable, params }.abi_encode())
            .with_to(self.contract_addrs.channels.into());
        Ok(tx)
    }

    fn register_safe_by_node(&self, safe_addr: Address) -> Result<TransactionRequest> {
        let tx = register_safe_tx(safe_addr).with_to(self.contract_addrs.node_safe_registry.into());
        Ok(tx)
    }

    fn deregister_node_by_safe(&self) -> Result<TransactionRequest> {
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

impl PayloadGenerator<TransactionRequest> for SafePayloadGenerator {
    fn approve(&self, spender: Address, amount: HoprBalance) -> Result<TransactionRequest> {
        let tx = approve_tx(spender, amount)
            .with_to(self.contract_addrs.token.into())
            .with_gas_limit(DEFAULT_TX_GAS);

        Ok(tx)
    }

    fn transfer<C: Currency>(&self, destination: Address, amount: Balance<C>) -> Result<TransactionRequest> {
        let to = if XDai::is::<C>() {
            destination
        } else if WxHOPR::is::<C>() {
            self.contract_addrs.token
        } else {
            return Err(InvalidArguments("invalid currency".into()));
        };
        let tx = transfer_tx(destination, amount)
            .with_to(to.into())
            .with_gas_limit(DEFAULT_TX_GAS);

        Ok(tx)
    }

    fn announce(&self, announcement: AnnouncementData) -> Result<TransactionRequest> {
        let call_data = match &announcement.key_binding {
            Some(binding) => {
                let serialized_signature = binding.signature.as_ref();

                bindKeysAnnounceSafeCall {
                    selfAddress: self.me.into(),
                    ed25519_sig_0: B256::from_slice(&serialized_signature[0..32]),
                    ed25519_sig_1: B256::from_slice(&serialized_signature[32..64]),
                    ed25519_pub_key: B256::from_slice(binding.packet_key.as_ref()),
                    baseMultiaddr: announcement.multiaddress().to_string(),
                }
                    .abi_encode()
            }
            None => announceSafeCall {
                selfAddress: self.me.into(),
                baseMultiaddr: announcement.multiaddress().to_string(),
            }
                .abi_encode(),
        };

        let tx = TransactionRequest::default()
            .with_input(
                execTransactionFromModuleCall {
                    to: self.contract_addrs.announcements.into(),
                    value: U256::ZERO,
                    data: call_data.into(),
                    operation: Operation::Call as u8,
                }
                    .abi_encode(),
            )
            .with_to(self.module.into())
            .with_gas_limit(DEFAULT_TX_GAS);

        Ok(tx)
    }

    fn fund_channel(&self, dest: Address, amount: HoprBalance) -> Result<TransactionRequest> {
        if dest.eq(&self.me) {
            return Err(InvalidArguments("Cannot fund channel to self".into()));
        }

        if amount.amount() > hopr_primitive_types::prelude::U256::from(ChannelEntry::MAX_CHANNEL_BALANCE) {
            return Err(InvalidArguments(
                "Cannot fund channel with amount larger than 96 bits".into(),
            ));
        }

        let call_data = fundChannelSafeCall {
            selfAddress: self.me.into(),
            account: dest.into(),
            amount: U96::from_be_slice(&amount.amount().to_be_bytes()[32 - 12..]),
        }
            .abi_encode();

        let tx = TransactionRequest::default()
            .with_input(channels_payload(self.contract_addrs.channels, call_data))
            .with_to(self.module.into())
            .with_gas_limit(DEFAULT_TX_GAS);

        Ok(tx)
    }

    fn close_incoming_channel(&self, source: Address) -> Result<TransactionRequest> {
        if source.eq(&self.me) {
            return Err(InvalidArguments("Cannot close incoming channel from self".into()));
        }

        let call_data = closeIncomingChannelSafeCall {
            selfAddress: self.me.into(),
            source: source.into(),
        }
            .abi_encode();

        let tx = TransactionRequest::default()
            .with_input(channels_payload(self.contract_addrs.channels, call_data))
            .with_to(self.module.into())
            .with_gas_limit(DEFAULT_TX_GAS);

        Ok(tx)
    }

    fn initiate_outgoing_channel_closure(&self, destination: Address) -> Result<TransactionRequest> {
        if destination.eq(&self.me) {
            return Err(InvalidArguments(
                "Cannot initiate closure of incoming channel to self".into(),
            ));
        }

        let call_data = initiateOutgoingChannelClosureSafeCall {
            selfAddress: self.me.into(),
            destination: destination.into(),
        }
            .abi_encode();

        let tx = TransactionRequest::default()
            .with_input(channels_payload(self.contract_addrs.channels, call_data))
            .with_to(self.module.into())
            .with_gas_limit(DEFAULT_TX_GAS);

        Ok(tx)
    }

    fn finalize_outgoing_channel_closure(&self, destination: Address) -> Result<TransactionRequest> {
        if destination.eq(&self.me) {
            return Err(InvalidArguments(
                "Cannot initiate closure of incoming channel to self".into(),
            ));
        }

        let call_data = finalizeOutgoingChannelClosureSafeCall {
            selfAddress: self.me.into(),
            destination: destination.into(),
        }
            .abi_encode();

        let tx = TransactionRequest::default()
            .with_input(channels_payload(self.contract_addrs.channels, call_data))
            .with_to(self.module.into())
            .with_gas_limit(DEFAULT_TX_GAS);

        Ok(tx)
    }

    fn redeem_ticket(&self, acked_ticket: RedeemableTicket) -> Result<TransactionRequest> {
        let redeemable = convert_acknowledged_ticket(&acked_ticket)?;

        let params = convert_vrf_parameters(
            &acked_ticket.vrf_params,
            &self.me,
            acked_ticket.ticket.verified_hash(),
            &acked_ticket.channel_dst,
        );

        let call_data = redeemTicketSafeCall {
            selfAddress: self.me.into(),
            redeemable,
            params,
        }
            .abi_encode();

        let tx = TransactionRequest::default()
            .with_input(channels_payload(self.contract_addrs.channels, call_data))
            .with_to(self.module.into())
            .with_gas_limit(DEFAULT_TX_GAS);

        Ok(tx)
    }

    fn register_safe_by_node(&self, safe_addr: Address) -> Result<TransactionRequest> {
        let tx = register_safe_tx(safe_addr)
            .with_to(self.contract_addrs.node_safe_registry.into())
            .with_gas_limit(DEFAULT_TX_GAS);

        Ok(tx)
    }

    fn deregister_node_by_safe(&self) -> Result<TransactionRequest> {
        let tx = TransactionRequest::default()
            .with_input(
                deregisterNodeBySafeCall {
                    nodeAddr: self.me.into(),
                }
                    .abi_encode(),
            )
            .with_to(self.module.into())
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
                indexOffset: off_chain.verified_ticket().index_offset,
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
    use std::sync::Arc;
    use alloy::{primitives::U256, providers::Provider};
    use alloy::network::EthereumWallet;
    use alloy::providers::fillers::{BlobGasFiller, ChainIdFiller, FillProvider, GasFiller, JoinFill, NonceFiller, WalletFiller};
    use alloy::providers::{Identity, RootProvider};
    use anyhow::Context;
    use hex_literal::hex;
    use hopr_chain_types::ContractInstances;
    use hopr_crypto_types::prelude::*;
    use hopr_internal_types::prelude::*;
    use hopr_primitive_types::prelude::HoprBalance;
    use multiaddr::Multiaddr;

    use super::{BasicPayloadGenerator, PayloadGenerator};

    const PRIVATE_KEY: [u8; 32] = hex!("c14b8faa0a9b8a5fa4453664996f23a7e7de606d42297d723fc4a794f375e260");
    const RESPONSE_TO_CHALLENGE: [u8; 32] = hex!("b58f99c83ae0e7dd6a69f755305b38c7610c7687d2931ff3f70103f8f92b90bb");

    pub type AnvilRpcClient = FillProvider<
        JoinFill<
            JoinFill<Identity, JoinFill<GasFiller, JoinFill<BlobGasFiller, JoinFill<NonceFiller, ChainIdFiller>>>>,
            WalletFiller<EthereumWallet>,
        >,
        RootProvider,
    >;

    pub fn create_rpc_client_to_anvil(
        anvil: &alloy::node_bindings::AnvilInstance,
        signer: &hopr_crypto_types::keypairs::ChainKeypair,
    ) -> Arc<AnvilRpcClient> {
        use alloy::{
            providers::ProviderBuilder, rpc::client::ClientBuilder, signers::local::PrivateKeySigner,
            transports::http::ReqwestTransport,
        };
        use hopr_crypto_types::keypairs::Keypair;

        let wallet = PrivateKeySigner::from_slice(signer.secret().as_ref()).expect("failed to construct wallet");

        let transport_client = ReqwestTransport::new(anvil.endpoint_url());

        let rpc_client = ClientBuilder::default().transport(transport_client.clone(), transport_client.guess_local());

        let provider = ProviderBuilder::new().wallet(wallet).connect_client(rpc_client);

        Arc::new(provider)
    }

    #[tokio::test]
    async fn test_announce() -> anyhow::Result<()> {
        let test_multiaddr = Multiaddr::from_str("/ip4/1.2.3.4/tcp/56")?;

        let anvil = hopr_chain_types::utils::create_anvil(None);
        let chain_key_0 = ChainKeypair::from_secret(anvil.keys()[0].to_bytes().as_ref())?;
        let client = create_rpc_client_to_anvil(&anvil, &chain_key_0);

        // Deploy contracts
        let contract_instances = ContractInstances::deploy_for_testing(client.clone(), &chain_key_0)
            .await
            .context("could not deploy contracts")?;

        let generator = BasicPayloadGenerator::new((&chain_key_0).into(), (&contract_instances).into());

        let ad = AnnouncementData::new(
            test_multiaddr,
            Some(KeyBinding::new(
                (&chain_key_0).into(),
                &OffchainKeypair::from_secret(&PRIVATE_KEY)?,
            )),
        )?;

        let tx = generator.announce(ad)?;

        assert!(client.send_transaction(tx).await?.get_receipt().await?.status());

        let test_multiaddr_reannounce = Multiaddr::from_str("/ip4/5.6.7.8/tcp/99")?;

        let ad_reannounce = AnnouncementData::new(test_multiaddr_reannounce, None)?;
        let reannounce_tx = generator.announce(ad_reannounce)?;

        assert!(
            client
                .send_transaction(reannounce_tx)
                .await?
                .get_receipt()
                .await?
                .status()
        );

        Ok(())
    }

    #[tokio::test]
    async fn redeem_ticket() -> anyhow::Result<()> {
        let anvil = hopr_chain_types::utils::create_anvil(None);
        let chain_key_alice = ChainKeypair::from_secret(anvil.keys()[0].to_bytes().as_ref())?;
        let chain_key_bob = ChainKeypair::from_secret(anvil.keys()[1].to_bytes().as_ref())?;
        let client = create_rpc_client_to_anvil(&anvil, &chain_key_alice);

        // Deploy contracts
        let contract_instances = ContractInstances::deploy_for_testing(client.clone(), &chain_key_alice).await?;

        // Mint 1000 HOPR to Alice
        let _ = hopr_chain_types::utils::mint_tokens(contract_instances.token.clone(), U256::from(1000_u128)).await;

        let domain_separator: Hash = contract_instances.channels.domainSeparator().call().await?.0.into();

        // Open channel Alice -> Bob
        let _ = hopr_chain_types::utils::fund_channel(
            (&chain_key_bob).into(),
            contract_instances.token.clone(),
            contract_instances.channels.clone(),
            U256::from(1_u128),
        )
            .await;

        // Fund Bob's node
        let _ = hopr_chain_types::utils::fund_node(
            (&chain_key_bob).into(),
            U256::from(1000000000000000000_u128),
            U256::from(10_u128),
            contract_instances.token.clone(),
        )
            .await;

        let response = Response::try_from(RESPONSE_TO_CHALLENGE.as_ref())?;

        // Alice issues a ticket to Bob
        let ticket = TicketBuilder::default()
            .addresses(&chain_key_alice, &chain_key_bob)
            .amount(1)
            .index(1)
            .index_offset(1)
            .win_prob(1.0.try_into()?)
            .channel_epoch(1)
            .challenge(response.to_challenge()?)
            .build_signed(&chain_key_alice, &domain_separator)?;

        // Bob acknowledges the ticket using the HalfKey from the Response
        let acked_ticket = ticket
            .into_acknowledged(response)
            .into_redeemable(&chain_key_bob, &domain_separator)?;

        // Bob redeems the ticket
        let generator = BasicPayloadGenerator::new((&chain_key_bob).into(), (&contract_instances).into());
        let redeem_ticket_tx = generator.redeem_ticket(acked_ticket)?;
        let client = create_rpc_client_to_anvil(&anvil, &chain_key_bob);

        assert!(
            client
                .send_transaction(redeem_ticket_tx)
                .await?
                .get_receipt()
                .await?
                .status()
        );

        Ok(())
    }

    #[tokio::test]
    async fn withdraw_token() -> anyhow::Result<()> {
        let anvil = hopr_chain_types::utils::create_anvil(None);
        let chain_key_alice = ChainKeypair::from_secret(anvil.keys()[0].to_bytes().as_ref())?;
        let chain_key_bob = ChainKeypair::from_secret(anvil.keys()[1].to_bytes().as_ref())?;
        let client = create_rpc_client_to_anvil(&anvil, &chain_key_alice);

        // Deploy contracts
        let contract_instances = ContractInstances::deploy_for_testing(client.clone(), &chain_key_alice).await?;
        let generator = BasicPayloadGenerator::new((&chain_key_alice).into(), (&contract_instances).into());

        // Mint 1000 HOPR to Alice
        let _ = hopr_chain_types::utils::mint_tokens(contract_instances.token.clone(), U256::from(1000_u128)).await;

        // Check balance is 1000 HOPR
        let balance = contract_instances
            .token
            .balanceOf(hopr_primitive_types::primitives::Address::from(&chain_key_alice).into())
            .call()
            .await?;
        assert_eq!(balance, U256::from(1000_u128));

        // Alice withdraws 100 HOPR (to Bob's address)
        let tx = generator.transfer((&chain_key_bob).into(), HoprBalance::from(100))?;

        assert!(client.send_transaction(tx).await?.get_receipt().await?.status());

        // Alice withdraws 100 HOPR, leaving 900 HOPR to the node
        let balance = contract_instances
            .token
            .balanceOf(hopr_primitive_types::primitives::Address::from(&chain_key_alice).into())
            .call()
            .await?;
        assert_eq!(balance, U256::from(900_u128));

        Ok(())
    }
}
