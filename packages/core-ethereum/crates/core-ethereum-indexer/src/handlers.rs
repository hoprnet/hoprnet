use crate::errors::{CoreEthereumIndexerError, Result};
use bindings::{
    hopr_announcements::HoprAnnouncementsEvents, hopr_channels::HoprChannelsEvents,
    hopr_network_registry::HoprNetworkRegistryEvents, hopr_token::HoprTokenEvents,
};
use core_crypto::types::OffchainSignature;
use core_ethereum_db::traits::HoprCoreEthereumDbActions;
use core_types::{
    account::{AccountEntry, AccountSignature, AccountType},
    channels::{generate_channel_id, ChannelEntry, ChannelStatus},
};
use ethers::{contract::EthLogDecode, core::abi::RawLog};
use ethnum::u256;
use utils_types::primitives::{Address, Balance, BalanceType, Snapshot, U256};

pub async fn on_announcement_event<T>(
    db: &mut T,
    log: &RawLog,
    block_number: u32,
    snapshot: &Snapshot,
) -> Result<AccountEntry>
where
    T: HoprCoreEthereumDbActions,
{
    Ok(match HoprAnnouncementsEvents::decode_log(log)? {
        HoprAnnouncementsEvents::AddressAnnouncementFilter(address_announcement) => {
            let maybe_account = db.get_account(&address_announcement.node.try_into()?).await?;

            if let Some(mut account) = maybe_account {
                let new_entry_type = AccountType::Announced {
                    multiaddr: address_announcement.base_multiaddr.try_into()?,
                    updated_block: block_number,
                };

                account.update(new_entry_type);
                db.update_account_and_snapshot(&account, snapshot).await?;

                account
            } else {
                return Err(CoreEthereumIndexerError::AnnounceBeforeKeyBinding);
            }
        }
        HoprAnnouncementsEvents::KeyBindingFilter(key_binding) => {
            if db.get_account(&key_binding.chain_key.try_into()?).await?.is_some() {
                return Err(CoreEthereumIndexerError::UnsupportedKeyRebinding);
            }

            let updated_account = AccountEntry::new(
                key_binding.ed_25519_pub_key.try_into()?,
                key_binding.chain_key.try_into()?,
                AccountType::NotAnnounced,
            );

            let sig = AccountSignature {
                signature: OffchainSignature::try_from((key_binding.ed_25519_sig_0, key_binding.ed_25519_sig_1))?,
                pub_key: key_binding.ed_25519_pub_key.try_into()?,
                chain_key: key_binding.chain_key.try_into()?,
            };

            if !sig.verify() {
                return Err(CoreEthereumIndexerError::AccountEntrySignatureVerification);
            }

            db.update_account_and_snapshot(&updated_account, snapshot).await?;

            updated_account
        }
        HoprAnnouncementsEvents::RevokeAnnouncementFilter(revocation) => {
            let maybe_account = db.get_account(&revocation.node.try_into()?).await?;

            if let Some(mut account) = maybe_account {
                account.update(AccountType::NotAnnounced);
                db.update_account_and_snapshot(&account, snapshot).await?;

                account
            } else {
                return Err(CoreEthereumIndexerError::RevocationBeforeKeyBinding);
            }
        }
    })
}

pub async fn on_channel_event<T>(db: &mut T, log: &RawLog, snapshot: &Snapshot) -> Result<()>
where
    T: HoprCoreEthereumDbActions,
{
    Ok(match HoprChannelsEvents::decode_log(log)? {
        HoprChannelsEvents::ChannelBalanceDecreasedFilter(balance_decreased) => {
            let maybe_channel = db.get_channel(&balance_decreased.channel_id.try_into()?).await?;

            if let Some(channel) = maybe_channel {
                channel
                    .balance
                    .sub(&Balance::new(balance_decreased.new_balance.into(), BalanceType::HOPR));

                db.update_channel_and_snapshot(&balance_decreased.channel_id.try_into()?, &channel, snapshot)
                    .await?;
            } else {
                return Err(CoreEthereumIndexerError::ChannelDoesNotExist);
            }
        }
        HoprChannelsEvents::ChannelBalanceIncreasedFilter(balance_increased) => {
            let maybe_channel = db.get_channel(&balance_increased.channel_id.try_into()?).await?;

            if let Some(channel) = maybe_channel {
                channel
                    .balance
                    .add(&Balance::new(balance_increased.new_balance.into(), BalanceType::HOPR));

                db.update_channel_and_snapshot(&balance_increased.channel_id.try_into()?, &channel, snapshot)
                    .await?;
            } else {
                return Err(CoreEthereumIndexerError::ChannelDoesNotExist);
            }
        }
        HoprChannelsEvents::ChannelClosedFilter(channel_closed) => {
            let maybe_channel = db.get_channel(&channel_closed.channel_id.try_into()?).await?;

            if let Some(mut channel) = maybe_channel {
                channel.status = ChannelStatus::Closed;

                db.update_channel_and_snapshot(&channel_closed.channel_id.try_into()?, &channel, snapshot)
                    .await?;
            } else {
                return Err(CoreEthereumIndexerError::ChannelDoesNotExist);
            }
        }
        HoprChannelsEvents::ChannelOpenedFilter(channel_opened) => {
            let source: Address = channel_opened.source.0.try_into()?;
            let destination: Address = channel_opened.destination.0.try_into()?;

            let channel_id = generate_channel_id(&source, &destination);

            let maybe_channel = db.get_channel(&channel_id).await?;

            if let Some(mut channel) = maybe_channel {
                channel.status = ChannelStatus::Open;

                db.update_channel_and_snapshot(&channel_id, &channel, snapshot).await?;
            } else {
                let new_channel = ChannelEntry::new(
                    source,
                    destination,
                    Balance::new(0u64.into(), utils_types::primitives::BalanceType::HOPR),
                    0u64.into(),
                    ChannelStatus::Open,
                    1u64.into(),
                    0u64.into(),
                );

                db.update_channel_and_snapshot(&channel_id, &new_channel, snapshot)
                    .await?;
            }
        }
        HoprChannelsEvents::TicketRedeemedFilter(ticket_redeemed) => {
            let maybe_channel = db.get_channel(&ticket_redeemed.channel_id.try_into()?).await?;

            if let Some(mut channel) = maybe_channel {
                channel.ticket_index = ticket_redeemed.new_ticket_index.into();

                db.update_channel_and_snapshot(&ticket_redeemed.channel_id.try_into()?, &channel, snapshot)
                    .await?;
            } else {
                return Err(CoreEthereumIndexerError::ChannelDoesNotExist);
            }
        }
        HoprChannelsEvents::OutgoingChannelClosureInitiatedFilter(closure_initiated) => {
            let maybe_channel = db.get_channel(&closure_initiated.channel_id.try_into()?).await?;

            if let Some(mut channel) = maybe_channel {
                channel.closure_time = closure_initiated.closure_time.into();

                db.update_channel_and_snapshot(&closure_initiated.channel_id.try_into()?, &channel, snapshot)
                    .await?;
            } else {
                return Err(CoreEthereumIndexerError::ChannelDoesNotExist);
            }
        }
    })
}

pub async fn on_token_event<T>(
    db: &mut T,
    log: &RawLog,
    address_to_monitor: &Address,
    snapshot: &Snapshot,
) -> Result<()>
where
    T: HoprCoreEthereumDbActions,
{
    Ok(match HoprTokenEvents::decode_log(log)? {
        HoprTokenEvents::TransferFilter(transfered) => {
            let from: Address = transfered.from.0.try_into()?;
            let to: Address = transfered.to.0.try_into()?;

            let value: U256 = u256::from_be_bytes(transfered.value.into()).into();

            if to.ne(address_to_monitor) && from.ne(address_to_monitor) {
                return Ok(());
            } else if to.eq(address_to_monitor) {
                db.add_hopr_balance(&Balance::new(value, BalanceType::HOPR), snapshot)
                    .await?;
            } else if from.eq(address_to_monitor) {
                db.sub_hopr_balance(&Balance::new(value, BalanceType::HOPR), snapshot)
                    .await?;
            }
        }
        _ => {
            // don't care. Not important to HOPR
        }
    })
}

pub async fn on_network_registry_event<T>(db: &mut T, log: &RawLog, snapshot: &Snapshot) -> Result<()>
where
    T: HoprCoreEthereumDbActions,
{
    Ok(match HoprNetworkRegistryEvents::decode_log(log)? {
        HoprNetworkRegistryEvents::DeregisteredByManagerFilter(deregistered) => {
            db.remove_from_network_registry(
                &deregistered.node_address.0.try_into()?,
                &deregistered.staking_account.0.try_into()?,
                snapshot,
            )
            .await?;
        }
        HoprNetworkRegistryEvents::DeregisteredFilter(deregistered) => {
            db.remove_from_network_registry(
                &deregistered.node_address.0.try_into()?,
                &deregistered.staking_account.0.try_into()?,
                snapshot,
            )
            .await?;
        }
        HoprNetworkRegistryEvents::EligibilityUpdatedFilter(eligibility_updated) => {
            db.set_eligible(
                &eligibility_updated.staking_account.0.try_into()?,
                eligibility_updated.eligibility,
                snapshot,
            )
            .await?;
        }
        HoprNetworkRegistryEvents::NetworkRegistryStatusUpdatedFilter(enabled) => {
            db.set_network_registry(enabled.is_enabled, snapshot).await?;
        }
        HoprNetworkRegistryEvents::RegisteredByManagerFilter(registered) => {
            db.add_to_network_registry(
                &registered.node_address.0.try_into()?,
                &registered.staking_account.0.try_into()?,
                snapshot,
            )
            .await?;
        }
        HoprNetworkRegistryEvents::RegisteredFilter(registered) => {
            db.add_to_network_registry(
                &registered.node_address.0.try_into()?,
                &registered.staking_account.0.try_into()?,
                snapshot,
            )
            .await?;
        }
        HoprNetworkRegistryEvents::RequirementUpdatedFilter(_) => {
            // TODO: implement this
        }
        _ => {
            // don't care. Not important to HOPR
        }
    })
}

#[cfg(test)]
pub mod tests {
    use async_std;
    use bindings::{
        hopr_announcements::{AddressAnnouncementFilter, KeyBindingFilter, RevokeAnnouncementFilter},
        hopr_network_registry::{
            DeregisteredByManagerFilter, DeregisteredFilter, NetworkRegistryStatusUpdatedFilter,
            RegisteredByManagerFilter, RegisteredFilter, EligibilityUpdatedFilter
        },
        hopr_token::TransferFilter,
    };
    use core_crypto::keypairs::{Keypair, OffchainKeypair};
    use core_ethereum_db::{db::CoreEthereumDb, traits::HoprCoreEthereumDbActions};
    use core_types::account::{AccountEntry, AccountSignature, AccountType};
    use ethers::{
        abi::{encode, Address as EthereumAddress, RawLog, Token},
        prelude::*,
        types::U256 as EthU256,
    };
    use hex_literal::hex;
    use multiaddr::Multiaddr;
    use primitive_types::H256;
    use std::str::FromStr;
    use std::sync::{Arc, Mutex};
    use utils_db::{db::DB, leveldb::rusty::RustyLevelDbShim};
    use utils_types::{
        primitives::{Address, Balance, BalanceType, Snapshot, U256},
        traits::BinarySerializable,
    };

    const SELF_PRIV_KEY: [u8; 32] = hex!("492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775");
    const SELF_CHAIN_ADDRESS: [u8; 20] = hex!("2e505638d318598334c0a2c2e887e0ff1a23ec6a");
    const STAKE_ADDRESS: [u8; 20] = hex!("4331eaa9542b6b034c43090d9ec1c2198758dbc3");

    fn create_mock_db() -> CoreEthereumDb<RustyLevelDbShim> {
        let opt = rusty_leveldb::in_memory();
        let db = rusty_leveldb::DB::open("test", opt).unwrap();

        CoreEthereumDb::new(
            DB::new(RustyLevelDbShim::new(Arc::new(Mutex::new(db)))),
            Address::random(),
        )
    }

    #[async_std::test]
    async fn announce_keybinding() {
        let mut db = create_mock_db();

        let keypair = OffchainKeypair::from_secret(&SELF_PRIV_KEY).unwrap();
        let chain_key = Address::from_bytes(&SELF_CHAIN_ADDRESS).unwrap();

        let sig = AccountSignature::new(&keypair, chain_key);

        let keybinding_log = RawLog {
            topics: vec![KeyBindingFilter::signature()],
            data: encode(&[
                Token::FixedBytes(Vec::from(sig.signature.to_bytes())),
                Token::FixedBytes(Vec::from(sig.pub_key.to_bytes())),
                Token::Address(EthereumAddress::from_slice(&chain_key.to_bytes())),
            ]),
        };

        let account_entry = AccountEntry::new(keypair.public().clone(), chain_key, AccountType::NotAnnounced);

        assert_eq!(
            super::on_announcement_event(&mut db, &keybinding_log, 0u32, &Snapshot::default())
                .await
                .unwrap(),
            account_entry
        );

        assert_eq!(db.get_account(&chain_key).await.unwrap().unwrap(), account_entry);
    }

    #[async_std::test]
    async fn announce_address_announcement() {
        let mut db = create_mock_db();

        let keypair = OffchainKeypair::from_secret(&SELF_PRIV_KEY).unwrap();
        let chain_key = Address::from_bytes(&SELF_CHAIN_ADDRESS).unwrap();

        // Assume that there is a keybinding
        let account_entry = AccountEntry::new(keypair.public().clone(), chain_key, AccountType::NotAnnounced);
        db.update_account_and_snapshot(&account_entry, &Snapshot::default())
            .await
            .unwrap();

        let test_multiaddr = Multiaddr::from_str("/ip4/1.2.3.4/tcp/56").unwrap();

        let address_announcement_log = RawLog {
            topics: vec![AddressAnnouncementFilter::signature()],
            data: encode(&[
                Token::Address(EthereumAddress::from_slice(&chain_key.to_bytes())),
                Token::String(test_multiaddr.to_string()),
            ]),
        };

        let announced_account_entry = AccountEntry::new(
            keypair.public().clone(),
            chain_key,
            AccountType::Announced {
                multiaddr: test_multiaddr,
                updated_block: 0,
            },
        );

        assert_eq!(
            super::on_announcement_event(&mut db, &address_announcement_log, 0u32, &Snapshot::default())
                .await
                .unwrap(),
            announced_account_entry
        );

        assert_eq!(
            db.get_account(&chain_key).await.unwrap().unwrap(),
            announced_account_entry
        );
    }

    #[async_std::test]
    async fn announce_revoke() {
        let mut db = create_mock_db();

        let keypair = OffchainKeypair::from_secret(&SELF_PRIV_KEY).unwrap();
        let chain_key = Address::from_bytes(&SELF_CHAIN_ADDRESS).unwrap();

        let test_multiaddr = Multiaddr::from_str("/ip4/1.2.3.4/tcp/56").unwrap();

        // Assume that there is a keybinding and an address announcement
        let announced_account_entry = AccountEntry::new(
            keypair.public().clone(),
            chain_key,
            AccountType::Announced {
                multiaddr: test_multiaddr,
                updated_block: 0,
            },
        );

        db.update_account_and_snapshot(&announced_account_entry, &Snapshot::default())
            .await
            .unwrap();

        let revoke_announcement_log = RawLog {
            topics: vec![RevokeAnnouncementFilter::signature()],
            data: encode(&[Token::Address(EthereumAddress::from_slice(&chain_key.to_bytes()))]),
        };

        let account_entry = AccountEntry::new(keypair.public().clone(), chain_key, AccountType::NotAnnounced);

        assert_eq!(
            super::on_announcement_event(&mut db, &revoke_announcement_log, 0u32, &Snapshot::default())
                .await
                .unwrap(),
            account_entry
        );

        assert_eq!(db.get_account(&chain_key).await.unwrap().unwrap(), account_entry);
    }

    #[async_std::test]
    async fn on_token_transfer_to() {
        let mut db = create_mock_db();

        let chain_key = Address::from_bytes(&SELF_CHAIN_ADDRESS).unwrap();

        let value = U256::max();

        let transferred_log = RawLog {
            topics: vec![
                TransferFilter::signature(),
                H256::from_slice(&Address::default().to_bytes32()),
                H256::from_slice(&chain_key.to_bytes32()),
            ],
            data: encode(&[Token::Uint(EthU256::from_big_endian(&value.to_bytes()))]),
        };

        assert!(
            super::on_token_event(&mut db, &transferred_log, &chain_key, &Snapshot::default())
                .await
                .is_ok()
        );

        assert_eq!(
            db.get_hopr_balance().await.unwrap(),
            Balance::new(value, BalanceType::HOPR)
        )
    }

    #[async_std::test]
    async fn on_token_transfer_from() {
        let mut db = create_mock_db();

        let chain_key = Address::from_bytes(&SELF_CHAIN_ADDRESS).unwrap();

        let value = U256::max();

        db.set_hopr_balance(&Balance::new(value, BalanceType::HOPR))
            .await
            .unwrap();

        let transferred_log = RawLog {
            topics: vec![
                TransferFilter::signature(),
                H256::from_slice(&chain_key.to_bytes32()),
                H256::from_slice(&Address::default().to_bytes32()),
            ],
            data: encode(&[Token::Uint(EthU256::from_big_endian(&value.to_bytes()))]),
        };

        assert!(
            super::on_token_event(&mut db, &transferred_log, &chain_key, &Snapshot::default())
                .await
                .is_ok()
        );

        assert_eq!(
            db.get_hopr_balance().await.unwrap(),
            Balance::new(U256::zero(), BalanceType::HOPR)
        )
    }

    #[async_std::test]
    async fn on_network_registry_event_registered() {
        let mut db = create_mock_db();

        let chain_key = Address::from_bytes(&SELF_CHAIN_ADDRESS).unwrap();
        let stake_address = Address::from_bytes(&STAKE_ADDRESS).unwrap();

        let registered_log = RawLog {
            topics: vec![
                RegisteredFilter::signature(),
                H256::from_slice(&stake_address.to_bytes32()),
                H256::from_slice(&chain_key.to_bytes32()),
            ],
            data: encode(&[]),
        };

        super::on_network_registry_event(&mut db, &registered_log, &Snapshot::default())
            .await
            .unwrap();

        let stored = db
            .find_hopr_node_using_account_in_network_registry(&stake_address)
            .await
            .unwrap();

        assert_eq!(stored, vec![chain_key]);
    }

    #[async_std::test]
    async fn on_network_registry_event_registered_by_manager() {
        let mut db = create_mock_db();

        let chain_key = Address::from_bytes(&SELF_CHAIN_ADDRESS).unwrap();
        let stake_address = Address::from_bytes(&STAKE_ADDRESS).unwrap();

        let registered_log = RawLog {
            topics: vec![
                RegisteredByManagerFilter::signature(),
                H256::from_slice(&stake_address.to_bytes32()),
                H256::from_slice(&chain_key.to_bytes32()),
            ],
            data: encode(&[]),
        };

        super::on_network_registry_event(&mut db, &registered_log, &Snapshot::default())
            .await
            .unwrap();

        let stored = db
            .find_hopr_node_using_account_in_network_registry(&stake_address)
            .await
            .unwrap();

        assert_eq!(stored, vec![chain_key]);
    }

    #[async_std::test]
    async fn on_network_registry_event_deregistered() {
        let mut db = create_mock_db();

        let chain_key = Address::from_bytes(&SELF_CHAIN_ADDRESS).unwrap();
        let stake_address = Address::from_bytes(&STAKE_ADDRESS).unwrap();

        db.add_to_network_registry(&chain_key, &stake_address, &Snapshot::default())
            .await
            .unwrap();

        let registered_log = RawLog {
            topics: vec![
                DeregisteredFilter::signature(),
                H256::from_slice(&stake_address.to_bytes32()),
                H256::from_slice(&chain_key.to_bytes32()),
            ],
            data: encode(&[]),
        };

        super::on_network_registry_event(&mut db, &registered_log, &Snapshot::default())
            .await
            .unwrap();

        let stored = db
            .find_hopr_node_using_account_in_network_registry(&stake_address)
            .await
            .unwrap();

        assert_eq!(stored, vec![]);
    }

    #[async_std::test]
    async fn on_network_registry_event_deregistered_by_manager() {
        let mut db = create_mock_db();

        let chain_key = Address::from_bytes(&SELF_CHAIN_ADDRESS).unwrap();
        let stake_address = Address::from_bytes(&STAKE_ADDRESS).unwrap();

        db.add_to_network_registry(&chain_key, &stake_address, &Snapshot::default())
            .await
            .unwrap();

        let registered_log = RawLog {
            topics: vec![
                DeregisteredByManagerFilter::signature(),
                H256::from_slice(&stake_address.to_bytes32()),
                H256::from_slice(&chain_key.to_bytes32()),
            ],
            data: encode(&[]),
        };

        super::on_network_registry_event(&mut db, &registered_log, &Snapshot::default())
            .await
            .unwrap();

        let stored = db
            .find_hopr_node_using_account_in_network_registry(&stake_address)
            .await
            .unwrap();

        assert_eq!(stored, vec![]);
    }

    #[async_std::test]
    async fn on_network_registry_event_enabled() {
        let mut db = create_mock_db();

        let nr_enabled = RawLog {
            topics: vec![NetworkRegistryStatusUpdatedFilter::signature(), H256::from_low_u64_be(1)],
            data: encode(&[]),
        };

        super::on_network_registry_event(&mut db, &nr_enabled, &Snapshot::default())
            .await
            .unwrap();

        assert!(db.is_network_registry_enabled().await.unwrap());
    }

    #[async_std::test]
    async fn on_network_registry_event_disabled() {
        let mut db = create_mock_db();

        db.set_network_registry(true, &Snapshot::default()).await.unwrap();

        let nr_disabled = RawLog {
            topics: vec![NetworkRegistryStatusUpdatedFilter::signature(), H256::from_low_u64_be(0)],
            data: encode(&[]),
        };

        super::on_network_registry_event(&mut db, &nr_disabled, &Snapshot::default())
            .await
            .unwrap();

        assert!(!db.is_network_registry_enabled().await.unwrap());
    }

    #[async_std::test]
    async fn on_network_registry_set_eligible() {
        let mut db = create_mock_db();

        let stake_address = Address::from_bytes(&STAKE_ADDRESS).unwrap();

        let set_eligible = RawLog {
            topics: vec![EligibilityUpdatedFilter::signature(), H256::from_slice(&stake_address.to_bytes32()), H256::from_low_u64_be(1)],
            data: encode(&[]),
        };

        super::on_network_registry_event(&mut db, &set_eligible, &Snapshot::default()).await.unwrap();

        assert!(db.is_eligible(&stake_address).await.unwrap());
    }

    #[async_std::test]
    async fn on_network_registry_set_not_eligible() {
        let mut db = create_mock_db();

        let stake_address = Address::from_bytes(&STAKE_ADDRESS).unwrap();

        db.set_eligible(&stake_address, false, &Snapshot::default()).await.unwrap();

        let set_eligible = RawLog {
            topics: vec![EligibilityUpdatedFilter::signature(), H256::from_slice(&stake_address.to_bytes32()), H256::from_low_u64_be(0)],
            data: encode(&[]),
        };

        super::on_network_registry_event(&mut db, &set_eligible, &Snapshot::default()).await.unwrap();

        assert!(!db.is_eligible(&stake_address).await.unwrap());
    }
}
#[cfg(feature = "wasm")]
pub mod wasm {
    use core_ethereum_db::db::wasm::Database;
    use core_types::account::AccountEntry;
    use ethers::core::abi::RawLog;
    use ethers::types::H256;
    use hex::decode_to_slice;
    use js_sys::{Array, Uint8Array};
    use utils_misc::{ok_or_jserr, utils::wasm::JsResult};
    use utils_types::primitives::Snapshot;
    use wasm_bindgen::prelude::*;
    use wasm_bindgen_futures;

    #[wasm_bindgen]
    pub async fn on_announcement_event(
        db: &Database,
        topics: Array,
        data: &str,
        block_number: &str,
        snapshot: &Snapshot,
    ) -> JsResult<AccountEntry> {
        let mut decoded_data = Vec::with_capacity(data.len() * 2);
        ok_or_jserr!(decode_to_slice(data, &mut decoded_data))?;

        let val = db.as_ref_counted();
        let mut g = val.write().await;

        let mut decoded_topics: Vec<H256> = vec![];

        for topic in topics.iter() {
            let mut decoded: [u8; 32] = [0u8; 32];

            ok_or_jserr!(decode_to_slice(
                Uint8Array::from(topic.to_owned()).to_vec(),
                &mut decoded
            ))?;

            decoded_topics.push(decoded.to_owned().into());
        }

        super::on_announcement_event(
            &mut *g,
            &RawLog {
                topics: decoded_topics,
                data: decoded_data,
            },
            u32::from_str_radix(block_number, 10).map_err(|e| JsValue::from(e.to_string()))?,
            snapshot,
        )
        .await
        .map_err(|e| JsValue::from(e.to_string()))
    }
}
