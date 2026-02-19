use futures::stream::BoxStream;
pub use hopr_network_types::types::{DestinationRouting, PathId};

/// Enum representation of different types of [`DestinationRouting`] the
/// [`ProbingTrafficGeneration::build`] is able to generate.
///
/// [`ProbeRouting::Neighbor`] corresponds to an in-protocol neighbor
/// discovery relying on neighbor cooperation.
/// [`ProbeRouting::Looping`] represents a unidirectional probe transparent
/// to the relayer, starting and ending in the emitter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProbeRouting {
    Neighbor(DestinationRouting),
    Looping((DestinationRouting, PathId)),
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
pub trait ProbingTrafficGeneration {
    /// Builds a stream of routes to be used for probing traffic.
    ///
    /// The stream should be neverending for continual probing traffic generation.
    fn build(&self) -> BoxStream<'static, ProbeRouting>;
}

/// A trait for types that can generate cover traffic routes for the HOPR network.
///
/// Implementors of this trait are responsible for producing an infinite stream
/// of `DestinationRouting` values, each representing a route for a cover (non-user) traffic.
/// Cover traffic is essential for privacy and network health, as it helps to obscure real user
/// traffic and maintain plausible deniability for network participants.
///
/// # Requirements
/// - The stream produced by `build` should not terminate under normal operation, as the termination will lead to the
///   cessation of cover traffic generation.
/// - Route selection should be randomized or follow a strategy that maximizes privacy and/or network utility.
pub trait CoverTrafficGeneration {
    /// Builds a stream of routes to be used for cover traffic.
    ///
    /// The stream should be neverending for continual cover traffic generation.
    fn build(&self) -> BoxStream<'static, DestinationRouting>;
}
