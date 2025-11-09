use std::{ops::Add, str::FromStr};
use std::collections::hash_map::Entry;
use blokli_client::BlokliTestStateMutator;
pub use blokli_client::{BlokliTestClient, BlokliTestState};
use hopr_api::chain::ChainInfo;
use hopr_chain_types::{ContractAddresses, ParsedHoprChainAction};
use hopr_crypto_types::prelude::{ChainKeypair, Keypair, OffchainKeypair};
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::*;

#[derive(Clone, Default)]
pub struct BlokliTestStateBuilder(BlokliTestState);

const DEFAULT_ALLOWANCE: u128 = 10_000_000_000_000_u128;
const DEFAULT_GRACE_PERIOD_SECONDS: u64 = 10;

impl BlokliTestStateBuilder {
    pub fn with_channels<I: IntoIterator<Item = ChannelEntry>>(mut self, channels: I) -> Self {
        self.0.channels.extend(channels.into_iter().map(|channel| {
            (channel.get_id().into(), blokli_client::api::types::Channel {
                balance: blokli_client::api::types::TokenValueString(channel.balance.to_string()),
                closure_time: if let ChannelStatus::PendingToClose(time) = channel.status {
                    Some(blokli_client::api::types::DateTime(
                        hopr_api::chain::DateTime::from(time).to_rfc3339(),
                    ))
                } else {
                    None
                },
                concrete_channel_id: hex::encode(channel.get_id()),
                destination: self
                    .0
                    .accounts
                    .values()
                    .find(|a| a.chain_key == hex::encode(channel.source))
                    .map(|a| a.keyid)
                    .expect(&format!("missing dst account {}", channel.source)),
                epoch: channel.channel_epoch.as_u32() as i32,
                source: self
                    .0
                    .accounts
                    .values()
                    .find(|a| a.chain_key == hex::encode(channel.destination))
                    .map(|a| a.keyid)
                    .expect(&format!("missing src account {}", channel.source)),
                status: match channel.status {
                    ChannelStatus::Closed => blokli_client::api::types::ChannelStatus::Closed,
                    ChannelStatus::Open => blokli_client::api::types::ChannelStatus::Open,
                    ChannelStatus::PendingToClose(_) => blokli_client::api::types::ChannelStatus::PendingToClose,
                },
                ticket_index: blokli_client::api::types::Uint64(channel.ticket_index.as_u64().to_string()),
            })
        }));
        self
    }

    pub fn with_accounts<I: IntoIterator<Item = AccountEntry>>(mut self, accounts: I) -> Self {
        for account in accounts {
            match self.0.accounts.entry(account.key_id.into()) {
                Entry::Occupied(_) => panic!("duplicate key id for account {}", account.chain_addr),
                Entry::Vacant(v) => {
                    v.insert(blokli_client::api::types::Account {
                        chain_key: hex::encode(account.chain_addr),
                        keyid: u32::from(account.key_id) as i32,
                        multi_addresses: account.get_multiaddr().iter().map(|a| a.to_string()).collect(),
                        packet_key: hex::encode(account.public_key),
                        safe_address: account.safe_address.map(|a| hex::encode(a)),
                        safe_transaction_count: Some(blokli_client::api::types::Uint64(DEFAULT_GRACE_PERIOD_SECONDS.to_string())),
                    });

                    self.0.token_balances.insert(hex::encode(account.chain_addr.clone()), blokli_client::api::types::HoprBalance {
                        __typename: "HoprBalance".to_string(),
                        balance: blokli_client::api::types::TokenValueString(HoprBalance::zero().to_string()),
                    });
                    self.0.native_balances.insert(hex::encode(account.chain_addr.clone()), blokli_client::api::types::NativeBalance {
                        __typename: "NativeBalance".to_string(),
                        balance: blokli_client::api::types::TokenValueString(XDaiBalance::zero().to_string()),
                    });
                    if let Some(addr) = account.safe_address.as_ref().map(|a| hex::encode(a)) {
                        self.0.token_balances.insert(addr.clone(), blokli_client::api::types::HoprBalance {
                            __typename: "HoprBalance".to_string(),
                            balance: blokli_client::api::types::TokenValueString(HoprBalance::zero().to_string()),
                        });
                        self.0.native_balances.insert(addr.clone(), blokli_client::api::types::NativeBalance {
                            __typename: "NativeBalance".to_string(),
                            balance: blokli_client::api::types::TokenValueString(XDaiBalance::zero().to_string()),
                        });
                        self.0.safe_allowances.insert(addr.clone(), blokli_client::api::types::SafeHoprAllowance {
                            __typename: "SafeHoprAllowance".to_string(),
                            allowance: blokli_client::api::types::TokenValueString(HoprBalance::new_base(DEFAULT_ALLOWANCE).to_string()),
                        });
                    }
                }
            }
        }
        self
    }

    pub fn with_random_accounts(self, addresses: &[&Address], public: bool) -> Self {
        let max_id = self.0.accounts.keys().max().copied().unwrap_or(0);
        self.with_accounts(addresses.iter().enumerate().map(|(index, &chain_addr)| {
            let ok = OffchainKeypair::random();
            AccountEntry {
                public_key: *ok.public(),
                chain_addr: *chain_addr,
                entry_type: if public {
                    AccountType::Announced(format!("/ip4/1.2.3.4/udp/{}/p2p/{}", 10000 + index, ok.public().to_peerid_str()).parse().unwrap())
                } else {
                    AccountType::NotAnnounced
                },
                safe_address: None,
                key_id: KeyIdent::from(max_id + index as u32),
            }
        }))
    }

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

    pub fn with_chain_info(mut self, info: ChainInfo) -> Self {
        self.0.chain_info.chain_id = info.chain_id as i32;
        self.0.chain_info.contract_addresses = blokli_client::api::types::ContractAddressMap(
            serde_json::to_string(&info.contract_addresses).expect("failed to serialize contract addresses"),
        );
        self
    }

    pub fn with_ticket_price(mut self, price: HoprBalance) -> Self {
        self.0.chain_info.ticket_price = blokli_client::api::types::TokenValueString(price.to_string());
        self
    }

    pub fn with_minimum_win_prob(mut self, prob: WinningProbability) -> Self {
        self.0.chain_info.min_ticket_winning_probability = prob.as_f64();
        self
    }

    pub fn with_closure_grace_period(mut self, grace_period: std::time::Duration) -> Self {
        self.0.chain_info.channel_closure_grace_period =
            Some(blokli_client::api::types::Uint64(grace_period.as_secs().to_string()));
        self
    }

    pub fn build(self) -> BlokliTestState {
        self.0
    }

    pub fn build_static_client(self) -> BlokliTestClient<StaticState> {
        BlokliTestClient::new(self.0, StaticState)
    }

    pub fn build_dynamic_client(self, module_address: Address) -> BlokliTestClient<FullStateEmulator> {
        BlokliTestClient::new(self.0, FullStateEmulator(module_address, None))
    }

    pub fn build_dynamic_client_with_tx_interceptor(self, module_address: Address)
        -> (BlokliTestClient<FullStateEmulator>, impl futures::Stream<Item = ParsedHoprChainAction>) {
        let (sender, receiver) = futures::channel::mpsc::unbounded();
        let client = BlokliTestClient::new(self.0, FullStateEmulator(module_address, Some(sender)));
        (client, receiver)
    }
}

#[derive(Clone, Debug)]
pub struct StaticState;

impl BlokliTestStateMutator for StaticState {
    fn update_state(
        &self,
        _signed_tx: &[u8],
        _state: &mut BlokliTestState,
    ) -> Result<(), blokli_client::errors::BlokliClientError> {
        Err(
            blokli_client::errors::ErrorKind::MockClientError(anyhow::anyhow!("static client must not update state"))
                .into(),
        )
    }
}

#[derive(Clone, Debug)]
pub struct FullStateEmulator(Address, Option<futures::channel::mpsc::UnboundedSender<ParsedHoprChainAction>>);

impl BlokliTestStateMutator for FullStateEmulator {
    fn update_state(
        &self,
        signed_tx: &[u8],
        state: &mut BlokliTestState,
    ) -> Result<(), blokli_client::errors::BlokliClientError> {
        let addresses: ContractAddresses =
            serde_json::from_str(&state.chain_info.contract_addresses.0).map_err(|_| {
                blokli_client::errors::ErrorKind::MockClientError(anyhow::anyhow!("failed to parse contract addresses"))
            })?;

        let (action, sender) = ParsedHoprChainAction::parse_from_eip2718(signed_tx, &self.0, &addresses)
            .map_err(|e| blokli_client::errors::ErrorKind::MockClientError(e.into()))?;

        match &action {
            ParsedHoprChainAction::RegisterSafeAddress(safe_address) => {
                state
                    .get_account_mut(&sender.into())
                    .ok_or(blokli_client::errors::ErrorKind::MockClientError(anyhow::anyhow!(
                        "missing account for {sender}"
                    )))?
                    .safe_address = Some(hex::encode(safe_address));

                tracing::debug!(%sender, %safe_address, "registered safe address");
            }
            ParsedHoprChainAction::Announce {
                packet_key,
                multiaddress,
            } => {
                if let Some(account) = state.get_account_mut(&sender.into()) {
                    account.packet_key = hex::encode(packet_key);
                    if let Some(multiaddress) = multiaddress.clone() {
                        if !multiaddress.is_empty() {
                            account.multi_addresses.push(multiaddress.to_string());
                        } else {
                            return Err(blokli_client::errors::ErrorKind::MockClientError(anyhow::anyhow!(
                                "multiaddress must not be empty"
                            ))
                            .into());
                        }
                    }
                } else {
                    let next_key_id= state.accounts.keys().max().map(|k| k + 1).unwrap_or(1);
                    state.accounts.insert(next_key_id, blokli_client::api::types::Account {
                        chain_key: hex::encode(sender),
                        keyid: next_key_id as i32,
                        multi_addresses: multiaddress.iter().map(|a| a.to_string()).collect(),
                        packet_key: hex::encode(packet_key),
                        safe_address: None,
                        safe_transaction_count: Some(blokli_client::api::types::Uint64("2".into())),
                    });
                }

                tracing::debug!(%sender, %packet_key, ?multiaddress, "node announced");
            }
            ParsedHoprChainAction::WithdrawNative(destination, amount) => {
                let balance = state.native_balances.get_mut(&hex::encode(sender)).ok_or(
                    blokli_client::errors::ErrorKind::MockClientError(anyhow::anyhow!(
                        "missing native balance for {sender}"
                    )),
                )?;

                let balance_num = balance.balance.0.parse::<XDaiBalance>().map_err(|_| {
                    blokli_client::errors::ErrorKind::MockClientError(anyhow::anyhow!(
                        "failed to parse token balance for {sender}"
                    ))
                })?;

                if &balance_num < amount {
                    return Err(blokli_client::errors::ErrorKind::MockClientError(anyhow::anyhow!(
                        "balance {balance_num} for {sender} is lower than amount {amount}"
                    ))
                    .into());
                }

                balance.balance = blokli_client::api::types::TokenValueString((balance_num - *amount).to_string());

                if let Some(dst_balance) = state.native_balances.get_mut(&hex::encode(destination)) {
                    let new_balance = dst_balance.balance.0.parse::<XDaiBalance>().map_err(|_| {
                        blokli_client::errors::ErrorKind::MockClientError(anyhow::anyhow!(
                            "failed to parse native balance for {destination}"
                        ))
                    })? + *amount;
                    dst_balance.balance = blokli_client::api::types::TokenValueString(new_balance.to_string());

                    tracing::debug!(%sender, %amount, %destination, "xdai withdrawn to an existing account");
                } else {
                    tracing::debug!(%sender, %amount, %destination, "xdai withdrawn");
                }
            }
            ParsedHoprChainAction::WithdrawToken(destination, amount) => {
                let balance = state.token_balances.get_mut(&hex::encode(sender)).ok_or(
                    blokli_client::errors::ErrorKind::MockClientError(anyhow::anyhow!(
                        "missing token balance for {sender}"
                    )),
                )?;

                let balance_num = balance.balance.0.parse::<HoprBalance>().map_err(|_| {
                    blokli_client::errors::ErrorKind::MockClientError(anyhow::anyhow!(
                        "failed to parse token balance for {sender}"
                    ))
                })?;

                if &balance_num < amount {
                    return Err(blokli_client::errors::ErrorKind::MockClientError(anyhow::anyhow!(
                        "balance {balance_num} for {sender} is lower than amount {amount}"
                    ))
                    .into());
                }

                balance.balance = blokli_client::api::types::TokenValueString((balance_num - *amount).to_string());

                if let Some(dst_balance) = state.token_balances.get_mut(&hex::encode(destination)) {
                    let new_balance = dst_balance.balance.0.parse::<HoprBalance>().map_err(|_| {
                        blokli_client::errors::ErrorKind::MockClientError(anyhow::anyhow!(
                            "failed to parse token balance for {destination}"
                        ))
                    })? + *amount;
                    dst_balance.balance = blokli_client::api::types::TokenValueString(new_balance.to_string());

                    tracing::debug!(%sender, %amount, %destination, "wxhopr withdrawn to an existing account");
                } else {
                    tracing::debug!(%sender, %amount, %destination, "wxhopr withdrawn");
                }
            }
            ParsedHoprChainAction::FundChannel(dst_addr, stake) => {
                let source = state
                    .get_account(&sender.into())
                    .cloned()
                    .ok_or(blokli_client::errors::ErrorKind::MockClientError(anyhow::anyhow!(
                        "missing account for {sender}"
                    )))?;
                let destination = state
                    .get_account(&(*dst_addr).into())
                    .cloned()
                    .ok_or(blokli_client::errors::ErrorKind::MockClientError(anyhow::anyhow!(
                        "missing account for {dst_addr}"
                    )))?;

                if stake.is_zero() {
                    return Err(blokli_client::errors::ErrorKind::MockClientError(anyhow::anyhow!(
                        "stake must be greater than zero"
                    ))
                    .into());
                }

                {
                    let safe_balance = state.get_account_safe_token_balance_mut(&sender.into())
                        .ok_or(blokli_client::errors::ErrorKind::MockClientError(anyhow::anyhow!(
                            "missing safe balance for {sender}"
                        )))?;


                    let safe_balance_num = safe_balance.balance.0.parse::<HoprBalance>().map_err(|_| {
                        blokli_client::errors::ErrorKind::MockClientError(anyhow::anyhow!(
                            "failed to parse safe balance for safe of {sender}"
                        ))
                    })?;

                    if &safe_balance_num < stake {
                        return Err(blokli_client::errors::ErrorKind::MockClientError(anyhow::anyhow!(
                            "safe balance of {sender} for {sender} is lower than stake {stake}"
                        ))
                            .into());
                    }

                    safe_balance.balance =
                        blokli_client::api::types::TokenValueString((safe_balance_num - *stake).to_string());
                }

                {
                    let safe_allowance = state.get_account_safe_allowance_mut(&sender.into())
                        .ok_or(blokli_client::errors::ErrorKind::MockClientError(anyhow::anyhow!(
                            "missing safe allowance for {sender}"
                        )))?;

                    let safe_allowance_num = safe_allowance.allowance.0.parse::<HoprBalance>().map_err(|_| {
                        blokli_client::errors::ErrorKind::MockClientError(anyhow::anyhow!(
                            "failed to parse safe allowance for {sender}"
                        ))
                    })?;

                    if &safe_allowance_num < stake {
                        return Err(blokli_client::errors::ErrorKind::MockClientError(anyhow::anyhow!(
                            "safe allowance for {sender} is lower than stake {stake}"
                        ))
                            .into());
                    }

                    safe_allowance.allowance =
                        blokli_client::api::types::TokenValueString((safe_allowance_num - *stake).to_string());
                }

                if let Some(existing_channel) = state
                    .channels
                    .values_mut()
                    .find(|c| c.source == source.keyid && c.destination == destination.keyid)
                {
                    if existing_channel.status != blokli_client::api::types::ChannelStatus::Closed {
                        return Err(blokli_client::errors::ErrorKind::MockClientError(anyhow::anyhow!(
                            "channel {sender} -> {dst_addr} already opened"
                        ))
                        .into());
                    }

                    let balance = existing_channel.balance.0.parse::<HoprBalance>().map_err(|_| {
                        blokli_client::errors::ErrorKind::MockClientError(anyhow::anyhow!(
                            "failed to parse balance on channel {sender} -> {dst_addr}"
                        ))
                    })?;

                    existing_channel.balance =
                        blokli_client::api::types::TokenValueString((balance + *stake).to_string());
                    existing_channel.status = blokli_client::api::types::ChannelStatus::Open;
                    existing_channel.ticket_index = blokli_client::api::types::Uint64("0".into());
                    existing_channel.closure_time = None;
                    existing_channel.epoch += 1;

                    tracing::debug!(%sender, %dst_addr, %stake, "channel re-opened");
                } else {
                    let new_id = generate_channel_id(&sender, &dst_addr);
                    state.channels.insert(new_id.into(),blokli_client::api::types::Channel {
                        balance: blokli_client::api::types::TokenValueString(stake.to_string()),
                        closure_time: None,
                        concrete_channel_id: hex::encode(new_id),
                        destination: destination.keyid,
                        epoch: 1,
                        source: source.keyid,
                        status: blokli_client::api::types::ChannelStatus::Open,
                        ticket_index: blokli_client::api::types::Uint64("0".into()),
                    });

                    tracing::debug!(%sender, %dst_addr, %stake, "channel opened");
                }
            }
            ParsedHoprChainAction::InitializeChannelClosure(channel_id) => {
                let grace_period = state
                    .chain_info
                    .channel_closure_grace_period
                    .clone()
                    .map(|p| p.0)
                    .unwrap_or("10".into());
                let grace_period = u64::from_str(&grace_period)
                    .map(|p| std::time::Duration::from_secs(p).max(std::time::Duration::from_secs(2)))
                    .map_err(|_| {
                        blokli_client::errors::ErrorKind::MockClientError(anyhow::anyhow!(
                            "failed to parse channel closure grace period"
                        ))
                    })?;

                let channel = state
                    .get_channel_by_id_mut(&channel_id.into())
                    .ok_or(blokli_client::errors::ErrorKind::MockClientError(anyhow::anyhow!(
                        "missing channel {channel_id}"
                    )))?;

                channel.status = blokli_client::api::types::ChannelStatus::PendingToClose;
                channel.closure_time = Some(blokli_client::api::types::DateTime(
                    hopr_api::chain::DateTime::from(std::time::SystemTime::now().add(grace_period)).to_rfc3339(),
                ));

                tracing::debug!(%channel_id, "channel closure initialized");
            }
            ParsedHoprChainAction::FinalizeChannelClosure(channel_id) => {
                let channel = state
                    .get_channel_by_id_mut(&channel_id.into())
                    .ok_or(blokli_client::errors::ErrorKind::MockClientError(anyhow::anyhow!(
                        "missing channel {channel_id}"
                    )))?;

                if channel.status != blokli_client::api::types::ChannelStatus::PendingToClose {
                    return Err(blokli_client::errors::ErrorKind::MockClientError(anyhow::anyhow!(
                        "channel {channel_id} is not pending to close"
                    ))
                    .into());
                }

                channel.status = blokli_client::api::types::ChannelStatus::Closed;

                tracing::debug!(%channel_id, "channel closure finalized");
            }
            ParsedHoprChainAction::IncomingChannelClosure(channel_id) => {
                let channel = state
                    .get_channel_by_id_mut(&channel_id.into())
                    .ok_or(blokli_client::errors::ErrorKind::MockClientError(anyhow::anyhow!(
                        "missing channel {channel_id}"
                    )))?;

                channel.status = blokli_client::api::types::ChannelStatus::Closed;

                tracing::debug!(%channel_id, "incoming channel closed");
            }
            ParsedHoprChainAction::RedeemTicket {
                channel_id,
                ticket_index,
                ticket_amount,
            } => {
                let channel = state
                    .get_channel_by_id_mut(&channel_id.into())
                    .ok_or(blokli_client::errors::ErrorKind::MockClientError(anyhow::anyhow!(
                        "missing channel {channel_id}"
                    )))?;

                if channel.status == blokli_client::api::types::ChannelStatus::Closed {
                    return Err(blokli_client::errors::ErrorKind::MockClientError(anyhow::anyhow!(
                        "channel {channel_id} is closed"
                    ))
                    .into());
                }

                let channel_ticket_index = u64::from_str(&channel.ticket_index.0).map_err(|_| {
                    blokli_client::errors::ErrorKind::MockClientError(anyhow::anyhow!(
                        "failed to parse ticket index of {channel_id}"
                    ))
                })?;

                if &channel_ticket_index < ticket_index {
                    return Err(blokli_client::errors::ErrorKind::MockClientError(anyhow::anyhow!(
                        "ticket index of {channel_id} is lower than redeemed ticket index {ticket_index}"
                    ))
                    .into());
                }

                let balance = channel.balance.0.parse::<HoprBalance>().map_err(|_| {
                    blokli_client::errors::ErrorKind::MockClientError(anyhow::anyhow!(
                        "failed to parse balance on channel {channel_id}"
                    ))
                })?;

                if &balance < ticket_amount {
                    return Err(blokli_client::errors::ErrorKind::MockClientError(anyhow::anyhow!(
                        "balance of channel {channel_id} is lower than ticket amount {ticket_amount}"
                    ))
                    .into());
                }

                channel.ticket_index = blokli_client::api::types::Uint64(ticket_index.to_string());
                channel.balance = blokli_client::api::types::TokenValueString((balance - *ticket_amount).to_string());

                let channel = channel.clone();
                if let Some(opposite_channel) = state
                    .channels
                    .values_mut()
                    .find(|c| c.source == channel.destination && c.destination == channel.source)
                {
                    let balance = opposite_channel.balance.0.parse::<HoprBalance>().map_err(|_| {
                        blokli_client::errors::ErrorKind::MockClientError(anyhow::anyhow!(
                            "failed to parse balance on opposite channel {channel_id}"
                        ))
                    })?;
                    opposite_channel.balance =
                        blokli_client::api::types::TokenValueString((balance + *ticket_amount).to_string());

                    tracing::debug!(%channel_id, %ticket_index, other_id = channel.concrete_channel_id, "ticket redeemed with channel rebalance");
                } else if let Some((safe_addr, safe_balance)) = state
                    .accounts
                    .get_mut(&(channel.destination as u32))
                    .and_then(|a| a.safe_address.clone())
                    .and_then(|safe_addr| state.token_balances.get_mut(&safe_addr).map(|b| (safe_addr, b)))
                {
                    let balance = safe_balance.balance.0.parse::<HoprBalance>().map_err(|_| {
                        blokli_client::errors::ErrorKind::MockClientError(anyhow::anyhow!(
                            "failed to parse balance on safe {safe_addr}"
                        ))
                    })?;
                    safe_balance.balance =
                        blokli_client::api::types::TokenValueString((balance + *ticket_amount).to_string());

                    tracing::debug!(%channel_id, %ticket_index, %safe_addr, "ticket redeemed into safe");
                } else {
                    tracing::debug!(%channel_id, %ticket_index, "ticket redeemed");
                }
            }
        }

        let my_account = state
            .get_account_mut(&sender.into())
            .ok_or(blokli_client::errors::ErrorKind::MockClientError(anyhow::anyhow!("cannot find own account {sender}")))?;

        let tx_count = my_account
            .safe_transaction_count
            .as_ref()
            .ok_or(
                blokli_client::errors::ErrorKind::MockClientError(anyhow::anyhow!("cannot own tx count {sender}"))
            )
            .and_then(|tx_count| tx_count.0.parse::<u64>().map_err(|_| blokli_client::errors::ErrorKind::MockClientError(anyhow::anyhow!("cannot parse tx count of {sender}"))))?;

        my_account.safe_transaction_count = Some(blokli_client::api::types::Uint64((tx_count + 1).to_string()));

        if let Some(sender) = &self.1 {
            sender.unbounded_send(action)
                .map_err(|_|
                    blokli_client::errors::ErrorKind::MockClientError(anyhow::anyhow!("failed to send tx to tx interceptor"))
                )?;
        }

        Ok(())
    }
}
