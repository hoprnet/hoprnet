pub mod native {
    pub fn current_timestamp() -> std::time::SystemTime {
        std::time::SystemTime::now()
    }
}
#[cfg(feature = "js")]
pub mod wasm {
    pub fn current_timestamp() -> std::time::SystemTime {
        use std::ops::Add;
        std::time::SystemTime::UNIX_EPOCH.add(std::time::Duration::from_millis(js_sys::Date::now() as u64))
    }
}
