use futures::{Sink, SinkExt, future::Either, pin_mut};
use hopr_async_runtime::prelude::sleep;
pub use hopr_crypto_packet::errors::PacketError;
use hopr_crypto_packet::errors::{PacketError::TransportError, Result};
use hopr_crypto_types::prelude::*;
use hopr_db_api::{
    prelude::HoprDbProtocolOperations,
    protocol::{IncomingPacket, OutgoingPacket},
};
use hopr_internal_types::prelude::*;
use hopr_network_types::prelude::ResolvedTransportRouting;
use hopr_primitive_types::prelude::*;
use hopr_protocol_app::prelude::ApplicationData;
use hopr_transport_identity::PeerId;
use tracing::error;

lazy_static::lazy_static! {
    /// Fixed price per packet to 0.01 HOPR
    pub static ref DEFAULT_PRICE_PER_PACKET: U256 = 10000000000000000u128.into();
}

#[async_trait::async_trait]
pub trait PacketWrapping {
    type Input;

    async fn send(&self, data: ApplicationData, routing: ResolvedTransportRouting) -> Result<OutgoingPacket>;
}

#[async_trait::async_trait]
pub trait PacketUnwrapping {
    type Packet;

    async fn recv(&self, peer: &PeerId, data: Box<[u8]>) -> Result<Self::Packet>;
}

/// Implements protocol acknowledgement logic for msg packets
#[derive(Debug, Clone)]
pub struct PacketProcessor<Db>
where
    Db: HoprDbProtocolOperations + Send + Sync + std::fmt::Debug + Clone,
{
    db: Db,
    cfg: PacketInteractionConfig,
}

#[async_trait::async_trait]
impl<Db> PacketWrapping for PacketProcessor<Db>
where
    Db: HoprDbProtocolOperations + Send + Sync + std::fmt::Debug + Clone,
{
    type Input = ApplicationData;

    #[tracing::instrument(level = "trace", skip(self, data), ret(Debug), err)]
    async fn send(&self, data: ApplicationData, routing: ResolvedTransportRouting) -> Result<OutgoingPacket> {
        let packet = self
            .db
            .to_send(
                data.to_bytes(),
                routing,
                self.determine_actual_outgoing_win_prob().await,
                self.determine_actual_outgoing_ticket_price().await?,
                Some(data.flags.bits()),
            )
            .await
            .map_err(|e| PacketError::PacketConstructionError(e.to_string()))?;

        Ok(packet)
    }
}

#[async_trait::async_trait]
impl<Db> PacketUnwrapping for PacketProcessor<Db>
where
    Db: HoprDbProtocolOperations + Send + Sync + std::fmt::Debug + Clone,
{
    type Packet = IncomingPacket;

    #[tracing::instrument(level = "trace", skip(self, data))]
    async fn recv(&self, peer: &PeerId, data: Box<[u8]>) -> Result<Self::Packet> {
        let previous_hop = OffchainPublicKey::try_from(peer)
            .map_err(|e| PacketError::LogicError(format!("failed to convert '{peer}' into the public key: {e}")))?;

        let packet = self
            .db
            .from_recv(
                data,
                &self.cfg.packet_keypair,
                previous_hop,
                self.determine_actual_outgoing_win_prob().await,
                self.determine_actual_outgoing_ticket_price().await?,
            )
            .await
            .map_err(|e| match e {
                hopr_db_api::errors::DbError::TicketValidationError(v) => {
                    PacketError::TicketValidation(hopr_crypto_packet::errors::TicketValidationError {
                        reason: v.1,
                        ticket: Box::new(v.0),
                    })
                }
                _ => PacketError::PacketConstructionError(e.to_string()),
            })?;

        Ok(packet)
    }
}

impl<Db> PacketProcessor<Db>
where
    Db: HoprDbProtocolOperations + Send + Sync + std::fmt::Debug + Clone,
{
    /// Creates a new instance given the DB and configuration.
    pub fn new(db: Db, cfg: PacketInteractionConfig) -> Self {
        Self { db, cfg }
    }

    // NOTE: as opposed to the winning probability, the ticket price does not have
    // a reasonable default and therefore the operation fails
    async fn determine_actual_outgoing_ticket_price(&self) -> Result<HoprBalance> {
        // This operation hits the cache unless the new value is fetched for the first time
        let network_ticket_price =
            self.db.get_network_ticket_price().await.map_err(|e| {
                PacketError::LogicError(format!("failed to determine current network ticket price: {e}"))
            })?;

        Ok(self.cfg.outgoing_ticket_price.unwrap_or(network_ticket_price))
    }

    async fn determine_actual_outgoing_win_prob(&self) -> WinningProbability {
        // This operation hits the cache unless the new value is fetched for the first time
        let network_win_prob = self
            .db
            .get_network_winning_probability()
            .await
            .inspect_err(|error| error!(%error, "failed to determine current network winning probability"))
            .ok();

        // If no explicit winning probability is configured, use the network value
        // or 1 if the network value was not determined.
        // This code does not take the max from those, as it is the upper layer's responsibility
        // to ensure the configured value is not smaller than the network value.
        self.cfg
            .outgoing_ticket_win_prob
            .or(network_win_prob)
            .unwrap_or_default() // Absolute default WinningProbability is 1.0
    }
}

/// Packet send finalizer notifying the awaiting future once the send has been acknowledged.
///
/// This is a remnant of the original logic that assumed that the p2p transport is invokable
/// and its result can be directly polled. As the `send_packet` logic is the only part visible
/// outside the communication loop from the protocol side, it is retained pending a larger
/// architectural overhaul of the hopr daemon.
#[derive(Debug)]
pub struct PacketSendFinalizer {
    tx: futures::channel::oneshot::Sender<std::result::Result<(), PacketError>>,
}

impl PacketSendFinalizer {
    pub fn finalize(self, result: std::result::Result<(), PacketError>) {
        if self.tx.send(result).is_err() {
            tracing::trace!("Failed to notify the awaiter about the successful packet transmission")
        }
    }
}

impl From<futures::channel::oneshot::Sender<std::result::Result<(), PacketError>>> for PacketSendFinalizer {
    fn from(value: futures::channel::oneshot::Sender<std::result::Result<(), PacketError>>) -> Self {
        Self { tx: value }
    }
}

/// Await on future until the confirmation of packet reception is received
#[derive(Debug)]
pub struct PacketSendAwaiter {
    rx: futures::channel::oneshot::Receiver<std::result::Result<(), PacketError>>,
}

impl From<futures::channel::oneshot::Receiver<std::result::Result<(), PacketError>>> for PacketSendAwaiter {
    fn from(value: futures::channel::oneshot::Receiver<std::result::Result<(), PacketError>>) -> Self {
        Self { rx: value }
    }
}

impl PacketSendAwaiter {
    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn consume_and_wait(self, until_timeout: std::time::Duration) -> Result<()> {
        let timeout = sleep(until_timeout);
        let rx = self.rx;
        pin_mut!(rx, timeout);
        match futures::future::select(rx, timeout).await {
            Either::Left((Ok(Ok(v)), _)) => Ok(v),
            Either::Left((Ok(Err(e)), _)) => Err(TransportError(e.to_string())),
            Either::Left((Err(_), _)) => Err(TransportError("Canceled".to_owned())),
            Either::Right(_) => Err(TransportError("Timed out on sending a packet".to_owned())),
        }
    }
}

pub type SendMsgInput = (ApplicationData, ResolvedTransportRouting, PacketSendFinalizer);

#[derive(Debug, Clone)]
pub struct MsgSender<T>
where
    T: Sink<SendMsgInput> + Send + Sync + Clone + 'static + std::marker::Unpin,
{
    tx: T,
}

impl<T> MsgSender<T>
where
    T: Sink<SendMsgInput> + Send + Sync + Clone + 'static + std::marker::Unpin,
{
    pub fn new(tx: T) -> Self {
        Self { tx }
    }

    /// Pushes a new packet into processing.
    #[tracing::instrument(level = "trace", skip(self, data))]
    pub async fn send_packet(
        &self,
        data: ApplicationData,
        routing: ResolvedTransportRouting,
    ) -> Result<PacketSendAwaiter> {
        let (tx, rx) = futures::channel::oneshot::channel::<std::result::Result<(), PacketError>>();

        self.tx
            .clone()
            .send((data, routing, tx.into()))
            .await
            .map_err(|_| TransportError("Failed to send a message".into()))
            .map(move |_| {
                let awaiter: PacketSendAwaiter = rx.into();
                awaiter
            })
    }
}

/// Configuration parameters for the packet interaction.
#[derive(Clone, Debug)]
pub struct PacketInteractionConfig {
    pub packet_keypair: OffchainKeypair,
    pub outgoing_ticket_win_prob: Option<WinningProbability>,
    pub outgoing_ticket_price: Option<HoprBalance>,
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use anyhow::Context;
    use futures::StreamExt;
    use hopr_crypto_random::Randomizable;
    use hopr_internal_types::prelude::HoprPseudonym;
    use hopr_path::ValidatedPath;
    use tokio::time::timeout;

    use super::*;

    #[tokio::test]
    pub async fn packet_send_finalizer_is_triggered() {
        let (tx, rx) = futures::channel::oneshot::channel::<std::result::Result<(), PacketError>>();

        let finalizer: PacketSendFinalizer = tx.into();
        let awaiter: PacketSendAwaiter = rx.into();

        finalizer.finalize(Ok(()));

        let result = awaiter.consume_and_wait(Duration::from_millis(20)).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    pub async fn message_sender_operation_reacts_on_finalizer_closure() -> anyhow::Result<()> {
        let (tx, mut rx) = futures::channel::mpsc::unbounded::<SendMsgInput>();

        let sender = MsgSender::new(tx);

        let expected_data = ApplicationData::from_bytes(&[0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09])?;
        let expected_path = ValidatedPath::direct(
            *OffchainKeypair::random().public(),
            ChainKeypair::random().public().to_address(),
        );

        let routing = ResolvedTransportRouting::Forward {
            pseudonym: HoprPseudonym::random(),
            forward_path: expected_path.clone(),
            return_paths: vec![],
        };

        let result = sender.send_packet(expected_data.clone(), routing.clone()).await;
        assert!(result.is_ok());

        let received = rx.next();
        let (data, path, finalizer) = timeout(Duration::from_millis(20), received)
            .await
            .context("Timeout")?
            .context("value should be present")?;

        assert_eq!(data, expected_data);
        assert!(matches!(path, ResolvedTransportRouting::Forward { forward_path,.. } if forward_path == expected_path));

        tokio::task::spawn(async move {
            tokio::time::sleep(Duration::from_millis(3)).await;
            finalizer.finalize(Ok(()))
        });

        assert!(
            result
                .context("Awaiter must be present")?
                .consume_and_wait(Duration::from_millis(10))
                .await
                .is_ok()
        );

        Ok(())
    }
}
