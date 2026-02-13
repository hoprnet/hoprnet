use std::str::FromStr;

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
            b256,
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
    ContractAddresses, a2al,
    errors::{
        ChainTypesError,
        ChainTypesError::{InvalidArguments, InvalidState, SigningError},
    },
    payload,
    payload::{GasEstimation, PayloadGenerator, SignableTransaction},
};

const DEFAULT_TX_GAS: u64 = 400_000;

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

sol! (
    #![sol(all_derives)]
    struct UserData {
        bytes32 functionIdentifier;
        uint256 nonce;
        bytes32 defaultTarget;
        address[] memory admins;
    }
);

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
        chain_id: u64,
        max_gas: Option<GasEstimation>,
        chain_keypair: &ChainKeypair,
    ) -> payload::Result<Box<[u8]>> {
        let max_gas = max_gas.unwrap_or_default();
        let signer: EthereumWallet = PrivateKeySigner::from_slice(chain_keypair.secret().as_ref())
            .map_err(|e| SigningError(e.into()))?
            .into();
        let signed: TxEnvelope = self
            .nonce(nonce)
            .with_chain_id(chain_id)
            .gas_limit(max_gas.gas_limit)
            .max_fee_per_gas(max_gas.max_fee_per_gas)
            .max_priority_fee_per_gas(max_gas.max_priority_fee_per_gas)
            .build(&signer)
            .await
            .map_err(|e| SigningError(e.into()))?;

        Ok(signed.encoded_2718().into_boxed_slice())
    }
}

fn channels_payload(hopr_channels: alloy::primitives::Address, call_data: Vec<u8>) -> Vec<u8> {
    execTransactionFromModuleCall {
        to: hopr_channels,
        value: U256::ZERO,
        data: call_data.into(),
        operation: Operation::Call as u8,
    }
    .abi_encode()
}

fn approve_tx(spender: alloy::primitives::Address, amount: HoprBalance) -> TransactionRequest {
    TransactionRequest::default().with_input(
        approveCall {
            spender,
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
                recipient: a2al(destination),
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
            safeAddr: a2al(safe_addr),
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
        let tx = approve_tx(a2al(spender), amount).with_to(self.contract_addrs.token);
        Ok(tx)
    }

    fn transfer<C: Currency>(&self, destination: Address, amount: Balance<C>) -> payload::Result<Self::TxRequest> {
        let to = if XDai::is::<C>() {
            a2al(destination)
        } else if WxHOPR::is::<C>() {
            self.contract_addrs.token
        } else {
            return Err(InvalidArguments("invalid currency"));
        };
        let tx = transfer_tx(destination, amount).with_to(to);
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
            callerNode: a2al(self.me),
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
            recipient: self.contract_addrs.announcements,
            amount: alloy::primitives::U256::from_be_slice(&key_binding_fee.amount().to_be_bytes()),
            data: inner_payload[32..].to_vec().into(),
        }
        .abi_encode();

        Ok(TransactionRequest::default()
            .with_input(call_data)
            .with_to(self.contract_addrs.token))
    }

    fn fund_channel(&self, dest: Address, amount: HoprBalance) -> payload::Result<Self::TxRequest> {
        if dest.eq(&self.me) {
            return Err(InvalidArguments("Cannot fund channel to self"));
        }

        let tx = TransactionRequest::default()
            .with_input(
                fundChannelCall {
                    account: a2al(dest),
                    amount: U96::from_be_slice(&amount.amount().to_be_bytes()[32 - 12..]),
                }
                .abi_encode(),
            )
            .with_to(self.contract_addrs.channels);
        Ok(tx)
    }

    fn close_incoming_channel(&self, source: Address) -> payload::Result<Self::TxRequest> {
        if source.eq(&self.me) {
            return Err(InvalidArguments("Cannot close incoming channel from self"));
        }

        let tx = TransactionRequest::default()
            .with_input(closeIncomingChannelCall { source: a2al(source) }.abi_encode())
            .with_to(self.contract_addrs.channels);
        Ok(tx)
    }

    fn initiate_outgoing_channel_closure(&self, destination: Address) -> payload::Result<Self::TxRequest> {
        if destination.eq(&self.me) {
            return Err(InvalidArguments("Cannot initiate closure of incoming channel to self"));
        }

        let tx = TransactionRequest::default()
            .with_input(
                initiateOutgoingChannelClosureCall {
                    destination: a2al(destination),
                }
                .abi_encode(),
            )
            .with_to(self.contract_addrs.channels);
        Ok(tx)
    }

    fn finalize_outgoing_channel_closure(&self, destination: Address) -> payload::Result<Self::TxRequest> {
        if destination.eq(&self.me) {
            return Err(InvalidArguments("Cannot initiate closure of incoming channel to self"));
        }

        let tx = TransactionRequest::default()
            .with_input(
                finalizeOutgoingChannelClosureCall {
                    destination: a2al(destination),
                }
                .abi_encode(),
            )
            .with_to(self.contract_addrs.channels);
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
            .with_to(self.contract_addrs.channels);
        Ok(tx)
    }

    fn register_safe_by_node(&self, safe_addr: Address) -> payload::Result<Self::TxRequest> {
        let tx = register_safe_tx(safe_addr).with_to(self.contract_addrs.node_safe_registry);
        Ok(tx)
    }

    fn deregister_node_by_safe(&self) -> payload::Result<Self::TxRequest> {
        Err(InvalidState("Can only deregister an address if Safe is activated"))
    }

    fn deploy_safe(
        &self,
        balance: HoprBalance,
        admins: &[Address],
        include_node: bool,
        nonce: [u8; 32],
    ) -> crate::payload::Result<Self::TxRequest> {
        const DEFAULT_CAPABILITY_PERMISSIONS: &str = "010103030303030303030303";
        let nonce = U256::from_be_slice(&nonce);
        let admins = admins
            .iter()
            .map(|a| alloy::primitives::Address::new((*a).into()))
            .collect::<Vec<_>>();
        let default_target = alloy::primitives::U256::from_str(&format!(
            "{:?}{DEFAULT_CAPABILITY_PERMISSIONS}",
            self.contract_addrs.channels
        ))
        .map_err(|e| ChainTypesError::ParseError(e.into()))?;

        let user_data = if include_node {
            UserData {
                functionIdentifier: b256!("0105b97dcdf19d454ebe36f91ed516c2b90ee79f4a46af96a0138c1f5403c1cc"),
                nonce,
                defaultTarget: default_target.into(),
                admins,
            }
            .abi_encode()[32..]
                .to_vec()
        } else {
            UserData {
                functionIdentifier: b256!("dd24c144db91d1bc600aac99393baf8f8c664ba461188f057e37f2c37b962b45"),
                nonce,
                defaultTarget: default_target.into(),
                admins,
            }
            .abi_encode()[32..]
                .to_vec()
        };

        let tx_payload = sendCall {
            recipient: self.contract_addrs.node_stake_factory,
            amount: alloy::primitives::U256::from_be_slice(&balance.amount().to_be_bytes()),
            data: user_data.into(),
        }
        .abi_encode();

        let tx = TransactionRequest::default()
            .with_to(self.contract_addrs.token)
            .with_input(tx_payload);
        Ok(tx)
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
        let tx = approve_tx(a2al(spender), amount)
            .with_to(self.contract_addrs.token)
            .with_gas_limit(DEFAULT_TX_GAS);

        Ok(tx)
    }

    fn transfer<C: Currency>(&self, destination: Address, amount: Balance<C>) -> payload::Result<Self::TxRequest> {
        let to = if XDai::is::<C>() {
            a2al(destination)
        } else if WxHOPR::is::<C>() {
            self.contract_addrs.token
        } else {
            return Err(InvalidArguments("invalid currency"));
        };
        let tx = transfer_tx(destination, amount)
            .with_to(to)
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
            callerNode: a2al(self.me),
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
            recipient: self.contract_addrs.announcements,
            amount: alloy::primitives::U256::from_be_slice(&key_binding_fee.amount().to_be_bytes()),
            data: inner_payload[32..].to_vec().into(),
        }
        .abi_encode();

        Ok(TransactionRequest::default()
            .with_input(
                execTransactionFromModuleCall {
                    to: self.contract_addrs.token,
                    value: U256::ZERO,
                    data: call_data.into(),
                    operation: Operation::Call as u8,
                }
                .abi_encode(),
            )
            .with_to(a2al(self.module))
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
            selfAddress: a2al(self.me),
            account: a2al(dest),
            amount: U96::from_be_slice(&amount.amount().to_be_bytes()[32 - 12..]),
        }
        .abi_encode();

        let tx = TransactionRequest::default()
            .with_input(channels_payload(self.contract_addrs.channels, call_data))
            .with_to(a2al(self.module))
            .with_gas_limit(DEFAULT_TX_GAS);

        Ok(tx)
    }

    fn close_incoming_channel(&self, source: Address) -> payload::Result<Self::TxRequest> {
        if source.eq(&self.me) {
            return Err(InvalidArguments("Cannot close incoming channel from self"));
        }

        let call_data = closeIncomingChannelSafeCall {
            selfAddress: a2al(self.me),
            source: a2al(source),
        }
        .abi_encode();

        let tx = TransactionRequest::default()
            .with_input(channels_payload(self.contract_addrs.channels, call_data))
            .with_to(a2al(self.module))
            .with_gas_limit(DEFAULT_TX_GAS);

        Ok(tx)
    }

    fn initiate_outgoing_channel_closure(&self, destination: Address) -> payload::Result<Self::TxRequest> {
        if destination.eq(&self.me) {
            return Err(InvalidArguments("Cannot initiate closure of incoming channel to self"));
        }

        let call_data = initiateOutgoingChannelClosureSafeCall {
            selfAddress: a2al(self.me),
            destination: a2al(destination),
        }
        .abi_encode();

        let tx = TransactionRequest::default()
            .with_input(channels_payload(self.contract_addrs.channels, call_data))
            .with_to(a2al(self.module))
            .with_gas_limit(DEFAULT_TX_GAS);

        Ok(tx)
    }

    fn finalize_outgoing_channel_closure(&self, destination: Address) -> payload::Result<Self::TxRequest> {
        if destination.eq(&self.me) {
            return Err(InvalidArguments("Cannot initiate closure of incoming channel to self"));
        }

        let call_data = finalizeOutgoingChannelClosureSafeCall {
            selfAddress: a2al(self.me),
            destination: a2al(destination),
        }
        .abi_encode();

        let tx = TransactionRequest::default()
            .with_input(channels_payload(self.contract_addrs.channels, call_data))
            .with_to(a2al(self.module))
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
            selfAddress: a2al(self.me),
            redeemable,
            params,
        }
        .abi_encode();

        let tx = TransactionRequest::default()
            .with_input(channels_payload(self.contract_addrs.channels, call_data))
            .with_to(a2al(self.module))
            .with_gas_limit(DEFAULT_TX_GAS);

        Ok(tx)
    }

    fn register_safe_by_node(&self, safe_addr: Address) -> payload::Result<Self::TxRequest> {
        let tx = register_safe_tx(safe_addr)
            .with_to(self.contract_addrs.node_safe_registry)
            .with_gas_limit(DEFAULT_TX_GAS);

        Ok(tx)
    }

    fn deregister_node_by_safe(&self) -> payload::Result<Self::TxRequest> {
        let tx = TransactionRequest::default()
            .with_input(
                deregisterNodeBySafeCall {
                    nodeAddr: a2al(self.me),
                }
                .abi_encode(),
            )
            .with_to(a2al(self.module))
            .with_gas_limit(DEFAULT_TX_GAS);

        Ok(tx)
    }

    fn deploy_safe(
        &self,
        _: HoprBalance,
        _: &[Address],
        _: bool,
        _: [u8; 32],
    ) -> crate::payload::Result<Self::TxRequest> {
        Err(InvalidState("cannot deploy Safe from SafePayloadGenerator"))
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
                channelId: B256::from_slice(off_chain.ticket.channel_id().as_ref()),
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
        static ref REDEEMABLE_TICKET: RedeemableTicket = postcard::from_bytes(&hex!(
            "bea83ba0fcee21da44a30c893f466e6bf0c29bbb0530783365387bffffffffffffff010000000000000000000000000000000000000000014038536c412ff92c3b070d98724a2ac167b7a914aa2151cf71eea3d192b0df195d0184aa92c73bccb27aded5f27fcd1cdcf65889f78cf2e62d2f630f659aa2fba220cba79e6dc2ea1205cb76833c9223cd912f056f3406d73d0d689602afe5e88abc668430def9eacd2b5064acf85d73fb0b351a1c8c20d7f3fa28f0caa757e81226e1ee86a9efdbe7991442286183797296ebaa4d292a2005a089ed04b7dbb28ad1c9074f13d10115b0002ca88f4d68ce14549099773c192103d14016cbfa555574e8a5a8fbcb52677dfb7e9267e99c05ebe29603e41b33327705ddecfc569b0125d1ae9a3d3cb637a3c8c9eaafe90e6a1877292227065fbdcc897e95962ce1604fb644782e9029a046650ed84c4f1043b753959d7819f53cec200000000000000000000000000000000000000000000000000000000000000000"
        )).unwrap();
    }

    // Use this to generate the REDEEMABLE_TICKET variable above
    // #[test]
    // fn gen_ticket() -> anyhow::Result<()> {
    // use hopr_crypto_types::crypto_traits::Randomizable;
    //
    // let hk1 = HalfKey::random();
    // let hk2 = HalfKey::random();
    //
    // let ticket = TicketBuilder::default()
    // .counterparty(&ChainKeypair::from_secret(&PRIVATE_KEY_2)?)
    // .amount(1000)
    // .index(123)
    // .channel_epoch(1)
    // .eth_challenge(EthereumChallenge::default())
    // .build_signed(&ChainKeypair::from_secret(&PRIVATE_KEY_1)?, &Default::default())?
    // .into_acknowledged(Response::from_half_keys(&hk1, &hk2)?)
    // .into_redeemable(&&ChainKeypair::from_secret(&PRIVATE_KEY_2)?, &Default::default())?;
    //
    // assert_eq!("", hex::encode(postcard::to_allocvec(&ticket)?));
    // Ok(())
    // }

    #[tokio::test]
    async fn test_announce() -> anyhow::Result<()> {
        let test_multiaddr = Multiaddr::from_str("/ip4/1.2.3.4/tcp/56")?;

        let chain_key_0 = ChainKeypair::from_secret(&PRIVATE_KEY_1)?;

        let generator = BasicPayloadGenerator::new((&chain_key_0).into(), *CONTRACT_ADDRS);

        let kb = KeyBinding::new((&chain_key_0).into(), &OffchainKeypair::from_secret(&PRIVATE_KEY_1)?);

        let ad = AnnouncementData::new(kb, Some(test_multiaddr))?;

        let signed_tx = generator
            .announce(ad, 100_u32.into())?
            .sign_and_encode_to_eip2718(2, 1, None, &chain_key_0)
            .await?;
        insta::assert_snapshot!("announce_basic", hex::encode(signed_tx));

        let test_multiaddr_reannounce = Multiaddr::from_str("/ip4/5.6.7.8/tcp/99")?;
        let ad_reannounce = AnnouncementData::new(kb, Some(test_multiaddr_reannounce))?;

        let signed_tx = generator
            .announce(ad_reannounce, 0_u32.into())?
            .sign_and_encode_to_eip2718(1, 1, None, &chain_key_0)
            .await?;
        insta::assert_snapshot!("announce_safe", hex::encode(signed_tx.clone()));

        Ok(())
    }

    #[tokio::test]
    async fn redeem_ticket_basic() -> anyhow::Result<()> {
        let chain_key_bob = ChainKeypair::from_secret(&PRIVATE_KEY_2)?;

        let acked_ticket = *REDEEMABLE_TICKET;

        // Bob redeems the ticket
        let generator = BasicPayloadGenerator::new((&chain_key_bob).into(), *CONTRACT_ADDRS);
        let redeem_ticket_tx = generator.redeem_ticket(acked_ticket)?;
        let signed_tx = redeem_ticket_tx
            .sign_and_encode_to_eip2718(1, 1, None, &chain_key_bob)
            .await?;

        insta::assert_snapshot!("redeem_ticket_basic", hex::encode(signed_tx));

        Ok(())
    }

    #[tokio::test]
    async fn redeem_ticket_safe() -> anyhow::Result<()> {
        let chain_key_bob = ChainKeypair::from_secret(&PRIVATE_KEY_2)?;

        let acked_ticket = *REDEEMABLE_TICKET;

        // Bob redeems the ticket
        let generator = SafePayloadGenerator::new(&chain_key_bob, *CONTRACT_ADDRS, [1u8; Address::SIZE].into());
        let redeem_ticket_tx = generator.redeem_ticket(acked_ticket)?;
        let signed_tx = redeem_ticket_tx
            .sign_and_encode_to_eip2718(2, 1, None, &chain_key_bob)
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

        let signed_tx = tx.sign_and_encode_to_eip2718(1, 1, None, &chain_key_bob).await?;

        insta::assert_snapshot!("withdraw_basic", hex::encode(signed_tx));

        let generator = SafePayloadGenerator::new(&chain_key_alice, *CONTRACT_ADDRS, [1u8; Address::SIZE].into());
        let tx = generator.transfer((&chain_key_bob).into(), HoprBalance::from(100))?;

        let signed_tx = tx.sign_and_encode_to_eip2718(2, 1, None, &chain_key_bob).await?;

        insta::assert_snapshot!("withdraw_safe", hex::encode(signed_tx));

        Ok(())
    }
}
