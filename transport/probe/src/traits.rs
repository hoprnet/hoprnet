use async_trait::async_trait;
use hopr_network_types::types::DestinationRouting;
use libp2p_identity::PeerId;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait PeerDiscoveryFetch {
    /// Get untested peers not observed since a specific timestamp.
    async fn get_peers(&self, from_timestamp: std::time::SystemTime) -> Vec<PeerId>;
}

#[async_trait]
pub trait ProbeStatusUpdate {
    /// Update the peer status after probing
    async fn on_finished(&self, peer: &PeerId, result: &crate::errors::Result<std::time::Duration>);
}

/// A trait for types that can produce a stream of cover traffic routes.
///
/// The basic assumption is that the implementor will provide the logic
/// to choose suitable route candidates for cover traffic based on a
/// custom algorithm.
///
/// The implementor should ensure that the produced routes are indefinite,
/// since the exhaustion of the stream might result in termination of the
/// cover traffic generation.
pub trait TrafficGeneration {
    fn build(
        self,
    ) -> (
        impl futures::Stream<Item = DestinationRouting> + Send,
        impl futures::Sink<crate::errors::Result<crate::types::Telemetry>, Error = impl std::error::Error>
        + Send
        + Sync
        + Clone
        + 'static,
    );
}

const _: () = assert!(size_of::<u128>() > crate::content::PathTelemetry::ID_SIZE);
