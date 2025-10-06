pub mod cover_traffic {
    use hopr_transport_session::ResolvedTransportRouting;

    pub use hopr_transport_probe::content::Message as CoverTrafficPayload;

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
        fn build(
            self,
        ) -> (
            impl futures::Stream<Item = (ResolvedTransportRouting, CoverTrafficPayload)>,
            impl futures::Sink<CoverTrafficPayload, Error = impl std::error::Error>,
        );
    }
}

pub mod path {
    
}
