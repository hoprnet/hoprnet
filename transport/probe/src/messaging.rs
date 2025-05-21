use anyhow::Context;
use hopr_transport_packet::prelude::{ApplicationData, ReservedTag, Tag};

/// Serializable and deserializable enum for the probe message content
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum NeighborProbe {
    /// Ping message with a random nonce
    Ping(u128),
    /// Pong message repliying to a specific nonce
    Pong(u128),
}

impl NeighborProbe {
    /// Returns the nonce of the message
    pub fn random_ping() -> Self {
        Self::Ping(rand::random::<u128>())
    }

    pub fn is_complement_to(&self, other: Self) -> bool {
        match (self, &other) {
            (Self::Ping(nonce1), Self::Pong(nonce2)) => nonce1 == nonce2,
            (Self::Pong(nonce1), Self::Ping(nonce2)) => nonce1 == nonce2,
            _ => false,
        }
    }
}

impl From<NeighborProbe> for u128 {
    fn from(probe: NeighborProbe) -> Self {
        match probe {
            NeighborProbe::Ping(nonce) | NeighborProbe::Pong(nonce) => nonce,
        }
    }
}

/// TODO: TBD as a separate task for network discovery
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct PathTelemetry {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Message {
    Telemetry(PathTelemetry),
    Probe(NeighborProbe),
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
    use super::*;

    #[test]
    fn random_generation_of_a_neighbor_probe_produces_a_ping() {
        let ping = NeighborProbe::random_ping();
        assert!(matches!(ping, NeighborProbe::Ping(_)));
    }

    #[test]
    fn check_for_complement_works_for_ping_and_pong_with_the_same_nonce() {
        let ping = NeighborProbe::random_ping();
        let pong = {
            let ping: u128 = ping.into();
            NeighborProbe::Pong(ping)
        };

        assert!(ping.is_complement_to(pong));
    }

    #[test]
    fn check_for_complement_fails_for_ping_and_pong_with_different_nonce() {
        let ping = NeighborProbe::random_ping();
        let pong = {
            let ping: u128 = ping.into();
            NeighborProbe::Pong(ping + 1)
        };

        assert!(!ping.is_complement_to(pong));
    }
}
