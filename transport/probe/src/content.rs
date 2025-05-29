use anyhow::Context;
use hopr_transport_packet::prelude::{ApplicationData, ReservedTag, Tag};

/// Serializable and deserializable enum for the probe message content
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum NeighborProbe {
    /// Ping message with a random nonce
    Ping([u8; Self::SIZE]),
    /// Pong message repliying to a specific nonce
    Pong([u8; Self::SIZE]),
}

impl NeighborProbe {
    pub const SIZE: usize = 32;

    /// Returns the nonce of the message
    pub fn random_nonce() -> Self {
        const SIZE: usize = NeighborProbe::SIZE;
        Self::Ping(hopr_crypto_random::random_bytes::<SIZE>())
    }

    pub fn is_complement_to(&self, other: Self) -> bool {
        match (self, &other) {
            (Self::Ping(nonce1), Self::Pong(nonce2)) => nonce1 == nonce2,
            (Self::Pong(nonce1), Self::Ping(nonce2)) => nonce1 == nonce2,
            _ => false,
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

impl From<NeighborProbe> for [u8; NeighborProbe::SIZE] {
    fn from(probe: NeighborProbe) -> Self {
        match probe {
            NeighborProbe::Ping(nonce) | NeighborProbe::Pong(nonce) => nonce,
        }
    }
}

/// TODO: TBD as a separate task for network discovery
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct PathTelemetry {
    id: [u8; 10],
    path: [u8; 10],
    timestamp: u128,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Message {
    Telemetry(PathTelemetry),
    Probe(NeighborProbe),
}

impl std::fmt::Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Message::Telemetry(telemetry) => write!(f, "Telemetry({:?})", telemetry),
            Message::Probe(probe) => write!(f, "Probe({})", probe),
        }
    }
}

impl TryFrom<Message> for ApplicationData {
    type Error = anyhow::Error;

    fn try_from(message: Message) -> Result<Self, Self::Error> {
        let tag: Tag = ReservedTag::Ping.into();
        Ok(ApplicationData {
            application_tag: tag,
            plain_text: bitcode::serialize(&message)
                .context("Failed to serialize message")?
                .into_boxed_slice(),
        })
    }
}

impl TryFrom<ApplicationData> for Message {
    type Error = anyhow::Error;

    fn try_from(value: ApplicationData) -> Result<Self, Self::Error> {
        let reserved_probe_tag: Tag = ReservedTag::Ping.into();

        if value.application_tag == reserved_probe_tag {
            let message: Message = bitcode::deserialize(&value.plain_text).context("Failed to deserialize message")?;
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
    fn random_generation_of_a_neighbor_probe_produces_a_ping() {
        let ping = NeighborProbe::random_nonce();
        assert!(matches!(ping, NeighborProbe::Ping(_)));
    }

    #[test]
    fn check_for_complement_works_for_ping_and_pong_with_the_same_nonce() {
        let ping = NeighborProbe::random_nonce();
        let pong = { NeighborProbe::Pong(ping.into()) };

        assert!(ping.is_complement_to(pong));
    }

    #[test]
    fn check_for_complement_fails_for_ping_and_pong_with_different_nonce() {
        let ping = NeighborProbe::random_nonce();
        let pong = {
            let mut other: [u8; NeighborProbe::SIZE] = ping.into();
            other[0] = other[0].wrapping_add(1); // Modify the first byte to ensure it's different
            NeighborProbe::Pong(other)
        };

        assert!(!ping.is_complement_to(pong));
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
