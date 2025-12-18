use hopr_api::Address;
use hopr_crypto_packet::errors::TicketValidationError;
use hopr_crypto_types::{prelude::HalfKeyChallenge, types::OffchainPublicKey};
use hopr_internal_types::prelude::ChannelId;
use hopr_primitive_types::balance::HoprBalance;
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
    ProcessingError(OffchainPublicKey, E),
    /// Packet is decodable, but the ticket is invalid.
    #[error("packet from {0} is decodable, but the ticket is invalid: {1}")]
    InvalidTicket(OffchainPublicKey, TicketValidationError),
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
    CoreTypesError(#[from] hopr_internal_types::errors::CoreTypesError),

    #[error(transparent)]
    PacketError(#[from] hopr_crypto_packet::errors::PacketError),

    #[error(transparent)]
    GeneralError(#[from] hopr_primitive_types::errors::GeneralError),
}
