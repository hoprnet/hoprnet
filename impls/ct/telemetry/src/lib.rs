use hopr_api::ct::{DestinationRouting, Telemetry, TrafficGeneration, traits::TrafficGenerationError};

// TODO: replace with impl on usage site
pub struct DummyCoverTrafficType {
    #[allow(dead_code)]
    _unconstructable: (),
}

impl TrafficGeneration for DummyCoverTrafficType {
    fn build(
        self,
    ) -> (
        impl futures::Stream<Item = DestinationRouting> + Send,
        impl futures::Sink<std::result::Result<Telemetry, TrafficGenerationError>, Error = impl std::error::Error>
        + Send
        + Sync
        + Clone
        + 'static,
    ) {
        (
            futures::stream::empty(),
            futures::sink::drain::<std::result::Result<Telemetry, TrafficGenerationError>>(),
        )
    }
}
