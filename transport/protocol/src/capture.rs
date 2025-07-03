use std::fs::File;

use futures::{FutureExt, StreamExt};
use hopr_db_api::prelude::{IncomingPacket, OutgoingPacket};
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
enum PacketDirection {
    Incoming,
    Outgoing,
}

trait PacketWriter {
    fn write_packet(&mut self, packet: &[u8], direction: PacketDirection) -> std::io::Result<()>;
}

struct PcapPacketWriter(PcapNgWriter<File>);

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
        self
            .0
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

struct UdpPacketDump(std::net::UdpSocket);

impl UdpPacketDump {
    pub fn new(addr: std::net::SocketAddr) -> std::io::Result<Self> {
        Ok(Self(std::net::UdpSocket::bind(addr)?))
    }
}

impl PacketWriter for UdpPacketDump {
    fn write_packet(&mut self, packet: &[u8], _direction: PacketDirection) -> std::io::Result<()> {
        let mut remaining = packet.len();
        while remaining > 0 {
            remaining -= self.0.send(packet)?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub(crate) struct PacketCapture {
    capture_tx: futures::channel::mpsc::Sender<(PacketDirection, Box<[u8]>)>,
}

impl PacketCapture {
    fn start_queue<W: PacketWriter + Send + 'static>(
        writer: W,
        capture_rx: futures::channel::mpsc::Receiver<(PacketDirection, Box<[u8]>)>,
    ) {
        let writer = std::sync::Arc::new(std::sync::Mutex::new(writer));
        hopr_async_runtime::prelude::spawn(capture_rx.for_each(move |(dir, data)| {
            let writer = writer.clone();
            hopr_async_runtime::prelude::spawn_blocking(move || {
                writer
                    .lock()
                    .map_err(|_| std::io::Error::other("lock poisoned"))
                    .and_then(|mut w| w.write_packet(&data, dir))
            })
            .map(|_| ())
        }));
    }

    pub fn new_file(file: File) -> std::io::Result<Self> {
        let (capture_tx, capture_rx) = futures::channel::mpsc::channel(20000);

        let pcap = PcapPacketWriter::new(file)?;
        Self::start_queue(pcap, capture_rx);

        tracing::debug!("packet capture started to a file");
        Ok(Self { capture_tx })
    }

    pub fn new_udp<A: std::net::ToSocketAddrs>(addr: A) -> std::io::Result<Self> {
        let (capture_tx, capture_rx) = futures::channel::mpsc::channel(20000);

        let addr = addr
            .to_socket_addrs()?
            .next()
            .ok_or(std::io::Error::other("no address to bind"))?;
        let udp = UdpPacketDump::new(addr)?;

        Self::start_queue(udp, capture_rx);

        tracing::debug!(%addr, "packet capture started to udp socket");
        Ok(Self { capture_tx })
    }

    pub fn capture_incoming(&mut self, packet: &IncomingPacket) -> std::io::Result<()> {
        self.capture_tx
            .try_send((PacketDirection::Incoming, encode_incoming_packet_capture(packet)))
            .map_err(std::io::Error::other)
    }

    pub fn capture_outgoing(&mut self, packet: &OutgoingPacket) -> std::io::Result<()> {
        self.capture_tx
            .try_send((PacketDirection::Outgoing, encode_outgoing_packet_capture(packet)))
            .map_err(std::io::Error::other)
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
            out.extend_from_slice(previous_hop.to_peerid_str().as_bytes());
            out.push(0); // Add null terminator to the string
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
            out.extend_from_slice(previous_hop.to_peerid_str().as_bytes());
            out.push(0); // Add null terminator to the string
            out.extend_from_slice(next_hop.as_ref());
            out.extend_from_slice(next_hop.to_peerid_str().as_bytes());
            out.push(0); // Add null terminator to the string
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
    out.extend_from_slice(packet.next_hop.to_peerid_str().as_bytes());
    out.push(0); // Add null terminator to the string
    out.extend_from_slice(packet.ack_challenge.as_ref());
    out.extend_from_slice((packet.data.len() as u16).to_be_bytes().as_ref());
    out.extend_from_slice(packet.data.as_ref());

    out.into_boxed_slice()
}

#[cfg(test)]
mod tests {
    use hex_literal::hex;
    use hopr_crypto_random::Randomizable;
    use hopr_crypto_types::prelude::{Keypair, OffchainKeypair, SimplePseudonym};
    use hopr_crypto_types::types::HalfKey;
    use hopr_internal_types::prelude::Acknowledgement;
    use hopr_network_types::prelude::protocol::{FrameAcknowledgements, SegmentRequest, SessionMessage};
    use hopr_network_types::prelude::{FrameInfo, Segment};
    use hopr_transport_packet::prelude::ApplicationData;
    use super::*;

    #[tokio::test]
    async fn test_file_capture() -> anyhow::Result<()> {
        let mut pcap = PacketCapture::new_file(File::create("test.pcap")?)?;

        let packet = IncomingPacket::Final {
            packet_tag: hopr_crypto_random::random_bytes(),
            previous_hop: *OffchainKeypair::random().public(),
            sender: SimplePseudonym::random(),
            plain_text: ApplicationData::new(10u64, &hex!("deadbeef")).to_bytes(),
            ack_key: HalfKey::random(),
        };

        pcap.capture_incoming(&packet)?;

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

        pcap.capture_incoming(&packet)?;

        let msg = SessionMessage::<1000>::Acknowledge(FrameAcknowledgements::from(vec![1,2,100]));

        let packet = IncomingPacket::Final {
            packet_tag: hopr_crypto_random::random_bytes(),
            previous_hop: *OffchainKeypair::random().public(),
            sender: SimplePseudonym::random(),
            plain_text: ApplicationData::new_from_owned(1024_u64, msg.into_encoded()).to_bytes(),
            ack_key: HalfKey::random(),
        };

        pcap.capture_incoming(&packet)?;

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
            }
        ]));

        let packet = IncomingPacket::Final {
            packet_tag: hopr_crypto_random::random_bytes(),
            previous_hop: *OffchainKeypair::random().public(),
            sender: SimplePseudonym::random(),
            plain_text: ApplicationData::new_from_owned(1024_u64, msg.into_encoded()).to_bytes(),
            ack_key: HalfKey::random(),
        };

        pcap.capture_incoming(&packet)?;

        let packet = IncomingPacket::Forwarded {
            packet_tag: hopr_crypto_random::random_bytes(),
            previous_hop: *OffchainKeypair::random().public(),
            next_hop: *OffchainKeypair::random().public(),
            data: Box::new([0x08]),
            ack: Acknowledgement::random(&OffchainKeypair::random())
        };

        pcap.capture_incoming(&packet)?;

        let packet = OutgoingPacket {
            next_hop: *OffchainKeypair::random().public(),
            ack_challenge: HalfKey::random().to_challenge(),
            data: ApplicationData::new(0u64, &hex!("cafebabe01")).to_bytes(),
        };

        pcap.capture_outgoing(&packet)?;

        let packet = OutgoingPacket {
            next_hop: *OffchainKeypair::random().public(),
            ack_challenge: HalfKey::random().to_challenge(),
            data: ApplicationData::new(1u64, &hex!("0004babe02")).to_bytes(),
        };

        pcap.capture_outgoing(&packet)?;


        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        Ok(())
    }

}
