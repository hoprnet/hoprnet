use tracing::{debug, trace};

use hopr_crypto_types::types::Hash;
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::*;

use crate::errors::TicketValidationError;

/// Performs validations of the given unacknowledged ticket and channel.
/// This is a higher-level function, hence it is not in `hopr-internal-types` crate.
pub fn validate_unacknowledged_ticket(
    ticket: Ticket,
    channel: &ChannelEntry,
    min_ticket_amount: Balance,
    required_win_prob: f64,
    unrealized_balance: Balance,
    domain_separator: &Hash,
) -> Result<VerifiedTicket, TicketValidationError> {
    debug!("validating unack ticket from {}", channel.source);

    // ticket signer MUST be the sender
    let verified_ticket = ticket
        .verify(&channel.source, domain_separator)
        .map_err(|ticket| TicketValidationError {
            reason: format!("ticket signer does not match the sender: {ticket}"),
            ticket,
        })?;

    let inner_ticket = verified_ticket.verified_ticket();

    // ticket amount MUST be greater or equal to minTicketAmount
    if !inner_ticket.amount.ge(&min_ticket_amount) {
        return Err(TicketValidationError {
            reason: format!(
                "ticket amount {} in not at least {min_ticket_amount}",
                inner_ticket.amount
            ),
            ticket: inner_ticket.clone().into(),
        });
    }

    // ticket must have at least required winning probability
    if verified_ticket.win_prob() < required_win_prob {
        return Err(TicketValidationError {
            reason: format!(
                "ticket winning probability {} is lower than required winning probability {required_win_prob}",
                verified_ticket.win_prob()
            ),
            ticket: inner_ticket.clone().into(),
        });
    }

    // channel MUST be open or pending to close
    if channel.status == ChannelStatus::Closed {
        return Err(TicketValidationError {
            reason: format!("payment channel {} is not opened or pending to close", channel.get_id()),
            ticket: inner_ticket.clone().into(),
        });
    }

    // ticket's channelEpoch MUST match the current channel's epoch
    if !channel.channel_epoch.eq(&inner_ticket.channel_epoch.into()) {
        return Err(TicketValidationError {
            reason: format!(
                "ticket was created for a different channel iteration {} != {} of channel {}",
                inner_ticket.channel_epoch,
                channel.channel_epoch,
                channel.get_id()
            ),
            ticket: inner_ticket.clone().into(),
        });
    }

    // ensure sender has enough funds
    if inner_ticket.amount.gt(&unrealized_balance) {
        return Err(TicketValidationError {
            reason: format!(
                "ticket value {} is greater than remaining unrealized balance {unrealized_balance} for channel {}",
                inner_ticket.amount,
                channel.get_id()
            ),
            ticket: inner_ticket.clone().into(),
        });
    }

    trace!("ticket validation done");
    Ok(verified_ticket)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validation::validate_unacknowledged_ticket;
    use hex_literal::hex;
    use hopr_crypto_types::prelude::*;
    use lazy_static::lazy_static;
    use std::ops::Add;

    const SENDER_PRIV_BYTES: [u8; 32] = hex!("492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775");
    const TARGET_PRIV_BYTES: [u8; 32] = hex!("5bf21ea8cccd69aa784346b07bf79c84dac606e00eecaa68bf8c31aff397b1ca");

    lazy_static! {
        static ref SENDER_PRIV_KEY: ChainKeypair = ChainKeypair::from_secret(&SENDER_PRIV_BYTES).unwrap();
        static ref TARGET_PRIV_KEY: ChainKeypair = ChainKeypair::from_secret(&TARGET_PRIV_BYTES).unwrap();
    }

    fn create_valid_ticket() -> Ticket {
        TicketBuilder::default()
            .addresses(&*SENDER_PRIV_KEY, &*TARGET_PRIV_KEY)
            .amount(1)
            .index(1)
            .index_offset(1)
            .win_prob(1.0)
            .channel_epoch(1)
            .challenge(Default::default())
            .build_signed(&SENDER_PRIV_KEY, &Hash::default())
            .unwrap()
            .leak()
    }

    fn create_channel_entry() -> ChannelEntry {
        ChannelEntry::new(
            SENDER_PRIV_KEY.public().to_address(),
            TARGET_PRIV_KEY.public().to_address(),
            Balance::new(100_u64, BalanceType::HOPR),
            U256::zero(),
            ChannelStatus::Open,
            U256::one(),
        )
    }

    #[async_std::test]
    async fn test_ticket_validation_should_pass_if_ticket_ok() {
        let ticket = create_valid_ticket();
        let channel = create_channel_entry();

        let more_than_ticket_balance = ticket.amount.add(&Balance::new(U256::from(500u128), BalanceType::HOPR));

        let ret = validate_unacknowledged_ticket(
            ticket,
            &channel,
            Balance::new(1_u64, BalanceType::HOPR),
            1.0f64,
            more_than_ticket_balance,
            &Hash::default(),
        );

        assert!(ret.is_ok());
    }

    #[async_std::test]
    async fn test_ticket_validation_should_fail_if_signer_not_sender() {
        let ticket = create_valid_ticket();
        let channel = create_channel_entry();

        let ret = validate_unacknowledged_ticket(
            ticket,
            &channel,
            Balance::new(1_u64, BalanceType::HOPR),
            1.0f64,
            Balance::zero(BalanceType::HOPR),
            &Hash::default(),
        );

        assert!(ret.is_err());
    }

    #[async_std::test]
    async fn test_ticket_validation_should_fail_if_ticket_amount_is_low() {
        let ticket = create_valid_ticket();
        let channel = create_channel_entry();

        let ret = validate_unacknowledged_ticket(
            ticket,
            &channel,
            Balance::new(2_u64, BalanceType::HOPR),
            1.0f64,
            Balance::zero(BalanceType::HOPR),
            &Hash::default(),
        );

        assert!(ret.is_err());
    }

    #[async_std::test]
    async fn test_ticket_validation_should_fail_if_ticket_chance_is_low() {
        let mut ticket = create_valid_ticket();
        ticket.encoded_win_prob = f64_to_win_prob(0.5f64).unwrap();
        let ticket = ticket
            .sign(&SENDER_PRIV_KEY, &Hash::default())
            .verified_ticket()
            .clone();

        let channel = create_channel_entry();

        let ret = validate_unacknowledged_ticket(
            ticket,
            &channel,
            Balance::new(1_u64, BalanceType::HOPR),
            1.0_f64,
            Balance::zero(BalanceType::HOPR),
            &Hash::default(),
        );

        assert!(ret.is_err());
    }

    #[async_std::test]
    async fn test_ticket_validation_should_fail_if_channel_is_closed() {
        let ticket = create_valid_ticket();
        let mut channel = create_channel_entry();
        channel.status = ChannelStatus::Closed;

        let ret = validate_unacknowledged_ticket(
            ticket,
            &channel,
            Balance::new(1_u64, BalanceType::HOPR),
            1.0_f64,
            Balance::zero(BalanceType::HOPR),
            &Hash::default(),
        );

        assert!(ret.is_err());
    }

    #[async_std::test]
    async fn test_ticket_validation_should_fail_if_ticket_epoch_does_not_match_2() {
        let mut ticket = create_valid_ticket();
        ticket.channel_epoch = 2u32;
        let ticket = ticket
            .sign(&SENDER_PRIV_KEY, &Hash::default())
            .verified_ticket()
            .clone();

        let channel = create_channel_entry();

        let ret = validate_unacknowledged_ticket(
            ticket,
            &channel,
            Balance::new(1_u64, BalanceType::HOPR),
            1.0_f64,
            Balance::zero(BalanceType::HOPR),
            &Hash::default(),
        );

        assert!(ret.is_err());
    }

    #[async_std::test]
    async fn test_ticket_validation_fail_if_does_not_have_funds() {
        let ticket = create_valid_ticket();
        let mut channel = create_channel_entry();
        channel.balance = Balance::zero(BalanceType::HOPR);
        channel.channel_epoch = U256::from(ticket.channel_epoch);

        let ret = validate_unacknowledged_ticket(
            ticket,
            &channel,
            Balance::new(1_u64, BalanceType::HOPR),
            1.0_f64,
            Balance::zero(BalanceType::HOPR),
            &Hash::default(),
        );

        assert!(ret.is_err());
    }
}
