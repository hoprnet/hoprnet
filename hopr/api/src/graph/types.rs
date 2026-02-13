use hopr_crypto_types::types::OffchainPublicKey;

#[derive(thiserror::Error, Debug)]
pub enum NetworkGraphError<P>
where
    P: MeasurablePath,
{
    #[error("timed out for near neighbor probe '{0:?}'")]
    ProbeNeighborTimeout(Box<OffchainPublicKey>),

    #[error("timed out for loopback probe")]
    ProbeLoopbackTimeout(P),
}

pub trait MeasurableNode: Into<OffchainPublicKey> {}

impl<T: Into<OffchainPublicKey>> MeasurableNode for T {}

/// Measurable peer attributes.
pub trait MeasurablePeer {
    fn peer(&self) -> &OffchainPublicKey;
    fn rtt(&self) -> std::time::Duration;
}

/// Measurable path telemetry.
pub trait MeasurablePath {
    fn id(&self) -> &[u8];
    fn path(&self) -> &[u8];
    fn timestamp(&self) -> u128;
}

#[derive(Debug, Copy, Clone)]
pub struct EdgeCapacityUpdate {
    pub capacity: Option<u128>,
    pub src: OffchainPublicKey,
    pub dest: OffchainPublicKey,
}

#[derive(Debug)]
pub enum MeasurableEdge<N, P>
where
    N: MeasurablePeer + Clone,
    P: MeasurablePath + Clone,
{
    Probe(std::result::Result<EdgeTransportTelemetry<N, P>, NetworkGraphError<P>>),
    Capacity(Box<EdgeCapacityUpdate>),
}

/// Enum representing different types of telemetry data used by the CT mechanism.
#[derive(Debug, Clone)]
pub enum EdgeTransportTelemetry<N, P>
where
    N: MeasurablePeer + Clone,
    P: MeasurablePath + Clone,
{
    /// Telemetry data looping the traffic through multiple peers back to self.
    ///
    /// Does not require a cooperating peer.
    Loopback(P),
    /// Immediate neighbor telemetry data.
    ///
    /// Assumes a cooperating immediate peer to receive responses for telemetry construction
    Neighbor(N),
}
