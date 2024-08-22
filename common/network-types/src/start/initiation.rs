use hopr_crypto_types::prelude::PeerId;

use crate::errors::NetworkTypeError;
use crate::start::prelude::StartError;
use crate::types::RoutingOptions;

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

/// Defines what should happen with the data at the recipient where the
/// data from the established session are supposed to be forwarded to some `target`.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum StartSessionTarget {
    /// Target is running over UDP with the given IP address and port.
    /// Host names are not supported.
    UdpStream(std::net::SocketAddr),
    /// Target is running over TCP with the given address and port.
    /// Host names are not supported
    TcpStream(std::net::SocketAddr),
    /// Target is a service directly at the exit node with a given service ID.
    ExitNode(u32),
}

/// The session initiation message of the Start protocol.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StartInitiation {
    /// Random challenge for this initiation.
    pub challenge: StartChallenge,
    /// [Target](StartSessionTarget) of the session, i.e., what should the other party do with the traffic.
    pub target: StartSessionTarget,
    /// Optional information on back routing from the other party back towards the
    /// session initiator.
    ///
    /// **NOTE:** This will not be used when the Return Path is introduced.
    pub back_routing: Option<(RoutingOptions, PeerId)>,
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

/// Lists all messages of the Start protocol for a session establishment
/// with `T` as session identifier.
#[derive(Debug, Clone, PartialEq, Eq, strum::EnumDiscriminants)]
// #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))] -- enforce serialization via encode/decode
#[strum_discriminants(derive(strum::FromRepr), repr(u8))]
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
impl<'a, T: serde::Serialize + serde::Deserialize<'a>> StartProtocol<T> {
    /// Serialize the message into a message tag and message data.
    /// Data is serialized using `bincode`.
    pub fn encode(self) -> crate::errors::Result<(u16, Box<[u8]>)> {
        let disc = StartProtocolDiscriminants::from(&self) as u8;
        let inner = match self {
            StartProtocol::StartSession(init) => bincode::serialize(&init),
            StartProtocol::SessionEstablished(est) => bincode::serialize(&est),
            StartProtocol::SessionError(err) => bincode::serialize(&err),
            StartProtocol::CloseSession(id) => bincode::serialize(&id),
        }
        .map_err(|_| NetworkTypeError::StartProtocolError(StartError::SerializerError))?;

        Ok((disc as u16, inner.into_boxed_slice()))
    }

    /// Deserialize the message from message tag and message data.
    /// Data is deserialized using `bincode`.
    pub fn decode(tag: u16, data: &'a [u8]) -> crate::errors::Result<Self> {
        match StartProtocolDiscriminants::from_repr(tag as u8)
            .ok_or(NetworkTypeError::StartProtocolError(StartError::ParseError))?
        {
            StartProtocolDiscriminants::StartSession => Ok(StartProtocol::StartSession(
                bincode::deserialize(data).map_err(|_| StartError::SerializerError)?,
            )),
            StartProtocolDiscriminants::SessionEstablished => Ok(StartProtocol::SessionEstablished(
                bincode::deserialize(data).map_err(|_| StartError::SerializerError)?,
            )),
            StartProtocolDiscriminants::SessionError => Ok(StartProtocol::SessionError(
                bincode::deserialize(data).map_err(|_| StartError::SerializerError)?,
            )),
            StartProtocolDiscriminants::CloseSession => Ok(StartProtocol::CloseSession(
                bincode::deserialize(data).map_err(|_| StartError::SerializerError)?,
            )),
        }
    }
}
