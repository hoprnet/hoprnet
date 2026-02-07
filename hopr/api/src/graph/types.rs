use multiaddr::PeerId;

#[derive(thiserror::Error, Debug)]
pub enum NetworkGraphError<P>
where
    P: MeasurablePath,
{
    #[error("timed out for near neighbor probe '{0:?}'")]
    ProbeNeighborTimeout(PeerId),

    #[error("timed out for loopback probe")]
    ProbeLoopbackTimeout(P),
}

/// Measurable neighbor telemetry.
pub trait MeasurableNeighbor {
    fn peer(&self) -> &PeerId;
    fn rtt(&self) -> std::time::Duration;
}

/// Measurable path telemetry.
pub trait MeasurablePath {
    fn id(&self) -> &[u8];
    fn seq_id(&self) -> u16;
    fn path(&self) -> &[u8];
    fn timestamp(&self) -> u128;
}

/// Enum representing different types of telemetry data used by the CT mechanism.
#[derive(Debug, Clone)]
pub enum Telemetry<N, P>
where
    N: MeasurableNeighbor + Clone,
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
