//! Module defining various Ethereum transaction payload generators for the actions.
//!
//! This module defines the basic [PayloadGenerator](payload::PayloadGenerator) trait that describes how an action
//! is translated into a [TypedTransaction](chain_types::TypedTransaction) that can be submitted on-chain (via an RPC provider)
//! using a [TransactionExecutor](crate::action_queue::TransactionExecutor).
//!
//! There are two main implementations:
//! - [BasicPayloadGenerator](payload::BasicPayloadGenerator) which implements generation of a direct EIP1559 transaction payload. This is currently
//! not used by a HOPR node.
//! - [SafePayloadGenerator](payload::SafePayloadGenerator) which implements generation of a payload that embeds the transaction data into the
//! SAFE transaction. This is currently the main mode of HOPR node operation.
//!
use crate::errors::{
    ChainActionsError::{InvalidArguments, InvalidState},
    Result,
};
use bindings::{
    hopr_announcements::{AnnounceCall, AnnounceSafeCall, BindKeysAnnounceCall, BindKeysAnnounceSafeCall},
    hopr_channels::{
        CloseIncomingChannelCall, CloseIncomingChannelSafeCall, CompactSignature, FinalizeOutgoingChannelClosureCall,
        FinalizeOutgoingChannelClosureSafeCall, FundChannelCall, FundChannelSafeCall,
        InitiateOutgoingChannelClosureCall, InitiateOutgoingChannelClosureSafeCall, RedeemTicketCall,
        RedeemTicketSafeCall, RedeemableTicket, TicketData, Vrfparameters,
    },
    hopr_node_management_module::ExecTransactionFromModuleCall,
    hopr_node_safe_registry::{DeregisterNodeBySafeCall, RegisterSafeByNodeCall},
    hopr_token::{ApproveCall, TransferCall},
};
use chain_types::ContractAddresses;
use chain_types::{create_eip1559_transaction, TypedTransaction};
use ethers::types::NameOrAddress;
use ethers::{
    abi::AbiEncode,
    types::{H160, H256, U256},
};
use hopr_crypto_types::prelude::*;
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::*;

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Operation {
    Call = 0,
    // Future use: DelegateCall = 1,
}

/// Trait for various implementations of generators of common on-chain transaction payloads.
pub trait PayloadGenerator<T: Into<TypedTransaction>> {
    /// Create a ERC20 approve transaction payload. Pre-requisite to open payment channels.
    /// The `spender` address is typically the HOPR Channels contract address.
    fn approve(&self, spender: &Address, amount: &Balance) -> Result<T>;

    /// Create a ERC20 transfer transaction payload
    fn transfer(&self, destination: &Address, amount: &Balance) -> Result<T>;

    /// Creates the transaction payload to announce a node on-chain.
    fn announce(&self, announcement: &AnnouncementData) -> Result<T>;

    /// Creates the transaction payload to open a payment channel
    fn fund_channel(&self, dest: &Address, amount: &Balance) -> Result<T>;

    /// Creates the transaction payload to immediately close an incoming payment channel
    fn close_incoming_channel(&self, source: &Address) -> Result<T>;

    /// Creates the transaction payload that initiates the closure of a payment channel.
    /// Once the notice period is due, the funds can be withdrawn using a
    /// finalizeChannelClosure transaction.
    fn initiate_outgoing_channel_closure(&self, destination: &Address) -> Result<T>;

    /// Creates a transaction payload that withdraws funds from
    /// an outgoing payment channel. This will succeed once the closure
    /// notice period is due.
    fn finalize_outgoing_channel_closure(&self, destination: &Address) -> Result<T>;

    /// Used to create the payload to claim incentives for relaying a mixnet packet.
    fn redeem_ticket(&self, acked_ticket: &ProvableWinningTicket, domain_separator: &Hash) -> Result<T>;

    /// Creates a transaction payload         chain_key: &ChainKeypair,

    fn register_safe_by_node(&self, safe_addr: &Address) -> Result<T>;

    /// Creates a transaction payload to remove the Safe instance. Once succeeded,
    /// the funds are no longer managed by the node.
    fn deregister_node_by_safe(&self) -> Result<T>;
}

fn channels_payload(hopr_channels: &Address, call_data: Vec<u8>) -> Vec<u8> {
    ExecTransactionFromModuleCall {
        to: hopr_channels.into(),
        value: U256::zero(),
        data: call_data.into(),
        operation: Operation::Call as u8,
    }
    .encode()
}

fn approve_tx(spender: &Address, amount: &Balance) -> TypedTransaction {
    let mut tx = create_eip1559_transaction();
    tx.set_data(
        ApproveCall {
            spender: spender.into(),
            value: amount.amount(),
        }
        .encode()
        .into(),
    );
    tx
}

fn transfer_tx(destination: &Address, amount: &Balance) -> TypedTransaction {
    let mut tx = create_eip1559_transaction();
    match amount.balance_type() {
        BalanceType::HOPR => {
            tx.set_data(
                TransferCall {
                    recipient: destination.into(),
                    amount: amount.amount(),
                }
                .encode()
                .into(),
            );
        }
        BalanceType::Native => {
            tx.set_value(amount.amount());
        }
    }
    tx
}

fn register_safe_tx(safe_addr: &Address) -> TypedTransaction {
    let mut tx = create_eip1559_transaction();
    tx.set_data(
        RegisterSafeByNodeCall {
            safe_addr: safe_addr.into(),
        }
        .encode()
        .into(),
    );
    tx
}

/// Generates transaction payloads that do not use Safe-compliant ABI
#[derive(Debug, Clone)]
pub struct BasicPayloadGenerator {
    me: Address,
    contract_addrs: ContractAddresses,
}

impl BasicPayloadGenerator {
    pub fn new(me: Address, contract_addrs: ContractAddresses) -> Self {
        Self { me, contract_addrs }
    }
}

impl PayloadGenerator<TypedTransaction> for BasicPayloadGenerator {
    fn approve(&self, spender: &Address, amount: &Balance) -> Result<TypedTransaction> {
        if amount.balance_type() != BalanceType::HOPR {
            return Err(InvalidArguments(
                "Invalid balance type. Expected a HOPR balance.".into(),
            ));
        }
        let mut tx = approve_tx(spender, amount);
        tx.set_to(NameOrAddress::Address(self.contract_addrs.token.into()));
        Ok(tx)
    }

    fn transfer(&self, destination: &Address, amount: &Balance) -> Result<TypedTransaction> {
        let mut tx = transfer_tx(destination, amount);
        tx.set_to(H160::from(match amount.balance_type() {
            BalanceType::Native => destination,
            BalanceType::HOPR => &self.contract_addrs.channels,
        }));
        Ok(tx)
    }

    fn announce(&self, announcement: &AnnouncementData) -> Result<TypedTransaction> {
        let mut tx = create_eip1559_transaction();
        tx.set_data(
            match &announcement.key_binding {
                Some(binding) => {
                    let serialized_signature = binding.signature.to_bytes();

                    BindKeysAnnounceCall {
                        ed_25519_sig_0: H256::from_slice(&serialized_signature[0..32]).into(),
                        ed_25519_sig_1: H256::from_slice(&serialized_signature[32..64]).into(),
                        ed_25519_pub_key: H256::from_slice(&binding.packet_key.to_bytes()).into(),
                        base_multiaddr: announcement.multiaddress().to_string(),
                    }
                    .encode()
                }
                None => AnnounceCall {
                    base_multiaddr: announcement.multiaddress().to_string(),
                }
                .encode(),
            }
            .into(),
        );
        tx.set_to(NameOrAddress::Address(self.contract_addrs.announcements.into()));
        Ok(tx)
    }

    fn fund_channel(&self, dest: &Address, amount: &Balance) -> Result<TypedTransaction> {
        if dest.eq(&self.me) {
            return Err(InvalidArguments("Cannot fund channel to self".into()));
        }

        if amount.balance_type() != BalanceType::HOPR {
            return Err(InvalidArguments(
                "Invalid balance type. Expected a HOPR balance.".into(),
            ));
        }

        let mut tx = create_eip1559_transaction();
        tx.set_data(
            FundChannelCall {
                account: dest.into(),
                amount: amount.amount().as_u128(),
            }
            .encode()
            .into(),
        );
        tx.set_to(NameOrAddress::Address(self.contract_addrs.channels.into()));

        Ok(tx)
    }

    fn close_incoming_channel(&self, source: &Address) -> Result<TypedTransaction> {
        if source.eq(&self.me) {
            return Err(InvalidArguments("Cannot close incoming channel from self".into()));
        }

        let mut tx = create_eip1559_transaction();
        tx.set_data(CloseIncomingChannelCall { source: source.into() }.encode().into());
        tx.set_to(NameOrAddress::Address(self.contract_addrs.channels.into()));

        Ok(tx)
    }

    fn initiate_outgoing_channel_closure(&self, destination: &Address) -> Result<TypedTransaction> {
        if destination.eq(&self.me) {
            return Err(InvalidArguments(
                "Cannot initiate closure of incoming channel to self".into(),
            ));
        }

        let mut tx = create_eip1559_transaction();
        tx.set_data(
            InitiateOutgoingChannelClosureCall {
                destination: destination.into(),
            }
            .encode()
            .into(),
        );
        tx.set_to(NameOrAddress::Address(self.contract_addrs.channels.into()));

        Ok(tx)
    }

    fn finalize_outgoing_channel_closure(&self, destination: &Address) -> Result<TypedTransaction> {
        if destination.eq(&self.me) {
            return Err(InvalidArguments(
                "Cannot initiate closure of incoming channel to self".into(),
            ));
        }

        let mut tx = create_eip1559_transaction();
        tx.set_data(
            FinalizeOutgoingChannelClosureCall {
                destination: destination.into(),
            }
            .encode()
            .into(),
        );
        tx.set_to(NameOrAddress::Address(self.contract_addrs.channels.into()));
        Ok(tx)
    }

    fn redeem_ticket(
        &self,
        winning_ticket: &ProvableWinningTicket,
        domain_separator: &Hash,
    ) -> Result<TypedTransaction> {
        let redeemable = convert_winning_ticket(winning_ticket)?;

        let ticket_hash = winning_ticket.ticket.get_hash(domain_separator);

        let params = convert_vrf_parameters(&winning_ticket.vrf_params, &self.me, &ticket_hash, domain_separator);
        let mut tx = create_eip1559_transaction();
        tx.set_data(RedeemTicketCall { redeemable, params }.encode().into());
        tx.set_to(NameOrAddress::Address(self.contract_addrs.channels.into()));
        Ok(tx)
    }

    fn register_safe_by_node(&self, safe_addr: &Address) -> Result<TypedTransaction> {
        let mut tx = register_safe_tx(safe_addr);
        tx.set_to(NameOrAddress::Address(self.contract_addrs.safe_registry.into()));
        Ok(tx)
    }

    fn deregister_node_by_safe(&self) -> Result<TypedTransaction> {
        Err(InvalidState(
            "Can only deregister an address if Safe is activated".into(),
        ))
    }
}

/// Payload generator that generates Safe-compliant ABI
#[derive(Debug, Clone)]
pub struct SafePayloadGenerator {
    me: Address,
    contract_addrs: ContractAddresses,
    module: Address,
}

pub const DEFAULT_TX_GAS: u64 = 400_000;

impl SafePayloadGenerator {
    pub fn new(chain_keypair: &ChainKeypair, contract_addrs: ContractAddresses, module: Address) -> Self {
        Self {
            me: chain_keypair.into(),
            contract_addrs,
            module,
        }
    }
}

impl PayloadGenerator<TypedTransaction> for SafePayloadGenerator {
    fn approve(&self, spender: &Address, amount: &Balance) -> Result<TypedTransaction> {
        if amount.balance_type() != BalanceType::HOPR {
            return Err(InvalidArguments(
                "Invalid balance type. Expected a HOPR balance.".into(),
            ));
        }
        let mut tx = approve_tx(spender, amount);
        tx.set_to(NameOrAddress::Address(self.contract_addrs.token.into()));
        tx.set_gas(DEFAULT_TX_GAS);

        Ok(tx)
    }

    fn transfer(&self, destination: &Address, amount: &Balance) -> Result<TypedTransaction> {
        let mut tx = transfer_tx(destination, amount);
        tx.set_to(NameOrAddress::Address(
            match amount.balance_type() {
                BalanceType::Native => destination,
                BalanceType::HOPR => &self.contract_addrs.channels,
            }
            .into(),
        ));
        tx.set_gas(DEFAULT_TX_GAS);

        Ok(tx)
    }

    fn announce(&self, announcement: &AnnouncementData) -> Result<TypedTransaction> {
        let call_data = match &announcement.key_binding {
            Some(binding) => {
                let serialized_signature = binding.signature.to_bytes();

                BindKeysAnnounceSafeCall {
                    self_: H160::from_slice(&self.me.to_bytes()),
                    ed_25519_sig_0: H256::from_slice(&serialized_signature[0..32]).into(),
                    ed_25519_sig_1: H256::from_slice(&serialized_signature[32..64]).into(),
                    ed_25519_pub_key: H256::from_slice(&binding.packet_key.to_bytes()).into(),
                    base_multiaddr: announcement.multiaddress().to_string(),
                }
                .encode()
            }
            None => AnnounceSafeCall {
                self_: self.me.into(),
                base_multiaddr: announcement.multiaddress().to_string(),
            }
            .encode(),
        };

        let mut tx = create_eip1559_transaction();
        tx.set_data(
            ExecTransactionFromModuleCall {
                to: self.contract_addrs.announcements.into(),
                value: U256::zero(),
                data: call_data.into(),
                operation: Operation::Call as u8,
            }
            .encode()
            .into(),
        );
        tx.set_to(NameOrAddress::Address(self.module.into()));
        tx.set_gas(DEFAULT_TX_GAS);

        Ok(tx)
    }

    fn fund_channel(&self, dest: &Address, amount: &Balance) -> Result<TypedTransaction> {
        if dest.eq(&self.me) {
            return Err(InvalidArguments("Cannot fund channel to self".into()));
        }

        if amount.balance_type() != BalanceType::HOPR {
            return Err(InvalidArguments(
                "Invalid balance type. Expected a HOPR balance.".into(),
            ));
        }

        let call_data = FundChannelSafeCall {
            self_: self.me.into(),
            account: dest.into(),
            amount: amount.amount().as_u128(),
        }
        .encode();

        let mut tx = create_eip1559_transaction();
        tx.set_data(channels_payload(&self.contract_addrs.channels, call_data).into());
        tx.set_to(NameOrAddress::Address(self.module.into()));
        tx.set_gas(DEFAULT_TX_GAS);

        Ok(tx)
    }

    fn close_incoming_channel(&self, source: &Address) -> Result<TypedTransaction> {
        if source.eq(&self.me) {
            return Err(InvalidArguments("Cannot close incoming channel from self".into()));
        }

        let call_data = CloseIncomingChannelSafeCall {
            self_: self.me.into(),
            source: source.into(),
        }
        .encode();

        let mut tx = create_eip1559_transaction();
        tx.set_data(channels_payload(&self.contract_addrs.channels, call_data).into());
        tx.set_to(NameOrAddress::Address(self.module.into()));
        tx.set_gas(DEFAULT_TX_GAS);

        Ok(tx)
    }

    fn initiate_outgoing_channel_closure(&self, destination: &Address) -> Result<TypedTransaction> {
        if destination.eq(&self.me) {
            return Err(InvalidArguments(
                "Cannot initiate closure of incoming channel to self".into(),
            ));
        }

        let call_data = InitiateOutgoingChannelClosureSafeCall {
            self_: self.me.into(),
            destination: destination.into(),
        }
        .encode();

        let mut tx = create_eip1559_transaction();
        tx.set_data(channels_payload(&self.contract_addrs.channels, call_data).into());
        tx.set_to(NameOrAddress::Address(self.module.into()));
        tx.set_gas(DEFAULT_TX_GAS);

        Ok(tx)
    }

    fn finalize_outgoing_channel_closure(&self, destination: &Address) -> Result<TypedTransaction> {
        if destination.eq(&self.me) {
            return Err(InvalidArguments(
                "Cannot initiate closure of incoming channel to self".into(),
            ));
        }

        let call_data = FinalizeOutgoingChannelClosureSafeCall {
            self_: self.me.into(),
            destination: destination.into(),
        }
        .encode();

        let mut tx = create_eip1559_transaction();
        tx.set_data(channels_payload(&self.contract_addrs.channels, call_data).into());
        tx.set_to(NameOrAddress::Address(self.module.into()));
        tx.set_gas(DEFAULT_TX_GAS);

        Ok(tx)
    }

    fn redeem_ticket(
        &self,
        winning_ticket: &ProvableWinningTicket,
        domain_separator: &Hash,
    ) -> Result<TypedTransaction> {
        let redeemable = convert_winning_ticket(winning_ticket)?;

        let ticket_hash = winning_ticket.ticket.get_hash(domain_separator);

        let params = convert_vrf_parameters(&winning_ticket.vrf_params, &self.me, &ticket_hash, domain_separator);

        let call_data = RedeemTicketSafeCall {
            self_: self.me.into(),
            redeemable,
            params,
        }
        .encode();

        let mut tx = create_eip1559_transaction();
        tx.set_data(channels_payload(&self.contract_addrs.channels, call_data).into());
        tx.set_to(NameOrAddress::Address(self.module.into()));
        tx.set_gas(DEFAULT_TX_GAS);

        Ok(tx)
    }

    fn register_safe_by_node(&self, safe_addr: &Address) -> Result<TypedTransaction> {
        let mut tx = register_safe_tx(safe_addr);
        tx.set_to(NameOrAddress::Address(self.contract_addrs.safe_registry.into()));
        tx.set_gas(DEFAULT_TX_GAS);

        Ok(tx)
    }

    fn deregister_node_by_safe(&self) -> Result<TypedTransaction> {
        let mut tx = create_eip1559_transaction();
        tx.set_data(
            DeregisterNodeBySafeCall {
                node_addr: self.me.into(),
            }
            .encode()
            .into(),
        );
        tx.set_to(NameOrAddress::Address(self.module.into()));
        tx.set_gas(DEFAULT_TX_GAS);

        Ok(tx)
    }
}

/// Converts off-chain representation of VRF parameters into a representation
/// that the smart contract understands
///
/// Specific for smart contract payload generation
pub fn convert_vrf_parameters(
    off_chain: &VrfParameters,
    signer: &Address,
    ticket_hash: &Hash,
    domain_separator: &Hash,
) -> Vrfparameters {
    // skip the secp256k1 curvepoint prefix
    let v = off_chain.get_decompressed_v().unwrap().to_bytes();
    let s_b = off_chain
        .get_s_b_witness(signer, &ticket_hash.into(), domain_separator.as_slice())
        .to_bytes();

    let h_v = off_chain.get_h_v_witness().to_bytes();

    Vrfparameters {
        vx: U256::from_big_endian(&v[1..33]),
        vy: U256::from_big_endian(&v[33..65]),
        s: U256::from_big_endian(&off_chain.s.to_bytes()),
        h: U256::from_big_endian(&off_chain.h.to_bytes()),
        s_bx: U256::from_big_endian(&s_b[1..33]),
        s_by: U256::from_big_endian(&s_b[33..65]),
        h_vx: U256::from_big_endian(&h_v[1..33]),
        h_vy: U256::from_big_endian(&h_v[33..65]),
    }
}

/// Convert off-chain representation of a winning ticket to a representation
/// that is understood by the smart contract
///
/// Specific for smart contract payload generation
pub fn convert_winning_ticket(off_chain: &ProvableWinningTicket) -> Result<RedeemableTicket> {
    let serialized_signature = off_chain.ticket.signature.to_bytes();

    let mut encoded_win_prob = [0u8; 8];
    encoded_win_prob[1..].copy_from_slice(&off_chain.ticket.encoded_win_prob);

    Ok(RedeemableTicket {
        data: TicketData {
            channel_id: off_chain.ticket.channel_id.into(),
            amount: off_chain.ticket.amount.amount().as_u128(),
            ticket_index: off_chain.ticket.index,
            index_offset: off_chain.ticket.index_offset,
            epoch: off_chain.ticket.channel_epoch,
            win_prob: u64::from_be_bytes(encoded_win_prob),
        },
        signature: CompactSignature {
            r: H256::from_slice(&serialized_signature[0..32]).into(),
            vs: H256::from_slice(&serialized_signature[32..64]).into(),
        },
        por_secret: U256::from_big_endian(off_chain.response.as_slice()),
    })
}

#[cfg(test)]
pub mod tests {
    use chain_rpc::client::create_rpc_client_to_anvil;
    use chain_rpc::client::native::SurfRequestor;
    use chain_types::ContractInstances;
    use ethers::providers::Middleware;
    use hex_literal::hex;
    use hopr_crypto_types::prelude::*;
    use hopr_internal_types::prelude::*;
    use hopr_primitive_types::prelude::*;
    use multiaddr::Multiaddr;
    use std::str::FromStr;

    use super::{BasicPayloadGenerator, PayloadGenerator};

    const PRIVATE_KEY: [u8; 32] = hex!("c14b8faa0a9b8a5fa4453664996f23a7e7de606d42297d723fc4a794f375e260");
    const RESPONSE_TO_CHALLENGE: [u8; 32] = hex!("b58f99c83ae0e7dd6a69f755305b38c7610c7687d2931ff3f70103f8f92b90bb");

    #[async_std::test]
    async fn test_announce() {
        let test_multiaddr = Multiaddr::from_str("/ip4/1.2.3.4/tcp/56").unwrap();

        let anvil = chain_types::utils::create_anvil(None);
        let chain_key_0 = ChainKeypair::from_secret(anvil.keys()[0].to_bytes().as_ref()).unwrap();
        let client = create_rpc_client_to_anvil(SurfRequestor::default(), &anvil, &chain_key_0);

        // Deploy contracts
        let contract_instances = ContractInstances::deploy_for_testing(client.clone(), &chain_key_0)
            .await
            .expect("could not deploy contracts");

        let generator = BasicPayloadGenerator::new((&chain_key_0).into(), (&contract_instances).into());

        let ad = AnnouncementData::new(
            test_multiaddr,
            Some(KeyBinding::new(
                (&chain_key_0).into(),
                &OffchainKeypair::from_secret(&PRIVATE_KEY).unwrap(),
            )),
        )
        .unwrap();

        let tx = generator.announce(&ad).expect("should generate tx");

        assert!(client
            .send_transaction(tx, None)
            .await
            .unwrap()
            .await
            .unwrap()
            .is_some());

        let test_multiaddr_reannounce = Multiaddr::from_str("/ip4/5.6.7.8/tcp/99").unwrap();

        let ad_reannounce = AnnouncementData::new(test_multiaddr_reannounce, None).unwrap();
        let reannounce_tx = generator.announce(&ad_reannounce).expect("should generate tx");

        assert!(client
            .send_transaction(reannounce_tx, None)
            .await
            .unwrap()
            .await
            .unwrap()
            .is_some());
    }

    #[async_std::test]
    async fn redeem_ticket() {
        let anvil = chain_types::utils::create_anvil(None);
        let chain_key_alice = ChainKeypair::from_secret(anvil.keys()[0].to_bytes().as_ref()).unwrap();
        let chain_key_bob = ChainKeypair::from_secret(anvil.keys()[1].to_bytes().as_ref()).unwrap();
        let client = create_rpc_client_to_anvil(SurfRequestor::default(), &anvil, &chain_key_alice);

        // Deploy contracts
        let contract_instances = ContractInstances::deploy_for_testing(client.clone(), &chain_key_alice)
            .await
            .expect("could not deploy contracts");

        // Mint 1000 HOPR to Alice
        chain_types::utils::mint_tokens(contract_instances.token.clone(), 1000_u128.into()).await;

        let domain_separator: Hash = contract_instances
            .channels
            .domain_separator()
            .call()
            .await
            .unwrap()
            .into();

        // Open channel Alice -> Bob
        chain_types::utils::fund_channel(
            (&chain_key_bob).into(),
            contract_instances.token.clone(),
            contract_instances.channels.clone(),
            1_u128.into(),
        )
        .await;

        // Fund Bob's node
        chain_types::utils::fund_node(
            (&chain_key_bob).into(),
            1000000000000000000_u128.into(),
            10_u128.into(),
            contract_instances.token.clone(),
        )
        .await;

        let response = Response::from_bytes(&RESPONSE_TO_CHALLENGE).unwrap();

        // Alice issues a ticket to Bob
        let ticket = Ticket::new(
            &(&chain_key_bob).into(),
            &Balance::new(U256::one(), BalanceType::HOPR),
            U256::one(),
            U256::one(),
            1.0_f64,
            U256::one(),
            &response.to_challenge().to_ethereum_challenge(),
            &chain_key_alice,
            &domain_separator,
        );

        assert!(validate_ticket(&ticket, &(&chain_key_bob).into(), &domain_separator).is_ok());

        // Bob acknowledges the ticket using the HalfKey from the Response
        let acked_ticket = AcknowledgedTicket::new(ticket, response);

        assert!(validate_acknowledged_ticket(&acked_ticket).is_ok());

        // Expand acknowledged ticket
        let winning_ticket =
            ProvableWinningTicket::from_acked_ticket(acked_ticket, &chain_key_bob, &domain_separator).unwrap();

        assert!(validate_provable_winning_ticket(&winning_ticket, &(&chain_key_bob).into(), &domain_separator).is_ok());

        // Bob redeems the ticket
        let generator = BasicPayloadGenerator::new((&chain_key_bob).into(), (&contract_instances).into());
        let redeem_ticket_tx = generator
            .redeem_ticket(&winning_ticket, &domain_separator)
            .expect("should create tx");
        let client = create_rpc_client_to_anvil(SurfRequestor::default(), &anvil, &chain_key_bob);
        println!(
            "{:?}",
            client.send_transaction(redeem_ticket_tx, None).await.unwrap().await
        );
    }
}
