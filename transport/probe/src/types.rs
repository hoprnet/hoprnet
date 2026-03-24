use hopr_api::{
    OffchainPublicKey,
    types::{
        crypto_random::Randomizable,
        internal::{
            NodeId,
            protocol::HoprPseudonym,
            routing::{DestinationRouting, PathId, RoutingOptions},
        },
        primitive::{bounded::BoundedVec, errors::GeneralError},
    },
};

pub struct TaggedDestinationRouting {
    /// The destination node.
    pub destination: Box<NodeId>,
    /// Pseudonym shown to the destination.
    pub pseudonym: HoprPseudonym,
    /// The path to the destination.
    pub forward_options: RoutingOptions,
    /// Optional return path.
    pub return_options: Option<RoutingOptions>,
}

impl TaggedDestinationRouting {
    pub fn neighbor(destination: Box<NodeId>) -> Self {
        Self {
            destination,
            pseudonym: HoprPseudonym::random(),
            forward_options: RoutingOptions::Hops(0.try_into().expect("0 is a valid u8")),
            return_options: Some(RoutingOptions::Hops(0.try_into().expect("0 is a valid u8"))),
        }
    }

    pub fn loopback(me: Box<NodeId>, path: BoundedVec<NodeId, { RoutingOptions::MAX_INTERMEDIATE_HOPS }>) -> Self {
        Self {
            destination: me,
            pseudonym: HoprPseudonym::random(),
            forward_options: RoutingOptions::IntermediatePath(path),
            return_options: None,
        }
    }
}

impl From<TaggedDestinationRouting> for DestinationRouting {
    fn from(value: TaggedDestinationRouting) -> Self {
        DestinationRouting::Forward {
            destination: value.destination,
            pseudonym: Some(value.pseudonym),
            forward_options: value.forward_options,
            return_options: value.return_options,
        }
    }
}

/// Serializable and deserializable enum for the probe message content.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, strum::EnumDiscriminants)]
#[strum_discriminants(vis(pub(crate)), derive(strum::FromRepr, strum::EnumCount), repr(u8))]
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
        Self::Ping(hopr_api::types::crypto_random::random_bytes::<{ Self::NONCE_SIZE }>())
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
    pub const ID_SIZE: usize = 8;
    pub const PATH_SIZE: usize = size_of::<PathId>();
    pub const SIZE: usize = Self::ID_SIZE + Self::PATH_SIZE + size_of::<u128>();

    pub fn to_bytes(self) -> Box<[u8]> {
        let mut out = Vec::with_capacity(Self::SIZE);
        out.extend_from_slice(&self.id);
        out.extend_from_slice(&self.path);
        out.extend_from_slice(&self.timestamp.to_be_bytes());
        out.into_boxed_slice()
    }
}

impl hopr_api::graph::MeasurablePath for PathTelemetry {
    fn id(&self) -> &[u8] {
        &self.id
    }

    fn path(&self) -> &[u8] {
        &self.path
    }

    fn timestamp(&self) -> u128 {
        self.timestamp
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
                id: (&value[0..Self::ID_SIZE])
                    .try_into()
                    .map_err(|_| GeneralError::ParseError("PathTelemetry.id".into()))?,
                path: (&value[Self::ID_SIZE..(Self::ID_SIZE + Self::PATH_SIZE)])
                    .try_into()
                    .map_err(|_| GeneralError::ParseError("PathTelemetry.path".into()))?,
                timestamp: u128::from_be_bytes(
                    (&value[(Self::ID_SIZE + Self::PATH_SIZE)..Self::SIZE])
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
    pub peer: OffchainPublicKey,
    pub rtt: std::time::Duration,
}

impl hopr_api::graph::MeasurablePeer for NeighborTelemetry {
    fn peer(&self) -> &OffchainPublicKey {
        &self.peer
    }

    fn rtt(&self) -> std::time::Duration {
        self.rtt
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Context;
    use hopr_api::types::crypto::prelude::{Keypair, OffchainKeypair};
    use rstest::rstest;

    use super::*;

    #[test]
    fn tagged_routing_neighbor_should_create_zero_hop_forward() -> anyhow::Result<()> {
        let dest = Box::new(NodeId::Offchain(*OffchainKeypair::random().public()));
        let routing = TaggedDestinationRouting::neighbor(dest.clone());

        anyhow::ensure!(
            matches!(routing.forward_options, RoutingOptions::Hops(h) if u8::from(h) == 0),
            "neighbor forward should be Hops(0)"
        );
        anyhow::ensure!(
            matches!(&routing.return_options, Some(RoutingOptions::Hops(h)) if u8::from(*h) == 0),
            "neighbor return should be Some(Hops(0))"
        );
        anyhow::ensure!(*routing.destination == *dest, "destination mismatch");
        Ok(())
    }

    #[test]
    fn tagged_routing_loopback_should_create_intermediate_path() -> anyhow::Result<()> {
        let me = Box::new(NodeId::Offchain(*OffchainKeypair::random().public()));
        let path = BoundedVec::try_from(vec![NodeId::Offchain(*OffchainKeypair::random().public())])
            .context("building path")?;

        let routing = TaggedDestinationRouting::loopback(me.clone(), path);

        anyhow::ensure!(
            matches!(routing.forward_options, RoutingOptions::IntermediatePath(_)),
            "loopback forward should be IntermediatePath"
        );
        anyhow::ensure!(routing.return_options.is_none(), "loopback should have no return");
        anyhow::ensure!(*routing.destination == *me, "destination mismatch");
        Ok(())
    }

    #[test]
    fn tagged_routing_should_convert_to_destination_routing() -> anyhow::Result<()> {
        let dest = Box::new(NodeId::Offchain(*OffchainKeypair::random().public()));
        let routing = TaggedDestinationRouting::neighbor(dest);
        let converted: DestinationRouting = routing.into();

        anyhow::ensure!(
            matches!(converted, DestinationRouting::Forward { .. }),
            "conversion should produce Forward variant"
        );
        Ok(())
    }

    #[rstest]
    #[case::ping_ping(
        NeighborProbe::Ping([1u8; NeighborProbe::NONCE_SIZE]),
        NeighborProbe::Ping([1u8; NeighborProbe::NONCE_SIZE])
    )]
    #[case::pong_pong(
        NeighborProbe::Pong([2u8; NeighborProbe::NONCE_SIZE]),
        NeighborProbe::Pong([2u8; NeighborProbe::NONCE_SIZE])
    )]
    fn neighbor_probe_complement_should_return_false_when_same_variant(
        #[case] a: NeighborProbe,
        #[case] b: NeighborProbe,
    ) {
        assert!(!a.is_complement_to(b));
    }

    #[test]
    fn neighbor_probe_deserialization_should_roundtrip_ping() -> anyhow::Result<()> {
        let ping = NeighborProbe::Ping([42u8; NeighborProbe::NONCE_SIZE]);
        let bytes = ping.to_bytes();
        let restored = NeighborProbe::try_from(bytes.as_ref()).context("deserializing ping")?;
        assert_eq!(ping, restored);
        Ok(())
    }

    #[test]
    fn neighbor_probe_deserialization_should_roundtrip_pong() -> anyhow::Result<()> {
        let pong = NeighborProbe::Pong([99u8; NeighborProbe::NONCE_SIZE]);
        let bytes = pong.to_bytes();
        let restored = NeighborProbe::try_from(bytes.as_ref()).context("deserializing pong")?;
        assert_eq!(pong, restored);
        Ok(())
    }

    #[test]
    fn neighbor_probe_deserialization_should_fail_on_wrong_size() {
        let short = [0u8; 5];
        assert!(matches!(
            NeighborProbe::try_from(short.as_slice()),
            Err(GeneralError::ParseError(ref s)) if s.contains("size")
        ));
    }

    #[test]
    fn neighbor_probe_display_should_show_variant_and_hex() {
        let nonce = [0xABu8; NeighborProbe::NONCE_SIZE];
        let ping = NeighborProbe::Ping(nonce);
        let pong = NeighborProbe::Pong(nonce);

        let ping_str = ping.to_string();
        let pong_str = pong.to_string();

        assert!(ping_str.starts_with("Ping("), "got: {ping_str}");
        assert!(pong_str.starts_with("Pong("), "got: {pong_str}");
        assert!(ping_str.contains("abab"), "should contain hex nonce");
    }

    #[test]
    fn path_telemetry_roundtrip_should_preserve_all_fields() -> anyhow::Result<()> {
        let telemetry = PathTelemetry {
            id: [1, 2, 3, 4, 5, 6, 7, 8],
            path: [0xFFu8; PathTelemetry::PATH_SIZE],
            timestamp: 1234567890,
        };
        let bytes = telemetry.to_bytes();
        let restored = PathTelemetry::try_from(bytes.as_ref()).context("deserializing telemetry")?;
        assert_eq!(telemetry, restored);
        Ok(())
    }

    #[test]
    fn path_telemetry_display_should_include_hex_id_and_timestamp() {
        let telemetry = PathTelemetry {
            id: [0xDE, 0xAD, 0xBE, 0xEF, 0xCA, 0xFE, 0xBA, 0xBE],
            path: [0u8; PathTelemetry::PATH_SIZE],
            timestamp: 42,
        };
        let display = telemetry.to_string();
        assert!(display.contains("deadbeefcafebabe"), "should contain hex id: {display}");
        assert!(display.contains("42"), "should contain timestamp: {display}");
    }
}
