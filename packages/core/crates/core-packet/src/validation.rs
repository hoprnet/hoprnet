use crate::errors::PacketError::{OutOfFunds, TicketValidation};
use crate::errors::Result;
use core_crypto::types::PublicKey;
use core_ethereum_db::traits::HoprCoreEthereumDbActions;
use core_types::channels::{ChannelEntry, ChannelStatus, Ticket};
use utils_log::{debug, info};
use utils_types::primitives::{Balance, BalanceType, U256};

/// Performs validations of the given unacknowledged ticket and channel.
pub async fn validate_unacknowledged_ticket<T: HoprCoreEthereumDbActions>(
    db: &T,
    ticket: &Ticket,
    channel: &ChannelEntry,
    sender: &PublicKey,
    min_ticket_amount: Balance,
    req_inverse_ticket_win_prob: U256,
    check_unrealized_balance: bool,
) -> Result<()> {
    let required_win_prob = U256::from_inverse_probability(req_inverse_ticket_win_prob)?;

    // ticket signer MUST be the sender
    ticket
        .verify(sender)
        .map_err(|e| TicketValidation(format!("ticket signer does not match the sender: {e}")))?;

    // ticket amount MUST be greater or equal to minTicketAmount
    if !ticket.amount.gte(&min_ticket_amount) {
        return Err(TicketValidation(format!(
            "ticket amount {} in not at least {min_ticket_amount}",
            ticket.amount
        )));
    }

    // ticket MUST have match X winning probability
    if !ticket.win_prob.eq(&required_win_prob) {
        return Err(TicketValidation(format!(
            "ticket winning probability {} is not equal to {required_win_prob}",
            ticket.win_prob
        )));
    }

    // channel MUST be open or pending to close
    if channel.status == ChannelStatus::Closed {
        return Err(TicketValidation(format!(
            "payment channel with {sender} is not opened or pending to close"
        )));
    }

    // ticket's epoch MUST match our channel's epoch
    if !ticket.epoch.eq(&channel.ticket_epoch) {
        return Err(TicketValidation(format!(
            "ticket epoch {} does not match our account epoch {} of channel {}",
            ticket.epoch,
            channel.ticket_epoch,
            channel.get_id()
        )));
    }

    // ticket's channelEpoch MUST match the current channel's epoch
    if !ticket.channel_epoch.eq(&channel.channel_epoch) {
        return Err(TicketValidation(format!(
            "ticket was created for a different channel iteration {} != {} of channel {}",
            ticket.channel_epoch,
            channel.channel_epoch,
            channel.get_id()
        )));
    }

    if check_unrealized_balance {
        info!("checking unrealized balances for channel {}", channel.get_id());

        let unrealized_balance = db
            .get_tickets(sender)
            .await? // all tickets from sender
            .into_iter()
            .filter(|t| t.epoch.eq(&channel.ticket_epoch) && t.channel_epoch.eq(&channel.channel_epoch))
            .fold(Some(channel.balance), |result, t| {
                result
                    .and_then(|b| b.value().value().checked_sub(t.amount.value().value().clone()))
                    .map(|u| Balance::new(u.into(), channel.balance.balance_type()))
            });

        debug!(
            "channel balance of {} after subtracting unrealized balance: {}",
            channel.get_id(),
            unrealized_balance.unwrap_or(Balance::zero(BalanceType::HOPR))
        );

        // ensure sender has enough funds
        if unrealized_balance.is_none() || ticket.amount.gt(&unrealized_balance.unwrap()) {
            return Err(OutOfFunds(channel.get_id().to_string()));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::errors::PacketError;
    use crate::validation::validate_unacknowledged_ticket;
    use async_trait::async_trait;
    use core_crypto::{
        iterated_hash::IteratedHash,
        types::{HalfKeyChallenge, Hash, PublicKey},
    };
    use core_ethereum_db::traits::HoprCoreEthereumDbActions;
    use core_types::acknowledgement::{AcknowledgedTicket, PendingAcknowledgement};
    use core_types::channels::ChannelStatus;
    use core_types::{
        account::AccountEntry,
        channels::{ChannelEntry, Ticket},
    };
    use hex_literal::hex;
    use lazy_static::lazy_static;
    use mockall::mock;
    use utils_types::primitives::{Address, Balance, BalanceType, Snapshot, U256};

    const SENDER_PRIV_KEY: [u8; 32] = hex!("492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775");
    const TARGET_PRIV_KEY: [u8; 32] = hex!("5bf21ea8cccd69aa784346b07bf79c84dac606e00eecaa68bf8c31aff397b1ca");

    lazy_static! {
        static ref SENDER_PUB: PublicKey = PublicKey::from_privkey(&SENDER_PRIV_KEY).unwrap();
        static ref TARGET_PUB: PublicKey = PublicKey::from_privkey(&TARGET_PRIV_KEY).unwrap();
        static ref TARGET_ADDR: Address = Address::new(&hex!("65e78d07acf7b654e5ae6777a93ebbf30f639356"));
    }

    mock! {
        pub Db { }
        #[async_trait(? Send)]
        impl HoprCoreEthereumDbActions for Db {
            async fn get_current_ticket_index(&self, channel_id: &Hash) -> core_ethereum_db::errors::Result<Option<U256>>;
            async fn set_current_ticket_index(&mut self, channel_id: &Hash, index: U256) -> core_ethereum_db::errors::Result<()>;
            async fn get_tickets(&self, signer: &PublicKey) -> core_ethereum_db::errors::Result<Vec<Ticket>>;
            async fn mark_rejected(&mut self, ticket: &Ticket) -> core_ethereum_db::errors::Result<()>;
            async fn check_and_set_packet_tag(&mut self, tag: &[u8]) -> core_ethereum_db::errors::Result<bool>;
            async fn get_pending_acknowledgement(
                &self,
                half_key_challenge: &HalfKeyChallenge,
            ) -> core_ethereum_db::errors::Result<Option<PendingAcknowledgement>>;
            async fn store_pending_acknowledgment(
                &mut self,
                half_key_challenge: HalfKeyChallenge,
                pending_acknowledgment: PendingAcknowledgement,
            ) -> core_ethereum_db::errors::Result<()>;
            async fn replace_unack_with_ack(
                &mut self,
                half_key_challenge: &HalfKeyChallenge,
                ack_ticket: AcknowledgedTicket,
            ) -> core_ethereum_db::errors::Result<()>;
            async fn get_acknowledged_tickets(&self, filter: Option<ChannelEntry>) -> core_ethereum_db::errors::Result<Vec<AcknowledgedTicket>>;
            async fn mark_pending(&mut self, ticket: &Ticket) -> core_ethereum_db::errors::Result<()>;
            async fn get_pending_balance_to(&self, counterparty: &Address) -> core_ethereum_db::errors::Result<Balance>;
            async fn get_channel_to(&self, dest: &PublicKey) -> core_ethereum_db::errors::Result<Option<ChannelEntry>>;
            async fn get_channel_from(&self, src: &PublicKey) -> core_ethereum_db::errors::Result<Option<ChannelEntry>>;
            async fn update_channel_and_snapshot(
                &mut self,
                channel_id: &Hash,
                channel: &ChannelEntry,
                snapshot: &Snapshot,
            ) -> core_ethereum_db::errors::Result<()>;
            async fn delete_acknowledged_tickets_from(&mut self, source: ChannelEntry) -> core_ethereum_db::errors::Result<()>;
            async fn delete_acknowledged_ticket(&mut self, ticket: &AcknowledgedTicket) -> core_ethereum_db::errors::Result<()>;
            async fn store_hash_intermediaries(&mut self, channel: &Hash, intermediates: &IteratedHash) -> core_ethereum_db::errors::Result<()>;
            async fn get_commitment(&self, channel: &Hash, iteration: usize) -> core_ethereum_db::errors::Result<Option<Hash>>;
            async fn get_current_commitment(&self, channel: &Hash) -> core_ethereum_db::errors::Result<Option<Hash>>;
            async fn set_current_commitment(&mut self, channel: &Hash, commitment: &Hash) -> core_ethereum_db::errors::Result<()>;
            async fn get_latest_block_number(&self) -> core_ethereum_db::errors::Result<u32>;
            async fn update_latest_block_number(&mut self, number: u32) -> core_ethereum_db::errors::Result<()>;
            async fn get_latest_confirmed_snapshot(&self) -> core_ethereum_db::errors::Result<Option<Snapshot>>;
            async fn get_channel(&self, channel: &Hash) -> core_ethereum_db::errors::Result<Option<ChannelEntry>>;
            async fn get_channels(&self) -> core_ethereum_db::errors::Result<Vec<ChannelEntry>>;
            async fn get_channels_open(&self) -> core_ethereum_db::errors::Result<Vec<ChannelEntry>>;
            async fn get_account(&self, address: &Address) -> core_ethereum_db::errors::Result<Option<AccountEntry>>;
            async fn update_account_and_snapshot(&mut self, account: &AccountEntry, snapshot: &Snapshot) -> core_ethereum_db::errors::Result<()>;
            async fn get_accounts(&self) -> core_ethereum_db::errors::Result<Vec<AccountEntry>>;
            async fn get_redeemed_tickets_value(&self) -> core_ethereum_db::errors::Result<Balance>;
            async fn get_redeemed_tickets_count(&self) -> core_ethereum_db::errors::Result<usize>;
            async fn get_neglected_tickets_count(&self) -> core_ethereum_db::errors::Result<usize>;
            async fn get_pending_tickets_count(&self) -> core_ethereum_db::errors::Result<usize>;
            async fn get_losing_tickets_count(&self) -> core_ethereum_db::errors::Result<usize>;
            async fn resolve_pending(&mut self, ticket: &Ticket, snapshot: &Snapshot) -> core_ethereum_db::errors::Result<()>;
            async fn mark_redeemed(&mut self, ticket: &AcknowledgedTicket) -> core_ethereum_db::errors::Result<()>;
            async fn mark_losing_acked_ticket(&mut self, ticket: &AcknowledgedTicket) -> core_ethereum_db::errors::Result<()>;
            async fn get_rejected_tickets_value(&self) -> core_ethereum_db::errors::Result<Balance>;
            async fn get_rejected_tickets_count(&self) -> core_ethereum_db::errors::Result<usize>;
            async fn get_channel_x(&self, src: &PublicKey, dest: &PublicKey) -> core_ethereum_db::errors::Result<Option<ChannelEntry>>;
            async fn get_channels_from(&self, address: Address) -> core_ethereum_db::errors::Result<Vec<ChannelEntry>>;
            async fn get_channels_to(&self, address: Address) -> core_ethereum_db::errors::Result<Vec<ChannelEntry>>;
            async fn get_hopr_balance(&self) -> core_ethereum_db::errors::Result<Balance>;
            async fn set_hopr_balance(&mut self, balance: &Balance) -> core_ethereum_db::errors::Result<()>;
            async fn add_hopr_balance(&mut self, balance: &Balance, snapshot: &Snapshot) -> core_ethereum_db::errors::Result<()>;
            async fn sub_hopr_balance(&mut self, balance: &Balance, snapshot: &Snapshot) -> core_ethereum_db::errors::Result<()>;
            async fn is_network_registry_enabled(&self) -> core_ethereum_db::errors::Result<bool>;
            async fn set_network_registry(&mut self, enabled: bool, snapshot: &Snapshot) -> core_ethereum_db::errors::Result<()>;
            async fn add_to_network_registry(
                &mut self,
                public_key: &PublicKey,
                account: &Address,
                snapshot: &Snapshot,
            ) -> core_ethereum_db::errors::Result<()>;
            async fn remove_from_network_registry(
                &mut self,
                public_key: &PublicKey,
                account: &Address,
                snapshot: &Snapshot,
            ) -> core_ethereum_db::errors::Result<()>;
            async fn get_account_from_network_registry(&self, public_key: &PublicKey) -> core_ethereum_db::errors::Result<Option<Address>>;
            async fn find_hopr_node_using_account_in_network_registry(&self, account: &Address) -> core_ethereum_db::errors::Result<Vec<PublicKey>>;
            async fn is_eligible(&self, account: &Address) -> core_ethereum_db::errors::Result<bool>;
            async fn set_eligible(&mut self, account: &Address, eligible: bool, snapshot: &Snapshot) -> core_ethereum_db::errors::Result<()>;
        }
    }

    fn create_valid_ticket() -> Ticket {
        Ticket::new(
            TARGET_ADDR.clone(),
            U256::one(),
            U256::one(),
            Balance::new(U256::one(), BalanceType::HOPR),
            U256::from_inverse_probability(U256::one()).unwrap(),
            U256::one(),
            &SENDER_PRIV_KEY,
        )
    }

    fn create_channel_entry() -> ChannelEntry {
        ChannelEntry::new(
            TARGET_PUB.clone(),
            TARGET_PUB.clone(),
            Balance::from_str("100", BalanceType::HOPR),
            Hash::create(&[&hex!("deadbeef")]),
            U256::one(),
            U256::zero(),
            ChannelStatus::Open,
            U256::one(),
            U256::zero(),
        )
    }

    #[async_std::test]
    async fn test_ticket_validation_should_pass_if_ticket_ok() {
        let mut db = MockDb::new();
        db.expect_get_tickets().returning(|_| Ok(Vec::<Ticket>::new()));

        let ticket = create_valid_ticket();
        let channel = create_channel_entry();

        let ret = validate_unacknowledged_ticket(
            &db,
            &ticket,
            &channel,
            &SENDER_PUB,
            Balance::from_str("1", BalanceType::HOPR),
            U256::one(),
            true,
        )
        .await;
        assert!(ret.is_ok());
    }

    #[async_std::test]
    async fn test_ticket_validation_should_fail_if_signer_not_sender() {
        let mut db = MockDb::new();
        db.expect_get_tickets().returning(|_| Ok(Vec::<Ticket>::new()));

        let ticket = create_valid_ticket();
        let channel = create_channel_entry();

        let ret = validate_unacknowledged_ticket(
            &db,
            &ticket,
            &channel,
            &TARGET_PUB,
            Balance::from_str("1", BalanceType::HOPR),
            U256::one(),
            true,
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
        let mut db = MockDb::new();
        db.expect_get_tickets().returning(|_| Ok(Vec::<Ticket>::new()));

        let ticket = create_valid_ticket();
        let channel = create_channel_entry();

        let ret = validate_unacknowledged_ticket(
            &db,
            &ticket,
            &channel,
            &SENDER_PUB,
            Balance::from_str("2", BalanceType::HOPR),
            U256::one(),
            true,
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
        let mut db = MockDb::new();
        db.expect_get_tickets().returning(|_| Ok(Vec::<Ticket>::new()));

        let mut ticket = create_valid_ticket();
        ticket.win_prob = U256::from_inverse_probability(2u32.into()).unwrap();
        ticket.sign(&SENDER_PRIV_KEY);

        let channel = create_channel_entry();

        let ret = validate_unacknowledged_ticket(
            &db,
            &ticket,
            &channel,
            &SENDER_PUB,
            Balance::from_str("1", BalanceType::HOPR),
            U256::one(),
            true,
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
        let mut db = MockDb::new();
        db.expect_get_tickets().returning(|_| Ok(Vec::<Ticket>::new()));

        let ticket = create_valid_ticket();
        let mut channel = create_channel_entry();
        channel.status = ChannelStatus::Closed;

        let ret = validate_unacknowledged_ticket(
            &db,
            &ticket,
            &channel,
            &SENDER_PUB,
            Balance::from_str("1", BalanceType::HOPR),
            U256::one(),
            true,
        )
        .await;

        assert!(ret.is_err());
        match ret.unwrap_err() {
            PacketError::TicketValidation(_) => {}
            _ => panic!("invalid error type"),
        }
    }

    #[async_std::test]
    async fn test_ticket_validation_should_fail_if_ticket_epoch_does_not_match() {
        let mut db = MockDb::new();
        db.expect_get_tickets().returning(|_| Ok(Vec::<Ticket>::new()));

        let ticket = create_valid_ticket();
        let mut channel = create_channel_entry();
        channel.ticket_epoch = 2u32.into();

        let ret = validate_unacknowledged_ticket(
            &db,
            &ticket,
            &channel,
            &SENDER_PUB,
            Balance::from_str("1", BalanceType::HOPR),
            U256::one(),
            true,
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
        let mut db = MockDb::new();
        db.expect_get_tickets().returning(|_| Ok(Vec::<Ticket>::new()));

        let mut ticket = create_valid_ticket();
        ticket.channel_epoch = 2u32.into();
        ticket.sign(&SENDER_PRIV_KEY);

        let channel = create_channel_entry();

        let ret = validate_unacknowledged_ticket(
            &db,
            &ticket,
            &channel,
            &SENDER_PUB,
            Balance::from_str("1", BalanceType::HOPR),
            U256::one(),
            true,
        )
        .await;

        assert!(ret.is_err());
        match ret.unwrap_err() {
            PacketError::TicketValidation(_) => {}
            _ => panic!("invalid error type"),
        }
    }

    #[async_std::test]
    async fn test_ticket_validation_ok_if_ticket_index_smaller_than_channel_index() {
        let mut db = MockDb::new();
        db.expect_get_tickets().returning(|_| Ok(Vec::<Ticket>::new()));

        let ticket = create_valid_ticket();
        let mut channel = create_channel_entry();
        channel.ticket_index = 2u32.into();

        let ret = validate_unacknowledged_ticket(
            &db,
            &ticket,
            &channel,
            &SENDER_PUB,
            Balance::from_str("1", BalanceType::HOPR),
            U256::one(),
            true,
        )
        .await;

        assert!(ret.is_ok());
    }

    #[async_std::test]
    async fn test_ticket_validation_ok_if_ticket_idx_smaller_than_channel_idx_unredeemed() {
        let mut db_ticket = create_valid_ticket();
        db_ticket.amount = Balance::from_str("100", BalanceType::HOPR);
        db_ticket.index = 2u32.into();
        db_ticket.sign(&SENDER_PRIV_KEY);

        let mut db = MockDb::new();
        db.expect_get_tickets().returning(move |_| Ok(vec![db_ticket.clone()]));

        let ticket = create_valid_ticket();
        let mut channel = create_channel_entry();
        channel.balance = Balance::from_str("200", BalanceType::HOPR);

        let ret = validate_unacknowledged_ticket(
            &db,
            &ticket,
            &channel,
            &SENDER_PUB,
            Balance::from_str("1", BalanceType::HOPR),
            U256::one(),
            true,
        )
        .await;

        assert!(ret.is_ok());
    }

    #[async_std::test]
    async fn test_ticket_validation_fail_if_does_not_have_funds() {
        let mut db = MockDb::new();
        db.expect_get_tickets().returning(|_| Ok(Vec::<Ticket>::new()));

        let ticket = create_valid_ticket();
        let mut channel = create_channel_entry();
        channel.balance = Balance::zero(BalanceType::HOPR);

        let ret = validate_unacknowledged_ticket(
            &db,
            &ticket,
            &channel,
            &SENDER_PUB,
            Balance::from_str("1", BalanceType::HOPR),
            U256::one(),
            true,
        )
        .await;

        assert!(ret.is_err());
        match ret.unwrap_err() {
            PacketError::OutOfFunds(_) => {}
            _ => panic!("invalid error type"),
        }
    }

    #[async_std::test]
    async fn test_ticket_validation_fail_if_does_not_have_funds_including_unredeemed() {
        let mut db_ticket = create_valid_ticket();
        db_ticket.amount = Balance::from_str("200", BalanceType::HOPR);
        db_ticket.sign(&SENDER_PRIV_KEY);

        let mut db = MockDb::new();
        db.expect_get_tickets().returning(move |_| Ok(vec![db_ticket.clone()]));

        let ticket = create_valid_ticket();
        let channel = create_channel_entry();

        let ret = validate_unacknowledged_ticket(
            &db,
            &ticket,
            &channel,
            &SENDER_PUB,
            Balance::from_str("1", BalanceType::HOPR),
            U256::one(),
            true,
        )
        .await;

        assert!(ret.is_err());
        match ret.unwrap_err() {
            PacketError::OutOfFunds(_) => {}
            _ => panic!("invalid error type"),
        }
    }

    #[async_std::test]
    async fn test_ticket_validation_ok_if_does_not_have_funds_including_unredeemed() {
        let mut db_ticket = create_valid_ticket();
        db_ticket.amount = Balance::from_str("200", BalanceType::HOPR);
        db_ticket.sign(&SENDER_PRIV_KEY);

        let mut db = MockDb::new();
        db.expect_get_tickets().returning(move |_| Ok(vec![db_ticket.clone()]));

        let ticket = create_valid_ticket();
        let channel = create_channel_entry();

        let ret = validate_unacknowledged_ticket(
            &db,
            &ticket,
            &channel,
            &SENDER_PUB,
            Balance::from_str("1", BalanceType::HOPR),
            U256::one(),
            false,
        )
        .await;

        assert!(ret.is_ok());
    }
}
