mod chain;
mod components;
pub mod config;
pub mod constants;
pub mod errors;
mod helpers;
mod processes;

pub use chain::{Network, ProtocolConfig};
use core_ethereum_actions::node::NodeActions;

use std::collections::HashMap;
use std::sync::Arc;
use std::{pin::Pin, str::FromStr};

use async_std::sync::RwLock;
use futures::{Future, StreamExt};

use crate::chain::ChainNetworkConfig;
use core_ethereum_api::HoprChain;
use core_ethereum_db::{db::CoreEthereumDb, traits::HoprCoreEthereumDbActions};
use core_transport::libp2p_identity::PeerId;
use core_transport::{
    ApplicationData, ChainKeypair, HalfKeyChallenge, Hash, Health, Keypair, Multiaddr, OffchainKeypair,
};

use utils_log::{error, info};
use utils_types::primitives::{Address, Balance, BalanceType, Snapshot, U256};

#[cfg(feature = "wasm")]
use {core_ethereum_db::db::wasm::Database, core_transport::wasm_impls::HoprTransport};

#[cfg(all(feature = "prometheus", not(test), not(feature = "wasm")))]
use utils_misc::time::native::current_timestamp;

#[cfg(all(feature = "prometheus", not(test), feature = "wasm"))]
use utils_misc::time::wasm::current_timestamp;

#[cfg(all(feature = "prometheus", not(test)))]
use utils_metrics::metrics::{MultiGauge, SimpleCounter, SimpleGauge};

#[cfg(all(feature = "prometheus", not(test)))]
lazy_static::lazy_static! {
    static ref METRIC_MESSAGE_FAIL_COUNT: SimpleCounter = SimpleCounter::new(
        "core_counter_failed_send_messages",
        "Number of sent messages failures"
    ).unwrap();
    static ref METRIC_PROCESS_START_TIME: SimpleGauge = SimpleGauge::new(
        "hoprd_gauge_startup_unix_time_seconds",
        "The unix timestamp at which the process was started"
    ).unwrap();
    static ref METRIC_HOPR_LIB_VERSION: MultiGauge = MultiGauge::new(
        "hoprd_mgauge_version",
        "Executed version of HOPRd",
        &["version"]
    ).unwrap();
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum State {
    Uninitialized = 0,
    Initializing = 1,
    Indexing = 2,
    Starting = 3,
    Running = 4,
}

// #[cfg(any(not(feature = "wasm"), test))]
// pub use native::Hopr;

#[cfg(all(feature = "wasm", not(test)))]
pub use wasm_impl::Hopr;

#[cfg(feature = "wasm")]
mod native {
    use core_ethereum_actions::transaction_queue::TransactionExecutor;
    use core_ethereum_actions::{
        channels::ChannelActions, redeem::TicketRedeemActions, transaction_queue::TransactionResult,
    };
    use core_ethereum_api::ChannelEntry;
    use core_transport::{wasm_impls::PublicNodesResult, TicketStatistics};
    use core_types::{
        account::AccountEntry,
        acknowledgement::AcknowledgedTicket,
        channels::{generate_channel_id, ChannelStatus, Ticket},
    };
    use utils_types::traits::PeerIdLike;

    use crate::wasm_impl::{CloseChannelResult, OpenChannelResult};

    use super::*;

    pub struct Hopr {
        me: OffchainKeypair,
        is_public: bool,
        state: State,

        /// RwLocked HashMap making sure that no Rust aliasing issues
        /// can occur, discard once Rust support is not needed
        aliases: Arc<RwLock<HashMap<String, PeerId>>>,
        transport_api: HoprTransport,
        chain_api: HoprChain,
        processes: Option<Vec<Pin<Box<dyn futures::Future<Output = components::HoprLoopComponents>>>>>,
        chain_cfg: ChainNetworkConfig,
        #[cfg(feature = "wasm")]
        chain_query: chain::wasm::WasmChainQuery,
        // TODO: remove once the entire construction happens in the new() method
        staking_safe_address: Address,
        staking_module_address: Address,
    }

    impl Hopr {
        pub fn new<FOnReceived, FOnSent, FSaveTbf, TxExec>(
            cfg: crate::config::HoprLibConfig,
            me: &OffchainKeypair,
            me_onchain: &ChainKeypair,
            my_addresses: Vec<Multiaddr>,
            db: Arc<RwLock<CoreEthereumDb<utils_db::rusty::RustyLevelDbShim>>>,
            tbf: core_types::protocol::TagBloomFilter,
            save_tbf: FSaveTbf,
            chain_config: ChainNetworkConfig,
            tx_executor: TxExec,
            #[cfg(feature = "wasm")] chain_query: chain::wasm::WasmChainQuery, // chain operations currently only in JS
            on_received: FOnReceived, // passed emit on the WasmHoprMessageEmitter on packet received
            on_sent: FOnSent,         // passed emit on the WasmHoprMessageEmitter on packet sent
        ) -> Self
        where
            FOnReceived: Fn(ApplicationData) + 'static,
            FOnSent: Fn(HalfKeyChallenge) + 'static,
            FSaveTbf: Fn(Box<[u8]>) + 'static,
            TxExec: TransactionExecutor + 'static,
        {
            // let mut packetCfg = PacketInteractionConfig::new(packetKeypair, chainKeypair)
            // packetCfg.check_unrealized_balance = cfg.chain.check_unrealized_balance

            let is_public = cfg.chain.announce;

            let (transport_api, chain_api, processes) = components::build_components(
                cfg.clone(),
                me.clone(),
                me_onchain.clone(),
                db,
                on_sent,
                on_received,
                tbf,
                save_tbf,
                tx_executor,
                my_addresses,
            );

            #[cfg(all(feature = "prometheus", not(test)))]
            {
                METRIC_PROCESS_START_TIME.set(current_timestamp() as f64 / 1000.0);
                METRIC_HOPR_LIB_VERSION.set(
                    &["version"],
                    f64::from_str(const_format::formatcp!(
                        "{}.{}",
                        env!("CARGO_PKG_VERSION_MAJOR"),
                        env!("CARGO_PKG_VERSION_MINOR")
                    ))
                    .unwrap_or(0.0),
                );
            }

            Self {
                state: State::Uninitialized,
                aliases: Arc::new(RwLock::new(HashMap::new())),
                is_public,
                me: me.clone(),
                transport_api,
                chain_api,
                processes: Some(processes),
                chain_cfg: chain_config,
                chain_query,
                staking_safe_address: cfg.safe_module.safe_address,
                staking_module_address: cfg.safe_module.module_address,
            }
        }

        pub fn status(&self) -> State {
            self.state
        }

        pub fn version_coerced(&self) -> String {
            String::from(constants::APP_VERSION_COERCED)
        }

        pub fn version(&self) -> String {
            String::from(constants::APP_VERSION)
        }

        pub async fn set_alias(&self, alias: String, peer: PeerId) {
            self.aliases.write().await.insert(alias, peer);
        }

        pub async fn remove_alias(&self, alias: &String) {
            self.aliases.write().await.remove(alias);
        }

        pub async fn get_alias(&self, alias: &String) -> Option<PeerId> {
            self.aliases.read().await.get(alias).map(|v| v.clone())
        }

        pub async fn get_aliases(&self) -> HashMap<String, PeerId> {
            self.aliases.read().await.clone()
        }

        pub fn chain_config(&self) -> ChainNetworkConfig {
            self.chain_cfg.clone()
        }

        pub async fn run(
            &mut self,
        ) -> errors::Result<Vec<Pin<Box<dyn Future<Output = components::HoprLoopComponents>>>>> {
            if self.state != State::Uninitialized {
                return Err(errors::HoprLibError::GeneralError(
                    "Cannot start the hopr node multiple times".to_owned(),
                ));
            }

            info!("Starting hopr node...");

            if let Err(_) = self.chain_query.waitForFunds().await {
                panic!("Failed to wait for the funds")
            }

            self.aliases
                .write()
                .await
                .insert("me".to_owned(), self.transport_api.me().clone());

            self.state = State::Initializing;

            let balance = self
                .chain_query
                .getNativeBalance()
                .await
                .map_err(|_| errors::HoprLibError::GeneralError("Failed to fetch own balance".to_owned()))
                .map(|v| {
                    info!("{}", v.as_string().unwrap().as_str());
                    Balance::from_str(v.as_string().unwrap().as_str()).expect("balance should be deserializable")
                })?;

            let minimum_balance = Balance::new(U256::new(constants::MIN_NATIVE_BALANCE), BalanceType::Native);

            info!(
                "Ethereum account {} has {}. Mininum balance is {}",
                self.chain_api.me_onchain(),
                balance.to_formatted_string(),
                minimum_balance.to_formatted_string()
            );

            if balance.lte(&minimum_balance) {
                return Err(errors::HoprLibError::GeneralError(
                    "Cannot start the node without a sufficiently funded wallet".to_string(),
                ));
            }

            info!("Linking chain and packet keys");
            self.chain_api
                .db()
                .write()
                .await
                .link_chain_and_packet_keys(&self.chain_api.me_onchain(), &self.me.public(), &Snapshot::default())
                .await
                .map_err(core_transport::errors::HoprTransportError::from)?;

            let mut allowed_initial_nodes = vec![];

            for node in self.transport_api.get_public_nodes().await?.into_iter() {
                let peer_id: String = node.id.clone().into();
                if let Ok(peer_id) = PeerId::from_str(peer_id.as_str()) {
                    if self.transport_api.is_allowed_to_access_network(&peer_id).await {
                        allowed_initial_nodes.push(node)
                    }
                }
            }

            info!(
                "Using initial nodes: {:?}",
                allowed_initial_nodes
                    .iter()
                    .map(|v| {
                        let n: String = v.id.clone().into();
                        n
                    })
                    .collect::<Vec<_>>()
            );

            // TODO: public nodes result should be native, not wasm based
            info!("Loading initial peers");
            for node in allowed_initial_nodes.into_iter() {
                self.transport_api.on_network_registry_update(&node.address, true).await;

                self.transport_api
                    .on_peer_announcement(
                        {
                            let peer: String = node.id.into();
                            PeerId::from_str(&peer).expect("Should be a valid PeerId string")
                        },
                        node.address.clone(),
                        node.multiaddrs
                            .iter()
                            .map(|v| {
                                let ma: String = v.into();
                                Multiaddr::from_str(&ma).expect("Should be a valid Multiaddr string")
                            })
                            .collect::<Vec<_>>(),
                    )
                    .await;
            }

            // TODO: this state now lives in the loops and cannot be extracted in the setup without
            // fully migrating the rest of the hopr-lib packages
            // self.state = State::Indexing;

            // wait for the indexer sync
            // info!("Starting chain interaction, which will trigger the indexer");
            // if let Err(_) = ck.startChainSync().await {
            //     panic!("Failed to start the chain operations");
            // }

            // Possibly register node-safe pair to NodeSafeRegistry. Following that the
            // connector is set to use safe tx variants.
            if self
                .chain_query
                .canRegisterWithSafe()
                .await
                .map(|v| v.as_bool().unwrap_or(false))
                .unwrap_or(false)
            {
                info!("Registering safe by node");

                if self.me_onchain() == self.staking_safe_address {
                    return Err(errors::HoprLibError::GeneralError(
                        "cannot self as staking safe address".into(),
                    ));
                }

                if let Ok(_) = self
                    .chain_api
                    .actions_ref()
                    .register_safe_by_node(self.staking_safe_address)
                    .await
                {
                    let db = self.chain_api.db().clone();
                    let mut db = db.write().await;
                    db.set_staking_safe_address(&self.staking_safe_address).await?;
                    db.set_staking_module_address(&self.staking_module_address).await?;
                } else {
                    // DB_ONLY
                    // Intentionally ignoring the errored state
                    error!("Failed to register node with safe")
                }
            }

            if self.is_public {
                // At this point the node is already registered with Safe, so
                // we can announce via Safe-compliant TX

                // TODO: allow announcing all addresses once that option is supported
                let multiaddresses_to_announce = self.transport_api.announceable_multiaddresses();
                info!("Announcing node on chain: {:?}", &multiaddresses_to_announce[0]);
                if let Err(_) = self
                    .chain_api
                    .actions_ref()
                    .announce(&multiaddresses_to_announce[0], &self.me)
                    .await
                {
                    // If the announcement fails we keep going to prevent the node from retrying
                    // after restart. Functionality is limited and users must check the logs for
                    // errors.
                    error!("Failed to announce a node")
                }
            }

            self.state = State::Running;

            {
                info!("Syncing channels from the previous runs");
                let locked_db = self.chain_api.db();
                let db = locked_db.read().await;
                if let Err(e) = self.chain_api.channel_graph().write().await.sync_channels(&*db).await {
                    error!("failed to initialize channel graph from the DB: {e}");
                }
            }

            // Note: this is not how a normal binary would return the operation, but it
            // is a valid way for WASM environment to yield `&mut` work to JS
            // info!("# STARTED NODE");
            // info!("ID {}", self.transport_api.me());
            // info!("Protocol version {}", constants::APP_VERSION);

            Ok(self.processes.take().expect("processes should be present in the node"))
        }

        // p2p transport =========
        /// Own PeerId used in the libp2p transport layer
        pub fn me_peer_id(&self) -> PeerId {
            self.me.public().to_peerid()
        }

        /// Get the list of all announced public nodes in the network
        pub async fn get_public_nodes(&self) -> errors::Result<Vec<PublicNodesResult>> {
            Ok(self.transport_api.get_public_nodes().await?)
        }

        /// Test whether the peer with PeerId is allowed to access the network
        pub async fn is_allowed_to_access_network(&self, peer: &PeerId) -> bool {
            self.transport_api.is_allowed_to_access_network(&peer).await
        }

        /// Ping another node in the network based on the PeerId
        pub async fn ping(&self, peer: &PeerId) -> errors::Result<Option<std::time::Duration>> {
            if self.status() != State::Running {
                return Err(crate::errors::HoprLibError::GeneralError(
                    "Node is not ready for on-chain operations".into(),
                ));
            }

            Ok(self.transport_api.ping(&peer).await)
        }

        /// Send a message to another peer in the network
        ///
        /// @param msg message to send
        /// @param destination PeerId of the destination
        /// @param intermediatePath optional set path manually
        /// @param hops optional number of required intermediate nodes
        /// @param applicationTag optional tag identifying the sending application
        /// @returns ack challenge
        pub async fn send_message(
            &self,
            msg: Box<[u8]>,
            destination: PeerId,
            intermediate_path: Option<Vec<PeerId>>,
            hops: Option<u16>,
            application_tag: Option<u16>,
        ) -> errors::Result<HalfKeyChallenge> {
            if self.status() != State::Running {
                return Err(crate::errors::HoprLibError::GeneralError(
                    "Node is not ready for on-chain operations".into(),
                ));
            }

            let result = self
                .transport_api
                .send_message(msg, destination, intermediate_path, hops, application_tag)
                .await;

            if result.is_err() {
                #[cfg(all(feature = "prometheus", not(test)))]
                SimpleCounter::increment(&METRIC_MESSAGE_FAIL_COUNT);
            }

            Ok(result?)
        }

        /// Attempts to aggregate all tickets in the given channel
        pub async fn aggregate_tickets(&mut self, channel: &Hash) -> errors::Result<()> {
            Ok(self.transport_api.aggregate_tickets(channel).await?)
        }

        /// List all multiaddresses announced
        pub fn local_multiaddresses(&self) -> Vec<Multiaddr> {
            self.transport_api.local_multiaddresses()
        }

        /// List all multiaddresses on which the node is listening
        pub async fn listening_multiaddresses(&self) -> Vec<Multiaddr> {
            self.transport_api.listening_multiaddresses().await
        }

        /// List all multiaddresses observed for a PeerId
        pub async fn network_observed_multiaddresses(&self, peer: &PeerId) -> Vec<Multiaddr> {
            self.transport_api.network_observed_multiaddresses(peer).await
        }

        /// List all multiaddresses for this node announced to DHT
        pub async fn multiaddresses_announced_to_dht(&self, peer: &PeerId) -> Vec<Multiaddr> {
            self.transport_api.multiaddresses_announced_to_dht(peer).await
        }

        // Network =========

        /// Get measured network health
        pub async fn network_health(&self) -> Health {
            self.transport_api.network_health().await
        }

        /// Called whenever a peer is announced
        /// @param peer newly announced peer
        pub async fn on_peer_announcement(&self, peer: PeerId, address: Address, multiaddresses: Vec<Multiaddr>) {
            if self.me_peer_id() != peer {
                self.transport_api
                    .on_peer_announcement(peer, address, multiaddresses)
                    .await
            }
        }

        /// Unregister a peer from the network
        pub async fn unregister(&self, peer: &PeerId) {
            self.transport_api.network_unregister(&peer).await
        }

        /// Called whenever the network registry changes
        pub async fn on_network_registry_update(&self, address: &Address, allowed: bool) {
            self.transport_api.on_network_registry_update(address, allowed).await
        }

        /// List all peers connected to this
        pub async fn network_connected_peers(&self) -> Vec<PeerId> {
            self.transport_api.network_connected_peers().await
        }

        /// Get all data collected from the network relevant for a PeerId
        pub async fn network_peer_info(&self, peer: &PeerId) -> Option<core_transport::PeerStatus> {
            self.transport_api.network_peer_info(peer).await
        }

        // Ticket ========
        /// Get all tickets in a channel specified by Hash
        pub async fn tickets_in_channel(&self, channel: &Hash) -> errors::Result<Vec<AcknowledgedTicket>> {
            Ok(self.transport_api.tickets_in_channel(channel).await?)
        }

        /// Get all tickets
        pub async fn all_tickets(&self) -> errors::Result<Vec<Ticket>> {
            Ok(self.transport_api.all_tickets().await?)
        }

        /// Get statistics for all tickets
        pub async fn ticket_statistics(&self) -> errors::Result<TicketStatistics> {
            Ok(self.transport_api.ticket_statistics().await?)
        }

        // Chain =========
        pub fn me_onchain(&self) -> Address {
            self.chain_api.me_onchain()
        }

        /// List of all accounts announced on the chain
        pub async fn accounts_announced_on_chain(&self) -> errors::Result<Vec<AccountEntry>> {
            Ok(self.chain_api.accounts_announced_on_chain().await?)
        }

        /// Get the channel entry from Hash.
        /// @returns the channel entry of those two nodes
        pub async fn channel_from_hash(&self, channel: &Hash) -> errors::Result<Option<ChannelEntry>> {
            Ok(self.chain_api.db().read().await.get_channel(channel).await?)
        }

        /// Get the channel entry between source and destination node.
        /// @param src Address
        /// @param dest Address
        /// @returns the channel entry of those two nodes
        pub async fn channel(&self, src: &Address, dest: &Address) -> errors::Result<ChannelEntry> {
            Ok(self.chain_api.channel(src, dest).await?)
        }

        /// List all channels open from a specified Address
        pub async fn channels_from(&self, src: &Address) -> errors::Result<Vec<ChannelEntry>> {
            Ok(self.chain_api.channels_from(src).await?)
        }

        /// List all channels open to a specified address
        pub async fn channels_to(&self, dest: &Address) -> errors::Result<Vec<ChannelEntry>> {
            Ok(self.chain_api.channels_to(&dest).await?)
        }

        /// List all channels
        pub async fn all_channels(&self) -> errors::Result<Vec<ChannelEntry>> {
            Ok(self.chain_api.all_channels().await?)
        }

        /// Current safe allowance balance
        pub async fn safe_allowance(&self) -> errors::Result<Balance> {
            Ok(self.chain_api.safe_allowance().await?)
        }

        /// Withdraw on-chain assets to a given address
        /// @param currency either native currency or HOPR tokens
        /// @param recipient the account where the assets should be transferred to
        /// @param amount how many tokens to be transferred
        pub async fn withdraw(&self, recipient: Address, amount: Balance) -> errors::Result<Hash> {
            if self.status() != State::Running {
                return Err(crate::errors::HoprLibError::GeneralError(
                    "Node is not ready for on-chain operations".into(),
                ));
            }

            let awaiter = self.chain_api.actions_ref().withdraw(recipient, amount).await?;
            match awaiter.await {
                TransactionResult::Withdrawn { tx_hash } => Ok(tx_hash),
                _ => Err(errors::HoprLibError::GeneralError("withdraw transaction failed".into())),
            }
        }

        /// Callback function used to react to on-chain channel update events.
        /// Specifically we trigger the strategy on channel close handler.
        pub async fn on_own_channel_updated(&self, channel: &ChannelEntry) {
            self.chain_api.on_channel_event(channel).await
        }

        /// Callback function used to react to on-chain channel ticket redeem events.
        /// Specifically we resolve the pending balance of the ticket.
        ///
        /// @param channel object
        /// @param ticket amount
        pub async fn on_ticket_redeemed(&self, channel: &ChannelEntry, value: &Balance) {
            self.chain_api.on_ticket_redeemed(channel, value).await
        }

        pub async fn open_channel(&self, destination: &Address, amount: &Balance) -> errors::Result<OpenChannelResult> {
            if self.status() != State::Running {
                return Err(crate::errors::HoprLibError::GeneralError(
                    "Node is not ready for on-chain operations".into(),
                ));
            }

            let awaiter = self
                .chain_api
                .actions_ref()
                .open_channel(destination.clone(), *amount)
                .await?;

            let channel_id = generate_channel_id(&self.chain_api.me_onchain(), destination);
            match awaiter.await {
                TransactionResult::ChannelFunded { tx_hash } => Ok(OpenChannelResult { tx_hash, channel_id }),
                _ => Err(errors::HoprLibError::GeneralError(
                    "open channel transaction failed".into(),
                )),
            }
        }

        pub async fn fund_channel(&self, channel_id: &Hash, amount: &Balance) -> errors::Result<Hash> {
            if self.status() != State::Running {
                return Err(crate::errors::HoprLibError::GeneralError(
                    "Node is not ready for on-chain operations".into(),
                ));
            }

            let awaiter = self.chain_api.actions_ref().fund_channel(*channel_id, *amount).await?;
            match awaiter.await {
                TransactionResult::ChannelFunded { tx_hash } => Ok(tx_hash),
                _ => Err(errors::HoprLibError::GeneralError(
                    "fund channel transaction failed".into(),
                )),
            }
        }

        pub async fn close_channel(
            &self,
            counterparty: &Address,
            direction: core_types::channels::ChannelDirection,
            redeem_before_close: bool,
        ) -> errors::Result<CloseChannelResult> {
            if self.status() != State::Running {
                return Err(crate::errors::HoprLibError::GeneralError(
                    "Node is not ready for on-chain operations".into(),
                ));
            }

            let awaiter = self
                .chain_api
                .actions_ref()
                .close_channel(*counterparty, direction, redeem_before_close)
                .await?;
            match awaiter.await {
                TransactionResult::ChannelClosureInitiated { tx_hash } => Ok(CloseChannelResult {
                    tx_hash,
                    status: ChannelStatus::PendingToClose,
                }),
                TransactionResult::ChannelClosed { tx_hash } => Ok(CloseChannelResult {
                    tx_hash,
                    status: ChannelStatus::Closed,
                }),
                _ => Err(errors::HoprLibError::GeneralError(
                    "close channel transaction failed".into(),
                )),
            }
        }

        pub async fn redeem_all_tickets(&self, only_aggregated: bool) -> errors::Result<()> {
            if self.status() != State::Running {
                return Err(crate::errors::HoprLibError::GeneralError(
                    "Node is not ready for on-chain operations".into(),
                ));
            }

            // We do not await the on-chain confirmation
            self.chain_api.actions_ref().redeem_all_tickets(only_aggregated).await?;

            Ok(())
        }

        pub async fn redeem_tickets_with_counterparty(
            &self,
            counterparty: &Address,
            only_aggregated: bool,
        ) -> errors::Result<()> {
            if self.status() != State::Running {
                return Err(crate::errors::HoprLibError::GeneralError(
                    "Node is not ready for on-chain operations".into(),
                ));
            }

            // We do not await the on-chain confirmation
            let _ = self
                .chain_api
                .actions_ref()
                .redeem_tickets_with_counterparty(counterparty, only_aggregated)
                .await?;

            Ok(())
        }

        pub async fn redeem_tickets_in_channel(&self, channel: &Hash, only_aggregated: bool) -> errors::Result<()> {
            if self.status() != State::Running {
                return Err(crate::errors::HoprLibError::GeneralError(
                    "Node is not ready for on-chain operations".into(),
                ));
            }

            let channel = self.chain_api.db().read().await.get_channel(channel).await?;

            if let Some(channel) = channel {
                if channel.destination == self.chain_api.me_onchain() {
                    // We do not await the on-chain confirmation
                    self.chain_api
                        .actions_ref()
                        .redeem_tickets_in_channel(&channel, only_aggregated)
                        .await?;
                }
            }

            Ok(())
        }

        pub async fn redeem_ticket(&self, ack_ticket: AcknowledgedTicket) -> errors::Result<()> {
            if self.status() != State::Running {
                return Err(crate::errors::HoprLibError::GeneralError(
                    "Node is not ready for on-chain operations".into(),
                ));
            }

            // We do not await the on-chain confirmation
            let _ = self.chain_api.actions_ref().redeem_ticket(ack_ticket).await?;

            Ok(())
        }

        pub async fn peerid_to_chain_key(&self, peer_id: &PeerId) -> errors::Result<Option<Address>> {
            let pk = core_transport::OffchainPublicKey::from_peerid(peer_id)?;
            Ok(self.chain_api.db().read().await.get_chain_key(&pk).await?)
        }

        pub async fn chain_key_to_peerid(&self, address: &Address) -> errors::Result<Option<PeerId>> {
            Ok(self
                .chain_api
                .db()
                .read()
                .await
                .get_packet_key(address)
                .await
                .map(|pk| pk.map(|v| v.to_peerid()))?)
        }
    }
}

#[cfg(feature = "wasm")]
pub mod wasm_impl {
    use super::*;

    use core_ethereum_actions::payload::SafePayloadGenerator;
    use core_types::acknowledgement::wasm::AcknowledgedTicket;
    use js_sys::{Array, JsString};
    use std::str::FromStr;
    use wasm_bindgen::prelude::*;

    use core_ethereum_api::executors::wasm::{
        WasmEthereumClient, WasmEthereumTransactionExecutor, WasmTaggingPayloadGenerator,
    };
    use core_ethereum_api::ChannelEntry;
    use core_ethereum_types::ContractAddresses;
    use core_transport::{Hash, TicketStatistics};
    use utils_log::{debug, warn};
    use utils_types::{
        primitives::{Address, Balance},
        traits::ToHex,
    };

    use crate::{chain::wasm::ChainConfiguration, processes::wasm::WasmHoprMessageEmitter};

    #[wasm_bindgen]
    pub struct HoprProcesses {
        processes: Vec<Pin<Box<dyn Future<Output = components::HoprLoopComponents>>>>,
    }

    impl HoprProcesses {
        pub fn new(processes: Vec<Pin<Box<dyn Future<Output = components::HoprLoopComponents>>>>) -> Self {
            Self { processes }
        }
    }

    /// Helper object to make sure that the HOPR node remains non-mutable
    /// and that execution over HOPR functionality can be done separately
    /// from wasm_bindgen API calls. Cannot be done as a JsPromise, because
    /// that gets executed immediately.
    #[wasm_bindgen]
    impl HoprProcesses {
        #[wasm_bindgen]
        pub async fn execute(self) -> Result<(), JsValue> {
            let mut futs = helpers::to_futures_unordered(self.processes);
            debug!("Starting the inner loop tasks");
            while let Some(process) = futs.next().await {
                if process.can_finish() {
                    continue;
                } else {
                    error!("CRITICAL: the core system loop unexpectedly stopped: '{}'", process);
                    return Err(JsValue::from(JsError::new(
                        "Futures inside the main loop should never terminate, but run in the background",
                    )));
                }
            }

            Ok(())
        }
    }

    #[wasm_bindgen]
    pub struct Hopr {
        hopr: super::native::Hopr,
        /// object for querying chain through WASM
        chain_query: chain::wasm::WasmChainQuery,
        /// Message emitting for WASM environments
        msg_emitter: processes::wasm::WasmHoprMessageEmitter,
    }

    #[wasm_bindgen]
    impl Hopr {
        #[wasm_bindgen(constructor)]
        pub fn _new(
            cfg: crate::config::HoprLibConfig,
            me: &OffchainKeypair,
            me_onchain: &ChainKeypair,
            db: Database,
            tbf: core_types::protocol::TagBloomFilter,
            save_tbf: js_sys::Function,
            send_eth_tx: js_sys::Function,
            msg_emitter: WasmHoprMessageEmitter, // emitter api delegating the 'on' operation for WSS
            chain_query: chain::wasm::WasmChainQuery, // chain operations currently only in JS
            on_received: js_sys::Function,       // passed emit on the WasmHoprMessageEmitter on packet received
            on_sent: js_sys::Function,           // passed emit on the WasmHoprMessageEmitter on packet sent
        ) -> Self {
            let multiaddress = match cfg.host.address() {
                core_transport::config::HostType::IPv4(ip) => {
                    Multiaddr::from_str(format!("/ip4/{}/tcp/{}", ip.as_str(), cfg.host.port).as_str()).unwrap()
                }
                core_transport::config::HostType::Domain(domain) => {
                    Multiaddr::from_str(format!("/dns4/{}/tcp/{}", domain.as_str(), cfg.host.port).as_str()).unwrap()
                }
            };

            let chain_config = chain::ChainNetworkConfig::new(
                &cfg.chain.network,
                cfg.chain.provider.clone().as_ref().map(|v| v.as_str()),
            )
            .expect("Valid configuration leads to valid network");

            // TODO: this needs refactoring of the config structures
            let contract_addrs = ContractAddresses {
                announcements: Address::from_str(&chain_config.announcements).unwrap(),
                channels: Address::from_str(&chain_config.channels).unwrap(),
                token: Address::from_str(&chain_config.token).unwrap(),
                price_oracle: Address::from_str(&chain_config.ticket_price_oracle).unwrap(),
                network_registry: Address::from_str(&chain_config.network_registry).unwrap(),
                network_registry_proxy: Address::from_str(&chain_config.network_registry_proxy).unwrap(),
                stake_factory: Address::from_str(&chain_config.node_stake_v2_factory).unwrap(),
                safe_registry: Address::from_str(&chain_config.node_safe_registry).unwrap(),
                module_implementation: Address::from_str(&chain_config.module_implementation).unwrap(),
            };

            // Replace this with an EthereumTransactionExecutor with RpcEthereumClient with NodeJs HTTP requestor
            // and after the full migration with Native HTTP requestor.
            let tx_exec = WasmEthereumTransactionExecutor::new(
                WasmEthereumClient::new(send_eth_tx),
                WasmTaggingPayloadGenerator(SafePayloadGenerator::new(
                    &me_onchain,
                    contract_addrs,
                    cfg.safe_module.module_address,
                )),
            );

            Self {
                hopr: super::native::Hopr::new(
                    cfg,
                    me,
                    me_onchain,
                    vec![multiaddress],
                    db.as_ref_counted(),
                    tbf,
                    move |content: Box<[u8]>| {
                        if let Ok(_) = save_tbf.call1(
                            &wasm_bindgen::JsValue::null(),
                            js_sys::Uint8Array::from(content.as_ref()).as_ref(),
                        ) {
                            debug!("tag bloom filter saved");
                        } else {
                            error!("failed to call save tbf closure");
                        }
                    },
                    chain_config,
                    tx_exec,
                    chain_query.clone(),
                    move |data: ApplicationData| {
                        if let Err(e) = on_received.call1(&JsValue::null(), &data.into()) {
                            error!("failed to call on_received_packet closure: {:?}", e.as_string());
                        }
                    },
                    move |ack_challenge: HalfKeyChallenge| {
                        if let Err(e) = on_sent.call1(&JsValue::null(), &ack_challenge.into()) {
                            error!(
                                "failed to call on_received_half_key_challenge closure: {:?}",
                                e.as_string()
                            );
                        }
                    },
                ),
                chain_query,
                msg_emitter,
            }
        }

        #[wasm_bindgen(js_name = getVersion)]
        pub fn _version(&self) -> JsString {
            JsString::from(self.hopr.version())
        }

        #[wasm_bindgen(js_name = getVersionCoerced)]
        pub fn _version_coerced(&self) -> JsString {
            JsString::from(self.hopr.version_coerced())
        }

        #[wasm_bindgen(js_name = setAlias)]
        pub async fn _set_alias(&self, alias: String, peer: String) -> Result<(), JsError> {
            let peer: String = peer;

            if let Ok(peer) = PeerId::from_str(&peer) {
                self.hopr.set_alias(alias, peer).await;
                Ok(())
            } else {
                Err(JsError::new(
                    format!("Failed to convert '{}' to a valid PeerId", peer).as_str(),
                ))
            }
        }

        #[wasm_bindgen(js_name = removeAlias)]
        pub async fn _remove_alias(&self, alias: String) -> Result<(), JsError> {
            Ok(self.hopr.remove_alias(&alias).await)
        }

        #[wasm_bindgen(js_name = getAlias)]
        pub async fn _get_alias(&self, alias: String) -> Result<Option<String>, JsError> {
            Ok(self.hopr.get_alias(&alias).await.map(|p| p.to_string()))
        }

        #[wasm_bindgen(js_name = getAliases)]
        pub async fn _get_aliases(&self) -> js_sys::Map {
            let aliases = js_sys::Map::new();
            for (k, v) in self.hopr.get_aliases().await.into_iter() {
                aliases.set(&JsString::from(k), &JsString::from(v.to_string()));
            }

            aliases
        }

        #[wasm_bindgen(js_name = getState)]
        pub fn _status(&self) -> State {
            self.hopr.status()
        }

        #[wasm_bindgen(js_name = run)]
        pub async fn _run(&mut self) -> Result<HoprProcesses, JsError> {
            self.hopr
                .run()
                .await
                .map(|processes| HoprProcesses::new(processes))
                .map_err(JsError::from)
        }

        // p2p transport =========

        /// Fetch the PeerId behind this P2P transport
        #[wasm_bindgen(js_name = peerId)]
        pub fn _me(&self) -> JsString {
            JsString::from(self.hopr.me_peer_id().to_string())
        }

        /// Get the list of all announced public nodes in the network
        #[wasm_bindgen(js_name = getPublicNodes)]
        pub async fn _get_public_nodes(&self) -> Result<js_sys::Array, JsError> {
            Ok(self
                .hopr
                .get_public_nodes()
                .await
                .map(|v| v.into_iter().map(JsValue::from).collect())?)
        }

        /// Test whether the peer with PeerId is allowed to access the network
        #[wasm_bindgen(js_name = isAllowedToAccessNetwork)]
        pub async fn _is_allowed_to_access_network(&self, peer: JsString) -> bool {
            let x: String = peer.into();

            if let Some(peer) = core_transport::libp2p_identity::PeerId::from_str(&x).ok() {
                self.hopr.is_allowed_to_access_network(&peer).await
            } else {
                false
            }
        }

        /// Ping another node in the network based on the PeerId
        #[wasm_bindgen(js_name = ping)]
        pub async fn _ping(&self, peer: JsString) -> Result<Option<u32>, JsError> {
            let x: String = peer.into();
            if let Some(converted) = core_transport::libp2p_identity::PeerId::from_str(&x).ok() {
                Ok(self.hopr.ping(&converted).await?.map(|v| v.as_millis() as u32))
            } else {
                Ok(None)
            }
        }

        /// Send a message to another peer in the network
        ///
        /// @param msg message to send
        /// @param destination PeerId of the destination
        /// @param intermediatePath optional set path manually
        /// @param hops optional number of required intermediate nodes
        /// @param applicationTag optional tag identifying the sending application
        /// @returns hex representation of ack challenge
        #[wasm_bindgen(js_name = sendMessage)]
        pub async fn _send_message(
            &self,
            msg: Box<[u8]>,
            destination: JsString,
            intermediate_path: Option<Vec<JsString>>,
            hops: Option<u16>,
            application_tag: Option<u16>,
        ) -> Result<JsString, JsError> {
            let x: String = destination.into();
            if let Some(destination) = core_transport::libp2p_identity::PeerId::from_str(&x).ok() {
                let (path, hops) = {
                    if let Some(intermediate_path) = intermediate_path {
                        let full_path = intermediate_path
                            .iter()
                            .filter_map(|peer| {
                                let p: String = peer.into();
                                core_transport::libp2p_identity::PeerId::from_str(&p).ok()
                            })
                            .collect::<Vec<_>>();

                        if full_path.len() != intermediate_path.len() {
                            #[cfg(all(feature = "prometheus", not(test)))]
                            SimpleCounter::increment(&METRIC_MESSAGE_FAIL_COUNT);

                            return Err(JsError::new("send msg: some peers in the path are not valid peer ids"));
                        }

                        (Some(full_path), hops)
                    } else if let Some(hops) = hops {
                        (None, Some(hops))
                    } else {
                        #[cfg(all(feature = "prometheus", not(test)))]
                        SimpleCounter::increment(&METRIC_MESSAGE_FAIL_COUNT);

                        return Err(JsError::new(
                            "send msg: one of either hops or intermediate path must be specified",
                        ));
                    }
                };

                let result = self
                    .hopr
                    .send_message(msg, destination, path, hops, application_tag)
                    .await
                    .map(|v| JsString::from(v.to_hex()))
                    .map_err(JsError::from);

                result
            } else {
                // TODO: Should this really be counted?
                #[cfg(all(feature = "prometheus", not(test)))]
                SimpleCounter::increment(&METRIC_MESSAGE_FAIL_COUNT);

                return Err(JsError::new("send msg: invalid destination peer id supplied"));
            }
        }

        /// Attempts to aggregate all tickets in the given channel
        /// @param channelId id of the channel
        #[wasm_bindgen(js_name = aggregateTickets)]
        pub async fn _aggregate_tickets(&mut self, channel: &Hash) -> Result<(), JsError> {
            self.hopr.aggregate_tickets(channel).await.map_err(JsError::from)
        }

        /// List all multiaddresses announced
        #[wasm_bindgen(js_name = getLocalMultiAddresses)]
        pub fn _local_multiaddresses(&self) -> Vec<JsString> {
            self.hopr
                .local_multiaddresses()
                .into_iter()
                .map(|ma| JsString::from(ma.to_string()))
                .collect::<Vec<_>>()
        }

        /// List all multiaddresses on which the node is listening
        #[wasm_bindgen(js_name = getListeningMultiaddresses)]
        pub async fn _listening_multiaddresses(&self) -> js_sys::Array {
            js_sys::Array::from_iter(
                self.hopr
                    .listening_multiaddresses()
                    .await
                    .into_iter()
                    .map(|ma| JsString::from(ma.to_string())),
            )
        }

        /// List all multiaddresses observed for a PeerId
        #[wasm_bindgen(js_name = getObservedMultiaddresses)]
        pub async fn _network_observed_multiaddresses(&self, peer: JsString) -> js_sys::Array {
            let peer: String = peer.into();
            match core_transport::libp2p_identity::PeerId::from_str(&peer) {
                Ok(peer) => js_sys::Array::from_iter(
                    self.hopr
                        .network_observed_multiaddresses(&peer)
                        .await
                        .into_iter()
                        .map(|ma| JsString::from(ma.to_string())),
                ),
                Err(e) => {
                    warn!(
                        "Failed to parse peer id {}, cannot get multiaddresses announced to DHT: {}",
                        peer,
                        e.to_string()
                    );
                    js_sys::Array::new()
                }
            }
        }

        /// List all multiaddresses for this node announced to DHT
        #[wasm_bindgen(js_name = getMultiaddressesAnnouncedToDHT)]
        pub async fn _multiaddresses_announced_to_dht(&self, peer: JsString) -> js_sys::Array {
            let peer: String = peer.into();
            match core_transport::libp2p_identity::PeerId::from_str(&peer) {
                Ok(peer) => js_sys::Array::from_iter(
                    self.hopr
                        .multiaddresses_announced_to_dht(&peer)
                        .await
                        .into_iter()
                        .map(|ma| JsString::from(ma.to_string())),
                ),
                Err(e) => {
                    warn!(
                        "Failed to parse peer id {}, cannot get multiaddresses announced to DHT: {}",
                        peer,
                        e.to_string()
                    );
                    js_sys::Array::new()
                }
            }
        }

        // Network =========

        /// Get measured network health
        #[wasm_bindgen(js_name = networkHealth)]
        pub async fn _network_health(&self) -> WasmHealth {
            self.hopr.network_health().await.into()
        }

        /// Called whenever a peer is announced
        /// @param peer newly announced peer
        #[wasm_bindgen(js_name = onPeerAnnouncement)]
        pub async fn _on_peer_announcement(&self, peer: JsString, address: Address, multiaddresses: js_sys::Array) {
            let peer: String = peer.into();
            match core_transport::libp2p_identity::PeerId::from_str(&peer) {
                Ok(peer) => {
                    if &peer != &self.hopr.me_peer_id() {
                        let mas = multiaddresses
                            .to_vec()
                            .into_iter()
                            .filter_map(|v| {
                                let v: String = JsString::from(v).into();
                                Multiaddr::from_str(&v).ok()
                            })
                            .collect::<Vec<Multiaddr>>();

                        self.hopr.on_peer_announcement(peer, address, mas).await
                    }
                }
                Err(e) => {
                    warn!(
                        "Failed to parse peer id {}, cannot announce multiaddresses: {}",
                        peer,
                        e.to_string()
                    );
                }
            }
        }

        /// Unregister a peer from the network
        #[wasm_bindgen(js_name = unregister)]
        pub async fn _unregister(&self, peer: JsString) {
            let peer: String = peer.into();
            match core_transport::libp2p_identity::PeerId::from_str(&peer) {
                Ok(peer) => self.hopr.unregister(&peer).await,
                Err(e) => {
                    warn!(
                        "Failed to parse peer id {}, network ignores the unregister attempt: {}",
                        peer,
                        e.to_string()
                    );
                }
            }
        }

        /// Called whenever the network registry changes
        #[wasm_bindgen(js_name = onNetworkRegistryUpdate)]
        pub async fn _on_network_registry_update(&self, address: &Address, allowed: bool) {
            self.hopr.on_network_registry_update(address, allowed).await
        }

        /// List all peers connected to this
        #[wasm_bindgen(js_name = getConnectedPeers)]
        pub async fn _network_connected_peers(&self) -> js_sys::Array {
            js_sys::Array::from_iter(
                self.hopr
                    .network_connected_peers()
                    .await
                    .into_iter()
                    .map(|x| JsValue::from(x.to_base58())),
            )
        }

        /// Get all data collected from the network relevant for a PeerId
        #[wasm_bindgen(js_name = getPeerInfo)]
        pub async fn _network_peer_info(&self, peer: JsString) -> Option<core_transport::PeerStatus> {
            let peer: String = peer.into();
            match core_transport::libp2p_identity::PeerId::from_str(&peer) {
                Ok(peer) => self.hopr.network_peer_info(&peer).await,
                Err(e) => {
                    warn!(
                        "Failed to parse peer id {}, peer info unavailable: {}",
                        peer,
                        e.to_string()
                    );
                    None
                }
            }
        }

        // Ticket ========
        /// Get all tickets in a channel specified by Hash
        #[wasm_bindgen(js_name = getTickets)]
        pub async fn _tickets_in_channel(&self, channel: &Hash) -> Result<Array, JsError> {
            self.hopr
                .tickets_in_channel(channel)
                .await
                .map(|tickets| {
                    tickets
                        .into_iter()
                        .map(core_types::acknowledgement::wasm::AcknowledgedTicket::from)
                        .map(|at| at.ticket())
                        .map(JsValue::from)
                        .collect()
                })
                .map_err(JsError::from)
        }

        /// Get all tickets
        #[wasm_bindgen(js_name = getAllTickets)]
        pub async fn _all_tickets(&self) -> Result<Array, JsError> {
            self.hopr
                .all_tickets()
                .await
                .map(|tickets| {
                    tickets
                        .into_iter()
                        .map(core_types::channels::wasm::Ticket::from)
                        .map(JsValue::from)
                        .collect()
                })
                .map_err(JsError::from)
        }

        /// Get statistics for all tickets
        #[wasm_bindgen(js_name = getTicketStatistics)]
        pub async fn _ticket_statistics(&self) -> Result<TicketStatistics, JsError> {
            self.hopr.ticket_statistics().await.map_err(JsError::from)
        }

        // Chain =========
        #[wasm_bindgen(js_name = getEthereumAddress)]
        pub fn _me_onchain(&self) -> Address {
            self.hopr.me_onchain()
        }

        /// List of all accounts announced on the chain
        #[wasm_bindgen(js_name = getAccountsAnnouncedOnChain)]
        pub async fn _accounts_announced_on_chain(&self) -> Result<Array, JsError> {
            self.hopr
                .accounts_announced_on_chain()
                .await
                .map_err(JsError::from)
                .map(|v| v.into_iter().map(JsValue::from).collect())
        }

        /// Get the channel entry from Hash.
        /// @returns the channel entry of those two nodes
        #[wasm_bindgen(js_name = getChannelFromHash)]
        pub async fn _channel_from_hash(&self, channel: &Hash) -> Result<Option<ChannelEntry>, JsError> {
            self.hopr.channel_from_hash(channel).await.map_err(JsError::from)
        }

        /// Get the channel entry between source and destination node.
        /// @param src Address
        /// @param dest Address
        /// @returns the channel entry of those two nodes
        #[wasm_bindgen(js_name = getChannel)]
        pub async fn _channel(&self, src: &Address, dest: &Address) -> Result<ChannelEntry, JsError> {
            self.hopr.channel(src, dest).await.map_err(JsError::from)
        }

        /// List all channels open from a specified Address
        #[wasm_bindgen(js_name = getChannelsFrom)]
        pub async fn _channels_from(&self, src: &Address) -> Result<Array, JsError> {
            self.hopr
                .channels_from(src)
                .await
                .map_err(JsError::from)
                .map(|channels| channels.into_iter().map(JsValue::from).collect())
        }

        /// List all channels open to a specified address
        #[wasm_bindgen(js_name = getChannelsTo)]
        pub async fn _channels_to(&self, dest: &Address) -> Result<Array, JsError> {
            self.hopr
                .channels_to(dest)
                .await
                .map_err(JsError::from)
                .map(|channels| channels.into_iter().map(JsValue::from).collect())
        }

        /// List all channels
        #[wasm_bindgen(js_name = getAllChannels)]
        pub async fn _all_channels(&self) -> Result<Array, JsError> {
            self.hopr
                .all_channels()
                .await
                .map(|channels| channels.into_iter().map(JsValue::from).collect())
                .map_err(JsError::from)
        }

        /// Current safe allowance balance
        #[wasm_bindgen(js_name = getSafeAllowance)]
        pub async fn _safe_allowance(&self) -> Result<Balance, JsError> {
            self.hopr.safe_allowance().await.map_err(JsError::from)
        }

        #[wasm_bindgen]
        pub async fn withdraw(&self, recipient: &Address, amount: &Balance) -> Result<Hash, JsError> {
            self.hopr
                .withdraw(recipient.clone(), amount.clone())
                .await
                .map_err(JsError::from)
        }

        #[wasm_bindgen(js_name = onOwnChannelUpdated)]
        pub async fn _on_own_channel_updated(&self, channel: &ChannelEntry) {
            self.hopr.on_own_channel_updated(channel).await
        }

        #[wasm_bindgen(js_name = onTicketRedeemed)]
        pub async fn _on_ticket_redeemed(&self, channel: &ChannelEntry, value: &Balance) {
            self.hopr.on_ticket_redeemed(channel, &value).await
        }

        #[wasm_bindgen(js_name = openChannel)]
        pub async fn open_channel(
            &self,
            destination: &Address,
            amount: &Balance,
        ) -> Result<OpenChannelResult, JsError> {
            self.hopr.open_channel(destination, amount).await.map_err(JsError::from)
        }

        #[wasm_bindgen(js_name = fundChannel)]
        pub async fn fund_channel(&self, channel_id: &Hash, amount: &Balance) -> Result<Hash, JsError> {
            self.hopr.fund_channel(channel_id, amount).await.map_err(JsError::from)
        }

        #[wasm_bindgen(js_name = closeChannel)]
        pub async fn _close_channel(
            &self,
            counterparty: &Address,
            direction: core_types::channels::ChannelDirection,
            redeem_before_close: bool,
        ) -> Result<CloseChannelResult, JsError> {
            self.hopr
                .close_channel(counterparty, direction, redeem_before_close)
                .await
                .map_err(JsError::from)
        }

        #[wasm_bindgen(js_name = redeemAllTickets)]
        pub async fn _redeem_all_tickets(&self, only_aggregated: bool) -> Result<(), JsError> {
            self.hopr
                .redeem_all_tickets(only_aggregated)
                .await
                .map_err(JsError::from)
        }

        #[wasm_bindgen(js_name = redeemTicketsWithCounterparty)]
        pub async fn _redeem_tickets_with_counterparty(
            &self,
            counterparty: &Address,
            only_aggregated: bool,
        ) -> Result<(), JsError> {
            self.hopr
                .redeem_tickets_with_counterparty(counterparty, only_aggregated)
                .await
                .map_err(JsError::from)
        }

        #[wasm_bindgen(js_name = redeemTicketsInChannel)]
        pub async fn _redeem_tickets_in_channel(&self, channel: &Hash, only_aggregated: bool) -> Result<(), JsError> {
            self.hopr
                .redeem_tickets_in_channel(&channel, only_aggregated)
                .await
                .map_err(JsError::from)
        }

        #[wasm_bindgen(js_name = redeemTicket)]
        pub async fn _redeem_ticket(&self, ack_ticket: &AcknowledgedTicket) -> Result<(), JsError> {
            self.hopr.redeem_ticket(ack_ticket.into()).await.map_err(JsError::from)
        }

        #[wasm_bindgen(js_name = smartContractInfo)]
        pub fn _smart_contract_info(&self) -> Result<ChainConfiguration, JsError> {
            self.chain_query
                .smartContractInfo()
                .map_err(|e| JsError::new(&format!("Fail on calling smartContractInfo: {:?}", e)))
                .and_then(|v| {
                    serde_wasm_bindgen::from_value::<ChainConfiguration>(v)
                        .map_err(|e| JsError::new(&format!("Fail on calling smartContractInfo: {:?}", e)))
                })
        }

        #[wasm_bindgen(js_name = getBalance)]
        pub async fn _balance(&self) -> Result<Balance, JsError> {
            match self.chain_query.getBalance().await {
                Ok(balance) => Ok(Balance::from_str(balance.as_string().unwrap().as_str())
                    .map_err(|_| JsError::new("Error converting balance from string"))?),
                Err(e) => Err(JsError::new(format!("Encountered issue: {:?}", e).as_str())),
            }
        }

        #[wasm_bindgen(js_name = getSafeBalance)]
        pub async fn _safe_balance(&self) -> Result<Balance, JsError> {
            match self.chain_query.getSafeBalance().await {
                Ok(balance) => Ok(Balance::from_str(balance.as_string().unwrap().as_str())
                    .map_err(|_| JsError::new("Error converting balance from string"))?),
                Err(e) => Err(JsError::new(format!("Encountered issue: {:?}", e).as_str())),
            }
        }

        #[wasm_bindgen(js_name = getNativeBalance)]
        pub async fn _native_balance(&self) -> Result<Balance, JsError> {
            match self.chain_query.getNativeBalance().await {
                Ok(balance) => Ok(Balance::from_str(balance.as_string().unwrap().as_str())
                    .map_err(|_| JsError::new("Error converting balance from string"))?),
                Err(e) => Err(JsError::new(format!("Encountered issue: {:?}", e).as_str())),
            }
        }

        #[wasm_bindgen(js_name = getSafeNativeBalance)]
        pub async fn _safe_native_balance(&self) -> Result<Balance, JsError> {
            match self.chain_query.getSafeNativeBalance().await {
                Ok(balance) => Ok(Balance::from_str(balance.as_string().unwrap().as_str())
                    .map_err(|_| JsError::new("Error converting balance from string"))?),
                Err(e) => Err(JsError::new(format!("Encountered issue: {:?}", e).as_str())),
            }
        }

        #[wasm_bindgen(js_name = peerIdToChainKey)]
        pub async fn _peerid_to_chain_key(&self, peer_id: JsString) -> Result<Option<Address>, JsError> {
            let peer_id: String = peer_id.into();
            let peer_id = PeerId::from_str(&peer_id)?;

            Ok(self.hopr.peerid_to_chain_key(&peer_id).await?)
        }

        #[wasm_bindgen(js_name = chainKeyToPeerId)]
        pub async fn _chain_key_to_peerid(&self, address: &Address) -> Result<Option<String>, JsError> {
            Ok(self
                .hopr
                .chain_key_to_peerid(address)
                .await
                .map(|v| v.map(|p| p.to_string()))?)
        }

        /// Emitter API: on
        #[wasm_bindgen(js_name = chainConfig)]
        pub fn _chain_config(&self) -> ChainNetworkConfig {
            self.hopr.chain_config()
        }

        /// Emitter API: on
        #[wasm_bindgen(js_name = on)]
        pub fn _on(&self, event: JsString, callback: js_sys::Function) {
            self.msg_emitter.delegate_on(event, callback)
        }
    }

    #[wasm_bindgen(getter_with_clone)]
    pub struct OpenChannelResult {
        pub tx_hash: Hash,
        pub channel_id: Hash,
    }

    #[wasm_bindgen(getter_with_clone)]
    pub struct CloseChannelResult {
        pub tx_hash: Hash,
        pub status: core_types::channels::ChannelStatus,
    }

    #[wasm_bindgen]
    impl WasmHealth {
        #[wasm_bindgen(js_name = green)]
        pub fn _green() -> Self {
            Self { h: Health::Green }
        }

        #[wasm_bindgen]
        pub fn unwrap(&self) -> Health {
            self.h
        }
    }
}

/// Wrapper object necessary for async wasm function return value
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
pub struct WasmHealth {
    h: Health,
}

impl From<Health> for WasmHealth {
    fn from(value: Health) -> Self {
        Self { h: value }
    }
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use utils_log::logger::wasm::JsLogger;
    use utils_misc::utils::wasm::JsResult;
    use wasm_bindgen::prelude::wasm_bindgen;

    static LOGGER: JsLogger = JsLogger {};

    #[allow(dead_code)]
    #[wasm_bindgen]
    pub fn hopr_lib_initialize_crate() {
        let _ = JsLogger::install(&LOGGER, None);

        // When the `console_error_panic_hook` feature is enabled, we can call the
        // `set_panic_hook` function at least once during initialization, and then
        // we will get better error messages if our code ever panics.
        //
        // For more details see
        // https://github.com/rustwasm/console_error_panic_hook#readme
        #[cfg(feature = "console_error_panic_hook")]
        console_error_panic_hook::set_once();
    }

    #[wasm_bindgen]
    pub fn hopr_lib_gather_metrics() -> JsResult<String> {
        utils_metrics::metrics::wasm::gather_all_metrics()
    }

    // When the `wee_alloc` feature is enabled, use `wee_alloc` as the global allocator.
    #[cfg(feature = "wee_alloc")]
    #[global_allocator]
    static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
}
