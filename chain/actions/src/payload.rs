use crate::errors::{
    CoreEthereumActionsError::{InvalidArguments, InvalidState},
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
use core_types::{acknowledgement::AcknowledgedTicket, announcement::AnnouncementData};
use ethers::types::NameOrAddress;
use ethers::{
    abi::AbiEncode,
    types::{H160, H256, U256},
};
use hopr_crypto_types::{keypairs::ChainKeypair, vrf::VrfParameters};
use utils_types::{
    primitives::{Address, Balance, BalanceType},
    traits::BinarySerializable,
};

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Operation {
    Call = 0,
    DelegateCall = 1,
}

/// Trait for various implementations of generators of common on-chain transaction payloads.
pub trait PayloadGenerator<T: Into<TypedTransaction>> {
    /// Create a ERC20 approve transaction payload. Pre-requisite to open payment channels.
    /// The `spender` address is typically the HOPR Channels contract address.
    fn approve(&self, spender: Address, amount: Balance) -> Result<T>;

    /// Create a ERC20 transfer transaction payload
    fn transfer(&self, destination: Address, amount: Balance) -> Result<T>;

    /// Creates the transaction payload to announce a node on-chain.
    fn announce(&self, announcement: AnnouncementData) -> Result<T>;

    /// Creates the transaction payload to open a payment channel
    fn fund_channel(&self, dest: Address, amount: Balance) -> Result<T>;

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
    fn redeem_ticket(&self, acked_ticket: AcknowledgedTicket) -> Result<T>;

    /// Creates a transaction payload to register a Safe instance which is used
    /// to manage the node's funds
    fn register_safe_by_node(&self, safe_addr: Address) -> Result<T>;

    /// Creates a transaction payload to remove the Safe instance. Once succeeded,
    /// the funds are no longer managed by the node.
    fn deregister_node_by_safe(&self) -> Result<T>;
}

fn channels_payload(hopr_channels: Address, call_data: Vec<u8>) -> Vec<u8> {
    ExecTransactionFromModuleCall {
        to: hopr_channels.into(),
        value: U256::zero(),
        data: call_data.into(),
        operation: Operation::Call as u8,
    }
    .encode()
}

fn approve_tx(spender: Address, amount: Balance) -> TypedTransaction {
    let mut tx = create_eip1559_transaction();
    tx.set_data(
        ApproveCall {
            spender: spender.into(),
            value: amount.value().into(),
        }
        .encode()
        .into(),
    );
    tx
}

fn transfer_tx(destination: Address, amount: Balance) -> TypedTransaction {
    let mut tx = create_eip1559_transaction();
    match amount.balance_type() {
        BalanceType::HOPR => {
            tx.set_data(
                TransferCall {
                    recipient: destination.into(),
                    amount: amount.amount().into(),
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

fn register_safe_tx(safe_addr: Address) -> TypedTransaction {
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
    fn approve(&self, spender: Address, amount: Balance) -> Result<TypedTransaction> {
        if amount.balance_type() != BalanceType::HOPR {
            return Err(InvalidArguments(
                "Invalid balance type. Expected a HOPR balance.".into(),
            ));
        }
        let mut tx = approve_tx(spender, amount);
        tx.set_to(NameOrAddress::Address(self.contract_addrs.token.into()));
        Ok(tx)
    }

    fn transfer(&self, destination: Address, amount: Balance) -> Result<TypedTransaction> {
        let mut tx = transfer_tx(destination, amount);
        tx.set_to(H160::from(match amount.balance_type() {
            BalanceType::Native => destination,
            BalanceType::HOPR => self.contract_addrs.channels,
        }));
        Ok(tx)
    }

    fn announce(&self, announcement: AnnouncementData) -> Result<TypedTransaction> {
        let mut tx = create_eip1559_transaction();
        tx.set_data(
            match &announcement.key_binding {
                Some(binding) => {
                    let serialized_signature = binding.signature.to_bytes();

                    BindKeysAnnounceCall {
                        ed_25519_sig_0: H256::from_slice(&serialized_signature[0..32]).into(),
                        ed_25519_sig_1: H256::from_slice(&serialized_signature[32..64]).into(),
                        ed_25519_pub_key: H256::from_slice(&binding.packet_key.to_bytes()).into(),
                        base_multiaddr: announcement.to_multiaddress_str(),
                    }
                    .encode()
                }
                None => AnnounceCall {
                    base_multiaddr: announcement.to_multiaddress_str(),
                }
                .encode(),
            }
            .into(),
        );
        tx.set_to(NameOrAddress::Address(self.contract_addrs.announcements.into()));
        Ok(tx)
    }

    fn fund_channel(&self, dest: Address, amount: Balance) -> Result<TypedTransaction> {
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
                amount: amount.value().as_u128(),
            }
            .encode()
            .into(),
        );
        tx.set_to(NameOrAddress::Address(self.contract_addrs.channels.into()));

        Ok(tx)
    }

    fn close_incoming_channel(&self, source: Address) -> Result<TypedTransaction> {
        if source.eq(&self.me) {
            return Err(InvalidArguments("Cannot close incoming channel from self".into()));
        }

        let mut tx = create_eip1559_transaction();
        tx.set_data(CloseIncomingChannelCall { source: source.into() }.encode().into());
        tx.set_to(NameOrAddress::Address(self.contract_addrs.channels.into()));

        Ok(tx)
    }

    fn initiate_outgoing_channel_closure(&self, destination: Address) -> Result<TypedTransaction> {
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

    fn finalize_outgoing_channel_closure(&self, destination: Address) -> Result<TypedTransaction> {
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

    fn redeem_ticket(&self, acked_ticket: AcknowledgedTicket) -> Result<TypedTransaction> {
        let redeemable = convert_acknowledged_ticket(&acked_ticket)?;
        let params = convert_vrf_parameters(&acked_ticket.vrf_params);

        let mut tx = create_eip1559_transaction();
        tx.set_data(RedeemTicketCall { redeemable, params }.encode().into());
        tx.set_to(NameOrAddress::Address(self.contract_addrs.channels.into()));
        Ok(tx)
    }

    fn register_safe_by_node(&self, safe_addr: Address) -> Result<TypedTransaction> {
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
    fn approve(&self, spender: Address, amount: Balance) -> Result<TypedTransaction> {
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

    fn transfer(&self, destination: Address, amount: Balance) -> Result<TypedTransaction> {
        let mut tx = transfer_tx(destination, amount);
        tx.set_to(NameOrAddress::Address(
            match amount.balance_type() {
                BalanceType::Native => destination,
                BalanceType::HOPR => self.contract_addrs.channels,
            }
            .into(),
        ));
        tx.set_gas(DEFAULT_TX_GAS);

        Ok(tx)
    }

    fn announce(&self, announcement: AnnouncementData) -> Result<TypedTransaction> {
        let call_data = match &announcement.key_binding {
            Some(binding) => {
                let serialized_signature = binding.signature.to_bytes();

                BindKeysAnnounceSafeCall {
                    self_: H160::from_slice(&self.me.to_bytes()),
                    ed_25519_sig_0: H256::from_slice(&serialized_signature[0..32]).into(),
                    ed_25519_sig_1: H256::from_slice(&serialized_signature[32..64]).into(),
                    ed_25519_pub_key: H256::from_slice(&binding.packet_key.to_bytes()).into(),
                    base_multiaddr: announcement.to_multiaddress_str(),
                }
                .encode()
            }
            None => AnnounceSafeCall {
                self_: self.me.into(),
                base_multiaddr: announcement.to_multiaddress_str(),
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

    fn fund_channel(&self, dest: Address, amount: Balance) -> Result<TypedTransaction> {
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
            amount: amount.value().as_u128(),
        }
        .encode();

        let mut tx = create_eip1559_transaction();
        tx.set_data(channels_payload(self.contract_addrs.channels, call_data).into());
        tx.set_to(NameOrAddress::Address(self.module.into()));
        tx.set_gas(DEFAULT_TX_GAS);

        Ok(tx)
    }

    fn close_incoming_channel(&self, source: Address) -> Result<TypedTransaction> {
        if source.eq(&self.me) {
            return Err(InvalidArguments("Cannot close incoming channel from self".into()));
        }

        let call_data = CloseIncomingChannelSafeCall {
            self_: self.me.into(),
            source: source.into(),
        }
        .encode();

        let mut tx = create_eip1559_transaction();
        tx.set_data(channels_payload(self.contract_addrs.channels, call_data).into());
        tx.set_to(NameOrAddress::Address(self.module.into()));
        tx.set_gas(DEFAULT_TX_GAS);

        Ok(tx)
    }

    fn initiate_outgoing_channel_closure(&self, destination: Address) -> Result<TypedTransaction> {
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
        tx.set_data(channels_payload(self.contract_addrs.channels, call_data).into());
        tx.set_to(NameOrAddress::Address(self.module.into()));
        tx.set_gas(DEFAULT_TX_GAS);

        Ok(tx)
    }

    fn finalize_outgoing_channel_closure(&self, destination: Address) -> Result<TypedTransaction> {
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
        tx.set_data(channels_payload(self.contract_addrs.channels, call_data).into());
        tx.set_to(NameOrAddress::Address(self.module.into()));
        tx.set_gas(DEFAULT_TX_GAS);

        Ok(tx)
    }

    fn redeem_ticket(&self, acked_ticket: AcknowledgedTicket) -> Result<TypedTransaction> {
        let redeemable = convert_acknowledged_ticket(&acked_ticket)?;
        let params = convert_vrf_parameters(&acked_ticket.vrf_params);

        let call_data = RedeemTicketSafeCall {
            self_: self.me.into(),
            redeemable,
            params,
        }
        .encode();

        let mut tx = create_eip1559_transaction();
        tx.set_data(channels_payload(self.contract_addrs.channels, call_data).into());
        tx.set_to(NameOrAddress::Address(self.module.into()));
        tx.set_gas(DEFAULT_TX_GAS);

        Ok(tx)
    }

    fn register_safe_by_node(&self, safe_addr: Address) -> Result<TypedTransaction> {
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

/// Convert off-chain representation of VRF parameters to representation
/// that the smart contract understands
///
/// Not implemented using From trait because logic fits better here
pub fn convert_vrf_parameters(off_chain: &VrfParameters) -> Vrfparameters {
    // skip the secp256k1 curvepoint prefix
    let v = off_chain.v.to_bytes();
    let s_b = off_chain.s_b.to_bytes();
    let h_v = off_chain.h_v.to_bytes();

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

/// Convert off-chain representation of acknowledged ticket to representation
/// that the smart contract understands
///
/// Not implemented using From trait because logic fits better here
pub fn convert_acknowledged_ticket(off_chain: &AcknowledgedTicket) -> Result<RedeemableTicket> {
    if let Some(ref signature) = off_chain.ticket.signature {
        let serialized_signature = signature.to_bytes();

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
            por_secret: U256::from_big_endian(&off_chain.response.to_bytes()),
        })
    } else {
        Err(InvalidArguments("Acknowledged ticket must be signed".into()))
    }
}

#[cfg(test)]
pub mod tests {
    use bindings::{hopr_channels::HoprChannels, hopr_token::HoprToken};
    use chain_rpc::client::create_rpc_client_to_anvil;
    use chain_rpc::client::native::SurfRequestor;
    use chain_types::{create_anvil, ContractInstances};
    use core_types::{
        acknowledgement::AcknowledgedTicket,
        announcement::{AnnouncementData, KeyBinding},
        channels::Ticket,
    };
    use ethers::{
        providers::Middleware,
        types::{Bytes, Eip1559TransactionRequest, H160, U256},
    };
    use hex_literal::hex;
    use hopr_crypto_types::{
        keypairs::{ChainKeypair, Keypair, OffchainKeypair},
        types::{Hash, Response},
    };
    use multiaddr::Multiaddr;
    use std::{str::FromStr, sync::Arc};
    use utils_types::primitives::Address;
    use utils_types::{
        primitives::{Address as HoprAddress, Balance, BalanceType, U256 as HoprU256},
        traits::BinarySerializable,
    };

    use super::{BasicPayloadGenerator, PayloadGenerator};

    const PRIVATE_KEY: [u8; 32] = hex!("c14b8faa0a9b8a5fa4453664996f23a7e7de606d42297d723fc4a794f375e260");
    const RESPONSE_TO_CHALLENGE: [u8; 32] = hex!("b58f99c83ae0e7dd6a69f755305b38c7610c7687d2931ff3f70103f8f92b90bb");

    async fn mint_tokens<M: Middleware + 'static>(hopr_token: HoprToken<M>, amount: U256, deployer: Address) {
        hopr_token
            .grant_role(hopr_token.minter_role().await.unwrap(), deployer.into())
            .send()
            .await
            .unwrap();
        hopr_token
            .mint(deployer.into(), amount, Bytes::new(), Bytes::new())
            .send()
            .await
            .unwrap();
    }

    async fn fund_node<M: Middleware>(
        node: &HoprAddress,
        native_token: &U256,
        hopr_token: &U256,
        hopr_token_contract: HoprToken<M>,
        client: Arc<M>,
    ) -> () {
        let node_address = H160::from_slice(&node.to_bytes());
        let native_transfer_tx = Eip1559TransactionRequest::new().to(node_address).value(native_token);
        client
            .send_transaction(native_transfer_tx, None)
            .await
            .unwrap()
            .await
            .unwrap();

        hopr_token_contract
            .transfer(node_address, hopr_token.clone())
            .send()
            .await
            .unwrap()
            .await
            .unwrap();
    }

    async fn fund_channel<M: Middleware>(
        counterparty: Address,
        hopr_token: HoprToken<M>,
        hopr_channels: HoprChannels<M>,
    ) {
        hopr_token
            .approve(hopr_channels.address(), 1u128.into())
            .send()
            .await
            .unwrap()
            .await
            .unwrap();

        hopr_channels
            .fund_channel(counterparty.into(), 1u128)
            .send()
            .await
            .unwrap()
            .await
            .unwrap();
    }

    #[async_std::test]
    async fn test_announce() {
        let test_multiaddr = Multiaddr::from_str("/ip4/1.2.3.4/tcp/56").unwrap();

        let anvil = create_anvil(None);
        let chain_key_0 = ChainKeypair::from_secret(anvil.keys()[0].to_bytes().as_ref()).unwrap();
        let client = create_rpc_client_to_anvil(SurfRequestor::default(), &anvil, &chain_key_0);

        // Deploy contracts
        let contract_instances = ContractInstances::deploy_for_testing(client.clone(), &chain_key_0)
            .await
            .expect("could not deploy contracts");

        let generator = BasicPayloadGenerator::new((&chain_key_0).into(), (&contract_instances).into());

        let ad = AnnouncementData::new(
            &test_multiaddr,
            Some(KeyBinding::new(
                (&chain_key_0).into(),
                &OffchainKeypair::from_secret(&PRIVATE_KEY).unwrap(),
            )),
        )
        .unwrap();

        let tx = generator.announce(ad).expect("should generate tx");

        assert!(client
            .send_transaction(tx, None)
            .await
            .unwrap()
            .await
            .unwrap()
            .is_some());

        let test_multiaddr_reannounce = Multiaddr::from_str("/ip4/5.6.7.8/tcp/99").unwrap();

        let ad_reannounce = AnnouncementData::new(&test_multiaddr_reannounce, None).unwrap();
        let reannounce_tx = generator.announce(ad_reannounce).expect("should generate tx");

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
        let anvil = create_anvil(None);
        let chain_key_alice = ChainKeypair::from_secret(anvil.keys()[0].to_bytes().as_ref()).unwrap();
        let chain_key_bob = ChainKeypair::from_secret(anvil.keys()[1].to_bytes().as_ref()).unwrap();
        let client = create_rpc_client_to_anvil(SurfRequestor::default(), &anvil, &chain_key_alice);

        // Deploy contracts
        let contract_instances = ContractInstances::deploy_for_testing(client.clone(), &chain_key_alice)
            .await
            .expect("could not deploy contracts");

        // Mint 1000 HOPR to Alice
        mint_tokens(contract_instances.token.clone(), 1000.into(), (&chain_key_alice).into()).await;

        let domain_separator: Hash = contract_instances
            .channels
            .domain_separator()
            .call()
            .await
            .unwrap()
            .into();

        // Open channel Alice -> Bob
        fund_channel(
            (&chain_key_bob).into(),
            contract_instances.token.clone(),
            contract_instances.channels.clone(),
        )
        .await;

        // Fund Bob's node
        fund_node(
            &(&chain_key_bob).into(),
            &U256::from(1000000000000000000u128),
            &U256::from(10u64),
            contract_instances.token.clone(),
            client.clone(),
        )
        .await;

        let response = Response::from_bytes(&RESPONSE_TO_CHALLENGE).unwrap();

        // Alice issues a ticket to Bob
        let ticket = Ticket::new(
            &(&chain_key_bob).into(),
            &Balance::new(HoprU256::one(), BalanceType::HOPR),
            HoprU256::one(),
            HoprU256::one(),
            1.0_f64,
            HoprU256::one(),
            response.to_challenge().to_ethereum_challenge(),
            &chain_key_alice,
            &domain_separator,
        )
        .unwrap();

        // Bob acknowledges the ticket using the HalfKey from the Response
        let acked_ticket = AcknowledgedTicket::new(
            ticket,
            response,
            (&chain_key_alice).into(),
            &chain_key_bob,
            &domain_separator,
        )
        .unwrap();

        // Bob redeems the ticket
        let generator = BasicPayloadGenerator::new((&chain_key_bob).into(), (&contract_instances).into());
        let redeem_ticket_tx = generator.redeem_ticket(acked_ticket).expect("should create tx");
        let client = create_rpc_client_to_anvil(SurfRequestor::default(), &anvil, &chain_key_bob);
        println!(
            "{:?}",
            client.send_transaction(redeem_ticket_tx, None).await.unwrap().await
        );
    }
}
