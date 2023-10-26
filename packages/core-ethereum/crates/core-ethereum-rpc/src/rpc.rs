use async_trait::async_trait;
use std::sync::Arc;
use ethers::prelude::*;
use ethers::prelude::k256::ecdsa::SigningKey;
use ethers::prelude::transaction::eip2718::TypedTransaction;
use ethers_providers::{JsonRpcClient, Middleware, Provider};
use serde::{Deserialize, Serialize};
use validator::Validate;
use core_crypto::keypairs::{ChainKeypair, Keypair};
use core_crypto::types::Hash;
use core_ethereum_misc::ContractAddresses;
use utils_types::primitives::{Address, Balance, BalanceType, U256};

use bindings::hopr_announcements::HoprAnnouncements;
use bindings::hopr_channels::HoprChannels;
use bindings::hopr_network_registry::HoprNetworkRegistry;
use bindings::hopr_node_management_module::HoprNodeManagementModule;
use bindings::hopr_node_safe_registry::HoprNodeSafeRegistry;
use bindings::hopr_token::HoprToken;

use crate::HoprRpcOperations;
use crate::errors::Result;

#[derive(Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize, Validate)]
pub struct RpcOperationsConfig {
    pub indexer_start_block_number: u64,
    pub chain_id: u64,
    pub contract_addrs: ContractAddresses,
    pub node_module: Address,
}

type HoprMiddleware<P> = SignerMiddleware<Provider<P>, Wallet<SigningKey>>;

pub struct RpcOperations<P: JsonRpcClient> {
    me: Address,
    signer: Arc<HoprMiddleware<P>>,
    channels: HoprChannels<HoprMiddleware<P>>,
    announcements: HoprAnnouncements<HoprMiddleware<P>>,
    safe_registry: HoprNodeSafeRegistry<HoprMiddleware<P>>,
    network_registry: HoprNetworkRegistry<HoprMiddleware<P>>,
    node_module: HoprNodeManagementModule<HoprMiddleware<P>>,
    token: HoprToken<HoprMiddleware<P>>,
    cfg: RpcOperationsConfig,
}

impl<P: JsonRpcClient> RpcOperations<P> {
    pub fn new(json_rpc: P, chain_key: ChainKeypair, cfg: RpcOperationsConfig) -> Result<Self> {
        let wallet = LocalWallet::from_bytes(chain_key.secret().as_ref())?;
        let signer = Arc::new(SignerMiddleware::new(Provider::new(json_rpc), wallet.with_chain_id(cfg.chain_id)));

        Ok(Self {
            me: chain_key.public().to_address(),
            channels: HoprChannels::new::<H160>(cfg.contract_addrs.channels.into(), signer.clone()),
            announcements: HoprAnnouncements::new::<H160>(cfg.contract_addrs.announcements.into(), signer.clone()),
            safe_registry: HoprNodeSafeRegistry::new::<H160>(cfg.contract_addrs.safe_registry.into(), signer.clone()),
            network_registry: HoprNetworkRegistry::new::<H160>(cfg.contract_addrs.network_registry.into(), signer.clone()),
            node_module: HoprNodeManagementModule::new::<H160>(cfg.node_module.into(), signer.clone()),
            token: HoprToken::new::<H160>(cfg.contract_addrs.token.into(), signer.clone()),
            cfg,
            signer,
        })
    }

    async fn get_block(&self, block_number: u64) -> Result<Option<Block<H256>>> {
        let block_id: BlockId = block_number.into();
        Ok(self.signer.get_block(block_id).await?)
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl<P: JsonRpcClient> HoprRpcOperations for RpcOperations<P> {
    async fn genesis_block(&self) -> Result<u64> {
        Ok(self.cfg.indexer_start_block_number)
    }

    async fn block_number(&self) -> Result<u64> {
        let r = self.signer.get_block_number().await?;
        Ok(r.as_u64())
    }

    async fn get_timestamp(&self, block_number: u64) -> Result<Option<u64>> {
        Ok(self.get_block(block_number).await?.map(|b| b.timestamp.as_u64()))
    }

    async fn get_balance(&self, balance_type: BalanceType) -> Result<Balance> {
        match balance_type {
            BalanceType::Native => {
                let addr: H160 = self.me.into();
                let native = self.signer.get_balance(addr, None).await?;
                Ok(Balance::new(native.into(), BalanceType::Native))
            },
            BalanceType::HOPR => {
                let token_balance = self.token.balance_of(self.me.into()).call().await?;
                Ok(Balance::new(token_balance.into(), BalanceType::HOPR))
            }
        }
    }

    async fn get_transactions_in_block(&self, block_number: u64) -> Result<Vec<Hash>> {
        Ok(self.get_block(block_number)
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
        let sent_tx = self.signer.send_transaction(tx, None).await?;
        Ok(sent_tx.0.into())
    }
}