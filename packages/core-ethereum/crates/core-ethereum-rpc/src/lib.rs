use async_trait::async_trait;
use core_crypto::types::Hash;
use primitive_types::H256;
use std::fmt::{Display, Formatter};
use utils_types::primitives::{Address, Balance, BalanceType, U256};
use utils_types::traits::BinarySerializable;

use crate::errors::Result;

pub use ethers::types::transaction::eip2718::TypedTransaction;
pub use ethers::types::TxHash;
pub use futures::channel::mpsc::UnboundedReceiver;

pub mod errors;
pub mod indexer;
pub mod rpc;

#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
pub mod nodejs;

/// A type containing selected fields from  the `eth_getBlockByHash`/`eth_getBlockByNumber` RPC
/// calls.
#[derive(Debug, Clone)]
pub struct Block {
    /// Block number
    pub number: Option<u64>,
    /// Block hash if any.
    pub hash: Option<Hash>,
    /// Block timestamp
    pub timestamp: U256,
    /// Transaction hashes within this block
    pub transactions: Vec<Hash>,
}

impl Display for Block {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} (@ {}) with {} txs",
            self.number
                .map(|i| format!("block #{i}"))
                .unwrap_or("pending block".into()),
            self.timestamp.as_u64(),
            self.transactions.len()
        )
    }
}

impl From<ethers::types::Block<H256>> for Block {
    fn from(value: ethers::prelude::Block<H256>) -> Self {
        Self {
            number: value.number.map(|u| u.as_u64()),
            hash: value.hash.map(|h| h.0.into()),
            timestamp: value.timestamp.into(),
            transactions: value.transactions.into_iter().map(|h| Hash::from(h.0)).collect(),
        }
    }
}

/// A type containing selected fields from  the `eth_getLogs` RPC calls.
#[derive(Debug, Clone)]
pub struct Log {
    /// Contract address
    pub address: Address,
    /// Topics
    pub topics: Vec<Hash>,
    /// Raw log data
    pub data: Box<[u8]>,
    /// Transaction index
    pub tx_index: Option<u64>,
    /// Corresponding block number
    pub block_number: Option<u64>,
    /// Log index
    pub log_index: Option<U256>,
}

impl From<ethers::types::Log> for Log {
    fn from(value: ethers::prelude::Log) -> Self {
        Self {
            address: value.address.into(),
            topics: value.topics.into_iter().map(|h| Hash::from(h.0)).collect(),
            data: Box::from(value.data.as_ref()),
            tx_index: value.transaction_index.map(|u| u.as_u64()),
            block_number: value.block_number.map(|u| u.as_u64()),
            log_index: value.log_index.map(|u| u.into()),
        }
    }
}

impl From<Log> for ethers::abi::RawLog {
    fn from(value: Log) -> Self {
        ethers::abi::RawLog {
            topics: value.topics.iter().map(|h| H256::from_slice(&h.to_bytes())).collect(),
            data: value.data.into(),
        }
    }
}

impl Display for Log {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "log of {} with {} topics", self.address, self.topics.len())
    }
}

/// Represents a mined block optionally with filtered logs (according to some `LogFilter`)
/// corresponding to the block.
#[derive(Debug, Clone)]
pub struct BlockWithLogs {
    /// Block with TX hashes.
    pub block: Block,
    /// Filtered logs of interest corresponding to the block, if any filtering was requested.
    pub logs: Vec<Log>,
}

impl Display for BlockWithLogs {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} and {} logs", self.block, self.logs.len())
    }
}

/// Represents a filter to extract logs containing specific contract events from a block.
#[derive(Debug, Clone)]
pub struct LogFilter {
    /// Contract addresses
    pub address: Vec<Address>,
    /// Event topics
    pub topics: Vec<TxHash>,
}

impl LogFilter {
    /// Indicates if this filter filters anything.
    pub fn is_empty(&self) -> bool {
        self.address.is_empty() && self.topics.is_empty()
    }
}

impl Display for LogFilter {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "filter of {} contracts with {} topics",
            self.address.len(),
            self.topics.len()
        )
    }
}

impl From<LogFilter> for ethers::types::Filter {
    fn from(value: LogFilter) -> Self {
        ethers::types::Filter::new()
            .address(
                value
                    .address
                    .into_iter()
                    .map(ethers::types::Address::from)
                    .collect::<Vec<_>>(),
            )
            .topic0(value.topics)
    }
}

/// Trait defining general set of operations an RPC provider
/// must provide to the HOPR node.
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait HoprRpcOperations {
    /// Retrieves the timestamp from the given block number.
    async fn get_timestamp(&self, block_number: u64) -> Result<Option<u64>>;

    /// Retrieves the node's account balance of the given type.
    async fn get_balance(&self, balance_type: BalanceType) -> Result<Balance>;

    /// Retrieves info of the given node module's target.
    async fn get_node_management_module_target_info(&self, target: Address) -> Result<Option<U256>>;

    /// Retrieves safe address of the given node address from the registry.
    async fn get_safe_from_node_safe_registry(&self, node: Address) -> Result<Address>;

    /// Retrieves target address of the node module.
    async fn get_module_target_address(&self) -> Result<Address>;

    /// Sends transaction to the RPC provider.
    async fn send_transaction(&self, tx: TypedTransaction) -> Result<Hash>;
}

/// Trait with RPC provider functionality required by the Indexer.
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait HoprIndexerRpcOperations {
    /// Retrieves the latest block number.
    async fn block_number(&self) -> Result<u64>;

    /// Starts streaming the blocks with logs from the given `start_block_number`.
    /// If no `start_block_number` is given, the stream starts from the latest block.
    /// The given `filter` are applied to retrieve the logs for each retrieved block.
    /// If the filter `is_empty()`, no logs are fetched, only blocks.
    /// The streaming stops only when the corresponding channel is closed by the returned receiver.
    async fn try_block_with_logs_stream(
        &self,
        start_block_number: Option<u64>,
        filter: LogFilter,
    ) -> Result<UnboundedReceiver<BlockWithLogs>>;
}

#[cfg(test)]
mod tests {
    use bindings::{
        hopr_announcements::HoprAnnouncements, hopr_channels::HoprChannels,
        hopr_dummy_proxy_for_network_registry::HoprDummyProxyForNetworkRegistry,
        hopr_network_registry::HoprNetworkRegistry, hopr_node_management_module::HoprNodeManagementModule,
        hopr_node_safe_registry::HoprNodeSafeRegistry, hopr_node_stake_factory::HoprNodeStakeFactory,
        hopr_ticket_price_oracle::HoprTicketPriceOracle, hopr_token::HoprToken,
    };
    use core_crypto::keypairs::{ChainKeypair, Keypair};
    use core_crypto::types::PublicKey;
    use core_ethereum_misc::ContractAddresses;
    use ethers::prelude::*;
    use ethers::{
        abi::Token,
        types::U256,
        utils::{Anvil, AnvilInstance},
    };
    use hex_literal::hex;
    use std::path::PathBuf;
    use std::str::FromStr;
    use std::sync::Arc;
    use std::time::Duration;
    use utils_log::debug;
    use utils_types::traits::BinarySerializable;

    use crate::rpc::tests::mock_config;
    use crate::rpc::RpcOperations;

    pub async fn deploy_contracts<M: Middleware + 'static>(client: Arc<M>, anvil: &AnvilInstance) -> ContractAddresses {
        debug!("deploying contracts...");

        // Fund 1820 deployer and deploy ERC1820Registry
        let mut tx = Eip1559TransactionRequest::new();
        tx = tx.to(H160::from_str("a990077c3205cbDf861e17Fa532eeB069cE9fF96").unwrap());
        tx = tx.value(80000000000000000u128);

        let tx = client.send_transaction(tx, None).await.unwrap();
        tx.await.unwrap();

        client.send_raw_transaction(
            hex!("f90a388085174876e800830c35008080b909e5608060405234801561001057600080fd5b506109c5806100206000396000f3fe608060405234801561001057600080fd5b50600436106100a5576000357c010000000000000000000000000000000000000000000000000000000090048063a41e7d5111610078578063a41e7d51146101d4578063aabbb8ca1461020a578063b705676514610236578063f712f3e814610280576100a5565b806329965a1d146100aa5780633d584063146100e25780635df8122f1461012457806365ba36c114610152575b600080fd5b6100e0600480360360608110156100c057600080fd5b50600160a060020a038135811691602081013591604090910135166102b6565b005b610108600480360360208110156100f857600080fd5b5035600160a060020a0316610570565b60408051600160a060020a039092168252519081900360200190f35b6100e06004803603604081101561013a57600080fd5b50600160a060020a03813581169160200135166105bc565b6101c26004803603602081101561016857600080fd5b81019060208101813564010000000081111561018357600080fd5b82018360208201111561019557600080fd5b803590602001918460018302840111640100000000831117156101b757600080fd5b5090925090506106b3565b60408051918252519081900360200190f35b6100e0600480360360408110156101ea57600080fd5b508035600160a060020a03169060200135600160e060020a0319166106ee565b6101086004803603604081101561022057600080fd5b50600160a060020a038135169060200135610778565b61026c6004803603604081101561024c57600080fd5b508035600160a060020a03169060200135600160e060020a0319166107ef565b604080519115158252519081900360200190f35b61026c6004803603604081101561029657600080fd5b508035600160a060020a03169060200135600160e060020a0319166108aa565b6000600160a060020a038416156102cd57836102cf565b335b9050336102db82610570565b600160a060020a031614610339576040805160e560020a62461bcd02815260206004820152600f60248201527f4e6f7420746865206d616e616765720000000000000000000000000000000000604482015290519081900360640190fd5b6103428361092a565b15610397576040805160e560020a62461bcd02815260206004820152601a60248201527f4d757374206e6f7420626520616e204552433136352068617368000000000000604482015290519081900360640190fd5b600160a060020a038216158015906103b85750600160a060020a0382163314155b156104ff5760405160200180807f455243313832305f4143434550545f4d4147494300000000000000000000000081525060140190506040516020818303038152906040528051906020012082600160a060020a031663249cb3fa85846040518363ffffffff167c01000000000000000000000000000000000000000000000000000000000281526004018083815260200182600160a060020a0316600160a060020a031681526020019250505060206040518083038186803b15801561047e57600080fd5b505afa158015610492573d6000803e3d6000fd5b505050506040513d60208110156104a857600080fd5b5051146104ff576040805160e560020a62461bcd02815260206004820181905260248201527f446f6573206e6f7420696d706c656d656e742074686520696e74657266616365604482015290519081900360640190fd5b600160a060020a03818116600081815260208181526040808320888452909152808220805473ffffffffffffffffffffffffffffffffffffffff19169487169485179055518692917f93baa6efbd2244243bfee6ce4cfdd1d04fc4c0e9a786abd3a41313bd352db15391a450505050565b600160a060020a03818116600090815260016020526040812054909116151561059a5750806105b7565b50600160a060020a03808216600090815260016020526040902054165b919050565b336105c683610570565b600160a060020a031614610624576040805160e560020a62461bcd02815260206004820152600f60248201527f4e6f7420746865206d616e616765720000000000000000000000000000000000604482015290519081900360640190fd5b81600160a060020a031681600160a060020a0316146106435780610646565b60005b600160a060020a03838116600081815260016020526040808220805473ffffffffffffffffffffffffffffffffffffffff19169585169590951790945592519184169290917f605c2dbf762e5f7d60a546d42e7205dcb1b011ebc62a61736a57c9089d3a43509190a35050565b600082826040516020018083838082843780830192505050925050506040516020818303038152906040528051906020012090505b92915050565b6106f882826107ef565b610703576000610705565b815b600160a060020a03928316600081815260208181526040808320600160e060020a031996909616808452958252808320805473ffffffffffffffffffffffffffffffffffffffff19169590971694909417909555908152600284528181209281529190925220805460ff19166001179055565b600080600160a060020a038416156107905783610792565b335b905061079d8361092a565b156107c357826107ad82826108aa565b6107b85760006107ba565b815b925050506106e8565b600160a060020a0390811660009081526020818152604080832086845290915290205416905092915050565b6000808061081d857f01ffc9a70000000000000000000000000000000000000000000000000000000061094c565b909250905081158061082d575080155b1561083d576000925050506106e8565b61084f85600160e060020a031961094c565b909250905081158061086057508015155b15610870576000925050506106e8565b61087a858561094c565b909250905060018214801561088f5750806001145b1561089f576001925050506106e8565b506000949350505050565b600160a060020a0382166000908152600260209081526040808320600160e060020a03198516845290915281205460ff1615156108f2576108eb83836107ef565b90506106e8565b50600160a060020a03808316600081815260208181526040808320600160e060020a0319871684529091529020549091161492915050565b7bffffffffffffffffffffffffffffffffffffffffffffffffffffffff161590565b6040517f01ffc9a7000000000000000000000000000000000000000000000000000000008082526004820183905260009182919060208160248189617530fa90519096909550935050505056fea165627a7a72305820377f4a2d4301ede9949f163f319021a6e9c687c292a5e2b2c4734c126b524e6c00291ba01820182018201820182018201820182018201820182018201820182018201820a01820182018201820182018201820182018201820182018201820182018201820")
                .into()).await.unwrap().await.unwrap();

        // Get deployer address
        let self_address = ethers::types::Address::from_slice(
            &PublicKey::from_privkey(anvil.keys()[0].to_bytes().as_ref())
                .unwrap()
                .to_address()
                .to_bytes(),
        );

        // Deploy node-stake factory contract
        let node_stake_factory = HoprNodeStakeFactory::deploy(client.clone(), ())
            .unwrap()
            .send()
            .await
            .unwrap();

        // Deploy node management module implementation contract
        let node_management_module = HoprNodeManagementModule::deploy(client.clone(), ())
            .unwrap()
            .send()
            .await
            .unwrap();

        // Deploy node-safe registry contract
        let node_safe_registry = HoprNodeSafeRegistry::deploy(client.clone(), ())
            .unwrap()
            .send()
            .await
            .unwrap();

        // Deploy token contract
        let hopr_token = HoprToken::deploy(client.clone(), ()).unwrap().send().await.unwrap();
        hopr_token
            .grant_role(hopr_token.minter_role().await.unwrap(), self_address)
            .send()
            .await
            .unwrap()
            .await
            .unwrap();

        // Mint us 1000 HOPR
        hopr_token
            .mint(self_address, 1000_u32.into(), Bytes::new(), Bytes::new())
            .send()
            .await
            .unwrap()
            .await
            .unwrap();

        // Deploy channels contract
        let hopr_channels = HoprChannels::deploy(
            client.clone(),
            Token::Tuple(vec![
                Token::Address(hopr_token.address()),
                Token::Uint(1_u32.into()),
                Token::Address(node_safe_registry.address()),
            ]),
        )
        .unwrap()
        .send()
        .await
        .unwrap();

        // Deploy network registry proxy (dummy proxy for local environment)
        let network_registry_proxy = HoprDummyProxyForNetworkRegistry::deploy(client.clone(), self_address)
            .unwrap()
            .send()
            .await
            .unwrap();

        // Deploy network registry
        let network_registry = HoprNetworkRegistry::deploy(
            client.clone(),
            (network_registry_proxy.address(), self_address, self_address),
        )
        .unwrap()
        .send()
        .await
        .unwrap();
        // Disable network registry in local environment
        network_registry.disable_registry().send().await.unwrap().await.unwrap();

        // Deploy ticket price oracle
        let ticket_price_oracle =
            HoprTicketPriceOracle::deploy(client.clone(), (self_address, U256::from(100000000000000000_u128)))
                .unwrap()
                .send()
                .await
                .unwrap();

        // Deploy announcements
        let hopr_announcements =
            HoprAnnouncements::deploy(client.clone(), Token::Address(node_safe_registry.address()))
                .unwrap()
                .send()
                .await
                .unwrap();

        debug!("contracts deployed");

        ContractAddresses {
            token: hopr_token.address().0.into(),
            channels: hopr_channels.address().0.into(),
            announcements: hopr_announcements.address().0.into(),
            network_registry: network_registry.address().0.into(),
            network_registry_proxy: network_registry_proxy.address().0.into(),
            safe_registry: node_safe_registry.address().0.into(),
            price_oracle: ticket_price_oracle.address().0.into(),
            stake_factory: node_stake_factory.address().0.into(),
            module_implementation: node_management_module.address().0.into(),
        }
    }

    pub fn create_anvil_with_provider(block_time: Duration) -> AnvilInstance {
        Anvil::new()
            .path(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../../.foundry/bin/anvil"))
            .block_time(block_time.as_secs())
            .spawn()
    }

    #[tokio::test]
    async fn test_deploy_contracts() {
        let anvil = create_anvil_with_provider(std::time::Duration::from_secs(2));
        let chain_key_0 = ChainKeypair::from_secret(anvil.keys()[0].to_bytes().as_ref()).unwrap();

        let cfg = mock_config();
        let rpc = RpcOperations::new(Http::from_str(&anvil.endpoint()).unwrap(), &chain_key_0, cfg)
            .expect("failed to construct rpc");

        let contract_addrs = deploy_contracts(rpc.provider.clone(), &anvil).await;

        assert_ne!(contract_addrs.token, Address::zero().into());
        assert_ne!(contract_addrs.channels, Address::zero().into());
        assert_ne!(contract_addrs.announcements, Address::zero().into());
        assert_ne!(contract_addrs.network_registry, Address::zero().into());
        assert_ne!(contract_addrs.safe_registry, Address::zero().into());
        assert_ne!(contract_addrs.price_oracle, Address::zero().into());
    }
}
