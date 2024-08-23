//! This module defines the Start sub-protocol used for HOPR Session initiation and management.

use crate::errors::TransportSessionError;
use crate::types::SessionTarget;
use crate::Capability;
use hopr_crypto_types::prelude::PeerId;
use hopr_internal_types::prelude::ApplicationData;
use hopr_network_types::prelude::RoutingOptions;
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
        }?;

        Ok((disc as u16, inner.into_boxed_slice()))
    }

    /// Convenience method to [encode] directly into [ApplicationData].
    pub fn encode_as_app_data(self) -> crate::errors::Result<ApplicationData> {
        let (tag, plain_text) = self.encode()?;
        Ok(ApplicationData {
            application_tag: Some(tag),
            plain_text,
        })
    }

    /// Deserialize the message from message tag and message data.
    /// Data is deserialized using `bincode`.
    pub fn decode(tag: u16, data: &'a [u8]) -> crate::errors::Result<Self> {
        match StartProtocolDiscriminants::from_repr(tag as u8).ok_or(TransportSessionError::PayloadSize)? {
            StartProtocolDiscriminants::StartSession => Ok(StartProtocol::StartSession(bincode::deserialize(data)?)),
            StartProtocolDiscriminants::SessionEstablished => {
                Ok(StartProtocol::SessionEstablished(bincode::deserialize(data)?))
            }
            StartProtocolDiscriminants::SessionError => Ok(StartProtocol::SessionError(bincode::deserialize(data)?)),
            StartProtocolDiscriminants::CloseSession => Ok(StartProtocol::CloseSession(bincode::deserialize(data)?)),
        }
    }
}
