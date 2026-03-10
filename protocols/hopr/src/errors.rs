use hopr_api::{
    Address,
    types::{
        crypto::{prelude::HalfKeyChallenge, types::OffchainPublicKey},
        internal::prelude::ChannelId,
        primitive::balance::HoprBalance,
    },
};
use hopr_crypto_packet::errors::TicketValidationError;
use thiserror::Error;

/// Error that can occur when processing an incoming packet.
#[derive(Debug, strum::EnumIs, strum::EnumTryAs, Error)]
pub enum IncomingPacketError<E: std::error::Error> {
    /// Packet is not decodable.
    ///
    /// Such errors are fatal and therefore the packet cannot be acknowledged.
    #[error("packet is not decodable: {0}")]
    Undecodable(E),
    /// Packet is decodable but cannot be processed due to other reasons.
    ///
    /// Such errors are protocol-related and packets must be acknowledged.
    #[error("packet from {0} decodable, but cannot be processed: {1}")]
    ProcessingError(Box<OffchainPublicKey>, E),
    /// Packet is decodable, but the ticket is invalid.
    #[error("packet from {0} is decodable, but the ticket is invalid: {1}")]
    InvalidTicket(Box<OffchainPublicKey>, TicketValidationError),
}

impl<E: std::error::Error> IncomingPacketError<E> {
    /// Packet is undecodable and should NOT be acknowledged.
    pub fn undecodable<Err: Into<E>>(error: Err) -> Self {
        Self::Undecodable(error.into())
    }
}

/// Error that can occur when creating a ticket.
#[derive(Debug, strum::EnumIs, strum::EnumTryAs, Error)]
pub enum TicketCreationError<E: std::error::Error> {
    #[error("channel {0} does not have at least {1} to create a ticket")]
    OutOfFunds(ChannelId, HoprBalance),
    #[error("could not create ticket: {0}")]
    Other(E),
}

#[derive(Error, Debug)]
pub enum HoprProtocolError {
    #[error("packet is in invalid state: {0}")]
    InvalidState(&'static str),

    #[error("cannot decode the sender address of the packet")]
    InvalidSender,

    #[error("failed to resolve chain key or packet key")]
    KeyNotFound,

    #[error("channel {0} does not have at least {1} to create a ticket")]
    OutOfFunds(ChannelId, HoprBalance),

    #[error("packet replay detected")]
    Replay,

    #[error("failed to find channel {0} -> {1}")]
    ChannelNotFound(Address, Address),

    #[error("could not find unacknowledged ticket for challenge {0}")]
    UnacknowledgedTicketNotFound(HalfKeyChallenge),

    #[error("chain resolver error: {0}")]
    ResolverError(#[source] anyhow::Error),

    #[error("ticket tracker error: {0}")]
    TicketTrackerError(#[source] anyhow::Error),

    #[error(transparent)]
    TicketValidationError(#[from] TicketValidationError),

    #[error(transparent)]
    CoreTypesError(#[from] hopr_api::types::internal::errors::CoreTypesError),

    #[error(transparent)]
    PacketError(#[from] hopr_crypto_packet::errors::PacketError),

    #[error(transparent)]
    GeneralError(#[from] hopr_api::types::primitive::errors::GeneralError),

    #[error("rayon thread pool queue full: {0}")]
    SpawnError(#[from] hopr_parallelize::cpu::SpawnError),
}

impl HoprProtocolError {
    pub fn resolver<E: Into<anyhow::Error>>(e: E) -> Self {
        Self::ResolverError(e.into())
    }

    pub fn ticket_tracker<E: Into<anyhow::Error>>(e: E) -> Self {
        Self::TicketTrackerError(e.into())
    }
}
