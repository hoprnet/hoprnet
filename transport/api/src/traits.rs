use hopr_transport_identity::PeerId;

/// Representation of the route inside an oriented graph that
/// is the mix network.
pub struct Route {
    path: Vec<PeerId>,
}

impl Route {
    pub fn path(&self) -> &[PeerId] {
        &self.path
    }
}

impl<T> From<T> for Route
where
    T: AsRef<[PeerId]>,
{
    fn from(value: T) -> Self {
        Self {
            path: value.as_ref().to_vec(),
        }
    }
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
pub trait CoverTraffic {
    fn routes(self) -> impl futures::Stream<Item = Route>;
}
