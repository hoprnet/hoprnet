
#[cfg(not(wasm))]
pub fn current_timestamp() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(d) => d.as_millis() as u64,
        Err(_) => 1,
    }
}


#[cfg(wasm)]
use wasm::current_timestamp;


#[cfg(wasm)]
mod wasm {
    pub fn current_timestamp() -> u64 {
        (js_sys::Date::now() / 1000.0) as u64
    }
}
