#[cfg(any(not(feature = "wasm"), test))]
pub fn current_timestamp() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(d) => d.as_millis() as u64,
        Err(_) => 1,
    }
}

#[cfg(any(feature = "wasm", not(test)))]
pub use wasm::current_timestamp;

#[cfg(any(feature = "wasm", not(test)))]
mod wasm {
    pub fn current_timestamp() -> u64 {
        js_sys::Date::now() as u64
    }
}
