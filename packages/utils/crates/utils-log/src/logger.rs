#[cfg(feature = "wasm")]
use log::{Level, Log, Metadata, Record, SetLoggerError};

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::wasm_bindgen;

/// Logging backend that passes output to `console.log`
#[cfg(feature = "wasm")]
pub struct JsLogger {}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console, js_name = "log")]
    pub fn js_log(s: &str);
}

#[cfg(feature = "wasm")]
impl JsLogger {
    /// Install this logger as a backend with optional maximum level.
    /// Maximum level defaults to DEBUG if not set (note: ERROR is the lowest, TRACE is the highest)
    pub fn install(logger: &'static JsLogger, max_level: Option<Level>) -> Result<(), SetLoggerError> {
        log::set_logger(logger).map(|_| log::set_max_level(max_level.unwrap_or(Level::Debug).to_level_filter()))
    }
}

#[cfg(feature = "wasm")]
impl Log for JsLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &Record) {
        let ts: String = js_sys::Date::new_0().to_iso_string().into();
        js_log(&format!(
            "{ts} [{}] {} {}",
            record.level(),
            record.target(),
            record.args()
        ));
    }

    fn flush(&self) {}
}
