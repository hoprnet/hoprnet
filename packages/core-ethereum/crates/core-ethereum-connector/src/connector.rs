use std::sync::Mutex;
use lazy_static::lazy_static;
use utils_types::channels::ChannelEntry;
use utils_types::crypto::PublicKey;
use crate::errors::Result;

pub struct HoprCoreEthereum ;

lazy_static! {
    pub static ref CONNECTOR_INSTANCE: Mutex<HoprCoreEthereum> = Mutex::new(HoprCoreEthereum {});
}

impl HoprCoreEthereum {

    pub async fn redeem_tickets_in_channel_by_counterparty(&self, counterparty: &PublicKey) -> Result<()>{
        let bridge = wasm::BridgeJsHoprCoreEthereum::new();
        bridge.redeem_tickets_in_channel_by_counterparty(counterparty).await
    }

    pub async fn redeem_tickets_in_channel(&self, channel: &ChannelEntry) -> Result<()> {
        let bridge = wasm::BridgeJsHoprCoreEthereum::new();
        bridge.redeem_tickets_in_channel(channel).await
    }

    pub fn get_public_key(&self) -> PublicKey {
        let bridge = wasm::BridgeJsHoprCoreEthereum::new();
        bridge.get_public_key()
    }
}

/// Unit tests of pure Rust code (must not contain anything WASM-specific)
#[cfg(test)]
mod tests {
    use super::*;

}

/// Module for WASM-specific Rust code
#[cfg(feature = "wasm")]
pub mod wasm {

    use wasm_bindgen::prelude::*;
    use utils_misc::utils::wasm::JsResult;
    use utils_types::channels::ChannelEntry;
    use utils_types::crypto::PublicKey;
    use js_sys::Uint8Array;
    use crate::errors::Result;
    use crate::errors::ConnectorError::{ExecutionError, UnknownError};

    #[wasm_bindgen(module = "@hoprnet/hopr-utils")]
    extern "C" {
        #[wasm_bindgen(js_name = PublicKey, typescript_type="PublicKey")]
        type JsPublicKey;

        #[wasm_bindgen(catch, static_method_of = JsPublicKey, js_name = "deserialize")]
        fn deserialize(arr: &[u8]) -> JsResult<JsPublicKey>;

        #[wasm_bindgen(method, js_name = "serializeUncompressed")]
        fn serializeUncompressed(this: &JsPublicKey) -> Uint8Array;

        #[wasm_bindgen(js_name = ChannelEntry, typescript_type="ChannelEntry")]
        type JsChannelEntry;

        #[wasm_bindgen(catch, static_method_of = JsChannelEntry, js_name = "deserialize")]
        fn deserialize(arr: &[u8]) -> JsResult<JsChannelEntry>;
    }

    // This will be removed once HoprCoreEthereum is re-implemented in Rust
    #[wasm_bindgen(module = "@hoprnet/hopr-core-ethereum")]
    extern "C" {
        #[wasm_bindgen(js_name = HoprCoreEthereum, typescript_type="HoprCoreEthereum")]
        type JsHoprCoreEthereum ;

        #[wasm_bindgen(static_method_of = JsHoprCoreEthereum, js_class = "HoprCoreEthereum", js_name = instance, getter)]
        fn hopr_core_ethereum_instance() -> JsHoprCoreEthereum;

        #[wasm_bindgen(method, js_class="HoprCoreEthereum", js_name = getPublicKey)]
        fn get_public_key(this: &JsHoprCoreEthereum) -> JsPublicKey;

        #[wasm_bindgen(catch, method, js_class="HoprCoreEthereum", js_name = redeemTicketsInChannelByCounterparty)]
        async fn redeem_tickets_in_channel_by_counterparty(this: &JsHoprCoreEthereum, counterparty: &JsPublicKey) -> JsResult<()>;

        #[wasm_bindgen(catch, method, js_class="HoprCoreEthereum", js_name = redeemTicketsInChannel)]
        async fn redeem_tickets_in_channel(this: &JsHoprCoreEthereum, channel: &JsChannelEntry) -> JsResult<()>;
    }

    pub struct BridgeJsHoprCoreEthereum {
        js_impl: JsHoprCoreEthereum
    }

    impl BridgeJsHoprCoreEthereum {
        pub fn new() -> Self {
            BridgeJsHoprCoreEthereum {
                js_impl: JsHoprCoreEthereum::hopr_core_ethereum_instance()
            }
        }

        pub async fn redeem_tickets_in_channel_by_counterparty(&self, counterparty: &PublicKey) -> Result<()>{
            let pk = JsPublicKey::deserialize(&counterparty.serialize(false)).unwrap();
            JsHoprCoreEthereum::redeem_tickets_in_channel_by_counterparty(&self.js_impl, &pk)
            .await
            .map_err(|e| e.as_string().map(|e| ExecutionError(e)).unwrap_or(UnknownError))?;
            Ok(())
        }

        pub async fn redeem_tickets_in_channel(&self, channel: &ChannelEntry) -> Result<()> {
            let ce = JsChannelEntry::deserialize(&channel.serialize()).unwrap();
            JsHoprCoreEthereum::redeem_tickets_in_channel(&self.js_impl, &ce)
                .await
                .map_err(|e| e.as_string().map(|e| ExecutionError(e)).unwrap_or(UnknownError))?;
            Ok(())
        }

        pub fn get_public_key(&self) -> PublicKey {
            let js_pk = JsHoprCoreEthereum::get_public_key(&self.js_impl);
            PublicKey::deserialize(&JsPublicKey::serializeUncompressed(&js_pk).to_vec()).unwrap()
        }
    }
}

