//! This crate defines the Start sub-protocol used for HOPR Session initiation and management.
//!
//! The Start protocol is used to establish Session as described in HOPR
//! [`RFC-0007`](https://github.com/hoprnet/rfc/tree/main/rfcs/RFC-0007-session-protocol).
//! and is implemented via the [`StartProtocol`] enum.
//!
//! The protocol is defined via generic arguments `I` (for Session ID), `T` (for Session Target)
//! and `C` (for Session capabilities).
//!
//! Per `RFC-0007`, the types `I` and `T` are serialized/deserialized to the CBOR binary format
//! (see [`RFC7049`](https://datatracker.ietf.org/doc/html/rfc7049)) and therefore must implement
//! `serde::Serialize + serde::Deserialize`.
//! The capability type `C` must be expressible as a single unsigned byte.
//!
//! See [`StartProtocol`] docs for the protocol diagram.

/// Contains errors raised by the Start protocol.
pub mod errors;

use hopr_crypto_packet::prelude::HoprPacket;
use hopr_protocol_app::prelude::{ApplicationData, ReservedTag, Tag};

use crate::errors::StartProtocolError;

/// Challenge that identifies a Start initiation protocol message.
pub type StartChallenge = u64;

/// Lists all Start protocol error reasons.
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, strum::Display, strum::FromRepr)]
pub enum StartErrorReason {
    /// Unknown error.
    Unknown = 0,
    /// No more slots are available at the recipient.
    NoSlotsAvailable = 1,
    /// Recipient is busy.
    Busy = 2,
}

/// Error message in the Start protocol.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct StartErrorType {
    /// Challenge that relates to this error.
    pub challenge: StartChallenge,
    /// The [reason](StartErrorReason) of this error.
    pub reason: StartErrorReason,
}

/// The session initiation message of the Start protocol.
///
/// ## Generic parameters
/// - `T` is the session target
/// - `C` are session capabilities
///
/// The `additional_data` are set dependent on the `capabilities`
/// or set to `0x00000000` to be ignored.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StartInitiation<T, C> {
    /// Random challenge for this initiation.
    pub challenge: StartChallenge,
    /// Target of the session, i.e., what should the other party do with the traffic.
    pub target: T,
    /// Capabilities of the session.
    pub capabilities: C,
    /// Additional options (might be `capabilities` dependent), ignored if `0x00000000`.
    pub additional_data: u32,
}

/// Message of the Start protocol that confirms the establishment of a session.
///
/// ## Generic parameters
/// `I` is for session identifier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StartEstablished<I> {
    /// Challenge that was used in the [initiation message](StartInitiation) to establish correspondence.
    pub orig_challenge: StartChallenge,
    /// Session ID that was selected by the recipient.
    pub session_id: I,
}

#[cfg_attr(doc, aquamarine::aquamarine)]
/// Lists all messages of the Start protocol for a session establishment.
///
/// ## Generic parameters
/// - `I` is the session identifier.
/// - `T` is the session target.
/// - `C` are session capabilities.
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
///     else If Exit cannot accept a new session
///     Exit->>Entry: SessionError (Challenge, Reason)
///     end
///     opt If initiation attempt times out
///     Note left of Entry: Failure
///     end
/// ```
#[derive(Debug, Clone, PartialEq, Eq, strum::EnumDiscriminants)]
#[strum_discriminants(vis(pub))]
#[strum_discriminants(derive(strum::FromRepr, strum::EnumCount), repr(u8))]
pub enum StartProtocol<I, T, C> {
    /// Request to initiate a new session.
    StartSession(StartInitiation<T, C>),
    /// Confirmation that a new session has been established by the counterparty.
    SessionEstablished(StartEstablished<I>),
    /// Counterparty could not establish a new session due to an error.
    SessionError(StartErrorType),
    /// A ping message to keep the session alive.
    KeepAlive(KeepAliveMessage<I>),
}

/// Keep-alive message for a Session with the identifier `T`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeepAliveMessage<I> {
    /// Session ID.
    pub session_id: I,
    /// Reserved for future use, always zero currently.
    pub flags: u8,
    /// Additional data (might be `flags` dependent), ignored if `0x00000000`.
    pub additional_data: u32,
}

impl<I> KeepAliveMessage<I> {
    /// The minimum number of SURBs a [`KeepAliveMessage`] must be able to carry.
    pub const MIN_SURBS_PER_MESSAGE: usize = HoprPacket::MAX_SURBS_IN_PACKET;
}

impl<I> From<I> for KeepAliveMessage<I> {
    fn from(value: I) -> Self {
        Self {
            session_id: value,
            flags: 0,
            additional_data: 0,
        }
    }
}

impl<I, T, C> StartProtocol<I, T, C> {
    /// Fixed [`Tag`] of every protocol message.
    pub const START_PROTOCOL_MESSAGE_TAG: Tag = Tag::Reserved(ReservedTag::SessionStart as u64);
    /// Current version of the Start protocol.
    pub const START_PROTOCOL_VERSION: u8 = 0x02;
}

impl<I, T, C> StartProtocol<I, T, C>
where
    I: serde::Serialize + for<'de> serde::Deserialize<'de>,
    T: serde::Serialize + for<'de> serde::Deserialize<'de>,
    C: Into<u8> + TryFrom<u8>,
{
    /// Tries to encode the message into binary format and [`Tag`]
    pub fn encode(self) -> errors::Result<(Tag, Box<[u8]>)> {
        let mut out = Vec::with_capacity(ApplicationData::PAYLOAD_SIZE);
        out.push(Self::START_PROTOCOL_VERSION);
        out.push(StartProtocolDiscriminants::from(&self) as u8);

        let mut data = Vec::with_capacity(ApplicationData::PAYLOAD_SIZE - 2);
        match self {
            StartProtocol::StartSession(init) => {
                data.extend_from_slice(&init.challenge.to_be_bytes());
                data.push(init.capabilities.into());
                data.extend_from_slice(&init.additional_data.to_be_bytes());
                let target = serde_cbor_2::to_vec(&init.target)?;
                data.extend_from_slice(&target);
            }
            StartProtocol::SessionEstablished(est) => {
                data.extend_from_slice(&est.orig_challenge.to_be_bytes());
                let session_id = serde_cbor_2::to_vec(&est.session_id)?;
                data.extend(session_id);
            }
            StartProtocol::SessionError(err) => {
                data.extend_from_slice(&err.challenge.to_be_bytes());
                data.push(err.reason as u8);
            }
            StartProtocol::KeepAlive(ping) => {
                data.push(ping.flags);
                data.extend_from_slice(&ping.additional_data.to_be_bytes());
                let session_id = serde_cbor_2::to_vec(&ping.session_id)?;
                data.extend(session_id);
            }
        }

        out.extend_from_slice(&(data.len() as u16).to_be_bytes());
        out.extend(data);

        Ok((Self::START_PROTOCOL_MESSAGE_TAG, out.into_boxed_slice()))
    }

    /// Tries to decode the message from the binary representation and [`Tag`].
    ///
    /// The `tag` must be currently [`START_PROTOCOL_MESSAGE_TAG`](Self::START_PROTOCOL_MESSAGE_TAG)
    /// and version [`START_PROTOCOL_VERSION`](Self::START_PROTOCOL_VERSION).
    pub fn decode(tag: Tag, data: &[u8]) -> errors::Result<Self> {
        if tag != Self::START_PROTOCOL_MESSAGE_TAG {
            return Err(StartProtocolError::UnknownTag);
        }

        if data.len() < 5 {
            return Err(StartProtocolError::InvalidLength);
        }

        if data[0] != Self::START_PROTOCOL_VERSION {
            return Err(StartProtocolError::InvalidVersion);
        }

        let disc = data[1];
        let len = u16::from_be_bytes(
            data[2..4]
                .try_into()
                .map_err(|_| StartProtocolError::ParseError("len".into()))?,
        ) as usize;
        let data_offset = 2 + size_of::<u16>();

        if data.len() < data_offset + len {
            return Err(StartProtocolError::InvalidLength);
        }

        Ok(
            match StartProtocolDiscriminants::from_repr(disc).ok_or(StartProtocolError::UnknownMessage)? {
                StartProtocolDiscriminants::StartSession => {
                    if data.len() <= data_offset + size_of::<StartChallenge>() + 1 + size_of::<u32>() {
                        return Err(StartProtocolError::InvalidLength);
                    }

                    StartProtocol::StartSession(StartInitiation {
                        challenge: StartChallenge::from_be_bytes(
                            data[data_offset..data_offset + size_of::<StartChallenge>()]
                                .try_into()
                                .map_err(|_| StartProtocolError::ParseError("init.challenge".into()))?,
                        ),
                        capabilities: data[data_offset + size_of::<StartChallenge>()]
                            .try_into()
                            .map_err(|_| StartProtocolError::ParseError("init.capabilities".into()))?,
                        additional_data: u32::from_be_bytes(
                            data[data_offset + size_of::<StartChallenge>() + 1
                                ..data_offset + size_of::<StartChallenge>() + 1 + size_of::<u32>()]
                                .try_into()
                                .map_err(|_| StartProtocolError::ParseError("init.additional_data".into()))?,
                        ),
                        target: serde_cbor_2::from_slice(
                            &data[data_offset + size_of::<StartChallenge>() + 1 + size_of::<u32>()..],
                        )?,
                    })
                }
                StartProtocolDiscriminants::SessionEstablished => {
                    if data.len() <= data_offset + size_of::<StartChallenge>() {
                        return Err(StartProtocolError::InvalidLength);
                    }
                    StartProtocol::SessionEstablished(StartEstablished {
                        orig_challenge: StartChallenge::from_be_bytes(
                            data[data_offset..data_offset + size_of::<StartChallenge>()]
                                .try_into()
                                .map_err(|_| StartProtocolError::ParseError("est.challenge".into()))?,
                        ),
                        session_id: serde_cbor_2::from_slice(&data[data_offset + size_of::<StartChallenge>()..])?,
                    })
                }
                StartProtocolDiscriminants::SessionError => {
                    if data.len() < data_offset + size_of::<StartChallenge>() + 1 {
                        return Err(StartProtocolError::InvalidLength);
                    }
                    StartProtocol::SessionError(StartErrorType {
                        challenge: StartChallenge::from_be_bytes(
                            data[data_offset..data_offset + size_of::<StartChallenge>()]
                                .try_into()
                                .map_err(|_| StartProtocolError::ParseError("err.challenge".into()))?,
                        ),
                        reason: StartErrorReason::from_repr(data[data_offset + size_of::<StartChallenge>()])
                            .ok_or(StartProtocolError::ParseError("err.reason".into()))?,
                    })
                }
                StartProtocolDiscriminants::KeepAlive => {
                    if data.len() <= data_offset + size_of::<u32>() {
                        return Err(StartProtocolError::InvalidLength);
                    }

                    StartProtocol::KeepAlive(KeepAliveMessage {
                        flags: data[data_offset],
                        additional_data: u32::from_be_bytes(
                            data[data_offset + 1..data_offset + 1 + size_of::<u32>()]
                                .try_into()
                                .map_err(|_| StartProtocolError::ParseError("ka.additional_data".into()))?,
                        ),
                        session_id: serde_cbor_2::from_slice(&data[data_offset + 1 + size_of::<u32>()..])?,
                    })
                }
            },
        )
    }
}

impl<I, T, C> TryFrom<StartProtocol<I, T, C>> for ApplicationData
where
    I: serde::Serialize + for<'de> serde::Deserialize<'de>,
    T: serde::Serialize + for<'de> serde::Deserialize<'de>,
    C: Into<u8> + TryFrom<u8>,
{
    type Error = StartProtocolError;

    fn try_from(value: StartProtocol<I, T, C>) -> Result<Self, Self::Error> {
        let (application_tag, plain_text) = value.encode()?;
        Ok(ApplicationData::new_from_owned(application_tag, plain_text))
    }
}

impl<I, T, C> TryFrom<ApplicationData> for StartProtocol<I, T, C>
where
    I: serde::Serialize + for<'de> serde::Deserialize<'de>,
    T: serde::Serialize + for<'de> serde::Deserialize<'de>,
    C: Into<u8> + TryFrom<u8>,
{
    type Error = StartProtocolError;

    fn try_from(value: ApplicationData) -> Result<Self, Self::Error> {
        Self::decode(value.application_tag, &value.plain_text)
    }
}

#[cfg(test)]
mod tests {
    use hopr_crypto_packet::prelude::HoprPacket;
    use hopr_protocol_app::prelude::Tag;

    use super::*;

    #[test]
    fn start_protocol_start_session_message_should_encode_and_decode() -> anyhow::Result<()> {
        let msg_1 = StartProtocol::StartSession(StartInitiation {
            challenge: 0,
            target: "127.0.0.1:1234".to_string(),
            capabilities: Default::default(),
            additional_data: 0x12345678,
        });

        let (tag, msg) = msg_1.clone().encode()?;
        let expected: Tag = StartProtocol::<(), (), ()>::START_PROTOCOL_MESSAGE_TAG;
        assert_eq!(tag, expected);

        let msg_2 = StartProtocol::<i32, String, u8>::decode(tag, &msg)?;

        assert_eq!(msg_1, msg_2);
        Ok(())
    }

    #[test]
    fn start_protocol_message_start_session_message_should_allow_for_at_least_one_surb() -> anyhow::Result<()> {
        let msg = StartProtocol::<i32, String, u8>::StartSession(StartInitiation {
            challenge: 0,
            target: "127.0.0.1:1234".to_string(),
            capabilities: 0xff,
            additional_data: 0xffffffff,
        });

        let len = msg.encode()?.1.len();
        assert!(
            HoprPacket::max_surbs_with_message(len) >= 1,
            "StartSession message size ({len}) must allow for at least 1 SURBs in packet",
        );

        Ok(())
    }

    #[test]
    fn start_protocol_session_established_message_should_encode_and_decode() -> anyhow::Result<()> {
        let msg_1 = StartProtocol::SessionEstablished(StartEstablished {
            orig_challenge: 0,
            session_id: 10_i32,
        });

        let (tag, msg) = msg_1.clone().encode()?;
        let expected: Tag = StartProtocol::<(), (), ()>::START_PROTOCOL_MESSAGE_TAG;
        assert_eq!(tag, expected);

        let msg_2 = StartProtocol::<i32, String, u8>::decode(tag, &msg)?;

        assert_eq!(msg_1, msg_2);
        Ok(())
    }

    #[test]
    fn start_protocol_session_error_message_should_encode_and_decode() -> anyhow::Result<()> {
        let msg_1 = StartProtocol::SessionError(StartErrorType {
            challenge: 10,
            reason: StartErrorReason::NoSlotsAvailable,
        });

        let (tag, msg) = msg_1.clone().encode()?;
        let expected: Tag = StartProtocol::<(), (), ()>::START_PROTOCOL_MESSAGE_TAG;
        assert_eq!(tag, expected);

        let msg_2 = StartProtocol::<i32, String, u8>::decode(tag, &msg)?;

        assert_eq!(msg_1, msg_2);
        Ok(())
    }

    #[test]
    fn start_protocol_keep_alive_message_should_encode_and_decode() -> anyhow::Result<()> {
        let msg_1 = StartProtocol::KeepAlive(KeepAliveMessage {
            session_id: 10_i32,
            flags: 0,
            additional_data: 0xffffffff,
        });

        let (tag, msg) = msg_1.clone().encode()?;
        let expected: Tag = StartProtocol::<(), (), ()>::START_PROTOCOL_MESSAGE_TAG;
        assert_eq!(tag, expected);

        let msg_2 = StartProtocol::<i32, String, u8>::decode(tag, &msg)?;

        assert_eq!(msg_1, msg_2);
        Ok(())
    }

    #[test]
    fn start_protocol_messages_must_fit_within_hopr_packet() -> anyhow::Result<()> {
        let msg = StartProtocol::<i32, String, u8>::StartSession(StartInitiation {
            challenge: StartChallenge::MAX,
            target: "example-of-a-very-very-long-second-level-name.on-a-very-very-long-domain-name.info:65530"
                .to_string(),
            capabilities: 0x80,
            additional_data: 0xffffffff,
        });

        assert!(
            msg.encode()?.1.len() <= HoprPacket::PAYLOAD_SIZE,
            "StartSession must fit within {}",
            HoprPacket::PAYLOAD_SIZE
        );

        let msg = StartProtocol::<String, String, u8>::SessionEstablished(StartEstablished {
            orig_challenge: StartChallenge::MAX,
            session_id: "example-of-a-very-very-long-session-id-that-should-still-fit-the-packet".to_string(),
        });

        assert!(
            msg.encode()?.1.len() <= HoprPacket::PAYLOAD_SIZE,
            "SessionEstablished must fit within {}",
            HoprPacket::PAYLOAD_SIZE
        );

        let msg = StartProtocol::<String, String, u8>::SessionError(StartErrorType {
            challenge: StartChallenge::MAX,
            reason: StartErrorReason::NoSlotsAvailable,
        });

        assert!(
            msg.encode()?.1.len() <= HoprPacket::PAYLOAD_SIZE,
            "SessionError must fit within {}",
            HoprPacket::PAYLOAD_SIZE
        );

        let msg = StartProtocol::<String, String, u8>::KeepAlive(KeepAliveMessage {
            session_id: "example-of-a-very-very-long-session-id-that-should-still-fit-the-packet".to_string(),
            flags: 0xff,
            additional_data: 0xffffffff,
        });
        assert!(
            msg.encode()?.1.len() <= HoprPacket::PAYLOAD_SIZE,
            "KeepAlive must fit within {}",
            HoprPacket::PAYLOAD_SIZE
        );

        Ok(())
    }

    #[test]
    fn start_protocol_message_keep_alive_message_should_allow_for_maximum_surbs() -> anyhow::Result<()> {
        let msg = StartProtocol::<String, String, u8>::KeepAlive(KeepAliveMessage {
            session_id: "example-of-a-very-very-long-session-id-that-should-still-fit-the-packet".to_string(),
            flags: 0xff,
            additional_data: 0xffffffff,
        });
        let len = msg.encode()?.1.len();
        assert_eq!(
            KeepAliveMessage::<String>::MIN_SURBS_PER_MESSAGE,
            HoprPacket::MAX_SURBS_IN_PACKET
        );
        assert!(
            HoprPacket::max_surbs_with_message(len) >= KeepAliveMessage::<String>::MIN_SURBS_PER_MESSAGE,
            "KeepAlive message size ({}) must allow for at least {} SURBs in packet",
            len,
            KeepAliveMessage::<String>::MIN_SURBS_PER_MESSAGE
        );

        Ok(())
    }
}
