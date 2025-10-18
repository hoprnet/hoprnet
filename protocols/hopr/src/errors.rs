use hopr_api::Address;
use hopr_crypto_packet::errors::TicketValidationError;
use hopr_crypto_types::prelude::HalfKeyChallenge;
use thiserror::Error;

/// Error that can occur when processing an incoming packet.
#[derive(Debug, strum::EnumIs, strum::EnumTryAs, Error)]
pub enum IncomingPacketError<E> {
    /// Packet is not decodable.
    ///
    /// Such errors are fatal and therefore the packet cannot be acknowledged.
    #[error("packet is not decodable: {0}")]
    Undecodable(E),
    /// Packet is decodable but cannot be processed due to other reasons.
    ///
    /// Such errors are protocol-related and packets must be acknowledged.
    #[error("packet is decodable, but cannot be processed: {0}")]
    ProcessingError(E),
    /// Packet is decodable, but the ticket is invalid.
    #[error("packet is decodable, but the ticket is invalid: {0}")]
    InvalidTicket(TicketValidationError),
}

#[derive(Error, Debug)]
pub enum HoprProtocolError {
    #[error("packet is in invalid state: {0}")]
    InvalidState(&'static str),

    #[error("failed to resolve chain key or packet key")]
    KeyNotFound,

    #[error("packet replay detected")]
    Replay,

    #[error("failed to find channel {0} -> {1}")]
    ChannelNotFound(Address, Address),

    #[error("could not find unacknowledged ticket for challenge {0}")]
    UnacknowledgedTicketNotFound(HalfKeyChallenge),

    #[error("chain resolver error: {0}")]
    ResolverError(anyhow::Error),

    #[error(transparent)]
    TicketValidationError(#[from] TicketValidationError),

    #[error(transparent)]
    CoreTypesError(#[from] hopr_internal_types::errors::CoreTypesError),

    #[error(transparent)]
    PacketError(#[from] hopr_crypto_packet::errors::PacketError),

    #[error(transparent)]
    GeneralError(#[from] hopr_primitive_types::errors::GeneralError),
}
