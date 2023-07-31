pub mod native {
    pub fn current_timestamp() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0u64)
    }
}
#[cfg(feature = "wasm")]
pub mod wasm {
    pub fn current_timestamp() -> u64 {
        js_sys::Date::now() as u64
    }
}
