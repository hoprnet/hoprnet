use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

use bimap::BiMap;
use dashmap::DashMap;
use futures::stream::{self, BoxStream};
use hopr_api::{
    chain::*,
    db::{ChannelTicketStatistics, HoprDbTicketOperations, TicketMarker, TicketSelector},
    types::{
        crypto::prelude::{Hash, OffchainPublicKey},
        internal::prelude::*,
        primitive::{
            balance::{Balance, Currency, HoprBalance},
            prelude::{Address, KeyIdMapping},
        },
    },
};

/// Error type for test stubs — never constructed, exists only to satisfy trait bounds.
#[derive(Debug, thiserror::Error)]
#[error("stub error")]
pub struct StubError;

// ---------------------------------------------------------------------------
// StubKeyIdMapper
// ---------------------------------------------------------------------------

/// Bidirectional mapping between [`HoprKeyIdent`] and [`OffchainPublicKey`].
///
/// Extracted from the test helper in `transport/path/src/planner.rs`.
#[derive(Clone)]
pub struct StubKeyIdMapper {
    map: Arc<BiMap<OffchainPublicKey, HoprKeyIdent>>,
}

impl KeyIdMapping<HoprKeyIdent, OffchainPublicKey> for StubKeyIdMapper {
    fn map_key_to_id(&self, key: &OffchainPublicKey) -> Option<HoprKeyIdent> {
        self.map.get_by_left(key).copied()
    }

    fn map_id_to_public(&self, id: &HoprKeyIdent) -> Option<OffchainPublicKey> {
        self.map.get_by_right(id).copied()
    }
}

// ---------------------------------------------------------------------------
// StubChainApi
// ---------------------------------------------------------------------------

/// Lightweight in-memory stub implementing the chain-API traits needed by
/// `HoprEncoder` / `HoprDecoder` / `HoprTicketProcessor`.
///
/// All lookups are simple BiMap or Vec scans — no async I/O, no database.
#[derive(Clone)]
pub struct StubChainApi {
    me: Address,
    key_addr_map: BiMap<OffchainPublicKey, Address>,
    channels: Vec<ChannelEntry>,
    id_mapper: StubKeyIdMapper,
    ticket_price: HoprBalance,
    win_prob: WinningProbability,
}

/// Builder for [`StubChainApi`].
pub struct StubChainApiBuilder {
    me: Option<Address>,
    key_addr_map: BiMap<OffchainPublicKey, Address>,
    key_id_map: BiMap<OffchainPublicKey, HoprKeyIdent>,
    channels: Vec<ChannelEntry>,
    ticket_price: HoprBalance,
    win_prob: WinningProbability,
    next_key_id: u32,
}

impl Default for StubChainApiBuilder {
    fn default() -> Self {
        Self {
            me: None,
            key_addr_map: BiMap::new(),
            key_id_map: BiMap::new(),
            channels: Vec::new(),
            ticket_price: HoprBalance::zero(),
            win_prob: WinningProbability::ALWAYS,
            next_key_id: 0,
        }
    }
}

impl StubChainApiBuilder {
    /// Sets the "self" address of this stub node.
    pub fn me(mut self, addr: Address) -> Self {
        self.me = Some(addr);
        self
    }

    /// Registers a single peer (offchain key ↔ chain address + key-id mapping).
    pub fn peer(mut self, offchain: &OffchainPublicKey, chain_addr: Address) -> Self {
        self.key_addr_map.insert(*offchain, chain_addr);
        self.key_id_map.insert(*offchain, self.next_key_id.into());
        self.next_key_id += 1;
        self
    }

    /// Adds a channel entry.
    pub fn channel(mut self, entry: ChannelEntry) -> Self {
        self.channels.push(entry);
        self
    }

    /// Sets the default outgoing ticket price.
    pub fn ticket_price(mut self, price: HoprBalance) -> Self {
        self.ticket_price = price;
        self
    }

    /// Sets the default winning probability.
    pub fn win_prob(mut self, prob: WinningProbability) -> Self {
        self.win_prob = prob;
        self
    }

    /// Finalizes the builder into a [`StubChainApi`].
    ///
    /// # Panics
    /// Panics if `me` was not set.
    pub fn build(self) -> StubChainApi {
        StubChainApi {
            me: self.me.expect("me address must be set"),
            key_addr_map: self.key_addr_map,
            channels: self.channels,
            id_mapper: StubKeyIdMapper {
                map: Arc::new(self.key_id_map),
            },
            ticket_price: self.ticket_price,
            win_prob: self.win_prob,
        }
    }
}

impl StubChainApi {
    /// Returns a new [`StubChainApiBuilder`].
    pub fn builder() -> StubChainApiBuilder {
        StubChainApiBuilder::default()
    }

    /// Returns the key-address mapping.
    pub fn key_addr_map(&self) -> &BiMap<OffchainPublicKey, Address> {
        &self.key_addr_map
    }

    /// Returns the registered channels.
    pub fn channels(&self) -> &[ChannelEntry] {
        &self.channels
    }
}

// -- ChainKeyOperations -----------------------------------------------------

#[async_trait::async_trait]
impl ChainKeyOperations for StubChainApi {
    type Error = StubError;
    type Mapper = StubKeyIdMapper;

    async fn chain_key_to_packet_key(&self, chain: &Address) -> Result<Option<OffchainPublicKey>, Self::Error> {
        Ok(self.key_addr_map.get_by_right(chain).copied())
    }

    async fn packet_key_to_chain_key(&self, packet: &OffchainPublicKey) -> Result<Option<Address>, Self::Error> {
        Ok(self.key_addr_map.get_by_left(packet).copied())
    }

    fn key_id_mapper_ref(&self) -> &Self::Mapper {
        &self.id_mapper
    }
}

// -- ChainReadChannelOperations ---------------------------------------------

#[async_trait::async_trait]
impl ChainReadChannelOperations for StubChainApi {
    type Error = StubError;

    fn me(&self) -> &Address {
        &self.me
    }

    async fn channel_by_id(&self, channel_id: &ChannelId) -> Result<Option<ChannelEntry>, Self::Error> {
        Ok(self.channels.iter().find(|c| c.get_id() == channel_id).cloned())
    }

    async fn stream_channels<'a>(
        &'a self,
        _selector: ChannelSelector,
    ) -> Result<BoxStream<'a, ChannelEntry>, Self::Error> {
        Ok(Box::pin(stream::iter(self.channels.clone())))
    }
}

// -- ChainValues ------------------------------------------------------------

#[async_trait::async_trait]
impl ChainValues for StubChainApi {
    type Error = StubError;

    async fn balance<C: Currency, A: Into<Address> + Send>(&self, _address: A) -> Result<Balance<C>, Self::Error> {
        Ok(Balance::new_base(1_000_000))
    }

    async fn domain_separators(&self) -> Result<DomainSeparators, Self::Error> {
        Ok(DomainSeparators {
            ledger: Hash::default(),
            safe_registry: Hash::default(),
            channel: Hash::default(),
        })
    }

    async fn minimum_incoming_ticket_win_prob(&self) -> Result<WinningProbability, Self::Error> {
        Ok(self.win_prob)
    }

    async fn minimum_ticket_price(&self) -> Result<HoprBalance, Self::Error> {
        Ok(self.ticket_price)
    }

    async fn key_binding_fee(&self) -> Result<HoprBalance, Self::Error> {
        Ok(HoprBalance::zero())
    }

    async fn channel_closure_notice_period(&self) -> Result<std::time::Duration, Self::Error> {
        Ok(std::time::Duration::from_secs(300))
    }

    async fn chain_info(&self) -> Result<ChainInfo, Self::Error> {
        Ok(ChainInfo {
            chain_id: 100,
            hopr_network_name: "stub".to_string(),
            contract_addresses: ContractAddresses::default(),
        })
    }

    // outgoing_ticket_values: uses default impl that calls minimum_ticket_price + minimum_incoming_ticket_win_prob
}

// ---------------------------------------------------------------------------
// StubTicketDb
// ---------------------------------------------------------------------------

/// Minimal in-memory stub for [`HoprDbTicketOperations`].
///
/// Only `get_or_create_outgoing_ticket_index` and `update_outgoing_ticket_index`
/// maintain real state (an atomic counter per channel+epoch). All other methods
/// return empty / zero defaults.
#[derive(Clone, Default)]
pub struct StubTicketDb {
    indices: Arc<DashMap<(ChannelId, u32), Arc<AtomicU64>>>,
}

#[async_trait::async_trait]
impl HoprDbTicketOperations for StubTicketDb {
    type Error = StubError;

    async fn stream_tickets<'c, S: Into<TicketSelector>, I: IntoIterator<Item = S> + Send>(
        &'c self,
        _selectors: I,
    ) -> Result<BoxStream<'c, RedeemableTicket>, Self::Error> {
        Ok(Box::pin(stream::empty()))
    }

    async fn insert_ticket(&self, _ticket: RedeemableTicket) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn mark_tickets_as<S: Into<TicketSelector> + Send, I: IntoIterator<Item = S> + Send>(
        &self,
        _selectors: I,
        _mark_as: TicketMarker,
    ) -> Result<usize, Self::Error> {
        Ok(0)
    }

    async fn mark_unsaved_ticket_rejected(&self, _issuer: &Address, _ticket: &Ticket) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn update_ticket_states_and_fetch<'a, S: Into<TicketSelector>, I: IntoIterator<Item = S> + Send>(
        &'a self,
        _selectors: I,
        _new_state: AcknowledgedTicketStatus,
    ) -> Result<BoxStream<'a, RedeemableTicket>, Self::Error> {
        Ok(Box::pin(stream::empty()))
    }

    async fn update_ticket_states<S: Into<TicketSelector>, I: IntoIterator<Item = S> + Send>(
        &self,
        _selectors: I,
        _new_state: AcknowledgedTicketStatus,
    ) -> Result<usize, Self::Error> {
        Ok(0)
    }

    async fn get_ticket_statistics(
        &self,
        _channel_id: Option<ChannelId>,
    ) -> Result<ChannelTicketStatistics, Self::Error> {
        Ok(ChannelTicketStatistics::default())
    }

    async fn reset_ticket_statistics(&self) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn get_tickets_value(&self, _selector: TicketSelector) -> Result<HoprBalance, Self::Error> {
        Ok(HoprBalance::zero())
    }

    async fn get_or_create_outgoing_ticket_index(
        &self,
        channel_id: &ChannelId,
        epoch: u32,
        current_index: u64,
    ) -> Result<Option<u64>, Self::Error> {
        let key = (*channel_id, epoch);
        let entry = self
            .indices
            .entry(key)
            .or_insert_with(|| Arc::new(AtomicU64::new(current_index)));
        let idx = entry.value().fetch_add(1, Ordering::Relaxed);
        // Return None on first creation (matching the real DB behaviour),
        // otherwise return the current index.
        if idx == current_index { Ok(None) } else { Ok(Some(idx)) }
    }

    async fn update_outgoing_ticket_index(
        &self,
        channel_id: &ChannelId,
        epoch: u32,
        index: u64,
    ) -> Result<(), Self::Error> {
        let key = (*channel_id, epoch);
        if let Some(entry) = self.indices.get(&key) {
            entry.value().store(index, Ordering::Relaxed);
        }
        Ok(())
    }

    async fn remove_outgoing_ticket_index(&self, channel_id: &ChannelId, epoch: u32) -> Result<(), Self::Error> {
        self.indices.remove(&(*channel_id, epoch));
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// StubPathResolver
// ---------------------------------------------------------------------------

/// Minimal stub for [`PathAddressResolver`], using the same BiMap lookups
/// as the `MockPathResolver` in `transport/protocol/tests/common/mod.rs`.
pub struct StubPathResolver {
    key_addr_map: BiMap<OffchainPublicKey, Address>,
    channels: Vec<ChannelEntry>,
}

impl StubPathResolver {
    /// Creates a new resolver sharing key/channel data with the given [`StubChainApi`].
    pub fn from_chain_api(api: &StubChainApi) -> Self {
        Self {
            key_addr_map: api.key_addr_map().clone(),
            channels: api.channels().to_vec(),
        }
    }
}

#[async_trait::async_trait]
impl PathAddressResolver for StubPathResolver {
    async fn resolve_transport_address(
        &self,
        address: &Address,
    ) -> Result<Option<OffchainPublicKey>, hopr_api::types::internal::errors::PathError> {
        Ok(self.key_addr_map.get_by_right(address).copied())
    }

    async fn resolve_chain_address(
        &self,
        key: &OffchainPublicKey,
    ) -> Result<Option<Address>, hopr_api::types::internal::errors::PathError> {
        Ok(self.key_addr_map.get_by_left(key).copied())
    }

    async fn get_channel(
        &self,
        src: &Address,
        dst: &Address,
    ) -> Result<Option<ChannelEntry>, hopr_api::types::internal::errors::PathError> {
        Ok(self
            .channels
            .iter()
            .find(|c| &c.source == src && &c.destination == dst)
            .cloned())
    }
}
