#[cfg(feature = "wasm")]
pub mod wasm {
    // Temporarily re-export crates

    #[allow(unused_imports)]
    use core_strategy::passive::wasm::*;

    #[allow(unused_imports)]
    use core_strategy::promiscuous::wasm::*;

    #[allow(unused_imports)]
    //use core_misc::constants::wasm::*;
    #[allow(unused_imports)]
    use core_network::network::wasm::*;

    #[allow(unused_imports)]
    use core_ethereum_db::db::wasm::*;

    #[allow(unused_imports)]
    use core_ethereum_misc::chain::wasm::*;

    #[allow(unused_imports)]
    use core_ethereum_misc::constants::wasm::*;

    #[allow(unused_imports)]
    use core_ethereum_indexer::handlers::wasm::*;

    #[allow(unused_imports)]
    use core_misc::environment::wasm::*;

    #[allow(unused_imports)]
    use core_hopr::wasm::*;

    #[allow(unused_imports)]
    use hoprd_misc::cli::wasm::*;

    #[allow(unused_imports)]
    use hoprd_inbox::inbox::wasm::*;

    #[allow(unused_imports)]
    use hoprd_keypair::key_pair::wasm::*;

    use utils_log::logger::wasm::JsLogger;

    #[allow(unused_imports)]
    use utils_misc::utils::wasm::*;

    #[allow(unused_imports)]
    use utils_metrics::metrics::wasm::*;

    use wasm_bindgen::prelude::wasm_bindgen;

    static LOGGER: JsLogger = JsLogger {};

    #[allow(dead_code)]
    #[wasm_bindgen]
    pub fn hoprd_hoprd_initialize_crate() {
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

    #[wasm_bindgen]
    pub fn hoprd_hoprd_gather_metrics() -> JsResult<String> {
        utils_metrics::metrics::wasm::gather_all_metrics()
    }

    // When the `wee_alloc` feature is enabled, use `wee_alloc` as the global allocator.
    #[cfg(feature = "wee_alloc")]
    #[global_allocator]
    static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
}
