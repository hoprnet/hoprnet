use crate::errors::{
    CoreEthereumError::{InvalidArguments, InvalidState},
    Result,
};
use bindings::{
    hopr_announcements::{BindKeysAnnounceCall, BindKeysAnnounceSafeCall},
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
use core_crypto::{
    keypairs::{ChainKeypair, Keypair, OffchainKeypair},
    types::VrfParameters,
};
use core_types::{account::AccountSignature, acknowledgement::AcknowledgedTicket};
use ethers::{
    abi::AbiEncode,
    types::{Address as EthereumAddress, H160, H256, U256},
};
use multiaddr::Multiaddr;
use std::str::FromStr;
use utils_types::{
    primitives::{Address, Balance, BalanceType},
    traits::{BinarySerializable, PeerIdLike},
};

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Operation {
    Call = 0,
    DelegateCall = 1,
}

pub struct ChainCalls {
    /// used to announce key binding of off-chain key (ed25519)
    /// and on-chain key (secp256k1)
    offchain_keypair: OffchainKeypair,
    /// own Ethereum address
    chain_key: Address,
    /// address of HoprChannels smart contract
    hopr_channels: Address,
    hopr_announcements: Address,
    /// stateful, if set to true, all methods returns
    /// Safe-compliant ABI
    use_safe: bool,
}

impl ChainCalls {
    pub fn new(
        offchain_keypair: &OffchainKeypair,
        chain_keypair: &ChainKeypair,
        hopr_channels: Address,
        hopr_announcements: Address,
    ) -> Self {
        Self {
            offchain_keypair: offchain_keypair.clone(),
            chain_key: chain_keypair.public().to_address(),
            hopr_channels,
            hopr_announcements,
            use_safe: false,
        }
    }

    pub fn set_use_safe(&mut self, enabled: bool) {
        self.use_safe = enabled;
    }

    pub fn get_use_safe(&mut self) -> bool {
        return self.use_safe;
    }

    pub fn announce(&self, announced_multiaddr: &Multiaddr, use_safe: bool) -> Result<Vec<u8>> {
        let account_sig = AccountSignature::new(&self.offchain_keypair, &self.chain_key);

        if let Some(ending) = announced_multiaddr.protocol_stack().last() {
            let expected: String = format!("/p2p/{}", self.offchain_keypair.public().to_peerid_str());
            if ending == "p2p" && !announced_multiaddr.ends_with(&Multiaddr::from_str(expected.as_str())?) {
                return Err(InvalidArguments(format!(
                    "Received a multiaddr with incorrect PeerId, got {} but expected {}",
                    announced_multiaddr, expected
                )));
            }
        }

        let serialized_signature = account_sig.signature.to_bytes();

        if use_safe {
            let call_data = BindKeysAnnounceSafeCall {
                self_: H160::from_slice(&self.chain_key.to_bytes()),
                ed_25519_sig_0: H256::from_slice(&serialized_signature[0..32]).into(),
                ed_25519_sig_1: H256::from_slice(&serialized_signature[32..64]).into(),
                ed_25519_pub_key: H256::from_slice(&self.offchain_keypair.public().to_bytes()).into(),
                base_multiaddr: announced_multiaddr.to_string(),
            }
            .encode();
            Ok(ExecTransactionFromModuleCall {
                to: H160::from_slice(&self.hopr_announcements.to_bytes()),
                value: U256::zero(),
                data: call_data.into(),
                operation: Operation::Call as u8,
            }
            .encode())
        } else {
            Ok(BindKeysAnnounceCall {
                base_multiaddr: announced_multiaddr.to_string(),
                ed_25519_pub_key: H256::from_slice(&self.offchain_keypair.public().to_bytes()).into(),
                ed_25519_sig_0: H256::from_slice(&serialized_signature[0..32]).into(),
                ed_25519_sig_1: H256::from_slice(&serialized_signature[32..64]).into(),
            }
            .encode())
        }
    }

    pub fn approve(&self, amount: &Balance) -> Result<Vec<u8>> {
        if amount.balance_type() != BalanceType::HOPR {
            return Err(InvalidArguments(
                "Invalid balance type. Expected a HOPR balance.".into(),
            ));
        }

        Ok(ApproveCall {
            spender: H160::from_slice(&self.hopr_channels.to_bytes()),
            value: U256::from_big_endian(&amount.value().to_bytes()),
        }
        .encode())
    }

    pub fn transfer(&self, destination: &Address, amount: &Balance) -> Result<Vec<u8>> {
        if amount.balance_type() != BalanceType::HOPR {
            return Err(InvalidArguments("Token transfer must have balance type HOPR".into()));
        }

        Ok(TransferCall {
            recipient: H160::from_slice(&destination.to_bytes()),
            amount: U256::from_big_endian(&amount.amount().to_bytes()),
        }
        .encode())
    }

    pub fn fund_channel(&self, dest: &Address, amount: &Balance) -> Result<Vec<u8>> {
        if dest.eq(&self.chain_key) {
            return Err(InvalidArguments("Cannot fund channel to self".into()));
        }

        if amount.balance_type() != BalanceType::HOPR {
            return Err(InvalidArguments(
                "Invalid balance type. Expected a HOPR balance.".into(),
            ));
        }

        if self.use_safe {
            let call_data = FundChannelSafeCall {
                self_: H160::from_slice(&self.chain_key.to_bytes()),
                account: EthereumAddress::from_slice(&dest.to_bytes()),
                amount: amount.value().as_u128(),
            }
            .encode();
            Ok(ExecTransactionFromModuleCall {
                to: H160::from_slice(&self.hopr_channels.to_bytes()),
                value: U256::zero(),
                data: call_data.into(),
                operation: Operation::Call as u8,
            }
            .encode())
        } else {
            Ok(FundChannelCall {
                account: EthereumAddress::from_slice(&dest.to_bytes()),
                amount: amount.value().as_u128(),
            }
            .encode())
        }
    }

    pub fn close_incoming_channel(&self, source: &Address) -> Result<Vec<u8>> {
        if source.eq(&self.chain_key) {
            return Err(InvalidArguments("Cannot close incoming channe from self".into()));
        }

        if self.use_safe {
            Ok(CloseIncomingChannelSafeCall {
                self_: H160::from_slice(&self.chain_key.to_bytes()),
                source: EthereumAddress::from_slice(&source.to_bytes()),
            }
            .encode())
        } else {
            Ok(CloseIncomingChannelCall {
                source: EthereumAddress::from_slice(&source.to_bytes()),
            }
            .encode())
        }
    }

    pub fn initiate_outgoing_channel_closure(&self, destination: &Address) -> Result<Vec<u8>> {
        if destination.eq(&self.chain_key) {
            return Err(InvalidArguments(
                "Cannot intiate closure of incoming channel to self".into(),
            ));
        }

        if self.use_safe {
            let call_data = InitiateOutgoingChannelClosureSafeCall {
                self_: H160::from_slice(&self.chain_key.to_bytes()),
                destination: EthereumAddress::from_slice(&destination.to_bytes()),
            }
            .encode();
            Ok(ExecTransactionFromModuleCall {
                to: H160::from_slice(&self.hopr_channels.to_bytes()),
                value: U256::zero(),
                data: call_data.into(),
                operation: Operation::Call as u8,
            }
            .encode())
        } else {
            Ok(InitiateOutgoingChannelClosureCall {
                destination: EthereumAddress::from_slice(&destination.to_bytes()),
            }
            .encode())
        }
    }

    pub fn redeem_ticket(&self, acked_ticket: &AcknowledgedTicket) -> Result<Vec<u8>> {
        let redeemable = convert_acknowledged_ticket(acked_ticket)?;
        let params = convert_vrf_parameters(&acked_ticket.vrf_params);

        if self.use_safe {
            let call_data = RedeemTicketSafeCall {
                self_: H160::from_slice(&self.chain_key.to_bytes()),
                redeemable,
                params,
            }
            .encode();
            Ok(ExecTransactionFromModuleCall {
                to: H160::from_slice(&self.hopr_channels.to_bytes()),
                value: U256::zero(),
                data: call_data.into(),
                operation: Operation::Call as u8,
            }
            .encode())
        } else {
            Ok(RedeemTicketCall { redeemable, params }.encode())
        }
    }

    pub fn finalize_outgoing_channel_closure(&self, destination: &Address) -> Result<Vec<u8>> {
        if destination.eq(&self.chain_key) {
            return Err(InvalidArguments(
                "Cannot intiate closure of incoming channel to self".into(),
            ));
        }

        if self.use_safe {
            let call_data = FinalizeOutgoingChannelClosureSafeCall {
                self_: H160::from_slice(&self.chain_key.to_bytes()),
                destination: H160::from_slice(&destination.to_bytes()),
            }
            .encode();
            Ok(ExecTransactionFromModuleCall {
                to: H160::from_slice(&self.hopr_channels.to_bytes()),
                value: U256::zero(),
                data: call_data.into(),
                operation: Operation::Call as u8,
            }
            .encode())
        } else {
            Ok(FinalizeOutgoingChannelClosureCall {
                destination: H160::from_slice(&destination.to_bytes()),
            }
            .encode())
        }
    }

    pub fn register_safe_by_node(&self, safe_addr: &Address) -> Result<Vec<u8>> {
        if safe_addr.eq(&self.chain_key) {
            return Err(InvalidArguments("Safe address must be different from node addr".into()));
        }
        Ok(RegisterSafeByNodeCall {
            safe_addr: H160::from_slice(&safe_addr.to_bytes()),
        }
        .encode())
    }

    pub fn deregister_node_by_safe(&self) -> Result<Vec<u8>> {
        if !self.use_safe {
            return Err(InvalidState(
                "Can only deregister an address if Safe is activated".into(),
            ));
        }

        Ok(DeregisterNodeBySafeCall {
            node_addr: H160::from_slice(&self.chain_key.to_bytes()),
        }
        .encode())
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
        return Err(InvalidArguments("Acknowledged ticket must be signed".into()));
    }
}

#[cfg(test)]
pub mod tests {
    use bindings::{
        hopr_announcements::HoprAnnouncements, hopr_channels::HoprChannels,
        hopr_node_safe_registry::HoprNodeSafeRegistry, hopr_token::HoprToken,
    };
    use core_crypto::{
        keypairs::{ChainKeypair, Keypair, OffchainKeypair},
        types::{Hash, Response},
    };
    use core_types::{acknowledgement::AcknowledgedTicket, channels::Ticket};
    use ethers::{
        abi::{Address, Token},
        core::utils::{Anvil, AnvilInstance},
        middleware::SignerMiddleware,
        prelude::k256::ecdsa::SigningKey,
        providers::{Http, Middleware, Provider},
        signers::Wallet,
        signers::{LocalWallet, Signer},
        types::{transaction::eip2718::TypedTransaction, Bytes, Eip1559TransactionRequest, H160, U256},
    };
    use hex_literal::hex;
    use multiaddr::Multiaddr;
    use std::{path::PathBuf, str::FromStr, sync::Arc};
    use utils_types::{
        primitives::{Address as HoprAddress, Balance, BalanceType, U256 as HoprU256},
        traits::BinarySerializable,
    };

    use super::ChainCalls;

    const PRIVATE_KEY: [u8; 32] = hex!("c14b8faa0a9b8a5fa4453664996f23a7e7de606d42297d723fc4a794f375e260");
    const RESPONSE_TO_CHALLENGE: [u8; 32] = hex!("b58f99c83ae0e7dd6a69f755305b38c7610c7687d2931ff3f70103f8f92b90bb");

    fn get_provider() -> (AnvilInstance, Arc<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>) {
        let anvil: AnvilInstance = Anvil::new()
            .path(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../../.foundry/bin/anvil"))
            .spawn();
        let wallet: LocalWallet = anvil.keys()[0].clone().into();

        let provider = Provider::<Http>::try_from(anvil.endpoint())
            .unwrap()
            .interval(std::time::Duration::from_millis(10u64));

        let client = SignerMiddleware::new(provider, wallet.with_chain_id(anvil.chain_id()));
        let client = Arc::new(client);

        (anvil, client)
    }

    async fn deploy_erc1820(client: Arc<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>) {
        let mut tx = Eip1559TransactionRequest::new();
        tx = tx.to(H160::from_str("a990077c3205cbDf861e17Fa532eeB069cE9fF96").unwrap());
        tx = tx.value(80000000000000000u128);

        client.send_transaction(tx, None).await.unwrap();

        client.send_raw_transaction(
            hex!("f90a388085174876e800830c35008080b909e5608060405234801561001057600080fd5b506109c5806100206000396000f3fe608060405234801561001057600080fd5b50600436106100a5576000357c010000000000000000000000000000000000000000000000000000000090048063a41e7d5111610078578063a41e7d51146101d4578063aabbb8ca1461020a578063b705676514610236578063f712f3e814610280576100a5565b806329965a1d146100aa5780633d584063146100e25780635df8122f1461012457806365ba36c114610152575b600080fd5b6100e0600480360360608110156100c057600080fd5b50600160a060020a038135811691602081013591604090910135166102b6565b005b610108600480360360208110156100f857600080fd5b5035600160a060020a0316610570565b60408051600160a060020a039092168252519081900360200190f35b6100e06004803603604081101561013a57600080fd5b50600160a060020a03813581169160200135166105bc565b6101c26004803603602081101561016857600080fd5b81019060208101813564010000000081111561018357600080fd5b82018360208201111561019557600080fd5b803590602001918460018302840111640100000000831117156101b757600080fd5b5090925090506106b3565b60408051918252519081900360200190f35b6100e0600480360360408110156101ea57600080fd5b508035600160a060020a03169060200135600160e060020a0319166106ee565b6101086004803603604081101561022057600080fd5b50600160a060020a038135169060200135610778565b61026c6004803603604081101561024c57600080fd5b508035600160a060020a03169060200135600160e060020a0319166107ef565b604080519115158252519081900360200190f35b61026c6004803603604081101561029657600080fd5b508035600160a060020a03169060200135600160e060020a0319166108aa565b6000600160a060020a038416156102cd57836102cf565b335b9050336102db82610570565b600160a060020a031614610339576040805160e560020a62461bcd02815260206004820152600f60248201527f4e6f7420746865206d616e616765720000000000000000000000000000000000604482015290519081900360640190fd5b6103428361092a565b15610397576040805160e560020a62461bcd02815260206004820152601a60248201527f4d757374206e6f7420626520616e204552433136352068617368000000000000604482015290519081900360640190fd5b600160a060020a038216158015906103b85750600160a060020a0382163314155b156104ff5760405160200180807f455243313832305f4143434550545f4d4147494300000000000000000000000081525060140190506040516020818303038152906040528051906020012082600160a060020a031663249cb3fa85846040518363ffffffff167c01000000000000000000000000000000000000000000000000000000000281526004018083815260200182600160a060020a0316600160a060020a031681526020019250505060206040518083038186803b15801561047e57600080fd5b505afa158015610492573d6000803e3d6000fd5b505050506040513d60208110156104a857600080fd5b5051146104ff576040805160e560020a62461bcd02815260206004820181905260248201527f446f6573206e6f7420696d706c656d656e742074686520696e74657266616365604482015290519081900360640190fd5b600160a060020a03818116600081815260208181526040808320888452909152808220805473ffffffffffffffffffffffffffffffffffffffff19169487169485179055518692917f93baa6efbd2244243bfee6ce4cfdd1d04fc4c0e9a786abd3a41313bd352db15391a450505050565b600160a060020a03818116600090815260016020526040812054909116151561059a5750806105b7565b50600160a060020a03808216600090815260016020526040902054165b919050565b336105c683610570565b600160a060020a031614610624576040805160e560020a62461bcd02815260206004820152600f60248201527f4e6f7420746865206d616e616765720000000000000000000000000000000000604482015290519081900360640190fd5b81600160a060020a031681600160a060020a0316146106435780610646565b60005b600160a060020a03838116600081815260016020526040808220805473ffffffffffffffffffffffffffffffffffffffff19169585169590951790945592519184169290917f605c2dbf762e5f7d60a546d42e7205dcb1b011ebc62a61736a57c9089d3a43509190a35050565b600082826040516020018083838082843780830192505050925050506040516020818303038152906040528051906020012090505b92915050565b6106f882826107ef565b610703576000610705565b815b600160a060020a03928316600081815260208181526040808320600160e060020a031996909616808452958252808320805473ffffffffffffffffffffffffffffffffffffffff19169590971694909417909555908152600284528181209281529190925220805460ff19166001179055565b600080600160a060020a038416156107905783610792565b335b905061079d8361092a565b156107c357826107ad82826108aa565b6107b85760006107ba565b815b925050506106e8565b600160a060020a0390811660009081526020818152604080832086845290915290205416905092915050565b6000808061081d857f01ffc9a70000000000000000000000000000000000000000000000000000000061094c565b909250905081158061082d575080155b1561083d576000925050506106e8565b61084f85600160e060020a031961094c565b909250905081158061086057508015155b15610870576000925050506106e8565b61087a858561094c565b909250905060018214801561088f5750806001145b1561089f576001925050506106e8565b506000949350505050565b600160a060020a0382166000908152600260209081526040808320600160e060020a03198516845290915281205460ff1615156108f2576108eb83836107ef565b90506106e8565b50600160a060020a03808316600081815260208181526040808320600160e060020a0319871684529091529020549091161492915050565b7bffffffffffffffffffffffffffffffffffffffffffffffffffffffff161590565b6040517f01ffc9a7000000000000000000000000000000000000000000000000000000008082526004820183905260009182919060208160248189617530fa90519096909550935050505056fea165627a7a72305820377f4a2d4301ede9949f163f319021a6e9c687c292a5e2b2c4734c126b524e6c00291ba01820182018201820182018201820182018201820182018201820182018201820a01820182018201820182018201820182018201820182018201820182018201820")
        .into()).await.unwrap();
    }

    async fn deploy_hopr_node_registry(
        client: Arc<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
    ) -> HoprNodeSafeRegistry<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>> {
        HoprNodeSafeRegistry::deploy(client, ()).unwrap().send().await.unwrap()
    }

    async fn deploy_hopr_token_and_mint_tokens(
        client: Arc<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
        amount: U256,
    ) -> HoprToken<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>> {
        let hopr_token = HoprToken::deploy(client.clone(), ()).unwrap().send().await.unwrap();

        hopr_token
            .grant_role(hopr_token.minter_role().await.unwrap(), client.address())
            .send()
            .await
            .unwrap();
        hopr_token
            .mint(client.address(), amount, Bytes::new(), Bytes::new())
            .send()
            .await
            .unwrap();

        hopr_token
    }

    async fn deploy_hopr_channels(
        token: &Address,
        closure_notice_period: u32,
        node_safe_registry: &Address,
        client: Arc<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
    ) -> HoprChannels<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>> {
        HoprChannels::deploy(
            client.clone(),
            Token::Tuple(vec![
                Token::Address(*token),
                Token::Uint(closure_notice_period.into()),
                Token::Address(*node_safe_registry),
            ]),
        )
        .unwrap()
        .send()
        .await
        .unwrap()
    }

    async fn deploy_hopr_announcements(
        hopr_node_safe_registry: &Address,
        client: Arc<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
    ) -> HoprAnnouncements<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>> {
        HoprAnnouncements::deploy(client.clone(), Token::Address(*hopr_node_safe_registry))
            .unwrap()
            .send()
            .await
            .unwrap()
    }

    async fn fund_node(
        node: &HoprAddress,
        native_token: &U256,
        hopr_token: &U256,
        hopr_token_contract: HoprToken<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
        client: Arc<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
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

    async fn fund_channel(
        counterparty: &Address,
        hopr_token: HoprToken<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
        hopr_channels: HoprChannels<SignerMiddleware<Provider<Http>, Wallet<SigningKey>>>,
    ) {
        hopr_token
            .approve(hopr_channels.address(), 1u128.into())
            .send()
            .await
            .unwrap()
            .await
            .unwrap();

        hopr_channels
            .fund_channel(*counterparty, 1u128)
            .send()
            .await
            .unwrap()
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_announce() {
        let (anvil, client) = get_provider();

        let hopr_node_safe_registry = deploy_hopr_node_registry(client.clone()).await;

        let hopr_announcements = deploy_hopr_announcements(&hopr_node_safe_registry.address(), client.clone()).await;

        let test_multiaddr = Multiaddr::from_str("/ip4/1.2.3.4/tcp/56").unwrap();

        deploy_erc1820(client.clone()).await;

        let hopr_node_safe_registry = deploy_hopr_node_registry(client.clone()).await;

        // Mint 1000 Hoprlis
        let hopr_token = deploy_hopr_token_and_mint_tokens(client.clone(), 1000.into()).await;

        let hopr_channels = deploy_hopr_channels(
            &hopr_token.address(),
            32u32,
            &hopr_node_safe_registry.address(),
            client.clone(),
        )
        .await;

        let chain = ChainCalls::new(
            &OffchainKeypair::from_secret(&PRIVATE_KEY).unwrap(),
            &ChainKeypair::from_secret(&anvil.keys()[0].clone().to_bytes().as_slice()).unwrap(),
            HoprAddress::from_bytes(&hopr_channels.address().0).unwrap(),
            HoprAddress::random(),
        );

        let payload = chain.announce(&test_multiaddr, false).unwrap();

        let mut tx = TypedTransaction::Eip1559(Eip1559TransactionRequest::new());

        tx.set_data(payload.into());
        tx.set_to(hopr_announcements.address());

        let receipt = client.send_transaction(tx, None).await.unwrap().await;

        println!("{:?}", receipt);
    }

    #[tokio::test]
    async fn redeem_ticket() {
        let (anvil, client) = get_provider();

        deploy_erc1820(client.clone()).await;

        let hopr_node_safe_registry = deploy_hopr_node_registry(client.clone()).await;

        // Mint 1000 Hoprlis
        let hopr_token = deploy_hopr_token_and_mint_tokens(client.clone(), 1000.into()).await;

        let hopr_channels = deploy_hopr_channels(
            &hopr_token.address(),
            1u32,
            &hopr_node_safe_registry.address(),
            client.clone(),
        )
        .await;

        let keypair = ChainKeypair::from_secret(&anvil.keys()[0].clone().to_bytes().as_slice()).unwrap();
        let chain = ChainCalls::new(
            &OffchainKeypair::from_secret(&PRIVATE_KEY).unwrap(),
            &keypair,
            HoprAddress::from_bytes(&hopr_channels.address().0).unwrap(),
            HoprAddress::random(),
        );

        let counterparty = ChainKeypair::from_secret(&anvil.keys()[1].clone().to_bytes().as_slice()).unwrap();

        let domain_separator: Hash = hopr_channels
            .domain_separator()
            .call()
            .await
            .unwrap()
            .try_into()
            .unwrap();

        fund_channel(
            &H160::from_slice(&counterparty.public().to_address().to_bytes()),
            hopr_token.clone(),
            hopr_channels.clone(),
        )
        .await;

        fund_node(
            &counterparty.public().to_address(),
            &U256::from(1000000000000000000u128),
            &U256::from(10u64),
            hopr_token,
            client.clone(),
        )
        .await;

        let response = Response::from_bytes(&RESPONSE_TO_CHALLENGE).unwrap();

        let ticket = Ticket::new(
            &counterparty.public().to_address(),
            &Balance::new(HoprU256::one(), BalanceType::HOPR),
            HoprU256::one(),
            HoprU256::one(),
            0.7,
            HoprU256::one(),
            response.to_challenge().to_ethereum_challenge(),
            &keypair,
            &domain_separator,
        )
        .unwrap();

        let acked_ticket = AcknowledgedTicket::new(
            ticket,
            response,
            keypair.public().to_address(),
            &counterparty,
            &domain_separator,
        )
        .unwrap();

        let redeem_ticket_tx = Eip1559TransactionRequest::new()
            .from(H160::from_slice(&counterparty.public().to_address().to_bytes()))
            .to(hopr_channels.address())
            .data(chain.redeem_ticket(&acked_ticket).unwrap());

        println!(
            "{:?}",
            client.send_transaction(redeem_ticket_tx, None).await.unwrap().await
        );
    }
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use core_crypto::keypairs::{ChainKeypair, OffchainKeypair};
    use core_types::acknowledgement::wasm::AcknowledgedTicket;
    use multiaddr::Multiaddr;
    use std::str::FromStr;
    use utils_misc::{ok_or_jserr, utils::wasm::JsResult};
    use utils_types::primitives::{Address, Balance};
    use wasm_bindgen::{prelude::*, JsValue};

    #[wasm_bindgen]
    pub struct ChainCalls {
        w: super::ChainCalls,
    }

    #[wasm_bindgen]
    impl ChainCalls {
        #[wasm_bindgen(constructor)]
        pub fn new(
            offchain_keypair: &OffchainKeypair,
            chain_keypair: &ChainKeypair,
            hopr_channels: Address,
            hopr_announcements: Address,
        ) -> Self {
            Self {
                w: super::ChainCalls::new(offchain_keypair, chain_keypair, hopr_channels, hopr_announcements),
            }
        }

        #[wasm_bindgen]
        pub fn set_use_safe(&mut self, enabled: bool) {
            self.w.set_use_safe(enabled)
        }

        #[wasm_bindgen]
        pub fn get_use_safe(&mut self) -> bool {
            self.w.get_use_safe()
        }

        #[wasm_bindgen]
        pub fn get_announce_payload(&self, announced_multiaddr: &str, use_safe: bool) -> JsResult<Vec<u8>> {
            let ma = match Multiaddr::from_str(announced_multiaddr) {
                Ok(ma) => ma,
                Err(e) => return Err(JsValue::from(e.to_string())),
            };
            ok_or_jserr!(self.w.announce(&ma, use_safe))
        }

        #[wasm_bindgen]
        pub fn get_approve_payload(&self, amount: &Balance) -> JsResult<Vec<u8>> {
            ok_or_jserr!(self.w.approve(amount))
        }

        #[wasm_bindgen]
        pub fn get_transfer_payload(&self, dest: &Address, amount: &Balance) -> JsResult<Vec<u8>> {
            ok_or_jserr!(self.w.transfer(dest, amount))
        }

        #[wasm_bindgen]
        pub fn get_fund_channel_payload(&self, dest: &Address, amount: &Balance) -> JsResult<Vec<u8>> {
            ok_or_jserr!(self.w.fund_channel(dest, amount))
        }

        #[wasm_bindgen]
        pub fn get_close_incoming_channel_payload(&self, source: &Address) -> JsResult<Vec<u8>> {
            ok_or_jserr!(self.w.close_incoming_channel(source))
        }

        #[wasm_bindgen]
        pub fn get_intiate_outgoing_channel_closure_payload(&self, dest: &Address) -> JsResult<Vec<u8>> {
            ok_or_jserr!(self.w.initiate_outgoing_channel_closure(dest))
        }

        #[wasm_bindgen]
        pub fn get_finalize_outgoing_channel_closure_payload(&self, dest: &Address) -> JsResult<Vec<u8>> {
            ok_or_jserr!(self.w.finalize_outgoing_channel_closure(dest))
        }

        #[wasm_bindgen]
        pub fn get_redeem_ticket_payload(&self, acked_ticket: &AcknowledgedTicket) -> JsResult<Vec<u8>> {
            ok_or_jserr!(self.w.redeem_ticket(&acked_ticket.into()))
        }

        #[wasm_bindgen]
        pub fn get_register_safe_by_node_payload(&self, safe_addr: &Address) -> JsResult<Vec<u8>> {
            ok_or_jserr!(self.w.register_safe_by_node(safe_addr))
        }

        #[wasm_bindgen]
        pub fn get_deregister_node_by_safe_payload(&self) -> JsResult<Vec<u8>> {
            ok_or_jserr!(self.w.deregister_node_by_safe())
        }
    }
}
