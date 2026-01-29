pub use hopr_network_types::types::DestinationRouting;

pub use crate::graph::traits::NetworkGraphView;

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
