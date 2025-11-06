pub use blokli_client::BlokliTestClient;
use hex::ToHex as HexHex;
use hopr_api::chain::ChainInfo;
use hopr_internal_types::channels::ChannelStatus;
use hopr_internal_types::prelude::{AccountEntry, ChannelEntry, WinningProbability};
use hopr_primitive_types::balance::Balance;
use hopr_primitive_types::prelude::{Address, Currency, HoprBalance, ToHex, WxHOPR, XDai};

#[derive(Default)]
pub struct BlokliTestClientBuilder(BlokliTestClient);

impl BlokliTestClientBuilder {
    pub fn with_channels<I: IntoIterator<Item = ChannelEntry>>(mut self, channels: I) -> Self {
        self.0.channels.extend(channels.into_iter().map(|channel| {
            blokli_client::api::types::Channel {
                balance: blokli_client::api::types::TokenValueString(channel.balance.to_string()),
                closure_time: if let ChannelStatus::PendingToClose(time) = channel.status {
                    Some(blokli_client::api::types::DateTime(hopr_api::chain::DateTime::from(time).to_rfc3339()))
                } else {
                    None
                },
                concrete_channel_id: channel.get_id().to_hex(),
                destination: self.0.accounts.iter().find(|a| a.chain_key == channel.source.to_hex()).map(|a| a.keyid).expect(&format!("missing dst account {}", channel.source)),
                epoch: channel.channel_epoch.as_u32() as i32,
                source: self.0.accounts.iter().find(|a| a.chain_key == channel.destination.to_hex()).map(|a| a.keyid).expect(&format!("missing src account {}", channel.source)),
                status: match channel.status {
                    ChannelStatus::Closed => blokli_client::api::types::ChannelStatus::Closed,
                    ChannelStatus::Open => blokli_client::api::types::ChannelStatus::Open,
                    ChannelStatus::PendingToClose(_) => blokli_client::api::types::ChannelStatus::PendingToClose,
                },
                ticket_index: blokli_client::api::types::Uint64(channel.ticket_index.as_u64().to_string()),
            }
        }));
        self
    }

    pub fn with_accounts<I: IntoIterator<Item = AccountEntry>>(mut self, accounts: I) -> Self {
        self.0.accounts.extend(accounts.into_iter().map(|account| {
            blokli_client::api::types::Account {
                chain_key: account.chain_addr.to_hex(),
                keyid: u32::from(account.key_id) as i32,
                multi_addresses: account.get_multiaddr().iter().map(|a| a.to_string()).collect(),
                packet_key: account.public_key.to_hex(),
                safe_address: account.safe_address.map(|a| a.to_hex()),
                safe_transaction_count: None,
            }
        }));
        self
    }

    pub fn with_balances<C: Currency>(mut self, balances: impl IntoIterator<Item = (Address, Balance<C>)>) -> Self {
        if C::is::<XDai>() {
            self.0.native_balances.extend(balances.into_iter()
                .map(|(addr, balance)| {
                    (addr.encode_hex(), blokli_client::api::types::NativeBalance {
                        __typename: "".into(),
                        balance: blokli_client::api::types::TokenValueString(balance.to_string()),
                    })
                }))
        } else if C::is::<WxHOPR>() {
            self.0.token_balances.extend(balances.into_iter()
                .map(|(addr, balance)| {
                    (addr.encode_hex(), blokli_client::api::types::HoprBalance {
                        __typename: "".into(),
                        balance: blokli_client::api::types::TokenValueString(balance.to_string()),
                    })
                }))
        } else {
            panic!("unsupported currency");
        }

        self
    }

    pub fn with_safe_allowances<I: IntoIterator<Item = (Address, HoprBalance)>>(mut self, balances: I) -> Self {
        self.0.safe_allowances.extend(balances.into_iter().map(|(addr, allowance)| {
            (addr.encode_hex(), blokli_client::api::types::SafeHoprAllowance {
                __typename: "".into(),
                allowance: blokli_client::api::types::TokenValueString(allowance.to_string()),
            })
        }));
        self
    }

    pub fn with_chain_info(mut self, info: ChainInfo) -> Self {
        self.0.chain_info.chain_id = info.chain_id as i32;
        self.0.chain_info.contract_addresses = blokli_client::api::types::ContractAddressMap(serde_json::to_string(&info.contract_addresses).expect("failed to serialize contract addresses"));
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
        self.0.chain_info.channel_closure_grace_period = Some(blokli_client::api::types::Uint64(grace_period.as_secs().to_string()));
        self
    }

    pub fn with_tx_client(mut self, tx_client: blokli_client::MockBlokliTransactionClientImpl) -> Self {
        self.0.tx_client = Some(tx_client);
        self
    }

    pub fn build(self) -> BlokliTestClient {
        self.0
    }
}