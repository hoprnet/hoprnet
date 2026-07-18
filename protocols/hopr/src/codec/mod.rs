mod decoder;
mod encoder;

pub use decoder::HoprDecoder;
pub use encoder::{HoprEncoder, MAX_ACKNOWLEDGEMENTS_BATCH_SIZE};

/// Configuration of [`HoprEncoder`] and [`HoprDecoder`].
#[cfg_attr(feature = "serde", cfg_eval::cfg_eval, serde_with::serde_as)]
#[derive(Clone, Copy, Debug, smart_default::SmartDefault, validator::Validate)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields)
)]
pub struct HoprCodecConfig {
    /// Optional price of outgoing tickets.
    ///
    /// If not set (default), the network default will be used, which is the minimum allowed ticket price in the HOPR
    /// network.
    #[cfg_attr(
        feature = "serde",
        serde(default),
        serde_as(as = "Option<serde_with::DisplayFromStr>")
    )]
    pub outgoing_ticket_price: Option<hopr_api::types::primitive::balance::HoprBalance>,
    /// Optional minimum price of incoming tickets.
    ///
    /// The value cannot be lower than the default outgoing ticket price times the node's path position.
    ///
    /// If not set (default), the network default outgoing ticket price times the node's path position
    /// will be used.
    #[cfg_attr(
        feature = "serde",
        serde(default),
        serde_as(as = "Option<serde_with::DisplayFromStr>")
    )]
    pub min_incoming_ticket_price: Option<hopr_api::types::primitive::balance::HoprBalance>,
    /// Optional probability of winning an outgoing ticket.
    ///
    /// If not set (default), the network default will be used, which is the minimum allowed winning probability in the
    /// HOPR network.
    #[cfg_attr(
        feature = "serde",
        serde(default),
        serde_as(as = "Option<serde_with::DisplayFromStr>")
    )]
    pub outgoing_win_prob: Option<hopr_api::types::internal::prelude::WinningProbability>,
}

impl PartialEq for HoprCodecConfig {
    fn eq(&self, other: &Self) -> bool {
        self.outgoing_ticket_price.eq(&other.outgoing_ticket_price)
            && self.min_incoming_ticket_price.eq(&other.min_incoming_ticket_price)
            && match (self.outgoing_win_prob, other.outgoing_win_prob) {
                (Some(a), Some(b)) => a.approx_eq(&b),
                (None, None) => true,
                _ => false,
            }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use hopr_api::types::{
        crypto::prelude::*,
        crypto_random::Randomizable,
        internal::{prelude::*, routing::ResolvedTransportRouting},
    };
    use hopr_chain_connector::{
        HoprBlockchainSafeConnector,
        testing::{BlokliTestClient, StaticState},
    };
    use hopr_crypto_packet::HoprPixSpec;
    use hopr_protocol_pix::{EntryShareGenerator, ExitAcknowledgementShareProcessor, SsaId, SsaShareGenerator};
    use hopr_ticket_manager::{HoprTicketFactory, MemoryStore};

    use crate::{
        HoprCodecConfig, HoprDecoder, HoprEncoder, MemorySurbStore, OutgoingPacket, PacketDecoder, PacketEncoder,
        SurbStore, codec::encoder::MAX_ACKNOWLEDGEMENTS_BATCH_SIZE, utils::*,
    };

    type TestEncoder = HoprEncoder<
        Arc<HoprBlockchainSafeConnector<BlokliTestClient<StaticState>>>,
        Arc<SsaShareGenerator<HoprPixSpec>>,
        Arc<MemorySurbStore>,
        HoprTicketFactory<MemoryStore>,
    >;

    type TestDecoder = HoprDecoder<
        Arc<HoprBlockchainSafeConnector<BlokliTestClient<StaticState>>>,
        Arc<MemorySurbStore>,
        HoprTicketFactory<MemoryStore>,
    >;

    pub fn create_encoder(sender: &Node) -> TestEncoder {
        HoprEncoder::new(
            sender.chain_key.clone(),
            sender.chain_api.clone(),
            sender.surb_store.clone(),
            HoprTicketFactory::new(MemoryStore::default()),
            Hash::default(),
            sender.ssa_gen.clone(),
            HoprCodecConfig::default(),
        )
    }

    pub fn create_decoder(receiver: &Node) -> TestDecoder {
        HoprDecoder::new(
            (receiver.offchain_key.clone(), receiver.chain_key.clone()),
            receiver.chain_api.clone(),
            receiver.surb_store.clone(),
            HoprTicketFactory::new(MemoryStore::default()),
            Hash::default(),
            HoprCodecConfig::default(),
        )
    }

    /// Shared harness for PIX round-trip tests.
    ///
    /// Owns three nodes (sender, relay, receiver) with encoders, decoders,
    /// and pre-built forward/return paths through `sender → relay → receiver`.
    struct PixTestFixture {
        sender: Node,
        relay: Node,
        receiver: Node,
        sender_encoder: TestEncoder,
        sender_decoder: TestDecoder,
        relay_decoder: TestDecoder,
        receiver_encoder: TestEncoder,
        receiver_decoder: TestDecoder,
        forward_path: ValidatedPath,
        return_path: ValidatedPath,
    }

    impl PixTestFixture {
        async fn new() -> anyhow::Result<Self> {
            let blokli_client = create_blokli_client()?;
            let sender = create_node(0, &blokli_client).await?;
            let relay = create_node(1, &blokli_client).await?;
            let receiver = create_node(2, &blokli_client).await?;

            let sender_encoder = create_encoder(&sender);
            let sender_decoder = create_decoder(&sender);
            let relay_decoder = create_decoder(&relay);
            let receiver_encoder = create_encoder(&receiver);
            let receiver_decoder = create_decoder(&receiver);

            let forward_path = ValidatedPath::new(
                sender.chain_key.public().to_address(),
                vec![
                    relay.chain_key.public().to_address(),
                    receiver.chain_key.public().to_address(),
                ],
                &sender.chain_api.as_path_resolver(),
            )
            .await?;

            let return_path = ValidatedPath::new(
                receiver.chain_key.public().to_address(),
                vec![
                    relay.chain_key.public().to_address(),
                    sender.chain_key.public().to_address(),
                ],
                &sender.chain_api.as_path_resolver(),
            )
            .await?;

            Ok(Self {
                sender,
                relay,
                receiver,
                sender_encoder,
                sender_decoder,
                relay_decoder,
                receiver_encoder,
                receiver_decoder,
                forward_path,
                return_path,
            })
        }

        /// Looks up the SURB sent by `sender_pseudonym` and encodes a reply packet.
        fn surb_reply(
            &self,
            sender_pseudonym: HoprPseudonym,
            resp: impl AsRef<[u8]> + Send + 'static,
        ) -> anyhow::Result<OutgoingPacket> {
            let surb = self
                .receiver
                .surb_store
                .find_surb(SurbMatcher::Pseudonym(sender_pseudonym))
                .ok_or(anyhow::anyhow!("no surb found for pseudonym"))?;
            Ok(self.receiver_encoder.encode_packet(
                resp,
                ResolvedTransportRouting::Return(surb.sender_id, surb.surb),
                None,
            )?)
        }

        /// Decodes the reply that was relayed back to the sender and asserts its plaintext.
        fn sender_assert_reply(&self, relay_forwarded_data: bytes::Bytes, expected: &[u8]) -> anyhow::Result<()> {
            let in_packet = self
                .sender_decoder
                .decode(self.relay.offchain_key.public().into(), relay_forwarded_data)?;
            let in_packet = in_packet.try_as_final().ok_or(anyhow::anyhow!("packet is not final"))?;
            assert_eq!(expected, in_packet.plain_text.as_ref());
            Ok(())
        }
    }

    #[tokio::test]
    async fn encode_decode_packet() -> anyhow::Result<()> {
        let blokli_client = create_blokli_client()?;
        let sender = create_node(0, &blokli_client).await?;
        let receiver = create_node(1, &blokli_client).await?;

        let encoder = create_encoder(&sender);
        let decoder = create_decoder(&receiver);

        let data = b"some random message to encode and decode";

        let out_packet = encoder.encode_packet(
            data,
            ResolvedTransportRouting::Forward {
                pseudonym: HoprPseudonym::random(),
                forward_path: ValidatedPath::direct(
                    *receiver.offchain_key.public(),
                    receiver.chain_key.public().to_address(),
                ),
                return_paths: vec![],
            },
            None,
        )?;

        let in_packet = decoder.decode(sender.offchain_key.public().into(), out_packet.data)?;
        let in_packet = in_packet.try_as_final().ok_or(anyhow::anyhow!("packet is not final"))?;

        assert_eq!(data, in_packet.plain_text.as_ref());
        Ok(())
    }

    #[tokio::test]
    async fn encode_decode_packet_should_fail_for_too_long_messages() -> anyhow::Result<()> {
        let blokli_client = create_blokli_client()?;
        let sender = create_node(0, &blokli_client).await?;
        let receiver = create_node(1, &blokli_client).await?;

        let encoder = create_encoder(&sender);

        let data = hopr_api::types::crypto_random::random_bytes::<2048>();

        assert!(
            encoder
                .encode_packet(
                    data,
                    ResolvedTransportRouting::Forward {
                        pseudonym: HoprPseudonym::random(),
                        forward_path: ValidatedPath::direct(
                            *receiver.offchain_key.public(),
                            receiver.chain_key.public().to_address(),
                        ),
                        return_paths: vec![],
                    },
                    None,
                )
                .is_err()
        );

        Ok(())
    }

    #[tokio::test]
    async fn encode_decode_packet_on_relay() -> anyhow::Result<()> {
        let blokli_client = create_blokli_client()?;
        let sender = create_node(0, &blokli_client).await?;
        let relay = create_node(1, &blokli_client).await?;
        let receiver = create_node(2, &blokli_client).await?;

        let sender_encoder = create_encoder(&sender);
        let relay_decoder = create_decoder(&relay);
        let receiver_decoder = create_decoder(&receiver);

        let data = b"some random message to encode and decode";

        let out_packet = sender_encoder.encode_packet(
            data,
            ResolvedTransportRouting::Forward {
                pseudonym: HoprPseudonym::random(),
                forward_path: ValidatedPath::new(
                    sender.chain_key.public().to_address(),
                    vec![
                        relay.chain_key.public().to_address(),
                        receiver.chain_key.public().to_address(),
                    ],
                    &sender.chain_api.as_path_resolver(),
                )
                .await?,
                return_paths: vec![],
            },
            None,
        )?;

        let fwd_packet = relay_decoder.decode(sender.offchain_key.public().into(), out_packet.data)?;
        let fwd_packet = fwd_packet
            .try_as_forwarded()
            .ok_or(anyhow::anyhow!("packet is not forwarded"))?;

        let in_packet = receiver_decoder.decode(relay.offchain_key.public().into(), fwd_packet.data)?;
        let in_packet = in_packet.try_as_final().ok_or(anyhow::anyhow!("packet is not final"))?;

        assert_eq!(data, in_packet.plain_text.as_ref());
        Ok(())
    }

    #[tokio::test]
    async fn encode_decode_packet_full_round_trip() -> anyhow::Result<()> {
        let f = PixTestFixture::new().await?;
        let data = b"some random message to encode and decode";
        let resp = b"some random response to encode and decode";

        let pseudonym = HoprPseudonym::random();
        let out_packet = f.sender_encoder.encode_packet(
            data,
            ResolvedTransportRouting::Forward {
                pseudonym,
                forward_path: f.forward_path.clone(),
                return_paths: vec![f.return_path.clone()],
            },
            None,
        )?;

        let fwd_packet = f
            .relay_decoder
            .decode(f.sender.offchain_key.public().into(), out_packet.data)?;
        let fwd_packet = fwd_packet
            .try_as_forwarded()
            .ok_or(anyhow::anyhow!("packet is not forwarded"))?;

        let in_packet = f
            .receiver_decoder
            .decode(f.relay.offchain_key.public().into(), fwd_packet.data)?;
        let in_packet = in_packet.try_as_final().ok_or(anyhow::anyhow!("packet is not final"))?;
        assert_eq!(data, in_packet.plain_text.as_ref());

        let out_packet = f.surb_reply(pseudonym, resp)?;

        let fwd_packet = f
            .relay_decoder
            .decode(f.receiver.offchain_key.public().into(), out_packet.data)?;
        let fwd_packet = fwd_packet
            .try_as_forwarded()
            .ok_or(anyhow::anyhow!("packet is not forwarded"))?;

        f.sender_assert_reply(fwd_packet.data, resp)?;

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn encode_decode_packet_full_round_trip_with_pix() -> anyhow::Result<()> {
        let f = PixTestFixture::new().await?;

        let pseudonym = HoprPseudonym::random();
        let ssa_id = SsaId::new(pseudonym, 1.try_into()?);

        // New commitment of the receiver
        let _ = f.receiver.ssa_rcn.new_exit_commitment(
            ssa_id,
            f.sender.ssa_gen.config().polynomials_per_ssa as usize,
            f.sender.ssa_gen.config().threshold as usize,
        )?;

        // Sender makes a commitment too and delivers it to the receiver
        let sender_commitment = f.sender.ssa_gen.new_ssa_commitment(&pseudonym, ssa_id.ssa_index())?;
        sender_commitment.process_into_reconstructor(&f.receiver.ssa_rcn)?;

        // Need to generate multiple messages to recover the SSA
        let threshold = f.sender.ssa_gen.config().threshold as usize;
        let num_polys = f.sender.ssa_gen.config().polynomials_per_ssa as usize;
        let surplus = f.sender.ssa_gen.config().surplus_shares;
        let recovery_at = (num_polys - 1) * (threshold + surplus) + (threshold - 1);

        let num_msgs_to_recover_ssa = num_polys * (threshold + surplus);
        for i in 0..num_msgs_to_recover_ssa {
            let data = format!("some random message #{i} to encode and decode");
            let resp = format!("some random response #{i} to encode and decode");

            // Sender creates a packet
            let out_packet = f.sender_encoder.encode_packet(
                data.clone(),
                ResolvedTransportRouting::Forward {
                    pseudonym,
                    forward_path: f.forward_path.clone(),
                    return_paths: vec![f.return_path.clone()],
                },
                None,
            )?;

            // Relay decodes the packet
            let fwd_packet = f
                .relay_decoder
                .decode(f.sender.offchain_key.public().into(), out_packet.data)?;
            let fwd_packet = fwd_packet
                .try_as_forwarded()
                .ok_or(anyhow::anyhow!("packet is not forwarded"))?;

            // Receiver receives the packet from the relay and decodes it
            let in_packet = f
                .receiver_decoder
                .decode(f.relay.offchain_key.public().into(), fwd_packet.data)?;
            let in_packet = in_packet.try_as_final().ok_or(anyhow::anyhow!("packet is not final"))?;

            assert_eq!(pseudonym, in_packet.sender);
            assert_eq!(data.as_bytes(), in_packet.plain_text.as_ref());

            // Receiver creates a response packet using a SURB
            let out_packet = f.surb_reply(pseudonym, resp.clone())?;

            // Receiver discovers the encrypted share from the used SURB
            let enc_share = out_packet
                .encrypted_pix_share
                .ok_or(anyhow::anyhow!("no pix share found"))?;
            assert_eq!(pseudonym, enc_share.pseudonym);

            // The receiver inserts the encrypted share to be decoded by the acknowledgement from the relay
            f.receiver.ssa_rcn.insert_encrypted_share(
                f.relay.offchain_key.public(),
                out_packet.ack_challenge,
                enc_share,
            )?;

            let fwd_packet = f
                .relay_decoder
                .decode(f.receiver.offchain_key.public().into(), out_packet.data)?;
            let fwd_packet = fwd_packet
                .try_as_forwarded()
                .ok_or(anyhow::anyhow!("packet is not forwarded"))?;

            // Relay delivers the acknowledgement to back to the receiver
            let resolutions = f.receiver.ssa_rcn.acknowledge_shares(
                *f.relay.offchain_key.public(),
                vec![VerifiedAcknowledgement::new(fwd_packet.ack_key_prev_hop, &f.relay.offchain_key).leak()],
            )?;

            // Once enough shares have been received, the receiver can recover the SSA
            if i == recovery_at {
                assert_eq!(1, resolutions.len());
                let recovered_ssa = resolutions[0]
                    .clone()
                    .try_as_recovered_ssa()
                    .ok_or(anyhow::anyhow!("share resolution is not recovered ssa"))?;
                assert_eq!(recovered_ssa.ssa_id, ssa_id);
            } else {
                // Other resolution possibilities should not happen at this point
                assert!(
                    resolutions.is_empty(),
                    "no resolutions must be present at #{i}: {resolutions:?}"
                );
            }

            // Sender decodes the response packet
            f.sender_assert_reply(fwd_packet.data, resp.as_bytes())?;
        }

        Ok(())
    }

    #[test_log::test(tokio::test)]
    async fn encode_decode_packet_full_round_trip_with_pix_multi_commitment() -> anyhow::Result<()> {
        let f = PixTestFixture::new().await?;

        const NUM_COMMITMENTS: usize = 2;
        const GAP_MSGS: usize = 10;

        let pseudonym = HoprPseudonym::random();

        let threshold = f.sender.ssa_gen.config().threshold as usize;
        let num_polys = f.sender.ssa_gen.config().polynomials_per_ssa as usize;
        let surplus = f.sender.ssa_gen.config().surplus_shares;
        let shares_per_poly = threshold + surplus;
        let msgs_per_commitment = num_polys * shares_per_poly;

        // Create SSA IDs for all commitments upfront
        let ssa_ids: Vec<SsaId<HoprPseudonym>> = (0..NUM_COMMITMENTS)
            .map(|k| SsaId::new(pseudonym, ((k + 1) as u32).try_into().unwrap()))
            .collect();

        // --- First commitment setup (before the loop) ---
        let _ = f
            .receiver
            .ssa_rcn
            .new_exit_commitment(ssa_ids[0], num_polys, threshold)?;
        let sender_commitment = f
            .sender
            .ssa_gen
            .new_ssa_commitment(&pseudonym, ssa_ids[0].ssa_index())?;
        sender_commitment.process_into_reconstructor(&f.receiver.ssa_rcn)?;

        let segment_len = msgs_per_commitment + GAP_MSGS;

        let mut recovered_ssa_ids: Vec<SsaId<HoprPseudonym>> = Vec::new();
        let mut no_share_indices: Vec<usize> = Vec::new();

        let total_msgs = NUM_COMMITMENTS * msgs_per_commitment + (NUM_COMMITMENTS - 1) * GAP_MSGS;

        for i in 0..total_msgs {
            let data = format!("some random message #{i} to encode and decode");
            let resp = format!("some random response #{i} to encode and decode");

            // 1. Forward: sender encodes with SURB
            let out_packet = f.sender_encoder.encode_packet(
                data.clone(),
                ResolvedTransportRouting::Forward {
                    pseudonym,
                    forward_path: f.forward_path.clone(),
                    return_paths: vec![f.return_path.clone()],
                },
                None,
            )?;

            // 2. Relay decodes forward packet
            let fwd_packet = f
                .relay_decoder
                .decode(f.sender.offchain_key.public().into(), out_packet.data)?;
            let fwd_packet = fwd_packet
                .try_as_forwarded()
                .ok_or(anyhow::anyhow!("packet is not forwarded"))?;

            // 3. Receiver decodes forward packet
            let in_packet = f
                .receiver_decoder
                .decode(f.relay.offchain_key.public().into(), fwd_packet.data)?;
            let in_packet = in_packet.try_as_final().ok_or(anyhow::anyhow!("packet is not final"))?;
            assert_eq!(pseudonym, in_packet.sender);
            assert_eq!(data.as_bytes(), in_packet.plain_text.as_ref());

            // 4. Receiver finds SURB and encodes reply
            let out_packet = f.surb_reply(pseudonym, resp.clone())?;

            // 5. Relay decodes the reply (always)
            let fwd_packet = f
                .relay_decoder
                .decode(f.receiver.offchain_key.public().into(), out_packet.data)?;
            let fwd_packet = fwd_packet
                .try_as_forwarded()
                .ok_or(anyhow::anyhow!("packet is not forwarded"))?;

            let pos_in_segment = i % segment_len;

            // 6. Determine if we are in a share phase or a gap
            if pos_in_segment < msgs_per_commitment {
                // Share phase — PIX share should exist
                let enc_share = out_packet
                    .encrypted_pix_share
                    .ok_or(anyhow::anyhow!("expected pix share at msg #{i}"))?;
                assert_eq!(pseudonym, enc_share.pseudonym);

                f.receiver.ssa_rcn.insert_encrypted_share(
                    f.relay.offchain_key.public(),
                    out_packet.ack_challenge,
                    enc_share,
                )?;

                let resolutions = f.receiver.ssa_rcn.acknowledge_shares(
                    *f.relay.offchain_key.public(),
                    vec![VerifiedAcknowledgement::new(fwd_packet.ack_key_prev_hop, &f.relay.offchain_key).leak()],
                )?;

                for resolution in resolutions {
                    let recovered = resolution
                        .try_as_recovered_ssa()
                        .ok_or(anyhow::anyhow!("share resolution is not recovered ssa at #{i}"))?;
                    recovered_ssa_ids.push(recovered.ssa_id);
                }
            } else {
                // Gap phase — no PIX share
                assert!(
                    out_packet.encrypted_pix_share.is_none(),
                    "unexpected PIX share during gap at message #{i}"
                );
                no_share_indices.push(i);

                // At the last message of this gap, set up the next commitment
                if pos_in_segment == segment_len - 1 {
                    let next_idx = i / segment_len + 1;
                    let _ = f
                        .receiver
                        .ssa_rcn
                        .new_exit_commitment(ssa_ids[next_idx], num_polys, threshold)?;
                    let next_commitment = f
                        .sender
                        .ssa_gen
                        .new_ssa_commitment(&pseudonym, ssa_ids[next_idx].ssa_index())?;
                    next_commitment.process_into_reconstructor(&f.receiver.ssa_rcn)?;
                }
            }

            // 7. Sender decodes the response (always)
            f.sender_assert_reply(fwd_packet.data, resp.as_bytes())?;
        }

        // --- Final verification ---

        // 1. Verify the number of recovered secrets equals the number of commitments made
        assert_eq!(
            recovered_ssa_ids.len(),
            NUM_COMMITMENTS,
            "expected {NUM_COMMITMENTS} RecoveredSsa (1 per commitment), got {}",
            recovered_ssa_ids.len()
        );

        // 2. Verify each recovered SSA corresponds to the correct commitment
        recovered_ssa_ids.sort_by_key(|id| id.ssa_index());
        for (k, ssa_id) in ssa_ids.iter().enumerate() {
            assert_eq!(
                recovered_ssa_ids[k], *ssa_id,
                "recovered SSA #{k} should match commitment {k}"
            );
        }

        // 3. Verify that no_share_indices covers exactly all gap periods
        let expected_no_share_indices: Vec<usize> = (0..NUM_COMMITMENTS - 1)
            .flat_map(|k| {
                let gap_start = (k + 1) * msgs_per_commitment + k * GAP_MSGS;
                gap_start..gap_start + GAP_MSGS
            })
            .collect();
        assert_eq!(
            no_share_indices, expected_no_share_indices,
            "no-share indices must match all gap periods"
        );

        Ok(())
    }

    #[tokio::test]
    async fn encode_decode_acknowledgements() -> anyhow::Result<()> {
        let blokli_client = create_blokli_client()?;
        let sender = create_node(0, &blokli_client).await?;
        let receiver = create_node(1, &blokli_client).await?;

        let encoder = create_encoder(&sender);
        let decoder = create_decoder(&receiver);

        let acks = (0..MAX_ACKNOWLEDGEMENTS_BATCH_SIZE)
            .map(|_| VerifiedAcknowledgement::random(&PEERS[0].1))
            .collect::<Vec<_>>();
        let out_packet = encoder.encode_acknowledgements(&acks, PEERS[1].1.public())?;

        let in_packet = decoder.decode(sender.offchain_key.public().into(), out_packet.data)?;
        let in_packet = in_packet
            .try_as_acknowledgement()
            .ok_or(anyhow::anyhow!("packet is not acknowledgement"))?;

        assert_eq!(acks.len(), in_packet.received_acks.len());

        for (i, ack) in in_packet.received_acks.into_iter().enumerate() {
            let verified = ack.verify(PEERS[0].1.public())?;
            assert_eq!(acks[i], verified);
        }

        Ok(())
    }

    #[tokio::test]
    async fn encode_should_fail_on_too_many_acknowledgements() -> anyhow::Result<()> {
        let blokli_client = create_blokli_client()?;
        let sender = create_node(0, &blokli_client).await?;

        let encoder = create_encoder(&sender);
        let acks = (0..MAX_ACKNOWLEDGEMENTS_BATCH_SIZE + 1)
            .map(|_| VerifiedAcknowledgement::random(&PEERS[0].1))
            .collect::<Vec<_>>();

        assert!(encoder.encode_acknowledgements(&acks, PEERS[1].1.public()).is_err());

        Ok(())
    }

    #[tokio::test]
    async fn decode_should_fail_on_invalid_data() -> anyhow::Result<()> {
        let blokli_client = create_blokli_client()?;
        let receiver = create_node(0, &blokli_client).await?;
        let decoder = create_decoder(&receiver);

        let sender_peer_id = PEERS[1].1.public().into();
        let invalid_data = bytes::Bytes::from(vec![0u8; 100]);

        let result = decoder.decode(sender_peer_id, invalid_data);
        assert!(result.is_err());
        assert!(result.unwrap_err().is_undecodable());

        Ok(())
    }

    #[tokio::test]
    async fn decode_should_fail_on_replay() -> anyhow::Result<()> {
        let blokli_client = create_blokli_client()?;
        let sender = create_node(0, &blokli_client).await?;
        let receiver = create_node(1, &blokli_client).await?;

        let encoder = create_encoder(&sender);
        let decoder = create_decoder(&receiver);

        let data = b"some message";
        let out_packet = encoder.encode_packet(
            data,
            ResolvedTransportRouting::Forward {
                pseudonym: HoprPseudonym::random(),
                forward_path: ValidatedPath::direct(
                    *receiver.offchain_key.public(),
                    receiver.chain_key.public().to_address(),
                ),
                return_paths: vec![],
            },
            None,
        )?;

        // First decode should succeed
        decoder.decode(sender.offchain_key.public().into(), out_packet.data.clone())?;

        // Second decode with same packet should fail (replay)
        let result = decoder.decode(sender.offchain_key.public().into(), out_packet.data);
        assert!(result.is_err());
        assert!(result.unwrap_err().is_processing_error());

        Ok(())
    }

    #[tokio::test]
    async fn decode_should_fail_on_incorrect_key() -> anyhow::Result<()> {
        let blokli_client = create_blokli_client()?;
        let sender = create_node(0, &blokli_client).await?;
        let receiver = create_node(1, &blokli_client).await?;
        let incorrect_receiver = create_node(2, &blokli_client).await?;

        let encoder = create_encoder(&sender);
        let decoder = create_decoder(&incorrect_receiver);

        let data = b"some message";
        let out_packet = encoder.encode_packet(
            data,
            ResolvedTransportRouting::Forward {
                pseudonym: HoprPseudonym::random(),
                forward_path: ValidatedPath::direct(
                    *receiver.offchain_key.public(),
                    receiver.chain_key.public().to_address(),
                ),
                return_paths: vec![],
            },
            None,
        )?;

        let result = decoder.decode(sender.offchain_key.public().into(), out_packet.data);
        assert!(result.is_err());
        assert!(result.unwrap_err().is_undecodable());

        Ok(())
    }
}
