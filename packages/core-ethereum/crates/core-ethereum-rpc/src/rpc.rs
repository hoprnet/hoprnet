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
        Ok(Box::pin(self
            .provider
            .watch(&query.into())
            .await?
            .map(|log| crate::Log::from(log))
        ))
    }
}

#[cfg(test)]
mod tests {}
