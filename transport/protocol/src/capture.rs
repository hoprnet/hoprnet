use std::{borrow::Cow, fs::File};

use futures::{FutureExt, StreamExt};
use hopr_async_runtime::{AbortHandle, spawn_as_abortable};
use hopr_crypto_types::types::OffchainPublicKey;
use hopr_db_api::prelude::IncomingPacket;
use hopr_internal_types::prelude::Acknowledgement;
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

use crate::HOPR_PACKET_SIZE;

#[derive(Copy, Clone, Debug, strum::Display)]
pub enum PacketDirection {
    Incoming,
    Outgoing,
}

pub trait PacketWriter {
    fn write_packet(&mut self, packet: &[u8], direction: PacketDirection) -> std::io::Result<()>;
}

pub struct NullWriter;

impl PacketWriter for NullWriter {
    fn write_packet(&mut self, _: &[u8], _: PacketDirection) -> std::io::Result<()> {
        Ok(())
    }
}

pub struct PcapPacketWriter(PcapNgWriter<File>);

impl PcapPacketWriter {
    pub fn new(file: File) -> std::io::Result<Self> {
        let mut writer = PcapNgWriter::new(file).map_err(std::io::Error::other)?;

        writer
            .write_pcapng_block(InterfaceDescriptionBlock {
                linktype: DataLink::USER0,
                snaplen: 0,
                options: vec![],
            })
            .map_err(std::io::Error::other)?;

        Ok(Self(writer))
    }
}

impl PacketWriter for PcapPacketWriter {
    fn write_packet(&mut self, packet: &[u8], direction: PacketDirection) -> std::io::Result<()> {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap();
        self.0
            .write_pcapng_block(EnhancedPacketBlock {
                interface_id: 0,
                timestamp,
                original_len: HOPR_PACKET_SIZE as u32,
                data: packet.into(),
                options: vec![EnhancedPacketOption::Comment(direction.to_string().into())],
            })
            .map(|_| ())
            .map_err(std::io::Error::other)
    }
}

pub struct UdpPacketDump(std::net::UdpSocket);

impl UdpPacketDump {
    pub fn new(addr: std::net::SocketAddr) -> std::io::Result<Self> {
        let sock = std::net::UdpSocket::bind(("0.0.0.0", 0))?;
        sock.connect(addr)?;
        Ok(Self(sock))
    }
}

#[cfg(feature = "runtime-tokio")]
impl PacketWriter for UdpPacketDump {
    fn write_packet(&mut self, packet: &[u8], _direction: PacketDirection) -> std::io::Result<()> {
        let mut sent = 0;
        while sent < packet.len() {
            sent += self.0.send(&packet[sent..])?;
        }
        Ok(())
    }
}

pub fn packet_capture_channel(
    writer: Box<dyn PacketWriter + Send>,
) -> (futures::channel::mpsc::Sender<CapturedPacket>, AbortHandle) {
    let (sender, receiver) = futures::channel::mpsc::channel(20_000);
    let writer = std::sync::Arc::new(std::sync::Mutex::new(writer));
    let ah = spawn_as_abortable!(receiver.for_each(move |packet: CapturedPacket| {
        let writer = writer.clone();
        hopr_async_runtime::prelude::spawn_blocking(move || {
            writer
                .lock()
                .map_err(|_| std::io::Error::other("lock poisoned"))
                .and_then(|mut w| w.write_packet(&packet.1, packet.0))
        })
        .map(|_| ())
    }));

    (sender, ah)
}

#[repr(u8)]
enum PacketType {
    Final = 0,
    Forwarded = 1,
    Outgoing = 2,
    InAck = 3,
    OutAck = 4,
}

pub enum PacketBeforeTransit<'a> {
    OutgoingPacket {
        me: OffchainPublicKey,
        next_hop: OffchainPublicKey,
        data: Cow<'a, [u8]>,
        ack_challenge: Cow<'a, [u8]>,
    },
    OutgoingAck {
        me: OffchainPublicKey,
        next_hop: OffchainPublicKey,
        ack: Acknowledgement,
        is_random: bool,
    },
    IncomingPacket {
        me: OffchainPublicKey,
        packet: &'a IncomingPacket,
    },
}

pub struct CapturedPacket(PacketDirection, Box<[u8]>);

impl<'a> From<PacketBeforeTransit<'a>> for CapturedPacket {
    fn from(value: PacketBeforeTransit<'a>) -> Self {
        let mut out = Vec::new();
        let mut direction = PacketDirection::Incoming;
        match value {
            PacketBeforeTransit::OutgoingPacket {
                me,
                next_hop,
                data,
                ack_challenge,
            } => {
                out.push(PacketType::Outgoing as u8);
                out.extend_from_slice(me.as_ref());
                out.extend_from_slice(me.to_peerid_str().as_bytes());
                out.push(0); // Add null terminator to the string
                out.extend_from_slice(next_hop.as_ref());
                out.extend_from_slice(next_hop.to_peerid_str().as_bytes());
                out.push(0); // Add null terminator to the string
                out.extend_from_slice(ack_challenge.as_ref());
                out.extend_from_slice((data.len() as u16).to_be_bytes().as_ref());
                out.extend_from_slice(data.as_ref());
                direction = PacketDirection::Outgoing;
            }
            PacketBeforeTransit::OutgoingAck {
                me,
                next_hop,
                ack,
                is_random,
            } => {
                out.push(PacketType::OutAck as u8);
                out.extend_from_slice(me.as_ref());
                out.extend_from_slice(me.to_peerid_str().as_bytes());
                out.push(0); // Add null terminator to the string
                out.extend_from_slice(next_hop.as_ref());
                out.extend_from_slice(next_hop.to_peerid_str().as_bytes());
                out.push(0); // Add null terminator to the string
                out.push(if is_random { 1 } else { 0 });
                out.extend_from_slice(ack.as_ref());
                direction = PacketDirection::Outgoing;
            }
            PacketBeforeTransit::IncomingPacket {
                me,
                packet:
                    IncomingPacket::Final {
                        packet_tag,
                        previous_hop,
                        sender,
                        plain_text,
                        ack_key,
                    },
            } => {
                out.push(PacketType::Final as u8);
                out.extend_from_slice(packet_tag);
                out.extend_from_slice(previous_hop.as_ref());
                out.extend_from_slice(previous_hop.to_peerid_str().as_bytes());
                out.push(0); // Add null terminator to the string
                out.extend_from_slice(me.as_ref());
                out.extend_from_slice(me.to_peerid_str().as_bytes());
                out.push(0); // Add null terminator to the string
                out.extend_from_slice(sender.as_ref());
                out.extend_from_slice(ack_key.as_ref());
                out.extend_from_slice((plain_text.len() as u16).to_be_bytes().as_ref());
                out.extend_from_slice(plain_text.as_ref());
            }
            PacketBeforeTransit::IncomingPacket {
                packet:
                    IncomingPacket::Forwarded {
                        packet_tag,
                        previous_hop,
                        next_hop,
                        data,
                        ack,
                    },
                ..
            } => {
                out.push(PacketType::Forwarded as u8);
                out.extend_from_slice(packet_tag);
                out.extend_from_slice(previous_hop.as_ref());
                out.extend_from_slice(previous_hop.to_peerid_str().as_bytes());
                out.push(0); // Add null terminator to the string
                out.extend_from_slice(next_hop.as_ref());
                out.extend_from_slice(next_hop.to_peerid_str().as_bytes());
                out.push(0); // Add null terminator to the string
                out.extend_from_slice(ack.as_ref());
                out.extend_from_slice((data.len() as u16).to_be_bytes().as_ref());
                out.extend_from_slice(data.as_ref());
            }
            PacketBeforeTransit::IncomingPacket {
                me,
                packet:
                    IncomingPacket::Acknowledgement {
                        packet_tag,
                        previous_hop,
                        ack,
                    },
            } => {
                out.push(PacketType::InAck as u8);
                out.extend_from_slice(packet_tag);
                out.extend_from_slice(previous_hop.as_ref());
                out.extend_from_slice(previous_hop.to_peerid_str().as_bytes());
                out.push(0); // Add null terminator to the string
                out.extend_from_slice(me.as_ref());
                out.extend_from_slice(me.to_peerid_str().as_bytes());
                out.push(0); // Add null terminator to the string
                out.extend_from_slice(ack.as_ref());
            }
        }

        Self(direction, out.into_boxed_slice())
    }
}

#[cfg(test)]
mod tests {
    use futures::{SinkExt, pin_mut};
    use hex_literal::hex;
    use hopr_crypto_random::Randomizable;
    use hopr_crypto_types::{
        prelude::{Keypair, OffchainKeypair, SimplePseudonym},
        types::HalfKey,
    };
    use hopr_internal_types::prelude::Acknowledgement;
    use hopr_network_types::prelude::{
        FrameInfo, Segment,
        protocol::{FrameAcknowledgements, SegmentRequest, SessionMessage},
    };
    use hopr_transport_packet::prelude::ApplicationData;
    use hopr_transport_probe::content::{NeighborProbe, PathTelemetry};

    use super::*;

    #[tokio::test]
    async fn test_file_capture() -> anyhow::Result<()> {
        let me = *OffchainKeypair::random().public();

        let (pcap, ah) = packet_capture_channel(Box::new(File::create("test.pcap").and_then(PcapPacketWriter::new)?));
        pin_mut!(pcap);

        let packet = IncomingPacket::Final {
            packet_tag: hopr_crypto_random::random_bytes(),
            previous_hop: *OffchainKeypair::random().public(),
            sender: SimplePseudonym::random(),
            plain_text: ApplicationData::new(10u64, &hex!("deadbeef")).to_bytes(),
            ack_key: HalfKey::random(),
        };

        let _ = pcap
            .send(PacketBeforeTransit::IncomingPacket { me, packet: &packet }.into())
            .await;

        let msg = SessionMessage::<1000>::Segment(Segment {
            frame_id: 1,
            seq_idx: 0,
            seq_len: 1,
            data: Box::new(hex!("474554202f20485454502f312e310d0a486f73743a207777772e6578616d706c652e636f6d0d0a557365722d4167656e743a206375726c2f382e372e310d0a4163636570743a202a2f2a0d0a0d0a")),
        });

        let packet = IncomingPacket::Final {
            packet_tag: hopr_crypto_random::random_bytes(),
            previous_hop: *OffchainKeypair::random().public(),
            sender: SimplePseudonym::random(),
            plain_text: ApplicationData::new_from_owned(1024_u64, msg.into_encoded()).to_bytes(),
            ack_key: HalfKey::random(),
        };

        let _ = pcap
            .send(PacketBeforeTransit::IncomingPacket { me, packet: &packet }.into())
            .await;

        let msg = SessionMessage::<1000>::Acknowledge(FrameAcknowledgements::from(vec![1, 2, 100]));

        let packet = IncomingPacket::Final {
            packet_tag: hopr_crypto_random::random_bytes(),
            previous_hop: *OffchainKeypair::random().public(),
            sender: SimplePseudonym::random(),
            plain_text: ApplicationData::new_from_owned(1024_u64, msg.into_encoded()).to_bytes(),
            ack_key: HalfKey::random(),
        };

        let _ = pcap
            .send(PacketBeforeTransit::IncomingPacket { me, packet: &packet }.into())
            .await;

        let msg = SessionMessage::<1000>::Request(SegmentRequest::from_iter([
            FrameInfo {
                frame_id: 15,
                missing_segments: [0b10100001].into(),
                total_segments: 8,
                last_update: std::time::SystemTime::now(),
            },
            FrameInfo {
                frame_id: 11,
                missing_segments: [0b00100001].into(),
                total_segments: 8,
                last_update: std::time::SystemTime::now(),
            },
        ]));

        let packet = IncomingPacket::Final {
            packet_tag: hopr_crypto_random::random_bytes(),
            previous_hop: *OffchainKeypair::random().public(),
            sender: SimplePseudonym::random(),
            plain_text: ApplicationData::new_from_owned(1024_u64, msg.into_encoded()).to_bytes(),
            ack_key: HalfKey::random(),
        };

        let _ = pcap
            .send(PacketBeforeTransit::IncomingPacket { me, packet: &packet }.into())
            .await;

        let kp = OffchainKeypair::random();
        let packet = IncomingPacket::Acknowledgement {
            packet_tag: hopr_crypto_random::random_bytes(),
            previous_hop: *kp.public(),
            ack: Acknowledgement::random(&kp),
        };

        let _ = pcap
            .send(PacketBeforeTransit::IncomingPacket { me, packet: &packet }.into())
            .await;

        let packet = IncomingPacket::Forwarded {
            packet_tag: hopr_crypto_random::random_bytes(),
            previous_hop: *OffchainKeypair::random().public(),
            next_hop: *OffchainKeypair::random().public(),
            data: Box::new([0x08]),
            ack: Acknowledgement::random(&OffchainKeypair::random()),
        };

        let _ = pcap
            .send(PacketBeforeTransit::IncomingPacket { me, packet: &packet }.into())
            .await;

        let hk = HalfKey::random().to_challenge();
        let packet = PacketBeforeTransit::OutgoingPacket {
            me,
            next_hop: *OffchainKeypair::random().public(),
            ack_challenge: hk.as_ref().into(),
            data: ApplicationData::from(hopr_transport_probe::content::Message::Telemetry(PathTelemetry {
                id: hopr_crypto_random::random_bytes(),
                path: hopr_crypto_random::random_bytes(),
                timestamp: 123456789_u128,
            }))
            .to_bytes()
            .to_vec()
            .into(),
        };

        let _ = pcap.send(packet.into()).await;

        let hk = HalfKey::random().to_challenge();
        let packet = PacketBeforeTransit::OutgoingPacket {
            me,
            next_hop: *OffchainKeypair::random().public(),
            ack_challenge: hk.as_ref().into(),
            data: ApplicationData::from(hopr_transport_probe::content::Message::Probe(
                NeighborProbe::random_nonce(),
            ))
            .to_bytes()
            .to_vec()
            .into(),
        };

        let _ = pcap.send(packet.into()).await;

        let hk = HalfKey::random().to_challenge();
        let packet = PacketBeforeTransit::OutgoingPacket {
            me,
            next_hop: *OffchainKeypair::random().public(),
            ack_challenge: hk.as_ref().into(),
            data: ApplicationData::from(hopr_transport_probe::content::Message::Probe(NeighborProbe::Pong(
                hopr_crypto_random::random_bytes(),
            )))
            .to_bytes()
            .to_vec()
            .into(),
        };

        let _ = pcap.send(packet.into()).await;

        let hk = HalfKey::random().to_challenge();
        let packet = PacketBeforeTransit::OutgoingPacket {
            me,
            next_hop: *OffchainKeypair::random().public(),
            ack_challenge: hk.as_ref().into(),
            data: ApplicationData::new(1u64, &hex!("0004babe02"))
                .to_bytes()
                .into_vec()
                .into(),
        };

        let _ = pcap.send(packet.into()).await;

        let kp = OffchainKeypair::random();

        let packet = PacketBeforeTransit::OutgoingAck {
            me,
            next_hop: *kp.public(),
            ack: Acknowledgement::random(&kp),
            is_random: false,
        };

        let _ = pcap.send(packet.into()).await;

        let packet = PacketBeforeTransit::OutgoingAck {
            me,
            next_hop: *kp.public(),
            ack: Acknowledgement::random(&kp),
            is_random: true,
        };

        let _ = pcap.send(packet.into()).await;

        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        ah.abort();
        Ok(())
    }
}
