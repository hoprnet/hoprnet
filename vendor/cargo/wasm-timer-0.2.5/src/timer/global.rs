pub(crate) use self::platform::*;

#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
#[path = "global/desktop.rs"]
mod platform;
#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
#[path = "global/wasm.rs"]
mod platform;
