use crate::errors::{CoreEthereumIndexerError, Result};
use bindings::{
    hopr_announcements::HoprAnnouncementsEvents,
    hopr_channels::HoprChannelsEvents,
    hopr_network_registry::HoprNetworkRegistryEvents,
    hopr_token::HoprTokenEvents,
    hopr_node_safe_registry::HoprNodeSafeRegistryEvents,
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

struct DeploymentExtract {
    channels: Address,
    token: Address,
    network_registry: Address,
    node_safe_registry: Address,
    announcements: Address,
}

struct Handlers {
    /// channels, announcements, network_registry, node_safe_registry, token: contract addresses
    /// whose event we process
    addresses: DeploymentExtract,
    /// monitor the Hopr Token events, ignore rest
    address_to_monitor: Address,
}

impl Handlers {
    pub(super) async fn on_announcement_event<T>(
        &self,
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

    pub(super) async fn on_channel_event<T>(&self, db: &mut T, log: &RawLog, snapshot: &Snapshot) -> Result<()>
    where
        T: HoprCoreEthereumDbActions,
    {
        Ok(match HoprChannelsEvents::decode_log(log)? {
            HoprChannelsEvents::ChannelBalanceDecreasedFilter(balance_decreased) => {
                let maybe_channel = db.get_channel(&balance_decreased.channel_id.try_into()?).await?;

                if let Some(mut channel) = maybe_channel {
                    channel.balance = channel
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

                if let Some(mut channel) = maybe_channel {
                    channel.balance = channel
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
                    channel.status = ChannelStatus::PendingToClose;
                    channel.closure_time = closure_initiated.closure_time.into();

                    db.update_channel_and_snapshot(&closure_initiated.channel_id.try_into()?, &channel, snapshot)
                        .await?;
                } else {
                    return Err(CoreEthereumIndexerError::ChannelDoesNotExist);
                }
            }
        })
    }

    pub(super) async fn on_token_event<T>(&self, db: &mut T, log: &RawLog, snapshot: &Snapshot) -> Result<()>
    where
        T: HoprCoreEthereumDbActions,
    {
        Ok(match HoprTokenEvents::decode_log(log)? {
            HoprTokenEvents::TransferFilter(transfered) => {
                let from: Address = transfered.from.0.try_into()?;
                let to: Address = transfered.to.0.try_into()?;

                let value: U256 = u256::from_be_bytes(transfered.value.into()).into();

                if to.ne(&self.address_to_monitor) && from.ne(&self.address_to_monitor) {
                    return Ok(());
                } else if to.eq(&self.address_to_monitor) {
                    db.add_hopr_balance(&Balance::new(value, BalanceType::HOPR), snapshot)
                        .await?;
                } else if from.eq(&self.address_to_monitor) {
                    db.sub_hopr_balance(&Balance::new(value, BalanceType::HOPR), snapshot)
                        .await?;
                }
            }
            _ => {
                // don't care. Not important to HOPR
            }
        })
    }

    pub(super) async fn on_network_registry_event<T>(&self, db: &mut T, log: &RawLog, snapshot: &Snapshot) -> Result<()>
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

    pub(super) async fn on_node_safe_registry_event<T>(&self, db: &mut T, log: &RawLog, snapshot: &Snapshot) -> Result<()>
    where
        T: HoprCoreEthereumDbActions,
    {
        Ok(match HoprNodeSafeRegistryEvents::decode_log(log)? {
            HoprNodeSafeRegistryEvents::DergisteredNodeSafeFilter(deregistered) => {
                db.remove_from_node_safe_registry(
                    &deregistered.node_address.0.try_into()?,
                    &deregistered.safe_address.0.try_into()?,
                    snapshot,
                )
                .await?;
            }
            HoprNodeSafeRegistryEvents::RegisteredNodeSafeFilter(registered) => {
                db.add_to_node_safe_registry(
                    &registered.node_address.0.try_into()?,
                    &registered.safe_address.0.try_into()?,
                    snapshot,
                )
                .await?;
            }
        })
    }

    pub async fn on_event<T>(
        &self,
        db: &mut T,
        address: &Address,
        block_number: u32,
        log: &RawLog,
        snapshot: &Snapshot,
    ) -> Result<()>
    where
        T: HoprCoreEthereumDbActions,
    {
        Ok(if address.eq(&self.addresses.announcements) {
            self.on_announcement_event(db, log, block_number, snapshot).await?;
        } else if address.eq(&self.addresses.channels) {
            self.on_channel_event(db, log, snapshot).await?;
        } else if address.eq(&self.addresses.network_registry) {
            self.on_network_registry_event(db, log, snapshot).await?;
        } else if address.eq(&self.addresses.node_safe_registry) {
            self.on_node_safe_registry_event(db, log, snapshot).await?;
        } else if address.eq(&self.addresses.token) {
            self.on_token_event(db, log, snapshot).await?;
        } else {
            return Err(CoreEthereumIndexerError::UnknownContract(address.clone()));
        })
    }
}

#[cfg(test)]
pub mod tests {
    use async_std;
    use bindings::{
        hopr_announcements::{AddressAnnouncementFilter, KeyBindingFilter, RevokeAnnouncementFilter},
        hopr_channels::{
            ChannelBalanceDecreasedFilter, ChannelBalanceIncreasedFilter, ChannelClosedFilter, ChannelOpenedFilter,
            OutgoingChannelClosureInitiatedFilter, TicketRedeemedFilter,
        },
        hopr_network_registry::{
            DeregisteredByManagerFilter, DeregisteredFilter, EligibilityUpdatedFilter,
            NetworkRegistryStatusUpdatedFilter, RegisteredByManagerFilter, RegisteredFilter,
        },
        hopr_node_safe_registry::{
            RegisteredNodeSafeFilter, DergisteredNodeSafeFilter,
        },
        hopr_token::TransferFilter,
    };
    use core_crypto::keypairs::{Keypair, OffchainKeypair};
    use core_ethereum_db::{db::CoreEthereumDb, traits::HoprCoreEthereumDbActions};
    use core_types::{
        account::{AccountEntry, AccountSignature, AccountType},
        channels::{generate_channel_id, ChannelEntry, ChannelStatus},
    };
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

    use super::Handlers;

    const SELF_PRIV_KEY: [u8; 32] = hex!("492057cf93e99b31d2a85bc5e98a9c3aa0021feec52c227cc8170e8f7d047775");
    const COUNTERPARTY_CHAIN_ADDRESS: [u8; 20] = hex!("f1a73ef496c45e260924a9279d2d9752ae378812");
    const SELF_CHAIN_ADDRESS: [u8; 20] = hex!("2e505638d318598334c0a2c2e887e0ff1a23ec6a");
    const STAKE_ADDRESS: [u8; 20] = hex!("4331eaa9542b6b034c43090d9ec1c2198758dbc3");
    const SAFE_ADDRESS: [u8; 20] = hex!("295026fd99ecbabef94940e229f6e022823f1774");

    const CHANNELS_ADDR: [u8; 20] = hex!("bab20aea98368220baa4e3b7f151273ee71df93b"); // just a dummy
    const TOKEN_ADDR: [u8; 20] = hex!("47d1677e018e79dcdd8a9c554466cb1556fa5007"); // just a dummy
    const NETWORK_REGISTRY_ADDR: [u8; 20] = hex!("a469d0225f884fb989cbad4fe289f6fd2fb98051"); // just a dummy
    const NODE_SAFE_REGISTRY_ADDR: [u8; 20] = hex!("0dcd1bf9a1b36ce34237eeafef220932846bcd82"); // just a dummy
    const ANNOUNCEMENTS_ADDR: [u8; 20] = hex!("11db4791bf45ef31a10ea4a1b5cb90f46cc72c7e"); // just a dummy

    fn create_mock_db() -> CoreEthereumDb<RustyLevelDbShim> {
        let opt = rusty_leveldb::in_memory();
        let db = rusty_leveldb::DB::open("test", opt).unwrap();

        CoreEthereumDb::new(
            DB::new(RustyLevelDbShim::new(Arc::new(Mutex::new(db)))),
            Address::random(),
        )
    }

    fn init_handlers() -> Handlers {
        Handlers {
            addresses: super::DeploymentExtract {
                channels: CHANNELS_ADDR.try_into().unwrap(),
                token: TOKEN_ADDR.try_into().unwrap(),
                network_registry: NETWORK_REGISTRY_ADDR.try_into().unwrap(),
                node_safe_registry: NODE_SAFE_REGISTRY_ADDR.try_into().unwrap(),
                announcements: ANNOUNCEMENTS_ADDR.try_into().unwrap(),
            },
            address_to_monitor: SELF_CHAIN_ADDRESS.try_into().unwrap(),
        }
    }

    #[async_std::test]
    async fn announce_keybinding() {
        let handlers = init_handlers();
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

        handlers
            .on_event(
                &mut db,
                &handlers.addresses.announcements,
                0u32,
                &keybinding_log,
                &Snapshot::default(),
            )
            .await
            .unwrap();

        assert_eq!(db.get_account(&chain_key).await.unwrap().unwrap(), account_entry);
    }

    #[async_std::test]
    async fn announce_address_announcement() {
        let handlers = init_handlers();
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

        handlers
            .on_event(
                &mut db,
                &handlers.addresses.announcements,
                0u32,
                &address_announcement_log,
                &Snapshot::default(),
            )
            .await
            .unwrap();

        assert_eq!(
            db.get_account(&chain_key).await.unwrap().unwrap(),
            announced_account_entry
        );
    }

    #[async_std::test]
    async fn announce_revoke() {
        let handlers = init_handlers();
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

        handlers
            .on_event(
                &mut db,
                &handlers.addresses.announcements,
                0u32,
                &revoke_announcement_log,
                &Snapshot::default(),
            )
            .await
            .unwrap();

        assert_eq!(db.get_account(&chain_key).await.unwrap().unwrap(), account_entry);
    }

    #[async_std::test]
    async fn on_token_transfer_to() {
        let handlers = init_handlers();
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

        handlers
            .on_event(
                &mut db,
                &handlers.addresses.token,
                0u32,
                &transferred_log,
                &Snapshot::default(),
            )
            .await
            .unwrap();

        assert_eq!(
            db.get_hopr_balance().await.unwrap(),
            Balance::new(value, BalanceType::HOPR)
        )
    }

    #[async_std::test]
    async fn on_token_transfer_from() {
        let handlers = init_handlers();
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

        handlers
            .on_event(
                &mut db,
                &handlers.addresses.token,
                0u32,
                &transferred_log,
                &Snapshot::default(),
            )
            .await;

        assert_eq!(
            db.get_hopr_balance().await.unwrap(),
            Balance::new(U256::zero(), BalanceType::HOPR)
        )
    }

    #[async_std::test]
    async fn on_network_registry_event_registered() {
        let handlers = init_handlers();
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

        handlers
            .on_event(
                &mut db,
                &handlers.addresses.network_registry,
                0u32,
                &registered_log,
                &Snapshot::default(),
            )
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
        let handlers = init_handlers();
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

        handlers
            .on_event(
                &mut db,
                &handlers.addresses.network_registry,
                0u32,
                &registered_log,
                &Snapshot::default(),
            )
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
        let handlers = init_handlers();
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

        handlers
            .on_event(
                &mut db,
                &handlers.addresses.network_registry,
                0u32,
                &registered_log,
                &Snapshot::default(),
            )
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
        let handlers = init_handlers();
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

        handlers
            .on_event(
                &mut db,
                &handlers.addresses.network_registry,
                0u32,
                &registered_log,
                &Snapshot::default(),
            )
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
        let handlers = init_handlers();
        let mut db = create_mock_db();

        let nr_enabled = RawLog {
            topics: vec![
                NetworkRegistryStatusUpdatedFilter::signature(),
                H256::from_low_u64_be(1),
            ],
            data: encode(&[]),
        };

        handlers
            .on_event(
                &mut db,
                &handlers.addresses.network_registry,
                0u32,
                &nr_enabled,
                &Snapshot::default(),
            )
            .await
            .unwrap();

        assert!(db.is_network_registry_enabled().await.unwrap());
    }

    #[async_std::test]
    async fn on_network_registry_event_disabled() {
        let handlers = init_handlers();
        let mut db = create_mock_db();

        db.set_network_registry(true, &Snapshot::default()).await.unwrap();

        let nr_disabled = RawLog {
            topics: vec![
                NetworkRegistryStatusUpdatedFilter::signature(),
                H256::from_low_u64_be(0),
            ],
            data: encode(&[]),
        };

        handlers
            .on_event(
                &mut db,
                &handlers.addresses.network_registry,
                0u32,
                &nr_disabled,
                &Snapshot::default(),
            )
            .await
            .unwrap();

        assert!(!db.is_network_registry_enabled().await.unwrap());
    }

    #[async_std::test]
    async fn on_network_registry_set_eligible() {
        let handlers = init_handlers();
        let mut db = create_mock_db();

        let stake_address = Address::from_bytes(&STAKE_ADDRESS).unwrap();

        let set_eligible = RawLog {
            topics: vec![
                EligibilityUpdatedFilter::signature(),
                H256::from_slice(&stake_address.to_bytes32()),
                H256::from_low_u64_be(1),
            ],
            data: encode(&[]),
        };

        handlers
            .on_event(
                &mut db,
                &handlers.addresses.network_registry,
                0u32,
                &set_eligible,
                &Snapshot::default(),
            )
            .await
            .unwrap();

        assert!(db.is_eligible(&stake_address).await.unwrap());
    }

    #[async_std::test]
    async fn on_network_registry_set_not_eligible() {
        let handlers = init_handlers();
        let mut db = create_mock_db();

        let stake_address = Address::from_bytes(&STAKE_ADDRESS).unwrap();

        db.set_eligible(&stake_address, false, &Snapshot::default())
            .await
            .unwrap();

        let set_eligible = RawLog {
            topics: vec![
                EligibilityUpdatedFilter::signature(),
                H256::from_slice(&stake_address.to_bytes32()),
                H256::from_low_u64_be(0),
            ],
            data: encode(&[]),
        };

        handlers
            .on_event(
                &mut db,
                &handlers.addresses.network_registry,
                0u32,
                &set_eligible,
                &Snapshot::default(),
            )
            .await
            .unwrap();

        assert!(!db.is_eligible(&stake_address).await.unwrap());
    }

    #[async_std::test]
    async fn on_node_safe_registry_event_registered() {
        let handlers = init_handlers();
        let mut db = create_mock_db();

        let chain_key = Address::from_bytes(&SELF_CHAIN_ADDRESS).unwrap();
        let safe_address = Address::from_bytes(&SAFE_ADDRESS).unwrap();

        let registered_log = RawLog {
            topics: vec![
                RegisteredNodeSafeFilter::signature(),
                H256::from_slice(&safe_address.to_bytes32()),
                H256::from_slice(&chain_key.to_bytes32()),
            ],
            data: encode(&[]),
        };

        handlers
            .on_event(
                &mut db,
                &handlers.addresses.node_safe_registry,
                0u32,
                &registered_log,
                &Snapshot::default(),
            )
            .await
            .unwrap();

        let stored = db
            .find_hopr_node_using_safe_in_node_safe_registry(&safe_address)
            .await
            .unwrap();

        assert_eq!(stored, vec![chain_key]);
    }

    #[async_std::test]
    async fn on_node_safe_registry_event_deregistered() {
        let handlers = init_handlers();
        let mut db = create_mock_db();

        let chain_key = Address::from_bytes(&SELF_CHAIN_ADDRESS).unwrap();
        let safe_address = Address::from_bytes(&SAFE_ADDRESS).unwrap();

        db.add_to_node_safe_registry(&chain_key, &safe_address, &Snapshot::default())
            .await
            .unwrap();

        let registered_log = RawLog {
            topics: vec![
                DergisteredNodeSafeFilter::signature(),
                H256::from_slice(&safe_address.to_bytes32()),
                H256::from_slice(&chain_key.to_bytes32()),
            ],
            data: encode(&[]),
        };

        handlers
            .on_event(
                &mut db,
                &handlers.addresses.node_safe_registry,
                0u32,
                &registered_log,
                &Snapshot::default(),
            )
            .await
            .unwrap();

        let stored = db
            .find_hopr_node_using_safe_in_node_safe_registry(&safe_address)
            .await
            .unwrap();

        assert_eq!(stored, vec![]);
    }

    #[async_std::test]
    async fn on_channel_event_balance_increased() {
        let handlers = init_handlers();
        let mut db = create_mock_db();

        let source = Address::from_bytes(&SELF_CHAIN_ADDRESS).unwrap();
        let destination = Address::from_bytes(&COUNTERPARTY_CHAIN_ADDRESS).unwrap();

        let channel_id = generate_channel_id(&source, &destination);

        db.update_channel_and_snapshot(
            &channel_id,
            &ChannelEntry::new(
                source,
                destination,
                Balance::new(U256::zero(), BalanceType::HOPR),
                U256::zero(),
                ChannelStatus::Open,
                U256::one(),
                U256::zero(),
            ),
            &Snapshot::default(),
        )
        .await
        .unwrap();

        let solidity_balance = U256::from((1u128 << 96) - 1);

        let balance_increased_log = RawLog {
            topics: vec![
                ChannelBalanceIncreasedFilter::signature(),
                H256::from_slice(&channel_id.to_bytes()),
            ],
            data: Vec::from(solidity_balance.to_bytes()),
        };

        handlers
            .on_event(
                &mut db,
                &handlers.addresses.channels,
                0u32,
                &balance_increased_log,
                &Snapshot::default(),
            )
            .await
            .unwrap();

        assert_eq!(
            *db.get_channel(&channel_id).await.unwrap().unwrap().balance.value(),
            solidity_balance
        );
    }

    #[async_std::test]
    async fn on_channel_event_balance_decreased() {
        let handlers = init_handlers();
        let mut db = create_mock_db();

        let source = Address::from_bytes(&SELF_CHAIN_ADDRESS).unwrap();
        let destination = Address::from_bytes(&COUNTERPARTY_CHAIN_ADDRESS).unwrap();

        let channel_id = generate_channel_id(&source, &destination);

        db.update_channel_and_snapshot(
            &channel_id,
            &ChannelEntry::new(
                source,
                destination,
                Balance::new(U256::from((1u128 << 96) - 1), BalanceType::HOPR),
                U256::zero(),
                ChannelStatus::Open,
                U256::one(),
                U256::zero(),
            ),
            &Snapshot::default(),
        )
        .await
        .unwrap();

        let solidity_balance = U256::from((1u128 << 96) - 1);

        let balance_increased_log = RawLog {
            topics: vec![
                ChannelBalanceDecreasedFilter::signature(),
                H256::from_slice(&channel_id.to_bytes()),
            ],
            data: Vec::from(solidity_balance.to_bytes()),
        };

        handlers
            .on_event(
                &mut db,
                &handlers.addresses.channels,
                0u32,
                &balance_increased_log,
                &Snapshot::default(),
            )
            .await
            .unwrap();

        assert_eq!(
            *db.get_channel(&channel_id).await.unwrap().unwrap().balance.value(),
            U256::zero()
        );
    }

    #[async_std::test]
    async fn on_channel_closed() {
        let handlers = init_handlers();
        let mut db = create_mock_db();

        let source = Address::from_bytes(&SELF_CHAIN_ADDRESS).unwrap();
        let destination = Address::from_bytes(&COUNTERPARTY_CHAIN_ADDRESS).unwrap();

        let channel_id = generate_channel_id(&source, &destination);

        db.update_channel_and_snapshot(
            &channel_id,
            &ChannelEntry::new(
                source,
                destination,
                Balance::new(U256::from((1u128 << 96) - 1), BalanceType::HOPR),
                U256::zero(),
                ChannelStatus::Open,
                U256::one(),
                U256::zero(),
            ),
            &Snapshot::default(),
        )
        .await
        .unwrap();

        let channel_closed_log = RawLog {
            topics: vec![
                ChannelClosedFilter::signature(),
                H256::from_slice(&channel_id.to_bytes()),
            ],
            data: encode(&[]),
        };

        handlers
            .on_event(
                &mut db,
                &handlers.addresses.channels,
                0u32,
                &channel_closed_log,
                &Snapshot::default(),
            )
            .await
            .unwrap();

        assert_eq!(
            db.get_channel(&channel_id).await.unwrap().unwrap().status,
            ChannelStatus::Closed
        );
    }

    #[async_std::test]
    async fn on_channel_opened() {
        let handlers = init_handlers();
        let mut db = create_mock_db();

        let source = Address::from_bytes(&SELF_CHAIN_ADDRESS).unwrap();
        let destination = Address::from_bytes(&COUNTERPARTY_CHAIN_ADDRESS).unwrap();

        let channel_id = generate_channel_id(&source, &destination);

        let channel_opened_log = RawLog {
            topics: vec![
                ChannelOpenedFilter::signature(),
                H256::from_slice(&source.to_bytes32()),
                H256::from_slice(&destination.to_bytes32()),
            ],
            data: encode(&[]),
        };

        handlers
            .on_event(
                &mut db,
                &handlers.addresses.channels,
                0u32,
                &channel_opened_log,
                &Snapshot::default(),
            )
            .await
            .unwrap();

        let channel = db.get_channel(&channel_id).await.unwrap().unwrap();

        assert_eq!(channel.status, ChannelStatus::Open);
    }

    #[async_std::test]
    async fn on_channel_ticket_redeemed() {
        let handlers = init_handlers();
        let mut db = create_mock_db();

        let source = Address::from_bytes(&SELF_CHAIN_ADDRESS).unwrap();
        let destination = Address::from_bytes(&COUNTERPARTY_CHAIN_ADDRESS).unwrap();

        let channel_id = generate_channel_id(&source, &destination);

        db.update_channel_and_snapshot(
            &channel_id,
            &ChannelEntry::new(
                source,
                destination,
                Balance::new(U256::from((1u128 << 96) - 1), BalanceType::HOPR),
                U256::zero(),
                ChannelStatus::Open,
                U256::one(),
                U256::zero(),
            ),
            &Snapshot::default(),
        )
        .await
        .unwrap();

        let ticket_index = U256::from((1u128 << 48) - 1);

        let ticket_redeemed_log = RawLog {
            topics: vec![
                TicketRedeemedFilter::signature(),
                H256::from_slice(&channel_id.to_bytes()),
            ],
            data: Vec::from(ticket_index.to_bytes()),
        };

        handlers
            .on_event(
                &mut db,
                &handlers.addresses.channels,
                0u32,
                &ticket_redeemed_log,
                &Snapshot::default(),
            )
            .await
            .unwrap();

        let channel = db.get_channel(&channel_id).await.unwrap().unwrap();

        assert_eq!(channel.ticket_index, ticket_index);
    }

    #[async_std::test]
    async fn on_channel_closure_initiated() {
        let handlers = init_handlers();
        let mut db = create_mock_db();

        let source = Address::from_bytes(&SELF_CHAIN_ADDRESS).unwrap();
        let destination = Address::from_bytes(&COUNTERPARTY_CHAIN_ADDRESS).unwrap();

        let channel_id = generate_channel_id(&source, &destination);

        db.update_channel_and_snapshot(
            &channel_id,
            &ChannelEntry::new(
                source,
                destination,
                Balance::new(U256::from((1u128 << 96) - 1), BalanceType::HOPR),
                U256::zero(),
                ChannelStatus::Open,
                U256::one(),
                U256::zero(),
            ),
            &Snapshot::default(),
        )
        .await
        .unwrap();

        let timestamp = U256::from((1u64 << 32) - 1);

        let closure_initiated_log = RawLog {
            topics: vec![
                OutgoingChannelClosureInitiatedFilter::signature(),
                H256::from_slice(&channel_id.to_bytes()),
            ],
            data: Vec::from(timestamp.to_bytes()),
        };

        handlers
            .on_event(
                &mut db,
                &handlers.addresses.channels,
                0u32,
                &closure_initiated_log,
                &Snapshot::default(),
            )
            .await
            .unwrap();

        let channel = db.get_channel(&channel_id).await.unwrap().unwrap();

        assert_eq!(channel.status, ChannelStatus::PendingToClose);
        assert_eq!(channel.closure_time, timestamp);
    }
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use core_ethereum_db::db::wasm::Database;
    use ethers::{core::abi::RawLog, types::H256};
    use hex::decode_to_slice;
    use js_sys::{Array, Uint8Array};
    use std::str::FromStr;
    use utils_misc::{ok_or_jserr, utils::wasm::JsResult};
    use utils_types::primitives::{Address, Snapshot};
    use wasm_bindgen::prelude::*;
    use wasm_bindgen_futures;

    #[wasm_bindgen]
    pub struct Handlers {
        w: super::Handlers,
    }

    #[wasm_bindgen]
    impl Handlers {
        #[wasm_bindgen]
        pub fn init(
            address_to_monitor: &str,
            channels: &str,
            token: &str,
            network_registry: &str,
            node_safe_registry: &str,
            announcements: &str,
        ) -> Handlers {
            Self {
                w: super::Handlers {
                    address_to_monitor: Address::from_str(address_to_monitor).unwrap(),
                    addresses: super::DeploymentExtract {
                        channels: Address::from_str(channels).unwrap(),
                        token: Address::from_str(token).unwrap(),
                        network_registry: Address::from_str(network_registry).unwrap(),
                        node_safe_registry: Address::from_str(node_safe_registry).unwrap(),
                        announcements: Address::from_str(announcements).unwrap(),
                    },
                },
            }
        }

        #[wasm_bindgen]
        pub async fn on_event(
            &mut self,
            db: &Database,
            address: &str,
            topics: Array,
            data: &str,
            block_number: &str,
            snapshot: &Snapshot,
        ) -> JsResult<()> {
            let contract_address = Address::from_str(address).unwrap();
            let u32_block_number = u32::from_str(block_number).unwrap();

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

            self.w
                .on_event(
                    &mut *g,
                    &contract_address,
                    u32_block_number,
                    &RawLog {
                        topics: decoded_topics,
                        data: decoded_data,
                    },
                    snapshot,
                )
                .await
                .map_err(|e| JsValue::from(e.to_string()))
        }
    }
}
