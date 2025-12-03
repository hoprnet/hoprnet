use multiaddr::PeerId;

pub use hopr_network_types::types::DestinationRouting;

use super::{PathTelemetry, Telemetry};

#[derive(thiserror::Error, Debug)]
pub enum TrafficGenerationError {
    #[error("timed out for near neighbor probe '{0:?}'")]
    ProbeNeighborTimeout(PeerId),

    #[error("timed out for loopback probe")]
    ProbeLoopbackTimeout(PathTelemetry),
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
        impl futures::Sink<std::result::Result<Telemetry, TrafficGenerationError>, Error = impl std::error::Error>
        + Send
        + Sync
        + Clone
        + 'static,
    );
}
