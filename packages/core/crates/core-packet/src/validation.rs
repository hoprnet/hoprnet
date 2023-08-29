use crate::errors::{
    PacketError::{OutOfFunds, TicketValidation},
    Result,
};
use core_crypto::types::Hash;
use core_ethereum_db::traits::HoprCoreEthereumDbActions;
use core_types::channels::{ChannelEntry, ChannelStatus, Ticket};
use utils_log::{debug, info};
use utils_types::primitives::{Address, Balance, BalanceType};

/// Performs validations of the given unacknowledged ticket and channel.
pub async fn validate_unacknowledged_ticket<T: HoprCoreEthereumDbActions>(
    db: &T,
    ticket: &Ticket,
    channel: &ChannelEntry,
    sender: &Address,
    min_ticket_amount: Balance,
    required_win_prob: f64,
    check_unrealized_balance: bool,
    domain_separator: &Hash,
) -> Result<()> {
    debug!(
        "validating unack ticket from {}",
        ticket.recover_signer(domain_separator)?
    );

    // ticket signer MUST be the sender
    ticket
        .verify(sender, &domain_separator)
        .map_err(|e| TicketValidation(format!("ticket signer does not match the sender: {e}")))?;

    // ticket amount MUST be greater or equal to minTicketAmount
    if !ticket.amount.gte(&min_ticket_amount) {
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

    if check_unrealized_balance {
        info!("checking unrealized balances for channel {}", channel.get_id());

        let unrealized_balance = db
            .get_tickets(Some(*sender))
            .await? // all tickets from sender
            .into_iter()
            .filter(|t| channel.channel_epoch.eq(&t.channel_epoch.into()))
            .fold(Some(channel.balance), |result, t| {
                result
                    .and_then(|b| b.value().value().checked_sub(*t.amount.value().value()))
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

    debug!("ticket validation done");
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::errors::PacketError;
    use crate::validation::validate_unacknowledged_ticket;
    use async_trait::async_trait;
    use core_crypto::random::random_bytes;
    use core_crypto::types::HalfKey;
    use core_crypto::types::OffchainPublicKey;
    use core_crypto::{
        keypairs::{ChainKeypair, Keypair},
        types::{HalfKeyChallenge, Hash},
    };
    use core_ethereum_db::db::CoreEthereumDb;
    use core_ethereum_db::traits::HoprCoreEthereumDbActions;
    use core_types::acknowledgement::{AcknowledgedTicket, PendingAcknowledgement, UnacknowledgedTicket};
    use core_types::channels::{f64_to_win_prob, ChannelStatus};
    use core_types::{
        account::AccountEntry,
        channels::{ChannelEntry, Ticket},
    };
    use hex_literal::hex;
    use lazy_static::lazy_static;
    use mockall::mock;
    use std::sync::{Arc, Mutex};
    use utils_db::db::DB;
    use utils_db::leveldb::rusty::RustyLevelDbShim;
    use utils_types::primitives::{
        Address, AuthorizationToken, Balance, BalanceType, EthereumChallenge, Snapshot, U256,
    };
    use utils_types::traits::BinarySerializable;

    const SENDER_PRIV_BYTES: [u8; 32] = hex!("492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775");
    const TARGET_PRIV_BYTES: [u8; 32] = hex!("5bf21ea8cccd69aa784346b07bf79c84dac606e00eecaa68bf8c31aff397b1ca");

    lazy_static! {
        static ref SENDER_PRIV_KEY: ChainKeypair = ChainKeypair::from_secret(&SENDER_PRIV_BYTES).unwrap();
        static ref TARGET_PRIV_KEY: ChainKeypair = ChainKeypair::from_secret(&TARGET_PRIV_BYTES).unwrap();
    }

    mock! {
        pub Db { }
        #[async_trait(? Send)]
        impl HoprCoreEthereumDbActions for Db {
            async fn get_current_ticket_index(&self, channel_id: &Hash) -> core_ethereum_db::errors::Result<Option<U256>>;
            async fn set_current_ticket_index(&mut self, channel_id: &Hash, index: U256) -> core_ethereum_db::errors::Result<()>;
            async fn get_tickets(&self, signer: Option<Address>) -> core_ethereum_db::errors::Result<Vec<Ticket>>;
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
            async fn get_unacknowledged_tickets(&self, filter: Option<ChannelEntry>) -> core_ethereum_db::errors::Result<Vec<UnacknowledgedTicket>>;
            async fn mark_pending(&mut self, counterparty: &Address, ticket: &Ticket) -> core_ethereum_db::errors::Result<()>;
            async fn get_pending_balance_to(&self, counterparty: &Address) -> core_ethereum_db::errors::Result<Balance>;
            async fn get_channel_to(&self, dest: &Address) -> core_ethereum_db::errors::Result<Option<ChannelEntry>>;
            async fn get_channel_from(&self, src: &Address) -> core_ethereum_db::errors::Result<Option<ChannelEntry>>;
            async fn update_channel_and_snapshot(
                &mut self,
                channel_id: &Hash,
                channel: &ChannelEntry,
                snapshot: &Snapshot,
            ) -> core_ethereum_db::errors::Result<()>;
            async fn get_packet_key(&self, chain_key: &Address) -> core_ethereum_db::errors::Result<Option<OffchainPublicKey>>;
            async fn get_chain_key(&self, packet_key: &OffchainPublicKey) -> core_ethereum_db::errors::Result<Option<Address>>;
            async fn link_chain_and_packet_keys(&mut self, chain_key: &Address, packet_key: &OffchainPublicKey, snapshot: &Snapshot) -> core_ethereum_db::errors::Result<()>;
            async fn delete_acknowledged_tickets_from(&mut self, source: ChannelEntry) -> core_ethereum_db::errors::Result<()>;
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
            async fn resolve_pending(&mut self, ticket: &Address, balance: &Balance, snapshot: &Snapshot) -> core_ethereum_db::errors::Result<()>;
            async fn mark_redeemed(&mut self, counterparty: &Address, ticket: &AcknowledgedTicket) -> core_ethereum_db::errors::Result<()>;
            async fn mark_losing_acked_ticket(&mut self, counterparty: &Address, ticket: &AcknowledgedTicket) -> core_ethereum_db::errors::Result<()>;
            async fn get_rejected_tickets_value(&self) -> core_ethereum_db::errors::Result<Balance>;
            async fn get_rejected_tickets_count(&self) -> core_ethereum_db::errors::Result<usize>;
            async fn get_channel_x(&self, src: &Address, dest: &Address) -> core_ethereum_db::errors::Result<Option<ChannelEntry>>;
            async fn get_channels_from(&self, address: &Address) -> core_ethereum_db::errors::Result<Vec<ChannelEntry>>;
            async fn get_channels_to(&self, address: &Address) -> core_ethereum_db::errors::Result<Vec<ChannelEntry>>;
            async fn get_public_node_accounts(&self) -> core_ethereum_db::errors::Result<Vec<AccountEntry>>;
            async fn get_hopr_balance(&self) -> core_ethereum_db::errors::Result<Balance>;
            async fn set_hopr_balance(&mut self, balance: &Balance) -> core_ethereum_db::errors::Result<()>;
            async fn get_ticket_price(&self) -> core_ethereum_db::errors::Result<Option<U256>>;
            async fn set_ticket_price(&mut self, ticket_price: &U256) -> core_ethereum_db::errors::Result<()>;
            async fn get_node_safe_registry_domain_separator(&self) -> core_ethereum_db::errors::Result<Option<Hash>>;
            async fn set_node_safe_registry_domain_separator(&mut self, node_safe_registry_domain_separator: &Hash, snapshot: &Snapshot) -> core_ethereum_db::errors::Result<()>;
            async fn get_channels_domain_separator(&self) -> core_ethereum_db::errors::Result<Option<Hash>>;
            async fn set_channels_domain_separator(&mut self, channels_domain_separator: &Hash, snapshot: &Snapshot) -> core_ethereum_db::errors::Result<()>;
            async fn get_channels_ledger_domain_separator(&self) -> core_ethereum_db::errors::Result<Option<Hash>>;
            async fn set_channels_ledger_domain_separator(&mut self, channels_ledger_domain_separator: &Hash, snapshot: &Snapshot) -> core_ethereum_db::errors::Result<()>;
            async fn get_staking_safe_address(&self) -> core_ethereum_db::errors::Result<Option<Address>>;
            async fn set_staking_safe_address(&mut self, safe_address: &Address) -> core_ethereum_db::errors::Result<()>;
            async fn get_staking_module_address(&self) -> core_ethereum_db::errors::Result<Option<Address>>;
            async fn set_staking_module_address(&mut self, module_address: &Address) -> core_ethereum_db::errors::Result<()>;
            async fn get_staking_safe_allowance(&self) -> core_ethereum_db::errors::Result<Balance>;
            async fn set_staking_safe_allowance(&mut self, allowance: &Balance, snapshot: &Snapshot) -> core_ethereum_db::errors::Result<()>;
            async fn add_hopr_balance(&mut self, balance: &Balance, snapshot: &Snapshot) -> core_ethereum_db::errors::Result<()>;
            async fn sub_hopr_balance(&mut self, balance: &Balance, snapshot: &Snapshot) -> core_ethereum_db::errors::Result<()>;
            async fn is_network_registry_enabled(&self) -> core_ethereum_db::errors::Result<bool>;
            async fn set_network_registry(&mut self, enabled: bool, snapshot: &Snapshot) -> core_ethereum_db::errors::Result<()>;
            async fn add_to_network_registry(
                &mut self,
                public_key: &Address,
                account: &Address,
                snapshot: &Snapshot,
            ) -> core_ethereum_db::errors::Result<()>;
            async fn remove_from_network_registry(
                &mut self,
                public_key: &Address,
                account: &Address,
                snapshot: &Snapshot,
            ) -> core_ethereum_db::errors::Result<()>;
            async fn is_eligible(&self, account: &Address) -> core_ethereum_db::errors::Result<bool>;
            async fn store_authorization(&mut self, token: AuthorizationToken) -> core_ethereum_db::errors::Result<()>;
            async fn retrieve_authorization(&self, id: String) -> core_ethereum_db::errors::Result<Option<AuthorizationToken>>;
            async fn delete_authorization(&mut self, id: String) -> core_ethereum_db::errors::Result<()>;
            async fn is_mfa_protected(&self) -> core_ethereum_db::errors::Result<Option<Address>>;
            async fn set_mfa_protected_and_update_snapshot(&mut self,maybe_mfa_address: Option<Address>,snapshot: &Snapshot) -> core_ethereum_db::errors::Result<()>;
            async fn is_allowed_to_access_network(&self, node: &Address) -> core_ethereum_db::errors::Result<bool>;
            async fn set_allowed_to_access_network(&mut self, node: &Address, allowed: bool, snapshot: &Snapshot) -> core_ethereum_db::errors::Result<()>;
            async fn get_from_network_registry(&self, stake_account: &Address) -> core_ethereum_db::errors::Result<Vec<Address>>;
            async fn set_eligible(&mut self, account: &Address, eligible: bool, snapshot: &Snapshot) -> core_ethereum_db::errors::Result<Vec<Address>>;
        }
    }

    fn create_valid_ticket() -> Ticket {
        Ticket::new(
            &TARGET_PRIV_KEY.public().to_address(),
            &Balance::new(1u64.into(), BalanceType::HOPR),
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
            Balance::new(100u64.into(), BalanceType::HOPR),
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
            &SENDER_PRIV_KEY.public().to_address(),
            Balance::new(1u64.into(), BalanceType::HOPR),
            1.0f64,
            true,
            &Hash::default(),
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
            &TARGET_PRIV_KEY.public().to_address(),
            Balance::new(1u64.into(), BalanceType::HOPR),
            1.0f64,
            true,
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
        let mut db = MockDb::new();
        db.expect_get_tickets().returning(|_| Ok(Vec::<Ticket>::new()));

        let ticket = create_valid_ticket();
        let channel = create_channel_entry();

        let ret = validate_unacknowledged_ticket(
            &db,
            &ticket,
            &channel,
            &SENDER_PRIV_KEY.public().to_address(),
            Balance::new(2u64.into(), BalanceType::HOPR),
            1.0f64,
            true,
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
        let mut db = MockDb::new();
        db.expect_get_tickets().returning(|_| Ok(Vec::<Ticket>::new()));

        let mut ticket = create_valid_ticket();
        ticket.encoded_win_prob = f64_to_win_prob(0.5f64).unwrap();
        ticket.sign(&SENDER_PRIV_KEY, &Hash::default());

        let channel = create_channel_entry();

        let ret = validate_unacknowledged_ticket(
            &db,
            &ticket,
            &channel,
            &SENDER_PRIV_KEY.public().to_address(),
            Balance::new(1u64.into(), BalanceType::HOPR),
            1.0f64,
            true,
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
        let mut db = MockDb::new();
        db.expect_get_tickets().returning(|_| Ok(Vec::<Ticket>::new()));

        let ticket = create_valid_ticket();
        let mut channel = create_channel_entry();
        channel.status = ChannelStatus::Closed;

        let ret = validate_unacknowledged_ticket(
            &db,
            &ticket,
            &channel,
            &SENDER_PRIV_KEY.public().to_address(),
            Balance::new(1u64.into(), BalanceType::HOPR),
            1.0f64,
            true,
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
        let mut db = MockDb::new();
        db.expect_get_tickets().returning(|_| Ok(Vec::<Ticket>::new()));

        let mut ticket = create_valid_ticket();
        ticket.channel_epoch = 2u32.into();
        ticket.sign(&SENDER_PRIV_KEY, &Hash::default());

        let channel = create_channel_entry();

        let ret = validate_unacknowledged_ticket(
            &db,
            &ticket,
            &channel,
            &SENDER_PRIV_KEY.public().to_address(),
            Balance::new(1u64.into(), BalanceType::HOPR),
            1.0f64,
            true,
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
            &SENDER_PRIV_KEY.public().to_address(),
            Balance::new(1u64.into(), BalanceType::HOPR),
            1.0f64,
            true,
            &Hash::default(),
        )
        .await;

        assert!(ret.is_ok());
    }

    #[async_std::test]
    async fn test_ticket_validation_ok_if_ticket_idx_smaller_than_channel_idx_unredeemed() {
        let mut db_ticket = create_valid_ticket();
        db_ticket.amount = Balance::from_str("100", BalanceType::HOPR);
        db_ticket.index = 2u32.into();
        db_ticket.sign(&SENDER_PRIV_KEY, &Hash::default());

        let mut db = MockDb::new();
        db.expect_get_tickets().returning(move |_| Ok(vec![db_ticket.clone()]));

        let ticket = create_valid_ticket();
        let mut channel = create_channel_entry();
        channel.balance = Balance::from_str("200", BalanceType::HOPR);

        let ret = validate_unacknowledged_ticket(
            &db,
            &ticket,
            &channel,
            &SENDER_PRIV_KEY.public().to_address(),
            Balance::new(1u64.into(), BalanceType::HOPR),
            1.0f64,
            true,
            &Hash::default(),
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
            &SENDER_PRIV_KEY.public().to_address(),
            Balance::new(1u64.into(), BalanceType::HOPR),
            1.0f64,
            true,
            &Hash::default(),
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
        db_ticket.amount = Balance::new(200u64.into(), BalanceType::HOPR);
        db_ticket.sign(&SENDER_PRIV_KEY, &Hash::default());

        let mut db = MockDb::new();
        db.expect_get_tickets().returning(move |_| Ok(vec![db_ticket.clone()]));

        let ticket = create_valid_ticket();
        let channel = create_channel_entry();

        let ret = validate_unacknowledged_ticket(
            &db,
            &ticket,
            &channel,
            &SENDER_PRIV_KEY.public().to_address(),
            Balance::new(1u64.into(), BalanceType::HOPR),
            1.0f64,
            true,
            &Hash::default(),
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
        db_ticket.amount = Balance::new(200u64.into(), BalanceType::HOPR);
        db_ticket.sign(&SENDER_PRIV_KEY, &Hash::default());

        let mut db = MockDb::new();
        db.expect_get_tickets().returning(move |_| Ok(vec![db_ticket.clone()]));

        let ticket = create_valid_ticket();
        let channel = create_channel_entry();

        let ret = validate_unacknowledged_ticket(
            &db,
            &ticket,
            &channel,
            &SENDER_PRIV_KEY.public().to_address(),
            Balance::new(1u64.into(), BalanceType::HOPR),
            1.0f64,
            false,
            &Hash::default(),
        )
        .await;

        assert!(ret.is_ok());
    }

    #[async_std::test]
    async fn test_ticket_workflow() {
        let level_db = Arc::new(Mutex::new(
            rusty_leveldb::DB::open("test", rusty_leveldb::in_memory()).unwrap(),
        ));
        let mut db = CoreEthereumDb::new(
            DB::new(RustyLevelDbShim::new(level_db)),
            SENDER_PRIV_KEY.public().to_address(),
        );

        let hkc = HalfKeyChallenge::new(&random_bytes::<{ HalfKeyChallenge::SIZE }>());
        let unack = UnacknowledgedTicket::new(
            create_valid_ticket(),
            HalfKey::new(&random_bytes::<{ HalfKey::SIZE }>()),
            SENDER_PRIV_KEY.public().to_address(),
        );

        db.store_pending_acknowledgment(hkc.clone(), PendingAcknowledgement::WaitingAsRelayer(unack))
            .await
            .unwrap();
        let num_tickets = db.get_tickets(None).await.unwrap();
        assert_eq!(1, num_tickets.len(), "db should find one ticket");

        let pending = db
            .get_pending_acknowledgement(&hkc)
            .await
            .unwrap()
            .expect("db should contain pending ack");
        match pending {
            PendingAcknowledgement::WaitingAsSender => panic!("must not be pending as sender"),
            PendingAcknowledgement::WaitingAsRelayer(ticket) => {
                let ack = ticket
                    .acknowledge(&HalfKey::default(), &TARGET_PRIV_KEY, &Hash::default())
                    .unwrap();
                db.replace_unack_with_ack(&hkc, ack).await.unwrap();

                let num_tickets = db.get_tickets(None).await.unwrap().len();
                let num_unack = db.get_unacknowledged_tickets(None).await.unwrap().len();
                let num_ack = db.get_acknowledged_tickets(None).await.unwrap().len();
                assert_eq!(1, num_tickets, "db should find one ticket");
                assert_eq!(0, num_unack, "db should not contain any unacknowledged tickets");
                assert_eq!(1, num_ack, "db should contain exactly one acknowledged ticket");
            }
        }
    }

    #[async_std::test]
    async fn test_db_should_store_ticket_index() {
        let level_db = Arc::new(Mutex::new(
            rusty_leveldb::DB::open("test", rusty_leveldb::in_memory()).unwrap(),
        ));
        let mut db = CoreEthereumDb::new(
            DB::new(RustyLevelDbShim::new(level_db)),
            SENDER_PRIV_KEY.public().to_address(),
        );

        let dummy_channel = Hash::new(&[0xffu8; Hash::SIZE]);
        let dummy_index = U256::one();

        db.set_current_ticket_index(&dummy_channel, dummy_index).await.unwrap();
        let idx = db
            .get_current_ticket_index(&dummy_channel)
            .await
            .unwrap()
            .expect("db must contain ticket index");

        assert_eq!(dummy_index, idx, "ticket index mismatch");
    }
}
