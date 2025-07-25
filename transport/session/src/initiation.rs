//! This module defines the Start sub-protocol used for HOPR Session initiation and management.
// TODO: Move this module as a standalone crate into hopr-protocol-start

use hopr_transport_packet::prelude::{ApplicationData, ReservedTag, Tag};

use crate::{Capabilities, errors::TransportSessionError, types::SessionTarget};

/// Challenge that identifies a Start initiation protocol message.
pub type StartChallenge = u64;

/// Lists all Start protocol error reasons.
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, strum::Display)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum StartErrorReason {
    /// No more slots are available at the recipient.
    NoSlotsAvailable,
    /// Recipient is busy.
    Busy,
}

/// Error message in the Start protocol.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StartErrorType {
    /// Challenge that relates to this error.
    pub challenge: StartChallenge,
    /// The [reason](StartErrorReason) of this error.
    pub reason: StartErrorReason,
}

/// The session initiation message of the Start protocol.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StartInitiation {
    /// Random challenge for this initiation.
    pub challenge: StartChallenge,
    /// [Target](SessionTarget) of the session, i.e., what should the other party do with the traffic.
    pub target: SessionTarget,
    /// Capabilities of the session.
    pub capabilities: Capabilities,
}

/// Message of the Start protocol that confirms the establishment of a session.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StartEstablished<T> {
    /// Challenge that was used in the [initiation message](StartInitiation) to establish correspondence.
    pub orig_challenge: StartChallenge,
    /// Session ID that was selected by the recipient.
    pub session_id: T,
}

#[cfg_attr(doc, aquamarine::aquamarine)]
/// Lists all messages of the Start protocol for a session establishment
/// with `T` as session identifier.
///
/// # Diagram of the protocol
/// ```mermaid
/// sequenceDiagram
///     Entry->>Exit: SessionInitiation (Challenge)
///     alt If Exit can accept a new session
///     Note right of Exit: SessionID [Pseudonym, Tag]
///     Exit->>Entry: SessionEstablished (Challenge, SessionID_Entry)
///     Note left of Entry: SessionID [Pseudonym, Tag]
///     Entry->>Exit: KeepAlive (SessionID)
///     Note over Entry,Exit: Data
///     Entry->>Exit: Close Session (SessionID)
///     else If Exit cannot accept a new session
///     Exit->>Entry: SessionError (Challenge, Reason)
///     end
///     opt If initiation attempt times out
///     Note left of Entry: Failure
///     end
/// ```
// Do not implement Serialize,Deserialize -> enforce serialization via encode/decode
#[derive(Debug, Clone, PartialEq, Eq, strum::EnumDiscriminants)]
#[strum_discriminants(vis(pub(crate)))]
#[strum_discriminants(derive(strum::FromRepr, strum::EnumCount), repr(u8))]
pub enum StartProtocol<T> {
    /// Request to initiate a new session.
    StartSession(StartInitiation),
    /// Confirmation that a new session has been established by the counterparty.
    SessionEstablished(StartEstablished<T>),
    /// Counterparty could not establish a new session due to an error.
    SessionError(StartErrorType),
    /// A ping message to keep the session alive.
    KeepAlive(KeepAliveMessage<T>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KeepAliveMessage<T> {
    /// Session ID.
    pub id: T,
    /// Reserved for future use, always zero currently.
    pub flags: u8,
}

impl<T> From<T> for KeepAliveMessage<T> {
    fn from(value: T) -> Self {
        Self { id: value, flags: 0 }
    }
}

impl<T> StartProtocol<T> {
    pub(crate) const START_PROTOCOL_MESSAGE_TAG: Tag = Tag::Reserved(ReservedTag::SessionStart as u64);
    const START_PROTOCOL_VERSION: u8 = 0x01;
}

// TODO: implement this without Serde, see #7145
#[cfg(feature = "serde")]
impl<T: serde::Serialize + for<'de> serde::Deserialize<'de>> StartProtocol<T> {
    const SESSION_BINCODE_CONFIGURATION: bincode::config::Configuration = bincode::config::standard()
        .with_little_endian()
        .with_variable_int_encoding();

    /// Serialize the message into a message tag and message data.
    /// Data is serialized using `bincode`.
    pub fn encode(self) -> crate::errors::Result<(Tag, Box<[u8]>)> {
        let mut out = Vec::with_capacity(ApplicationData::PAYLOAD_SIZE);
        out.push(Self::START_PROTOCOL_VERSION);
        out.push(StartProtocolDiscriminants::from(&self) as u8);

        match self {
            StartProtocol::StartSession(init) => {
                bincode::serde::encode_into_std_write(&init, &mut out, Self::SESSION_BINCODE_CONFIGURATION)
            }
            StartProtocol::SessionEstablished(est) => {
                bincode::serde::encode_into_std_write(&est, &mut out, Self::SESSION_BINCODE_CONFIGURATION)
            }
            StartProtocol::SessionError(err) => {
                bincode::serde::encode_into_std_write(err, &mut out, Self::SESSION_BINCODE_CONFIGURATION)
            }
            StartProtocol::KeepAlive(msg) => {
                bincode::serde::encode_into_std_write(&msg, &mut out, Self::SESSION_BINCODE_CONFIGURATION)
            }
        }?;

        Ok((Self::START_PROTOCOL_MESSAGE_TAG, out.into_boxed_slice()))
    }

    /// Deserialize the message from message tag and message data.
    /// Data is deserialized using `bincode`.
    pub fn decode(tag: Tag, data: &[u8]) -> crate::errors::Result<Self> {
        if tag != Self::START_PROTOCOL_MESSAGE_TAG {
            return Err(TransportSessionError::StartProtocolError("unknown message tag".into()));
        }

        if data.len() < 3 {
            return Err(TransportSessionError::StartProtocolError("message too short".into()));
        }

        if data[0] != Self::START_PROTOCOL_VERSION {
            return Err(TransportSessionError::StartProtocolError(
                "unknown message version".into(),
            ));
        }

        match StartProtocolDiscriminants::from_repr(data[1])
            .ok_or(TransportSessionError::StartProtocolError("unknown message".into()))?
        {
            StartProtocolDiscriminants::StartSession => Ok(StartProtocol::StartSession(
                bincode::serde::borrow_decode_from_slice(&data[2..], Self::SESSION_BINCODE_CONFIGURATION)
                    .map(|(v, _bytes)| v)?,
            )),
            StartProtocolDiscriminants::SessionEstablished => Ok(StartProtocol::SessionEstablished(
                bincode::serde::borrow_decode_from_slice(&data[2..], Self::SESSION_BINCODE_CONFIGURATION)
                    .map(|(v, _bytes)| v)?,
            )),
            StartProtocolDiscriminants::SessionError => Ok(StartProtocol::SessionError(
                bincode::serde::borrow_decode_from_slice(&data[2..], Self::SESSION_BINCODE_CONFIGURATION)
                    .map(|(v, _bytes)| v)?,
            )),
            StartProtocolDiscriminants::KeepAlive => Ok(StartProtocol::KeepAlive(
                bincode::serde::borrow_decode_from_slice(&data[2..], Self::SESSION_BINCODE_CONFIGURATION)
                    .map(|(v, _bytes)| v)?,
            )),
        }
    }
}

#[cfg(not(feature = "serde"))]
impl<T> StartProtocol<T> {
    pub fn encode(self) -> crate::errors::Result<(u16, Box<[u8]>)> {
        unimplemented!()
    }

    pub fn decode(_tag: u16, _data: &[u8]) -> crate::errors::Result<Self> {
        unimplemented!()
    }
}

#[cfg(feature = "serde")]
impl<T: serde::Serialize + for<'de> serde::Deserialize<'de>> TryFrom<StartProtocol<T>> for ApplicationData {
    type Error = TransportSessionError;

    fn try_from(value: StartProtocol<T>) -> Result<Self, Self::Error> {
        let (application_tag, plain_text) = value.encode()?;
        Ok(ApplicationData {
            application_tag,
            plain_text,
        })
    }
}

#[cfg(not(feature = "serde"))]
impl<T> TryFrom<StartProtocol<T>> for ApplicationData {
    type Error = TransportSessionError;

    fn try_from(value: StartProtocol<T>) -> Result<Self, Self::Error> {
        let (application_tag, plain_text) = value.encode()?;
        Ok(ApplicationData {
            application_tag,
            plain_text,
        })
    }
}

#[cfg(feature = "serde")]
impl<T: serde::Serialize + for<'de> serde::Deserialize<'de>> TryFrom<ApplicationData> for StartProtocol<T> {
    type Error = TransportSessionError;

    fn try_from(value: ApplicationData) -> Result<Self, Self::Error> {
        Self::decode(value.application_tag, &value.plain_text)
    }
}

#[cfg(not(feature = "serde"))]
impl<T> TryFrom<ApplicationData> for StartProtocol<T> {
    type Error = TransportSessionError;

    fn try_from(value: ApplicationData) -> Result<Self, Self::Error> {
        Self::decode(value.application_tag, &value.plain_text)
    }
}

#[cfg(test)]
mod tests {
    use hopr_crypto_packet::prelude::HoprPacket;
    use hopr_crypto_random::Randomizable;
    use hopr_internal_types::prelude::HoprPseudonym;
    use hopr_network_types::prelude::SealedHost;
    use hopr_transport_packet::prelude::Tag;

    use super::*;
    use crate::{Capability, SessionId};

    #[cfg(feature = "serde")]
    #[test]
    fn start_protocol_start_session_message_should_encode_and_decode() -> anyhow::Result<()> {
        let msg_1 = StartProtocol::<i32>::StartSession(StartInitiation {
            challenge: 0,
            target: SessionTarget::TcpStream(SealedHost::Plain("127.0.0.1:1234".parse()?)),
            capabilities: Default::default(),
        });

        let (tag, msg) = msg_1.clone().encode()?;
        let expected: Tag = StartProtocol::<()>::START_PROTOCOL_MESSAGE_TAG;
        assert_eq!(tag, expected);

        let msg_2 = StartProtocol::<i32>::decode(tag, &msg)?;

        assert_eq!(msg_1, msg_2);
        Ok(())
    }

    #[test]
    fn start_protocol_message_start_session_message_should_allow_for_at_least_one_surb() -> anyhow::Result<()> {
        let msg = StartProtocol::<SessionId>::StartSession(StartInitiation {
            challenge: 0,
            target: SessionTarget::TcpStream(SealedHost::Plain("127.0.0.1:1234".parse()?)),
            capabilities: Default::default(),
        });

        let len = msg.encode()?.1.len();
        assert!(
            HoprPacket::max_surbs_with_message(len) >= 1,
            "KeepAlive message size ({len}) must allow for at least 1 SURBs in packet",
        );

        Ok(())
    }

    #[cfg(feature = "serde")]
    #[test]
    fn start_protocol_session_established_message_should_encode_and_decode() -> anyhow::Result<()> {
        let msg_1 = StartProtocol::<i32>::SessionEstablished(StartEstablished {
            orig_challenge: 0,
            session_id: 10,
        });

        let (tag, msg) = msg_1.clone().encode()?;
        let expected: Tag = StartProtocol::<()>::START_PROTOCOL_MESSAGE_TAG;
        assert_eq!(tag, expected);

        let msg_2 = StartProtocol::<i32>::decode(tag, &msg)?;

        assert_eq!(msg_1, msg_2);
        Ok(())
    }

    #[cfg(feature = "serde")]
    #[test]
    fn start_protocol_session_error_message_should_encode_and_decode() -> anyhow::Result<()> {
        let msg_1 = StartProtocol::<i32>::SessionError(StartErrorType {
            challenge: 10,
            reason: StartErrorReason::NoSlotsAvailable,
        });

        let (tag, msg) = msg_1.clone().encode()?;
        let expected: Tag = StartProtocol::<()>::START_PROTOCOL_MESSAGE_TAG;
        assert_eq!(tag, expected);

        let msg_2 = StartProtocol::<i32>::decode(tag, &msg)?;

        assert_eq!(msg_1, msg_2);
        Ok(())
    }

    #[cfg(feature = "serde")]
    #[test]
    fn start_protocol_keep_alive_message_should_encode_and_decode() -> anyhow::Result<()> {
        let msg_1 = StartProtocol::<i32>::KeepAlive(10.into());

        let (tag, msg) = msg_1.clone().encode()?;
        let expected: Tag = StartProtocol::<()>::START_PROTOCOL_MESSAGE_TAG;
        assert_eq!(tag, expected);

        let msg_2 = StartProtocol::<i32>::decode(tag, &msg)?;

        assert_eq!(msg_1, msg_2);
        Ok(())
    }

    #[cfg(feature = "serde")]
    #[test]
    fn start_protocol_messages_must_fit_within_hopr_packet() -> anyhow::Result<()> {
        let msg = StartProtocol::<SessionId>::StartSession(StartInitiation {
            challenge: StartChallenge::MAX,
            target: SessionTarget::TcpStream(SealedHost::Plain(
                "example-of-a-very-very-long-second-level-name.on-a-very-very-long-domain-name.info:65530".parse()?,
            )),
            capabilities: Capability::RetransmissionAck | Capability::RetransmissionNack | Capability::Segmentation,
        });

        assert!(
            msg.encode()?.1.len() <= HoprPacket::PAYLOAD_SIZE,
            "StartSession must fit within {}",
            HoprPacket::PAYLOAD_SIZE
        );

        let msg = StartProtocol::SessionEstablished(StartEstablished {
            orig_challenge: StartChallenge::MAX,
            session_id: SessionId::new(Tag::MAX, HoprPseudonym::random()),
        });

        assert!(
            msg.encode()?.1.len() <= HoprPacket::PAYLOAD_SIZE,
            "SessionEstablished must fit within {}",
            HoprPacket::PAYLOAD_SIZE
        );

        let msg = StartProtocol::<i32>::SessionError(StartErrorType {
            challenge: StartChallenge::MAX,
            reason: StartErrorReason::NoSlotsAvailable,
        });

        assert!(
            msg.encode()?.1.len() <= HoprPacket::PAYLOAD_SIZE,
            "SessionError must fit within {}",
            HoprPacket::PAYLOAD_SIZE
        );

        let msg = StartProtocol::KeepAlive(SessionId::new(Tag::MAX, HoprPseudonym::random()).into());
        assert!(
            msg.encode()?.1.len() <= HoprPacket::PAYLOAD_SIZE,
            "KeepAlive must fit within {}",
            HoprPacket::PAYLOAD_SIZE
        );

        Ok(())
    }

    #[test]
    fn start_protocol_message_keep_alive_message_should_allow_for_maximum_surbs() -> anyhow::Result<()> {
        let msg = StartProtocol::KeepAlive(SessionId::new(Tag::MAX, HoprPseudonym::random()).into());
        let len = msg.encode()?.1.len();
        assert!(
            HoprPacket::max_surbs_with_message(len) >= HoprPacket::MAX_SURBS_IN_PACKET,
            "KeepAlive message size ({}) must allow for at least {} SURBs in packet",
            len,
            HoprPacket::MAX_SURBS_IN_PACKET
        );

        Ok(())
    }
}
