
/// Network quality threshold from which a node is considered
/// available enough to be used
pub const DEFAULT_NETWORK_QUALITY_THRESHOLD: f32 = 0.5;

#[cfg(feature = "wasm")]
pub mod wasm {
    // Need to load as wasm_bindgen to make field annotations work
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    pub struct CoreConstants {
        #[wasm_bindgen(readonly, js_name = "DEFAULT_NETWORK_QUALITY_THRESHOLD")]
        pub default_network_quality_threshold: f32,
    }

    /// Returns a struct with readonly constants, needs to be a function
    /// because Rust does not support exporting constants to WASM
    #[wasm_bindgen(js_name = "CORE_CONSTANTS")]
    pub fn get_constants() -> CoreConstants {
        CoreConstants {
            default_network_quality_threshold: super::DEFAULT_NETWORK_QUALITY_THRESHOLD,
        }
    }
}
