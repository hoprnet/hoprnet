use hopr_api::ct::{DestinationRouting, NetworkGraphView, TrafficGeneration};

// TODO: replace with impl on usage site
pub struct DummyCoverTrafficType {
    #[allow(dead_code)]
    _unconstructable: (),
}

pub struct DummyNetworkGraphView {
    #[allow(dead_code)]
    _unconstructable: (),
}

impl TrafficGeneration for DummyCoverTrafficType {
    fn build<T>(self, _network_graph: T) -> impl futures::Stream<Item = DestinationRouting> + Send
    where
        T: NetworkGraphView + Send + Sync + 'static,
    {
        futures::stream::empty()
    }
}
