pub mod native {
    pub fn current_timestamp() -> std::time::Duration {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| crate::error::PlatformError::TimeError(e.to_string()))
            .unwrap_or(std::time::Duration::from_secs(0)) // system time is positive after unix epoch
    }
}
#[cfg(feature = "js")]
pub mod wasm {
    pub fn current_timestamp() -> std::time::Duration {
        Ok(std::time::Duration::from_millis(js_sys::Date::now() as u128))
    }
}
