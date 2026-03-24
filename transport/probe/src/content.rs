use hopr_api::types::primitive::prelude::GeneralError;
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
        if value.len() < 2 {
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
    use anyhow::Context;
    use hopr_api::types::primitive::traits::AsUnixTimestamp;
    use hopr_platform::time::native::current_time;
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
            id: hopr_api::types::crypto_random::random_bytes(),
            path: hopr_api::types::crypto_random::random_bytes(),
            timestamp: 1234567890,
        });
        let m2 = Message::try_from(m1.to_bytes().as_ref())?;

        assert_eq!(m1, m2);
        Ok(())
    }

    #[test]
    fn random_generation_of_a_neighbor_probe_produces_a_ping() -> anyhow::Result<()> {
        let ping = NeighborProbe::random_nonce();
        anyhow::ensure!(matches!(ping, NeighborProbe::Ping(_)), "expected Ping variant");
        Ok(())
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
            id: [1; 8],
            path: [1; 5 * size_of::<u64>()],
            timestamp: current_time().as_unix_timestamp().as_millis(),
        };
        let as_data: ApplicationData = Message::Telemetry(telemetry).try_into()?;

        assert_lt!(
            as_data.plain_text.len(),
            ApplicationData::PAYLOAD_SIZE - hopr_crypto_packet::HoprSurb::SIZE
        );

        Ok(())
    }

    #[test]
    fn message_from_empty_bytes_fails() -> anyhow::Result<()> {
        let err = Message::try_from([].as_slice())
            .err()
            .context("expected error for empty bytes")?;
        anyhow::ensure!(
            matches!(&err, GeneralError::ParseError(s) if s.contains("size")),
            "expected ParseError about size, got: {err}"
        );
        Ok(())
    }

    #[test]
    fn message_from_truncated_header_fails() -> anyhow::Result<()> {
        let err = Message::try_from([Message::VERSION].as_slice())
            .err()
            .context("expected error for truncated header")?;
        anyhow::ensure!(
            matches!(&err, GeneralError::ParseError(s) if s.contains("size")),
            "expected ParseError about size, got: {err}"
        );
        Ok(())
    }

    #[test]
    fn message_from_wrong_version_fails() -> anyhow::Result<()> {
        // Version 0xFF instead of Message::VERSION (1)
        let data = [0xFF, 0x00];
        let err = Message::try_from(data.as_slice())
            .err()
            .context("expected error for wrong version")?;
        anyhow::ensure!(
            matches!(&err, GeneralError::ParseError(s) if s.contains("version")),
            "expected ParseError about version, got: {err}"
        );
        Ok(())
    }

    #[test]
    fn message_from_invalid_discriminant_fails() -> anyhow::Result<()> {
        // Valid version but invalid discriminant
        let data = [Message::VERSION, 0xFF];
        let err = Message::try_from(data.as_slice())
            .err()
            .context("expected error for invalid discriminant")?;
        anyhow::ensure!(
            matches!(&err, GeneralError::ParseError(s) if s.contains("disc")),
            "expected ParseError about discriminant, got: {err}"
        );
        Ok(())
    }

    #[test]
    fn message_from_wrong_probe_size_fails() -> anyhow::Result<()> {
        // Valid version + probe discriminant but truncated data
        let data = [Message::VERSION, MessageDiscriminants::Probe as u8, 0x00];
        let err = Message::try_from(data.as_slice())
            .err()
            .context("expected error for wrong probe size")?;
        anyhow::ensure!(
            matches!(&err, GeneralError::ParseError(s) if s.contains("probe.size")),
            "expected ParseError about probe size, got: {err}"
        );
        Ok(())
    }

    #[test]
    fn message_application_data_roundtrip() -> anyhow::Result<()> {
        let probe = Message::Probe(NeighborProbe::random_nonce());
        let app_data: ApplicationData = probe.try_into()?;
        let restored: Message = app_data.try_into()?;

        anyhow::ensure!(
            matches!(restored, Message::Probe(NeighborProbe::Ping(_))),
            "expected Probe(Ping) variant"
        );

        Ok(())
    }

    #[test]
    fn message_application_data_wrong_tag_fails() -> anyhow::Result<()> {
        let app_data = ApplicationData::new(Tag::MAX, b"not a probe")?;
        let err = Message::try_from(app_data)
            .err()
            .context("expected error for wrong tag")?;
        anyhow::ensure!(err.to_string().contains("tag"), "expected error about tag, got: {err}");
        Ok(())
    }
}
