mod emulator;

pub use emulator::{FullStateEmulator, StaticState};

use std::collections::hash_map::Entry;

pub use blokli_client::{BlokliTestClient, BlokliTestState};

use hopr_api::chain::ChainInfo;
use hopr_chain_types::ParsedHoprChainAction;
use hopr_crypto_types::prelude::{Keypair, OffchainKeypair};
use hopr_crypto_types::types::Hash;
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::*;

/// Allows easily building the [`BlokliTestState`] using the HOPR native types.
#[derive(Clone, Default)]
pub struct BlokliTestStateBuilder(BlokliTestState);

const DEFAULT_ALLOWANCE: u128 = 10_000_000_000_000_u128;

impl From<BlokliTestState> for BlokliTestStateBuilder {
    fn from(state: BlokliTestState) -> Self {
        Self(state)
    }
}

impl BlokliTestStateBuilder {
    /// Sets the initial [`ChannelEntries`](ChannelEntry) in the state.
    ///
    /// The function will panic if any channels refer to accounts which were not previously
    /// present in the state or added via a [`BlokliTestStateBuilder::with_accounts`] call.
    #[must_use]
    pub fn with_channels<I: IntoIterator<Item = ChannelEntry>>(mut self, channels: I) -> Self {
        self.0.channels.extend(channels.into_iter().map(|channel| {
            (
                hex::encode(channel.get_id()),
                blokli_client::api::types::Channel {
                    balance: blokli_client::api::types::TokenValueString(channel.balance.to_string()),
                    closure_time: if let ChannelStatus::PendingToClose(time) = channel.status {
                        Some(blokli_client::api::types::DateTime(
                            hopr_api::chain::DateTime::from(time).to_rfc3339(),
                        ))
                    } else {
                        None
                    },
                    concrete_channel_id: hex::encode(channel.get_id()),
                    source: self
                        .0
                        .accounts
                        .values()
                        .find(|a| a.chain_key == hex::encode(channel.source))
                        .map(|a| a.keyid)
                        .expect(&format!("missing dst account {}", channel.source)),
                    epoch: channel.channel_epoch.as_u32() as i32,
                    destination: self
                        .0
                        .accounts
                        .values()
                        .find(|a| a.chain_key == hex::encode(channel.destination))
                        .map(|a| a.keyid)
                        .expect(&format!("missing dst account {}", channel.destination)),
                    status: match channel.status {
                        ChannelStatus::Closed => blokli_client::api::types::ChannelStatus::Closed,
                        ChannelStatus::Open => blokli_client::api::types::ChannelStatus::Open,
                        ChannelStatus::PendingToClose(_) => blokli_client::api::types::ChannelStatus::PendingToClose,
                    },
                    ticket_index: blokli_client::api::types::Uint64(channel.ticket_index.as_u64().to_string()),
                },
            )
        }));
        self
    }

    /// Sets the initial [`AccountEntries`](AccountEntry) in the state.
    #[must_use]
    pub fn with_accounts<I: IntoIterator<Item = (AccountEntry, HoprBalance, XDaiBalance)>>(mut self, accounts: I) -> Self {
        for (account, hopr_balance, native_balance) in accounts {
            match self.0.accounts.entry(account.key_id.into()) {
                Entry::Occupied(_) => panic!("duplicate key id for account {}", account.chain_addr),
                Entry::Vacant(v) => {
                    v.insert(blokli_client::api::types::Account {
                        chain_key: hex::encode(account.chain_addr),
                        keyid: u32::from(account.key_id) as i32,
                        multi_addresses: account.get_multiaddr().iter().map(|a| a.to_string()).collect(),
                        packet_key: hex::encode(account.public_key),
                        safe_address: account.safe_address.map(|a| hex::encode(a)),
                    });

                    self.0.token_balances.insert(
                        hex::encode(account.chain_addr.clone()),
                        blokli_client::api::types::HoprBalance {
                            __typename: "HoprBalance".to_string(),
                            balance: blokli_client::api::types::TokenValueString(HoprBalance::zero().to_string()),
                        },
                    );
                    self.0.native_balances.insert(
                        hex::encode(account.chain_addr.clone()),
                        blokli_client::api::types::NativeBalance {
                            __typename: "NativeBalance".to_string(),
                            balance: blokli_client::api::types::TokenValueString(native_balance.to_string()),
                        },
                    );
                    if let Some(addr) = account.safe_address.as_ref().map(|a| hex::encode(a)) {
                        self.0.token_balances.insert(
                            addr.clone(),
                            blokli_client::api::types::HoprBalance {
                                __typename: "HoprBalance".to_string(),
                                balance: blokli_client::api::types::TokenValueString(hopr_balance.to_string()),
                            },
                        );
                        self.0.native_balances.insert(
                            addr.clone(),
                            blokli_client::api::types::NativeBalance {
                                __typename: "NativeBalance".to_string(),
                                balance: blokli_client::api::types::TokenValueString(XDaiBalance::zero().to_string()),
                            },
                        );
                        self.0.safe_allowances.insert(
                            addr.clone(),
                            blokli_client::api::types::SafeHoprAllowance {
                                __typename: "SafeHoprAllowance".to_string(),
                                allowance: blokli_client::api::types::TokenValueString(
                                    HoprBalance::new_base(DEFAULT_ALLOWANCE).to_string(),
                                ),
                            },
                        );
                    }
                }
            }
        }
        self
    }

    /// Generates [`AccountEntries`](AccountEntry) for the given addresses.
    ///
    /// The off-chain keys and safe addresses are chosen deterministically using a
    /// pseudorandom function of each given address.
    #[must_use]
    pub fn with_generated_accounts(self, addresses: &[&Address], public: bool, native: XDaiBalance, token: HoprBalance) -> Self {
        let max_id = self.0.accounts.keys().max().copied().unwrap_or(0);
        self.with_accounts(addresses.iter().enumerate().map(|(index, &chain_addr)| {
            let pseudorandom_data = Hash::create(&[chain_addr.as_ref()]);
            let ok = OffchainKeypair::from_secret(pseudorandom_data.as_ref()).expect("offchain keypair creation cannot fail");
            let safe_addr = pseudorandom_data.hash();
            (AccountEntry {
                public_key: *ok.public(),
                chain_addr: *chain_addr,
                entry_type: if public {
                    AccountType::Announced(
                        format!("/ip4/1.2.3.4/udp/{}/p2p/{}", 10000 + index, ok.public().to_peerid_str())
                            .parse()
                            .unwrap(),
                    )
                } else {
                    AccountType::NotAnnounced
                },
                safe_address: Some(Address::new(&safe_addr.as_ref()[0..Address::SIZE])),
                key_id: KeyIdent::from(max_id + index as u32),
            }, token, native)
        }))
    }

    #[must_use]
    pub fn with_balances<C: Currency>(mut self, balances: impl IntoIterator<Item = (Address, Balance<C>)>) -> Self {
        if C::is::<XDai>() {
            self.0
                .native_balances
                .extend(balances.into_iter().map(|(addr, balance)| {
                    (
                        hex::encode(addr),
                        blokli_client::api::types::NativeBalance {
                            __typename: "NativeBalance".into(),
                            balance: blokli_client::api::types::TokenValueString(balance.to_string()),
                        },
                    )
                }))
        } else if C::is::<WxHOPR>() {
            self.0
                .token_balances
                .extend(balances.into_iter().map(|(addr, balance)| {
                    (
                        hex::encode(addr),
                        blokli_client::api::types::HoprBalance {
                            __typename: "HoprBalance".into(),
                            balance: blokli_client::api::types::TokenValueString(balance.to_string()),
                        },
                    )
                }))
        } else {
            panic!("unsupported currency");
        }

        self
    }

    /// Sets the initial Safe allowances for the given Safe addresses.
    #[must_use]
    pub fn with_safe_allowances<I: IntoIterator<Item = (Address, HoprBalance)>>(mut self, balances: I) -> Self {
        self.0
            .safe_allowances
            .extend(balances.into_iter().map(|(addr, allowance)| {
                (
                    hex::encode(addr),
                    blokli_client::api::types::SafeHoprAllowance {
                        __typename: "SafeAllowance".into(),
                        allowance: blokli_client::api::types::TokenValueString(allowance.to_string()),
                    },
                )
            }));
        self
    }

    /// Sets [`ChainInfo`] to the state. If not set, the default values are used.
    #[must_use]
    pub fn with_chain_info(mut self, info: ChainInfo) -> Self {
        self.0.chain_info.chain_id = info.chain_id as i32;
        self.0.chain_info.contract_addresses = blokli_client::api::types::ContractAddressMap(
            serde_json::to_string(&info.contract_addresses).expect("failed to serialize contract addresses"),
        );
        self
    }

    /// Sets the ticket price. If not set, the default value is used.
    #[must_use]
    pub fn with_ticket_price(mut self, price: HoprBalance) -> Self {
        self.0.chain_info.ticket_price = blokli_client::api::types::TokenValueString(price.to_string());
        self
    }

    /// Sets the minimum winning probability. If not set, the default value is used.
    #[must_use]
    pub fn with_minimum_win_prob(mut self, prob: WinningProbability) -> Self {
        self.0.chain_info.min_ticket_winning_probability = prob.as_f64();
        self
    }

    /// Sets the channel closure grace period. If not set, the default value is used.
    #[must_use]
    pub fn with_closure_grace_period(mut self, grace_period: std::time::Duration) -> Self {
        self.0.chain_info.channel_closure_grace_period =
            Some(blokli_client::api::types::Uint64(grace_period.as_secs().to_string()));
        self
    }

    /// Builds the state.
    #[must_use]
    pub fn build(self) -> BlokliTestState {
        self.0
    }

    /// Builds the state and returns a [`BlokliTestClient`] that cannot mutate the state.
    ///
    /// This is useful for simple static tests that only read data from the Chain.
    #[must_use]
    pub fn build_static_client(self) -> BlokliTestClient<StaticState> {
        BlokliTestClient::new(self.0, StaticState)
    }

    /// Builds the state and returns a [`BlokliTestClient`] that can also mutate the state.
    ///
    /// This is useful for more advanced dynamic tests which involve on-chain interactions.
    ///
    /// Because the underlying client performs all transactions via Safe, a `module_address` must be
    /// given.
    ///
    /// If the returned client is cloned, the state will be shared amongst them, and each
    /// client will see the changes made by others.
    #[must_use]
    pub fn build_dynamic_client(self, module_address: Address) -> BlokliTestClient<FullStateEmulator> {
        BlokliTestClient::new(self.0, FullStateEmulator(module_address, None))
    }

    /// Similar to [`BlokliTestStateBuilder::build_dynamic_client`], but also creates
    /// a stream of [`ParsedHoprChainAction`] to allow interception of all on-chain actions
    /// sent by the client.
    #[must_use]
    pub fn build_dynamic_client_with_tx_interceptor(
        self,
        module_address: Address,
    ) -> (
        BlokliTestClient<FullStateEmulator>,
        impl futures::Stream<Item = ParsedHoprChainAction>,
    ) {
        let (sender, receiver) = futures::channel::mpsc::unbounded();
        let client = BlokliTestClient::new(self.0, FullStateEmulator(module_address, Some(sender)));
        (client, receiver)
    }
}

