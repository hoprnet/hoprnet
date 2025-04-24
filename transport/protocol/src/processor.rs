use futures::{future::Either, SinkExt};
use futures::{pin_mut, Sink};
use tracing::error;

use hopr_async_runtime::prelude::sleep;
use hopr_crypto_packet::errors::PacketError;
use hopr_crypto_packet::errors::{
    PacketError::{TagReplay, TransportError},
    Result,
};
use hopr_crypto_types::prelude::*;
use hopr_db_api::prelude::HoprDbProtocolOperations;
use hopr_db_api::protocol::IncomingPacket;
use hopr_internal_types::prelude::*;
use hopr_network_types::prelude::ResolvedTransportRouting;
use hopr_primitive_types::prelude::*;
use hopr_transport_identity::PeerId;

use crate::bloom;

lazy_static::lazy_static! {
    /// Fixed price per packet to 0.01 HOPR
    pub static ref DEFAULT_PRICE_PER_PACKET: U256 = 10000000000000000u128.into();
}

#[async_trait::async_trait]
pub trait PacketWrapping {
    type Input;

    async fn send(&self, data: ApplicationData, routing: ResolvedTransportRouting) -> Result<(PeerId, Box<[u8]>)>;
}

pub struct SendPkt {
    pub peer: PeerId,
    pub data: Box<[u8]>,
}

pub struct SendAck {
    pub peer: PeerId,
    pub ack: Box<[u8]>,
}

pub enum RecvOperation {
    Receive { data: ApplicationData, ack: SendAck },
    Forward { msg: SendPkt, ack: SendAck },
}

#[async_trait::async_trait]
pub trait PacketUnwrapping {
    type Packet;

    async fn recv(&self, peer: &PeerId, data: Box<[u8]>) -> Result<Option<Self::Packet>>;
}

/// Implements protocol acknowledgement logic for msg packets
#[derive(Debug, Clone)]
pub struct PacketProcessor<Db>
where
    Db: HoprDbProtocolOperations + Send + Sync + std::fmt::Debug + Clone,
{
    db: Db,
    tbf: bloom::WrappedTagBloomFilter,
    cfg: PacketInteractionConfig,
}

#[async_trait::async_trait]
impl<Db> PacketWrapping for PacketProcessor<Db>
where
    Db: HoprDbProtocolOperations + Send + Sync + std::fmt::Debug + Clone,
{
    type Input = ApplicationData;

    #[tracing::instrument(level = "trace", skip(self, data))]
    async fn send(&self, data: ApplicationData, routing: ResolvedTransportRouting) -> Result<(PeerId, Box<[u8]>)> {
        let packet = self
            .db
            .to_send(
                data.to_bytes(),
                routing,
                self.determine_actual_outgoing_win_prob().await,
                self.determine_actual_outgoing_ticket_price().await?,
            )
            .await
            .map_err(|e| PacketError::PacketConstructionError(e.to_string()))?;

        Ok((packet.next_hop.into(), packet.data))
    }
}

#[async_trait::async_trait]
impl<Db> PacketUnwrapping for PacketProcessor<Db>
where
    Db: HoprDbProtocolOperations + Send + Sync + std::fmt::Debug + Clone,
{
    type Packet = RecvOperation;

    #[tracing::instrument(level = "trace", skip(self, data))]
    async fn recv(&self, peer: &PeerId, data: Box<[u8]>) -> Result<Option<RecvOperation>> {
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

        // Indicating
        match packet {
            Some(packet) => {
                #[allow(irrefutable_let_patterns)]
                if let IncomingPacket::Final { packet_tag, .. } | IncomingPacket::Forwarded { packet_tag, .. } = &packet
                {
                    if self.is_tag_replay(packet_tag).await {
                        return Err(TagReplay);
                    }
                };

                Ok(match packet {
                    IncomingPacket::Final {
                        previous_hop,
                        plain_text,
                        ack_key,
                        ..
                    } => {
                        let app_data = ApplicationData::from_bytes(plain_text.as_ref())?;

                        let ack = Acknowledgement::new(ack_key, &self.cfg.packet_keypair);
                        let ack_packet = self
                            .db
                            .to_send_no_ack(Box::from_iter(ack.as_ref().iter().copied()), previous_hop) // TODO: Optimize this copy
                            .await
                            .map_err(|e| PacketError::PacketConstructionError(e.to_string()))?;

                        Some(RecvOperation::Receive {
                            data: app_data,
                            ack: SendAck {
                                peer: ack_packet.next_hop.into(),
                                ack: ack_packet.data,
                            },
                        })
                    }
                    IncomingPacket::Forwarded {
                        previous_hop,
                        next_hop,
                        data,
                        ack,
                        ..
                    } => {
                        let ack_packet = self
                            .db
                            .to_send_no_ack(Box::from_iter(ack.as_ref().iter().copied()), previous_hop) // TODO: Optimize this copy
                            .await
                            .map_err(|e| PacketError::PacketConstructionError(e.to_string()))?;

                        Some(RecvOperation::Forward {
                            msg: SendPkt {
                                peer: next_hop.into(),
                                data,
                            },
                            ack: SendAck {
                                peer: ack_packet.next_hop.into(),
                                ack: ack_packet.data,
                            },
                        })
                    }
                })
            }
            None => Ok(None),
        }
    }
}

impl<Db> PacketProcessor<Db>
where
    Db: HoprDbProtocolOperations + Send + Sync + std::fmt::Debug + Clone,
{
    /// Creates a new instance given the DB and configuration.
    pub fn new(db: Db, tbf: bloom::WrappedTagBloomFilter, cfg: PacketInteractionConfig) -> Self {
        Self { db, tbf, cfg }
    }

    #[tracing::instrument(level = "trace", name = "check_tag_replay", skip(self, tag))]
    /// Check whether the packet is replayed using a packet tag.
    ///
    /// There is a 0.1% chance that the positive result is not a replay because a Bloom filter is used.
    pub async fn is_tag_replay(&self, tag: &PacketTag) -> bool {
        self.tbf
            .with_write_lock(|inner: &mut TagBloomFilter| inner.check_and_set(tag))
            .await
    }

    // NOTE: as opposed to the winning probability, the ticket price does not have
    // a reasonable default and therefore the operation fails
    async fn determine_actual_outgoing_ticket_price(&self) -> Result<Balance> {
        // This operation hits the cache unless the new value is fetched for the first time
        let network_ticket_price =
            self.db.get_network_ticket_price().await.map_err(|e| {
                PacketError::LogicError(format!("failed to determine current network ticket price: {e}"))
            })?;

        Ok(self.cfg.outgoing_ticket_price.unwrap_or(network_ticket_price))
    }

    async fn determine_actual_outgoing_win_prob(&self) -> f64 {
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
            .unwrap_or(DEFAULT_OUTGOING_TICKET_WIN_PROB)
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
            error!("Failed to notify the awaiter about the successful packet transmission")
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
    pub chain_keypair: ChainKeypair,
    pub outgoing_ticket_win_prob: Option<f64>,
    pub outgoing_ticket_price: Option<Balance>,
}

impl PacketInteractionConfig {
    pub fn new(
        packet_keypair: &OffchainKeypair,
        chain_keypair: &ChainKeypair,
        outgoing_ticket_win_prob: Option<f64>,
        outgoing_ticket_price: Option<Balance>,
    ) -> Self {
        Self {
            packet_keypair: packet_keypair.clone(),
            chain_keypair: chain_keypair.clone(),
            outgoing_ticket_win_prob,
            outgoing_ticket_price,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use anyhow::Context;
    use async_std::future::timeout;
    use futures::StreamExt;
    use hopr_crypto_random::Randomizable;
    use hopr_internal_types::prelude::HoprPseudonym;
    use hopr_path::ValidatedPath;
    use std::time::Duration;

    // #[test]
    // fn multiple_acknowledgements_fit_into_a_message_sized_payload() {
    //     assert_eq!(std::mem::size_of::<Acknowledgement>(), 1);
    // }

    #[async_std::test]
    pub async fn packet_send_finalizer_is_triggered() {
        let (tx, rx) = futures::channel::oneshot::channel::<std::result::Result<(), PacketError>>();

        let finalizer: PacketSendFinalizer = tx.into();
        let awaiter: PacketSendAwaiter = rx.into();

        finalizer.finalize(Ok(()));

        let result = awaiter.consume_and_wait(Duration::from_millis(20)).await;

        assert!(result.is_ok());
    }

    #[async_std::test]
    pub async fn message_sender_operation_reacts_on_finalizer_closure() -> anyhow::Result<()> {
        let (tx, mut rx) = futures::channel::mpsc::unbounded::<SendMsgInput>();

        let sender = MsgSender::new(tx);

        let expected_data = ApplicationData::from_bytes(&[0x01, 0x02, 0x03])?;
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

        async_std::task::spawn(async move {
            async_std::task::sleep(Duration::from_millis(3)).await;
            finalizer.finalize(Ok(()))
        });

        assert!(result
            .context("Awaiter must be present")?
            .consume_and_wait(Duration::from_millis(10))
            .await
            .is_ok());

        Ok(())
    }
}
