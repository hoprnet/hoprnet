/// Lightweight in-memory test doubles for `HoprTransport` trait bounds.
///
/// These implement the minimum required trait surface to construct
/// and test `HoprTransport` without real database, chain, or network infra.
use std::{collections::HashSet, time::Duration};

use async_trait::async_trait;
use futures::stream::{self, BoxStream};
use hopr_api::{
    Multiaddr, PeerId,
    chain::*,
    db::*,
    network::{Health, traits::NetworkView},
    types::{
        crypto::prelude::{ChainKeypair, Keypair, OffchainKeypair, OffchainPublicKey},
        internal::prelude::*,
        primitive::{
            balance::{Balance, Currency, HoprBalance},
            prelude::{Address, KeyIdMapping, KeyIdent},
        },
    },
};

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, thiserror::Error)]
#[error("stub: {0}")]
pub struct StubError(pub String);

// ---------------------------------------------------------------------------
// StubChain — satisfies Chain trait bounds
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct StubChain {
    me_addr: Address,
    me_offchain: OffchainPublicKey,
    mapper: StubKeyMapper,
}

impl StubChain {
    pub fn new(offchain: &OffchainKeypair, chain: &ChainKeypair) -> Self {
        Self {
            me_addr: chain.public().to_address(),
            me_offchain: *offchain.public(),
            mapper: StubKeyMapper,
        }
    }
}

#[async_trait]
impl ChainReadChannelOperations for StubChain {
    type Error = StubError;

    fn me(&self) -> &Address {
        &self.me_addr
    }

    async fn channel_by_id(&self, _channel_id: &ChannelId) -> Result<Option<ChannelEntry>, Self::Error> {
        Ok(None)
    }

    async fn stream_channels<'a>(
        &'a self,
        _selector: ChannelSelector,
    ) -> Result<BoxStream<'a, ChannelEntry>, Self::Error> {
        Ok(Box::pin(stream::empty()))
    }
}

#[async_trait]
impl ChainReadAccountOperations for StubChain {
    type Error = StubError;

    async fn stream_accounts<'a>(
        &'a self,
        _selector: AccountSelector,
    ) -> Result<BoxStream<'a, AccountEntry>, Self::Error> {
        Ok(Box::pin(stream::empty()))
    }

    async fn count_accounts(&self, _selector: AccountSelector) -> Result<usize, Self::Error> {
        Ok(0)
    }

    async fn await_key_binding(
        &self,
        _offchain_key: &OffchainPublicKey,
        _timeout: Duration,
    ) -> Result<AccountEntry, Self::Error> {
        Err(StubError("not implemented".into()))
    }
}

#[derive(Debug, Clone)]
pub struct StubKeyMapper;

impl KeyIdMapping<KeyIdent, OffchainPublicKey> for StubKeyMapper {
    fn map_key_to_id(&self, _key: &OffchainPublicKey) -> Option<KeyIdent> {
        None
    }

    fn map_id_to_public(&self, _id: &KeyIdent) -> Option<OffchainPublicKey> {
        None
    }
}

#[async_trait]
impl ChainKeyOperations for StubChain {
    type Error = StubError;
    type Mapper = StubKeyMapper;

    async fn chain_key_to_packet_key(&self, chain: &Address) -> Result<Option<OffchainPublicKey>, Self::Error> {
        Ok(if chain == &self.me_addr {
            Some(self.me_offchain)
        } else {
            None
        })
    }

    async fn packet_key_to_chain_key(&self, packet: &OffchainPublicKey) -> Result<Option<Address>, Self::Error> {
        Ok(if packet == &self.me_offchain {
            Some(self.me_addr)
        } else {
            None
        })
    }

    fn key_id_mapper_ref(&self) -> &Self::Mapper {
        &self.mapper
    }
}

#[async_trait]
impl ChainValues for StubChain {
    type Error = StubError;

    async fn balance<C: Currency, A: Into<Address> + Send>(&self, _address: A) -> Result<Balance<C>, Self::Error> {
        Ok(Balance::zero())
    }

    async fn domain_separators(&self) -> Result<DomainSeparators, Self::Error> {
        Ok(DomainSeparators {
            ledger: hopr_api::types::crypto::types::Hash::default(),
            safe_registry: hopr_api::types::crypto::types::Hash::default(),
            channel: hopr_api::types::crypto::types::Hash::default(),
        })
    }

    async fn minimum_incoming_ticket_win_prob(&self) -> Result<WinningProbability, Self::Error> {
        Ok(WinningProbability::ALWAYS)
    }

    async fn minimum_ticket_price(&self) -> Result<HoprBalance, Self::Error> {
        Ok(HoprBalance::zero())
    }

    async fn key_binding_fee(&self) -> Result<HoprBalance, Self::Error> {
        Ok(HoprBalance::zero())
    }

    async fn channel_closure_notice_period(&self) -> Result<Duration, Self::Error> {
        Ok(Duration::from_secs(60))
    }

    async fn chain_info(&self) -> Result<ChainInfo, Self::Error> {
        Ok(ChainInfo {
            chain_id: 100,
            hopr_network_name: "stub-test".into(),
            contract_addresses: Default::default(),
        })
    }
}

// ---------------------------------------------------------------------------
// StubDb — satisfies Db trait bounds
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct StubDb;

#[async_trait]
impl HoprDbTicketOperations for StubDb {
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
        _channel_id: &ChannelId,
        _epoch: u32,
        _current_index: u64,
    ) -> Result<Option<u64>, Self::Error> {
        Ok(None)
    }

    async fn update_outgoing_ticket_index(
        &self,
        _channel_id: &ChannelId,
        _epoch: u32,
        _index: u64,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn remove_outgoing_ticket_index(&self, _channel_id: &ChannelId, _epoch: u32) -> Result<(), Self::Error> {
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// StubNet — satisfies Net trait bounds (never used before run())
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct StubNet;

impl NetworkView for StubNet {
    fn listening_as(&self) -> HashSet<Multiaddr> {
        HashSet::new()
    }

    fn multiaddress_of(&self, _peer: &PeerId) -> Option<HashSet<Multiaddr>> {
        None
    }

    fn discovered_peers(&self) -> HashSet<PeerId> {
        HashSet::new()
    }

    fn connected_peers(&self) -> HashSet<PeerId> {
        HashSet::new()
    }

    fn is_connected(&self, _peer: &PeerId) -> bool {
        false
    }

    fn health(&self) -> Health {
        Health::Red
    }
}

#[async_trait]
impl hopr_api::network::traits::NetworkStreamControl for StubNet {
    fn accept(
        self,
    ) -> Result<
        impl futures::Stream<Item = (PeerId, impl futures::AsyncRead + futures::AsyncWrite + Send)> + Send,
        impl std::error::Error,
    > {
        Ok::<_, StubError>(stream::empty::<(PeerId, futures::io::Cursor<Vec<u8>>)>())
    }

    async fn open(
        self,
        _peer: PeerId,
    ) -> Result<impl futures::AsyncRead + futures::AsyncWrite + Send, impl std::error::Error> {
        Err::<futures::io::Cursor<Vec<u8>>, _>(StubError("stub cannot open".into()))
    }
}
