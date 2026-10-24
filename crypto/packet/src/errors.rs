use std::fmt::Debug;

use hopr_types::{
    crypto::{errors::CryptoError, prelude::SimplePseudonym},
    internal::{
        errors::CoreTypesError,
        prelude::{ChannelId, Ticket, WinningProbability},
    },
    primitive::{
        errors::GeneralError,
        prelude::{Address, HoprBalance},
    },
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PacketError {
    #[error("failed to decode packet: {0}")]
    PacketDecodingError(String),

    #[error("failed to construct packet: {0}")]
    PacketConstructionError(String),

    #[error("Proof of Relay challenge could not be verified")]
    PoRVerificationError,

    #[error("logic error during packet processing: {0}")]
    LogicError(String),

    #[error("underlying transport error while sending packet: {0}")]
    TransportError(String),

    #[error(transparent)]
    CryptographicError(#[from] CryptoError),

    #[error(transparent)]
    CoreTypesError(#[from] CoreTypesError),

    #[error(transparent)]
    SphinxError(#[from] crate::sphinx::errors::SphinxError),

    #[error(transparent)]
    PixError(#[from] hopr_protocol_pix::errors::PixError<SimplePseudonym>),

    #[error(transparent)]
    Other(#[from] GeneralError),
}

pub type Result<T> = std::result::Result<T, PacketError>;

/// Contains all possible validation errors that can occur during [ticket
/// validation](crate::validation::validate_unacknowledged_ticket).
#[derive(Debug, Clone, Copy, Error, strum::AsRefStr)]
#[strum(serialize_all = "snake_case")]
pub enum ValidationErrorKind {
    /// Ticket signer does not match the sender.
    #[error("ticket signer does not match the sender")]
    InvalidSigner,
    /// Ticket amount is lower than the given minimum value.
    #[error("ticket amount is lower than {0}")]
    LowValue(HoprBalance),
    /// Ticket winning probability is lower than the given minimum value.
    #[error("ticket winning probability is lower than {0}")]
    LowWinProb(WinningProbability),
    /// The given channel is closed or pending to close.
    #[error("channel {0} is closed or pending to close")]
    ChannelClosed(ChannelId),
    /// Ticket epoch does not match the given channel epoch.
    #[error("ticket epoch does not match channel epoch {0}")]
    EpochMismatch(u32),
    /// Ticket index is lower than the given channel ticket index.
    #[error("ticket index is lower than channel index {0}")]
    IndexTooLow(u64),
    /// Ticket counterparty does not match the channel destination.
    #[error("ticket counterparty does not match the channel destination {0}")]
    WrongCounterparty(Address),
    /// Not enough funds in the given channel to pay for the ticket.
    #[error("ticket value is greater than remaining unrealized balance {1} in channel {0}")]
    InsufficientFunds(ChannelId, HoprBalance),
}

/// Contains errors returned by [validate_unacknowledged_ticket](crate::validation::validate_unacknowledged_ticket).
#[derive(Debug, Clone, Error)]
#[error("validation error of {ticket}: {kind}")]
pub struct TicketValidationError {
    /// Error description.
    pub kind: ValidationErrorKind,
    /// Invalid ticket that failed to validate.
    pub ticket: Box<Ticket>,
    /// Issuer of the ticket.
    ///
    /// This value is present if at least the ticket signature validation succeeded.
    pub issuer: Option<Address>,
}

#[cfg(test)]
mod tests {
    use hopr_types::{
        crypto::prelude::{ChainKeypair, Keypair},
        internal::prelude::TicketBuilder,
        primitive::traits::BytesRepresentable,
    };

    use super::*;

    #[test]
    fn test_validation_error_kind_display() {
        let kind = ValidationErrorKind::LowWinProb(WinningProbability::ALWAYS);
        assert_eq!(
            kind.to_string(),
            format!(
                "ticket winning probability is lower than {}",
                WinningProbability::ALWAYS
            )
        );
        assert_eq!(kind.as_ref(), "low_win_prob");
    }

    /// The `AsRefStr` representation of every variant is used as the value of the `type` label of
    /// the `hopr_packet_ticket_validation_errors` metric, so the wire-visible names are pinned here
    /// to catch an accidental rename or an added variant that would silently change dashboards.
    #[test]
    fn validation_error_kind_metric_labels_are_stable() {
        let destination = Address::new(&[0x11; Address::SIZE]);

        let labels = [
            (ValidationErrorKind::InvalidSigner, "invalid_signer"),
            (ValidationErrorKind::LowValue(HoprBalance::zero()), "low_value"),
            (
                ValidationErrorKind::LowWinProb(WinningProbability::ALWAYS),
                "low_win_prob",
            ),
            (ValidationErrorKind::ChannelClosed(Default::default()), "channel_closed"),
            (ValidationErrorKind::EpochMismatch(1), "epoch_mismatch"),
            (ValidationErrorKind::IndexTooLow(1), "index_too_low"),
            (
                ValidationErrorKind::WrongCounterparty(destination),
                "wrong_counterparty",
            ),
            (
                ValidationErrorKind::InsufficientFunds(Default::default(), HoprBalance::zero()),
                "insufficient_funds",
            ),
        ];

        for (kind, expected) in labels {
            assert_eq!(kind.as_ref(), expected, "unexpected metric label for {kind:?}");
        }
    }

    /// The counterparty check is a rejection path reachable from an attacker-controlled ticket,
    /// so its message must name the expected destination.
    #[test]
    fn validation_error_kind_wrong_counterparty_display() {
        let destination = Address::new(&[0xAB; Address::SIZE]);
        let kind = ValidationErrorKind::WrongCounterparty(destination);

        assert!(
            kind.to_string().contains(&destination.to_string()),
            "message should name the channel destination, got: {kind}"
        );
    }

    #[test]
    fn test_ticket_validation_error_display() {
        let src = ChainKeypair::random();
        let dst = ChainKeypair::random();

        let ticket = TicketBuilder::zero_hop()
            .counterparty(&dst)
            .eth_challenge(Default::default())
            .build_signed(&src, &Default::default())
            .unwrap()
            .leak();

        let error = TicketValidationError {
            kind: ValidationErrorKind::LowWinProb(WinningProbability::default()),
            ticket: Box::new(ticket),
            issuer: None,
        };

        assert_eq!(
            error.to_string(),
            format!(
                "validation error of {ticket}: ticket winning probability is lower than {}",
                WinningProbability::ALWAYS
            )
        );
    }
}
