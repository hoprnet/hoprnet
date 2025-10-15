use thiserror::Error;
use hopr_api::Address;
use hopr_api::chain::HoprBalance;
use hopr_crypto_types::prelude::HalfKeyChallenge;
use hopr_internal_types::errors::CoreTypesError;

/// Error that can occur when processing an incoming packet.
#[derive(Debug, strum::EnumIs, strum::EnumTryAs)]
pub enum IncomingPacketError<E> {
    /// Packet is not decodable.
    ///
    /// Such errors are fatal and therefore the packet cannot be acknowledged.
    Undecodable(E),
    /// Packet is decodable but cannot be processed due to other reasons.
    ///
    /// Such errors are protocol-related and packets must be acknowledged.
    ProcessingError(E),
}

#[derive(Error, Debug)]
pub enum PacketProcessorError {
    #[error("packet is in invalid state: {0}")]
    InvalidState(&'static str),

    #[error("failed to resolve chain key or packet key")]
    KeyNotFound,

    #[error("channel with counterparty {0} is below {1}")]
    OutOfFunds(Address, HoprBalance),

    #[error("failed to find channel {0} -> {1}")]
    ChannelNotFound(Address, Address),

    #[error("could not find unacknowledged ticket for challenge {0}")]
    UnacknowledgedTicketNotFound(HalfKeyChallenge),

    #[error("chain resolver error: {0}")]
    ResolverError(anyhow::Error),

    #[error("node db error: {0}")]
    NodeDbError(anyhow::Error),

    #[error(transparent)]
    TicketValidationError(hopr_crypto_packet::errors::TicketValidationError),

    #[error(transparent)]
    CoreTypesError(#[from] CoreTypesError),

    #[error(transparent)]
    PacketError(#[from] hopr_crypto_packet::errors::PacketError),

    #[error(transparent)]
    GeneralError(#[from] GeneralError),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}