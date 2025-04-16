//! This module defines the Start sub-protocol used for HOPR Session initiation and management.

use crate::errors::TransportSessionError;
use crate::types::SessionTarget;
use crate::Capability;
use hopr_internal_types::prelude::ApplicationData;
use hopr_network_types::prelude::RoutingOptions;
use hopr_primitive_types::prelude::Address;
use std::collections::HashSet;

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
    /// Optional information on back routing from the other party back towards the
    /// session initiator.
    ///
    /// **NOTE:** This will not be used when the Return Path is introduced.
    pub back_routing: Option<(RoutingOptions, Address)>,
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
///     Note right of Exit: SessionID_Exit [Entry PeerID, Tag]
///     Exit->>Entry: SessionEstablished (Challenge, SessionID_Entry)
///     Note left of Entry: SessionID_Entry [Exit PeerID, Tag]
///     Note over Entry,Exit: Data
///     Entry->>Exit: Close Session (SessionID_Entry)
///     Exit->>Entry: Close Session (SessionID_Exit)
///     else If Exit cannot accept a new session
///     Exit->>Entry: SesssionError (Challenge, Reason)
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
}

#[cfg(feature = "serde")]
impl<T: serde::Serialize + for<'de> serde::Deserialize<'de>> StartProtocol<T> {
    /// Serialize the message into a message tag and message data.
    /// Data is serialized using `bincode`.
    pub fn encode(self) -> crate::errors::Result<(u16, Box<[u8]>)> {
        let disc = StartProtocolDiscriminants::from(&self) as u8 + 1;
        let inner = match self {
            StartProtocol::StartSession(init) => bincode::serialize(&init),
            StartProtocol::SessionEstablished(est) => bincode::serialize(&est),
            StartProtocol::SessionError(err) => bincode::serialize(&err),
            StartProtocol::CloseSession(id) => bincode::serialize(&id),
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
            StartProtocolDiscriminants::StartSession => Ok(StartProtocol::StartSession(bincode::deserialize(data)?)),
            StartProtocolDiscriminants::SessionEstablished => {
                Ok(StartProtocol::SessionEstablished(bincode::deserialize(data)?))
            }
            StartProtocolDiscriminants::SessionError => Ok(StartProtocol::SessionError(bincode::deserialize(data)?)),
            StartProtocolDiscriminants::CloseSession => Ok(StartProtocol::CloseSession(bincode::deserialize(data)?)),
        }
    }
}

impl<T: serde::Serialize + for<'de> serde::Deserialize<'de>> TryFrom<StartProtocol<T>> for ApplicationData {
    type Error = TransportSessionError;

    fn try_from(value: StartProtocol<T>) -> Result<Self, Self::Error> {
        let (tag, plain_text) = value.encode()?;
        Ok(ApplicationData {
            application_tag: Some(tag),
            plain_text,
        })
    }
}

impl<T: serde::Serialize + for<'de> serde::Deserialize<'de>> TryFrom<ApplicationData> for StartProtocol<T> {
    type Error = TransportSessionError;

    fn try_from(value: ApplicationData) -> Result<Self, Self::Error> {
        Self::decode(
            value.application_tag.ok_or(TransportSessionError::Tag)?,
            &value.plain_text,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SessionId;
    use hopr_internal_types::prelude::PAYLOAD_SIZE;
    use hopr_network_types::prelude::SealedHost;

    #[cfg(feature = "serde")]
    #[test]
    fn start_protocol_start_session_message_should_encode_and_decode() -> anyhow::Result<()> {
        let msg_1 = StartProtocol::<i32>::StartSession(StartInitiation {
            challenge: 0,
            target: SessionTarget::TcpStream(SealedHost::Plain("127.0.0.1:1234".parse()?)),
            capabilities: Default::default(),
            back_routing: Some((
                RoutingOptions::IntermediatePath(vec![PeerId::random()].try_into()?),
                PeerId::random(),
            )),
        });

        let (tag, msg) = msg_1.clone().encode()?;
        assert_eq!(1, tag);

        let msg_2 = StartProtocol::<i32>::decode(tag, &msg)?;

        assert_eq!(msg_1, msg_2);
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
        assert_eq!(2, tag);

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
        assert_eq!(3, tag);

        let msg_2 = StartProtocol::<i32>::decode(tag, &msg)?;

        assert_eq!(msg_1, msg_2);
        Ok(())
    }

    #[cfg(feature = "serde")]
    #[test]
    fn start_protocol_close_session_message_should_encode_and_decode() -> anyhow::Result<()> {
        let msg_1 = StartProtocol::<i32>::CloseSession(10);

        let (tag, msg) = msg_1.clone().encode()?;
        assert_eq!(4, tag);

        let msg_2 = StartProtocol::<i32>::decode(tag, &msg)?;

        assert_eq!(msg_1, msg_2);
        Ok(())
    }

    #[cfg(feature = "serde")]
    #[test]
    fn start_protocol_messages_must_fit_within_hopr_packet() -> anyhow::Result<()> {
        let msg = StartProtocol::<i32>::StartSession(StartInitiation {
            challenge: StartChallenge::MAX,
            target: SessionTarget::TcpStream(SealedHost::Plain(
                "example-of-a-very-very-long-second-level-name.on-a-very-very-long-domain-name.info:65530".parse()?,
            )),
            capabilities: HashSet::from_iter([Capability::Retransmission, Capability::Segmentation]),
            back_routing: Some((
                RoutingOptions::IntermediatePath(
                    vec![PeerId::random(), PeerId::random(), PeerId::random()].try_into()?,
                ),
                PeerId::random(),
            )),
        });

        assert!(
            msg.encode()?.1.len() <= PAYLOAD_SIZE,
            "StartSession must fit within {PAYLOAD_SIZE}"
        );

        let msg = StartProtocol::SessionEstablished(StartEstablished {
            orig_challenge: StartChallenge::MAX,
            session_id: SessionId::new(u16::MAX, PeerId::random()),
        });

        assert!(
            msg.encode()?.1.len() <= PAYLOAD_SIZE,
            "SessionEstablished must fit within {PAYLOAD_SIZE}"
        );

        let msg = StartProtocol::<i32>::SessionError(StartErrorType {
            challenge: StartChallenge::MAX,
            reason: StartErrorReason::NoSlotsAvailable,
        });

        assert!(
            msg.encode()?.1.len() <= PAYLOAD_SIZE,
            "SessionError must fit within {PAYLOAD_SIZE}"
        );

        let msg = StartProtocol::CloseSession(SessionId::new(u16::MAX, PeerId::random()));
        assert!(
            msg.encode()?.1.len() <= PAYLOAD_SIZE,
            "CloseSession must fit within {PAYLOAD_SIZE}"
        );

        Ok(())
    }
}
