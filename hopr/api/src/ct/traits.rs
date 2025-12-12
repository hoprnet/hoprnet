use futures::Stream;
pub use hopr_network_types::types::DestinationRouting;
use multiaddr::PeerId;

use super::PathTelemetry;

#[derive(thiserror::Error, Debug)]
pub enum TrafficGenerationError {
    #[error("timed out for near neighbor probe '{0:?}'")]
    ProbeNeighborTimeout(PeerId),

    #[error("timed out for loopback probe")]
    ProbeLoopbackTimeout(PathTelemetry),
}

/// A trait specifying the graph traversal functionality
#[async_trait::async_trait]
pub trait NetworkGraphView {
    async fn nodes(&self) -> impl Stream<Item = PeerId>;

    async fn find_routes_to(&self, destination: &PeerId, length: usize) -> Vec<DestinationRouting>;
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
    fn build<T>(self, network_graph: T) -> impl futures::Stream<Item = DestinationRouting> + Send
    where
        T: NetworkGraphView + Send + Sync + 'static;
}
