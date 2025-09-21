//! This module contains the [NodeActions] trait defining action which relate to HOPR node itself.
//!
//! An implementation of this trait is added to [ChainActions] which realizes the redemption
//! operations via [ActionQueue](crate::action_queue::ActionQueue).
//!
//! There are 3 functions that can be used to redeem tickets in the [NodeActions] trait:
//! - [withdraw](NodeActions::withdraw)
//! - [announce](NodeActions::announce)
//! - [register_safe_by_node](NodeActions::register_safe_by_node)
//!
//! All necessary pre-requisites are checked by the implementation before the respective [Action] is submitted
//! to the [ActionQueue](crate::action_queue::ActionQueue).

use async_trait::async_trait;
use hopr_chain_types::actions::Action;
use hopr_crypto_types::{keypairs::OffchainKeypair, prelude::Keypair};
use hopr_db_sql::accounts::HoprDbAccountOperations;
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::*;
use multiaddr::Multiaddr;
use tracing::info;

use crate::{
    ChainActions,
    action_queue::PendingAction,
    errors::{
        ChainActionsError::{AlreadyAnnounced, InvalidArguments},
        Result,
    },
};

/// Contains all on-chain calls specific to the HOPR node itself.
#[async_trait]
pub trait NodeActions {
    /// Withdraws the specified `amount` of tokens to the given `recipient`.
    async fn withdraw(&self, recipient: Address, amount: U256) -> Result<PendingAction>;

    /// Withdraws the specified `amount` of native coins to the given `recipient`.
    async fn withdraw_native(&self, recipient: Address, amount: U256) -> Result<PendingAction>;

    /// Announces node on-chain with key binding.
    /// The operation should also check if such an announcement has not been already made on-chain.
    async fn announce(&self, multiaddrs: &[Multiaddr], offchain_key: &OffchainKeypair) -> Result<PendingAction>;

    /// Registers the safe address with the node
    async fn register_safe_by_node(&self, safe_address: Address) -> Result<PendingAction>;
}

#[async_trait]
impl<Db> NodeActions for ChainActions<Db>
where
    Db: HoprDbAccountOperations + Clone + Send + Sync + std::fmt::Debug,
{
    #[tracing::instrument(level = "debug", skip(self))]
    async fn withdraw(&self, recipient: Address, amount: U256) -> Result<PendingAction> {
        if !amount.is_zero() {
            info!(%amount, %recipient, "initiating withdrawal");
            self.tx_sender.send(Action::Withdraw(recipient, amount.into())).await
        } else {
            Err(InvalidArguments("cannot withdraw zero amount".into()))
        }
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn withdraw_native(&self, recipient: Address, amount: U256) -> Result<PendingAction> {
        if !amount.is_zero() {
            info!(%amount, %recipient, "initiating native withdrawal");
            self.tx_sender
                .send(Action::WithdrawNative(recipient, amount.into()))
                .await
        } else {
            Err(InvalidArguments("cannot withdraw zero amount".into()))
        }
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn announce(&self, multiaddrs: &[Multiaddr], offchain_key: &OffchainKeypair) -> Result<PendingAction> {
        // TODO: allow announcing all addresses once that option is supported
        let announcement_data = AnnouncementData::new(
            multiaddrs[0].clone(),
            Some(KeyBinding::new(self.self_address(), offchain_key)),
        )?;

        if !self.db.get_accounts(None, true).await?.into_iter().any(|account| {
            account.public_key.eq(offchain_key.public())
                && account
                    .get_multiaddr()
                    .is_some_and(|ma| decapsulate_multiaddress(ma).eq(announcement_data.multiaddress()))
        }) {
            info!(%announcement_data, "initiating announcement");
            self.tx_sender.send(Action::Announce(announcement_data)).await
        } else {
            Err(AlreadyAnnounced)
        }
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn register_safe_by_node(&self, safe_address: Address) -> Result<PendingAction> {
        info!(%safe_address, "initiating safe address registration");
        self.tx_sender.send(Action::RegisterSafe(safe_address)).await
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use futures::FutureExt;
    use hex_literal::hex;
    use hopr_chain_types::{
        actions::Action,
        chain_events::{ChainEventType, SignificantChainEvent},
    };
    use hopr_crypto_random::random_bytes;
    use hopr_crypto_types::prelude::*;
    use hopr_db_sql::{
        accounts::HoprDbAccountOperations, api::info::DomainSeparator, db::HoprDb, info::HoprDbInfoOperations,
    };
    use hopr_internal_types::prelude::*;
    use hopr_primitive_types::prelude::*;
    use multiaddr::Multiaddr;

    use crate::{
        ChainActions,
        action_queue::{ActionQueue, MockTransactionExecutor},
        action_state::MockActionState,
        errors::ChainActionsError,
        node::NodeActions,
    };

    lazy_static::lazy_static! {
        static ref ALICE_KP: ChainKeypair = ChainKeypair::from_secret(&hex!("492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775")).expect("lazy static keypair should be constructible");
        static ref BOB_KP: ChainKeypair = ChainKeypair::from_secret(&hex!("48680484c6fc31bc881a0083e6e32b6dc789f9eaba0f8b981429fd346c697f8c")).expect("lazy static keypair should be constructible");
        static ref ALICE: Address = ALICE_KP.public().to_address();
        static ref BOB: Address = BOB_KP.public().to_address();
        static ref ALICE_OFFCHAIN: OffchainKeypair = OffchainKeypair::from_secret(&hex!("e0bf93e9c916104da00b1850adc4608bd7e9087bbd3f805451f4556aa6b3fd6e")).expect("lazy static keypair should be constructible");
    }

    #[tokio::test]
    async fn test_announce() -> anyhow::Result<()> {
        let random_hash = Hash::from(random_bytes::<{ Hash::SIZE }>());
        let announce_multiaddr = Multiaddr::from_str("/ip4/1.2.3.4/tcp/9009")?;

        let db = HoprDb::new_in_memory(ALICE_KP.clone()).await?;
        db.set_domain_separator(None, DomainSeparator::Channel, Default::default())
            .await?;

        let ma = announce_multiaddr.clone();
        let pubkey_clone = *ALICE_OFFCHAIN.public();
        let mut tx_exec = MockTransactionExecutor::new();
        tx_exec
            .expect_announce()
            .once()
            .withf(move |ad| {
                let kb = ad.key_binding.clone().expect("key binding must be present");
                ma.eq(ad.multiaddress()) && kb.packet_key == pubkey_clone && kb.chain_key == *ALICE
            })
            .returning(move |_| Ok(random_hash));

        let ma = announce_multiaddr.clone();
        let pk = *ALICE_OFFCHAIN.public();
        let mut indexer_action_tracker = MockActionState::new();
        indexer_action_tracker
            .expect_register_expectation()
            .once()
            .returning(move |_| {
                Ok(futures::future::ok(SignificantChainEvent {
                    tx_hash: random_hash,
                    event_type: ChainEventType::Announcement {
                        peer: pk.into(),
                        multiaddresses: vec![ma.clone()],
                        address: *ALICE,
                    },
                })
                .boxed())
            });

        let tx_queue = ActionQueue::new(db.clone(), indexer_action_tracker, tx_exec, Default::default());
        let tx_sender = tx_queue.new_sender();
        tokio::task::spawn(async move {
            tx_queue.start().await;
        });

        let actions = ChainActions::new(&ALICE_KP, db.clone(), tx_sender.clone());
        let tx_res = actions.announce(&[announce_multiaddr], &ALICE_OFFCHAIN).await?.await?;

        assert_eq!(tx_res.tx_hash, random_hash, "tx hashes must be equal");
        assert!(matches!(tx_res.action, Action::Announce(_)), "must be announce action");
        assert!(
            matches!(tx_res.event, Some(ChainEventType::Announcement { .. })),
            "must correspond to announcement chain event"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_announce_should_not_allow_reannouncing_with_same_multiaddress() -> anyhow::Result<()> {
        let announce_multiaddr = Multiaddr::from_str("/ip4/1.2.3.4/tcp/9009")?;

        let db = HoprDb::new_in_memory(ALICE_KP.clone()).await?;
        db.set_domain_separator(None, DomainSeparator::Channel, Default::default())
            .await?;

        db.insert_account(
            None,
            AccountEntry {
                public_key: *ALICE_OFFCHAIN.public(),
                chain_addr: *ALICE,
                entry_type: AccountType::Announced {
                    multiaddr: announce_multiaddr.clone(),
                    updated_block: 0,
                },
                published_at: 1,
            },
        )
        .await?;

        let tx_queue = ActionQueue::new(
            db.clone(),
            MockActionState::new(),
            MockTransactionExecutor::new(),
            Default::default(),
        );
        let tx_sender = tx_queue.new_sender();

        let actions = ChainActions::new(&ALICE_KP, db.clone(), tx_sender.clone());

        let res = actions.announce(&[announce_multiaddr], &ALICE_OFFCHAIN).await;
        assert!(
            matches!(res, Err(ChainActionsError::AlreadyAnnounced)),
            "must not be able to re-announce with same address"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_withdraw() -> anyhow::Result<()> {
        let stake = HoprBalance::from(10_u32);
        let random_hash = Hash::from(random_bytes::<{ Hash::SIZE }>());

        let db = HoprDb::new_in_memory(ALICE_KP.clone()).await?;
        db.set_domain_separator(None, DomainSeparator::Channel, Default::default())
            .await?;

        let mut tx_exec = MockTransactionExecutor::new();
        tx_exec
            .expect_withdraw()
            .times(1)
            .withf(move |dst, balance| *BOB == *dst && stake.eq(balance))
            .returning(move |_, _| Ok(random_hash));

        let mut indexer_action_tracker = MockActionState::new();
        indexer_action_tracker.expect_register_expectation().never();

        let tx_queue = ActionQueue::new(db.clone(), indexer_action_tracker, tx_exec, Default::default());
        let tx_sender = tx_queue.new_sender();
        tokio::task::spawn(async move {
            tx_queue.start().await;
        });

        let actions = ChainActions::new(&ALICE_KP, db.clone(), tx_sender.clone());

        let tx_res = actions.withdraw(*BOB, stake).await?.await?;

        assert_eq!(tx_res.tx_hash, random_hash, "tx hashes must be equal");
        assert!(
            matches!(tx_res.action, Action::Withdraw(_, _)),
            "must be withdraw action"
        );
        assert!(
            tx_res.event.is_none(),
            "withdraw tx must not connect to any chain event"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_should_not_withdraw_zero_amount() -> anyhow::Result<()> {
        let db = HoprDb::new_in_memory(ALICE_KP.clone()).await?;
        db.set_domain_separator(None, DomainSeparator::Channel, Default::default())
            .await?;

        let tx_queue = ActionQueue::new(
            db.clone(),
            MockActionState::new(),
            MockTransactionExecutor::new(),
            Default::default(),
        );
        let actions = ChainActions::new(&ALICE_KP, db.clone(), tx_queue.new_sender());

        assert!(
            matches!(
                actions
                    .withdraw(*BOB, HoprBalance::zero())
                    .await
                    .err()
                    .expect("must be error"),
                ChainActionsError::InvalidArguments(_)
            ),
            "should not allow to withdraw 0"
        );

        Ok(())
    }
}
