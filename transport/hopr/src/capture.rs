//! Diagnostic packet capture.
//!
//! Packets are written as a pcapng file with link type [`DataLink::USER0`] (LINKTYPE_USER0, 147),
//! one *capture frame* per block. A capture frame is not a wire packet: it is the node's own
//! dissection of a packet at the moment it entered or left the transport, so that a reader can see
//! the plaintext the node saw without holding any key.
//!
//! # Frame layout
//!
//! Every frame starts with [`CAPTURE_FORMAT_VERSION`] followed by a [`PacketType`] byte. All
//! multi-byte integers are big-endian; `peerid` fields are NUL-terminated ASCII.
//!
//! ```text
//! Final     (0): tag(16) prev(32) prev_peerid(z) me(32) me_peerid(z) pseudonym(10)
//!                ack_key(32) signals(1) data_len(2) data(N)
//! Forwarded (1): tag(16) prev(32) prev_peerid(z) next(32) next_peerid(z)
//!                ack_key(32) ticket_len(1) ticket(ticket_len) data_len(2) data(N)
//! Outgoing  (2): me(32) me_peerid(z) next(32) next_peerid(z) ack_challenge(33)
//!                ticket_len(1) ticket(ticket_len) num_surbs(1) is_forwarded(1)
//!                signals(1) data_len(2) data(N)
//! InAck     (3): tag(16) prev(32) prev_peerid(z) me(32) me_peerid(z) count(2) acks(count*96)
//! OutAck    (4): me(32) me_peerid(z) next(32) next_peerid(z) is_random(1)
//!                count(2) acks(count*96)
//! ```
//!
//! `data` of a `Final`/`Outgoing` frame is an `ApplicationData` (8-byte tag + payload); `data` of a
//! `Forwarded` frame is the opaque onion payload destined for the next hop.
//!
//! **Any change to this layout must bump [`CAPTURE_FORMAT_VERSION`] and be mirrored in
//! `transport/hopr/hopr.lua`.** The `dissector` tests below enforce the second half of that.

use std::{borrow::Cow, fs::File};

use bytes::Bytes;
use futures::StreamExt;
use hopr_api::types::{
    crypto::{prelude::HalfKeyChallenge, types::OffchainPublicKey},
    internal::{
        prelude::{Ticket, VerifiedAcknowledgement},
        routing::ResolvedTransportRouting,
    },
    primitive::prelude::{BytesEncodable, BytesRepresentable},
};
use hopr_crypto_packet::{HoprSurb, prelude::PacketSignals};
use hopr_protocol_hopr::{
    IncomingAcknowledgementPacket, IncomingFinalPacket, IncomingForwardedPacket, IncomingPacket, IncomingPacketError,
    OutgoingPacket, PacketDecoder, PacketEncoder,
};
use hopr_utils::runtime::AbortHandle;
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

use crate::PeerId;

/// Version of the capture frame layout documented at the module level.
///
/// Written as the first byte of every captured frame so that a reader can refuse a file it does not
/// understand instead of silently mis-parsing it. Bump this whenever the layout changes.
pub const CAPTURE_FORMAT_VERSION: u8 = 1;

/// Link type the capture is written with.
///
/// Named so the dissector-sync test can emit both the LINKTYPE number that lands in the file and
/// the `WTAP_ENCAP_` name Wireshark keys its dissector table by, without either being restated.
pub const CAPTURE_LINK_TYPE: DataLink = DataLink::USER0;

/// Width of the `ack_challenge` field of an `Outgoing` frame.
///
/// Fixed rather than "however many bytes the challenge happened to be": a relayed packet has no
/// challenge of its own and used to contribute zero bytes here, which left the field's presence
/// decidable only by `is_forwarded` — a byte that comes *later* in the frame. Relayed packets now
/// write [`ZERO_ACK_CHALLENGE`] instead.
const ACK_CHALLENGE_SIZE: usize = HalfKeyChallenge::SIZE;

/// Placeholder written in place of the `ack_challenge` of a relayed packet.
const ZERO_ACK_CHALLENGE: [u8; ACK_CHALLENGE_SIZE] = [0u8; ACK_CHALLENGE_SIZE];

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
                linktype: CAPTURE_LINK_TYPE,
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

/// Creates a queue that processes captured packets into a [`PacketWriter`].
pub fn packet_capture_channel(
    writer: Box<dyn PacketWriter + Send>,
) -> (crossfire::MAsyncTx<crossfire::mpsc::Array<CapturedPacket>>, AbortHandle) {
    let (sender, receiver) = crossfire::mpsc::bounded_async::<CapturedPacket>(20_000);
    let writer = std::sync::Arc::new(std::sync::Mutex::new(writer));
    let ah = hopr_utils::spawn_as_abortable!(async move {
        let mut rx_stream = receiver.into_stream();
        while let Some(packet) = rx_stream.next().await {
            let writer = writer.clone();
            match hopr_utils::runtime::prelude::spawn_blocking(move || {
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
    });
    (sender, ah)
}

/// Discriminant of a capture frame, written as its second byte.
///
/// [`strum::VariantArray`] is derived so that the dissector tests can walk every variant and fail to
/// compile when one is added without the Lua dissector being told about it.
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, strum::VariantArray)]
enum PacketType {
    Final = 0,
    Forwarded = 1,
    Outgoing = 2,
    InAck = 3,
    OutAck = 4,
}

/// Represents a customized dissection of a HOPR packet before it goes into the transport.
enum PacketBeforeTransit<'a> {
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
        acks: Vec<VerifiedAcknowledgement>,
        is_random: bool,
    },
    IncomingPacket {
        me: OffchainPublicKey,
        packet: &'a IncomingPacket,
    },
}

impl<'a> From<PacketBeforeTransit<'a>> for CapturedPacket {
    fn from(value: PacketBeforeTransit<'a>) -> Self {
        let mut out = vec![CAPTURE_FORMAT_VERSION];
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

                // Always exactly ACK_CHALLENGE_SIZE bytes, whatever the caller handed over: the
                // field has no length prefix, so a short one would desynchronise every field after
                // it. Padded rather than rejected because a capture must never be able to fail a
                // packet that the transport itself accepted.
                debug_assert_eq!(
                    ack_challenge.len(),
                    ACK_CHALLENGE_SIZE,
                    "captured ack challenge must be of a fixed width"
                );
                let mut challenge = ZERO_ACK_CHALLENGE;
                let copied = ack_challenge.len().min(ACK_CHALLENGE_SIZE);
                challenge[..copied].copy_from_slice(&ack_challenge[..copied]);
                out.extend_from_slice(&challenge);

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
                acks,
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
                out.extend((acks.len() as u16).to_be_bytes());
                acks.into_iter()
                    .for_each(|ack| out.extend_from_slice(ack.leak().as_ref()));
                direction = PacketDirection::Outgoing;
            }
            PacketBeforeTransit::IncomingPacket {
                me,
                packet: IncomingPacket::Final(final_packet),
            } => {
                let IncomingFinalPacket {
                    packet_tag,
                    previous_hop,
                    sender,
                    plain_text,
                    ack_key,
                    info,
                    // Which SURB the reply came back on is routing telemetry, not wire content;
                    // the capture format records the packet as it travelled.
                    replied_on_surb: _,
                } = final_packet.as_ref();

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
                out.push(info.packet_signals.bits());
                out.extend_from_slice((plain_text.len() as u16).to_be_bytes().as_ref());
                out.extend_from_slice(plain_text.as_ref());
            }
            PacketBeforeTransit::IncomingPacket {
                packet: IncomingPacket::Forwarded(fwd_packet),
                ..
            } => {
                let IncomingForwardedPacket {
                    packet_tag,
                    previous_hop,
                    next_hop,
                    data,
                    received_ticket: ticket,
                    ack_key_prev_hop: ack_key,
                    ..
                } = fwd_packet.as_ref();
                let ticket = (*ticket.verified_ticket()).into_encoded();
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
                packet: IncomingPacket::Acknowledgement(ack_packet),
                ..
            } => {
                let IncomingAcknowledgementPacket {
                    packet_tag,
                    previous_hop,
                    received_acks,
                } = ack_packet.as_ref();
                out.push(PacketType::InAck as u8);
                out.extend_from_slice(packet_tag);
                out.extend_from_slice(previous_hop.as_ref());
                out.extend_from_slice(previous_hop.to_peerid_str().as_bytes());
                out.push(0); // Add null terminator to the string
                out.extend_from_slice(me.as_ref());
                out.extend_from_slice(me.to_peerid_str().as_bytes());
                out.push(0); // Add null terminator to the string
                out.extend((received_acks.len() as u16).to_be_bytes());
                received_acks.iter().for_each(|ack| out.extend_from_slice(ack.as_ref()));
            }
        }

        Self {
            direction,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default(),
            orig_len: hopr_crypto_packet::prelude::HoprPacket::SIZE as u32,
            data: out.into_boxed_slice(),
        }
    }
}

pub struct CapturePacketCodec<C> {
    inner: std::sync::Arc<C>,
    packet_key: OffchainPublicKey,
    sender: crossfire::MAsyncTx<crossfire::mpsc::Array<CapturedPacket>>,
}

impl<C> Clone for CapturePacketCodec<C> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            packet_key: self.packet_key,
            sender: self.sender.clone(),
        }
    }
}

impl<C> CapturePacketCodec<C> {
    pub fn new(
        inner: C,
        packet_key: OffchainPublicKey,
        sender: crossfire::MAsyncTx<crossfire::mpsc::Array<CapturedPacket>>,
    ) -> Self {
        Self {
            inner: std::sync::Arc::new(inner),
            packet_key,
            sender,
        }
    }
}

fn inspect_ticket_data_in_packet(raw_packet: &[u8]) -> &[u8] {
    if raw_packet.len() >= Ticket::SIZE {
        &raw_packet[raw_packet.len() - Ticket::SIZE..]
    } else {
        &[]
    }
}

impl<C: PacketDecoder + Send + Sync> PacketDecoder for CapturePacketCodec<C> {
    type Error = C::Error;

    fn decode(&self, peer: PeerId, data: Bytes) -> Result<IncomingPacket, IncomingPacketError<Self::Error>> {
        let packet = self.inner.decode(peer, data)?;

        if let Err(error) = self.sender.try_send(
            PacketBeforeTransit::IncomingPacket {
                me: self.packet_key,
                packet: &packet,
            }
            .into(),
        ) {
            tracing::debug!(%error, "failed to send incoming packet to capture");
        }

        if let IncomingPacket::Forwarded(fwd_packet) = &packet {
            let IncomingForwardedPacket { next_hop, data, .. } = fwd_packet.as_ref();

            if let Err(error) = self.sender.try_send(
                PacketBeforeTransit::OutgoingPacket {
                    me: self.packet_key,
                    next_hop: *next_hop,
                    num_surbs: 0,
                    is_forwarded: true,
                    data: data.as_ref().into(),
                    // A relayed packet carries no challenge of ours; the field is fixed-width, so a
                    // placeholder goes in rather than nothing at all.
                    ack_challenge: Cow::Borrowed(&ZERO_ACK_CHALLENGE),
                    signals: None.into(),
                    ticket: inspect_ticket_data_in_packet(data.as_ref()).into(),
                }
                .into(),
            ) {
                tracing::debug!(%error, "failed to send forwarded packet to capture");
            }
        }

        Ok(packet)
    }
}

impl<C: PacketEncoder + Send + Sync> PacketEncoder for CapturePacketCodec<C> {
    type Error = C::Error;

    fn encode_packet<T: AsRef<[u8]> + Send + 'static, S: Into<PacketSignals> + Send + 'static>(
        &self,
        data: T,
        routing: ResolvedTransportRouting<HoprSurb>,
        signals: S,
        generation: Option<u8>,
    ) -> Result<OutgoingPacket, Self::Error> {
        let data_clone = data.as_ref().to_vec();
        let signals = signals.into();
        let num_surbs = routing.count_return_paths() as u8;

        let packet = self.inner.encode_packet(data, routing, signals, generation)?;

        if let Err(error) = self.sender.try_send(
            PacketBeforeTransit::OutgoingPacket {
                me: self.packet_key,
                next_hop: packet.next_hop,
                num_surbs,
                is_forwarded: false,
                data: data_clone.into(),
                ack_challenge: packet.ack_challenge.as_ref().into(),
                signals,
                ticket: inspect_ticket_data_in_packet(packet.data.as_ref()).into(),
            }
            .into(),
        ) {
            tracing::debug!(%error, "failed to send outgoing packet to capture");
        }

        Ok(packet)
    }

    fn encode_acknowledgements(
        &self,
        acks: &[VerifiedAcknowledgement],
        destination: &OffchainPublicKey,
    ) -> Result<OutgoingPacket, Self::Error> {
        let packet_ack = self.inner.encode_acknowledgements(acks, destination)?;

        if let Err(error) = self.sender.try_send(
            PacketBeforeTransit::OutgoingAck {
                me: self.packet_key,
                is_random: false,
                next_hop: *destination,
                acks: acks.to_vec(),
            }
            .into(),
        ) {
            tracing::debug!(%error, "failed to send acknowledgement packet to capture");
        }

        Ok(packet_ack)
    }
}

#[cfg(test)]
mod fixtures {
    //! A deterministic corpus of capture frames, one per frame kind and one per application-layer
    //! message the dissector has to recognise.
    //!
    //! Deterministic on purpose: the corpus is snapshotted byte-for-byte by
    //! [`super::dissector`], so nothing here may use `random()`. Every keypair, pseudonym and
    //! ticket is derived from a fixed constant, and the two signature schemes involved (Ed25519 and
    //! RFC 6979 ECDSA) are themselves deterministic.

    use hopr_api::types::{
        crypto::{
            prelude::{ChainKeypair, Keypair, OffchainKeypair, SimplePseudonym},
            types::{HalfKey, Hash},
        },
        internal::prelude::{TicketBuilder, VerifiedAcknowledgement, VerifiedTicket, WinningProbability},
        primitive::{
            prelude::{Address, BytesRepresentable},
            primitives::EthereumChallenge,
            traits::BytesEncodable,
        },
    };
    use hopr_crypto_packet::prelude::{HoprPixCommitmentProof, HoprPixGroupElement, PacketSignal};
    use hopr_protocol_app::prelude::{ApplicationData, ReservedTag};
    use hopr_protocol_hopr::{
        IncomingAcknowledgementPacket, IncomingFinalPacket, IncomingForwardedPacket, IncomingPacket,
    };
    use hopr_protocol_pix::{PixParams, SsaIndex};
    use hopr_protocol_session::types::{
        FrameAcknowledgements, FrameId, Segment, SegmentRequest, SeqIndicator, SessionMessage,
    };
    use hopr_protocol_start::{
        ErrorIdentifier, KeepAliveFlag, KeepAliveMessage, SsaClientCommitmentMessage, SsaServerCommitmentMessage,
        StartErrorReason, StartErrorType, StartEstablished, StartInitiation,
    };
    use hopr_transport_probe::types::{NeighborProbe, PathTelemetry};
    use hopr_transport_session::{
        Capability, HoprPixDepositPayload, HoprSessionCapabilities, HoprStartProtocol, LOCAL_PIX_SUITE, SessionId,
        SessionTarget,
    };
    use hopr_utils::network_types::types::SealedHost;

    use crate::capture::{CapturedPacket, PacketBeforeTransit};

    /// Session MTU the fixture Session messages are built for. Any value works; it only bounds the
    /// zero padding of the request/acknowledgement messages.
    pub(super) const SESSION_MTU: usize = 466;

    /// What the Lua dissector is expected to report for one fixture frame.
    ///
    /// Only the discriminants: the point is to prove the dissector walked to the right offsets and
    /// dispatched to the right sub-protocol, not to restate the corpus a second time.
    #[derive(Debug, Default, Clone, Copy)]
    pub(super) struct Expect {
        pub hopr_type: u8,
        pub appdata_tag: Option<u64>,
        pub start_type: Option<u8>,
        pub session_type: Option<u8>,
        pub probe_type: Option<u8>,
    }

    pub(super) struct Fixture {
        pub name: &'static str,
        pub packet: CapturedPacket,
        pub expect: Expect,
    }

    fn offchain_keypair(seed: u8) -> OffchainKeypair {
        OffchainKeypair::from_secret(&[seed; 32]).expect("fixture offchain secret must be valid")
    }

    fn chain_keypair(seed: u8) -> ChainKeypair {
        ChainKeypair::from_secret(&[seed; 32]).expect("fixture chain secret must be valid")
    }

    fn half_key(seed: u8) -> HalfKey {
        HalfKey::try_from([seed; HalfKey::SIZE].as_ref()).expect("fixture half key must be valid")
    }

    fn pseudonym(seed: u8) -> SimplePseudonym {
        SimplePseudonym::from([seed; SimplePseudonym::SIZE])
    }

    fn ticket() -> VerifiedTicket {
        TicketBuilder::default()
            .amount(10)
            .counterparty(Address::new(&[0xa1u8; Address::SIZE]))
            .eth_challenge(EthereumChallenge(Address::new(&[0xc3u8; Address::SIZE])))
            .win_prob(WinningProbability::try_from_f64(0.5).expect("0.5 is a valid winning probability"))
            .channel_epoch(1)
            .index(10)
            .build_signed(&chain_keypair(0x11), &Hash::default())
            .expect("fixture ticket must sign")
    }

    fn group_element(seed: u8) -> HoprPixGroupElement {
        HoprPixGroupElement::try_from(vec![seed; HoprStartProtocol::PIX_COEFF_COMMITMENT_REPR_SIZE].as_slice())
            .expect("fixture group element must be of the build's repr size")
    }

    fn commitment_proof() -> HoprPixCommitmentProof {
        HoprPixCommitmentProof::try_from(vec![0x5au8; HoprStartProtocol::PIX_COMMITMENT_PROOF_SIZE].as_slice())
            .expect("fixture proof must be of the build's proof size")
    }

    fn ssa_index(value: u32) -> SsaIndex {
        SsaIndex::new(value).expect("fixture SSA index must be non-zero")
    }

    fn pix_params() -> PixParams {
        PixParams::try_new(1024, 64, 8, LOCAL_PIX_SUITE).expect("fixture PIX dimensions must be in range")
    }

    /// The Session ID every fixture Start message refers to.
    pub(super) fn session_id() -> SessionId {
        pseudonym(0x7e)
    }

    /// Every Start protocol message, in discriminant order.
    pub(super) fn start_messages() -> Vec<(&'static str, HoprStartProtocol)> {
        vec![
            (
                "start.StartSession",
                HoprStartProtocol::StartSession(StartInitiation {
                    challenge: 0x0123_4567_89ab_cdef,
                    target: SessionTarget::UdpStream(SealedHost::Plain(
                        "some-dns-name.com:1234".parse().expect("fixture host must parse"),
                    )),
                    capabilities: HoprSessionCapabilities::from(
                        Capability::Segmentation | Capability::NoRateControl | Capability::UsePIX,
                    ),
                    additional_data: pix_params().into_additional_data(0x0000_2000),
                }),
            ),
            (
                "start.SessionEstablished",
                HoprStartProtocol::SessionEstablished(StartEstablished {
                    orig_challenge: 0x0123_4567_89ab_cdef,
                    session_id: session_id(),
                }),
            ),
            (
                "start.SsaCommit",
                HoprStartProtocol::SsaCommit(SsaClientCommitmentMessage {
                    session_id: session_id(),
                    ssa_index: ssa_index(7),
                    coefficient_index: 0,
                    commitment_proof: Some(commitment_proof()),
                    coefficient_commitments: [(0, group_element(0x21)), (1, group_element(0x22))]
                        .into_iter()
                        .collect(),
                }),
            ),
            (
                "start.SsaRequest",
                HoprStartProtocol::SsaRequest(SsaServerCommitmentMessage::new(
                    session_id(),
                    pix_params(),
                    [(ssa_index(7), group_element(0x31)), (ssa_index(8), group_element(0x32))],
                    [
                        (ssa_index(7), HoprPixDepositPayload(Box::new([0xd0, 0xd1]))),
                        (ssa_index(8), HoprPixDepositPayload(Box::new([0xd2, 0xd3]))),
                    ],
                )),
            ),
            (
                "start.SsaRequest.recommit",
                HoprStartProtocol::SsaRequest(SsaServerCommitmentMessage::recommit(
                    session_id(),
                    pix_params(),
                    [(ssa_index(7), vec![(0, 31), (64, 95)])],
                )),
            ),
            (
                "start.SessionError.challenge",
                HoprStartProtocol::SessionError(StartErrorType {
                    identifier: ErrorIdentifier::Challenge(0x0123_4567_89ab_cdef),
                    reason: StartErrorReason::Busy,
                }),
            ),
            (
                "start.SessionError.session_id",
                HoprStartProtocol::SessionError(StartErrorType {
                    identifier: ErrorIdentifier::SessionId(session_id()),
                    reason: StartErrorReason::UnacceptablePixParams,
                }),
            ),
            (
                "start.KeepAlive",
                HoprStartProtocol::KeepAlive(KeepAliveMessage {
                    session_id: session_id(),
                    flags: KeepAliveFlag::BalancerTarget.into(),
                    additional_data: 0x0000_0000_0000_1234,
                }),
            ),
        ]
    }

    /// Every Session protocol message, in discriminant order.
    pub(super) fn session_messages() -> Vec<(&'static str, SessionMessage<SESSION_MTU>)> {
        vec![
            (
                "session.Segment",
                SessionMessage::Segment(Segment {
                    frame_id: 1,
                    seq_idx: 0,
                    seq_flags: SeqIndicator::new_with_flags(2, true),
                    // Deliberately not protocol-shaped: the dissector offers the segment payload to
                    // Wireshark's heuristic dissectors, and a payload one of them claimed would make
                    // the expected dissection depend on which ones are installed.
                    data: Box::new(*b"hopr-dissector-fixture-payload"),
                }),
            ),
            (
                "session.Request",
                SessionMessage::Request(SegmentRequest::from_iter([
                    (11 as FrameId, [0b0010_0001].into()),
                    (15 as FrameId, [0b1010_0001].into()),
                ])),
            ),
            (
                "session.Acknowledge",
                SessionMessage::Acknowledge(
                    FrameAcknowledgements::try_from(vec![1 as FrameId, 2, 100])
                        .expect("fixture acknowledgement must fit"),
                ),
            ),
        ]
    }

    /// Every Probe protocol message, in discriminant order.
    pub(super) fn probe_messages() -> Vec<(&'static str, hopr_transport_probe::content::Message)> {
        use hopr_transport_probe::content::Message;
        vec![
            (
                "probe.Telemetry",
                Message::Telemetry(PathTelemetry {
                    id: [0x41; PathTelemetry::ID_SIZE],
                    path: [0x42; PathTelemetry::PATH_SIZE],
                    timestamp: 123_456_789,
                }),
            ),
            ("probe.Probe.Ping", Message::Probe(NeighborProbe::Ping([0x51; 32]))),
            ("probe.Probe.Pong", Message::Probe(NeighborProbe::Pong([0x52; 32]))),
        ]
    }

    /// Wraps `data` into an outgoing frame originated by this node.
    fn outgoing(data: Vec<u8>) -> CapturedPacket {
        PacketBeforeTransit::OutgoingPacket {
            me: *offchain_keypair(0x01).public(),
            next_hop: *offchain_keypair(0x02).public(),
            ack_challenge: half_key(0x03)
                .to_challenge()
                .expect("fixture half key must yield a challenge")
                .as_ref()
                .to_vec()
                .into(),
            data: data.into(),
            ticket: ticket().verified_ticket().into_boxed().into_vec().into(),
            num_surbs: 2,
            is_forwarded: false,
            signals: PacketSignal::SurbDistress.into(),
        }
        .into()
    }

    /// Wraps `data` into an incoming frame addressed to this node.
    fn incoming_final(data: Box<[u8]>) -> CapturedPacket {
        PacketBeforeTransit::IncomingPacket {
            me: *offchain_keypair(0x01).public(),
            packet: &IncomingPacket::Final(
                IncomingFinalPacket {
                    packet_tag: [0x9a; 16],
                    previous_hop: *offchain_keypair(0x02).public(),
                    sender: pseudonym(0x7e),
                    replied_on_surb: None,
                    plain_text: data,
                    ack_key: half_key(0x04),
                    info: Default::default(),
                }
                .into(),
            ),
        }
        .into()
    }

    /// The full corpus, in a stable order. Frame `n` of the generated pcapng is entry `n - 1`.
    pub(super) fn corpus() -> Vec<Fixture> {
        let mut out = Vec::new();

        let mut push =
            |name: &'static str, packet: CapturedPacket, expect: Expect| out.push(Fixture { name, packet, expect });

        // --- application payloads, carried both ways -------------------------------------------
        for (name, msg) in probe_messages() {
            let probe_type = msg.to_bytes()[1];
            let data = ApplicationData::try_from(msg)
                .expect("fixture probe message must fit an application payload")
                .to_bytes();
            push(
                name,
                incoming_final(data),
                Expect {
                    hopr_type: super::PacketType::Final as u8,
                    appdata_tag: Some(ReservedTag::Ping as u64),
                    probe_type: Some(probe_type),
                    ..Default::default()
                },
            );
        }

        for (name, msg) in start_messages() {
            let (tag, encoded) = msg.encode().expect("fixture start message must encode");
            let start_type = encoded[1];
            let data = ApplicationData::new(tag, encoded.into_vec())
                .expect("fixture start message must fit an application payload")
                .to_bytes();
            push(
                name,
                outgoing(data.into_vec()),
                Expect {
                    hopr_type: super::PacketType::Outgoing as u8,
                    appdata_tag: Some(ReservedTag::SessionStart as u64),
                    start_type: Some(start_type),
                    ..Default::default()
                },
            );
        }

        for (name, msg) in session_messages() {
            let encoded = msg.into_encoded();
            let session_type = encoded[1];
            let data = ApplicationData::new(ReservedTag::Session, encoded.into_vec())
                .expect("fixture session message must fit an application payload")
                .to_bytes();
            push(
                name,
                incoming_final(data),
                Expect {
                    hopr_type: super::PacketType::Final as u8,
                    appdata_tag: Some(ReservedTag::Session as u64),
                    session_type: Some(session_type),
                    ..Default::default()
                },
            );
        }

        push(
            "app.opaque",
            incoming_final(
                ApplicationData::new(1234u64, b"opaque application traffic".as_ref())
                    .expect("fixture application payload must fit")
                    .to_bytes(),
            ),
            Expect {
                hopr_type: super::PacketType::Final as u8,
                appdata_tag: Some(1234),
                ..Default::default()
            },
        );

        // --- frame kinds that carry no application data ----------------------------------------
        push(
            "forwarded",
            PacketBeforeTransit::IncomingPacket {
                me: *offchain_keypair(0x01).public(),
                packet: &IncomingPacket::Forwarded(
                    IncomingForwardedPacket {
                        packet_tag: [0x9b; 16],
                        previous_hop: *offchain_keypair(0x02).public(),
                        next_hop: *offchain_keypair(0x03).public(),
                        data: bytes::Bytes::from_static(&[0x08; 64]),
                        ack_challenge: half_key(0x05)
                            .to_challenge()
                            .expect("fixture half key must yield a challenge"),
                        received_ticket: ticket().into_unacknowledged(half_key(0x06)),
                        ack_key_prev_hop: half_key(0x07),
                    }
                    .into(),
                ),
            }
            .into(),
            Expect {
                hopr_type: super::PacketType::Forwarded as u8,
                ..Default::default()
            },
        );

        push(
            "outgoing.relayed",
            PacketBeforeTransit::OutgoingPacket {
                me: *offchain_keypair(0x01).public(),
                next_hop: *offchain_keypair(0x03).public(),
                // The relaying path leaves no challenge of ours; the field is fixed-width all the
                // same, which is the whole point of covering this frame.
                ack_challenge: std::borrow::Cow::Borrowed(&super::ZERO_ACK_CHALLENGE),
                data: vec![0x08; 64].into(),
                ticket: ticket().verified_ticket().into_boxed().into_vec().into(),
                num_surbs: 0,
                is_forwarded: true,
                signals: None.into(),
            }
            .into(),
            Expect {
                hopr_type: super::PacketType::Outgoing as u8,
                ..Default::default()
            },
        );

        let ack_keypair = offchain_keypair(0x02);
        let acks = vec![
            VerifiedAcknowledgement::new(half_key(0x08), &ack_keypair),
            VerifiedAcknowledgement::new(half_key(0x09), &ack_keypair),
        ];

        push(
            "ack.incoming",
            PacketBeforeTransit::IncomingPacket {
                me: *offchain_keypair(0x01).public(),
                packet: &IncomingPacket::Acknowledgement(
                    IncomingAcknowledgementPacket {
                        packet_tag: [0x9c; 16],
                        previous_hop: *ack_keypair.public(),
                        received_acks: acks.iter().map(|ack| ack.leak()).collect(),
                    }
                    .into(),
                ),
            }
            .into(),
            Expect {
                hopr_type: super::PacketType::InAck as u8,
                ..Default::default()
            },
        );

        push(
            "ack.outgoing",
            PacketBeforeTransit::OutgoingAck {
                me: *offchain_keypair(0x01).public(),
                next_hop: *ack_keypair.public(),
                acks: acks.clone(),
                is_random: false,
            }
            .into(),
            Expect {
                hopr_type: super::PacketType::OutAck as u8,
                ..Default::default()
            },
        );

        push(
            "ack.outgoing.random",
            PacketBeforeTransit::OutgoingAck {
                me: *offchain_keypair(0x01).public(),
                next_hop: *ack_keypair.public(),
                acks,
                is_random: true,
            }
            .into(),
            Expect {
                hopr_type: super::PacketType::OutAck as u8,
                ..Default::default()
            },
        );

        out
    }
}

#[cfg(test)]
mod dissector {
    //! Keeps `transport/hopr/hopr.lua` honest about the capture format.
    //!
    //! Three layers, each catching a class of drift the others cannot:
    //!
    //! 1. the `*_name` functions below are exhaustive `match`es over the protocol enums, so adding or removing a
    //!    message *fails to compile* here until someone has looked at the dissector;
    //! 2. [`lua_wire_constants_are_in_sync`] regenerates the constants block of `hopr.lua` from the Rust items and
    //!    diffs it, so a changed version, discriminant, flag or field width fails;
    //! 3. [`capture_frame_layout_is_stable`] snapshots the actual bytes of a deterministic corpus, so a *new field* —
    //!    which layers 1 and 2 cannot see — fails too.
    //!
    //! [`lua_dissects_the_fixture_capture`] then runs the real `tshark` over that corpus, which is
    //! the only layer that proves the Lua loads and reaches the right offsets.

    use std::fmt::Write as _;

    use flagset::Flags as _;
    use hopr_api::types::{
        crypto::types::{HalfKey, OffchainPublicKey, PacketTag},
        internal::prelude::{Acknowledgement, HoprPseudonym, Ticket, WinningProbability},
        primitive::{
            prelude::{Address, BytesRepresentable},
            primitives::EthereumChallenge,
            traits::BytesEncodable,
        },
    };
    use hopr_crypto_packet::{
        HoprSurb,
        prelude::{HoprPacket, PacketSignal},
    };
    use hopr_protocol_app::prelude::{ApplicationData, ReservedTag, Tag};
    use hopr_protocol_pix::{CoefficientIndex, MAX_POLYS_PER_SSA, PolynomialIndex, RawSsaIndex};
    use hopr_protocol_session::types::{
        FrameId, Segment, SegmentRequest, SeqIndicator, SeqNum, SessionMessage, SessionMessageDiscriminants,
    };
    use hopr_protocol_start::{
        ErrorIdentifierDiscriminants, KeepAliveFlag, StartChallenge, StartErrorReason, StartProtocol,
        StartProtocolDiscriminants,
    };
    use hopr_transport_probe::{
        content::Message as ProbeMessage,
        types::{NeighborProbe, PathTelemetry},
    };
    use hopr_transport_session::{Capability, HoprStartProtocol, LOCAL_PIX_SUITE, SESSION_APPLICATION_TAG};
    use strum::VariantArray;

    use super::{
        ACK_CHALLENGE_SIZE, CAPTURE_FORMAT_VERSION, CAPTURE_LINK_TYPE, PacketType, PacketWriter, PcapPacketWriter,
        fixtures, fixtures::SESSION_MTU,
    };

    /// Marks the region of `hopr.lua` this test owns.
    const BEGIN_MARKER: &str = "-- >>> BEGIN GENERATED WIRE CONSTANTS";
    const END_MARKER: &str = "-- <<< END GENERATED WIRE CONSTANTS";

    /// Set to rewrite the generated region in place instead of failing.
    const UPDATE_ENV: &str = "HOPR_UPDATE_DISSECTOR";

    /// Set to keep the generated fixture capture around for inspection in Wireshark.
    const KEEP_CAPTURE_ENV: &str = "HOPR_KEEP_CAPTURE";

    /// Set where a missing `tshark` must fail rather than skip — see
    /// [`lua_dissects_the_fixture_capture`].
    const REQUIRE_TSHARK_ENV: &str = "HOPR_REQUIRE_TSHARK";

    fn dissector_path() -> std::path::PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("hopr.lua")
    }

    // --- layer 1: exhaustive names ---------------------------------------------------------------
    //
    // Every one of these is a total `match`. A variant added to (or removed from) the protocol
    // breaks this file, which is the point: the Lua value-string tables below are built from them.

    fn capture_frame_name(kind: PacketType) -> &'static str {
        match kind {
            PacketType::Final => "Final",
            PacketType::Forwarded => "Forwarded",
            PacketType::Outgoing => "Outgoing",
            PacketType::InAck => "AcknowledgementIn",
            PacketType::OutAck => "AcknowledgementOut",
        }
    }

    fn reserved_tag_name(tag: ReservedTag) -> &'static str {
        match tag {
            ReservedTag::Ping => "Probe",
            ReservedTag::SessionStart => "Start",
            ReservedTag::Session => "Session",
            ReservedTag::Undefined => "Undefined",
        }
    }

    fn start_message_name<I, T, C, G, K, D>(message: &StartProtocol<I, T, C, G, K, D>) -> &'static str {
        match message {
            StartProtocol::StartSession(_) => "StartSession",
            StartProtocol::SessionEstablished(_) => "SessionEstablished",
            StartProtocol::SsaCommit(_) => "SsaCommit",
            StartProtocol::SsaRequest(_) => "SsaRequest",
            StartProtocol::SessionError(_) => "SessionError",
            StartProtocol::KeepAlive(_) => "KeepAlive",
        }
    }

    fn start_error_reason_name(reason: StartErrorReason) -> &'static str {
        match reason {
            StartErrorReason::Unknown => "Unknown",
            StartErrorReason::NoSlotsAvailable => "No slots available",
            StartErrorReason::Busy => "Busy",
            StartErrorReason::UnacceptablePixParams => "Unacceptable PIX parameters",
            StartErrorReason::TargetNotAdmitted => "Target not admitted",
        }
    }

    fn error_identifier_name(identifier: ErrorIdentifierDiscriminants) -> &'static str {
        match identifier {
            ErrorIdentifierDiscriminants::Challenge => "Challenge",
            ErrorIdentifierDiscriminants::SessionId => "SessionId",
        }
    }

    fn session_message_name<const C: usize>(message: &SessionMessage<C>) -> &'static str {
        match message {
            SessionMessage::Segment(_) => "Segment",
            SessionMessage::Request(_) => "SegmentRequest",
            SessionMessage::Acknowledge(_) => "FrameAcknowledgements",
        }
    }

    fn probe_message_name(message: &ProbeMessage) -> &'static str {
        match message {
            ProbeMessage::Telemetry(_) => "Telemetry",
            ProbeMessage::Probe(_) => "Probe",
        }
    }

    fn neighbor_probe_name(probe: &NeighborProbe) -> &'static str {
        match probe {
            NeighborProbe::Ping(_) => "Ping",
            NeighborProbe::Pong(_) => "Pong",
        }
    }

    fn pix_suite_name(suite: hopr_protocol_pix::PixSuite) -> &'static str {
        match suite {
            hopr_protocol_pix::PixSuite::BabyJubJub => "BabyJubJub",
            hopr_protocol_pix::PixSuite::Secp256k1 => "secp256k1",
        }
    }

    /// Widths of a PIX group element and of a commitment proof, per curve suite.
    ///
    /// Only the suite this build was compiled for can be derived from the types — the other curve's
    /// items are behind a Cargo feature this build does not have. The pair is asserted against the
    /// derived one whenever a build of that flavour runs, so each row is checked by the CI job that
    /// can check it.
    fn pix_suite_widths(suite: hopr_protocol_pix::PixSuite) -> (usize, usize) {
        match suite {
            // Compressed Edwards point: 32-byte `y` with the sign of `x` in a spare high bit.
            hopr_protocol_pix::PixSuite::BabyJubJub => (32, 32 + 32),
            // SEC1 compressed point: a parity byte followed by the 32-byte `x`.
            hopr_protocol_pix::PixSuite::Secp256k1 => (33, 33 + 32),
        }
    }

    /// Bit mask of a single flag.
    ///
    /// `flagset` rewrites the declared values into a `From` impl and leaves the variants with
    /// declaration-order discriminants, so `flag as u8` is *not* the mask that travels.
    fn flag_bits<F>(flag: F) -> u8
    where
        F: flagset::Flags<Type = u8>,
        flagset::FlagSet<F>: From<F>,
    {
        flagset::FlagSet::from(flag).bits()
    }

    /// Enumerates a C-like enum through its `FromRepr`, so the table cannot silently miss a variant
    /// that was added without a name being given to it above.
    fn reprs_of<T>(from_repr: impl Fn(u8) -> Option<T>) -> Vec<(u8, T)> {
        (0u8..=u8::MAX)
            .filter_map(|repr| from_repr(repr).map(|value| (repr, value)))
            .collect()
    }

    // --- layer 2: the generated Lua constants ----------------------------------------------------

    /// Minimal Lua table-literal writer. Deliberately not a generic serializer: the output is read
    /// by humans reviewing a diff, so its shape is spelled out at each call site.
    #[derive(Default)]
    struct Lua {
        out: String,
        depth: usize,
    }

    impl Lua {
        fn line(&mut self, text: &str) {
            for _ in 0..self.depth {
                self.out.push_str("  ");
            }
            self.out.push_str(text);
            self.out.push('\n');
        }

        fn open(&mut self, key: &str) {
            self.line(&format!("{key} = {{"));
            self.depth += 1;
        }

        fn close(&mut self) {
            self.depth -= 1;
            self.line("},");
        }

        fn num(&mut self, key: &str, value: impl std::fmt::Display) {
            self.line(&format!("{key} = {value},"));
        }

        fn hex(&mut self, key: &str, value: u8) {
            self.line(&format!("{key} = 0x{value:02x},"));
        }

        /// `{ [value] = "Name", ... }` plus the reverse `Name = value` lookup, which is what the
        /// dissection code branches on.
        fn enum_table(&mut self, key: &str, entries: &[(u8, &str)]) {
            self.open(key);
            for (value, name) in entries {
                self.line(&format!("[0x{value:02x}] = \"{name}\","));
            }
            self.close();
        }

        /// An ordered list of `{ bit, name }` pairs for a flag set.
        fn flag_table(&mut self, key: &str, entries: &[(u8, String)]) {
            self.open(key);
            for (bit, name) in entries {
                self.line(&format!("{{ bit = 0x{bit:02x}, name = \"{name}\" }},"));
            }
            self.close();
        }
    }

    /// Builds the generated block of `hopr.lua` from the Rust items — never from re-typed literals.
    fn render_wire_constants() -> String {
        // Sub-widths of the ticket that `hopr-types` keeps as literals inside `encode_tail_without_
        // signature`. They are asserted against `Ticket::SIZE` below, so a change to any of them
        // still fails this test rather than silently mis-aligning the dissector.
        const TICKET_AMOUNT: usize = 12;
        const TICKET_INDEX: usize = 6;
        const TICKET_CHANNEL_EPOCH: usize = 3;
        let ticket_signature = Ticket::SIZE
            - Address::SIZE
            - TICKET_AMOUNT
            - TICKET_INDEX
            - TICKET_CHANNEL_EPOCH
            - WinningProbability::SIZE
            - EthereumChallenge::SIZE;
        assert_eq!(
            ticket_signature, 64,
            "ticket layout changed: re-derive the field widths from hopr-types before regenerating"
        );

        let mut lua = Lua::default();
        lua.line(BEGIN_MARKER);
        lua.line("-- Generated from the Rust protocol definitions; do not edit by hand.");
        lua.line("-- Regenerate with:");
        lua.line("--   HOPR_UPDATE_DISSECTOR=1 cargo nextest run -p hopr-transport --features capture --lib dissector");
        lua.line("local WIRE = {");
        lua.depth += 1;

        // --- capture container ---
        lua.open("capture");
        lua.num("format_version", CAPTURE_FORMAT_VERSION);
        // Both spellings of the same link type: pcapng stores the LINKTYPE number, while
        // Wireshark's `wtap_encap` dissector table is keyed by its own WTAP_ENCAP constant, which
        // the dissector looks up by name.
        lua.num("link_type", u32::from(CAPTURE_LINK_TYPE));
        lua.line(&format!("wtap_encap = \"{CAPTURE_LINK_TYPE:?}\","));
        lua.enum_table(
            "frame_type_names",
            &PacketType::VARIANTS
                .iter()
                .map(|kind| (*kind as u8, capture_frame_name(*kind)))
                .collect::<Vec<_>>(),
        );
        lua.open("frame_type");
        for kind in PacketType::VARIANTS {
            lua.num(&format!("{kind:?}"), *kind as u8);
        }
        lua.close();
        lua.close();

        // --- field widths ---
        lua.open("size");
        lua.num("packet_tag", size_of::<PacketTag>());
        lua.num("public_key", OffchainPublicKey::SIZE);
        lua.num("pseudonym", HoprPseudonym::SIZE);
        lua.num("half_key", HalfKey::SIZE);
        lua.num("ack_challenge", ACK_CHALLENGE_SIZE);
        lua.num("acknowledgement", Acknowledgement::SIZE);
        lua.num("surb", HoprSurb::SIZE);
        lua.num("hopr_packet", HoprPacket::SIZE);
        lua.close();

        lua.open("ticket");
        lua.num("size", Ticket::SIZE);
        lua.num("counterparty", Address::SIZE);
        lua.num("amount", TICKET_AMOUNT);
        lua.num("index", TICKET_INDEX);
        lua.num("channel_epoch", TICKET_CHANNEL_EPOCH);
        lua.num("win_prob", WinningProbability::SIZE);
        lua.num("eth_challenge", EthereumChallenge::SIZE);
        lua.num("signature", ticket_signature);
        lua.close();

        // --- application layer ---
        lua.open("app");
        lua.num("tag_size", Tag::SIZE);
        lua.num("payload_size", ApplicationData::PAYLOAD_SIZE);
        lua.num("reserved_upper_bound", ReservedTag::UPPER_BOUND);
        lua.num("undefined_tag", ReservedTag::Undefined as u64);
        lua.enum_table(
            "reserved_tag_names",
            &<ReservedTag as strum::IntoEnumIterator>::iter()
                .map(|tag| (tag as u8, reserved_tag_name(tag)))
                .collect::<Vec<_>>(),
        );
        lua.open("reserved_tag");
        lua.num("probe", ReservedTag::Ping as u64);
        lua.num("start", ReservedTag::SessionStart as u64);
        lua.num("session", SESSION_APPLICATION_TAG.as_u64());
        lua.close();
        lua.flag_table(
            "packet_signals",
            &PacketSignal::LIST
                .iter()
                .map(|signal| (flag_bits(*signal), signal.to_string()))
                .collect::<Vec<_>>(),
        );
        lua.close();

        // --- probe protocol ---
        let mut probe_discriminants: Vec<(u8, &'static str)> = fixtures::probe_messages()
            .iter()
            .map(|(_, message)| (message.to_bytes()[1], probe_message_name(message)))
            .collect();
        probe_discriminants.sort_unstable();
        probe_discriminants.dedup();

        lua.open("probe");
        lua.num("version", ProbeMessage::VERSION);
        lua.num("header_size", 2);
        lua.num("nonce_size", NeighborProbe::NONCE_SIZE);
        lua.num("telemetry_id_size", PathTelemetry::ID_SIZE);
        lua.num("telemetry_path_size", PathTelemetry::PATH_SIZE);
        lua.num("telemetry_timestamp_size", size_of::<u128>());
        lua.enum_table("message_names", &probe_discriminants);
        lua.open("message");
        for (value, name) in &probe_discriminants {
            lua.num(name, *value);
        }
        lua.close();
        lua.enum_table(
            "neighbor_names",
            &[NeighborProbe::Ping([0; 32]), NeighborProbe::Pong([0; 32])]
                .iter()
                .map(|probe| (probe.to_bytes()[0], neighbor_probe_name(probe)))
                .collect::<Vec<_>>(),
        );
        lua.close();

        // --- start protocol ---
        let start_messages = fixtures::start_messages();
        let mut start_discriminants: Vec<(u8, &'static str)> = start_messages
            .iter()
            .map(|(_, message)| {
                (
                    message.clone().encode().expect("fixture start message must encode").1[1],
                    start_message_name(message),
                )
            })
            .collect();
        start_discriminants.sort_unstable();
        start_discriminants.dedup();
        assert_eq!(
            start_discriminants.len(),
            reprs_of(StartProtocolDiscriminants::from_repr).len(),
            "the Start fixture corpus no longer covers every Start message"
        );

        lua.open("start");
        lua.num("version", HoprStartProtocol::START_PROTOCOL_VERSION);
        lua.num("header_size", 2 * size_of::<u8>() + size_of::<u16>());
        lua.num("challenge_size", size_of::<StartChallenge>());
        lua.num("additional_data_size", size_of::<u64>());
        lua.enum_table("message_names", &start_discriminants);
        lua.open("message");
        for (value, name) in &start_discriminants {
            lua.num(name, *value);
        }
        lua.close();
        lua.enum_table(
            "error_reason_names",
            &reprs_of(StartErrorReason::from_repr)
                .into_iter()
                .map(|(repr, reason)| (repr, start_error_reason_name(reason)))
                .collect::<Vec<_>>(),
        );
        lua.enum_table(
            "error_identifier_names",
            &reprs_of(ErrorIdentifierDiscriminants::from_repr)
                .into_iter()
                .map(|(repr, identifier)| (repr, error_identifier_name(identifier)))
                .collect::<Vec<_>>(),
        );
        lua.hex(
            "error_identifier_challenge",
            ErrorIdentifierDiscriminants::Challenge as u8,
        );
        lua.flag_table(
            "capabilities",
            &Capability::LIST
                .iter()
                .map(|capability| (flag_bits(*capability), capability.to_string()))
                .collect::<Vec<_>>(),
        );
        lua.flag_table(
            "keep_alive_flags",
            &KeepAliveFlag::LIST
                .iter()
                .map(|flag| (flag_bits(*flag), format!("{flag:?}")))
                .collect::<Vec<_>>(),
        );
        lua.close();

        // --- PIX sizes carried by the two SSA messages ---
        //
        // Both curve suites are emitted, because an `SsaRequest` names the suite it was built under
        // and a capture may come from a node built for the other one. Only the local suite's pair
        // can be read off the types; the other is stated by `pix_suite_widths` and checked whenever
        // a build of that flavour runs this test.
        let suites = reprs_of(|bits| hopr_protocol_pix::PixSuite::try_from_bits(bits).ok());

        lua.open("pix");
        lua.num("ssa_index_size", size_of::<RawSsaIndex>());
        lua.num("polynomial_index_size", size_of::<PolynomialIndex>());
        lua.num("coefficient_index_size", size_of::<CoefficientIndex>());
        lua.num("missing_run_entry_size", HoprStartProtocol::MISSING_RUN_ENTRY_SIZE);
        lua.num("max_polys_per_ssa", MAX_POLYS_PER_SSA);
        lua.num("max_ssas_per_request", HoprStartProtocol::MAX_SSAS_PER_REQUEST);
        lua.num("build_suite", LOCAL_PIX_SUITE as u8);
        lua.enum_table(
            "suite_names",
            &suites
                .iter()
                .map(|(repr, suite)| (*repr, pix_suite_name(*suite)))
                .collect::<Vec<_>>(),
        );
        lua.open("suite");
        for (repr, suite) in &suites {
            let (group_repr, commitment_proof) = pix_suite_widths(*suite);
            if *suite == LOCAL_PIX_SUITE {
                assert_eq!(
                    group_repr,
                    HoprStartProtocol::PIX_COEFF_COMMITMENT_REPR_SIZE,
                    "the {suite} commitment width changed"
                );
                assert_eq!(
                    commitment_proof,
                    HoprStartProtocol::PIX_COMMITMENT_PROOF_SIZE,
                    "the {suite} commitment proof width changed"
                );
            }
            lua.open(&format!("[0x{repr:02x}]"));
            lua.num("group_repr", group_repr);
            lua.num("commitment_proof", commitment_proof);
            lua.close();
        }
        lua.close();
        lua.close();

        // --- session protocol ---
        let session_messages = fixtures::session_messages();
        let session_discriminants: Vec<(u8, &'static str)> = session_messages
            .iter()
            .map(|(_, message)| (message.clone().into_encoded()[1], session_message_name(message)))
            .collect();
        assert_eq!(
            session_discriminants.len(),
            reprs_of(SessionMessageDiscriminants::from_repr).len(),
            "the Session fixture corpus no longer covers every Session message"
        );

        lua.open("session");
        lua.num("version", SessionMessage::<SESSION_MTU>::VERSION);
        lua.num("header_size", SessionMessage::<SESSION_MTU>::HEADER_SIZE);
        lua.num("segment_header_size", Segment::HEADER_SIZE);
        lua.num("frame_id_size", size_of::<FrameId>());
        lua.num("seq_num_size", size_of::<SeqNum>());
        lua.hex("seq_terminating_mask", 0x80);
        lua.hex("seq_len_mask", SeqIndicator::MAX);
        lua.num("request_entry_size", SegmentRequest::<SESSION_MTU>::ENTRY_SIZE);
        lua.num(
            "max_missing_segments_per_frame",
            SegmentRequest::<SESSION_MTU>::MAX_MISSING_SEGMENTS_PER_FRAME,
        );
        lua.num("ack_entry_size", size_of::<FrameId>());
        lua.enum_table("message_names", &session_discriminants);
        lua.open("message");
        for (value, name) in &session_discriminants {
            lua.num(name, *value);
        }
        lua.close();
        lua.close();

        lua.depth -= 1;
        lua.line("}");
        lua.line(END_MARKER);

        lua.out
    }

    #[test]
    fn lua_wire_constants_are_in_sync() -> anyhow::Result<()> {
        let path = dissector_path();
        let current = std::fs::read_to_string(&path)
            .map_err(|error| anyhow::anyhow!("cannot read {}: {error}", path.display()))?;

        let (head, rest) = current
            .split_once(BEGIN_MARKER)
            .ok_or_else(|| anyhow::anyhow!("{} is missing {BEGIN_MARKER}", path.display()))?;
        let (body, tail) = rest
            .split_once(END_MARKER)
            .ok_or_else(|| anyhow::anyhow!("{} is missing {END_MARKER}", path.display()))?;

        let expected = render_wire_constants();
        let actual = format!("{BEGIN_MARKER}{body}{END_MARKER}");

        if actual.trim_end() == expected.trim_end() {
            return Ok(());
        }

        let updated = format!("{head}{}{tail}", expected.trim_end());
        if std::env::var_os(UPDATE_ENV).is_some() {
            std::fs::write(&path, updated)?;
            eprintln!("{} regenerated; review the diff", path.display());
            return Ok(());
        }

        anyhow::bail!(
            "{} is out of date with the Rust protocol definitions.\nRe-run with {UPDATE_ENV}=1 to regenerate it, then \
             check that the dissection code below the generated block still reads the right fields.\n\n--- expected \
             ---\n{expected}\n--- found ---\n{actual}\n",
            path.display()
        );
    }

    // --- layer 3: the byte layout ----------------------------------------------------------------

    fn to_hex(bytes: &[u8]) -> String {
        bytes
            .iter()
            .fold(String::with_capacity(bytes.len() * 2), |mut out, byte| {
                let _ = write!(out, "{byte:02x}");
                out
            })
    }

    #[test]
    fn capture_frame_layout_is_stable() {
        let mut rendered = String::new();
        for fixture in fixtures::corpus() {
            writeln!(
                rendered,
                "{:<28} {:<8} {}",
                fixture.name,
                fixture.packet.direction.to_string(),
                to_hex(&fixture.packet.data)
            )
            .expect("writing to a String cannot fail");
        }

        // A diff here means the bytes on the wire moved. Update `hopr.lua` to match *before*
        // accepting the new snapshot, and bump `CAPTURE_FORMAT_VERSION` if the layout itself
        // changed rather than just a fixture value.
        insta::assert_snapshot!(rendered);
    }

    #[test]
    fn every_frame_starts_with_the_format_version() {
        for fixture in fixtures::corpus() {
            assert_eq!(
                fixture.packet.data.first().copied(),
                Some(CAPTURE_FORMAT_VERSION),
                "{} does not start with the capture format version",
                fixture.name
            );
        }
    }

    // --- layer 4: the dissector actually runs ----------------------------------------------------

    /// Writes the corpus to a pcapng the way a live node would.
    fn write_fixture_capture() -> anyhow::Result<std::path::PathBuf> {
        let path = std::env::temp_dir().join(format!("hopr-dissector-fixture-{}.pcapng", std::process::id()));
        let mut writer = PcapPacketWriter::new(std::fs::File::create(&path)?)?;
        for (index, fixture) in fixtures::corpus().into_iter().enumerate() {
            let mut packet = fixture.packet;
            // Fixed, increasing timestamps keep the file reproducible.
            packet.timestamp = std::time::Duration::from_secs(index as u64 + 1);
            writer.write_packet(packet)?;
        }
        Ok(path)
    }

    fn tshark(capture: &std::path::Path, filter: &str) -> anyhow::Result<Vec<String>> {
        let output = std::process::Command::new("tshark")
            .arg("-r")
            .arg(capture)
            .arg("-X")
            .arg(format!("lua_script:{}", dissector_path().display()))
            .args(["-Y", filter])
            .args(["-T", "fields", "-e", "frame.number"])
            .output()?;

        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::ensure!(output.status.success(), "tshark failed for filter `{filter}`: {stderr}");
        // A Lua script that fails to load leaves tshark running with a broken dissector, so the
        // exit status alone is not enough.
        anyhow::ensure!(
            !stderr.contains("Lua:") && !stderr.contains("lua:"),
            "the dissector reported a Lua error: {stderr}"
        );

        Ok(String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(|line| line.trim().to_string())
            .filter(|line| !line.is_empty())
            .collect())
    }

    #[test]
    fn lua_dissects_the_fixture_capture() -> anyhow::Result<()> {
        if which_tshark().is_none() {
            // A silent skip is fine on a workstation without Wireshark, and unacceptable in CI —
            // this is the only test that proves the dissector loads at all. The nix check phase
            // therefore sets `REQUIRE_TSHARK_ENV`, turning a missing `tshark` into a failure.
            anyhow::ensure!(
                std::env::var_os(REQUIRE_TSHARK_ENV).is_none(),
                "`tshark` is required here ({REQUIRE_TSHARK_ENV} is set) but is not on PATH"
            );
            eprintln!("skipping: `tshark` is not on PATH (it ships in the nix devShell)");
            return Ok(());
        }

        let capture = write_fixture_capture()?;
        let corpus = fixtures::corpus();

        // One disjunct per frame, each pinning the frame number to the discriminants the dissector
        // is expected to produce for it. Display filters compare numerically, so this is
        // independent of how the fields are formatted for display.
        let filter = corpus
            .iter()
            .enumerate()
            .map(|(index, fixture)| {
                let mut terms = vec![
                    format!("frame.number == {}", index + 1),
                    format!("hopr.type == {}", fixture.expect.hopr_type),
                ];
                if let Some(tag) = fixture.expect.appdata_tag {
                    terms.push(format!("hopr.appdata.tag == {tag}"));
                }
                if let Some(kind) = fixture.expect.start_type {
                    terms.push(format!("hopr_start.type == {kind}"));
                }
                if let Some(kind) = fixture.expect.session_type {
                    terms.push(format!("hopr_session.type == {kind}"));
                }
                if let Some(kind) = fixture.expect.probe_type {
                    terms.push(format!("hopr_probe.type == {kind}"));
                }
                format!("({})", terms.join(" && "))
            })
            .collect::<Vec<_>>()
            .join(" || ");

        let matched = tshark(&capture, &filter)?;
        let expected = (1..=corpus.len()).map(|n| n.to_string()).collect::<Vec<_>>();
        assert_eq!(
            matched,
            expected,
            "frames the dissector did not decode as expected: {:?}",
            expected
                .iter()
                .enumerate()
                .filter(|(_, frame)| !matched.contains(frame))
                .map(|(index, _)| corpus[index].name)
                .collect::<Vec<_>>()
        );

        // Nothing malformed, nothing truncated, no trailing bytes: the dissector must walk every
        // frame to its end without complaining. Below WARN sits Wireshark's own rendering of the
        // per-packet direction comment, and the deliberate random-acknowledgement note asserted
        // just below.
        const EXPERT_SEVERITY_WARN: u32 = 0x0060_0000;
        let complaints = tshark(&capture, &format!("_ws.expert.severity >= {EXPERT_SEVERITY_WARN}"))?;
        assert!(
            complaints.is_empty(),
            "the dissector raised warnings on frames {complaints:?}; run tshark by hand to see them"
        );

        // The absence of complaints only means something if the dissector can raise one at all, so
        // check the one the corpus deliberately triggers.
        let random_acks = tshark(&capture, "hopr.ack.random")?;
        let expected_random = corpus
            .iter()
            .enumerate()
            .filter(|(_, fixture)| fixture.name == "ack.outgoing.random")
            .map(|(index, _)| (index + 1).to_string())
            .collect::<Vec<_>>();
        assert_eq!(
            random_acks, expected_random,
            "the random-acknowledgement note did not fire where the corpus expects it"
        );

        // ... and that the completeness guard fires: a frame with one byte too many is exactly what
        // a capture written by a newer node would look like to this dissector.
        let extended = extend_first_frame(&capture)?;
        let trailing = tshark(&extended, "hopr.trailing_bytes")?;
        assert_eq!(
            trailing,
            vec!["1".to_string()],
            "the dissector did not notice a trailing byte it could not account for"
        );

        // Kept on failure, and on request: opening the fixture in Wireshark is how a mismatch here
        // is actually diagnosed.
        if std::env::var_os(KEEP_CAPTURE_ENV).is_none() {
            std::fs::remove_file(&capture)?;
            std::fs::remove_file(&extended)?;
        } else {
            eprintln!("fixture capture kept at {}", capture.display());
        }
        Ok(())
    }

    /// Writes a copy of the corpus whose first frame carries one unaccounted-for extra byte.
    fn extend_first_frame(source: &std::path::Path) -> anyhow::Result<std::path::PathBuf> {
        let path = source.with_extension("extended.pcapng");
        let mut writer = PcapPacketWriter::new(std::fs::File::create(&path)?)?;
        for (index, fixture) in fixtures::corpus().into_iter().enumerate() {
            let mut packet = fixture.packet;
            packet.timestamp = std::time::Duration::from_secs(index as u64 + 1);
            if index == 0 {
                let mut data = packet.data.into_vec();
                data.push(0xff);
                packet.data = data.into_boxed_slice();
            }
            writer.write_packet(packet)?;
        }
        Ok(path)
    }

    fn which_tshark() -> Option<std::path::PathBuf> {
        std::env::var_os("PATH").and_then(|paths| {
            std::env::split_paths(&paths)
                .map(|dir| dir.join("tshark"))
                .find(|candidate| candidate.is_file())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{fixtures, *};

    #[tokio::test]
    async fn pcap_writer_should_persist_every_captured_packet() -> anyhow::Result<()> {
        let path = std::env::temp_dir().join(format!("hopr-capture-{}.pcapng", std::process::id()));
        let (pcap, ah) =
            packet_capture_channel(Box::new(std::fs::File::create(&path).and_then(PcapPacketWriter::new)?));

        let corpus = fixtures::corpus();
        for fixture in &corpus {
            pcap.send(fixture.packet.clone()).await?;
        }

        // The writer drains on a blocking task; give it a moment before checking the file.
        for _ in 0..50 {
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
            if std::fs::metadata(&path)?.len() > 0 {
                break;
            }
        }
        ah.abort();

        let written = std::fs::metadata(&path)?.len() as usize;
        let payload: usize = corpus.iter().map(|fixture| fixture.packet.data.len()).sum();
        anyhow::ensure!(
            written > payload,
            "the pcapng ({written} B) is smaller than the packets it should contain ({payload} B)"
        );

        std::fs::remove_file(&path)?;
        Ok(())
    }
}
