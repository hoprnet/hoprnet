use std::{borrow::Cow, fs::File};

use futures::{StreamExt, pin_mut};
use hopr_api::db::IncomingPacket;
use hopr_async_runtime::{AbortHandle, spawn_as_abortable};
use hopr_crypto_packet::prelude::PacketSignals;
use hopr_crypto_types::types::OffchainPublicKey;
use hopr_internal_types::prelude::VerifiedAcknowledgement;
use pcap_file::{
    DataLink,
    pcapng::{
        PcapNgWriter,
        blocks::{
            enhanced_packet::{EnhancedPacketBlock, EnhancedPacketOption},
            interface_description::{InterfaceDescriptionBlock, InterfaceDescriptionOption},
        },
    },
};

use crate::HOPR_PACKET_SIZE;

/// Direction of the packet.
#[derive(Copy, Clone, Debug, PartialEq, Eq, strum::Display)]
pub enum PacketDirection {
    Incoming,
    Outgoing,
}

/// A captured packet that can be written to a [`PacketWriter`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapturedPacket {
    pub direction: PacketDirection,
    pub timestamp: std::time::Duration,
    pub orig_len: u32,
    pub data: Box<[u8]>,
}

/// A trait that allows implementing different packet capture backends.
pub trait PacketWriter {
    /// Writes the [`CapturedPacket`] into the backend.
    fn write_packet(&mut self, packet: CapturedPacket) -> std::io::Result<()>;
}

/// A [`PacketWriter`] that does nothing.
pub struct NullWriter;

impl PacketWriter for NullWriter {
    fn write_packet(&mut self, _: CapturedPacket) -> std::io::Result<()> {
        // An error causes the packet capture channel to terminate
        Err(std::io::Error::other("null writer cannot write captured packets"))
    }
}

/// A [`PacketWriter`] that writes captured packets into a Pcap file.
pub struct PcapPacketWriter(PcapNgWriter<File>);

impl PcapPacketWriter {
    pub fn new(file: File) -> std::io::Result<Self> {
        let mut writer = PcapNgWriter::new(file).map_err(std::io::Error::other)?;

        writer
            .write_pcapng_block(InterfaceDescriptionBlock {
                linktype: DataLink::USER0,
                snaplen: 0,
                options: vec![InterfaceDescriptionOption::IfTsResol(0x09)],
            })
            .map_err(std::io::Error::other)?;

        Ok(Self(writer))
    }
}

impl PacketWriter for PcapPacketWriter {
    fn write_packet(&mut self, packet: CapturedPacket) -> std::io::Result<()> {
        self.0
            .write_pcapng_block(EnhancedPacketBlock {
                interface_id: 0,
                timestamp: packet.timestamp,
                original_len: packet.orig_len,
                data: packet.data.into_vec().into(),
                options: vec![EnhancedPacketOption::Comment(packet.direction.to_string().into())],
            })
            .map(|_| ())
            .map_err(std::io::Error::other)
    }
}

/// A [`PacketWriter`] that sends captured packets over UDP socket.
pub struct UdpPacketDump(std::net::UdpSocket);

impl UdpPacketDump {
    pub fn new(addr: std::net::SocketAddr) -> std::io::Result<Self> {
        let sock = std::net::UdpSocket::bind(("0.0.0.0", 0))?;
        sock.connect(addr)?;
        Ok(Self(sock))
    }
}

impl PacketWriter for UdpPacketDump {
    fn write_packet(&mut self, packet: CapturedPacket) -> std::io::Result<()> {
        let mut sent = 0;
        let data = packet.data;
        while sent < data.len() {
            sent += self.0.send(&data[sent..])?;
        }
        Ok(())
    }
}

/// Creates a queue that processes captured packets into a [`PacketWriter`].
pub fn packet_capture_channel(
    writer: Box<dyn PacketWriter + Send>,
) -> (futures::channel::mpsc::Sender<CapturedPacket>, AbortHandle) {
    let (sender, receiver) = futures::channel::mpsc::channel(20_000);
    let writer = std::sync::Arc::new(std::sync::Mutex::new(writer));
    let ah = spawn_as_abortable!(async move {
        pin_mut!(receiver);
        while let Some(packet) = receiver.next().await {
            if let Ok(start_capturing_path) = std::env::var("HOPR_CAPTURE_PATH_TRIGGER") {
                let path = std::path::Path::new(&start_capturing_path);
                if path.exists() {
                    let writer = writer.clone();
                    match hopr_async_runtime::prelude::spawn_blocking(move || {
                        writer
                            .lock()
                            .map_err(|_| std::io::Error::other("lock poisoned"))
                            .and_then(|mut w| w.write_packet(packet))
                    })
                    .await
                    .map_err(std::io::Error::other)
                    {
                        Err(error) | Ok(Err(error)) => {
                            tracing::warn!(%error, "cannot capture more packets due to error");
                            break;
                        }
                        _ => {}
                    }
                }
            }
        }
    });

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

/// Represents a customized dissection of a HOPR packet before it goes into the transport.
pub enum PacketBeforeTransit<'a> {
    OutgoingPacket {
        me: OffchainPublicKey,
        next_hop: OffchainPublicKey,
        num_surbs: u8,
        is_forwarded: bool,
        data: Cow<'a, [u8]>,
        ack_challenge: Cow<'a, [u8]>,
        signals: PacketSignals,
        ticket: Cow<'a, [u8]>,
    },
    OutgoingAck {
        me: OffchainPublicKey,
        next_hop: OffchainPublicKey,
        ack: VerifiedAcknowledgement,
        is_random: bool,
    },
    IncomingPacket {
        me: OffchainPublicKey,
        packet: &'a IncomingPacket,
        ticket: Cow<'a, [u8]>,
    },
}

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
                signals,
                ticket,
                num_surbs,
                is_forwarded,
            } => {
                out.push(PacketType::Outgoing as u8);
                out.extend_from_slice(me.as_ref());
                out.extend_from_slice(me.to_peerid_str().as_bytes());
                out.push(0); // Add null terminator to the string
                out.extend_from_slice(next_hop.as_ref());
                out.extend_from_slice(next_hop.to_peerid_str().as_bytes());
                out.push(0); // Add null terminator to the string
                out.extend_from_slice(ack_challenge.as_ref());
                out.push(ticket.len() as u8);
                out.extend_from_slice(ticket.as_ref());
                out.push(num_surbs);
                out.push(if is_forwarded { 1 } else { 0 });
                out.push(signals.bits());
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
                out.extend_from_slice(ack.leak().as_ref());
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
                        info,
                    },
                ticket,
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
                out.push(ticket.len() as u8);
                out.extend_from_slice(ticket.as_ref());
                out.push(info.packet_signals.bits());
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
                        ack_key,
                    },
                ticket,
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
                out.extend_from_slice(ack_key.as_ref());
                out.push(ticket.len() as u8);
                out.extend_from_slice(ticket.as_ref());
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
                ..
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

        Self {
            direction,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default(),
            orig_len: HOPR_PACKET_SIZE as u32,
            data: out.into_boxed_slice(),
        }
    }
}

#[cfg(test)]
mod tests {
    use futures::{SinkExt, pin_mut};
    use hex_literal::hex;
    use hopr_crypto_packet::prelude::PacketSignal;
    use hopr_crypto_random::Randomizable;
    use hopr_crypto_types::{
        prelude::{ChainKeypair, Keypair, OffchainKeypair, SimplePseudonym},
        types::{HalfKey, Hash},
    };
    use hopr_internal_types::prelude::{HoprPseudonym, TicketBuilder, VerifiedAcknowledgement, WinningProbability};
    use hopr_network_types::types::SealedHost;
    use hopr_primitive_types::{primitives::EthereumChallenge, traits::BytesEncodable};
    use hopr_protocol_app::prelude::ApplicationData;
    use hopr_protocol_session::types::*;
    use hopr_protocol_start::{KeepAliveMessage, StartErrorReason, StartErrorType, StartEstablished, StartInitiation};
    use hopr_transport_probe::content::{NeighborProbe, PathTelemetry};
    use hopr_transport_session::{ByteCapabilities, Capability, SessionId, SessionTarget};

    use super::*;

    #[tokio::test]
    async fn test_file_capture() -> anyhow::Result<()> {
        let me = *OffchainKeypair::random().public();

        std::env::set_var("HOPR_CAPTURE_PATH_TRIGGER", "/tmp/start_capturing");
        File::create("/tmp/start_capturing")?;
        let (pcap, ah) = packet_capture_channel(Box::new(File::create("test.pcap").and_then(PcapPacketWriter::new)?));
        pin_mut!(pcap);

        let packet = IncomingPacket::Final {
            packet_tag: hopr_crypto_random::random_bytes(),
            previous_hop: *OffchainKeypair::random().public(),
            sender: SimplePseudonym::random(),
            plain_text: ApplicationData::new(10u64, &hex!("deadbeef"))?.to_bytes(),
            ack_key: HalfKey::random(),
            info: Default::default(),
        };

        let ticket = TicketBuilder::default()
            .amount(10)
            .channel_id(Hash::create(&[b"test"]))
            .eth_challenge(EthereumChallenge::default())
            .win_prob(WinningProbability::try_from_f64(0.5)?)
            .channel_epoch(1)
            .index(10)
            .index_offset(1)
            .build_signed(&ChainKeypair::random(), &Hash::default())?
            .leak()
            .into_encoded();

        let _ = pcap
            .send(
                PacketBeforeTransit::IncomingPacket {
                    me,
                    packet: &packet,
                    ticket: ticket.to_vec().into(),
                }
                .into(),
            )
            .await;

        let msg = SessionMessage::<1000>::Segment(Segment {
            frame_id: 1,
            seq_idx: 0,
            seq_flags: SeqIndicator::new_with_flags(1, true),
            data: Box::new(hex!(
                "474554202f20485454502f312e310d0a486f73743a207777772e6578616d706c652e636f6d0d0a557365722d4167656e743a206375726c2f382e372e310d0a4163636570743a202a2f2a0d0a0d0a"
            )),
        });

        let packet = IncomingPacket::Final {
            packet_tag: hopr_crypto_random::random_bytes(),
            previous_hop: *OffchainKeypair::random().public(),
            sender: SimplePseudonym::random(),
            plain_text: ApplicationData::new(1024_u64, msg.into_encoded().into_vec())?.to_bytes(),
            ack_key: HalfKey::random(),
            info: Default::default(),
        };

        let _ = pcap
            .send(
                PacketBeforeTransit::IncomingPacket {
                    me,
                    packet: &packet,
                    ticket: ticket.to_vec().into(),
                }
                .into(),
            )
            .await;

        let msg = SessionMessage::<1000>::Segment(Segment {
            frame_id: 2,
            seq_idx: 0,
            seq_flags: SeqIndicator::new_with_flags(10, false),
            data: Box::new(hex!(
                "474554202f20485454502f312e310d0a486f73743a207777772e6578616d706c652e636f6d0d0a557365722d4167656e743a206375726c2f382e372e310d0a4163636570743a202a2f2a0d0a0d0a"
            )),
        });

        let packet = IncomingPacket::Final {
            packet_tag: hopr_crypto_random::random_bytes(),
            previous_hop: *OffchainKeypair::random().public(),
            sender: SimplePseudonym::random(),
            plain_text: ApplicationData::new(1024_u64, msg.into_encoded().into_vec())?.to_bytes(),
            ack_key: HalfKey::random(),
            info: Default::default(),
        };

        let _ = pcap
            .send(
                PacketBeforeTransit::IncomingPacket {
                    me,
                    packet: &packet,
                    ticket: ticket.to_vec().into(),
                }
                .into(),
            )
            .await;

        let msg = SessionMessage::<1000>::Acknowledge(FrameAcknowledgements::try_from(vec![1, 2, 100])?);

        let packet = IncomingPacket::Final {
            packet_tag: hopr_crypto_random::random_bytes(),
            previous_hop: *OffchainKeypair::random().public(),
            sender: SimplePseudonym::random(),
            plain_text: ApplicationData::new(1024_u64, msg.into_encoded().into_vec())?.to_bytes(),
            ack_key: HalfKey::random(),
            info: Default::default(),
        };

        let _ = pcap
            .send(
                PacketBeforeTransit::IncomingPacket {
                    me,
                    packet: &packet,
                    ticket: ticket.to_vec().into(),
                }
                .into(),
            )
            .await;

        let msg = SessionMessage::<1000>::Request(SegmentRequest::from_iter([
            (15 as FrameId, [0b10100001].into()),
            (11 as FrameId, [0b00100001].into()),
        ]));

        let packet = IncomingPacket::Final {
            packet_tag: hopr_crypto_random::random_bytes(),
            previous_hop: *OffchainKeypair::random().public(),
            sender: SimplePseudonym::random(),
            plain_text: ApplicationData::new(1024_u64, msg.into_encoded().into_vec())?.to_bytes(),
            ack_key: HalfKey::random(),
            info: Default::default(),
        };

        let _ = pcap
            .send(
                PacketBeforeTransit::IncomingPacket {
                    me,
                    packet: &packet,
                    ticket: ticket.to_vec().into(),
                }
                .into(),
            )
            .await;

        let kp = OffchainKeypair::random();
        let packet = IncomingPacket::Acknowledgement {
            packet_tag: hopr_crypto_random::random_bytes(),
            previous_hop: *kp.public(),
            ack: VerifiedAcknowledgement::random(&kp).leak(),
        };

        let _ = pcap
            .send(
                PacketBeforeTransit::IncomingPacket {
                    me,
                    packet: &packet,
                    ticket: ticket.to_vec().into(),
                }
                .into(),
            )
            .await;

        let packet = IncomingPacket::Forwarded {
            packet_tag: hopr_crypto_random::random_bytes(),
            previous_hop: *OffchainKeypair::random().public(),
            next_hop: *OffchainKeypair::random().public(),
            data: Box::new([0x08]),
            ack_key: HalfKey::random(),
        };

        let _ = pcap
            .send(
                PacketBeforeTransit::IncomingPacket {
                    me,
                    packet: &packet,
                    ticket: ticket.to_vec().into(),
                }
                .into(),
            )
            .await;

        let hk = HalfKey::random().to_challenge()?;
        let packet = PacketBeforeTransit::OutgoingPacket {
            me,
            next_hop: *OffchainKeypair::random().public(),
            ack_challenge: hk.as_ref().into(),
            data: ApplicationData::try_from(hopr_transport_probe::content::Message::Telemetry(PathTelemetry {
                id: hopr_crypto_random::random_bytes(),
                path: hopr_crypto_random::random_bytes(),
                timestamp: 123456789_u128,
            }))?
            .to_bytes()
            .to_vec()
            .into(),
            ticket: ticket.to_vec().into(),
            num_surbs: 2,
            is_forwarded: false,
            signals: PacketSignal::OutOfSurbs.into(),
        };

        let _ = pcap.send(packet.into()).await;

        let hk = HalfKey::random().to_challenge()?;
        let packet = PacketBeforeTransit::OutgoingPacket {
            me,
            next_hop: *OffchainKeypair::random().public(),
            ack_challenge: hk.as_ref().into(),
            data: ApplicationData::try_from(hopr_transport_probe::content::Message::Probe(
                NeighborProbe::random_nonce(),
            ))?
            .to_bytes()
            .to_vec()
            .into(),
            ticket: ticket.to_vec().into(),
            num_surbs: 2,
            is_forwarded: false,
            signals: None.into(),
        };

        let _ = pcap.send(packet.into()).await;

        let hk = HalfKey::random().to_challenge()?;
        let packet = PacketBeforeTransit::OutgoingPacket {
            me,
            next_hop: *OffchainKeypair::random().public(),
            ack_challenge: hk.as_ref().into(),
            data: ApplicationData::try_from(hopr_transport_probe::content::Message::Probe(NeighborProbe::Pong(
                hopr_crypto_random::random_bytes(),
            )))?
            .to_bytes()
            .to_vec()
            .into(),
            ticket: ticket.to_vec().into(),
            num_surbs: 2,
            is_forwarded: false,
            signals: None.into(),
        };

        let _ = pcap.send(packet.into()).await;

        let hk = HalfKey::random().to_challenge()?;
        let packet = PacketBeforeTransit::OutgoingPacket {
            me,
            next_hop: *OffchainKeypair::random().public(),
            ack_challenge: hk.as_ref().into(),
            data: ApplicationData::try_from(hopr_protocol_start::StartProtocol::<
                SessionId,
                SessionTarget,
                ByteCapabilities,
            >::StartSession(StartInitiation {
                challenge: 0x01234567_89abcdef,
                target: SessionTarget::UdpStream(SealedHost::Plain("some-dns-name.com:1234".parse()?)),
                capabilities: (Capability::Segmentation | Capability::NoRateControl).into(),
                additional_data: 0x12345678,
            }))?
            .to_bytes()
            .into_vec()
            .into(),
            ticket: ticket.to_vec().into(),
            num_surbs: 2,
            is_forwarded: false,
            signals: None.into(),
        };

        let _ = pcap.send(packet.into()).await;

        let hk = HalfKey::random().to_challenge()?;
        let packet = PacketBeforeTransit::OutgoingPacket {
            me,
            next_hop: *OffchainKeypair::random().public(),
            ack_challenge: hk.as_ref().into(),
            data: ApplicationData::try_from(hopr_protocol_start::StartProtocol::<
                SessionId,
                SessionTarget,
                ByteCapabilities,
            >::SessionError(StartErrorType {
                challenge: 0x01234567_89abcdef,
                reason: StartErrorReason::Busy,
            }))?
            .to_bytes()
            .into_vec()
            .into(),
            ticket: ticket.to_vec().into(),
            num_surbs: 2,
            is_forwarded: false,
            signals: None.into(),
        };

        let _ = pcap.send(packet.into()).await;

        let hk = HalfKey::random().to_challenge()?;
        let packet = PacketBeforeTransit::OutgoingPacket {
            me,
            next_hop: *OffchainKeypair::random().public(),
            ack_challenge: hk.as_ref().into(),
            data: ApplicationData::try_from(hopr_protocol_start::StartProtocol::<
                SessionId,
                SessionTarget,
                ByteCapabilities,
            >::SessionEstablished(StartEstablished {
                orig_challenge: 0x01234567_89abcdef,
                session_id: SessionId::new(1234u64, HoprPseudonym::random()),
            }))?
            .to_bytes()
            .into_vec()
            .into(),
            ticket: ticket.to_vec().into(),
            num_surbs: 2,
            is_forwarded: false,
            signals: None.into(),
        };

        let _ = pcap.send(packet.into()).await;

        let hk = HalfKey::random().to_challenge()?;
        let packet = PacketBeforeTransit::OutgoingPacket {
            me,
            next_hop: *OffchainKeypair::random().public(),
            ack_challenge: hk.as_ref().into(),
            data: ApplicationData::try_from(hopr_protocol_start::StartProtocol::<
                SessionId,
                SessionTarget,
                ByteCapabilities,
            >::KeepAlive(KeepAliveMessage {
                session_id: SessionId::new(1234u64, HoprPseudonym::random()),
                flags: 0xff,
                additional_data: 0xffffffff,
            }))?
            .to_bytes()
            .into_vec()
            .into(),
            ticket: ticket.to_vec().into(),
            num_surbs: 2,
            is_forwarded: false,
            signals: None.into(),
        };

        let _ = pcap.send(packet.into()).await;

        let hk = HalfKey::random().to_challenge()?;
        let packet = PacketBeforeTransit::OutgoingPacket {
            me,
            next_hop: *OffchainKeypair::random().public(),
            ack_challenge: hk.as_ref().into(),
            data: hex!("deadbeefcafebabe").to_vec().into(),
            ticket: ticket.to_vec().into(),
            num_surbs: 0,
            is_forwarded: true,
            signals: None.into(),
        };

        let _ = pcap.send(packet.into()).await;

        let kp = OffchainKeypair::random();

        let packet = PacketBeforeTransit::OutgoingAck {
            me,
            next_hop: *kp.public(),
            ack: VerifiedAcknowledgement::random(&kp),
            is_random: false,
        };

        let _ = pcap.send(packet.into()).await;

        let packet = PacketBeforeTransit::OutgoingAck {
            me,
            next_hop: *kp.public(),
            ack: VerifiedAcknowledgement::random(&kp),
            is_random: true,
        };

        let _ = pcap.send(packet.into()).await;

        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        ah.abort();
        Ok(())
    }
}
