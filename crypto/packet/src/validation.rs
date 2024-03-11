use tracing::{debug, trace};

use crate::errors::{
    PacketError::{OutOfFunds, TicketValidation},
    Result,
};
use hopr_crypto_types::types::Hash;
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::*;

/// Performs validations of the given unacknowledged ticket and channel.
#[allow(clippy::too_many_arguments)] // TODO: The number of arguments and the logic needs to be refactored
pub async fn validate_unacknowledged_ticket(
    ticket: &Ticket,
    channel: &ChannelEntry,
    sender: &Address,
    min_ticket_amount: Balance,
    required_win_prob: f64,
    unrealized_balance: Option<Balance>,
    domain_separator: &Hash,
) -> Result<()> {
    debug!("validating unack ticket from {sender}");

    // ticket signer MUST be the sender
    ticket
        .verify(sender, domain_separator)
        .map_err(|e| TicketValidation(format!("ticket signer does not match the sender: {e}")))?;

    // ticket amount MUST be greater or equal to minTicketAmount
    if !ticket.amount.ge(&min_ticket_amount) {
        return Err(TicketValidation(format!(
            "ticket amount {} in not at least {min_ticket_amount}",
            ticket.amount
        )));
    }

    // ticket must have at least required winning probability
    if ticket.win_prob() < required_win_prob {
        return Err(TicketValidation(format!(
            "ticket winning probability {} is lower than required winning probability {required_win_prob}",
            ticket.win_prob()
        )));
    }

    // channel MUST be open or pending to close
    if channel.status == ChannelStatus::Closed {
        return Err(TicketValidation(format!(
            "payment channel with {sender} is not opened or pending to close"
        )));
    }

    // ticket's channelEpoch MUST match the current channel's epoch
    if !channel.channel_epoch.eq(&ticket.channel_epoch.into()) {
        return Err(TicketValidation(format!(
            "ticket was created for a different channel iteration {} != {} of channel {}",
            ticket.channel_epoch,
            channel.channel_epoch,
            channel.get_id()
        )));
    }

    if let Some(unrealized_balance) = unrealized_balance {
        // ensure sender has enough funds
        if ticket.amount.gt(&unrealized_balance) {
            return Err(OutOfFunds(channel.get_id().to_string()));
        }
    }

    trace!("ticket validation done");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::PacketError;
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
        Ticket::new(
            &TARGET_PRIV_KEY.public().to_address(),
            &Balance::new(1_u64, BalanceType::HOPR),
            1u64.into(),
            1u64.into(),
            1.0f64,
            1u64.into(),
            EthereumChallenge::default(),
            &SENDER_PRIV_KEY,
            &Hash::default(),
        )
        .unwrap()
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
            &ticket,
            &channel,
            &SENDER_PRIV_KEY.public().to_address(),
            Balance::new(1_u64, BalanceType::HOPR),
            1.0f64,
            Some(more_than_ticket_balance),
            &Hash::default(),
        )
        .await;
        assert!(ret.is_ok());
    }

    #[async_std::test]
    async fn test_ticket_validation_should_fail_if_signer_not_sender() {
        let ticket = create_valid_ticket();
        let channel = create_channel_entry();

        let ret = validate_unacknowledged_ticket(
            &ticket,
            &channel,
            &TARGET_PRIV_KEY.public().to_address(),
            Balance::new(1_u64, BalanceType::HOPR),
            1.0f64,
            Some(Balance::zero(BalanceType::HOPR)),
            &Hash::default(),
        )
        .await;

        assert!(ret.is_err());
        match ret.unwrap_err() {
            PacketError::TicketValidation(_) => {}
            _ => panic!("invalid error type"),
        }
    }

    #[async_std::test]
    async fn test_ticket_validation_should_fail_if_ticket_amount_is_low() {
        let ticket = create_valid_ticket();
        let channel = create_channel_entry();

        let ret = validate_unacknowledged_ticket(
            &ticket,
            &channel,
            &SENDER_PRIV_KEY.public().to_address(),
            Balance::new(2_u64, BalanceType::HOPR),
            1.0f64,
            Some(Balance::zero(BalanceType::HOPR)),
            &Hash::default(),
        )
        .await;

        assert!(ret.is_err());
        match ret.unwrap_err() {
            PacketError::TicketValidation(_) => {}
            _ => panic!("invalid error type"),
        }
    }

    #[async_std::test]
    async fn test_ticket_validation_should_fail_if_ticket_chance_is_low() {
        let mut ticket = create_valid_ticket();
        ticket.encoded_win_prob = f64_to_win_prob(0.5f64).unwrap();
        ticket.sign(&SENDER_PRIV_KEY, &Hash::default());

        let channel = create_channel_entry();

        let ret = validate_unacknowledged_ticket(
            &ticket,
            &channel,
            &SENDER_PRIV_KEY.public().to_address(),
            Balance::new(1_u64, BalanceType::HOPR),
            1.0_f64,
            Some(Balance::zero(BalanceType::HOPR)),
            &Hash::default(),
        )
        .await;

        assert!(ret.is_err());
        match ret.unwrap_err() {
            PacketError::TicketValidation(_) => {}
            _ => panic!("invalid error type"),
        }
    }

    #[async_std::test]
    async fn test_ticket_validation_should_fail_if_channel_is_closed() {
        let ticket = create_valid_ticket();
        let mut channel = create_channel_entry();
        channel.status = ChannelStatus::Closed;

        let ret = validate_unacknowledged_ticket(
            &ticket,
            &channel,
            &SENDER_PRIV_KEY.public().to_address(),
            Balance::new(1_u64, BalanceType::HOPR),
            1.0_f64,
            Some(Balance::zero(BalanceType::HOPR)),
            &Hash::default(),
        )
        .await;

        assert!(ret.is_err());
        match ret.unwrap_err() {
            PacketError::TicketValidation(_) => {}
            _ => panic!("invalid error type"),
        }
    }

    #[async_std::test]
    async fn test_ticket_validation_should_fail_if_ticket_epoch_does_not_match_2() {
        let mut ticket = create_valid_ticket();
        ticket.channel_epoch = 2u32;
        ticket.sign(&SENDER_PRIV_KEY, &Hash::default());

        let channel = create_channel_entry();

        let ret = validate_unacknowledged_ticket(
            &ticket,
            &channel,
            &SENDER_PRIV_KEY.public().to_address(),
            Balance::new(1_u64, BalanceType::HOPR),
            1.0_f64,
            Some(Balance::zero(BalanceType::HOPR)),
            &Hash::default(),
        )
        .await;

        assert!(ret.is_err());
        match ret.unwrap_err() {
            PacketError::TicketValidation(_) => {}
            _ => panic!("invalid error type"),
        }
    }

    #[async_std::test]
    async fn test_ticket_validation_fail_if_does_not_have_funds() {
        let ticket = create_valid_ticket();
        let mut channel = create_channel_entry();
        channel.balance = Balance::zero(BalanceType::HOPR);
        channel.channel_epoch = U256::from(ticket.channel_epoch);

        let ret = validate_unacknowledged_ticket(
            &ticket,
            &channel,
            &SENDER_PRIV_KEY.public().to_address(),
            Balance::new(1_u64, BalanceType::HOPR),
            1.0_f64,
            Some(Balance::zero(BalanceType::HOPR)),
            &Hash::default(),
        )
        .await;

        assert!(ret.is_err());
        // assert_eq!(ret.unwrap_err().to_string(), "");
        match ret.unwrap_err() {
            PacketError::OutOfFunds(_) => {}
            _ => panic!("invalid error type"),
        }
    }
}
