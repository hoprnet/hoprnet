use hopr_api::{
    chain::{ChainKeyOperations, ChainReadChannelOperations, ChainValues},
    db::{HoprDbProtocolOperations, IncomingPacket, IncomingPacketError, OutgoingPacket},
};
pub use hopr_crypto_packet::errors::PacketError;
use hopr_crypto_types::prelude::*;
use hopr_internal_types::prelude::*;
use hopr_network_types::prelude::ResolvedTransportRouting;
use hopr_primitive_types::prelude::*;
use hopr_protocol_app::prelude::*;

#[async_trait::async_trait]
pub trait PacketWrapping {
    type Input;
    type Error;

    async fn send(
        &self,
        data: ApplicationDataOut,
        routing: ResolvedTransportRouting,
    ) -> Result<OutgoingPacket, Self::Error>;
}

#[async_trait::async_trait]
pub trait PacketUnwrapping {
    type Packet;

    type Error;

    async fn recv(
        &self,
        peer: OffchainPublicKey,
        data: Box<[u8]>,
    ) -> Result<Self::Packet, IncomingPacketError<Self::Error>>;
}

/// Implements protocol acknowledgement logic for msg packets
#[derive(Debug, Clone)]
pub struct PacketProcessor<Db, R> {
    db: Db,
    resolver: R,
    cfg: PacketInteractionConfig,
}

#[async_trait::async_trait]
impl<Db, R> PacketWrapping for PacketProcessor<Db, R>
where
    Db: HoprDbProtocolOperations + Send + Sync + Clone,
    R: ChainReadChannelOperations + ChainKeyOperations + ChainValues + Send + Sync,
{
    type Error = Db::Error;
    type Input = ApplicationDataOut;

    #[tracing::instrument(level = "trace", skip(self, data), ret(Debug), err)]
    async fn send(
        &self,
        data: ApplicationDataOut,
        routing: ResolvedTransportRouting,
    ) -> Result<OutgoingPacket, Self::Error> {
        self.db
            .to_send(
                data.data.to_bytes(),
                routing,
                self.cfg.outgoing_ticket_win_prob,
                self.cfg.outgoing_ticket_price,
                data.packet_info.unwrap_or_default().signals_to_destination,
                &self.resolver,
            )
            .await
    }
}

#[async_trait::async_trait]
impl<Db, R> PacketUnwrapping for PacketProcessor<Db, R>
where
    Db: HoprDbProtocolOperations + Send + Sync + Clone,
    R: ChainReadChannelOperations + ChainKeyOperations + ChainValues + Send + Sync,
{
    type Error = Db::Error;
    type Packet = IncomingPacket;

    #[tracing::instrument(level = "trace", skip(self, data))]
    async fn recv(
        &self,
        previous_hop: OffchainPublicKey,
        data: Box<[u8]>,
    ) -> Result<Self::Packet, IncomingPacketError<Self::Error>> {
        self.db
            .from_recv(
                data,
                &self.cfg.packet_keypair,
                previous_hop,
                self.cfg.outgoing_ticket_win_prob,
                self.cfg.outgoing_ticket_price,
                &self.resolver,
            )
            .await
    }
}

impl<Db, R> PacketProcessor<Db, R>
where
    Db: HoprDbProtocolOperations + Send + Sync + Clone,
    R: ChainReadChannelOperations + ChainKeyOperations + ChainValues + Send + Sync,
{
    /// Creates a new instance given the DB and configuration.
    pub fn new(db: Db, resolver: R, cfg: PacketInteractionConfig) -> Self {
        Self { db, resolver, cfg }
    }
}

/// Configuration parameters for the packet interaction.
#[derive(Clone, Debug)]
pub struct PacketInteractionConfig {
    pub packet_keypair: OffchainKeypair,
    pub outgoing_ticket_win_prob: Option<WinningProbability>,
    pub outgoing_ticket_price: Option<HoprBalance>,
}
