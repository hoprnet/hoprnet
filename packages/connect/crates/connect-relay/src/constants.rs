/// Default timeout to wait when checking whether a relayed
/// connection is usable.
pub const DEFAULT_RELAYED_CONNECTION_PING_TIMEOUT: u32 = 300;

#[cfg(feature = "wasm")]
pub mod wasm {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    pub struct ConnectConstants {
        #[wasm_bindgen(readonly, js_name = "DEFAULT_RELAYED_CONNECTION_PING_TIMEOUT")]
        pub default_relayed_connection_ping_timeout: u32,
    }

    /// Returns a struct with readonly constants, needs to be a function
    /// because Rust does not support exporting constants to WASM
    #[wasm_bindgen(js_name = "CONNECT_CONSTANTS")]
    pub fn get_constants() -> ConnectConstants {
        ConnectConstants {
            default_relayed_connection_ping_timeout: super::DEFAULT_RELAYED_CONNECTION_PING_TIMEOUT,
        }
    }
}
