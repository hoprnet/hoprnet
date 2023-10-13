pub mod channel_graph;
pub mod errors;
pub mod path;
pub mod selectors;

use async_std::sync::RwLock;
use async_trait::async_trait;
use core_crypto::types::OffchainPublicKey;
use core_ethereum_db::traits::HoprCoreEthereumDbActions;
use core_types::protocol::PeerAddressResolver;
use std::sync::Arc;
use utils_log::error;
use utils_types::primitives::Address;

/// DB backed packet key to chain key resolver
#[derive(Clone)]
pub struct DbPeerAddressResolver<Db: HoprCoreEthereumDbActions>(pub Arc<RwLock<Db>>);

#[async_trait(? Send)]
impl<Db: HoprCoreEthereumDbActions> PeerAddressResolver for DbPeerAddressResolver<Db> {
    async fn resolve_packet_key(&self, onchain_key: &Address) -> Option<OffchainPublicKey> {
        match self.0.read().await.get_packet_key(onchain_key).await {
            Ok(k) => k,
            Err(e) => {
                error!("failed to resolve packet key for {onchain_key}: {e}");
                None
            }
        }
    }

    async fn resolve_chain_key(&self, offchain_key: &OffchainPublicKey) -> Option<Address> {
        match self.0.read().await.get_chain_key(offchain_key).await {
            Ok(k) => k,
            Err(e) => {
                error!("failed to resolve chain key for {offchain_key}: {e}");
                None
            }
        }
    }
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use utils_log::logger::wasm::JsLogger;
    use wasm_bindgen::prelude::wasm_bindgen;

    static LOGGER: JsLogger = JsLogger {};

    #[allow(dead_code)]
    #[wasm_bindgen]
    pub fn core_path_initialize_crate() {
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

    // When the `wee_alloc` feature is enabled, use `wee_alloc` as the global allocator.
    #[cfg(feature = "wee_alloc")]
    #[global_allocator]
    static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
}
