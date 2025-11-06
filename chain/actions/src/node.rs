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
use futures::StreamExt;
use hopr_chain_types::actions::Action;
use hopr_crypto_types::{keypairs::OffchainKeypair, prelude::Keypair};
use hopr_db_sql::prelude::HoprDbAccountOperations;
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
    /// Withdraws the specified `amount` of tokens or native coins to the given `recipient`.
    async fn withdraw<C: Currency + Send>(&self, recipient: Address, amount: Balance<C>) -> Result<PendingAction>;

    /// Announces node on-chain with key binding.
    /// The operation should also check if such an announcement has not been already made on-chain.
    async fn announce(&self, multiaddrs: Option<&[Multiaddr]>, offchain_key: &OffchainKeypair)
    -> Result<PendingAction>;

    /// Registers the safe address with the node
    async fn register_safe_by_node(&self, safe_address: Address) -> Result<PendingAction>;
}

#[async_trait]
impl<Db: Sync> NodeActions for ChainActions<Db> {
    #[tracing::instrument(level = "debug", skip(self))]
    async fn withdraw<C: Currency + Send>(&self, recipient: Address, amount: Balance<C>) -> Result<PendingAction> {
        if !amount.is_zero() {
            if C::is::<XDai>() {
                info!(%amount, %recipient, "initiating native withdrawal");
                self.tx_sender
                    .send(Action::WithdrawNative(recipient, amount.amount().into()))
                    .await
            } else if C::is::<WxHOPR>() {
                info!(%amount, %recipient, "initiating token withdrawal");
                self.tx_sender
                    .send(Action::Withdraw(recipient, amount.amount().into()))
                    .await
            } else {
                Err(InvalidArguments("invalid currency".into()))
            }
        } else {
            Err(InvalidArguments("cannot withdraw zero amount".into()))
        }
    }

    #[tracing::instrument(level = "debug", skip(self))]
    async fn announce(
        &self,
        multiaddrs: Option<&[Multiaddr]>,
        offchain_key: &OffchainKeypair,
    ) -> Result<PendingAction> {
        // TODO: allow announcing all addresses once that option is supported
        let announcement_data = AnnouncementData::new(
            multiaddrs.map(|ma| ma[0].clone()),
            KeyBinding::new(self.self_address(), offchain_key),
        )?;

        let count_announced = self
            .index_db
            .stream_accounts(true)
            .await?
            .filter(|account| {
                futures::future::ready(
                    &account.public_key == offchain_key.public()
                        && account.get_multiaddr().is_some_and(|ma| {
                            decapsulate_multiaddress(ma).eq(announcement_data.multiaddress().as_ref().unwrap())
                        }),
                )
            })
            .count()
            .await;

        // TODO: Read if the off-chain key and on-chain key has been bound already
        let is_key_bound = true;
        // TODO: Read keyBindingFee from contract instead of hardcoding
        // This should be the `keyBindingFee` value,
        // updated from event KeyBindingFeeUpdate(uint256 newFee, uint256 oldFee) in the Announcements,sol.
        // If the keys have been bounded, supply U256::ZERO,
        let key_binding_fee = if is_key_bound {
            U256::from(0_u128)
        } else {
            U256::from(10_000_000_000_000_000_u128)
        }; // 0.01 HOPR

        if count_announced == 0 {
            info!(%announcement_data, "initiating announcement");
            self.tx_sender
                .send(Action::Announce(announcement_data, key_binding_fee.into()))
                .await
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
    use hopr_db_node::HoprNodeDb;
    use hopr_db_sql::{
        HoprIndexerDb, accounts::HoprDbAccountOperations, info::HoprDbInfoOperations, prelude::DomainSeparator,
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

        let db = HoprIndexerDb::new_in_memory(ALICE_KP.clone()).await?;
        let node_db = HoprNodeDb::new_in_memory(ALICE_KP.clone()).await?;
        db.set_domain_separator(None, DomainSeparator::Channel, Default::default())
            .await?;

        let ma = announce_multiaddr.clone();
        let pubkey_clone = *ALICE_OFFCHAIN.public();
        let mut tx_exec = MockTransactionExecutor::new();
        tx_exec
            .expect_announce()
            .once()
            .withf(move |ad, key_binding_fee| {
                let kb = ad.key_binding.clone();
                ma.eq(ad.multiaddress().as_ref().unwrap())
                    && kb.packet_key == pubkey_clone
                    && kb.chain_key == *ALICE
                    && *key_binding_fee == U256::from(0_u128)
            })
            .returning(move |_, _| Ok(random_hash));

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

        let tx_queue = ActionQueue::new(node_db.clone(), indexer_action_tracker, tx_exec, Default::default());
        let tx_sender = tx_queue.new_sender();
        tokio::task::spawn(async move {
            tx_queue.start().await;
        });

        let actions = ChainActions::new(&ALICE_KP, db.clone(), node_db.clone(), tx_sender.clone());
        let tx_res = actions
            .announce(Some(&[announce_multiaddr]), &ALICE_OFFCHAIN)
            .await?
            .await?;

        assert_eq!(tx_res.tx_hash, random_hash, "tx hashes must be equal");
        assert!(
            matches!(tx_res.action, Action::Announce(_, _)),
            "must be announce action"
        );
        assert!(
            matches!(tx_res.event, Some(ChainEventType::Announcement { .. })),
            "must correspond to announcement chain event"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_announce_should_not_allow_reannouncing_with_same_multiaddress() -> anyhow::Result<()> {
        let announce_multiaddr = Multiaddr::from_str("/ip4/1.2.3.4/tcp/9009")?;

        let db = HoprIndexerDb::new_in_memory(ALICE_KP.clone()).await?;
        let node_db = HoprNodeDb::new_in_memory(ALICE_KP.clone()).await?;
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
            node_db.clone(),
            MockActionState::new(),
            MockTransactionExecutor::new(),
            Default::default(),
        );
        let tx_sender = tx_queue.new_sender();

        let actions = ChainActions::new(&ALICE_KP, db.clone(), node_db.clone(), tx_sender.clone());

        let res = actions.announce(Some(&[announce_multiaddr]), &ALICE_OFFCHAIN).await;
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

        let db = HoprIndexerDb::new_in_memory(ALICE_KP.clone()).await?;
        let node_db = HoprNodeDb::new_in_memory(ALICE_KP.clone()).await?;
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

        let tx_queue = ActionQueue::new(node_db.clone(), indexer_action_tracker, tx_exec, Default::default());
        let tx_sender = tx_queue.new_sender();
        tokio::task::spawn(async move {
            tx_queue.start().await;
        });

        let actions = ChainActions::new(&ALICE_KP, db.clone(), node_db.clone(), tx_sender.clone());

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
        let db = HoprIndexerDb::new_in_memory(ALICE_KP.clone()).await?;
        let node_db = HoprNodeDb::new_in_memory(ALICE_KP.clone()).await?;
        db.set_domain_separator(None, DomainSeparator::Channel, Default::default())
            .await?;

        let tx_queue = ActionQueue::new(
            node_db.clone(),
            MockActionState::new(),
            MockTransactionExecutor::new(),
            Default::default(),
        );
        let actions = ChainActions::new(&ALICE_KP, db.clone(), node_db.clone(), tx_queue.new_sender());

        assert!(
            matches!(
                actions
                    .withdraw::<WxHOPR>(*BOB, 0_u32.into())
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
