
/// Network quality threshold from which a node is considered
/// available enough to be used
pub const DEFAULT_NETWORK_QUALITY_THRESHOLD: f32 = 0.5;
/// Number of parallel connection handled by js-libp2p
/// pub const DEFAULT_MAX_PARALLEL_CONNECTIONS: u32 = 100;
/// FIXME: reduce default again once connection recyclying was fixed
pub const DEFAULT_MAX_PARALLEL_CONNECTIONS: u32 = 50_000;
/// Number of parallel connection handled by js-libp2p
/// when running as a public relay node (i.e. the --announce flag is set)
pub const DEFAULT_MAX_PARALLEL_CONNECTION_PUBLIC_RELAY: u32 = 50_000;

#[cfg(feature = "wasm")]
pub mod wasm {
    // Need to load as wasm_bindgen to make field annotations work
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    pub struct CoreConstants {
        #[wasm_bindgen(readonly, js_name = "DEFAULT_NETWORK_QUALITY_THRESHOLD")]
        pub default_network_quality_threshold: f32,
        #[wasm_bindgen(readonly, js_name = "DEFAULT_MAX_PARALLEL_CONNECTIONS")]
        pub default_max_parallel_connections: u32,
        #[wasm_bindgen(readonly, js_name = "DEFAULT_MAX_PARALLEL_CONNECTIONS_PUBLIC_RELAY")]
        pub default_max_parallel_connections_public_relay: u32,
    }

    /// Returns a struct with readonly constants, needs to be a function
    /// because Rust does not support exporting constants to WASM
    #[wasm_bindgen(js_name = "CORE_CONSTANTS")]
    pub fn get_constants() -> CoreConstants {
        CoreConstants {
            default_network_quality_threshold: super::DEFAULT_NETWORK_QUALITY_THRESHOLD,
            default_max_parallel_connections: super::DEFAULT_MAX_PARALLEL_CONNECTIONS,
            default_max_parallel_connections_public_relay: super::DEFAULT_MAX_PARALLEL_CONNECTION_PUBLIC_RELAY,
        }
    }
}
