use utils_types::primitives::Address;

/// Object needed only to simplify the iteration over the address and quality pair until
/// the strategy is migrated into Rust
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct PeerQuality {
    peers_with_quality: Vec<(Address, f64)>,
}

impl PeerQuality {
    pub fn new(peers: Vec<(Address, f64)>) -> Self {
        Self {
            peers_with_quality: peers,
        }
    }

    pub fn take(&self) -> Vec<(Address, f64)> {
        self.peers_with_quality.clone()
    }
}
