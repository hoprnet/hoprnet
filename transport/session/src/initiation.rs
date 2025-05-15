//! This module defines the Start sub-protocol used for HOPR Session initiation and management.

use hopr_internal_types::prelude::ApplicationData;
use std::collections::HashSet;

use crate::errors::TransportSessionError;
use crate::types::SessionTarget;
use crate::Capability;

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
    pub capabilities: HashSet<Capability>,
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
///     Exit->>Entry: Close Session (SessionID)
///     else If Exit cannot accept a new session
///     Exit->>Entry: SessionError (Challenge, Reason)
///     end
///     opt If initiation attempt times out
///     Note left of Entry: Failure
///     end
/// ```
#[derive(Debug, Clone, PartialEq, Eq, strum::EnumDiscriminants)]
// #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))] -- enforce serialization via encode/decode
#[strum_discriminants(vis(pub(crate)))]
#[strum_discriminants(derive(strum::FromRepr, strum::EnumCount), repr(u8))]
pub enum StartProtocol<T> {
    /// Request to initiate a new session.
    StartSession(StartInitiation),
    /// Confirmation that a new session has been established by the counterparty.
    SessionEstablished(StartEstablished<T>),
    /// Counterparty could not establish a new session due to an error.
    SessionError(StartErrorType),
    /// Counterparty has closed the session.
    CloseSession(T),
    /// A ping message to keep the session alive.
    KeepAlive(T),
}

const SESSION_BINCODE_CONFIGURATION: bincode::config::Configuration = bincode::config::standard()
    .with_little_endian()
    .with_variable_int_encoding();

#[cfg(feature = "serde")]
impl<T: serde::Serialize + for<'de> serde::Deserialize<'de>> StartProtocol<T> {
    /// Serialize the message into a message tag and message data.
    /// Data is serialized using `bincode`.
    pub fn encode(self) -> crate::errors::Result<(u16, Box<[u8]>)> {
        let disc = StartProtocolDiscriminants::from(&self) as u8 + 1;
        let inner = match self {
            StartProtocol::StartSession(init) => bincode::serde::encode_to_vec(&init, SESSION_BINCODE_CONFIGURATION),
            StartProtocol::SessionEstablished(est) => {
                bincode::serde::encode_to_vec(&est, SESSION_BINCODE_CONFIGURATION)
            }
            StartProtocol::SessionError(err) => bincode::serde::encode_to_vec(err, SESSION_BINCODE_CONFIGURATION),
            StartProtocol::CloseSession(id) => bincode::serde::encode_to_vec(&id, SESSION_BINCODE_CONFIGURATION),
            StartProtocol::KeepAlive(id) => bincode::serde::encode_to_vec(&id, SESSION_BINCODE_CONFIGURATION),
        }?;

        Ok((disc as u16, inner.into_boxed_slice()))
    }

    /// Deserialize the message from message tag and message data.
    /// Data is deserialized using `bincode`.
    pub fn decode(tag: u16, data: &[u8]) -> crate::errors::Result<Self> {
        if tag == 0 {
            return Err(TransportSessionError::Tag);
        }

        match StartProtocolDiscriminants::from_repr(tag as u8 - 1).ok_or(TransportSessionError::PayloadSize)? {
            StartProtocolDiscriminants::StartSession => Ok(StartProtocol::StartSession(
                bincode::serde::borrow_decode_from_slice(data, SESSION_BINCODE_CONFIGURATION).map(|(v, _bytes)| v)?,
            )),
            StartProtocolDiscriminants::SessionEstablished => Ok(StartProtocol::SessionEstablished(
                bincode::serde::borrow_decode_from_slice(data, SESSION_BINCODE_CONFIGURATION).map(|(v, _bytes)| v)?,
            )),
            StartProtocolDiscriminants::SessionError => Ok(StartProtocol::SessionError(
                bincode::serde::borrow_decode_from_slice(data, SESSION_BINCODE_CONFIGURATION).map(|(v, _bytes)| v)?,
            )),
            StartProtocolDiscriminants::CloseSession => Ok(StartProtocol::CloseSession(
                bincode::serde::borrow_decode_from_slice(data, SESSION_BINCODE_CONFIGURATION).map(|(v, _bytes)| v)?,
            )),
            StartProtocolDiscriminants::KeepAlive => Ok(StartProtocol::KeepAlive(
                bincode::serde::borrow_decode_from_slice(data, SESSION_BINCODE_CONFIGURATION).map(|(v, _bytes)| v)?,
            )),
        }
    }
}

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

impl<T: serde::Serialize + for<'de> serde::Deserialize<'de>> TryFrom<ApplicationData> for StartProtocol<T> {
    type Error = TransportSessionError;

    fn try_from(value: ApplicationData) -> Result<Self, Self::Error> {
        Self::decode(value.application_tag, &value.plain_text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hopr_crypto_packet::prelude::HoprPacket;
    use hopr_crypto_random::Randomizable;
    use hopr_internal_types::prelude::{HoprPseudonym, Tag};
    use hopr_network_types::prelude::SealedHost;

    use crate::SessionId;

    #[cfg(feature = "serde")]
    #[test]
    fn start_protocol_start_session_message_should_encode_and_decode() -> anyhow::Result<()> {
        let msg_1 = StartProtocol::<i32>::StartSession(StartInitiation {
            challenge: 0,
            target: SessionTarget::TcpStream(SealedHost::Plain("127.0.0.1:1234".parse()?)),
            capabilities: Default::default(),
        });

        let (tag, msg) = msg_1.clone().encode()?;
        assert_eq!(StartProtocolDiscriminants::StartSession as Tag + 1, tag);

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
            "KeepAlive message size ({}) must allow for at least 1 SURBs in packet",
            len,
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
        assert_eq!(StartProtocolDiscriminants::SessionEstablished as Tag + 1, tag);

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
        assert_eq!(StartProtocolDiscriminants::SessionError as Tag + 1, tag);

        let msg_2 = StartProtocol::<i32>::decode(tag, &msg)?;

        assert_eq!(msg_1, msg_2);
        Ok(())
    }

    #[cfg(feature = "serde")]
    #[test]
    fn start_protocol_close_session_message_should_encode_and_decode() -> anyhow::Result<()> {
        let msg_1 = StartProtocol::<i32>::CloseSession(10);

        let (tag, msg) = msg_1.clone().encode()?;
        assert_eq!(StartProtocolDiscriminants::CloseSession as Tag + 1, tag);

        let msg_2 = StartProtocol::<i32>::decode(tag, &msg)?;

        assert_eq!(msg_1, msg_2);
        Ok(())
    }

    #[cfg(feature = "serde")]
    #[test]
    fn start_protocol_keep_alive_message_should_encode_and_decode() -> anyhow::Result<()> {
        let msg_1 = StartProtocol::<i32>::KeepAlive(10);

        let (tag, msg) = msg_1.clone().encode()?;
        assert_eq!(StartProtocolDiscriminants::KeepAlive as Tag + 1, tag);

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
            capabilities: HashSet::from_iter([Capability::Retransmission, Capability::Segmentation]),
        });

        assert!(
            msg.encode()?.1.len() <= HoprPacket::PAYLOAD_SIZE,
            "StartSession must fit within {}",
            HoprPacket::PAYLOAD_SIZE
        );

        let msg = StartProtocol::SessionEstablished(StartEstablished {
            orig_challenge: StartChallenge::MAX,
            session_id: SessionId::new(u16::MAX, HoprPseudonym::random()),
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

        let msg = StartProtocol::CloseSession(SessionId::new(u16::MAX, HoprPseudonym::random()));
        assert!(
            msg.encode()?.1.len() <= HoprPacket::PAYLOAD_SIZE,
            "CloseSession must fit within {}",
            HoprPacket::PAYLOAD_SIZE
        );

        let msg = StartProtocol::KeepAlive(SessionId::new(u16::MAX, HoprPseudonym::random()));
        assert!(
            msg.encode()?.1.len() <= HoprPacket::PAYLOAD_SIZE,
            "KeepAlive must fit within {}",
            HoprPacket::PAYLOAD_SIZE
        );

        Ok(())
    }

    #[test]
    fn start_protocol_message_keep_alive_message_should_allow_for_maximum_surbs() -> anyhow::Result<()> {
        let msg = StartProtocol::KeepAlive(SessionId::new(u16::MAX, HoprPseudonym::random()));
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
