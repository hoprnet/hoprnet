use hopr_primitive_types::prelude::GeneralError;
use hopr_protocol_app::prelude::{ApplicationData, ReservedTag, Tag};

use crate::errors::ProbeError;

/// Serializable and deserializable enum for the probe message content
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

    /// Returns the nonce of the message
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

/// TODO: TBD as a separate task for network discovery
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PathTelemetry {
    pub id: [u8; Self::ID_SIZE],
    pub path: [u8; Self::PATH_SIZE],
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, strum::EnumDiscriminants, strum::Display)]
#[strum_discriminants(vis(pub(crate)))]
#[strum_discriminants(derive(strum::FromRepr, strum::EnumCount), repr(u8))]
pub enum Message {
    Telemetry(PathTelemetry),
    Probe(NeighborProbe),
}

impl Message {
    pub const VERSION: u8 = 1;

    pub fn to_bytes(self) -> Box<[u8]> {
        let mut out = Vec::<u8>::with_capacity(1 + NeighborProbe::SIZE.max(PathTelemetry::SIZE));
        out.push(Self::VERSION);
        out.push(MessageDiscriminants::from(&self) as u8);

        match self {
            Message::Telemetry(telemetry) => out.extend(telemetry.to_bytes()),
            Message::Probe(probe) => out.extend(probe.to_bytes()),
        }

        out.into_boxed_slice()
    }
}

impl<'a> TryFrom<&'a [u8]> for Message {
    type Error = GeneralError;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Err(GeneralError::ParseError("Message.size".into()));
        }

        if value[0] != Self::VERSION {
            return Err(GeneralError::ParseError("Message.version".into()));
        }

        match MessageDiscriminants::from_repr(value[1]).ok_or(GeneralError::ParseError("Message.disc".into()))? {
            MessageDiscriminants::Telemetry => {
                if value.len() == 2 + PathTelemetry::SIZE {
                    Ok(Self::Telemetry(
                        (&value[2..])
                            .try_into()
                            .map_err(|_| GeneralError::ParseError("Message.telemetry".into()))?,
                    ))
                } else {
                    Err(GeneralError::ParseError("Message.telemetry.size".into()))?
                }
            }
            MessageDiscriminants::Probe => {
                if value.len() == 2 + NeighborProbe::SIZE {
                    Ok(Self::Probe(
                        (&value[2..])
                            .try_into()
                            .map_err(|_| GeneralError::ParseError("Message.probe".into()))?,
                    ))
                } else {
                    Err(GeneralError::ParseError("Message.probe.size".into()))?
                }
            }
        }
    }
}

impl TryFrom<Message> for ApplicationData {
    type Error = ProbeError;

    fn try_from(message: Message) -> Result<Self, Self::Error> {
        Ok(ApplicationData::new(ReservedTag::Ping, message.to_bytes().into_vec())?)
    }
}

impl TryFrom<ApplicationData> for Message {
    type Error = anyhow::Error;

    fn try_from(value: ApplicationData) -> Result<Self, Self::Error> {
        let reserved_probe_tag: Tag = ReservedTag::Ping.into();

        if value.application_tag == reserved_probe_tag {
            let message: Message = value.plain_text.as_ref().try_into()?;
            Ok(message)
        } else {
            Err(anyhow::anyhow!("Non probing application tag found"))
        }
    }
}

#[cfg(test)]
mod tests {
    use hopr_platform::time::native::current_time;
    use hopr_primitive_types::traits::AsUnixTimestamp;
    use more_asserts::assert_lt;

    use super::*;

    #[test]
    fn probe_message_should_serialize_and_deserialize() -> anyhow::Result<()> {
        let m1 = Message::Probe(NeighborProbe::random_nonce());
        let m2 = Message::try_from(m1.to_bytes().as_ref())?;

        assert_eq!(m1, m2);

        let m1 = Message::Telemetry(PathTelemetry {
            id: hopr_crypto_random::random_bytes(),
            path: hopr_crypto_random::random_bytes(),
            timestamp: 1234567890,
        });
        let m2 = Message::try_from(m1.to_bytes().as_ref())?;

        assert_eq!(m1, m2);
        Ok(())
    }

    #[test]
    fn random_generation_of_a_neighbor_probe_produces_a_ping() {
        let ping = NeighborProbe::random_nonce();
        assert!(matches!(ping, NeighborProbe::Ping(_)));
    }

    #[test]
    fn check_for_complement_works_for_ping_and_pong_with_the_same_nonce() -> anyhow::Result<()> {
        let ping = NeighborProbe::random_nonce();
        let pong = { NeighborProbe::Pong(ping.as_ref().try_into()?) };

        assert!(ping.is_complement_to(pong));
        Ok(())
    }

    #[test]
    fn check_for_complement_fails_for_ping_and_pong_with_different_nonce() -> anyhow::Result<()> {
        let ping = NeighborProbe::random_nonce();
        let pong = {
            let mut other: [u8; NeighborProbe::NONCE_SIZE] = ping.as_ref().try_into()?;
            other[0] = other[0].wrapping_add(1); // Modify the first byte to ensure it's different
            NeighborProbe::Pong(other)
        };

        assert!(!ping.is_complement_to(pong));

        Ok(())
    }

    #[test]
    fn check_that_at_least_one_surb_can_fit_into_the_payload_for_direct_probing() -> anyhow::Result<()> {
        let ping = NeighborProbe::random_nonce();
        let as_data: ApplicationData = Message::Probe(ping).try_into()?;

        assert_lt!(
            as_data.plain_text.len(),
            ApplicationData::PAYLOAD_SIZE - hopr_db_api::protocol::HoprSurb::SIZE
        );

        Ok(())
    }

    #[test]
    fn check_that_at_least_one_surb_can_fit_into_the_payload_for_path_telemetry() -> anyhow::Result<()> {
        let telemetry = PathTelemetry {
            id: [1; 10],
            path: [1; 10],
            timestamp: current_time().as_unix_timestamp().as_millis(),
        };
        let as_data: ApplicationData = Message::Telemetry(telemetry).try_into()?;

        assert_lt!(
            as_data.plain_text.len(),
            ApplicationData::PAYLOAD_SIZE - hopr_db_api::protocol::HoprSurb::SIZE
        );

        Ok(())
    }
}
