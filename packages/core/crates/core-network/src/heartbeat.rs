use std::time::Duration;

use libp2p::PeerId;

const MAX_PARALLEL_HEARTBEATS: u16 = 14;
const HEART_BEAT_ROUND_TIMEOUT: Duration = Duration::from_secs(60);

mod metrics {

}

#[derive(Debug)]
pub struct HeartbeatPingResult {
    pub destination: PeerId,
    pub last_seen: Option<u64>
}


// // This type will be made available to both WASM (when the "wasm" feature is turned on)
// // and non-WASM (pure Rust)
// #[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
// pub struct MyStruct {
//     foo: u32,        // private members do not need to have WASM-compatible types
//     pub bar: u32,    // public members MUST have WASM-compatible types, if struct used with #[cfg_attr(feature = "wasm"...
//     // NOTE: you can use #[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(skip))]
//     // on a public attribute that shall not be available to WASM
// }
//
// // This function won't be available in WASM, even when the "wasm" feature is turned on
// // You can use WASM-incompatible types in its arguments and return types freely.
// pub fn foo() -> u32 { 42 }
//
// // This function must not use WASM-specific types. Only WASM-compatible types must be used,
// // because the attribute makes it available to both WASM and non-WASM (pure Rust)
// #[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
// pub fn bar() -> u32 { 0 }
//
// impl MyStruct {
//     // Here, specify methods with types that are strictly NOT WASM-compatible.
// }
//
// #[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
// impl MyStruct {
//     // Here, specify methods with types that are strictly WASM-compatible, but not WASM-specific.
// }
//
// // Trait implementations for types can NEVER be made available for WASM
// impl std::string::ToString for MyStruct {
//     fn to_string(&self) -> String {
//         format!("{}", self.foo)
//     }
// }

//
// /// Module for WASM-specific Rust code
// #[cfg(feature = "wasm")]
// pub mod wasm {
//
//      // Use this module to specify everything that is WASM-specific (e.g. uses wasm-bindgen types, js_sys, ...etc.)
//
//     use super::*;
//     use wasm_bindgen::prelude::*;
//     use wasm_bindgen::JsValue;
//
//     #[wasm_bindgen]
//     pub fn foo(_val: JsValue) -> u32 {
//         super::foo()
//     }
//
//     #[wasm_bindgen]
//     impl MyStruct {
//         // Specify methods of MyStruct which use WASM-specific
//     }
//
// }
//
