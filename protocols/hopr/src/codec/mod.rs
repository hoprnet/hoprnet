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
    pub outgoing_ticket_price: Option<hopr_primitive_types::balance::HoprBalance>,
    /// Optional probability of winning an outgoing ticket.
    ///
    /// If not set (default), the network default will be used, which is the minimum allowed winning probability in the
    /// HOPR network.
    #[cfg_attr(
        feature = "serde",
        serde(default),
        serde_as(as = "Option<serde_with::DisplayFromStr>")
    )]
    pub outgoing_win_prob: Option<hopr_internal_types::prelude::WinningProbability>,
}

impl PartialEq for HoprCodecConfig {
    fn eq(&self, other: &Self) -> bool {
        self.outgoing_ticket_price.eq(&other.outgoing_ticket_price)
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

    use hopr_chain_connector::{
        HoprBlockchainSafeConnector,
        testing::{BlokliTestClient, StaticState},
    };
    use hopr_crypto_random::Randomizable;
    use hopr_crypto_types::prelude::*;
    use hopr_db_node::HoprNodeDb;
    use hopr_internal_types::prelude::*;
    use hopr_network_types::prelude::ResolvedTransportRouting;

    use crate::{
        HoprCodecConfig, HoprDecoder, HoprEncoder, HoprTicketProcessor, HoprTicketProcessorConfig, MemorySurbStore,
        PacketDecoder, PacketEncoder, SurbStoreConfig, codec::encoder::MAX_ACKNOWLEDGEMENTS_BATCH_SIZE, utils::*,
    };

    type TestEncoder = HoprEncoder<
        Arc<HoprBlockchainSafeConnector<BlokliTestClient<StaticState>>>,
        MemorySurbStore,
        HoprTicketProcessor<Arc<HoprBlockchainSafeConnector<BlokliTestClient<StaticState>>>, HoprNodeDb>,
    >;

    type TestDecoder = HoprDecoder<
        Arc<HoprBlockchainSafeConnector<BlokliTestClient<StaticState>>>,
        MemorySurbStore,
        HoprTicketProcessor<Arc<HoprBlockchainSafeConnector<BlokliTestClient<StaticState>>>, HoprNodeDb>,
    >;

    pub fn create_encoder(sender: &Node) -> TestEncoder {
        HoprEncoder::new(
            sender.chain_key.clone(),
            sender.chain_api.clone(),
            MemorySurbStore::new(SurbStoreConfig::default()),
            HoprTicketProcessor::new(
                sender.chain_api.clone(),
                sender.node_db.clone(),
                sender.chain_key.clone(),
                Hash::default(),
                HoprTicketProcessorConfig::default(),
            ),
            Hash::default(),
            HoprCodecConfig::default(),
        )
    }

    pub fn create_decoder(receiver: &Node) -> TestDecoder {
        HoprDecoder::new(
            (receiver.offchain_key.clone(), receiver.chain_key.clone()),
            receiver.chain_api.clone(),
            MemorySurbStore::new(SurbStoreConfig::default()),
            HoprTicketProcessor::new(
                receiver.chain_api.clone(),
                receiver.node_db.clone(),
                receiver.chain_key.clone(),
                Hash::default(),
                HoprTicketProcessorConfig::default(),
            ),
            Hash::default(),
            HoprCodecConfig::default(),
        )
    }

    #[tokio::test]
    async fn encode_decode_packet() -> anyhow::Result<()> {
        let blokli_client = create_blokli_client()?;
        let sender = create_node(0, &blokli_client).await?;
        let receiver = create_node(1, &blokli_client).await?;

        let encoder = create_encoder(&sender);
        let decoder = create_decoder(&receiver);

        let data = b"some random message to encode and decode";

        let out_packet = encoder
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
            .await?;

        let in_packet = decoder
            .decode(sender.offchain_key.public().into(), out_packet.data)
            .await?;
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

        let data = hopr_crypto_random::random_bytes::<2048>();

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
                .await
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

        let out_packet = sender_encoder
            .encode_packet(
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
            )
            .await?;

        let fwd_packet = relay_decoder
            .decode(sender.offchain_key.public().into(), out_packet.data)
            .await?;
        let fwd_packet = fwd_packet
            .try_as_forwarded()
            .ok_or(anyhow::anyhow!("packet is not forwarded"))?;

        let in_packet = receiver_decoder
            .decode(relay.offchain_key.public().into(), fwd_packet.data)
            .await?;
        let in_packet = in_packet.try_as_final().ok_or(anyhow::anyhow!("packet is not final"))?;

        assert_eq!(data, in_packet.plain_text.as_ref());
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
        let out_packet = encoder.encode_acknowledgements(&acks, PEERS[1].1.public()).await?;

        let in_packet = decoder
            .decode(sender.offchain_key.public().into(), out_packet.data)
            .await?;
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

        assert!(
            encoder
                .encode_acknowledgements(&acks, PEERS[1].1.public())
                .await
                .is_err()
        );

        Ok(())
    }
}
