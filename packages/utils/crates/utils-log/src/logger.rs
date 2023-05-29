use log::{Level, Log, Metadata, Record};
use wasm_bindgen::prelude::wasm_bindgen;

/// Logging backend that passes output to `console.log`
pub struct JsLogger {}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);
}

impl JsLogger {
    /// Install this logger as a backend with optional maximum level.
    pub fn install(logger: &'static JsLogger, max_level: Option<Level>) -> Result<(), String> {
        log::set_logger(logger).map_err(|e| e.to_string())?;
        if let Some(lvl) = max_level {
            log::set_max_level(lvl.to_level_filter());
        }
        Ok(())
    }
}

impl Log for JsLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &Record) {
        let ts: String = js_sys::Date::new_0().to_iso_string().into();
        log(&format!("{ts} [{}] {} {}", record.level(), record.target(), record.args()));
    }

    fn flush(&self) { }
}