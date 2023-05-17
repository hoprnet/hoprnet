use crate::errors::PacketError::{InvalidPacketState, PacketDecodingError};
use core_crypto::derivation::{derive_ack_key_share, derive_packet_tag};
use core_crypto::primitives::{DigestLike, SimpleMac};
use core_crypto::prp::{PRPParameters, PRP};
use core_crypto::routing::{forward_header, header_length, ForwardedHeader, RoutingInfo};
use core_crypto::shared_keys::Secp256k1SharedKeys;
use core_crypto::types::{Challenge, CurvePoint, HalfKey, HalfKeyChallenge, PublicKey};
use core_types::acknowledgement::{Acknowledgement, AcknowledgementChallenge};
use core_types::channels::Ticket;
use libp2p_identity::PeerId;
use utils_types::errors::GeneralError::ParseError;
use utils_types::traits::{BinarySerializable, PeerIdLike};

use crate::errors::Result;
use crate::packet::ForwardedMetaPacket::{FinalPacket, RelayedPacket};
use crate::packet::PacketState::{Final, Forwarded, Outgoing};
use crate::por::{pre_verify, ProofOfRelayString, ProofOfRelayValues, POR_SECRET_LENGTH};

/// Number of intermediate hops: 3 relayers and 1 destination
pub const INTERMEDIATE_HOPS: usize = 3;

/// Maximum size of the packet payload
pub const PAYLOAD_SIZE: usize = 500;

/// Length of the packet including header and the payload
pub const PACKET_LENGTH: usize = packet_length(INTERMEDIATE_HOPS + 1, POR_SECRET_LENGTH, 0);

/// Tag used to separate padding from data
const PADDING_TAG: &[u8] = b"HOPR";

/// Determines the total length (header + payload) of the packet given the header information.
pub const fn packet_length(
    max_hops: usize,
    additional_data_relayer_len: usize,
    additional_data_last_hop_len: usize,
) -> usize {
    PublicKey::SIZE_COMPRESSED
        + header_length(max_hops, additional_data_relayer_len, additional_data_last_hop_len)
        + SimpleMac::SIZE
        + PAYLOAD_SIZE
}

fn add_padding(msg: &[u8]) -> Box<[u8]> {
    assert!(
        msg.len() <= PAYLOAD_SIZE - PADDING_TAG.len(),
        "message too long for padding"
    );
    let mut ret = vec![0u8; PAYLOAD_SIZE];
    ret[PAYLOAD_SIZE - msg.len()..PAYLOAD_SIZE].copy_from_slice(msg);
    ret[PAYLOAD_SIZE - msg.len() - PADDING_TAG.len()..PAYLOAD_SIZE - msg.len()].copy_from_slice(PADDING_TAG);
    ret.into_boxed_slice()
}

fn remove_padding(msg: &[u8]) -> Option<&[u8]> {
    assert_eq!(PAYLOAD_SIZE, msg.len(), "padded message must be PAYLOAD_SIZE long");
    let pos = msg
        .windows(PADDING_TAG.len())
        .position(|window| window == PADDING_TAG)?;
    Some(&msg.split_at(pos).1[PADDING_TAG.len()..])
}

struct MetaPacket {
    packet: Box<[u8]>,
    header_len: usize,
}

#[allow(dead_code)]
enum ForwardedMetaPacket {
    RelayedPacket {
        packet: MetaPacket,
        next_node: PublicKey,
        additional_info: Box<[u8]>,
        derived_secret: Box<[u8]>,
        packet_tag: Box<[u8]>,
    },
    FinalPacket {
        plain_text: Box<[u8]>,
        additional_data: Box<[u8]>,
        derived_secret: Box<[u8]>,
        packet_tag: Box<[u8]>,
    },
}

impl MetaPacket {
    pub fn new(
        shared_keys: Secp256k1SharedKeys,
        msg: &[u8],
        path: &[&PeerId],
        max_hops: usize,
        additional_relayer_data_len: usize,
        additional_data_relayer: &[&[u8]],
        additional_data_last_hop: Option<&[u8]>,
    ) -> MetaPacket {
        assert!(msg.len() <= PAYLOAD_SIZE, "message too long to fit into a packet");

        let mut padded = add_padding(msg);
        let secrets = shared_keys.secrets.iter().map(|s| s.as_slice()).collect::<Vec<_>>();
        let routing_info = RoutingInfo::new(
            max_hops,
            &path
                .iter()
                .map(|peer| PublicKey::from_peerid(peer).unwrap_or_else(|e| panic!("invalid peer id given: {e}")))
                .collect::<Vec<_>>(),
            &secrets,
            additional_relayer_data_len,
            additional_data_relayer,
            additional_data_last_hop,
        );

        // Encrypt packet payload using the derived shared secrets
        for secret in secrets.iter().rev() {
            let prp = PRP::from_parameters(PRPParameters::new(secret));
            prp.forward_inplace(&mut padded)
                .unwrap_or_else(|e| panic!("onion encryption error {e}"))
        }

        MetaPacket::new_from_parts(
            &shared_keys.alpha,
            &routing_info.routing_information,
            &routing_info.mac,
            &padded,
        )
    }

    fn new_from_parts(alpha: &CurvePoint, routing_info: &[u8], mac: &[u8], payload: &[u8]) -> Self {
        assert!(routing_info.len() > 0);
        assert_eq!(SimpleMac::SIZE, mac.len());
        assert_eq!(PAYLOAD_SIZE, payload.len());

        let mut packet = Vec::with_capacity(Self::size(routing_info.len()));
        packet.extend_from_slice(&alpha.serialize_compressed());
        packet.extend_from_slice(routing_info);
        packet.extend_from_slice(mac);
        packet.extend_from_slice(payload);

        Self {
            packet: packet.into_boxed_slice(),
            header_len: routing_info.len(),
        }
    }

    pub fn alpha(&self) -> &[u8] {
        &self.packet[..CurvePoint::SIZE_COMPRESSED]
    }

    pub fn routing_info(&self) -> &[u8] {
        &self.packet[CurvePoint::SIZE_COMPRESSED..CurvePoint::SIZE_COMPRESSED + self.header_len]
    }

    pub fn mac(&self) -> &[u8] {
        &self.packet[CurvePoint::SIZE_COMPRESSED + self.header_len
            ..CurvePoint::SIZE_COMPRESSED + self.header_len + SimpleMac::SIZE]
    }

    pub fn payload(&self) -> &[u8] {
        &self.packet[CurvePoint::SIZE_COMPRESSED + self.header_len + SimpleMac::SIZE
            ..CurvePoint::SIZE_COMPRESSED + self.header_len + SimpleMac::SIZE + PAYLOAD_SIZE]
    }

    pub const fn size(header_len: usize) -> usize {
        CurvePoint::SIZE_COMPRESSED + header_len + SimpleMac::SIZE + PAYLOAD_SIZE
    }

    pub fn from_bytes(packet: &[u8], header_len: usize) -> utils_types::errors::Result<MetaPacket> {
        if packet.len() == Self::size(header_len) {
            Ok(Self {
                packet: packet.into(),
                header_len,
            })
        } else {
            Err(ParseError)
        }
    }

    pub fn to_bytes(&self) -> &[u8] {
        &self.packet
    }

    pub fn forward(
        &self,
        private_key: &[u8],
        max_hops: usize,
        additional_data_relayer_len: usize,
        additional_data_last_hop_len: usize,
    ) -> Result<ForwardedMetaPacket> {
        let (alpha, secret) = Secp256k1SharedKeys::forward_transform(
            CurvePoint::from_bytes(self.alpha())?,
            private_key.try_into().expect("invalid packet private key")
        )?;

        let mut routing_info_mut: Vec<u8> = self.routing_info().into();
        let fwd_header = forward_header(
            &secret,
            &mut routing_info_mut,
            self.mac(),
            max_hops,
            additional_data_relayer_len,
            additional_data_last_hop_len,
        )?;

        let prp = PRP::from_parameters(PRPParameters::new(&secret));
        let decrypted = prp.inverse(self.payload())?;

        match fwd_header {
            ForwardedHeader::RelayNode {
                header,
                mac,
                next_node,
                additional_info,
            } => Ok(RelayedPacket {
                packet: MetaPacket::new_from_parts(&alpha, &header, &mac, &decrypted),
                packet_tag: derive_packet_tag(&secret)?,
                derived_secret: secret.into(),
                next_node,
                additional_info,
            }),
            ForwardedHeader::FinalNode { additional_data } => Ok(FinalPacket {
                packet_tag: derive_packet_tag(&secret)?,
                derived_secret: secret.into(),
                plain_text: remove_padding(&decrypted)
                    .ok_or(PacketDecodingError("couldn't remove padding".into()))?
                    .into(),
                additional_data,
            }),
        }
    }
}

/// Indicates if the packet is supposed to be forwarded to the next hop or if it is intended for us.
#[derive(Debug)]
pub enum PacketState {
    /// Packet is intended for us
    Final {
        packet_tag: Box<[u8]>,
        ack_key: HalfKey,
        previous_hop: PublicKey,
        plain_text: Box<[u8]>,
        old_challenge: Option<AcknowledgementChallenge>,
    },
    /// Packet must be forwarded
    Forwarded {
        ack_challenge: HalfKeyChallenge,
        packet_tag: Box<[u8]>,
        ack_key: HalfKey,
        previous_hop: PublicKey,
        own_key: HalfKey,
        own_share: HalfKeyChallenge,
        next_hop: PublicKey,
        next_challenge: Challenge,
        old_challenge: Option<AcknowledgementChallenge>,
    },
    /// Packet that is being sent out by us
    Outgoing {
        next_hop: PublicKey,
        ack_challenge: HalfKeyChallenge,
    },
}

/// Represents a HOPR packet
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen(getter_with_clone))]
pub struct Packet {
    packet: MetaPacket,
    pub challenge: AcknowledgementChallenge,
    pub ticket: Ticket,
    state: PacketState,
}

impl Packet {
    /// Size of the packet including header, payload, ticket and ack challenge.
    pub const SIZE: usize = PACKET_LENGTH + AcknowledgementChallenge::SIZE + Ticket::SIZE;

    /// Constructs new outgoing packet with the given path.
    /// # Arguments
    /// * `msg` packet payload
    /// * `path` complete path for the packet to take
    /// * `private_key` private key of the local node
    /// * `first_ticket` ticket for the first hop on the path
    pub fn new(msg: &[u8], path: &[&PeerId], private_key: &[u8], first_ticket: Ticket) -> Result<Self> {
        assert!(!path.is_empty(), "path must not be empty");

        let shared_keys = Secp256k1SharedKeys::new(path)?;

        let por_values = ProofOfRelayValues::new(shared_keys.secrets[0], shared_keys.secrets.get(1).cloned());

        let por_strings = (1..path.len())
            .map(|i| ProofOfRelayString::new(shared_keys.secrets[i], shared_keys.secrets.get(i + 1).cloned()).to_bytes())
            .collect::<Vec<_>>();

        let mut ticket = first_ticket;
        ticket.challenge = por_values.ticket_challenge.to_ethereum_challenge();
        ticket.sign(private_key);

        Ok(Self {
            challenge: AcknowledgementChallenge::new(&por_values.ack_challenge, private_key),
            packet: MetaPacket::new(
                shared_keys,
                msg,
                path,
                INTERMEDIATE_HOPS + 1,
                POR_SECRET_LENGTH,
                &por_strings.iter().map(Box::as_ref).collect::<Vec<_>>(),
                None,
            ),
            ticket,
            state: Outgoing {
                next_hop: PublicKey::from_peerid(&path[0])?,
                ack_challenge: por_values.ack_challenge,
            },
        })
    }

    /// Deserializes the packet and performs the forward-transformation, so the
    /// packet can be further delivered (relayed to the next hop or read).
    pub fn from_bytes(data: &[u8], private_key: &[u8], sender: &PeerId) -> Result<Self> {
        if data.len() == Self::SIZE {
            let (pre_packet, r0) = data.split_at(PACKET_LENGTH);
            let (pre_challenge, pre_ticket) = r0.split_at(AcknowledgementChallenge::SIZE);
            let previous_hop = PublicKey::from_peerid(sender)?;

            let header_len = header_length(INTERMEDIATE_HOPS + 1, POR_SECRET_LENGTH, 0);
            let mp = MetaPacket::from_bytes(pre_packet, header_len)?;

            match mp.forward(private_key, INTERMEDIATE_HOPS + 1, POR_SECRET_LENGTH, 0)? {
                RelayedPacket {
                    packet,
                    derived_secret,
                    additional_info,
                    packet_tag,
                    next_node,
                    ..
                } => {
                    let ack_key = derive_ack_key_share(&derived_secret);
                    let mut challenge = AcknowledgementChallenge::from_bytes(pre_challenge)?;
                    challenge
                        .validate(ack_key.to_challenge(), &previous_hop)
                        .then(|| ())
                        .ok_or(PacketDecodingError(
                            "couldn't validate acknowledgement challenge on the relayed packet".into(),
                        ))?;

                    let ticket = Ticket::from_bytes(pre_ticket)?;
                    let verification_output = pre_verify(&derived_secret, &additional_info, &ticket.challenge)?;
                    Ok(Self {
                        packet,
                        challenge,
                        ticket,
                        state: Forwarded {
                            packet_tag,
                            ack_key,
                            previous_hop,
                            own_key: verification_output.own_key,
                            own_share: verification_output.own_share,
                            next_hop: next_node,
                            next_challenge: verification_output.next_ticket_challenge,
                            ack_challenge: verification_output.ack_challenge,
                            old_challenge: None,
                        },
                    })
                }
                FinalPacket {
                    packet_tag,
                    plain_text,
                    derived_secret,
                    additional_data: _,
                } => {
                    let ack_key = derive_ack_key_share(&derived_secret);
                    let mut challenge = AcknowledgementChallenge::from_bytes(pre_challenge)?;
                    challenge
                        .validate(ack_key.to_challenge(), &previous_hop)
                        .then(|| ())
                        .ok_or(PacketDecodingError(
                            "couldn't validate acknowledgement challenge on the final packet".into(),
                        ))?;

                    let ticket = Ticket::from_bytes(pre_ticket)?;
                    Ok(Self {
                        packet: mp,
                        challenge,
                        ticket,
                        state: Final {
                            packet_tag,
                            ack_key,
                            previous_hop,
                            plain_text,
                            old_challenge: None,
                        },
                    })
                }
            }
        } else {
            Err(PacketDecodingError("packet has invalid size".into()))
        }
    }

    /// State of this packet
    pub fn state(&self) -> &PacketState {
        &self.state
    }

    /// Forwards the packet to the next hop.
    /// Requires private key of the local node and prepared ticket for the next recipient.
    /// Panics if the packet is not meant to be forwarded.
    pub fn forward(&mut self, private_key: &[u8], next_ticket: Ticket) -> Result<()> {
        match &mut self.state {
            Forwarded {
                next_challenge,
                old_challenge,
                ack_challenge,
                ..
            } => {
                let mut ticket = next_ticket;
                ticket.challenge = next_challenge.to_ethereum_challenge();
                ticket.sign(private_key);
                self.ticket = ticket;

                let _ = old_challenge.insert(self.challenge.clone());
                self.challenge = AcknowledgementChallenge::new(ack_challenge, private_key);
                Ok(())
            }
            _ => Err(InvalidPacketState),
        }
    }
}

#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
impl Packet {
    pub fn to_bytes(&self) -> Box<[u8]> {
        let mut ret = Vec::with_capacity(Self::SIZE);
        ret.extend_from_slice(self.packet.to_bytes());
        ret.extend_from_slice(&self.challenge.to_bytes());
        ret.extend_from_slice(&self.ticket.to_bytes());
        ret.into_boxed_slice()
    }

    /// Creates an acknowledgement for this packet.
    /// Returns None if this packet is sent by us.
    pub fn create_acknowledgement(&self, private_key: &[u8]) -> Option<Acknowledgement> {
        match &self.state {
            Final {
                ack_key, old_challenge, ..
            }
            | Forwarded {
                ack_key, old_challenge, ..
            } => Some(Acknowledgement::new(
                old_challenge.clone().unwrap_or(self.challenge.clone()),
                ack_key.clone(),
                private_key,
            )),
            Outgoing { .. } => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::packet::{
        add_padding, remove_padding, ForwardedMetaPacket, MetaPacket, Packet, PacketState, INTERMEDIATE_HOPS,
        PADDING_TAG,
    };
    use crate::por::{ProofOfRelayString, POR_SECRET_LENGTH};
    use core_crypto::shared_keys::SharedKeys;
    use core_crypto::types::PublicKey;
    use core_types::channels::Ticket;
    use libp2p_identity::{Keypair, PeerId};
    use parameterized::parameterized;
    use utils_types::primitives::{Balance, BalanceType, U256};
    use utils_types::traits::{BinarySerializable, PeerIdLike};

    #[test]
    fn test_padding() {
        let data = b"test";
        let padded = add_padding(data);

        let mut expected = vec![0u8; 492];
        expected.extend_from_slice(PADDING_TAG);
        expected.extend_from_slice(data);
        assert_eq!(&expected, padded.as_ref());

        let unpadded = remove_padding(&padded);
        assert!(unpadded.is_some());
        assert_eq!(data, &unpadded.unwrap());
    }

    fn generate_keypairs(amount: usize) -> (Vec<[u8; 32]>, Vec<PeerId>) {
        (0..amount)
            .map(|_| Keypair::generate_secp256k1())
            .map(|kp| {
                (
                    kp.clone().try_into_secp256k1().unwrap().secret().to_bytes(),
                    kp.public().to_peer_id(),
                )
            })
            .unzip()
    }

    #[parameterized(amount = { 4, 3, 2 })]
    fn test_meta_packet(amount: usize) {
        let (secrets, path) = generate_keypairs(amount);
        let shared_keys = SharedKeys::new(&path.iter().collect::<Vec<_>>()).unwrap();
        let por_strings = (1..path.len())
            .map(|i| ProofOfRelayString::new(shared_keys.secrets[i], shared_keys.secrets.get(i + 1).cloned()).to_bytes())
            .collect::<Vec<_>>();

        let msg = b"some random test message";

        let mut mp = MetaPacket::new(
            shared_keys,
            msg,
            &path.iter().collect::<Vec<_>>(),
            INTERMEDIATE_HOPS + 1,
            POR_SECRET_LENGTH,
            &por_strings.iter().map(|pors| pors.as_ref()).collect::<Vec<_>>(),
            None,
        );

        for (i, secret) in secrets.iter().enumerate() {
            let fwd = mp
                .forward(secret, INTERMEDIATE_HOPS + 1, POR_SECRET_LENGTH, 0)
                .expect(&format!("failed to unwrap at {i}"));
            match fwd {
                ForwardedMetaPacket::RelayedPacket { packet, .. } => {
                    assert!(i < path.len() - 1);
                    mp = packet;
                }
                ForwardedMetaPacket::FinalPacket {
                    plain_text,
                    additional_data,
                    ..
                } => {
                    assert_eq!(path.len() - 1, i);
                    assert_eq!(msg, plain_text.as_ref());
                    assert!(additional_data.is_empty());
                }
            }
        }
    }

    fn mock_ticket(next_peer: &PeerId, path_len: usize, private_key: &[u8]) -> Ticket {
        assert!(path_len > 0);
        const PRICE_PER_PACKET: u128 = 10000000000000000u128;
        const INVERSE_TICKET_WIN_PROB: u128 = 1;

        if path_len > 1 {
            Ticket::new(
                PublicKey::from_peerid(next_peer).unwrap().to_address(),
                None,
                U256::zero(),
                U256::zero(),
                Balance::new(
                    (PRICE_PER_PACKET * INVERSE_TICKET_WIN_PROB * (path_len - 1) as u128).into(),
                    BalanceType::HOPR,
                ),
                U256::one(),
                U256::zero(),
                private_key,
            )
        } else {
            Ticket::new_zero_hop(PublicKey::from_peerid(next_peer).unwrap(), None, private_key)
        }
    }

    #[parameterized(amount = { 4, 3, 2 })]
    fn test_packet_create_and_transform(amount: usize) {
        let (mut node_private_keys, mut path) = generate_keypairs(amount);

        let private_key = node_private_keys.drain(..1).last().unwrap();
        let public_key = path.drain(..1).last().unwrap();

        // Create ticket for the first peer on the path
        let ticket = mock_ticket(&path[0], path.len(), &private_key);

        let test_message = b"some testing message";
        let mut packet = Packet::new(test_message, &path.iter().collect::<Vec<_>>(), &private_key, ticket)
            .expect("failed to construct packet");

        match &packet.state() {
            PacketState::Outgoing { .. } => {}
            _ => panic!("invalid packet initial state"),
        }

        for (i, (node_private, node_id)) in node_private_keys.iter().zip(path.iter()).enumerate() {
            let sender = (i == 0)
                .then_some(&public_key)
                .unwrap_or_else(|| path.get(i - 1).unwrap());

            packet = Packet::from_bytes(&packet.to_bytes(), node_private, &sender)
                .unwrap_or_else(|e| panic!("failed to deserialize packet at hop {i}: {e}"));

            match packet.state() {
                PacketState::Final { plain_text, .. } => {
                    assert_eq!(path.len() - 1, i);
                    assert_eq!(&test_message, &plain_text.as_ref());
                }
                PacketState::Forwarded { .. } => {
                    let ticket = mock_ticket(&node_id, path.len() - i - 1, node_private);
                    packet.forward(node_private, ticket).unwrap();
                }
                PacketState::Outgoing { .. } => panic!("invalid packet state"),
            }
        }
    }
}

#[cfg(feature = "wasm")]
pub mod wasm {
    use crate::packet::wasm::WasmPacketState::{Final, Forwarded, Outgoing};
    use crate::packet::{Packet, PacketState};
    use core_crypto::types::{HalfKey, HalfKeyChallenge, PublicKey};
    use core_types::channels::Ticket;
    use js_sys::JsString;
    use libp2p_identity::{ParseError, PeerId};
    use std::str::FromStr;
    use utils_misc::ok_or_jserr;
    use utils_misc::utils::wasm::JsResult;
    use wasm_bindgen::prelude::wasm_bindgen;

    #[wasm_bindgen]
    pub enum WasmPacketState {
        Final,
        Forwarded,
        Outgoing,
    }

    #[wasm_bindgen]
    impl Packet {
        #[wasm_bindgen(constructor)]
        pub fn _new(msg: &[u8], path: Vec<JsString>, private_key: &[u8], first_ticket: Ticket) -> JsResult<Packet> {
            let peers = ok_or_jserr!(path
                .into_iter()
                .map(|s| PeerId::from_str(&s.as_string().unwrap()))
                .collect::<Result<Vec<_>, ParseError>>())?;
            ok_or_jserr!(Self::new(
                msg,
                &peers.iter().collect::<Vec<_>>(),
                private_key,
                first_ticket
            ))
        }

        #[wasm_bindgen(js_name = "serialize")]
        pub fn _serialize(&self) -> Box<[u8]> {
            self.to_bytes()
        }

        #[wasm_bindgen(js_name = "deserialize")]
        pub fn _deserialize(data: &[u8], private_key: &[u8], sender: &str) -> JsResult<Packet> {
            let sender = ok_or_jserr!(PeerId::from_str(sender))?;
            ok_or_jserr!(Self::from_bytes(data, private_key, &sender))
        }

        #[wasm_bindgen(js_name = "forward")]
        pub fn _forward(&mut self, private_key: &[u8], next_ticket: Ticket) -> JsResult<()> {
            ok_or_jserr!(self.forward(private_key, next_ticket))
        }

        #[wasm_bindgen(js_name = "own_key")]
        pub fn _own_key(&self) -> Option<HalfKey> {
            match &self.state {
                PacketState::Forwarded { own_key, .. } => Some(own_key.clone()),
                PacketState::Outgoing { .. } | PacketState::Final { .. } => None,
            }
        }

        #[wasm_bindgen(js_name = "ack_challenge")]
        pub fn _ack_challenge(&self) -> Option<HalfKeyChallenge> {
            match &self.state {
                PacketState::Forwarded { ack_challenge, .. } | PacketState::Outgoing { ack_challenge, .. } => {
                    Some(ack_challenge.clone())
                }
                PacketState::Final { .. } => None,
            }
        }

        #[wasm_bindgen(js_name = "next_hop")]
        pub fn _next_hop(&self) -> Option<PublicKey> {
            match &self.state {
                PacketState::Forwarded { next_hop, .. } | PacketState::Outgoing { next_hop, .. } => {
                    Some(next_hop.clone())
                }
                PacketState::Final { .. } => None,
            }
        }

        #[wasm_bindgen(js_name = "previous_hop")]
        pub fn _previous_hop(&self) -> Option<PublicKey> {
            match &self.state {
                PacketState::Final { previous_hop, .. } | PacketState::Forwarded { previous_hop, .. } => {
                    Some(previous_hop.clone())
                }
                PacketState::Outgoing { .. } => None,
            }
        }

        #[wasm_bindgen(js_name = "packet_tag")]
        pub fn _packet_tag(&self) -> Option<Box<[u8]>> {
            match &self.state {
                PacketState::Final { packet_tag, .. } | PacketState::Forwarded { packet_tag, .. } => {
                    Some(packet_tag.clone())
                }
                PacketState::Outgoing { .. } => None,
            }
        }

        #[wasm_bindgen(js_name = "plaintext")]
        pub fn _plaintext(&self) -> Option<Box<[u8]>> {
            match &self.state {
                PacketState::Final { plain_text, .. } => Some(plain_text.clone()),
                PacketState::Forwarded { .. } | PacketState::Outgoing { .. } => None,
            }
        }

        #[wasm_bindgen(js_name = "state")]
        pub fn _state(&self) -> WasmPacketState {
            match &self.state {
                PacketState::Final { .. } => Final,
                PacketState::Forwarded { .. } => Forwarded,
                PacketState::Outgoing { .. } => Outgoing,
            }
        }

        pub fn size() -> u32 {
            Self::SIZE as u32
        }
    }
}
