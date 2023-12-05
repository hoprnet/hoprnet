//! http-client implementation for curl + fetch

#[cfg(all(feature = "curl_client", not(target_arch = "wasm32")))]
pub use super::isahc::IsahcClient as NativeClient;

#[cfg(all(feature = "wasm_client", target_arch = "wasm32"))]
pub use super::wasm::WasmClient as NativeClient;
