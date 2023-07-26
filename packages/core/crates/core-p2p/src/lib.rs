pub use libp2p_identity as identity;

use libp2p_identity::PeerId;
use libp2p_core::{upgrade, Transport};
use libp2p_swarm::{SwarmBuilder, SwarmEvent};

pub fn build_p2p_network(me: &PeerId) -> libp2p_swarm::Swarm<libp2p_swarm::dummy::Behaviour> {
    // TODO: this needs to be passed from above, packet key
    let id_keys = libp2p_identity::Keypair::generate_ed25519();

    let transport = libp2p_wasm_ext::ExtTransport::new(libp2p_wasm_ext::ffi::tcp_transport())
        .upgrade(upgrade::Version::V1)
        .authenticate(libp2p_noise::Config::new(&id_keys).expect("signing libp2p-noise static keypair"))
        .multiplex(libp2p_mplex::MplexConfig::default())
        .timeout(std::time::Duration::from_secs(20))
        .boxed();

    let network_behavior = libp2p_swarm::dummy::Behaviour;

    SwarmBuilder::with_wasm_executor(transport, network_behavior, me.clone()).build()
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use utils_log::logger::JsLogger;
    // use utils_misc::utils::wasm::JsResult;
    use wasm_bindgen::prelude::*;

    // When the `wee_alloc` feature is enabled, use `wee_alloc` as the global allocator.
    #[cfg(feature = "wee_alloc")]
    #[global_allocator]
    static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

    static LOGGER: JsLogger = JsLogger {};

    #[allow(dead_code)]
    #[wasm_bindgen]
    pub fn core_p2p_initialize_crate() {
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

    // #[wasm_bindgen]
    // pub fn core_p2p_gather_metrics() -> JsResult<String> {
    //     utils_metrics::metrics::wasm::gather_all_metrics()
    // }
}
