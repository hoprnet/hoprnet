//! The summary a node logs once at startup.
//!
//! A log, read on its own, must say which build, identity and settings produced it: otherwise it
//! can be neither analysed nor reproduced. The summary holds only what is known locally, so it is
//! logged before anything that may fail or hang, such as reaching the chain.

use std::time::Duration;

use hopr_api::types::{
    crypto::{
        keypairs::Keypair,
        prelude::{ChainKeypair, OffchainKeypair},
    },
    internal::prelude::WinningProbability,
    primitive::prelude::{Address, HoprBalance},
};

use crate::{config::HoprLibConfig, constants};

/// Build, identity and main settings of a starting node.
#[derive(Clone, Debug)]
pub(crate) struct StartupSummary {
    version: &'static str,
    /// Enabled crate features that change the node's behavior.
    features: Vec<&'static str>,
    /// The node's peer id, derived from its packet key. Transport logs name peers by it.
    peer: String,
    /// The node's on-chain address, derived from its chain key. On-chain analysis starts from it.
    peer_addr: Address,
    safe: Address,
    module: Address,
    host: String,
    publish: bool,
    outgoing_ticket_price: Option<HoprBalance>,
    outgoing_win_prob: Option<WinningProbability>,
    max_sessions: usize,
    session_idle_timeout: Duration,
    probe_interval: Duration,
    /// Which mixer runs, with its delay settings.
    mixer: String,
}

impl StartupSummary {
    pub(crate) fn new(cfg: &HoprLibConfig, chain_key: &ChainKeypair, packet_key: &OffchainKeypair) -> Self {
        Self {
            version: constants::APP_VERSION,
            features: enabled_features(),
            peer: packet_key.public().to_peerid_str(),
            peer_addr: chain_key.public().to_address(),
            safe: cfg.safe_module.safe_address,
            module: cfg.safe_module.module_address,
            host: cfg.host.to_string(),
            publish: cfg.publish,
            outgoing_ticket_price: cfg.protocol.packet.codec.outgoing_ticket_price,
            outgoing_win_prob: cfg.protocol.packet.codec.outgoing_win_prob,
            max_sessions: cfg.protocol.session.maximum_managed_sessions,
            session_idle_timeout: cfg.protocol.session.idle_timeout,
            probe_interval: cfg.protocol.probe.interval,
            mixer: format!("{:?}", cfg.protocol.mixer.mixer_type),
        }
    }

    /// Logs the summary as one INFO line, and the whole configuration at DEBUG.
    pub(crate) fn log(&self, cfg: &HoprLibConfig) {
        tracing::info!(
            version = self.version,
            features = ?self.features,
            peer = %self.peer,
            peer_addr = %self.peer_addr,
            safe = %self.safe,
            module = %self.module,
            host = %self.host,
            publish = self.publish,
            outgoing_ticket_price = ?self.outgoing_ticket_price.map(|price| price.to_string()),
            outgoing_win_prob = ?self.outgoing_win_prob.map(|prob| prob.to_string()),
            max_sessions = self.max_sessions,
            session_idle_timeout = ?self.session_idle_timeout,
            probe_interval = ?self.probe_interval,
            mixer = %self.mixer,
            "starting node"
        );
        tracing::debug!(config = ?cfg, "node configuration");
    }
}

/// Crate features that change what a node does, as far as they are enabled in this build.
fn enabled_features() -> Vec<&'static str> {
    [
        (cfg!(feature = "session-client"), "session-client"),
        (cfg!(feature = "session-server"), "session-server"),
        (cfg!(feature = "pix-bjj"), "pix-bjj"),
        (cfg!(feature = "pix-secp256k1"), "pix-secp256k1"),
        (cfg!(feature = "transport-announce-quic"), "transport-announce-quic"),
        (cfg!(feature = "telemetry"), "telemetry"),
        (cfg!(feature = "testing"), "testing"),
        (cfg!(debug_assertions), "debug-assertions"),
    ]
    .into_iter()
    .filter_map(|(enabled, name)| enabled.then_some(name))
    .collect()
}

#[cfg(test)]
mod tests {
    use hopr_api::types::crypto::{
        keypairs::Keypair,
        prelude::{ChainKeypair, OffchainKeypair},
    };

    use super::{StartupSummary, enabled_features};
    use crate::config::HoprLibConfig;

    #[test_log::test]
    fn the_summary_names_the_node_by_both_its_peer_id_and_its_chain_address() {
        let chain_key = ChainKeypair::random();
        let packet_key = OffchainKeypair::random();
        let cfg = HoprLibConfig::default();

        let summary = StartupSummary::new(&cfg, &chain_key, &packet_key);

        assert_eq!(summary.peer, packet_key.public().to_peerid_str());
        assert_eq!(summary.peer_addr, chain_key.public().to_address());
        assert_eq!(summary.version, env!("CARGO_PKG_VERSION"));
        summary.log(&cfg);
    }

    #[test]
    fn the_summary_takes_the_settings_from_the_configuration() {
        let mut cfg = HoprLibConfig {
            publish: true,
            ..Default::default()
        };
        cfg.protocol.session.maximum_managed_sessions = 7;

        let summary = StartupSummary::new(&cfg, &ChainKeypair::random(), &OffchainKeypair::random());

        assert!(summary.publish);
        assert_eq!(summary.max_sessions, 7);
        assert_eq!(summary.host, cfg.host.to_string());
        assert_eq!(summary.safe, cfg.safe_module.safe_address);
        assert_eq!(summary.probe_interval, cfg.protocol.probe.interval);
    }

    #[test]
    fn the_feature_list_follows_the_build() {
        let features = enabled_features();

        assert_eq!(features.contains(&"testing"), cfg!(feature = "testing"));
        assert_eq!(features.contains(&"debug-assertions"), cfg!(debug_assertions));
    }
}
