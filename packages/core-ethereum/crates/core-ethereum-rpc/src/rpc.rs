use async_std::prelude::Stream;
use async_trait::async_trait;
use core_crypto::keypairs::{ChainKeypair, Keypair};
use core_crypto::types::Hash;
use core_ethereum_misc::ContractAddresses;
use ethers::middleware::{MiddlewareBuilder, NonceManagerMiddleware, SignerMiddleware};
use ethers::prelude::k256::ecdsa::SigningKey;
use ethers::prelude::transaction::eip2718::TypedTransaction;
use ethers::signers::{LocalWallet, Signer, Wallet};
use ethers::types::BlockId;
use ethers_providers::{FilterWatcher, JsonRpcClient, Middleware, Provider};
use futures::StreamExt;
use pin_project::pin_project;
use primitive_types::{H160, H256};
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Duration;
use utils_types::primitives::{Address, Balance, BalanceType, U256};
use validator::Validate;

use bindings::hopr_announcements::HoprAnnouncements;
use bindings::hopr_channels::HoprChannels;
use bindings::hopr_network_registry::HoprNetworkRegistry;
use bindings::hopr_node_management_module::HoprNodeManagementModule;
use bindings::hopr_node_safe_registry::HoprNodeSafeRegistry;
use bindings::hopr_token::HoprToken;
use utils_log::error;

use crate::errors::Result;
use crate::{Block, EventsQuery, HoprRpcOperations, Log};

#[derive(Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize, Validate)]
pub struct RpcOperationsConfig {
    pub indexer_start_block_number: u64,
    pub chain_id: u64,
    pub contract_addrs: ContractAddresses,
    pub node_module: Address,
    pub polling_interval: Duration,
}

type HoprMiddleware<P> = NonceManagerMiddleware<SignerMiddleware<Provider<P>, Wallet<SigningKey>>>;

pub struct RpcOperations<P: JsonRpcClient> {
    me: Address,
    provider: Arc<HoprMiddleware<P>>,
    channels: HoprChannels<HoprMiddleware<P>>,
    announcements: HoprAnnouncements<HoprMiddleware<P>>,
    safe_registry: HoprNodeSafeRegistry<HoprMiddleware<P>>,
    network_registry: HoprNetworkRegistry<HoprMiddleware<P>>,
    node_module: HoprNodeManagementModule<HoprMiddleware<P>>,
    token: HoprToken<HoprMiddleware<P>>,
    cfg: RpcOperationsConfig,
}

impl<P: JsonRpcClient + 'static> RpcOperations<P> {
    pub fn new(json_rpc: P, chain_key: &ChainKeypair, cfg: RpcOperationsConfig) -> Result<Self> {
        let wallet = LocalWallet::from_bytes(chain_key.secret().as_ref())?;
        let provider = Arc::new(
            Provider::new(json_rpc)
                .interval(cfg.polling_interval)
                .with_signer(wallet.with_chain_id(cfg.chain_id))
                .nonce_manager(chain_key.public().to_address().into()),
        );

        Ok(Self {
            me: chain_key.public().to_address(),
            channels: HoprChannels::new::<H160>(cfg.contract_addrs.channels.into(), provider.clone()),
            announcements: HoprAnnouncements::new::<H160>(cfg.contract_addrs.announcements.into(), provider.clone()),
            safe_registry: HoprNodeSafeRegistry::new::<H160>(cfg.contract_addrs.safe_registry.into(), provider.clone()),
            network_registry: HoprNetworkRegistry::new::<H160>(
                cfg.contract_addrs.network_registry.into(),
                provider.clone(),
            ),
            node_module: HoprNodeManagementModule::new::<H160>(cfg.node_module.into(), provider.clone()),
            token: HoprToken::new::<H160>(cfg.contract_addrs.token.into(), provider.clone()),
            cfg,
            provider,
        })
    }

    async fn get_block(&self, block_number: u64) -> Result<Option<ethers::types::Block<H256>>> {
        let block_id: BlockId = block_number.into();
        Ok(self.provider.get_block(block_id).await?)
    }
}

#[cfg(target_arch = "wasm32")]
type PinBoxFut<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;

#[cfg(not(target_arch = "wasm32"))]
type PinBoxFut<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

type HoprMiddlewareResult<T, P> = std::result::Result<T, <HoprMiddleware<P> as Middleware>::Error>;

#[must_use = "streams do nothing unless polled"]
#[pin_project]
pub struct BlockStream<'a, P, S>
where
    P: JsonRpcClient,
    S: Stream<Item = H256>,
{
    rpc: &'a RpcOperations<P>,
    #[pin]
    inner: S,
    #[pin]
    future: Option<PinBoxFut<'a, HoprMiddlewareResult<Option<ethers::types::Block<H256>>, P>>>,
}

impl<'a, P, S> BlockStream<'a, P, S>
where
    P: JsonRpcClient,
    S: Stream<Item = H256>,
{
    fn new(rpc: &'a RpcOperations<P>, inner: S) -> Self {
        Self {
            rpc,
            inner,
            future: None,
        }
    }
}

impl<'a, P, S> Stream for BlockStream<'a, P, S>
where
    P: JsonRpcClient,
    S: Stream<Item = H256>,
{
    type Item = Block;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        Poll::Ready(loop {
            if let Some(fut) = this.future.as_mut().as_pin_mut() {
                let ready = futures::ready!(fut.poll(cx));
                this.future.set(None);

                match ready {
                    Ok(Some(item)) => {
                        break Some(crate::Block::from(item));
                    }
                    Ok(None) => {
                        error!("encountered non-existing block id");
                    }
                    Err(e) => {
                        error!("failed to retrieve block: {e}");
                    }
                };
            } else if let Some(item) = futures::ready!(this.inner.as_mut().poll_next(cx)) {
                let f = this.rpc.provider.get_block(BlockId::Hash(item));
                this.future.set(Some(f));
            } else {
                break None;
            }
        })
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl<P: JsonRpcClient + 'static> HoprRpcOperations for RpcOperations<P> {
    type BlockStream<'a> = BlockStream<'a, P, FilterWatcher<'a, P, H256>>;
    type LogStream<'a> = Pin<Box<dyn Stream<Item = Log> + 'a>>;

    async fn genesis_block(&self) -> Result<u64> {
        Ok(self.cfg.indexer_start_block_number)
    }

    async fn block_number(&self) -> Result<u64> {
        let r = self.provider.get_block_number().await?;
        Ok(r.as_u64())
    }

    async fn get_timestamp(&self, block_number: u64) -> Result<Option<u64>> {
        Ok(self.get_block(block_number).await?.map(|b| b.timestamp.as_u64()))
    }

    async fn get_balance(&self, balance_type: BalanceType) -> Result<Balance> {
        match balance_type {
            BalanceType::Native => {
                let addr: H160 = self.me.into();
                let native = self.provider.get_balance(addr, None).await?;
                Ok(Balance::new(native.into(), BalanceType::Native))
            }
            BalanceType::HOPR => {
                let token_balance = self.token.balance_of(self.me.into()).call().await?;
                Ok(Balance::new(token_balance.into(), BalanceType::HOPR))
            }
        }
    }

    async fn get_transactions_in_block(&self, block_number: u64) -> Result<Vec<Hash>> {
        Ok(self
            .get_block(block_number)
            .await?
            .map(|block| block.transactions.iter().map(|h| Hash::from(h.0)).collect::<Vec<_>>())
            .unwrap_or_default())
    }

    async fn get_node_management_module_target_info(&self, target: Address) -> Result<Option<U256>> {
        let (exists, target) = self.node_module.try_get_target(target.into()).call().await?;
        Ok(exists.then_some(target.into()))
    }

    async fn get_safe_from_node_safe_registry(&self, node_address: Address) -> Result<Address> {
        let addr = self.safe_registry.node_to_safe(node_address.into()).call().await?;
        Ok(addr.into())
    }

    async fn get_module_target_address(&self) -> Result<Address> {
        let owner = self.node_module.owner().call().await?;
        Ok(owner.into())
    }

    async fn send_transaction(&self, tx: TypedTransaction) -> Result<Hash> {
        let sent_tx = self.provider.send_transaction(tx, None).await?;
        Ok(sent_tx.0.into())
    }

    async fn subscribe_blocks<'a>(&'a self) -> Result<Self::BlockStream<'a>> {
        Ok(BlockStream::new(self, self.provider.watch_blocks().await?))
    }

    async fn subscribe_logs<'a>(&'a self, query: EventsQuery) -> Result<Self::LogStream<'a>> {
        Ok(Box::pin(
            self.provider
                .watch(&query.into())
                .await?
                .map(|log| crate::Log::from(log)),
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::rpc::{RpcOperations, RpcOperationsConfig};
    use crate::{Block, HoprRpcOperations, TypedTransaction};
    use core_crypto::keypairs::{ChainKeypair, Keypair};
    use core_crypto::types::Hash;
    use core_ethereum_misc::ContractAddresses;
    use ethers::prelude::U64;
    use ethers::types::Eip1559TransactionRequest;
    use ethers::utils::{Anvil, AnvilInstance};
    use ethers_providers::{Http, MockProvider};
    use futures::future::Either;
    use futures::StreamExt;
    use primitive_types::H160;
    use std::path::PathBuf;
    use std::pin::pin;
    use std::str::FromStr;
    use std::time::Duration;
    use utils_types::primitives::{Address, BalanceType, U256};

    fn anvil_provider() -> (Http, ChainKeypair, AnvilInstance) {
        let anvil: AnvilInstance = Anvil::new()
            .path(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../../.foundry/bin/anvil"))
            .block_time(1_u64)
            .spawn();

        (
            Http::from_str(&anvil.endpoint()).unwrap(),
            ChainKeypair::from_secret(anvil.keys()[0].to_bytes().as_ref()).unwrap(),
            anvil,
        )
    }

    fn mock_config() -> RpcOperationsConfig {
        RpcOperationsConfig {
            indexer_start_block_number: 0,
            polling_interval: Duration::from_millis(100),
            chain_id: 31337, // Anvil default chain id
            contract_addrs: ContractAddresses {
                channels: Address::random(),
                announcements: Address::random(),
                token: Address::random(),
                safe_registry: Address::random(),
                network_registry: Address::random(),
                price_oracle: Address::random(),
            },
            node_module: Address::random(),
        }
    }

    fn transfer_eth_tx(to: Address, amount: U256) -> TypedTransaction {
        let mut tx = TypedTransaction::Eip1559(Eip1559TransactionRequest::new());
        tx.set_to(H160::from(to));
        tx.set_value(ethers::types::U256(primitive_types::U256::from(amount).0));
        tx
    }

    async fn wait_until_tx<Rpc: HoprRpcOperations>(tx_hash: Hash, rpc: &Rpc) -> Block {
        let stream = rpc.subscribe_blocks().await.expect("failed to get block stream");

        let timeout = pin!(tokio::time::sleep(Duration::from_secs(8)));
        let block = pin!(stream
            .filter(|block| futures::future::ready(block.transactions.contains(&tx_hash)))
            .take(1)
            .collect::<Vec<Block>>());

        match futures::future::select(block, timeout).await {
            Either::Left((mut vec, _)) => vec.pop().unwrap(),
            Either::Right(_) => panic!("timeout"),
        }
    }

    #[async_std::test]
    async fn test_get_block_number() {
        let prov = MockProvider::new();
        let block_num = U64::from(2);

        prov.push(block_num).unwrap();

        let chain_key = ChainKeypair::random();
        let cfg = mock_config();

        let rpc = RpcOperations::new(prov, &chain_key, cfg).unwrap();

        let bn = rpc.block_number().await.unwrap();
        assert_eq!(block_num.as_u64(), bn);
    }

    #[tokio::test]
    async fn test_get_txs_in_block() {
        let (prov, chain_key, _instance) = anvil_provider();

        let cfg = mock_config();
        let rpc = RpcOperations::new(prov, &chain_key, cfg).expect("failed to construct rpc");

        // Send 1 ETH to some random address
        let tx_hash = rpc
            .send_transaction(transfer_eth_tx(Address::random(), 1_u32.into()))
            .await
            .expect("failed to send tx");

        let block = wait_until_tx(tx_hash, &rpc).await;

        assert!(rpc.get_transactions_in_block(block.number.unwrap()).await.unwrap().contains(&tx_hash), "must contain tx");
    }

    #[tokio::test]
    async fn test_get_balance_native() {
        let (prov, chain_key, _instance) = anvil_provider();

        let cfg = mock_config();
        let rpc = RpcOperations::new(prov, &chain_key, cfg).expect("failed to construct rpc");

        let balance_1 = rpc.get_balance(BalanceType::Native).await.unwrap();
        assert!(balance_1.value().as_u64() > 0, "balance must be greater than 0");

        // Send 1 ETH to some random address
        let tx_hash = rpc
            .send_transaction(transfer_eth_tx(Address::random(), 1_u32.into()))
            .await
            .expect("failed to send tx");

        let _ = wait_until_tx(tx_hash, &rpc).await;

        let balance_2 = rpc.get_balance(BalanceType::Native).await.unwrap();
        assert!(balance_2.lt(&balance_1), "balance must be diminished");
    }

    #[tokio::test]
    async fn test_subscribe_blocks() {
        let (prov, chain_key, _instance) = anvil_provider();

        let cfg = mock_config();
        let rpc = RpcOperations::new(prov, &chain_key, cfg).expect("failed to construct rpc");

        // Send 1 ETH to some random address
        let tx_hash = rpc
            .send_transaction(transfer_eth_tx(Address::random(), 1_u32.into()))
            .await
            .expect("failed to send tx");

        let stream = rpc.subscribe_blocks().await.expect("failed to get block stream");

        let timeout = pin!(tokio::time::sleep(Duration::from_secs(8)));
        let hash = pin!(stream.any(|block| async move { block.transactions.contains(&tx_hash) }));

        match futures::future::select(hash, timeout).await {
            Either::Left(_) => {}
            Either::Right(_) => panic!("timeout"),
        };
    }
}
