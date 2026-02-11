use hopr_primitive_types::prelude::GeneralError;
use hopr_protocol_app::prelude::{ApplicationData, ReservedTag, Tag};

use crate::{
    errors::ProbeError,
    types::{NeighborProbe, PathTelemetry},
};

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
    fn probe_message_variant_probe_should_serialize_and_deserialize() -> anyhow::Result<()> {
        let m1 = Message::Probe(NeighborProbe::random_nonce());
        let m2 = Message::try_from(m1.to_bytes().as_ref())?;

        assert_eq!(m1, m2);

        Ok(())
    }

    #[test]
    fn probe_message_variant_telemetry_should_serialize_and_deserialize() -> anyhow::Result<()> {
        let m1 = Message::Telemetry(PathTelemetry {
            id: hopr_crypto_random::random_bytes::<10>(),
            path: hopr_crypto_random::random_bytes::<{ 10 * std::mem::size_of::<u128>() }>(),
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
            ApplicationData::PAYLOAD_SIZE - hopr_crypto_packet::HoprSurb::SIZE
        );

        Ok(())
    }

    #[test]
    fn check_that_at_least_one_surb_can_fit_into_the_payload_for_path_telemetry() -> anyhow::Result<()> {
        let telemetry = PathTelemetry {
            id: [1; 10],
            path: [1; 10 * size_of::<u128>()],
            timestamp: current_time().as_unix_timestamp().as_millis(),
        };
        let as_data: ApplicationData = Message::Telemetry(telemetry).try_into()?;

        assert_lt!(
            as_data.plain_text.len(),
            ApplicationData::PAYLOAD_SIZE - hopr_crypto_packet::HoprSurb::SIZE
        );

        Ok(())
    }
}
