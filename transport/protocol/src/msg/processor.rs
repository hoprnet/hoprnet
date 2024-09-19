use futures::pin_mut;
use futures::{future::Either, SinkExt};
use hopr_crypto_packet::errors::PacketError;
use hopr_db_api::protocol::TransportPacketWithChainData;
use libp2p_identity::PeerId;
use tracing::error;

use core_path::path::{Path, TransportPath};
use hopr_async_runtime::prelude::sleep;
use hopr_crypto_packet::errors::{
    PacketError::{TagReplay, TransportError},
    Result,
};
use hopr_crypto_types::prelude::*;
use hopr_db_api::prelude::HoprDbProtocolOperations;
use hopr_internal_types::prelude::*;
use hopr_primitive_types::prelude::*;

use super::packet::OutgoingPacket;
use crate::bloom;
use crate::msg::mixer::MixerConfig;

lazy_static::lazy_static! {
    /// Fixed price per packet to 0.01 HOPR
    pub static ref DEFAULT_PRICE_PER_PACKET: U256 = 10000000000000000u128.into();
}

#[async_trait::async_trait]
pub trait PacketWrapping {
    type Input;

    async fn send(&self, data: ApplicationData, path: TransportPath) -> Result<(PeerId, Box<[u8]>)>;
}

pub struct SendPkt {
    pub peer: PeerId,
    pub data: Box<[u8]>,
}

pub struct SendAck {
    pub peer: PeerId,
    pub ack: Acknowledgement,
}

pub enum RecvOperation {
    Receive { data: ApplicationData, ack: SendAck },
    Forward { msg: SendPkt, ack: SendAck },
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
    async fn send(&self, data: ApplicationData, path: TransportPath) -> Result<(PeerId, Box<[u8]>)> {
        let path: std::result::Result<Vec<OffchainPublicKey>, hopr_primitive_types::errors::GeneralError> =
            path.hops().iter().map(OffchainPublicKey::try_from).collect();

        let packet = self
            .db
            .to_send(
                data.to_bytes(),
                self.cfg.chain_keypair.clone(),
                path?,
                self.cfg.outgoing_ticket_win_prob,
            )
            .await
            .map_err(|e| PacketError::PacketConstructionError(e.to_string()))?;

        let packet: OutgoingPacket = packet
            .try_into()
            .map_err(|e: crate::errors::ProtocolError| PacketError::LogicError(e.to_string()))?;

        Ok((packet.next_hop, packet.data))
    }
}

#[async_trait::async_trait]
impl<Db> PacketUnwrapping for PacketProcessor<Db>
where
    Db: HoprDbProtocolOperations + Send + Sync + std::fmt::Debug + Clone,
{
    type Packet = RecvOperation;

    #[tracing::instrument(level = "trace", skip(self, data))]
    async fn recv(&self, peer: &PeerId, data: Box<[u8]>) -> Result<RecvOperation> {
        let previous_hop = OffchainPublicKey::try_from(peer)
            .map_err(|e| PacketError::LogicError(format!("failed to convert '{peer}' into the public key: {e}")))?;

        let packet = self
            .db
            .from_recv(
                data,
                self.cfg.chain_keypair.clone(),
                &self.cfg.packet_keypair,
                previous_hop,
                self.cfg.outgoing_ticket_win_prob,
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

        if let TransportPacketWithChainData::Final { packet_tag, .. }
        | TransportPacketWithChainData::Forwarded { packet_tag, .. } = &packet
        {
            if self.is_tag_replay(packet_tag).await {
                return Err(TagReplay);
            }
        };

        Ok(match packet {
            TransportPacketWithChainData::Final {
                previous_hop,
                plain_text,
                ack,
                ..
            } => {
                let app_data = ApplicationData::from_bytes(plain_text.as_ref())?;
                RecvOperation::Receive {
                    data: app_data,
                    ack: SendAck {
                        peer: previous_hop.into(),
                        ack,
                    },
                }
            }
            TransportPacketWithChainData::Forwarded {
                previous_hop,
                next_hop,
                data,
                ack,
                ..
            } => RecvOperation::Forward {
                msg: SendPkt {
                    peer: next_hop.into(),
                    data,
                },
                ack: SendAck {
                    peer: previous_hop.into(),
                    ack,
                },
            },
            TransportPacketWithChainData::Outgoing { .. } => {
                return Err(PacketError::LogicError(
                    "Attempting to process an outgoing packet in the incoming pipeline".into(),
                ))
            }
        })
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

pub type SendMsgInput = (ApplicationData, TransportPath, PacketSendFinalizer);

#[derive(Debug)]
pub struct MsgSender {
    tx: futures::channel::mpsc::UnboundedSender<SendMsgInput>,
}

impl MsgSender {
    pub fn new(tx: futures::channel::mpsc::UnboundedSender<SendMsgInput>) -> Self {
        Self { tx }
    }

    /// Pushes a new packet into processing.
    #[tracing::instrument(level = "trace", skip(self, data))]
    pub async fn send_packet(&self, data: ApplicationData, path: TransportPath) -> Result<PacketSendAwaiter> {
        let (tx, rx) = futures::channel::oneshot::channel::<std::result::Result<(), PacketError>>();

        self.tx
            .clone()
            .send((data, path, tx.into()))
            .await
            .map_err(|_| TransportError("Failed to send a message".to_owned()))
            .map(move |_| {
                let awaiter: PacketSendAwaiter = rx.into();
                awaiter
            })
    }
}

/// Configuration parameters for the packet interaction.
#[derive(Clone, Debug)]
pub struct PacketInteractionConfig {
    pub check_unrealized_balance: bool,
    pub packet_keypair: OffchainKeypair,
    pub chain_keypair: ChainKeypair,
    pub mixer: MixerConfig,
    pub outgoing_ticket_win_prob: f64,
}

impl PacketInteractionConfig {
    pub fn new(packet_keypair: &OffchainKeypair, chain_keypair: &ChainKeypair, outgoing_ticket_win_prob: f64) -> Self {
        Self {
            packet_keypair: packet_keypair.clone(),
            chain_keypair: chain_keypair.clone(),
            check_unrealized_balance: true,
            mixer: MixerConfig::default(),
            outgoing_ticket_win_prob,
        }
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Context;
    use async_std::future::timeout;
    use futures::StreamExt;

    use super::*;
    use std::time::Duration;

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
        let expected_path = TransportPath::direct(PeerId::random());

        let result = sender.send_packet(expected_data.clone(), expected_path.clone()).await;
        assert!(result.is_ok());

        let received = rx.next();
        let (data, path, finalizer) = timeout(Duration::from_millis(20), received)
            .await
            .context("Timeout")?
            .context("value should be present")?;

        assert_eq!(data, expected_data);
        assert_eq!(path, expected_path);

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
