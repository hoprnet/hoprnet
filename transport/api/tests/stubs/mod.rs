/// Lightweight in-memory test doubles for `HoprTransport` trait bounds.
///
/// These implement the minimum required trait surface to construct
/// and test `HoprTransport` without real database, chain, or network infra.
use std::{collections::HashSet, time::Duration};

use async_trait::async_trait;
use bimap::BiMap;
use futures::stream::{self, BoxStream};
use hopr_api::{
    Multiaddr, PeerId,
    chain::*,
    network::{Health, traits::NetworkView},
    types::{
        crypto::prelude::{ChainKeypair, Keypair, OffchainKeypair, OffchainPublicKey},
        primitive::{
            balance::{Balance, Currency, HoprBalance},
            prelude::{Address, KeyIdMapping, KeyIdent},
        },
    },
};

/// Stub error type for test doubles.
#[derive(Debug, Clone, thiserror::Error)]
#[error("stub: {0}")]
pub struct StubError(pub String);

/// Stub chain connector satisfying `Chain` trait bounds.
#[derive(Debug, Clone)]
pub struct StubChain {
    me_addr: Address,
    keys: BiMap<Address, OffchainPublicKey>,
    mapper: StubKeyMapper,
}

impl StubChain {
    pub fn new(offchain: &OffchainKeypair, chain: &ChainKeypair) -> Self {
        let addr = chain.public().to_address();
        let mut keys = BiMap::new();
        keys.insert(addr, *offchain.public());
        Self {
            me_addr: addr,
            keys,
            mapper: StubKeyMapper,
        }
    }
}

impl ChainReadChannelOperations for StubChain {
    type Error = StubError;

    fn me(&self) -> &Address {
        &self.me_addr
    }

    fn channel_by_id(&self, _channel_id: &ChannelId) -> Result<Option<ChannelEntry>, Self::Error> {
        Ok(None)
    }

    fn stream_channels<'a>(&'a self, _selector: ChannelSelector) -> Result<BoxStream<'a, ChannelEntry>, Self::Error> {
        Ok(Box::pin(stream::empty()))
    }
}

#[async_trait]
impl ChainReadAccountOperations for StubChain {
    type Error = StubError;

    fn stream_accounts<'a>(&'a self, _selector: AccountSelector) -> Result<BoxStream<'a, AccountEntry>, Self::Error> {
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

impl ChainReadTicketOperations for StubChain {
    type Error = StubError;

    fn outgoing_ticket_values(
        &self,
        configured_wp: Option<WinningProbability>,
        configured_price: Option<HoprBalance>,
    ) -> Result<(WinningProbability, HoprBalance), Self::Error> {
        Ok((
            configured_wp.unwrap_or(WinningProbability::ALWAYS),
            configured_price.unwrap_or(1.into()),
        ))
    }

    fn incoming_ticket_values(&self) -> Result<(WinningProbability, HoprBalance), Self::Error> {
        Ok((WinningProbability::ALWAYS, 1.into()))
    }
}

#[async_trait]
impl ChainWriteTicketOperations for StubChain {
    type Error = StubError;

    async fn redeem_ticket<'a>(
        &'a self,
        ticket: RedeemableTicket,
    ) -> Result<
        futures::future::BoxFuture<'a, Result<(VerifiedTicket, ChainReceipt), TicketRedeemError<Self::Error>>>,
        TicketRedeemError<Self::Error>,
    > {
        Err(TicketRedeemError::ProcessingError(
            ticket.ticket,
            StubError("stubs do not redeem tickets".into()),
        ))
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

impl ChainKeyOperations for StubChain {
    type Error = StubError;
    type Mapper = StubKeyMapper;

    fn chain_key_to_packet_key(&self, chain: &Address) -> Result<Option<OffchainPublicKey>, Self::Error> {
        Ok(self.keys.get_by_left(chain).copied())
    }

    fn packet_key_to_chain_key(&self, packet: &OffchainPublicKey) -> Result<Option<Address>, Self::Error> {
        Ok(self.keys.get_by_right(packet).copied())
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

/// Stub network view satisfying `Net` trait bounds (never used before `run()`).
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
