pub mod errors;

use async_lock::RwLock;
use core_ethereum_db::db::CoreEthereumDb;

use futures::channel::mpsc::UnboundedSender;
use futures::SinkExt;
use std::sync::Arc;
use utils_log::error;

pub use core_types::channels::ChannelEntry;

use core_crypto::keypairs::{ChainKeypair, Keypair};
use core_ethereum_actions::CoreEthereumActions;
use core_ethereum_db::traits::HoprCoreEthereumDbActions;
use core_types::account::AccountEntry;
use utils_db::rusty::RustyLevelDbShim;
use utils_types::primitives::{Address, Balance};

use crate::errors::HoprChainError;

#[async_trait::async_trait]
pub trait ChainQueries {
    async fn is_node_safe_registered(&self) -> errors::Result<bool>;

    async fn can_register_with_safe(&self) -> errors::Result<bool>;

    async fn wait_for_funds(&self) -> errors::Result<()>;

    async fn clone(&self) -> Self;
}

/// This is used by the indexer to emit events when a change on channel entry is detected.
#[derive(Clone)]
pub struct ChannelEventEmitter {
    pub tx: UnboundedSender<ChannelEntry>,
}

impl ChannelEventEmitter {
    pub async fn send_event(&self, channel: &ChannelEntry) {
        let mut sender = self.tx.clone();
        let _ = sender.send(channel.clone()).await;
    }
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
#[derive(Clone)]
pub struct HoprChain {
    me_onchain: ChainKeypair,
    db: Arc<RwLock<CoreEthereumDb<utils_db::rusty::RustyLevelDbShim>>>,
    chain_actions: CoreEthereumActions<CoreEthereumDb<RustyLevelDbShim>>,
    channel_events: ChannelEventEmitter,
    // TODO: change this object for non-wasm and extract API
    // channel_graph: core_path::channel_graph::wasm::ChannelGraph,
    channel_graph: Arc<RwLock<core_path::channel_graph::ChannelGraph>>,
}

impl HoprChain {
    pub fn new(
        me_onchain: ChainKeypair,
        db: Arc<RwLock<CoreEthereumDb<utils_db::rusty::RustyLevelDbShim>>>,
        chain_actions: CoreEthereumActions<CoreEthereumDb<RustyLevelDbShim>>,
        channel_events: ChannelEventEmitter,
        channel_graph: Arc<RwLock<core_path::channel_graph::ChannelGraph>>,
    ) -> Self {
        Self {
            me_onchain,
            db,
            chain_actions,
            channel_events,
            channel_graph,
        }
    }

    pub fn me_onchain(&self) -> Address {
        self.me_onchain.public().to_address()
    }

    pub async fn accounts_announced_on_chain(&self) -> errors::Result<Vec<AccountEntry>> {
        Ok(self.db.read().await.get_accounts().await?)
    }

    pub async fn channel(&self, src: &Address, dest: &Address) -> errors::Result<ChannelEntry> {
        Ok(self
            .db
            .read()
            .await
            .get_channel_x(&src, &dest)
            .await
            .map_err(HoprChainError::from)
            .and_then(|v| {
                v.ok_or(errors::HoprChainError::Api(format!(
                    "Channel entry not available {}-{}",
                    src, dest
                )))
            })?)
    }

    pub async fn channels_from(&self, src: &Address) -> errors::Result<Vec<ChannelEntry>> {
        Ok(self.db.read().await.get_channels_from(&src).await?)
    }

    pub async fn channels_to(&self, dest: &Address) -> errors::Result<Vec<ChannelEntry>> {
        Ok(self.db.read().await.get_channels_to(&dest).await?)
    }

    pub async fn all_channels(&self) -> errors::Result<Vec<ChannelEntry>> {
        Ok(self.db.read().await.get_channels().await?)
    }

    pub async fn safe_allowance(&self) -> errors::Result<Balance> {
        Ok(self.db.read().await.get_staking_safe_allowance().await?)
    }

    pub async fn on_ticket_redeemed(&self, channel: &ChannelEntry, ticket_amount: &Balance) {
        if channel.source.eq(&self.me_onchain()) {
            if let Err(e) = self
                .db
                .write()
                .await
                .resolve_pending(&channel.destination, &ticket_amount)
                .await
            {
                error!("on_ticket_redeemed failed with: {}", e);
            }
        }
    }

    pub async fn on_channel_event(&self, entry: &ChannelEntry) {
        //utils_log::debug!("Received a channel event {:?}", entry);
        self.channel_events.send_event(entry).await;
    }

    pub fn actions_ref(&self) -> &CoreEthereumActions<CoreEthereumDb<RustyLevelDbShim>> {
        &self.chain_actions
    }

    pub fn actions_mut_ref(&mut self) -> &mut CoreEthereumActions<CoreEthereumDb<RustyLevelDbShim>> {
        &mut self.chain_actions
    }

    // NOTE: needed early in the initialization to sync
    pub fn channel_graph(&self) -> Arc<RwLock<core_path::channel_graph::ChannelGraph>> {
        self.channel_graph.clone()
    }

    // NOTE: needed early in the initialization to sync
    pub fn db(&self) -> Arc<RwLock<CoreEthereumDb<utils_db::rusty::RustyLevelDbShim>>> {
        self.db.clone()
    }
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use utils_log::logger::wasm::JsLogger;
    use utils_misc::utils::wasm::JsResult;
    use wasm_bindgen::prelude::*;

    // When the `wee_alloc` feature is enabled, use `wee_alloc` as the global allocator.
    #[cfg(feature = "wee_alloc")]
    #[global_allocator]
    static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

    static LOGGER: JsLogger = JsLogger {};

    #[allow(dead_code)]
    #[wasm_bindgen]
    pub fn core_ethereum_api_initialize_crate() {
        let _ = JsLogger::install(&LOGGER, None);

        // When the `console_error_panic_hook` feature is enabled, we can call the
        // `set_panic_hook` function at least once during initialization, and then
        // we will get better error messages if our code ever panics.
        //
        // For more details see
        // https://github.com/rustwasm/console_error_panic_hook#readme
        #[cfg(feature = "console_error_panic_hook")]
        console_error_panic_hook::set_once();
    }

    #[wasm_bindgen]
    pub fn core_ethereum_api_gather_metrics() -> JsResult<String> {
        utils_metrics::metrics::wasm::gather_all_metrics()
    }
}
