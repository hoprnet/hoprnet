pub mod errors;
pub mod interaction;
pub mod packet;
pub mod por;
pub mod validation;

#[cfg(feature = "wasm")]
pub mod wasm {

    use utils_log::logger::wasm::JsLogger;
    use wasm_bindgen::prelude::*;

    // When the `wee_alloc` feature is enabled, use `wee_alloc` as the global allocator.
    #[cfg(feature = "wee_alloc")]
    #[global_allocator]
    static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

    static LOGGER: JsLogger = JsLogger {};

    #[allow(dead_code)]
    #[wasm_bindgen]
    pub fn core_packet_initialize_crate() {
        let _ = JsLogger::install(&LOGGER, None);

        // When the `console_error_panic_hook` feature is enabled, we can call the
        // `set_panic_hook` function at least once during initialization, and then
        // we will get better error messages if our code ever panics.
        //
        // For more details see
        // https://github.com/rustwasm/console_error_panic_hook#readme
        #[cfg(feature = "console_error_panic_hook")]
        console_error_panic_hook::set_once();
    }

    #[cfg(feature = "prometheus")]
    #[wasm_bindgen]
    pub fn core_packet_gather_metrics() -> utils_misc::utils::wasm::JsResult<String> {
        utils_metrics::metrics::wasm::gather_all_metrics()
    }
}
