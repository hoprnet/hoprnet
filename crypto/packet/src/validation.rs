use hopr_crypto_types::types::Hash;
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::*;
use tracing::{debug, instrument};

use crate::errors::TicketValidationError;

/// Performs validations of the given unacknowledged ticket and channel.
/// This is a higher-level function, hence it is not in `hopr-internal-types` crate.
#[instrument(level = "trace", skip_all, err)]
pub fn validate_unacknowledged_ticket(
    ticket: Ticket,
    channel: &ChannelEntry,
    min_ticket_amount: HoprBalance,
    required_win_prob: WinningProbability,
    unrealized_balance: HoprBalance,
    domain_separator: &Hash,
) -> Result<VerifiedTicket, TicketValidationError> {
    debug!(source = %channel.source, %ticket, "validating unacknowledged ticket");

    // The ticket signer MUST be the sender
    let verified_ticket = ticket
        .verify(&channel.source, domain_separator)
        .map_err(|ticket| TicketValidationError {
            reason: format!("ticket signer does not match the sender: {ticket}"),
            ticket,
        })?;

    let inner_ticket = verified_ticket.verified_ticket();

    // The ticket amount MUST be greater or equal to min_ticket_amount
    if !inner_ticket.amount.ge(&min_ticket_amount) {
        return Err(TicketValidationError {
            reason: format!(
                "ticket amount {} in not at least {min_ticket_amount}",
                inner_ticket.amount
            ),
            ticket: (*inner_ticket).into(),
        });
    }

    // The ticket must have at least the required winning probability
    if verified_ticket.win_prob().approx_cmp(&required_win_prob).is_lt() {
        return Err(TicketValidationError {
            reason: format!(
                "ticket winning probability {} is lower than required winning probability {required_win_prob}",
                verified_ticket.win_prob()
            ),
            ticket: (*inner_ticket).into(),
        });
    }

    // The channel MUST be open or pending to close
    if channel.status == ChannelStatus::Closed {
        return Err(TicketValidationError {
            reason: format!("payment channel {} is not opened or pending to close", channel.get_id()),
            ticket: (*inner_ticket).into(),
        });
    }

    // The ticket's channelEpoch MUST match the current channel's epoch
    if channel.channel_epoch != inner_ticket.channel_epoch {
        return Err(TicketValidationError {
            reason: format!(
                "ticket was created for a different channel iteration {} != {} of channel {}",
                inner_ticket.channel_epoch,
                channel.channel_epoch,
                channel.get_id()
            ),
            ticket: (*inner_ticket).into(),
        });
    }

    // Ensure that the sender has enough funds
    debug!(%unrealized_balance, channel_id = %channel.get_id(), "checking if sender has enough funds");
    if inner_ticket.amount.gt(&unrealized_balance) {
        return Err(TicketValidationError {
            reason: format!(
                "ticket value {} is greater than remaining unrealized balance {unrealized_balance} for channel {}",
                inner_ticket.amount,
                channel.get_id()
            ),
            ticket: (*inner_ticket).into(),
        });
    }

    Ok(verified_ticket)
}

#[cfg(test)]
mod tests {
    use std::ops::Add;

    use hex_literal::hex;
    use hopr_crypto_types::prelude::*;
    use lazy_static::lazy_static;

    use super::*;
    use crate::validation::validate_unacknowledged_ticket;

    const SENDER_PRIV_BYTES: [u8; 32] = hex!("492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775");
    const TARGET_PRIV_BYTES: [u8; 32] = hex!("5bf21ea8cccd69aa784346b07bf79c84dac606e00eecaa68bf8c31aff397b1ca");

    lazy_static! {
        static ref SENDER_PRIV_KEY: ChainKeypair =
            ChainKeypair::from_secret(&SENDER_PRIV_BYTES).expect("lazy static keypair should be valid");
        static ref TARGET_PRIV_KEY: ChainKeypair =
            ChainKeypair::from_secret(&TARGET_PRIV_BYTES).expect("lazy static keypair should be valid");
    }

    fn create_valid_ticket() -> anyhow::Result<Ticket> {
        Ok(TicketBuilder::default()
            .counterparty(&*TARGET_PRIV_KEY)
            .amount(1)
            .index(1)
            .win_prob(1.0.try_into()?)
            .channel_epoch(1)
            .eth_challenge(Default::default())
            .build_signed(&SENDER_PRIV_KEY, &Hash::default())?
            .leak())
    }

    fn create_channel_entry() -> ChannelEntry {
        ChannelEntry::new(
            SENDER_PRIV_KEY.public().to_address(),
            TARGET_PRIV_KEY.public().to_address(),
            100.into(),
            0,
            ChannelStatus::Open,
            1,
        )
    }

    #[tokio::test]
    async fn test_ticket_validation_should_pass_if_ticket_ok() -> anyhow::Result<()> {
        let ticket = create_valid_ticket()?;
        let channel = create_channel_entry();

        let more_than_ticket_balance = ticket.amount.add(500);

        let ret = validate_unacknowledged_ticket(
            ticket,
            &channel,
            1.into(),
            1.0.try_into()?,
            more_than_ticket_balance,
            &Hash::default(),
        );

        assert!(ret.is_ok());

        Ok(())
    }

    #[tokio::test]
    async fn test_ticket_validation_should_fail_if_signer_not_sender() -> anyhow::Result<()> {
        let ticket = create_valid_ticket()?;
        let channel = create_channel_entry();

        let ret = validate_unacknowledged_ticket(
            ticket,
            &channel,
            1.into(),
            1.0f64.try_into()?,
            0.into(),
            &Hash::default(),
        );

        assert!(ret.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn test_ticket_validation_should_fail_if_ticket_amount_is_low() -> anyhow::Result<()> {
        let ticket = create_valid_ticket()?;
        let channel = create_channel_entry();

        let ret =
            validate_unacknowledged_ticket(ticket, &channel, 2.into(), 1.0.try_into()?, 0.into(), &Hash::default());

        assert!(ret.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn test_ticket_validation_should_fail_if_ticket_chance_is_low() -> anyhow::Result<()> {
        let mut ticket = create_valid_ticket()?;
        ticket.encoded_win_prob = WinningProbability::try_from(0.5f64)?.into();
        let ticket = ticket
            .sign(&SENDER_PRIV_KEY, &Hash::default())
            .verified_ticket()
            .clone();

        let channel = create_channel_entry();

        let ret =
            validate_unacknowledged_ticket(ticket, &channel, 1.into(), 1.0.try_into()?, 0.into(), &Hash::default());

        assert!(ret.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn test_ticket_validation_should_fail_if_channel_is_closed() -> anyhow::Result<()> {
        let ticket = create_valid_ticket()?;
        let mut channel = create_channel_entry();
        channel.status = ChannelStatus::Closed;

        let ret =
            validate_unacknowledged_ticket(ticket, &channel, 1.into(), 1.0.try_into()?, 0.into(), &Hash::default());

        assert!(ret.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn test_ticket_validation_should_fail_if_ticket_epoch_does_not_match_2() -> anyhow::Result<()> {
        let mut ticket = create_valid_ticket()?;
        ticket.channel_epoch = 2u32;
        let ticket = ticket
            .sign(&SENDER_PRIV_KEY, &Hash::default())
            .verified_ticket()
            .clone();

        let channel = create_channel_entry();

        let ret =
            validate_unacknowledged_ticket(ticket, &channel, 1.into(), 1.0.try_into()?, 0.into(), &Hash::default());

        assert!(ret.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn test_ticket_validation_fail_if_does_not_have_funds() -> anyhow::Result<()> {
        let ticket = create_valid_ticket()?;
        let mut channel = create_channel_entry();
        channel.balance = 0.into();
        channel.channel_epoch = ticket.channel_epoch;

        let ret =
            validate_unacknowledged_ticket(ticket, &channel, 1.into(), 1.0.try_into()?, 0.into(), &Hash::default());

        assert!(ret.is_err());

        Ok(())
    }
}
