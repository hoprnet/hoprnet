use hopr_primitive_types::prelude::GeneralError;
use multiaddr::PeerId;

#[derive(thiserror::Error, Debug)]
pub enum TrafficGenerationError {
    #[error("timed out for near neighbor probe '{0:?}'")]
    ProbeNeighborTimeout(PeerId),

    #[error("timed out for loopback probe")]
    ProbeLoopbackTimeout(PathTelemetry),
}

/// Serializable and deserializable enum for the probe message content.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, strum::EnumDiscriminants)]
#[strum_discriminants(vis(pub(crate)))]
#[strum_discriminants(derive(strum::FromRepr, strum::EnumCount), repr(u8))]
pub enum NeighborProbe {
    /// Ping message with random nonce
    Ping([u8; Self::NONCE_SIZE]),
    /// Pong message replying to a specific nonce
    Pong([u8; Self::NONCE_SIZE]),
}

impl NeighborProbe {
    pub const NONCE_SIZE: usize = 32;
    pub const SIZE: usize = 1 + Self::NONCE_SIZE;

    /// Creates a new Ping probe with a random nonce
    pub fn random_nonce() -> Self {
        Self::Ping(hopr_crypto_random::random_bytes::<{ Self::NONCE_SIZE }>())
    }

    pub fn is_complement_to(&self, other: Self) -> bool {
        match (self, &other) {
            (Self::Ping(nonce1), Self::Pong(nonce2)) => nonce1 == nonce2,
            (Self::Pong(nonce1), Self::Ping(nonce2)) => nonce1 == nonce2,
            _ => false,
        }
    }

    pub fn to_bytes(self) -> Box<[u8]> {
        let mut out = Vec::with_capacity(1 + Self::NONCE_SIZE);
        out.push(NeighborProbeDiscriminants::from(&self) as u8);
        out.extend_from_slice(self.as_ref());
        out.into_boxed_slice()
    }
}

impl<'a> TryFrom<&'a [u8]> for NeighborProbe {
    type Error = GeneralError;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        if value.len() == 1 + Self::NONCE_SIZE {
            match NeighborProbeDiscriminants::from_repr(value[0])
                .ok_or(GeneralError::ParseError("NeighborProbe.disc".into()))?
            {
                NeighborProbeDiscriminants::Ping => {
                    Ok(Self::Ping((&value[1..]).try_into().map_err(|_| {
                        GeneralError::ParseError("NeighborProbe.ping_nonce".into())
                    })?))
                }
                NeighborProbeDiscriminants::Pong => {
                    Ok(Self::Pong((&value[1..]).try_into().map_err(|_| {
                        GeneralError::ParseError("NeighborProbe.pong_nonce".into())
                    })?))
                }
            }
        } else {
            Err(GeneralError::ParseError("NeighborProbe.size".into()))
        }
    }
}

impl std::fmt::Display for NeighborProbe {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NeighborProbe::Ping(nonce) => write!(f, "Ping({})", hex::encode(nonce)),
            NeighborProbe::Pong(nonce) => write!(f, "Pong({})", hex::encode(nonce)),
        }
    }
}

impl AsRef<[u8]> for NeighborProbe {
    fn as_ref(&self) -> &[u8] {
        match self {
            NeighborProbe::Ping(nonce) | NeighborProbe::Pong(nonce) => nonce,
        }
    }
}

/// Path telemetry data for multi-hop loopback probing.
///
/// Contains an identifier, path information, and timestamp for tracking
/// telemetry through the network back to self.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PathTelemetry {
    /// Unique identifier for the telemetry data
    pub id: [u8; Self::ID_SIZE],
    /// Path information encoded as bytes
    pub path: [u8; Self::PATH_SIZE],
    /// Timestamp when the telemetry was created
    pub timestamp: u128,
}

impl PathTelemetry {
    pub const ID_SIZE: usize = 10;
    pub const PATH_SIZE: usize = 10;
    pub const SIZE: usize = Self::ID_SIZE + Self::PATH_SIZE + size_of::<u128>();

    pub fn to_bytes(self) -> Box<[u8]> {
        let mut out = Vec::with_capacity(Self::SIZE);
        out.extend_from_slice(&self.id);
        out.extend_from_slice(&self.path);
        out.extend_from_slice(&self.timestamp.to_be_bytes());
        out.into_boxed_slice()
    }
}

const _: () = assert!(size_of::<u128>() > PathTelemetry::ID_SIZE);

impl std::fmt::Display for PathTelemetry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "PathTelemetry(id: {}, path: {}, timestamp: {})",
            hex::encode(self.id),
            hex::encode(self.path),
            self.timestamp
        )
    }
}

impl<'a> TryFrom<&'a [u8]> for PathTelemetry {
    type Error = GeneralError;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        if value.len() == Self::SIZE {
            Ok(Self {
                id: (&value[0..10])
                    .try_into()
                    .map_err(|_| GeneralError::ParseError("PathTelemetry.id".into()))?,
                path: (&value[10..20])
                    .try_into()
                    .map_err(|_| GeneralError::ParseError("PathTelemetry.path".into()))?,
                timestamp: u128::from_be_bytes(
                    (&value[20..36])
                        .try_into()
                        .map_err(|_| GeneralError::ParseError("PathTelemetry.timestamp".into()))?,
                ),
            })
        } else {
            Err(GeneralError::ParseError("PathTelemetry".into()))
        }
    }
}

/// Intermediate neighbor telemetry object.
///
/// Represents the finding of an intermediate peer probing operation.
#[derive(Debug, Clone)]
pub struct NeighborTelemetry {
    pub peer: PeerId,
    pub rtt: std::time::Duration,
}

/// Enum representing different types of telemetry data used by the CT mechanism.
#[derive(Debug, Clone)]
pub enum Telemetry {
    /// Telemetry data looping the traffic through multiple peers back to self.
    ///
    /// Does not require a cooperating peer.
    Loopback(PathTelemetry),
    /// Immediate neighbor telemetry data.
    ///
    /// Assumes a cooperating immediate peer to receive responses for telemetry construction
    Neighbor(NeighborTelemetry),
}
