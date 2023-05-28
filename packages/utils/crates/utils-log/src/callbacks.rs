#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
extern "C" {
    #[cfg(feature = "wasm")]
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    pub fn js_log(s: &str);
}

#[inline]
pub fn timestamp_iso() -> String {
    js_sys::Date::new_0().to_iso_string().into()
}

#[inline]
pub fn log_natural(level: log::Level, line: &str) {
    log::log!(level, "{}", line)
}
