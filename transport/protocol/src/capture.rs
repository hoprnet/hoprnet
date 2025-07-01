use std::fs::File;

use futures::{StreamExt, future::Either, pin_mut};
use hopr_db_api::prelude::{IncomingPacket, OutgoingPacket};
use hopr_primitive_types::errors::GeneralError;
use pcap_file::{
    DataLink,
    pcapng::{
        PcapNgWriter,
        blocks::{
            enhanced_packet::{EnhancedPacketBlock, EnhancedPacketOption},
            interface_description::InterfaceDescriptionBlock,
        },
    },
};
use uuid::Uuid;

use crate::HOPR_PACKET_SIZE;

type CaptureSender = std::sync::Arc<std::sync::Mutex<futures::channel::mpsc::Sender<Either<Box<[u8]>, Box<[u8]>>>>>;

#[derive(Clone, Debug)]
pub(crate) struct PacketCapture {
    capture_tx: CaptureSender,
}

impl Default for PacketCapture {
    fn default() -> Self {
        let (capture_tx, capture_rx) = futures::channel::mpsc::channel::<Either<Box<[u8]>, Box<[u8]>>>(20000);
        hopr_async_runtime::prelude::spawn(async move {
            match File::create_new(format!("{}.pcap", Uuid::new_v4()))
                .and_then(|f| PcapNgWriter::new(f).map_err(std::io::Error::other))
            {
                Ok(pcap_writer) => {
                    let pcap_writer = std::sync::Arc::new(std::sync::Mutex::new(pcap_writer));

                    let pcap_writer_clone = pcap_writer.clone();
                    hopr_async_runtime::prelude::spawn_blocking(move || {
                        let _ = pcap_writer_clone
                            .lock()
                            .map_err(|_| std::io::Error::other("lock error"))
                            .and_then(|mut w| {
                                w.write_pcapng_block(InterfaceDescriptionBlock {
                                    linktype: DataLink::ETHERNET,
                                    snaplen: 0,
                                    options: vec![],
                                })
                                .map_err(std::io::Error::other)
                            });
                    });

                    pin_mut!(capture_rx);
                    while let Some(packet) = capture_rx.next().await {
                        let timestamp = std::time::SystemTime::now()
                            .duration_since(std::time::SystemTime::UNIX_EPOCH)
                            .unwrap();
                        let block = match packet {
                            Either::Left(outgoing) => EnhancedPacketBlock {
                                interface_id: 0,
                                timestamp,
                                original_len: HOPR_PACKET_SIZE as u32,
                                data: outgoing.into_vec().into(),
                                options: vec![EnhancedPacketOption::Comment("outgoing".into())],
                            },
                            Either::Right(incoming) => EnhancedPacketBlock {
                                interface_id: 0,
                                timestamp,
                                original_len: HOPR_PACKET_SIZE as u32,
                                data: incoming.into_vec().into(),
                                options: vec![EnhancedPacketOption::Comment("incoming".into())],
                            },
                        };

                        let pcap_writer_clone = pcap_writer.clone();
                        hopr_async_runtime::prelude::spawn_blocking(move || {
                            let _ = pcap_writer_clone
                                .lock()
                                .map_err(|_| std::io::Error::other("lock error"))
                                .and_then(|mut w| w.write_pcapng_block(block).map_err(std::io::Error::other));
                        });
                    }
                }
                Err(error) => tracing::error!(%error, "capture could not be created"),
            }
        });

        Self {
            capture_tx: std::sync::Arc::new(std::sync::Mutex::new(capture_tx)),
        }
    }
}

impl PacketCapture {
    pub fn capture_incoming(&self, packet: &IncomingPacket) -> crate::errors::Result<()> {
        Ok(self
            .capture_tx
            .lock()
            .map_err(|_| GeneralError::NonSpecificError("capture: cannot lock".into()))
            .and_then(|mut tx| {
                tx.try_send(Either::Right(encode_incoming_packet_capture(packet)))
                    .map_err(|_| GeneralError::NonSpecificError("capture: cannot send packet".into()))
            })?)
    }

    pub fn capture_outgoing(&self, packet: &OutgoingPacket) -> crate::errors::Result<()> {
        Ok(self
            .capture_tx
            .lock()
            .map_err(|_| GeneralError::NonSpecificError("capture: cannot lock".into()))
            .and_then(|mut tx| {
                tx.try_send(Either::Left(encode_outgoing_packet_capture(packet)))
                    .map_err(|_| GeneralError::NonSpecificError("capture: cannot send packet".into()))
            })?)
    }
}

#[repr(u8)]
enum PacketType {
    Final = 0,
    Forwarded = 1,
    Outgoing = 2,
}

fn encode_incoming_packet_capture(packet: &IncomingPacket) -> Box<[u8]> {
    let mut out = Vec::new();
    match packet {
        IncomingPacket::Final {
            packet_tag,
            previous_hop,
            sender,
            plain_text,
            ack_key,
        } => {
            out.push(PacketType::Final as u8);
            out.extend_from_slice(packet_tag);
            out.extend_from_slice(previous_hop.as_ref());
            out.extend_from_slice(sender.as_ref());
            out.extend_from_slice(ack_key.as_ref());
            out.extend_from_slice((plain_text.len() as u16).to_be_bytes().as_ref());
            out.extend_from_slice(plain_text.as_ref());
        }
        IncomingPacket::Forwarded {
            packet_tag,
            previous_hop,
            next_hop,
            data,
            ack,
        } => {
            out.push(PacketType::Forwarded as u8);
            out.extend_from_slice(packet_tag);
            out.extend_from_slice(previous_hop.as_ref());
            out.extend_from_slice(next_hop.as_ref());
            out.extend_from_slice(ack.as_ref());
            out.extend_from_slice((data.len() as u16).to_be_bytes().as_ref());
            out.extend_from_slice(data.as_ref());
        }
    }
    out.into_boxed_slice()
}

fn encode_outgoing_packet_capture(packet: &OutgoingPacket) -> Box<[u8]> {
    let mut out = Vec::new();
    out.push(PacketType::Outgoing as u8);
    out.extend_from_slice(packet.next_hop.as_ref());
    out.extend_from_slice(packet.ack_challenge.as_ref());
    out.extend_from_slice((packet.data.len() as u16).to_be_bytes().as_ref());
    out.extend_from_slice(packet.data.as_ref());

    out.into_boxed_slice()
}
