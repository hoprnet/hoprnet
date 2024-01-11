use crate::errors::{
    PacketError::{OutOfFunds, TicketValidation},
    Result,
};
use hopr_crypto::types::Hash;
use chain_db::traits::HoprCoreEthereumDbActions;
use core_types::channels::{ChannelEntry, ChannelStatus, Ticket};
use log::{debug, info};
use utils_types::primitives::{Address, Balance};

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
        .verify(sender, domain_separator)
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
        // ticket's channelEpoch MUST match the current DB channel's epoch
        match db.get_channel_epoch(&ticket.channel_id).await? {
            Some(epoch) => {
                if !epoch.eq(&ticket.channel_epoch.into()) {
                    return Err(TicketValidation(format!(
                        "ticket was created for a different channel iteration than is present in the DB {} != {} of channel {}",
                        ticket.channel_epoch,
                        epoch,
                        channel.get_id()
                    )));
                }

                info!("checking unrealized balances for channel {}", channel.get_id());

                let unrealized_balance = db.get_unrealized_balance(&ticket.channel_id).await?;

                debug!(
                    "channel balance of {} after subtracting unrealized balance: {unrealized_balance}",
                    channel.get_id()
                );

                // ensure sender has enough funds
                if ticket.amount.gt(&unrealized_balance) {
                    return Err(OutOfFunds(channel.get_id().to_string()));
                }
            }
            None => {
                return Err(TicketValidation(format!(
                    "no such channel {} available in the database",
                    ticket.channel_id,
                )));
            }
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
    use hopr_crypto::random::random_bytes;
    use hopr_crypto::types::HalfKey;
    use hopr_crypto::types::OffchainPublicKey;
    use hopr_crypto::{
        keypairs::{ChainKeypair, Keypair},
        types::{HalfKeyChallenge, Hash},
    };
    use chain_db::db::CoreEthereumDb;
    use chain_db::traits::HoprCoreEthereumDbActions;
    use core_types::acknowledgement::{AcknowledgedTicket, PendingAcknowledgement, UnacknowledgedTicket};
    use core_types::channels::{f64_to_win_prob, ChannelStatus};
    use core_types::{
        account::AccountEntry,
        channels::{ChannelEntry, Ticket},
    };
    use hex_literal::hex;
    use lazy_static::lazy_static;
    use mockall::mock;
    use utils_db::db::DB;
    use utils_db::CurrentDbShim;
    use utils_types::primitives::{Address, Balance, BalanceType, EthereumChallenge, Snapshot, U256};
    use utils_types::traits::BinarySerializable;

    const SENDER_PRIV_BYTES: [u8; 32] = hex!("492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775");
    const TARGET_PRIV_BYTES: [u8; 32] = hex!("5bf21ea8cccd69aa784346b07bf79c84dac606e00eecaa68bf8c31aff397b1ca");

    lazy_static! {
        static ref SENDER_PRIV_KEY: ChainKeypair = ChainKeypair::from_secret(&SENDER_PRIV_BYTES).unwrap();
        static ref TARGET_PRIV_KEY: ChainKeypair = ChainKeypair::from_secret(&TARGET_PRIV_BYTES).unwrap();
    }

    mock! {
        pub Db { }

        #[async_trait]
        impl HoprCoreEthereumDbActions for Db {
            async fn get_current_ticket_index(&self, channel_id: &Hash) -> chain_db::errors::Result<Option<U256>>;
            async fn set_current_ticket_index(&mut self, channel_id: &Hash, index: U256) -> chain_db::errors::Result<()>;
            async fn increase_current_ticket_index(&mut self, channel_id: &Hash) -> chain_db::errors::Result<()>;
            async fn ensure_current_ticket_index_gte(&mut self, channel_id: &Hash, index: U256) -> chain_db::errors::Result<()>;
            async fn get_tickets(&self, signer: Option<Address>) -> chain_db::errors::Result<Vec<Ticket>>;
            async fn get_unrealized_balance(&self, signer: &Hash) -> chain_db::errors::Result<Balance>;
            async fn get_channel_epoch(&self, channel: &Hash) -> chain_db::errors::Result<Option<U256>>;
            async fn cleanup_invalid_channel_tickets(&mut self, channel: &ChannelEntry) -> chain_db::errors::Result<()>;
            async fn mark_rejected(&mut self, ticket: &Ticket) -> chain_db::errors::Result<()>;
            async fn get_pending_acknowledgement(
                &self,
                half_key_challenge: &HalfKeyChallenge,
            ) -> chain_db::errors::Result<Option<PendingAcknowledgement>>;
            async fn store_pending_acknowledgment(
                &mut self,
                half_key_challenge: HalfKeyChallenge,
                pending_acknowledgment: PendingAcknowledgement,
            ) -> chain_db::errors::Result<()>;
            async fn replace_unack_with_ack(
                &mut self,
                half_key_challenge: &HalfKeyChallenge,
                ack_ticket: AcknowledgedTicket,
            ) -> chain_db::errors::Result<()>;
            async fn get_acknowledged_tickets_count(&self, filter: Option<ChannelEntry>) -> chain_db::errors::Result<usize>;
            async fn get_acknowledged_tickets(&self, filter: Option<ChannelEntry>) -> chain_db::errors::Result<Vec<AcknowledgedTicket>>;
            async fn get_acknowledged_tickets_range(
                &self,
                channel_id: &Hash,
                epoch: u32,
                index_start: u64,
                index_end: u64,
            ) -> chain_db::errors::Result<Vec<AcknowledgedTicket>>;
            async fn update_acknowledged_ticket(&mut self, ticket: &AcknowledgedTicket) -> chain_db::errors::Result<()>;
            async fn prepare_aggregatable_tickets(
                &mut self,
                channel_id: &Hash,
                epoch: u32,
                index_start: u64,
                index_end: u64,
            ) -> chain_db::errors::Result<Vec<AcknowledgedTicket>>;
            async fn replace_acked_tickets_by_aggregated_ticket(&mut self, aggregated_ticket: AcknowledgedTicket) -> chain_db::errors::Result<()>;
            async fn get_unacknowledged_tickets(&self, filter: Option<ChannelEntry>) -> chain_db::errors::Result<Vec<UnacknowledgedTicket>>;
            async fn get_channel_to(&self, dest: &Address) -> chain_db::errors::Result<Option<ChannelEntry>>;
            async fn get_channel_from(&self, src: &Address) -> chain_db::errors::Result<Option<ChannelEntry>>;
            async fn update_channel_and_snapshot(
                &mut self,
                channel_id: &Hash,
                channel: &ChannelEntry,
                snapshot: &Snapshot,
            ) -> chain_db::errors::Result<()>;
            async fn get_packet_key(&self, chain_key: &Address) -> chain_db::errors::Result<Option<OffchainPublicKey>>;
            async fn get_chain_key(&self, packet_key: &OffchainPublicKey) -> chain_db::errors::Result<Option<Address>>;
            async fn link_chain_and_packet_keys(&mut self, chain_key: &Address, packet_key: &OffchainPublicKey, snapshot: &Snapshot) -> chain_db::errors::Result<()>;
            async fn mark_acknowledged_tickets_neglected(&mut self, source: ChannelEntry) -> chain_db::errors::Result<()>;
            async fn get_latest_block_number(&self) -> chain_db::errors::Result<Option<u32>>;
            async fn update_latest_block_number(&mut self, number: u32) -> chain_db::errors::Result<()>;
            async fn get_latest_confirmed_snapshot(&self) -> chain_db::errors::Result<Option<Snapshot>>;
            async fn get_channel(&self, channel: &Hash) -> chain_db::errors::Result<Option<ChannelEntry>>;
            async fn get_channels(&self) -> chain_db::errors::Result<Vec<ChannelEntry>>;
            async fn get_channels_open(&self) -> chain_db::errors::Result<Vec<ChannelEntry>>;
            async fn get_account(&self, address: &Address) -> chain_db::errors::Result<Option<AccountEntry>>;
            async fn update_account_and_snapshot(&mut self, account: &AccountEntry, snapshot: &Snapshot) -> chain_db::errors::Result<()>;
            async fn get_accounts(&self) -> chain_db::errors::Result<Vec<AccountEntry>>;
            async fn get_redeemed_tickets_value(&self) -> chain_db::errors::Result<Balance>;
            async fn get_redeemed_tickets_count(&self) -> chain_db::errors::Result<usize>;
            async fn get_neglected_tickets_count(&self) -> chain_db::errors::Result<usize>;
            async fn get_neglected_tickets_value(&self) -> chain_db::errors::Result<Balance>;
            async fn get_losing_tickets_count(&self) -> chain_db::errors::Result<usize>;
            async fn mark_redeemed(&mut self, ticket: &AcknowledgedTicket) -> chain_db::errors::Result<()>;
            async fn mark_losing_acked_ticket(&mut self, ticket: &AcknowledgedTicket) -> chain_db::errors::Result<()>;
            async fn get_rejected_tickets_value(&self) -> chain_db::errors::Result<Balance>;
            async fn get_rejected_tickets_count(&self) -> chain_db::errors::Result<usize>;
            async fn get_channel_x(&self, src: &Address, dest: &Address) -> chain_db::errors::Result<Option<ChannelEntry>>;
            async fn get_channels_from(&self, address: &Address) -> chain_db::errors::Result<Vec<ChannelEntry>>;
            async fn get_outgoing_channels(&self) -> chain_db::errors::Result<Vec<ChannelEntry>>;
            async fn get_channels_to(&self, address: &Address) -> chain_db::errors::Result<Vec<ChannelEntry>>;
            async fn get_incoming_channels(&self) -> chain_db::errors::Result<Vec<ChannelEntry>>;
            async fn get_public_node_accounts(&self) -> chain_db::errors::Result<Vec<AccountEntry>>;
            async fn get_hopr_balance(&self) -> chain_db::errors::Result<Balance>;
            async fn set_hopr_balance(&mut self, balance: &Balance) -> chain_db::errors::Result<()>;
            async fn get_ticket_price(&self) -> chain_db::errors::Result<Option<U256>>;
            async fn set_ticket_price(&mut self, ticket_price: &U256) -> chain_db::errors::Result<()>;
            async fn get_node_safe_registry_domain_separator(&self) -> chain_db::errors::Result<Option<Hash>>;
            async fn set_node_safe_registry_domain_separator(&mut self, node_safe_registry_domain_separator: &Hash, snapshot: &Snapshot) -> chain_db::errors::Result<()>;
            async fn get_channels_domain_separator(&self) -> chain_db::errors::Result<Option<Hash>>;
            async fn set_channels_domain_separator(&mut self, channels_domain_separator: &Hash, snapshot: &Snapshot) -> chain_db::errors::Result<()>;
            async fn get_channels_ledger_domain_separator(&self) -> chain_db::errors::Result<Option<Hash>>;
            async fn set_channels_ledger_domain_separator(&mut self, channels_ledger_domain_separator: &Hash, snapshot: &Snapshot) -> chain_db::errors::Result<()>;
            async fn get_staking_safe_address(&self) -> chain_db::errors::Result<Option<Address>>;
            async fn set_staking_safe_address(&mut self, safe_address: &Address) -> chain_db::errors::Result<()>;
            async fn get_staking_module_address(&self) -> chain_db::errors::Result<Option<Address>>;
            async fn set_staking_module_address(&mut self, module_address: &Address) -> chain_db::errors::Result<()>;
            async fn get_staking_safe_allowance(&self) -> chain_db::errors::Result<Balance>;
            async fn set_staking_safe_allowance(&mut self, allowance: &Balance, snapshot: &Snapshot) -> chain_db::errors::Result<()>;
            async fn add_hopr_balance(&mut self, balance: &Balance, snapshot: &Snapshot) -> chain_db::errors::Result<()>;
            async fn sub_hopr_balance(&mut self, balance: &Balance, snapshot: &Snapshot) -> chain_db::errors::Result<()>;
            async fn is_network_registry_enabled(&self) -> chain_db::errors::Result<bool>;
            async fn set_network_registry(&mut self, enabled: bool, snapshot: &Snapshot) -> chain_db::errors::Result<()>;
            async fn is_eligible(&self, account: &Address) -> chain_db::errors::Result<bool>;
            async fn is_mfa_protected(&self) -> chain_db::errors::Result<Option<Address>>;
            async fn set_mfa_protected_and_update_snapshot(&mut self,maybe_mfa_address: Option<Address>,snapshot: &Snapshot) -> chain_db::errors::Result<()>;
            async fn is_allowed_to_access_network(&self, node: &Address) -> chain_db::errors::Result<bool>;
            async fn set_allowed_to_access_network(&mut self, node: &Address, allowed: bool, snapshot: &Snapshot) -> chain_db::errors::Result<()>;
            async fn get_from_network_registry(&self, stake_account: &Address) -> chain_db::errors::Result<Vec<Address>>;
            async fn set_eligible(&mut self, account: &Address, eligible: bool, snapshot: &Snapshot) -> chain_db::errors::Result<Vec<Address>>;
        }
    }

    impl Clone for MockDb {
        fn clone(&self) -> Self {
            MockDb::new()
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

        let ticket = create_valid_ticket();
        let channel = create_channel_entry();

        let more_than_ticket_balance = ticket.amount.add(&Balance::new(U256::from(500u128), BalanceType::HOPR));
        let channel_epoch = U256::from(ticket.channel_epoch);
        db.expect_get_channel_epoch()
            .returning(move |_| Ok(Some(channel_epoch)));
        db.expect_get_unrealized_balance()
            .returning(move |_| Ok(more_than_ticket_balance));

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
    async fn test_ticket_validation_fail_if_does_not_have_funds() {
        let mut db = MockDb::new();

        let ticket = create_valid_ticket();
        let mut channel = create_channel_entry();
        channel.balance = Balance::zero(BalanceType::HOPR);
        channel.channel_epoch = U256::from(ticket.channel_epoch);

        let channel_epoch = U256::from(ticket.channel_epoch);
        db.expect_get_channel_epoch()
            .returning(move |_| Ok(Some(channel_epoch)));
        db.expect_get_unrealized_balance()
            .returning(move |_| Ok(Balance::zero(BalanceType::HOPR)));

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
        // assert_eq!(ret.unwrap_err().to_string(), "");
        match ret.unwrap_err() {
            PacketError::OutOfFunds(_) => {}
            _ => panic!("invalid error type"),
        }
    }

    #[async_std::test]
    async fn test_ticket_workflow() {
        let mut db = CoreEthereumDb::new(
            DB::new(CurrentDbShim::new_in_memory().await),
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
        let mut db = CoreEthereumDb::new(
            DB::new(CurrentDbShim::new_in_memory().await),
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

    #[async_std::test]
    async fn test_db_should_increase_ticket_index() {
        let mut db = CoreEthereumDb::new(
            DB::new(CurrentDbShim::new_in_memory().await),
            SENDER_PRIV_KEY.public().to_address(),
        );

        let dummy_channel = Hash::new(&[0xffu8; Hash::SIZE]);

        // increase current ticket index of a non-existing channel, the result should be 1
        db.increase_current_ticket_index(&dummy_channel).await.unwrap();
        let idx = db
            .get_current_ticket_index(&dummy_channel)
            .await
            .unwrap()
            .expect("db must contain ticket index");
        assert_eq!(idx, U256::one(), "ticket index mismatch. Expecting 1");

        // increase current ticket index of an existing channel where previous value is 1, the result should be 2
        db.increase_current_ticket_index(&dummy_channel).await.unwrap();
        let idx = db
            .get_current_ticket_index(&dummy_channel)
            .await
            .unwrap()
            .expect("db must contain ticket index");
        assert_eq!(idx, U256::new("2"), "ticket index mismatch. Expecting 2");
    }

    #[async_std::test]
    async fn test_db_should_ensure_ticket_index_not_smaller_than_given_index() {
        let mut db = CoreEthereumDb::new(
            DB::new(CurrentDbShim::new_in_memory().await),
            SENDER_PRIV_KEY.public().to_address(),
        );

        let dummy_channel = Hash::new(&[0xffu8; Hash::SIZE]);
        let dummy_index = U256::new("123");

        // the ticket index should be equal or greater than the given dummy index
        db.ensure_current_ticket_index_gte(&dummy_channel, dummy_index)
            .await
            .unwrap();
        let idx = db
            .get_current_ticket_index(&dummy_channel)
            .await
            .unwrap()
            .expect("db must contain ticket index");
        assert_eq!(idx, dummy_index, "ticket index mismatch. Expecting 2");
    }
}
